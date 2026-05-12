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

    /// Returns the underlying path as a UTF-8 string slice if it is valid
    /// UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
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

    /// Returns the underlying path as a UTF-8 string slice if it is valid
    /// UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
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

/// A validated file path (absolute or relative).
///
/// **Phase 2 Revision (2026-05-12):** Changed from wrapping `RelativePath`
/// to wrapping `PathBuf` to support absolute paths from `walkdir::DirEntry`.
/// Use `as_relative(base)` to convert to vault-relative paths at storage
/// boundary.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct FilePath(#[rkyv(with = AsString)] PathBuf);

impl FilePath {
    /// Create a new file path from a `PathBuf` (absolute or relative).
    ///
    /// # Errors
    /// Returns error if path is empty, contains `..` or `.` components,
    /// or has platform-specific prefixes.
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        // Check for . components
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
        // Check for .. and prefix components
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
        Ok(Self(path))
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Convert to vault-relative path by stripping base prefix.
    ///
    /// # Errors
    /// Returns error if path is not within the base directory.
    #[inline]
    pub fn as_relative(
        &self,
        base: &Path,
    ) -> Result<RelativePath, super::error::ParseError> {
        use super::error::ParseError;

        let rel = self.0.strip_prefix(base).map_err(|_| ParseError::Io {
            path: self.0.clone(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Path {} is not within base {}",
                    self.0.display(),
                    base.display()
                ),
            ),
        })?;

        RelativePath::try_from(rel).map_err(|e| ParseError::Io {
            path: self.0.clone(),
            source: e,
        })
    }
}

impl TryFrom<RelativePath> for FilePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::new(path.0)
    }
}

impl TryFrom<&str> for FilePath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(PathBuf::from(value))
    }
}

impl AsRef<Path> for FilePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

/// A validated directory path (absolute or relative).
///
/// **Phase 2 Revision (2026-05-12):** Changed from wrapping `RelativePath`
/// to wrapping `PathBuf` to support absolute paths from `walkdir::DirEntry`.
/// Use `as_relative(base)` to convert to vault-relative paths at storage
/// boundary.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct DirPath(#[rkyv(with = AsString)] PathBuf);

impl DirPath {
    /// Create a new directory path from a `PathBuf` (absolute or relative).
    ///
    /// # Errors
    /// Returns error if path is empty, contains `..` or `.` components,
    /// or has platform-specific prefixes.
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        // Check for . components
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
        // Check for .. and prefix components
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
        Ok(Self(path))
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Convert to vault-relative path by stripping base prefix.
    ///
    /// # Errors
    /// Returns error if path is not within the base directory.
    #[inline]
    pub fn as_relative(
        &self,
        base: &Path,
    ) -> Result<RelativePath, super::error::ParseError> {
        use super::error::ParseError;

        let rel = self.0.strip_prefix(base).map_err(|_| ParseError::Io {
            path: self.0.clone(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Path {} is not within base {}",
                    self.0.display(),
                    base.display()
                ),
            ),
        })?;

        RelativePath::try_from(rel).map_err(|e| ParseError::Io {
            path: self.0.clone(),
            source: e,
        })
    }
}

impl TryFrom<RelativePath> for DirPath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::new(path.0)
    }
}

impl TryFrom<&str> for DirPath {
    type Error = std::io::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(PathBuf::from(value))
    }
}

impl AsRef<Path> for DirPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

/// Unified path enum representing either a file or a directory.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FsPath {
    /// A file path.
    File(FilePath),
    /// A directory path.
    Dir(DirPath),
}

impl FsPath {
    /// Returns `true` if this is a file path.
    #[inline]
    #[must_use]
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Returns `true` if this is a directory path.
    #[inline]
    #[must_use]
    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// Returns the file path if this is a file.
    #[inline]
    #[must_use]
    pub fn as_file(&self) -> Option<&FilePath> {
        match self {
            Self::File(p) => Some(p),
            Self::Dir(_) => None,
        }
    }

    /// Returns the directory path if this is a directory.
    #[inline]
    #[must_use]
    pub fn as_dir(&self) -> Option<&DirPath> {
        match self {
            Self::Dir(p) => Some(p),
            Self::File(_) => None,
        }
    }

