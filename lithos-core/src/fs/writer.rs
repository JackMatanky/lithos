//! Filesystem writer utilities for safe writes.

use std::{
    io,
    path::{Path, PathBuf},
};

use super::validator::Validator;

/// Abstraction over filesystem write operations.
pub trait FsWriter: Send + Sync {
    /// Error type for file operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Writes bytes to a file using an atomic replace strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or renamed.
    fn atomic_write(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), Self::Error>;

    /// Creates all directories in the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if directories cannot be created.
    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error>;

    /// Removes a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed.
    fn remove_file(&self, path: &Path) -> Result<(), Self::Error>;

    /// Renames a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the rename fails.
    fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error>;

    /// Writes bytes to a file, creating or truncating it.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    fn write_file(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), Self::Error>;
}

/// Production filesystem writer using `std::fs`.
#[derive(Debug, Clone)]
pub struct OsFsWriter {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl OsFsWriter {
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
}

impl FsWriter for OsFsWriter {
    type Error = io::Error;

    #[inline]
    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error> {
        Self::validate_path(path)?;
        std::fs::create_dir_all(self.resolve(path))
    }

    #[inline]
    fn write_file(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), Self::Error> {
        Self::validate_path(path)?;
        std::fs::write(self.resolve(path), contents)
    }

    #[inline]
    fn atomic_write(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), Self::Error> {
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

    #[inline]
    fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
        Self::validate_path(from)?;
        Self::validate_path(to)?;
        std::fs::rename(self.resolve(from), self.resolve(to))
    }

    #[inline]
    fn remove_file(&self, path: &Path) -> Result<(), Self::Error> {
        Self::validate_path(path)?;
        std::fs::remove_file(self.resolve(path))
    }
}
