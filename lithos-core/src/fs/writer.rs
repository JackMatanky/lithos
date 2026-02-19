//! Filesystem writer utilities for safe writes.
//!
//! The writer keeps all paths scoped to a root directory and validates inputs
//! before touching the filesystem. This preserves adapter safety guarantees
//! while providing atomic replace semantics for file updates.

use std::{
    io,
    path::{Path, PathBuf},
};

use super::validator::Validator;

/// Production filesystem writer using `std::fs`.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use lithos_core::fs::writer::Writer;
/// # let unique = format!("lithos_fs_writer_example_{}", std::process::id());
/// # let root = std::env::temp_dir().join(unique);
/// # std::fs::create_dir_all(&root)?;
/// let writer = Writer::new(root.as_path());
/// writer.write_file(Path::new("output.txt"), b"hello")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct Writer {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl Writer {
    /// Creates a new filesystem writer scoped to the given root directory.
    #[inline]
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    #[inline]
    fn resolve(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    #[inline]
    fn validate_path(path: &Path) -> io::Result<()> {
        Validator::new_flexible()
            .validate(path)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
    }

    /// Creates all directories in the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if directories cannot be created.
    #[inline]
    pub fn create_dir_all(&self, path: &Path) -> Result<(), io::Error> {
        Self::validate_path(path)?;
        std::fs::create_dir_all(self.resolve(path))
    }

    /// Writes bytes to a file, creating or truncating it.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    #[inline]
    pub fn write_file(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), io::Error> {
        Self::validate_path(path)?;
        std::fs::write(self.resolve(path), contents)
    }

    /// Writes bytes to a file using an atomic replace strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or renamed.
    #[inline]
    pub fn atomic_write(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), io::Error> {
        use std::io::Write as _;

        Self::validate_path(path)?;
        let target = self.resolve(path);
        let parent = target.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no parent")
        })?;

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let file_name =
            target.file_name().and_then(|name| name.to_str()).unwrap_or("file");
        let tmp_path = parent.join(format!(".{file_name}.{suffix}.tmp"));

        let mut temp_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;
        temp_file.write_all(contents)?;
        temp_file.sync_all()?;
        drop(temp_file);

        std::fs::rename(&tmp_path, &target)
    }

    /// Renames a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the rename fails.
    #[inline]
    pub fn rename(&self, from: &Path, to: &Path) -> Result<(), io::Error> {
        Self::validate_path(from)?;
        Self::validate_path(to)?;
        std::fs::rename(self.resolve(from), self.resolve(to))
    }

    /// Removes a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed.
    #[inline]
    pub fn remove_file(&self, path: &Path) -> Result<(), io::Error> {
        Self::validate_path(path)?;
        std::fs::remove_file(self.resolve(path))
    }
}