    /// Convert to vault-relative path by stripping base prefix.
    ///
    /// # Errors
    /// Returns error if path is not within the base directory.
    #[inline]
    pub fn as_relative(
        &self,
        base: &Path,
    ) -> Result<RelativePath, super::error::ParseError> {
        match self {
            Self::File(p) => p.as_relative(base),
            Self::Dir(p) => p.as_relative(base),
        }
    }
}

/// A zero-copy view of a parent directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentDir<'a> {
    /// The vault root.
    Root,
    /// A sub-directory path view.
    Path(&'a Path),
}

impl<'a> ParentDir<'a> {
    /// Extract the parent directory from a path.
    #[inline]
    #[must_use]
    pub fn from_path(path: &'a Path) -> Self {
        match path.parent() {
            Some(p) if p.as_os_str().is_empty() => Self::Root,
            Some(p) => Self::Path(p),
            None => Self::Root,
        }
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

    mod file_path {
        use super::*;

        #[test]
        fn should_wrap_relative_path() {
            let path = FilePath::try_from("a/b/file.txt").unwrap();
            assert_eq!(path.as_path(), Path::new("a/b/file.txt"));
        }

        #[test]
        fn should_reject_invalid_paths() {
            assert!(FilePath::try_from("../traversal").is_err());
            // Absolute paths are now accepted (Phase 1 revision)
        }

        #[test]
        fn should_accept_absolute_paths() {
            let path =
                FilePath::new(PathBuf::from("/vault/notes/file.txt")).unwrap();
            assert_eq!(path.as_path(), Path::new("/vault/notes/file.txt"));
        }

        #[test]
        fn should_convert_absolute_to_relative() {
            let path =
                FilePath::new(PathBuf::from("/vault/notes/file.txt")).unwrap();
            let base = Path::new("/vault");
            let relative = path.as_relative(base).unwrap();
            assert_eq!(relative.as_path(), Path::new("notes/file.txt"));
        }

        #[test]
        fn should_error_when_path_outside_base() {
            let path = FilePath::new(PathBuf::from("/other/file.txt")).unwrap();
            let base = Path::new("/vault");
            assert!(path.as_relative(base).is_err());
        }
    }

    mod dir_path {
        use super::*;

        #[test]
        fn should_wrap_relative_path() {
            let path = DirPath::try_from("a/b/dir").unwrap();
            assert_eq!(path.as_path(), Path::new("a/b/dir"));
        }

        #[test]
        fn should_reject_invalid_paths() {
            assert!(DirPath::try_from("..").is_err());
        }
    }

    mod fs_path {
        use super::*;

        #[test]
        fn should_handle_file_and_dir_variants() {
            let file = FilePath::try_from("file.txt").unwrap();
            let dir = DirPath::try_from("dir").unwrap();

            let fs_file = FsPath::File(file);
            let fs_dir = FsPath::Dir(dir);

            assert!(fs_file.is_file());
            assert!(!fs_file.is_dir());
            assert!(fs_dir.is_dir());
            assert!(!fs_dir.is_file());
        }

        #[test]
        fn should_convert_to_relative_with_base() {
            let file = FilePath::new(PathBuf::from("/vault/file.txt")).unwrap();
            let dir = DirPath::new(PathBuf::from("/vault/dir")).unwrap();

            let fs_file = FsPath::File(file);
            let fs_dir = FsPath::Dir(dir);

            let base = Path::new("/vault");
            assert_eq!(
                fs_file.as_relative(base).unwrap().as_path(),
                Path::new("file.txt")
            );
            assert_eq!(
                fs_dir.as_relative(base).unwrap().as_path(),
                Path::new("dir")
            );
        }
    }

    mod parent_dir {
        use super::*;

        #[test]
        fn parent_dir_extracts_parent_path() {
            let path = Path::new("a/b/c");
            let parent = ParentDir::from_path(path);

            match parent {
                ParentDir::Path(p) => assert_eq!(p, Path::new("a/b")),
                #[allow(
                    clippy::panic,
                    reason = "Test expects ParentDir::Path variant"
                )]
                ParentDir::Root => panic!("Expected ParentDir::Path"),
            }

            let root_path = Path::new("file.txt");
            let root_parent = ParentDir::from_path(root_path);
            assert!(matches!(root_parent, ParentDir::Root));
        }
    }
}
