//! Three-tier path taxonomy for the Lithos core library.
//!
//! This module provides a hierarchy of type-safe path wrappers that enforce
//! filesystem invariants and security policies. The path system is organized
//! into three distinct tiers based on their purpose and validation rules.
//!
//! # Path Taxonomy
//!
//! | Tier                 | Types                                  | Where                                     | Example                               |
//! | -------------------- | -------------------------------------- | ----------------------------------------- | ------------------------------------- |
//! | **Filesystem I/O**   | [`FsPath`], [`FilePath`], [`DirPath`]  | Scanner, reader, writer, vault processor  | `DirPath::append_file(&rel_file)`     |
//! | **Display / Config** | [`RelativePath`] enum, `Relative*Path` | CLI display, config values, serialization | `RelativeDirPath::try_new("schemas")` |
//! | **Storage Keys**     | [`PathKey`]                            | Repository traits, DB tables              | `fn find_file_view_by_path(&PathKey)` |
//!
//! # Type Choice Guidance
//!
//! - **Use FS I/O types** when performing actual filesystem operations (read,
//!   write, metadata).
//! - **Use Display/Config types** for values coming from configuration files,
//!   CLI arguments, or when sending paths to a UI. The [`RelativePath`] enum
//!   specifically is intended for display and serialization — it is NOT for
//!   filesystem I/O, and NOT for storage keys.
//! - **Use Storage Keys** for database indices, serialized metadata, or
//!   cross-platform identifiers.
//!
//! # Conversion Seams
//!
//! | Source → Target        | Method                                                            | Fallible? |
//! | ---------------------- | ----------------------------------------------------------------- | --------- |
//! | Config value → FS path | [`DirPath::append_dir(&RelativeDirPath)`][DirPath::append_dir]    | Yes       |
//! | Config value → FS path | [`DirPath::append_file(&RelativeFilePath)`][DirPath::append_file] | Yes       |
//! | FS path → Storage key  | `file_path.as_key(root)` / `dir_path.as_key(root)`                | Yes       |
//! | FS path → Display      | `file_path.as_relative(base) → RelativePath::File(...)`           | Yes       |
//! | FS path → Display      | `dir_path.as_relative(base) → RelativePath::Dir(...)`             | Yes       |
//!
//! # Examples
//!
//! ```rust
//! # use lithos_core::fs::PathKey;
//! // Normalizing a string for storage
//! let key = PathKey::try_new("notes\\daily\\today.md").unwrap();
//! assert_eq!(key.as_str(), "notes/daily/today.md");
//!
//! // Validating a storage key
//! let key = PathKey::try_new("data/config.json");
//! assert!(key.is_ok());
//! ```

#![expect(
    clippy::module_name_repetitions,
    reason = "RelativePath is a canonical name"
)]

use std::path::{Component, Path, PathBuf};

use rkyv::{Archive, Deserialize, Serialize, with::AsString};

use super::{
    format::FileExtensionRef,
    name::{BaseName, BaseNameRef, DirName, DirNameRef, FileName, FileNameRef},
};

