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
    pub fn try_filename(&self) -> Result<Option<FileName>, super::PathError> {
        match self.0.file_name() {
            Some(_) => FileName::try_from(self.0.as_path()).map(Some),
            None => Ok(None),
        }
    }

    /// Private helper to validate relative path invariants.
    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if path.is_absolute() {
            return Err(super::PathError::NotRelative(path.to_path_buf()));
        }
        let has_cur_dir = if let Some(s) = path.to_str() {
            s.split(['/', '\\']).any(|segment| segment == ".")
        } else {
            path.to_string_lossy()
                .split(['/', '\\'])
                .any(|segment| segment == ".")
        };

        if has_cur_dir {
            return Err(super::PathError::CurrentDirComponent(
                path.to_path_buf(),
            ));
        }

        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(super::PathError::ParentTraversal(
                        path.to_path_buf(),
                    ));
                }
                Component::Prefix(_) => {
                    return Err(super::PathError::PlatformPrefix(
                        path.to_path_buf(),
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
    type Error = super::PathError;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for RelativePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::validate(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl TryFrom<&str> for RelativePath {
    type Error = super::PathError;

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

/// Normalized vault-relative path using forward slashes.
///
/// This type enforces vault-relative path constraints and normalizes all
/// paths to use forward slashes (`/`) for cross-platform compatibility.
///
/// # Use Cases
///
/// - Database storage keys (consistent across platforms)
/// - Serialized path representation in rkyv archives
/// - Path comparison and hashing (`HashMap` keys, `HashSet` members)
///
/// # Comparison with [`RelativePath`]
///
/// - [`NormalizedPath`]: Forward slashes, `Box<str>` storage, `as_str() ->
///   &str`
/// - [`RelativePath`]: Platform slashes, `PathBuf` storage, `as_path() ->
///   &Path`
///
/// Use [`RelativePath`] for filesystem operations; use [`NormalizedPath`]
/// for storage and serialization.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NormalizedPath(Box<str>);

impl NormalizedPath {
    /// Creates a new normalized vault-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`PathError`] when path validation fails.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, super::PathError> {
        let normalized = Self::normalize_slashes(path);
        let normalized = normalized.as_ref().trim();

        if normalized.is_empty() {
            return Err(super::PathError::Empty);
        }

        let path_buf = PathBuf::from(normalized);
        if path_buf.is_absolute() {
            return Err(super::PathError::NotRelative(path_buf));
        }

        for component in path_buf.components() {
            match component {
                Component::ParentDir => {
                    return Err(super::PathError::ParentTraversal(
                        PathBuf::from(normalized),
                    ));
                }
                Component::CurDir => {
                    return Err(super::PathError::CurrentDirComponent(
                        PathBuf::from(normalized),
                    ));
                }
                Component::Prefix(_) => {
                    return Err(super::PathError::PlatformPrefix(
                        PathBuf::from(normalized),
                    ));
                }
                Component::RootDir | Component::Normal(_) => {}
            }
        }

        Ok(Self(normalized.into()))
    }

    #[inline]
    fn normalize_slashes(path: &str) -> std::borrow::Cow<'_, str> {
        if path.contains('\\') {
            let mut owned = String::with_capacity(path.len());
            for ch in path.chars() {
                if ch == '\\' {
                    owned.push('/');
                } else {
                    owned.push(ch);
                }
            }
            std::borrow::Cow::Owned(owned)
        } else {
            std::borrow::Cow::Borrowed(path)
        }
    }

    /// Returns the normalized path string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
    pub fn try_filename(&self) -> Result<Option<FileName>, super::PathError> {
        match self.0.file_name() {
            Some(_) => FileName::try_from(self.0.as_path()).map(Some),
            None => Ok(None),
        }
    }

    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if !path.is_absolute() {
            return Err(super::PathError::NotAbsolute(path.to_path_buf()));
        }
        Ok(())
    }
}

impl TryFrom<PathBuf> for AbsolutePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for AbsolutePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::validate(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl TryFrom<&str> for AbsolutePath {
    type Error = super::PathError;

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
    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if !path.is_file() {
            return Err(super::PathError::NotAFile(path.to_path_buf()));
        }
        Ok(())
    }

    /// Create a new file path from a `PathBuf` (absolute or relative).
    ///
    /// # Errors
    /// Returns error if path is empty or does not refer to a file.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, super::PathError> {
        Self::validate(path.as_path())?;
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
    ) -> Result<RelativePath, super::error::FsError> {
        use super::error::{FsError, ReadError};

        let rel =
            self.0.strip_prefix(base).map_err(|_| ReadError::NotInBase {
                path: self.0.clone(),
                base: base.to_path_buf(),
            })?;

        RelativePath::try_from(rel).map_err(FsError::from)
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
    type Error = super::PathError;

    #[inline]
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::try_new(path.0)
    }
}

impl TryFrom<&str> for FilePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl TryFrom<PathBuf> for FilePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_new(path)
    }
}

