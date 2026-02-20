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
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules use conventional use-before-mod ordering"
)]
mod tests {
    use std::path::{Path, PathBuf};

    use rstest::rstest;
    use tempfile::TempDir;

    use super::*;

    mod write_operations {
        use super::*;

        #[test]
        fn write_file_creates_and_overwrites() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());

            writer.write_file(Path::new("file.txt"), b"first").expect("write");
            writer
                .write_file(Path::new("file.txt"), b"second")
                .expect("overwrite");

            let content =
                std::fs::read(dir.path().join("file.txt")).expect("read");
            assert_eq!(content, b"second");
        }

        #[rstest]
        #[case::traversal("../escape.txt")]
        #[case::hidden(".secret")]
        fn write_file_rejects_invalid_paths(#[case] path: &str) {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(writer.write_file(Path::new(path), b"data").is_err());
        }

        #[test]
        fn write_file_returns_error_when_parent_missing() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(
                writer
                    .write_file(Path::new("nested/deep/file.txt"), b"data")
                    .is_err()
            );
        }
    }

    mod atomic_write {
        use super::*;

        #[test]
        fn writes_content_correctly() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());

            writer
                .atomic_write(Path::new("file.txt"), b"content")
                .expect("write");

            let content =
                std::fs::read(dir.path().join("file.txt")).expect("read");
            assert_eq!(content, b"content");
        }

        #[test]
        fn no_orphaned_temp_file() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            writer.atomic_write(Path::new("file.txt"), b"data").expect("write");

            let count =
                std::fs::read_dir(dir.path()).expect("read dir").count();
            assert_eq!(count, 1);
        }

        #[test]
        fn overwrites_existing_file() {
            let dir = TempDir::new().expect("tempdir");
            std::fs::write(dir.path().join("file.txt"), b"original")
                .expect("write");
            let writer = Writer::new(dir.path());

            writer
                .atomic_write(Path::new("file.txt"), b"replaced")
                .expect("write");

            let content =
                std::fs::read(dir.path().join("file.txt")).expect("read");
            assert_eq!(content, b"replaced");
        }

        #[rstest]
        #[case::traversal("../escape.txt")]
        #[case::hidden(".secret")]
        fn rejects_invalid_paths(#[case] path: &str) {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(writer.atomic_write(Path::new(path), b"data").is_err());
        }

        #[test]
        fn returns_error_when_parent_missing() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(
                writer
                    .atomic_write(Path::new("nested/file.txt"), b"data")
                    .is_err()
            );
        }

        #[test]
        fn creates_in_existing_subdirectory() {
            let dir = TempDir::new().expect("tempdir");
            std::fs::create_dir_all(dir.path().join("sub")).expect("create");
            let writer = Writer::new(dir.path());

            writer
                .atomic_write(Path::new("sub/file.txt"), b"data")
                .expect("write");
            assert!(dir.path().join("sub/file.txt").exists());
        }
    }

    mod create_dir_all {
        use super::*;

        #[test]
        fn creates_nested_directories() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            writer.create_dir_all(Path::new("a/b/c")).expect("create");
            assert!(dir.path().join("a/b/c").exists());
        }

        #[test]
        fn idempotent_for_existing() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            writer.create_dir_all(Path::new("dir")).expect("create");
            writer.create_dir_all(Path::new("dir")).expect("create again");
        }

        #[test]
        fn rejects_traversal() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(writer.create_dir_all(Path::new("../escape")).is_err());
        }
    }

    mod rename {
        use super::*;

        #[test]
        fn moves_file() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "old.txt", b"data");
            let writer = Writer::new(dir.path());

            writer
                .rename(Path::new("old.txt"), Path::new("new.txt"))
                .expect("rename");

            assert!(!dir.path().join("old.txt").exists());
            assert!(dir.path().join("new.txt").exists());
        }

        #[rstest]
        #[case::source_traversal("../escape", "safe.txt")]
        #[case::dest_traversal("safe.txt", "../escape")]
        #[case::source_hidden(".secret", "public.txt")]
        #[case::dest_hidden("public.txt", ".secret")]
        fn rejects_invalid_paths(#[case] from: &str, #[case] to: &str) {
            let dir = TempDir::new().expect("tempdir");
            if !from.starts_with('.') && !from.starts_with('.') {
                write_file(dir.path(), from, b"data");
            }
            let writer = Writer::new(dir.path());
            assert!(writer.rename(Path::new(from), Path::new(to)).is_err());
        }
    }

    mod remove_file {
        use super::*;

        #[test]
        fn deletes_file() {
            let dir = TempDir::new().expect("tempdir");
            write_file(dir.path(), "file.txt", b"data");
            let writer = Writer::new(dir.path());

            writer.remove_file(Path::new("file.txt")).expect("remove");
            assert!(!dir.path().join("file.txt").exists());
        }

        #[test]
        fn returns_error_for_nonexistent() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(writer.remove_file(Path::new("nonexistent.txt")).is_err());
        }

        #[rstest]
        #[case::traversal("../escape.txt")]
        #[case::hidden(".secret")]
        fn rejects_invalid_paths(#[case] path: &str) {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            assert!(writer.remove_file(Path::new(path)).is_err());
        }
    }

    fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, contents).expect("write test file");
        path
    }
}