/// Unified path enum representing either a file or a directory.
///
/// This type is used when an operation can handle both files and directories,
/// such as walking a directory tree or representing a general filesystem node.
/// It wraps the more specific [`FilePath`] and [`DirPath`] types.
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
    /// Returns the underlying path as a standard library [`Path`].
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

    /// Returns the file path if this is a file, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn as_file(&self) -> Option<&FilePath> {
        match self {
            Self::File(p) => Some(p),
            Self::Dir(_) => None,
        }
    }

    /// Returns the directory path if this is a directory, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn as_dir(&self) -> Option<&DirPath> {
        match self {
            Self::Dir(p) => Some(p),
            Self::File(_) => None,
        }
    }

    /// Convert to vault-relative path by stripping the base prefix.
    ///
    /// # Errors
    ///
    /// Returns [`FsError::Read(ReadError::RootScope)`] if the path is not
    /// within the base directory boundary.
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

    /// Converts this filesystem path to a root-scoped persistence key.
    ///
    /// # Errors
    ///
    /// Returns [`PathError::RootScope`] if this path is outside `root`,
    /// [`PathError::InvalidUtf8`] if the path contains non-UTF8 characters,
    /// or other [`PathError`] variants if validation fails.
    #[inline]
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, super::PathError> {
        match self {
            Self::File(path) => path.as_key(root),
            Self::Dir(path) => path.as_key(root),
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

/// A validated file path that is guaranteed to exist and be a file.
///
/// This type represents a rooted path on the filesystem. Use
/// [`as_relative`](Self::as_relative) to convert it to a vault-relative path
/// for storage, or [`as_key`](Self::as_key) to create a persistence key.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct FilePath(#[rkyv(with = AsString)] PathBuf);

impl FilePath {
    /// Create a new file path from a [`PathBuf`].
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Empty`] if the path is empty, or
    /// [`PathError::NotAFile`] if the path does not exist or refers to a
    /// directory.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, super::PathError> {
        Self::validate(path.as_path())?;
        Ok(Self(path))
    }

    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if !path.is_file() {
            return Err(super::PathError::NotAFile(path.to_path_buf()));
        }
        Ok(())
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

        let rel = self.0.strip_prefix(base).map_err(|_| {
            ReadError::RootScope(
                super::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path: self.0.clone(),
                    root: base.to_path_buf(),
                },
            )
        })?;

        let rel_str = rel.to_str().ok_or_else(|| {
            ReadError::RootScope(
                super::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path: self.0.clone(),
                    root: base.to_path_buf(),
                },
            )
        })?;

        Ok(RelativePath::File(
            RelativeFilePath::try_new(rel_str).map_err(FsError::from)?,
        ))
    }

    /// Converts this file path to a root-scoped persistence key.
    ///
    /// # Errors
    /// Returns an error if this path is outside `root`, not valid UTF-8 after
    /// root stripping, or fails `PathKey` validation.
    #[inline]
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, super::PathError> {
        PathKey::from_rooted_path(root, self.as_path())
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

/// A validated directory path that is guaranteed to exist and be a directory.
///
/// This type represents a rooted path on the filesystem. Use
/// [`as_relative`](Self::as_relative) to convert it to a vault-relative path
/// for storage, or [`as_key`](Self::as_key) to create a persistence key.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct DirPath(#[rkyv(with = AsString)] PathBuf);

impl DirPath {
    /// Create a new directory path from a [`PathBuf`].
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Empty`] if the path is empty, or
    /// [`PathError::NotADirectory`] if the path does not exist or refers to a
    /// file.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, super::PathError> {
        Self::validate(path.as_path())?;
        Ok(Self(path))
    }

    fn validate(path: &Path) -> Result<(), super::PathError> {
        if path.as_os_str().is_empty() {
            return Err(super::PathError::Empty);
        }
        if !path.is_dir() {
            return Err(super::PathError::NotADirectory(path.to_path_buf()));
        }
        Ok(())
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

        let rel = self.0.strip_prefix(base).map_err(|_| {
            ReadError::RootScope(
                super::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path: self.0.clone(),
                    root: base.to_path_buf(),
                },
            )
        })?;

        let rel_str = rel.to_str().ok_or_else(|| {
            ReadError::RootScope(
                super::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path: self.0.clone(),
                    root: base.to_path_buf(),
                },
            )
        })?;

        Ok(RelativePath::Dir(
            RelativeDirPath::try_new(rel_str).map_err(FsError::from)?,
        ))
    }

    /// Converts this directory path to a root-scoped persistence key.
    ///
    /// # Errors
    /// Returns an error if this path is outside `root`, not valid UTF-8 after
    /// root stripping, or fails `PathKey` validation.
    #[inline]
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, super::PathError> {
        PathKey::from_rooted_path(root, self.as_path())
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

    /// Appends a file fragment to this directory path.
    ///
    /// The fragment can be a [`FileName`] or a [`RelativeFilePath`].
    ///
    /// # Errors
    ///
    /// Returns [`PathError::NotAFile`] if the resulting path does not exist or
    /// refers to a directory.
    #[inline]
    pub fn append_file<T: FileFragment>(
        &self,
        file: &T,
    ) -> Result<FilePath, super::PathError> {
        FilePath::try_new(self.join(Path::new(file.as_str())))
    }

    /// Appends a directory fragment to this directory path.
    ///
    /// The fragment can be a [`DirName`] or a [`RelativeDirPath`].
    ///
    /// # Errors
    ///
    /// Returns [`PathError::NotADirectory`] if the resulting path does not
    /// exist or refers to a file.
    #[inline]
    pub fn append_dir<T: DirFragment>(
        &self,
        dir: &T,
    ) -> Result<DirPath, super::PathError> {
        DirPath::try_new(self.join(dir.as_str()))
    }

    /// Appends a validated filename to this directory path.
    ///
    /// # Errors
    /// Returns error if the resulting path does not refer to an existing file.
    #[inline]
    pub fn append_filename(
        &self,
        file: &FileName,
    ) -> Result<FilePath, super::PathError> {
        self.append_file(file)
    }

    /// Appends a validated directory name to this directory path.
    ///
    /// # Errors
    /// Returns error if the resulting path does not refer to an existing
    /// directory.
    #[inline]
    pub fn append_dirname(
        &self,
        dir: &DirName,
    ) -> Result<DirPath, super::PathError> {
        self.append_dir(dir)
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

/// Unified view of a declarative relative path.
///
/// This enum is intended for **Display** and **Configuration** purposes. It
/// provides a platform-agnostic way to represent paths before they are resolved
/// against a filesystem root.
///
/// For filesystem operations, use [`FsPath`]. For storage, use [`PathKey`].
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub enum RelativePath {
    /// A relative path to a file.
    File(RelativeFilePath),
    /// A relative path to a directory.
    Dir(RelativeDirPath),
}

impl RelativePath {
    /// Return the underlying path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        match self {
            Self::File(f) => Path::new(f.as_str()),
            Self::Dir(d) => Path::new(d.as_str()),
        }
    }

    /// Returns the underlying path as a UTF-8 string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::File(f) => f.as_str(),
            Self::Dir(d) => d.as_str(),
        }
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
        match self {
            Self::File(f) => Path::new(f.0.as_ref()),
            Self::Dir(d) => Path::new(d.0.as_ref()),
        }
    }
}