impl AsRef<Path> for FilePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
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
    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if !path.is_dir() {
            return Err(super::PathError::NotADirectory(path.to_path_buf()));
        }
        Ok(())
    }

    /// Create a new directory path from a `PathBuf` (absolute or relative).
    ///
    /// # Errors
    /// Returns error if path is empty or does not refer to a directory.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, super::PathError> {
        Self::validate(path.as_path())?;
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
    ) -> Result<RelativePath, super::error::FsError> {
        use super::error::{FsError, ReadError};

        let rel =
            self.0.strip_prefix(base).map_err(|_| ReadError::NotInBase {
                path: self.0.clone(),
                base: base.to_path_buf(),
            })?;

        RelativePath::try_from(rel).map_err(FsError::from)
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

    /// Joins a child path and classifies it by path shape.
    ///
    /// Classification is syntactic: paths with an extension become
    /// [`FsPath::File`], otherwise [`FsPath::Dir`].
    #[inline]
    #[must_use]
    pub fn join_path<P>(&self, child: P) -> FsPath
    where
        P: AsRef<Path>,
    {
        let child = child.as_ref();
        let joined = self.0.join(child);

        if child.extension().is_some() {
            return FsPath::File(FilePath(joined));
        }

        FsPath::Dir(DirPath(joined))
    }
}

impl TryFrom<RelativePath> for DirPath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::try_new(path.0)
    }
}

impl TryFrom<&str> for DirPath {
    type Error = super::PathError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl TryFrom<PathBuf> for DirPath {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_new(path)
    }
}

impl AsRef<Path> for DirPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
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
            Self::File(p) => p.as_relative(base),
            Self::Dir(p) => p.as_relative(base),
        }
    }
}

impl TryFrom<walkdir::DirEntry> for FsPath {
    type Error = super::PathError;

    #[inline]
    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        let file_type = entry.file_type();
        let path = entry.into_path();

        if file_type.is_dir() {
            DirPath::try_new(path).map(Self::Dir)
        } else {
            FilePath::try_new(path).map(Self::File)
        }
    }
}

