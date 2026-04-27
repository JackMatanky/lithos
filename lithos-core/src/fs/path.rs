//! Validated path types for the Lithos core library.
//!
//! Provides type-safe wrappers for relative and absolute paths, ensuring
//! consistent validation and policy across the project.

#![expect(
    clippy::module_name_repetitions,
    reason = "RelativePath and AbsolutePath are canonical names"
)]

use std::path::{Component, Path, PathBuf};

use rkyv::{Archive, Deserialize, Serialize, with::AsString};

use super::filename::Filename;

/// A validated vault-relative path.
///
/// This type ensures that paths do not escape the vault root using `..`
/// traversal and are kept relative for portability.
///
/// # Invariants
///
/// - Must be a relative path.
/// - Must not contain `..` (parent directory traversal).
/// - Must not be empty.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct RelativePath(
    /// Internal path storage.
    #[rkyv(with = AsString)]
    PathBuf,
);

impl RelativePath {
    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns the filename component of this path if it exists.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> Option<Filename> {
        Filename::try_from(self.0.as_path()).ok()
    }

    /// Returns the basename (filename without extension) if it exists.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> Option<&str> {
        let stem = self.0.file_stem()?;
        stem.to_str()
    }

    /// Private helper to validate relative path invariants.
    fn validate_relative_path_invariants(
        path: &Path,
    ) -> Result<(), std::io::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        if path.is_absolute() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must be relative",
            ));
        }
        if path.components().any(|component| component == Component::ParentDir)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must not contain parent components (..)",
            ));
        }
        Ok(())
    }
}

impl ArchivedRelativePath {
    /// Return the inner path as a standard library [`Path`].
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.0.as_str())
    }
}

impl TryFrom<PathBuf> for RelativePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::validate_relative_path_invariants(&path)?;
        Ok(Self(path))
    }
}

impl From<&'static str> for RelativePath {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self(PathBuf::from(value))
    }
}

impl std::fmt::Display for RelativePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

/// A validated absolute path.
///
/// This type ensures that paths are fully resolved and absolute on the
/// filesystem.
///
/// # Invariants
///
/// - Must be an absolute path.
/// - Must not be empty.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct AbsolutePath(
    /// Internal path storage.
    #[rkyv(with = AsString)]
    PathBuf,
);

impl AbsolutePath {
    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<PathBuf> for AbsolutePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        if !path.is_absolute() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path must be absolute: {}", path.to_string_lossy()),
            ));
        }
        Ok(Self(path))
    }
}

impl std::fmt::Display for AbsolutePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod relative_path {
        use super::*;

        #[test]
        fn should_reject_empty() {
            let result = RelativePath::try_from(PathBuf::from(""));
            let _error = result.unwrap_err();
        }

        #[test]
        fn should_reject_absolute() {
            let result = RelativePath::try_from(PathBuf::from("/abs"));
            let _error = result.unwrap_err();
        }

        #[test]
        fn should_reject_parent_traversal() {
            let result = RelativePath::try_from(PathBuf::from("a/../b"));
            let _error = result.unwrap_err();
        }

        #[test]
        fn should_accept_valid_relative() {
            let result = RelativePath::try_from(PathBuf::from("a/b/c"));
            let path = result.unwrap();
            assert_eq!(path.as_path(), Path::new("a/b/c"));
        }

        #[test]
        fn should_extract_filename() {
            let path =
                RelativePath::try_from(PathBuf::from("a/b/file.txt")).unwrap();
            let filename = path.filename().unwrap();
            assert_eq!(filename.as_str(), "file.txt");
        }

        #[test]
        fn should_extract_basename() {
            let path =
                RelativePath::try_from(PathBuf::from("a/b/file.txt")).unwrap();
            assert_eq!(path.basename(), Some("file"));
        }
    }

    mod absolute_path {
        use super::*;

        #[test]
        fn should_reject_empty() {
            let result = AbsolutePath::try_from(PathBuf::from(""));
            let _error = result.unwrap_err();
        }

        #[test]
        fn should_reject_relative() {
            let result = AbsolutePath::try_from(PathBuf::from("rel"));
            let _error = result.unwrap_err();
        }

        #[test]
        fn should_accept_valid_absolute() {
            let result = AbsolutePath::try_from(PathBuf::from("/a/b/c"));
            let _path = result.unwrap();
        }
    }
}