impl std::fmt::Display for RelativePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(p) => f.write_str(p.as_str()),
            Self::Dir(p) => f.write_str(p.as_str()),
        }
    }
}

/// A validated vault-relative file path.
///
/// This type ensures the path is relative, does not contain traversal
/// components (`..`), and is not absolute. It preserves platform-specific
/// separators.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct RelativeFilePath(Box<str>);

impl RelativeFilePath {
    /// Creates a new validated relative file path.
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Empty`], [`PathError::NotRelative`],
    /// [`PathError::ParentTraversal`], or [`PathError::CurrentDirComponent`]
    /// if the path fails validation.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, super::PathError> {
        let validated = RelativePathValidator::validate(path)?;
        Ok(Self(validated.to_owned().into_boxed_str()))
    }

    /// Returns the validated declaration string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for RelativeFilePath {
    type Error = super::PathError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// A validated vault-relative directory path.
///
/// This type ensures the path is relative, does not contain traversal
/// components (`..`), and is not absolute. It preserves platform-specific
/// separators.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct RelativeDirPath(Box<str>);

impl RelativeDirPath {
    /// Creates a new validated relative directory path.
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Empty`], [`PathError::NotRelative`],
    /// [`PathError::ParentTraversal`], or [`PathError::CurrentDirComponent`]
    /// if the path fails validation.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, super::PathError> {
        let validated = RelativePathValidator::validate(path)?;
        Ok(Self(validated.to_owned().into_boxed_str()))
    }

    /// Returns the validated declaration string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for RelativeDirPath {
    type Error = super::PathError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
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
/// - [`PathKey`]: Forward slashes, `Box<str>` storage, `as_str() -> &str`
/// - [`RelativePath`]: Platform slashes, `PathBuf` storage, `as_path() ->
///   &Path`
///
/// Use [`RelativePath`] for filesystem operations; use [`PathKey`]
/// for storage and serialization.
///
/// # Examples
///
/// ```
/// # use lithos_core::fs::PathKey;
/// // Normalizes backslashes to forward slashes
/// let key = PathKey::try_new("notes\\daily\\today.md").unwrap();
/// assert_eq!(key.as_str(), "notes/daily/today.md");
///
/// // Removes duplicate slashes
/// let key = PathKey::try_new("notes///daily//today.md").unwrap();
/// assert_eq!(key.as_str(), "notes/daily/today.md");
///
/// // Removes trailing slashes
/// let key = PathKey::try_new("notes/daily/").unwrap();
/// assert_eq!(key.as_str(), "notes/daily");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct PathKey(Box<str>);

impl PathKey {
    /// Creates a new normalized vault-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Empty`], [`PathError::NotRelative`],
    /// [`PathError::ParentTraversal`], or [`PathError::CurrentDirComponent`]
    /// if the path fails validation.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, super::PathError> {
        // 1. Validate the raw input first.
        RelativePathValidator::validate(path)?;

        // 2. Normalize the safe-but-potentially-messy input.
        let normalized = Self::normalize(path.trim());

        // 3. Final safety check on the canonical form.
        RelativePathValidator::validate(normalized.as_ref())?;