/// A zero-copy reference to either a file or directory path.
///
/// This enum provides a borrowed view into an `FsPath` without cloning the
/// underlying paths. Useful for operations that need to inspect paths without
/// taking ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsPathRef<'a> {
    /// A reference to a file path.
    File(&'a FilePath),
    /// A reference to a directory path.
    Dir(&'a DirPath),
}

impl<'a> FsPathRef<'a> {
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
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Returns `true` if this is a directory path.
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// Returns the file path if this is a file.
    #[inline]
    #[must_use]
    pub const fn as_file(&self) -> Option<&'a FilePath> {
        match self {
            Self::File(p) => Some(p),
            Self::Dir(_) => None,
        }
    }

    /// Returns the directory path if this is a directory.
    #[inline]
    #[must_use]
    pub const fn as_dir(&self) -> Option<&'a DirPath> {
        match self {
            Self::Dir(p) => Some(p),
            Self::File(_) => None,
        }
    }

    /// Convert to an owned `FsPath` by cloning the underlying path.
    #[inline]
    #[must_use]
    pub fn to_owned(&self) -> FsPath {
        match self {
            Self::File(p) => FsPath::File((*p).clone()),
            Self::Dir(p) => FsPath::Dir((*p).clone()),
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

    mod relative {
        use super::*;

        mod validation {
            use super::*;

            #[test]
            fn rejects_empty_path() {
                let result = RelativePath::try_from(PathBuf::from(""));
                assert!(result.is_err(), "expected error for empty path");
            }

            #[test]
            fn rejects_absolute_path() {
                let result = RelativePath::try_from(PathBuf::from("/abs"));
                assert!(result.is_err(), "expected error for absolute path");
            }

            #[test]
            fn rejects_parent_traversal_component() {
                let result = RelativePath::try_from(PathBuf::from("a/../b"));
                assert!(result.is_err(), "expected parent traversal error");
            }

            #[test]
            fn rejects_current_dir_component() {
                let result = RelativePath::try_from(PathBuf::from("a/./b"));
                assert!(
                    result.is_err(),
                    "expected current-dir component error"
                );
            }

            #[test]
            fn accepts_valid_relative_path() {
                let path = RelativePath::try_from(PathBuf::from("a/b/c"))
                    .expect("expected valid relative path");
                assert_eq!(path.as_path(), Path::new("a/b/c"));
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_filename_when_present() {
                let path =
                    RelativePath::try_from(PathBuf::from("a/b/file.txt"))
                        .expect("path should be valid");
                let filename = path.filename().expect("filename should exist");
                assert_eq!(filename.as_str(), "file.txt");
            }

            #[test]
            fn returns_filename_from_try_filename_when_present() {
                let path =
                    RelativePath::try_from(PathBuf::from("a/b/file.txt"))
                        .expect("path should be valid");
                let filename = path
                    .try_filename()
                    .expect("try_filename should succeed")
                    .expect("filename should exist");
                assert_eq!(filename.as_str(), "file.txt");
            }

            #[cfg(unix)]
            #[test]
            fn returns_invalid_data_when_try_filename_is_non_utf8() {
                use std::os::unix::ffi::OsStringExt as _;

                let path = PathBuf::from("a")
                    .join(std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]));

                let relative =
                    RelativePath::try_from(path).expect("path should be valid");
                let error = relative
                    .try_filename()
                    .expect_err("expected invalid utf-8 filename error");
                assert!(matches!(error, crate::fs::PathError::InvalidUtf8(_)));
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn accepts_try_from_str_when_valid() {
                let path = RelativePath::try_from("a/b/file.txt")
                    .expect("string path should convert");
                assert_eq!(path.as_path(), Path::new("a/b/file.txt"));
            }
        }
    }

    mod normalized {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn accepts_forward_slashes_when_valid() {
                let path = NormalizedPath::try_new("notes/daily/today.md")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily/today.md");
            }

            #[test]
            fn normalizes_backslashes_to_forward_slashes() {
                let path = NormalizedPath::try_new("notes\\daily\\today.md")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily/today.md");
            }
        }

        mod validation {
            use super::*;
            use crate::fs::PathError;

            #[test]
            fn rejects_parent_traversal_component() {
                let path = NormalizedPath::try_new("../outside.md");
                assert!(matches!(path, Err(PathError::ParentTraversal(_))));
            }

            #[test]
            fn rejects_current_dir_component() {
                let path = NormalizedPath::try_new("./notes/file.md");
                assert!(matches!(path, Err(PathError::CurrentDirComponent(_))));
            }

            #[test]
            fn rejects_empty_string() {
                let path = NormalizedPath::try_new("");
                assert!(matches!(path, Err(PathError::Empty)));
            }

            #[test]
            fn rejects_absolute_path() {
                let path = NormalizedPath::try_new("/usr/local/file.md");
                assert!(matches!(path, Err(PathError::NotRelative(_))));
            }
        }

        mod serialization {
            use super::*;

            #[test]
            fn preserves_value_across_rkyv_roundtrip() {
                let original = NormalizedPath::try_new("notes/test.md")
                    .expect("valid path");
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
                    .expect("serialize");
                let archived = rkyv::access::<
                    ArchivedNormalizedPath,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("archive access");
                let deserialized: NormalizedPath = rkyv::deserialize::<
                    NormalizedPath,
                    rkyv::rancor::Error,
                >(archived)
                .expect("deserialize");
                assert_eq!(original, deserialized);
            }
        }
    }

    mod absolute {
        use super::*;

        mod validation {
            use super::*;

            #[test]
            fn rejects_empty_path() {
                let result = AbsolutePath::try_from(PathBuf::from(""));
                assert!(result.is_err(), "expected error for empty path");
            }

            #[test]
            fn rejects_relative_path() {
                let result = AbsolutePath::try_from(PathBuf::from("rel"));
                assert!(result.is_err(), "expected error for relative path");
            }

            #[test]
            fn accepts_valid_absolute_path() {
                let result = AbsolutePath::try_from(PathBuf::from("/a/b/c"));
                assert!(result.is_ok(), "expected valid absolute path");
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_filename_when_present() {
                let path =
                    AbsolutePath::try_from(PathBuf::from("/a/b/file.txt"))
                        .expect("path should be valid");
                let filename = path.filename().expect("filename should exist");
                assert_eq!(filename.as_str(), "file.txt");
            }

            #[test]
            fn returns_filename_from_try_filename_when_present() {
                let path =
                    AbsolutePath::try_from(PathBuf::from("/a/b/file.txt"))
                        .expect("path should be valid");
                let filename = path
                    .try_filename()
                    .expect("try_filename should succeed")
                    .expect("filename should exist");
                assert_eq!(filename.as_str(), "file.txt");
            }

            #[cfg(unix)]
            #[test]
            fn returns_invalid_data_when_try_filename_is_non_utf8() {
                use std::os::unix::ffi::OsStringExt as _;

                let path = PathBuf::from("/tmp")
                    .join(std::ffi::OsString::from_vec(vec![0x66, 0x6f, 0x80]));

                let absolute =
                    AbsolutePath::try_from(path).expect("path should be valid");
                let error = absolute
                    .try_filename()
                    .expect_err("expected invalid utf-8 filename error");
                assert!(matches!(error, crate::fs::PathError::InvalidUtf8(_)));
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn accepts_try_from_str_when_valid_absolute() {
                let path = AbsolutePath::try_from("/a/b/file.txt")
                    .expect("string path should convert");
                assert_eq!(path.as_path(), Path::new("/a/b/file.txt"));
            }
        }
    }

    mod file {
        use tempfile::NamedTempFile;

        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn wraps_pathbuf_with_try_from() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_from(temp.path().to_path_buf())
                    .expect("file path should convert");
                assert_eq!(path.as_path(), temp.path());
            }

            #[test]
            fn rejects_empty_path() {
                assert!(FilePath::try_new(PathBuf::from("")).is_err());
            }

            #[test]
            fn rejects_non_file_path() {
                assert!(
                    FilePath::try_new(PathBuf::from("/nonexistent/file.txt"))
                        .is_err()
                );
            }

            #[test]
            fn accepts_existing_absolute_file_path() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                assert_eq!(path.as_path(), temp.path());
            }
        }

        mod lookup {
            use super::*;

            #[test]
            fn returns_relative_path_when_within_base() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let temp = NamedTempFile::new_in(temp_dir.path())
                    .expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                let relative = path
                    .as_relative(temp_dir.path())
                    .expect("path should be within base");
                assert_eq!(
                    relative.as_path(),
                    Path::new(
                        temp.path()
                            .file_name()
                            .expect("file name should exist")
                    )
                );
            }

            #[test]
            fn returns_error_when_outside_base() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                assert!(path.as_relative(Path::new("/other")).is_err());
            }

            #[test]
            fn returns_filename_when_present() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                let filename =
                    path.filename().expect("filename should be present");
                assert_eq!(
                    filename.as_str(),
                    temp.path()
                        .file_name()
                        .expect("file name should exist")
                        .to_str()
                        .expect("utf-8 file name expected")
                );
            }

            #[test]
            fn returns_basename_when_present() {
                let temp = NamedTempFile::new_in(std::env::temp_dir())
                    .expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                assert!(path.basename().is_some());
            }

            #[test]
            fn returns_extension_when_present() {
                let temp_path = std::env::temp_dir().join("test.md");
                std::fs::write(&temp_path, "content")
                    .expect("temp file should be writable");
                let path = FilePath::try_new(temp_path.clone())
                    .expect("file path should be valid");
                assert!(path.has_extension());
                assert!(path.extension_ref().is_some());
                std::fs::remove_file(&temp_path).ok();
            }

            #[test]
            fn returns_parent_directory_view() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let temp = NamedTempFile::new_in(temp_dir.path())
                    .expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                assert!(matches!(path.parent(), ParentDir::Path(_)));
                if let ParentDir::Path(parent) = path.parent() {
                    assert_eq!(parent, temp_dir.path());
                }
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_true_for_is_absolute_on_absolute_path() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                assert!(path.is_absolute());
                assert!(!path.is_relative());
            }

            #[test]
            fn returns_true_for_relative_status_on_relative_example_path() {
                let rel_path = Path::new("notes/file.md");
                assert!(rel_path.is_relative());
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn converts_into_pathbuf_preserving_value() {
                let temp =
                    NamedTempFile::new().expect("temp file should be created");
                let path = FilePath::try_new(temp.path().to_path_buf())
                    .expect("file path should be valid");
                let buf: PathBuf = path.into();
                assert_eq!(buf, temp.path().to_path_buf());
            }
        }
    }

    mod dir {
        use tempfile::TempDir;

        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn wraps_pathbuf_with_new() {
                let temp = TempDir::new().expect("temp dir should be created");
                let path = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                assert_eq!(path.as_path(), temp.path());
            }

            #[test]
            fn rejects_empty_path() {
                assert!(DirPath::try_new(PathBuf::from("")).is_err());
            }

            #[test]
            fn rejects_non_directory_path() {
                assert!(
                    DirPath::try_new(PathBuf::from("/nonexistent/dir"))
                        .is_err()
                );
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_true_for_is_absolute_on_absolute_path() {
                let temp = TempDir::new().expect("temp dir should be created");
                let path = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                assert!(path.is_absolute());
                assert!(!path.is_relative());
            }

            #[test]
            fn returns_dirname_when_present() {
                let temp = TempDir::new().expect("temp dir should be created");
                let path = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                assert!(path.dirname().is_some());
            }

            #[test]
            fn returns_parent_directory_view() {
                let temp = TempDir::new().expect("temp dir should be created");
                let path = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                assert!(matches!(path.parent(), ParentDir::Path(_)));
            }
        }

        mod normalization {
            use super::*;

            #[test]
            fn joins_child_path_preserving_structure() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let joined = dir.join("subdir/file.txt");
                assert!(joined.to_string_lossy().ends_with("subdir/file.txt"));
            }

            #[test]
            fn joins_child_file_path() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let child = temp.path().join("notes.txt");
                std::fs::write(&child, "content")
                    .expect("child file should be writable");
                let file = dir
                    .join_path("notes.txt")
                    .as_file()
                    .expect("joined path should be file")
                    .clone();
                assert!(
                    file.as_path().to_string_lossy().ends_with("notes.txt")
                );
            }

            #[test]
            fn joins_child_directory_path() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                std::fs::create_dir_all(temp.path().join("notes"))
                    .expect("child directory should be created");
                let subdir = dir
                    .join_path("notes")
                    .as_dir()
                    .expect("joined path should be directory")
                    .clone();
                assert!(subdir.as_path().to_string_lossy().ends_with("notes"));
            }

            #[test]
            fn joins_child_path_as_fs_path_file_variant() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                std::fs::write(temp.path().join("child.txt"), "content")
                    .expect("child file should be writable");

                let joined = dir.join_path("child.txt");
                assert!(joined.is_file());
            }

            #[test]
            fn joins_child_path_as_fs_path_dir_variant() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                std::fs::create_dir_all(temp.path().join("child"))
                    .expect("child directory should be created");

                let joined = dir.join_path("child");
                assert!(joined.is_dir());
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn converts_into_pathbuf_preserving_value() {
                let temp = TempDir::new().expect("temp dir should be created");
                let path = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let buf: PathBuf = path.into();
                assert_eq!(buf, temp.path().to_path_buf());
            }
        }
    }

    mod fs_path {
        use tempfile::{NamedTempFile, TempDir};

        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_variant_flags_for_file_and_directory() {
                let temp_dir =
                    TempDir::new().expect("temp dir should be created");
                let temp_file = NamedTempFile::new_in(temp_dir.path())
                    .expect("temp file should be created");

                let file = FilePath::try_new(temp_file.path().to_path_buf())
                    .expect("file path should be valid");
                let dir = DirPath::try_new(temp_dir.path().to_path_buf())
                    .expect("dir path should be valid");

                let fs_file = FsPath::File(file);
                let fs_dir = FsPath::Dir(dir);

                assert!(fs_file.is_file());
                assert!(!fs_file.is_dir());
                assert!(fs_dir.is_dir());
                assert!(!fs_dir.is_file());
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn returns_relative_path_when_within_base_for_file_variant() {
                let temp_dir =
                    TempDir::new().expect("temp dir should be created");
                let temp_file = NamedTempFile::new_in(temp_dir.path())
                    .expect("temp file should be created");

                let file = FilePath::try_new(temp_file.path().to_path_buf())
                    .expect("file path should be valid");
                let fs_file = FsPath::File(file);

                let rel_file = fs_file
                    .as_relative(temp_dir.path())
                    .expect("relative conversion should succeed");
                assert_eq!(
                    rel_file.as_path(),
                    temp_file
                        .path()
                        .file_name()
                        .expect("file name should exist")
                );
            }

            #[test]
            fn converts_walkdir_file_entry_to_file_variant() {
                let temp_dir =
                    TempDir::new().expect("temp dir should be created");
                let file_path = temp_dir.path().join("note.md");
                std::fs::write(&file_path, "content")
                    .expect("test file should be writable");

                let entry = walkdir::WalkDir::new(temp_dir.path())
                    .min_depth(1)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                    .find(|entry| entry.path() == file_path.as_path())
                    .expect("expected file entry");

                let fs_path =
                    FsPath::try_from(entry).expect("conversion should succeed");
                assert!(fs_path.is_file());
            }

            #[test]
            fn converts_walkdir_directory_entry_to_dir_variant() {
                let temp_dir =
                    TempDir::new().expect("temp dir should be created");
                let subdir_path = temp_dir.path().join("notes");
                std::fs::create_dir_all(&subdir_path)
                    .expect("test directory should be created");

                let entry = walkdir::WalkDir::new(temp_dir.path())
                    .min_depth(1)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                    .find(|entry| entry.path() == subdir_path.as_path())
                    .expect("expected directory entry");

                let fs_path =
                    FsPath::try_from(entry).expect("conversion should succeed");
                assert!(fs_path.is_dir());
            }
        }
    }

    mod parent_dir {
        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_parent_path_when_parent_exists() {
                let path = Path::new("a/b/c");
                let parent = ParentDir::from_path(path);
                assert!(matches!(parent, ParentDir::Path(_)));
                if let ParentDir::Path(value) = parent {
                    assert_eq!(value, Path::new("a/b"));
                }
            }

            #[test]
            fn returns_root_when_no_parent_exists() {
                let root_parent = ParentDir::from_path(Path::new("file.txt"));
                assert!(matches!(root_parent, ParentDir::Root));
            }
        }
    }
}
