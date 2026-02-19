//! Filesystem writer utilities for safe writes.
//!
//! The writer keeps all paths scoped to a root directory and validates inputs
//! before touching the filesystem. This preserves adapter safety guarantees
//! while providing atomic replace semantics for file updates.

use std::{
    io,
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use super::validator::Validator;

/// Production filesystem writer using `std::fs`.
///
/// All paths are validated against traversal and hidden-file attacks before
/// filesystem operations. The writer uses atomic replace semantics for file
/// updates via [`Writer::atomic_write`].
#[derive(Debug, Clone)]
pub(crate) struct Writer {
    /// Root directory for scoped file access.
    root: PathBuf,
    /// Path validator for security checks.
    validator: Validator,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pub(crate) API for future callers; will be used when \
                  template module adapter is implemented."
    )
)]
impl Writer {
    /// Creates a new filesystem writer with flexible path validation.
    #[inline]
    #[must_use]
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            validator: Validator::new_flexible(),
        }
    }

    #[inline]
    fn resolve(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    #[inline]
    fn validate_path(&self, path: &Path) -> io::Result<()> {
        self.validator
            .validate(path)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
    }

    /// Creates all directories in the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if directories cannot be created or the path is
    /// invalid.
    #[inline]
    pub(crate) fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.validate_path(path)?;
        std::fs::create_dir_all(self.resolve(path))
    }

    /// Writes bytes to a file, creating or truncating it.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or the path is invalid.
    #[inline]
    pub(crate) fn write_file(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> io::Result<()> {
        self.validate_path(path)?;
        std::fs::write(self.resolve(path), contents)
    }

    /// Writes bytes to a file using an atomic replace strategy.
    ///
    /// Uses `NamedTempFile::new_in` to create a temp file with a
    /// cryptographically unique name in the target's parent directory. The
    /// temp file is automatically cleaned up on drop if `persist()` is
    /// never called, preventing orphaned temp files on panic or early
    /// return.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written, synced, or renamed.
    #[inline]
    pub(crate) fn atomic_write(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> io::Result<()> {
        use std::io::Write as _;

        self.validate_path(path)?;
        let target = self.resolve(path);
        let parent = target.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no parent")
        })?;

        // NamedTempFile::new_in uses O_CREAT|O_EXCL with a cryptographically
        // unique name. Its Drop impl deletes the temp file if persist() is
        // never called, preventing orphaned temp files on panic or early
        // return.
        let mut temp = NamedTempFile::new_in(parent)?;
        temp.write_all(contents)?;
        temp.as_file().sync_all()?; // durability: flush data before rename
        temp.persist(&target).map_err(|e| e.error)?;
        Ok(())
    }

    /// Renames a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the rename fails or either path is invalid.
    #[inline]
    pub(crate) fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.validate_path(from)?;
        self.validate_path(to)?;
        std::fs::rename(self.resolve(from), self.resolve(to))
    }

    /// Removes a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed or the path is invalid.
    #[inline]
    pub(crate) fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.validate_path(path)?;
        std::fs::remove_file(self.resolve(path))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::*;

    fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, contents).expect("write test file");
        path
    }

    #[test]
    fn write_file_creates_file_with_correct_content() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        writer
            .write_file(Path::new("output.txt"), b"hello world")
            .expect("write file");

        let content =
            std::fs::read(dir.path().join("output.txt")).expect("read file");
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn write_file_overwrites_existing_file() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        writer
            .write_file(Path::new("output.txt"), b"original")
            .expect("write file");
        writer
            .write_file(Path::new("output.txt"), b"replaced")
            .expect("overwrite file");

        let content =
            std::fs::read(dir.path().join("output.txt")).expect("read file");
        assert_eq!(content, b"replaced");
    }

    #[test]
    fn write_file_rejects_path_traversal() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        let result = writer.write_file(Path::new("../escape.txt"), b"data");
        assert!(result.is_err(), "should reject path traversal");
    }

    #[test]
    fn write_file_rejects_hidden_file() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        let result = writer.write_file(Path::new(".secret"), b"data");
        assert!(result.is_err(), "should reject hidden file");
    }

    #[test]
    fn atomic_write_content_is_correct() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        writer
            .atomic_write(Path::new("atomic.txt"), b"atomic content")
            .expect("atomic write");

        let content =
            std::fs::read(dir.path().join("atomic.txt")).expect("read file");
        assert_eq!(content, b"atomic content");
    }

    #[test]
    fn atomic_write_no_orphaned_temp_file() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        writer
            .atomic_write(Path::new("atomic.txt"), b"content")
            .expect("atomic write");

        // Count files in the directory - should only have the target file
        let file_count =
            std::fs::read_dir(dir.path()).expect("read dir").count();
        assert_eq!(
            file_count, 1,
            "should have exactly one file (no temp orphan)"
        );
    }

    #[test]
    fn atomic_write_rejects_path_traversal() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        let result = writer.atomic_write(Path::new("../escape.txt"), b"data");
        assert!(result.is_err(), "should reject path traversal");
    }

    #[test]
    fn create_dir_all_creates_nested_directories() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        writer.create_dir_all(Path::new("a/b/c/d")).expect("create dirs");

        assert!(dir.path().join("a/b/c/d").exists());
    }

    #[test]
    fn rename_moves_file() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "old.txt", b"data");
        let writer = Writer::new(dir.path());

        writer
            .rename(Path::new("old.txt"), Path::new("new.txt"))
            .expect("rename");

        assert!(!dir.path().join("old.txt").exists());
        assert!(dir.path().join("new.txt").exists());
    }

    #[test]
    fn remove_file_deletes_file() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "to_delete.txt", b"data");
        let writer = Writer::new(dir.path());

        writer.remove_file(Path::new("to_delete.txt")).expect("remove file");

        assert!(!dir.path().join("to_delete.txt").exists());
    }

    #[test]
    fn remove_file_returns_error_for_non_existent() {
        let dir = TempDir::new().expect("tempdir");
        let writer = Writer::new(dir.path());

        let result = writer.remove_file(Path::new("nonexistent.txt"));
        assert!(result.is_err(), "should error for non-existent file");
    }
}