        Ok(Self(normalized.into_owned().into_boxed_str()))
    }

    /// Converts an absolute or rooted filesystem path into a `PathKey` scoped
    /// to `root`.
    ///
    /// # Errors
    /// Returns a root-scope error when `path` is not within `root`,
    /// [`PathError::InvalidUtf8`] when the relative slice is not UTF-8, or any
    /// validation error produced by [`PathKey::try_new`].
    #[inline]
    pub fn from_rooted_path(
        root: &DirPath,
        path: &Path,
    ) -> Result<Self, super::PathError> {
        let relative = path.strip_prefix(root.as_path()).map_err(|_| {
            super::error::RootScopeError::PathOutsideVaultRootBoundary {
                root: root.as_path().to_path_buf(),
                path: path.to_path_buf(),
            }
        })?;
        let utf8 = relative
            .to_str()
            .ok_or_else(|| super::PathError::InvalidUtf8(path.to_path_buf()))?;
        Self::try_new(utf8)
    }

    #[inline]
    fn normalize(path: &str) -> std::borrow::Cow<'_, str> {
        let ctx = Self::collect_normalization_context(path);
        if !ctx.has_backslash()
            && !ctx.has_duplicate_separators()
            && !ctx.has_trailing_separator()
        {
            return std::borrow::Cow::Borrowed(path);
        }

        let canonicalized = Self::apply_separator_canonicalization(path);
        std::borrow::Cow::Owned(Self::apply_trailing_separator_policy(
            canonicalized,
        ))
    }

    #[inline]
    fn collect_normalization_context(path: &str) -> PathNormalizationContext {
        let mut context = PathNormalizationContext::new();

        if path.contains('\\') {
            context.set(PathNormalizationContext::HAS_BACKSLASH);
        }
        if path.as_bytes().windows(2).any(|window| {
            matches!(
                window,
                [a, b]
                    if (*a == b'/' || *a == b'\\')
                        && (*b == b'/' || *b == b'\\')
            )
        }) {
            context.set(PathNormalizationContext::HAS_DUPLICATE_SEPARATORS);
        }
        if path.len() > 1 && (path.ends_with('/') || path.ends_with('\\')) {
            context.set(PathNormalizationContext::HAS_TRAILING_SEPARATOR);
        }

        context
    }

    fn apply_separator_canonicalization(path: &str) -> String {
        let mut owned = String::with_capacity(path.len());
        let mut prev_was_separator = false;

        for ch in path.chars() {
            let is_separator = ch == '/' || ch == '\\';
            if is_separator {
                if !prev_was_separator {
                    owned.push('/');
                    prev_was_separator = true;
                }
            } else {
                owned.push(ch);
                prev_was_separator = false;
            }
        }

        owned
    }

    fn apply_trailing_separator_policy(mut path: String) -> String {
        if path.len() > 1 && path.ends_with('/') {
            path.pop();
        }
        path
    }

    /// Returns the normalized path string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PathKey {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

/// A trait for types that can be appended to a directory to form a file path.
///
/// This trait is implemented by types that represent a single filename or a
/// relative path to a file, allowing them to be used with
/// [`DirPath::append_file`].
///
/// Implementors: [`FileName`], [`RelativeFilePath`].
pub trait FileFragment {
    /// Returns the fragment as a string slice.
    fn as_str(&self) -> &str;
}

impl FileFragment for FileName {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl FileFragment for RelativeFilePath {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

/// A trait for types that can be appended to a directory to form a directory
/// path.
///
/// This trait is implemented by types that represent a single directory name or
/// a relative path to a directory, allowing them to be used with
/// [`DirPath::append_dir`].
///
/// Implementors: [`DirName`], [`RelativeDirPath`].
pub trait DirFragment {
    /// Returns the fragment as a string slice.
    fn as_str(&self) -> &str;
}

impl DirFragment for DirName {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl DirFragment for RelativeDirPath {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

struct RelativePathValidator;

impl RelativePathValidator {
    fn validate(path: &str) -> Result<&str, super::PathError> {
        let trimmed = path.trim();
        Self::reject_empty(trimmed)?;

        let ctx = PathValidationContext::analyze(trimmed);
        Self::reject_absolute(trimmed, ctx)?;
        Self::reject_parent_traversal(trimmed, ctx)?;
        Self::reject_current_dir_component(trimmed, ctx)?;
        Self::reject_platform_prefix(trimmed, ctx)?;

        Ok(trimmed)
    }

    fn reject_empty(path: &str) -> Result<(), super::PathError> {
        if path.is_empty() {
            return Err(super::PathError::Empty);
        }
        Ok(())
    }

    fn reject_absolute(
        path: &str,
        ctx: PathValidationContext,
    ) -> Result<(), super::PathError> {
        if ctx.is_absolute() {
            return Err(super::PathError::NotRelative(PathBuf::from(path)));
        }
        Ok(())
    }

    fn reject_parent_traversal(
        path: &str,
        ctx: PathValidationContext,
    ) -> Result<(), super::PathError> {
        if ctx.has_parent_traversal() {
            return Err(super::PathError::ParentTraversal(PathBuf::from(path)));
        }
        Ok(())
    }

    fn reject_current_dir_component(
        path: &str,
        ctx: PathValidationContext,
    ) -> Result<(), super::PathError> {
        if ctx.has_current_dir_component() {
            return Err(super::PathError::CurrentDirComponent(PathBuf::from(
                path,
            )));
        }
        Ok(())
    }

