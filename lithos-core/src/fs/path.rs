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

use super::{
    format::FileExtensionRef,
    name::{BaseName, BaseNameRef, DirName, DirNameRef, FileName, FileNameRef},
};

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
        let has_cur_dir = if let Some(s) = path.to_str() {
            s.split(['/', '\\']).any(|segment| segment == ".")
        } else {
            path.to_string_lossy()
                .split(['/', '\\'])
                .any(|segment| segment == ".")
        };

        if has_cur_dir {
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
    /// Returns error if path is empty or does not refer to a file.
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        if !path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path does not refer to a file",
            ));
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
    ) -> Result<RelativePath, super::error::ReadError> {
        use super::error::ReadError;

        let rel =
            self.0.strip_prefix(base).map_err(|_| ReadError::NotInBase {
                path: self.0.clone(),
                base: base.to_path_buf(),
            })?;

        RelativePath::try_from(rel).map_err(|e| ReadError::Io {
            path: self.0.clone(),
            source: e,
        })
    }

    /// Returns `true` if the path is absolute.
    #[inline]
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Returns `true` if the path is relative.
    #[inline]
    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.0.is_relative()
    }

    /// Returns the parent directory, or [`ParentDir::Root`] if none exists.
    #[inline]
    #[must_use]
    pub fn parent(&self) -> ParentDir<'_> {
        ParentDir::from_path(&self.0)
    }

    /// Returns the borrowed filename view, if present.
    #[inline]
    #[must_use]
    pub fn filename_ref(&self) -> Option<FileNameRef<'_>> {
        self.0.file_name().map(FileNameRef)
    }

    /// Returns the owned UTF-8 filename, if present and valid UTF-8.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> Option<FileName> {
        FileName::try_from(self.0.as_path()).ok()
    }

    /// Returns the borrowed basename view directly from the path.
    ///
    /// Gets the file stem (basename without extension) directly from the path
    /// without going through filename validation.
    #[inline]
    #[must_use]
    pub fn basename_ref(&self) -> Option<BaseNameRef<'_>> {
        self.0.file_stem().map(BaseNameRef)
    }

    /// Returns the owned UTF-8 basename directly from the path.
    ///
    /// Gets the file stem (basename without extension) directly from the path
    /// without going through filename validation.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> Option<BaseName> {
        BaseName::try_from(self.0.as_path()).ok()
    }

    /// Returns the borrowed file extension view, if present.
    #[inline]
    #[must_use]
    pub fn extension_ref(&self) -> Option<FileExtensionRef<'_>> {
        self.0.extension().map(FileExtensionRef)
    }

    /// Returns `true` if the path has a filename component.
    #[inline]
    #[must_use]
    pub fn has_filename(&self) -> bool {
        self.0.file_name().is_some()
    }

    /// Returns `true` if the path has an extension.
    #[inline]
    #[must_use]
    pub fn has_extension(&self) -> bool {
        self.0.extension().is_some()
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

impl From<PathBuf> for FilePath {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<FilePath> for PathBuf {
    #[inline]
    fn from(path: FilePath) -> Self {
        path.0
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
    /// Returns error if path is empty or does not refer to a directory.
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if path.as_os_str().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }
        if !path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path does not refer to a directory",
            ));
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
    ) -> Result<RelativePath, super::error::ReadError> {
        use super::error::ReadError;

        let rel =
            self.0.strip_prefix(base).map_err(|_| ReadError::NotInBase {
                path: self.0.clone(),
                base: base.to_path_buf(),
            })?;

        RelativePath::try_from(rel).map_err(|e| ReadError::Io {
            path: self.0.clone(),
            source: e,
        })
    }

    /// Returns `true` if the path is absolute.
    #[inline]
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Returns `true` if the path is relative.
    #[inline]
    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.0.is_relative()
    }

    /// Returns the parent directory, or [`ParentDir::Root`] if none exists.
    #[inline]
    #[must_use]
    pub fn parent(&self) -> ParentDir<'_> {
        ParentDir::from_path(&self.0)
    }

    /// Returns the borrowed directory name view, if present.
    #[inline]
    #[must_use]
    pub fn dirname_ref(&self) -> Option<DirNameRef<'_>> {
        self.0.file_name().map(DirNameRef)
    }

    /// Returns the owned UTF-8 directory name, if present and valid UTF-8.
    #[inline]
    #[must_use]
    pub fn dirname(&self) -> Option<DirName> {
        self.0
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| DirName::new(s.into()))
    }

    /// Returns `true` if the path has a directory name component.
    #[inline]
    #[must_use]
    pub fn has_dirname(&self) -> bool {
        self.0.file_name().is_some()
    }

    /// Joins a child path onto this directory path.
    #[inline]
    #[must_use]
    pub fn join<P>(&self, child: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.0.join(child)
    }

    /// Joins a child filename onto this directory path.
    #[inline]
    #[must_use]
    pub fn join_file<P>(&self, child: P) -> FilePath
    where
        P: AsRef<Path>,
    {
        FilePath(self.0.join(child))
    }

    /// Joins a child directory name onto this directory path.
    #[inline]
    #[must_use]
    pub fn join_dir<P>(&self, child: P) -> DirPath
    where
        P: AsRef<Path>,
    {
        DirPath(self.0.join(child))
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

impl From<PathBuf> for DirPath {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<DirPath> for PathBuf {
    #[inline]
    fn from(path: DirPath) -> Self {
        path.0
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
    /// Returns the underlying path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        match self {
            Self::File(p) => p.as_path(),
            Self::Dir(p) => p.as_path(),
        }
    }

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
    ) -> Result<RelativePath, super::error::FsError> {
        match self {
            Self::File(p) => {
                p.as_relative(base).map_err(super::error::FsError::Read)
            }
            Self::Dir(p) => {
                p.as_relative(base).map_err(super::error::FsError::Read)
            }
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
        use tempfile::NamedTempFile;

        use super::*;

        #[test]
        fn should_wrap_relative_path() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::from(temp.path().to_path_buf());
            assert_eq!(path.as_path(), temp.path());
        }

        #[test]
        fn should_reject_empty_path() {
            assert!(FilePath::new(PathBuf::from("")).is_err());
        }

        #[test]
        fn should_reject_nonexistent_path() {
            assert!(
                FilePath::new(PathBuf::from("/nonexistent/file.txt")).is_err()
            );
        }

        #[test]
        fn should_accept_absolute_paths() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            assert_eq!(path.as_path(), temp.path());
        }

        #[test]
        fn should_convert_absolute_to_relative() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let temp = NamedTempFile::new_in(temp_dir.path()).unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let base = temp_dir.path();
            let relative = path.as_relative(base).unwrap();
            assert_eq!(
                relative.as_path(),
                Path::new(temp.path().file_name().unwrap())
            );
        }

        #[test]
        fn should_error_when_path_outside_base() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let base = Path::new("/other");
            assert!(path.as_relative(base).is_err());
        }

        #[test]
        fn should_detect_absolute_path() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            assert!(path.is_absolute());
            assert!(!path.is_relative());
        }

        #[test]
        fn should_detect_relative_path() {
            let temp_dir = tempfile::tempdir().unwrap();
            let rel_path = temp_dir.path().file_name().unwrap();
            assert!(Path::new(rel_path).is_relative());
        }

        #[test]
        fn should_extract_filename() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let filename = path.filename().unwrap();
            assert_eq!(
                filename.as_str(),
                temp.path().file_name().unwrap().to_str().unwrap()
            );
        }

        #[test]
        fn should_extract_basename() {
            let temp = NamedTempFile::new_in(std::env::temp_dir()).unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let base = path.basename();
            assert!(base.is_some());
        }

        #[test]
        fn should_extract_extension() {
            let temp_path = std::env::temp_dir().join("test.md");
            std::fs::write(&temp_path, "content").unwrap();
            let path = FilePath::new(temp_path.clone()).unwrap();
            assert!(path.has_extension());
            let ext = path.extension_ref();
            assert!(ext.is_some());
            std::fs::remove_file(&temp_path).ok();
        }

        #[test]
        fn should_return_parent() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let temp = NamedTempFile::new_in(temp_dir.path()).unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let is_root = matches!(path.parent(), ParentDir::Root);
            assert!(!is_root, "expected path, got Root");
            if let ParentDir::Path(p) = path.parent() {
                assert_eq!(p, temp_dir.path());
            }
        }

        #[test]
        fn should_convert_to_pathbuf() {
            let temp = NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let buf: PathBuf = path.into();
            assert_eq!(buf, temp.path().to_path_buf());
        }
    }

    mod dir_path {
        use tempfile::TempDir;

        use super::*;

        #[test]
        fn should_wrap_relative_path() {
            let temp = TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            assert_eq!(path.as_path(), temp.path());
        }

        #[test]
        fn should_reject_empty_path() {
            assert!(DirPath::new(PathBuf::from("")).is_err());
        }

        #[test]
        fn should_reject_nonexistent_path() {
            assert!(DirPath::new(PathBuf::from("/nonexistent/dir")).is_err());
        }

        #[test]
        fn should_detect_absolute_path() {
            let temp = TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            assert!(path.is_absolute());
            assert!(!path.is_relative());
        }

        #[test]
        fn should_extract_dirname() {
            let temp = TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            let name = path.dirname();
            assert!(name.is_some());
        }

        #[test]
        fn should_join_child_path() {
            let temp = TempDir::new().unwrap();
            let dir = DirPath::new(temp.path().to_path_buf()).unwrap();
            let joined = dir.join("subdir/file.txt");
            assert!(joined.to_string_lossy().ends_with("subdir/file.txt"));
        }

        #[test]
        fn should_join_child_file() {
            let temp = TempDir::new().unwrap();
            let dir = DirPath::new(temp.path().to_path_buf()).unwrap();
            let file = dir.join_file("notes.txt");
            assert!(file.as_path().to_string_lossy().ends_with("notes.txt"));
        }

        #[test]
        fn should_join_child_dir() {
            let temp = TempDir::new().unwrap();
            let dir = DirPath::new(temp.path().to_path_buf()).unwrap();
            let subdir = dir.join_dir("notes");
            assert!(subdir.as_path().to_string_lossy().ends_with("notes"));
        }

        #[test]
        fn should_return_parent() {
            let temp = TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            let is_root = matches!(path.parent(), ParentDir::Root);
            assert!(!is_root, "expected path, got Root");
        }

        #[test]
        fn should_convert_to_pathbuf() {
            let temp = TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            let buf: PathBuf = path.into();
            assert_eq!(buf, temp.path().to_path_buf());
        }
    }

    mod fs_path {
        use tempfile::{NamedTempFile, TempDir};

        use super::*;

        #[test]
        fn should_handle_file_and_dir_variants() {
            let temp_dir = TempDir::new().unwrap();
            let temp_file = NamedTempFile::new_in(temp_dir.path()).unwrap();

            let file = FilePath::new(temp_file.path().to_path_buf()).unwrap();
            let dir = DirPath::new(temp_dir.path().to_path_buf()).unwrap();

            let fs_file = FsPath::File(file);
            let fs_dir = FsPath::Dir(dir);

            assert!(fs_file.is_file());
            assert!(!fs_file.is_dir());
            assert!(fs_dir.is_dir());
            assert!(!fs_dir.is_file());
        }

        #[test]
        fn should_convert_to_relative_with_base() {
            let temp_dir = TempDir::new().unwrap();
            let temp_file = NamedTempFile::new_in(temp_dir.path()).unwrap();

            let dir_path = temp_dir.path().to_path_buf();
            let base = dir_path.clone();
            let file = FilePath::new(temp_file.path().to_path_buf()).unwrap();
            let _dir = DirPath::new(dir_path).unwrap();

            let fs_file = FsPath::File(file);

            let rel_file = fs_file.as_relative(&base).unwrap();
            assert_eq!(
                rel_file.as_path(),
                temp_file.path().file_name().unwrap()
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
