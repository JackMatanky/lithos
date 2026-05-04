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

use super::file::FileName;

/// A validated vault-relative path.
///
/// This type ensures that paths do not escape the vault root using `..`
/// traversal and are kept relative for portability.
///
/// # Invariants
///
/// - Must be a relative path.
/// - Must not contain `..` (parent directory traversal).
/// - Must not contain `.` (current directory components).
/// - Must not contain platform-specific path prefixes.
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
    pub fn filename(&self) -> Option<FileName> {
        self.try_filename().ok().flatten()
    }

    /// Returns the filename component of this path if it exists.
    ///
    /// # Errors
    /// Returns an error when a filename exists but cannot be represented as
    /// valid UTF-8.
    #[inline]
    pub fn try_filename(&self) -> Result<Option<FileName>, std::io::Error> {
        match self.0.file_name() {
            Some(_) => FileName::try_from(self.0.as_path()).map(Some),
            None => Ok(None),
        }
    }

    /// Private helper to validate relative path invariants.
    fn validate(path: &Path) -> Result<(), std::io::Error> {
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
        if path
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|segment| segment == ".")
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must not contain current directory components (.)",
            ));
        }
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Path must not contain parent components (..)",
                    ));
                }
                Component::Prefix(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Path must not contain platform-specific prefixes",
                    ));
                }
                Component::CurDir
                | Component::Normal(_)
                | Component::RootDir => {}
            }
        }
        Ok(())
    }
}

impl TryFrom<PathBuf> for RelativePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for RelativePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::validate(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl TryFrom<&str> for RelativePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(value))
    }
}

impl AsRef<Path> for RelativePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
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

    /// Returns the filename component of this path if it exists.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> Option<FileName> {
        self.try_filename().ok().flatten()
    }

    /// Returns the filename component of this path if it exists.
    ///
    /// # Errors
    /// Returns an error when a filename exists but cannot be represented as
    /// valid UTF-8.
    #[inline]
    pub fn try_filename(&self) -> Result<Option<FileName>, std::io::Error> {
        match self.0.file_name() {
            Some(_) => FileName::try_from(self.0.as_path()).map(Some),
            None => Ok(None),
        }
    }

    fn validate(path: &Path) -> Result<(), std::io::Error> {
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
        Ok(())
    }
}

impl TryFrom<PathBuf> for AbsolutePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for AbsolutePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::validate(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl TryFrom<&str> for AbsolutePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(value))
    }
}

impl AsRef<Path> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl ArchivedAbsolutePath {
    /// Return the inner path as a standard library [`Path`].
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.0.as_str())
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
        fn should_reject_curdir_component() {
            let result = RelativePath::try_from(PathBuf::from("a/./b"));
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
        fn try_filename_should_extract_filename() {
            let path =
                RelativePath::try_from(PathBuf::from("a/b/file.txt")).unwrap();
            let filename = path.try_filename().unwrap().unwrap();
            assert_eq!(filename.as_str(), "file.txt");
        }

        #[cfg(unix)]
        #[test]
        fn try_filename_should_report_invalid_utf8() {
            use std::os::unix::ffi::OsStringExt as _;

            let path = PathBuf::from("a")
                .join(std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]));

            let relative = RelativePath::try_from(path).unwrap();
            let error = relative.try_filename().unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        }

        #[test]
        fn should_support_try_from_str() {
            let path = RelativePath::try_from("a/b/file.txt").unwrap();
            assert_eq!(path.as_path(), Path::new("a/b/file.txt"));
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

        #[test]
        fn should_support_try_from_str() {
            let path = AbsolutePath::try_from("/a/b/file.txt").unwrap();
            assert_eq!(path.as_path(), Path::new("/a/b/file.txt"));
        }

        #[test]
        fn should_extract_filename() {
            let path =
                AbsolutePath::try_from(PathBuf::from("/a/b/file.txt")).unwrap();
            let filename = path.filename().unwrap();
            assert_eq!(filename.as_str(), "file.txt");
        }

        #[test]
        fn try_filename_should_extract_filename() {
            let path =
                AbsolutePath::try_from(PathBuf::from("/a/b/file.txt")).unwrap();
            let filename = path.try_filename().unwrap().unwrap();
            assert_eq!(filename.as_str(), "file.txt");
        }

        #[cfg(unix)]
        #[test]
        fn try_filename_should_report_invalid_utf8() {
            use std::os::unix::ffi::OsStringExt as _;

            let path = PathBuf::from("/tmp")
                .join(std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]));

            let absolute = AbsolutePath::try_from(path).unwrap();
            let error = absolute.try_filename().unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        }
    }
}