    fn reject_platform_prefix(
        path: &str,
        ctx: PathValidationContext,
    ) -> Result<(), super::PathError> {
        if ctx.has_platform_prefix() {
            return Err(super::PathError::PlatformPrefix(PathBuf::from(path)));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct PathValidationContext {
    flags: u8,
}

impl PathValidationContext {
    const HAS_CURRENT_DIR_COMPONENT: u8 = 1 << 2;
    const HAS_PARENT_TRAVERSAL: u8 = 1 << 1;
    const HAS_PLATFORM_PREFIX: u8 = 1 << 3;
    const IS_ABSOLUTE: u8 = 1 << 0;

    fn new() -> Self {
        Self {
            flags: 0,
        }
    }

    fn analyze(path: &str) -> Self {
        let path_buf = PathBuf::from(path);
        let mut context = Self::new();

        if path_buf.is_absolute() {
            context.set(Self::IS_ABSOLUTE);
        }

        for component in path_buf.components() {
            match component {
                Component::ParentDir => {
                    context.set(Self::HAS_PARENT_TRAVERSAL);
                }
                Component::CurDir => {
                    context.set(Self::HAS_CURRENT_DIR_COMPONENT);
                }
                Component::Prefix(_) => {
                    context.set(Self::HAS_PLATFORM_PREFIX);
                }
                Component::RootDir | Component::Normal(_) => {}
            }
        }

        if path.split('/').any(|segment| segment == ".") {
            context.set(Self::HAS_CURRENT_DIR_COMPONENT);
        }

        context
    }

    fn set(&mut self, flag: u8) {
        self.flags |= flag;
    }

    fn is_absolute(self) -> bool {
        self.flags & Self::IS_ABSOLUTE != 0
    }

    fn has_parent_traversal(self) -> bool {
        self.flags & Self::HAS_PARENT_TRAVERSAL != 0
    }

    fn has_current_dir_component(self) -> bool {
        self.flags & Self::HAS_CURRENT_DIR_COMPONENT != 0
    }

    fn has_platform_prefix(self) -> bool {
        self.flags & Self::HAS_PLATFORM_PREFIX != 0
    }
}

#[derive(Debug, Clone, Copy)]
struct PathNormalizationContext {
    flags: u8,
}

impl PathNormalizationContext {
    const HAS_BACKSLASH: u8 = 1 << 0;
    const HAS_DUPLICATE_SEPARATORS: u8 = 1 << 1;
    const HAS_TRAILING_SEPARATOR: u8 = 1 << 2;

    fn new() -> Self {
        Self {
            flags: 0,
        }
    }

    fn set(&mut self, flag: u8) {
        self.flags |= flag;
    }

    fn has_backslash(self) -> bool {
        self.flags & Self::HAS_BACKSLASH != 0
    }

    fn has_duplicate_separators(self) -> bool {
        self.flags & Self::HAS_DUPLICATE_SEPARATORS != 0
    }

    fn has_trailing_separator(self) -> bool {
        self.flags & Self::HAS_TRAILING_SEPARATOR != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod relative_path {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_file_variant_when_constructed_directly() {
                let path = RelativePath::File(
                    RelativeFilePath::try_new("notes/file.md")
                        .expect("valid file path"),
                );
                assert_eq!(path.as_str(), "notes/file.md");
            }

            #[test]
            fn returns_dir_variant_when_constructed_directly() {
                let path = RelativePath::Dir(
                    RelativeDirPath::try_new("notes/dir")
                        .expect("valid dir path"),
                );
                assert_eq!(path.as_str(), "notes/dir");
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn as_path_returns_path_for_file_variant() {
                let path = RelativePath::File(
                    RelativeFilePath::try_new("notes/file.md")
                        .expect("valid file path"),
                );
                assert_eq!(path.as_path(), Path::new("notes/file.md"));
            }

            #[test]
            fn as_path_returns_path_for_dir_variant() {
                let path = RelativePath::Dir(
                    RelativeDirPath::try_new("notes/dir")
                        .expect("valid dir path"),
                );
                assert_eq!(path.as_path(), Path::new("notes/dir"));
            }

            #[test]
            fn as_str_returns_utf8_slice_for_file_variant() {
                let path = RelativePath::File(
                    RelativeFilePath::try_new("notes/file.md")
                        .expect("valid file path"),
                );
                assert_eq!(path.as_str(), "notes/file.md");
            }

            #[test]
            fn as_str_returns_utf8_slice_for_dir_variant() {
                let path = RelativePath::Dir(
                    RelativeDirPath::try_new("notes/dir")
                        .expect("valid dir path"),
                );
                assert_eq!(path.as_str(), "notes/dir");
            }

            #[test]
            fn display_matches_inner_string() {
                let path = RelativePath::File(
                    RelativeFilePath::try_new("notes/file.md")
                        .expect("valid file path"),
                );
                assert_eq!(path.to_string(), "notes/file.md");
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn from_file_path_as_relative_returns_file_variant() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let temp_file =
                    tempfile::NamedTempFile::new_in(temp_dir.path())
                        .expect("temp file should be created");
                let file = FilePath::try_new(temp_file.path().to_path_buf())
                    .expect("file path should be valid");
                let rel = file
                    .as_relative(temp_dir.path())
                    .expect("path should be within base");
                assert!(matches!(rel, RelativePath::File(_)));
            }

            #[test]
            fn from_dir_path_as_relative_returns_dir_variant() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let sub_dir = temp_dir.path().join("sub");
                std::fs::create_dir_all(&sub_dir)
                    .expect("sub dir should be created");
                let dir = DirPath::try_new(sub_dir.clone())
                    .expect("dir path should be valid");
                let rel = dir
                    .as_relative(temp_dir.path())
                    .expect("path should be within base");
                assert!(matches!(rel, RelativePath::Dir(_)));
            }

            #[test]
            fn from_fs_path_file_as_relative_returns_file_variant() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let temp_file =
                    tempfile::NamedTempFile::new_in(temp_dir.path())
                        .expect("temp file should be created");
                let file = FilePath::try_new(temp_file.path().to_path_buf())
                    .expect("file path should be valid");
                let fs_path = FsPath::File(file);
                let rel = fs_path
                    .as_relative(temp_dir.path())
                    .expect("path should be within base");
                assert!(matches!(rel, RelativePath::File(_)));
            }

            #[test]
            fn from_fs_path_dir_as_relative_returns_dir_variant() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let sub_dir = temp_dir.path().join("sub");
                std::fs::create_dir_all(&sub_dir)
                    .expect("sub dir should be created");
                let dir = DirPath::try_new(sub_dir.clone())
                    .expect("dir path should be valid");
                let fs_path = FsPath::Dir(dir);
                let rel = fs_path
                    .as_relative(temp_dir.path())
                    .expect("path should be within base");
                assert!(matches!(rel, RelativePath::Dir(_)));
            }

            #[test]
            fn returns_error_when_outside_base() {
                let temp_dir = tempfile::TempDir::new()
                    .expect("temp dir should be created");
                let temp_file =
                    tempfile::NamedTempFile::new_in(temp_dir.path())
                        .expect("temp file should be created");
                let file = FilePath::try_new(temp_file.path().to_path_buf())
                    .expect("file path should be valid");
                assert!(file.as_relative(Path::new("/other")).is_err());
            }
        }

        mod serialization {
            use super::*;

            #[test]
            fn preserves_value_across_rkyv_roundtrip() {
                let original = RelativePath::File(
                    RelativeFilePath::try_new("notes/test.md")
                        .expect("valid file path"),
                );
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
                    .expect("serialize");
                let archived = rkyv::access::<
                    ArchivedRelativePath,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("archive access");
                let deserialized: RelativePath = rkyv::deserialize::<
                    RelativePath,
                    rkyv::rancor::Error,
                >(archived)
                .expect("deserialize");
                assert_eq!(original, deserialized);
            }
        }
    }

    mod pathkey {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn accepts_canonical_paths_without_allocation() {
                let path = PathKey::try_new("notes/daily/today.md")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily/today.md");
            }
        }

        mod normalization {
            use super::*;

            #[test]
            fn normalizes_backslashes_to_forward_slashes() {
                let path = PathKey::try_new("notes\\daily\\today.md")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily/today.md");
            }

            #[test]
            fn normalizes_duplicate_slashes() {
                let path = PathKey::try_new("notes//daily///today.md")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily/today.md");
            }

            #[test]
            fn removes_trailing_slashes() {
                let path = PathKey::try_new("notes/daily/")
                    .expect("path should be valid");
                assert_eq!(path.as_str(), "notes/daily");
            }
        }

        mod validation {
            use super::*;
            use crate::fs::PathError;

            #[test]
            fn rejects_parent_traversals() {
                let path = PathKey::try_new("../outside.md");
                assert!(matches!(path, Err(PathError::ParentTraversal(_))));
            }

            #[test]
            fn rejects_current_dir_component() {
                let path = PathKey::try_new("./notes/file.md");
                assert!(matches!(path, Err(PathError::CurrentDirComponent(_))));
            }

            #[test]
            fn rejects_embedded_current_dir_component() {
                let path = PathKey::try_new("notes/./file.md");
                assert!(matches!(path, Err(PathError::CurrentDirComponent(_))));
            }

            #[test]
            fn rejects_messy_dangerous_path_early() {
                // This would be rejected anyway, but we want to ensure
                // validation happens on the raw input too.
                let path = PathKey::try_new("notes/../secret.md");
                assert!(matches!(path, Err(PathError::ParentTraversal(_))));
            }

            #[test]
            fn rejects_empty_paths() {
                let path = PathKey::try_new("");
                assert!(matches!(path, Err(PathError::Empty)));
            }

            #[test]
            fn rejects_absolute_paths() {
                let path = PathKey::try_new("/usr/local/file.md");
                assert!(matches!(path, Err(PathError::NotRelative(_))));
            }
        }

        mod conversions {
            use super::*;
            use crate::fs::PathError;

            #[test]
            fn returns_key_when_path_is_within_root() {
                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let file_path = root_dir.path().join("notes").join("daily.md");
                std::fs::create_dir_all(
                    file_path.parent().expect("file should have parent"),
                )
                .expect("create parent dirs");
                std::fs::write(&file_path, "# daily").expect("write file");

                let key = PathKey::from_rooted_path(&root, &file_path)
                    .expect("key should convert");
                assert_eq!(key.as_str(), "notes/daily.md");
            }

            #[test]
            fn returns_error_when_path_is_outside_root() {
                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let outside_dir = tempfile::TempDir::new().expect("temp dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let outside_file = outside_dir.path().join("outside.md");
                std::fs::write(&outside_file, "# outside").expect("write file");

                let error = PathKey::from_rooted_path(&root, &outside_file)
                    .expect_err("path outside root should fail");
                assert!(matches!(error, PathError::RootScope(_)));
            }

            #[test]
            fn returns_key_from_file_path_as_key_when_within_root() {
                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let file_path = root_dir.path().join("notes").join("story.md");
                std::fs::create_dir_all(
                    file_path.parent().expect("file should have parent"),
                )
                .expect("create parent dirs");
                std::fs::write(&file_path, "# story").expect("write file");
                let file = FilePath::try_new(file_path).expect("file path");

                let key = file.as_key(&root).expect("key should convert");
                assert_eq!(key.as_str(), "notes/story.md");
            }

            #[test]
            fn returns_key_from_dir_path_as_key_when_within_root() {
                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let notes_dir = root_dir.path().join("notes");
                std::fs::create_dir_all(&notes_dir).expect("create notes dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let dir = DirPath::try_new(notes_dir).expect("dir path");

                let key = dir.as_key(&root).expect("key should convert");
                assert_eq!(key.as_str(), "notes");
            }

            #[test]
            fn returns_key_from_fs_path_as_key_when_within_root() {
                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let file_path = root_dir.path().join("notes.md");
                std::fs::write(&file_path, "# note").expect("write file");
                let file = FilePath::try_new(file_path).expect("file path");
                let fs_path = FsPath::File(file);

                let key = fs_path.as_key(&root).expect("key should convert");
                assert_eq!(key.as_str(), "notes.md");
            }

            #[cfg(unix)]
            #[test]
            fn returns_invalid_utf8_when_relative_path_is_not_utf8() {
                use std::os::unix::ffi::OsStringExt as _;

                let root_dir = tempfile::TempDir::new().expect("temp dir");
                let root = DirPath::try_new(root_dir.path().to_path_buf())
                    .expect("root");
                let invalid = root_dir
                    .path()
                    .join(std::ffi::OsString::from_vec(vec![0x66, 0x80]));

                let error = PathKey::from_rooted_path(&root, &invalid)
                    .expect_err("invalid utf-8 should fail");
                assert!(matches!(error, PathError::InvalidUtf8(_)));
            }
        }

        mod serialization {
            use super::*;

            #[test]
            fn preserves_value_across_rkyv_roundtrip() {
                let original =
                    PathKey::try_new("notes/test.md").expect("valid path");
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
                    .expect("serialize");
                let archived = rkyv::access::<
                    ArchivedPathKey,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("archive access");
                let deserialized: PathKey =
                    rkyv::deserialize::<PathKey, rkyv::rancor::Error>(archived)
                        .expect("deserialize");
                assert_eq!(original, deserialized);
            }
        }
    }

    mod relative_config_path {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_ok_when_relative_dir_path_is_valid() {
                let result = RelativeDirPath::try_new("schemas/cache");
                assert!(result.is_ok(), "expected valid relative dir path");
            }

            #[test]
            fn returns_ok_when_relative_file_path_is_valid() {
                let result =
                    RelativeFilePath::try_new("schemas/property_bank.json");
                assert!(result.is_ok(), "expected valid relative file path");
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn rejects_path_when_absolute() {
                let dir_result = RelativeDirPath::try_new("/schemas");
                let file_result =
                    RelativeFilePath::try_new("/schemas/bank.json");
                assert!(dir_result.is_err());
                assert!(file_result.is_err());
            }

            #[test]
            fn rejects_path_when_contains_current_dir_component() {
                let dir_result = RelativeDirPath::try_new("schemas/./cache");
                let file_result =
                    RelativeFilePath::try_new("schemas/./property_bank.json");
                assert!(dir_result.is_err());
                assert!(file_result.is_err());
            }

            #[test]
            fn rejects_path_when_contains_parent_dir_component() {
                let dir_result = RelativeDirPath::try_new("schemas/../cache");
                let file_result =
                    RelativeFilePath::try_new("schemas/../property_bank.json");
                assert!(dir_result.is_err());
                assert!(file_result.is_err());
            }

            #[test]
            fn accepts_path_when_contains_duplicate_forward_separators() {
                let dir_result = RelativeDirPath::try_new("schemas//cache");
                let file_result =
                    RelativeFilePath::try_new("schemas//property_bank.json");
                assert!(dir_result.is_ok());
                assert!(file_result.is_ok());
            }

            #[test]
            fn accepts_path_when_contains_backslashes() {
                let dir_result = RelativeDirPath::try_new("schemas\\cache");
                let file_result =
                    RelativeFilePath::try_new("schemas\\property_bank.json");
                assert!(dir_result.is_ok());
                assert!(file_result.is_ok());
            }

            #[test]
            fn rejects_path_when_empty() {
                let dir_result = RelativeDirPath::try_new("  ");
                let file_result = RelativeFilePath::try_new("");
                assert!(dir_result.is_err());
                assert!(file_result.is_err());
            }
        }

        mod accessors {
            use super::*;

            #[test]
            fn returns_original_string_when_value_is_valid() {
                let dir = RelativeDirPath::try_new("schemas/cache")
                    .expect("valid dir path should construct");
                let file =
                    RelativeFilePath::try_new("schemas/property_bank.json")
                        .expect("valid file path should construct");
                assert_eq!(dir.as_str(), "schemas/cache");
                assert_eq!(file.as_str(), "schemas/property_bank.json");
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

            #[test]
            fn appends_filename_to_existing_file() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let child_name = FileName::new("notes.txt".into());
                let child = temp.path().join("notes.txt");
                std::fs::write(&child, "content")
                    .expect("child file should be writable");

                let file = dir
                    .append_filename(&child_name)
                    .expect("append filename should produce file path");
                assert_eq!(file.as_path(), child.as_path());
            }

            #[test]
            fn appends_filename_rejects_missing_file() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let child_name = FileName::new("missing.txt".into());

                assert!(dir.append_filename(&child_name).is_err());
            }

            #[test]
            fn appends_dirname_to_existing_dir() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let child_name = DirName::new("notes".into());
                let child = temp.path().join("notes");
                std::fs::create_dir_all(&child)
                    .expect("child directory should be created");

                let subdir = dir
                    .append_dirname(&child_name)
                    .expect("append dirname should produce dir path");
                assert_eq!(subdir.as_path(), child.as_path());
            }

            #[test]
            fn appends_dirname_rejects_missing_dir() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let child_name = DirName::new("missing".into());

                assert!(dir.append_dirname(&child_name).is_err());
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

        mod append {
            use super::*;

            #[test]
            fn returns_file_path_when_appending_relative_file_path() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let rel_file = RelativeFilePath::try_new("notes/daily.md")
                    .expect("valid rel file");

                // Ensure the file exists for validation
                let abs_file = temp.path().join("notes/daily.md");
                std::fs::create_dir_all(abs_file.parent().unwrap()).unwrap();
                std::fs::write(&abs_file, "content").unwrap();

                let file = dir
                    .append_file(&rel_file)
                    .expect("append_file should succeed");
                assert_eq!(file.as_path(), abs_file);
            }

            #[test]
            fn returns_file_path_when_appending_file_name() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let file_name = FileName::new("note.md".into());

                let abs_file = temp.path().join("note.md");
                std::fs::write(&abs_file, "content").unwrap();

                let file = dir
                    .append_file(&file_name)
                    .expect("append_file should succeed");
                assert_eq!(file.as_path(), abs_file);
            }

            #[test]
            fn returns_dir_path_when_appending_relative_dir_path() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let rel_dir = RelativeDirPath::try_new("schemas/cache")
                    .expect("valid rel dir");

                let abs_dir = temp.path().join("schemas/cache");
                std::fs::create_dir_all(&abs_dir).unwrap();

                let result = dir
                    .append_dir(&rel_dir)
                    .expect("append_dir should succeed");
                assert_eq!(result.as_path(), abs_dir);
            }

            #[test]
            fn returns_dir_path_when_appending_dir_name() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let dir_name = DirName::new("notes".into());

                let abs_dir = temp.path().join("notes");
                std::fs::create_dir_all(&abs_dir).unwrap();

                let result = dir
                    .append_dir(&dir_name)
                    .expect("append_dir should succeed");
                assert_eq!(result.as_path(), abs_dir);
            }

            #[test]
            fn returns_error_when_appended_file_does_not_exist() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let rel_file = RelativeFilePath::try_new("missing.md")
                    .expect("valid rel file");

                assert!(dir.append_file(&rel_file).is_err());
            }

            #[test]
            fn returns_error_when_appended_dir_does_not_exist() {
                let temp = TempDir::new().expect("temp dir should be created");
                let dir = DirPath::try_new(temp.path().to_path_buf())
                    .expect("dir path should be valid");
                let rel_dir = RelativeDirPath::try_new("missing_dir")
                    .expect("valid rel dir");

                assert!(dir.append_dir(&rel_dir).is_err());
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
