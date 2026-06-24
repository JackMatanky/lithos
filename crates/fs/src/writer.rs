//! Filesystem writer utilities for safe writes.
//!
//! The writer keeps all paths scoped to a root directory.
//! Safety is guaranteed by the caller passing a pre-validated `WriteTarget`.

use std::{
    io,
    path::{Path, PathBuf},
};

use super::{WriteTarget, error::WriteError};

/// A port for writing files safely to a vault root.
pub trait FileWriter {
    /// Atomically creates a new file at the given target, failing if it exists.
    ///
    /// Parent directories are created automatically. The target is
    /// pre-validated by [`WriteTarget`].
    ///
    /// # Errors
    /// Returns [`WriteError::AlreadyExists`] if the file exists, or
    /// [`WriteError::Io`] for filesystem failures.
    fn create_new(
        &self,
        target: &WriteTarget,
        contents: &[u8],
    ) -> Result<(), WriteError>;
}

/// Production filesystem writer using `std::fs`.
///
/// Implements [`FileWriter`] to perform atomic operations within a scoped root.
#[derive(Debug, Clone)]
pub struct Writer {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl Writer {
    /// Creates a new filesystem writer.
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

    /// Creates all directories in the given path.
    #[inline]
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(self.resolve(path))
    }
}

impl FileWriter for Writer {
    #[inline]
    fn create_new(
        &self,
        target: &WriteTarget,
        contents: &[u8],
    ) -> Result<(), WriteError> {
        use std::io::Write as _;

        let path = target.as_path();

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            self.create_dir_all(parent).map_err(|source| WriteError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        }

        let resolved = self.resolve(path);
        let mut file = std::fs::File::create_new(resolved).map_err(|err| {
            if err.kind() == io::ErrorKind::AlreadyExists {
                WriteError::AlreadyExists {
                    path: path.to_path_buf(),
                }
            } else {
                WriteError::Io {
                    path: path.to_path_buf(),
                    source: err,
                }
            }
        })?;

        file.write_all(contents).map_err(|source| WriteError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules use conventional use-before-mod ordering"
)]
mod tests {
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;

    mod create_new {
        use super::*;

        #[test]
        fn creates_file_when_not_exists() {
            let dir = TempDir::new().expect("tempdir");
            let writer = Writer::new(dir.path());
            let target = WriteTarget::try_new("file.txt").unwrap();

            writer.create_new(&target, b"content").expect("create");

            let content =
                std::fs::read(dir.path().join("file.txt")).expect("read");
            assert_eq!(content, b"content");
        }

        #[test]
        fn rejects_existing_file() {
            let dir = TempDir::new().expect("tempdir");
            std::fs::create_dir_all(dir.path().join("sub"))
                .expect("expected pre-created parent directory");
            std::fs::write(dir.path().join("sub/x.md"), b"original")
                .expect("write");
            let writer = Writer::new(dir.path());
            let target = WriteTarget::try_new("sub/x.md").unwrap();

            let result = writer.create_new(&target, b"new");
            assert!(matches!(result, Err(WriteError::AlreadyExists { .. })));
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
    }
}
