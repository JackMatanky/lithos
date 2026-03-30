//! Vault domain types for files and folders.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    time::SystemTime,
};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::error::VaultPathError;
use crate::fs::PathValidator;

/// Validated vault-relative path.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultPath(Box<str>);

impl VaultPath {
    /// Creates a new validated vault path.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if the path is invalid.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, VaultPathError> {
        let normalized = Self::normalize(path);
        let normalized = normalized.as_ref().trim();
        PathValidator::validate_vault_path(normalized, None)
            .map_err(VaultPathError::from)?;
        Ok(Self(normalized.into()))
    }

    /// Creates a validated vault path from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if the path is invalid or not UTF-8.
    #[inline]
    pub fn try_from_path(path: &Path) -> Result<Self, VaultPathError> {
        let path_str = path.to_str().ok_or_else(|| {
            VaultPathError::InvalidPathEncoding {
                path: path.to_path_buf(),
            }
        })?;
        Self::try_new(path_str)
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the path as a `Path` reference.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.0.as_ref())
    }

    #[inline]
    fn normalize(path: &str) -> Cow<'_, str> {
        if path.contains('\\') {
            let mut owned = String::with_capacity(path.len());
            for ch in path.chars() {
                if ch == '\\' {
                    owned.push('/');
                } else {
                    owned.push(ch);
                }
            }
            Cow::Owned(owned)
        } else {
            Cow::Borrowed(path)
        }
    }
}

/// File metadata tracked for a vault entry.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultFile {
    path: VaultPath,
    basename: Box<str>,
    filename: Box<str>,
    parent: Box<str>,
    extension: Option<Box<str>>,
    size: u64,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl VaultFile {
    /// Builds a vault file entry from a path and filesystem metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if path components are invalid or UTF-8
    /// conversion fails.
    #[inline]
    pub fn try_new(
        path: VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<Self, VaultPathError> {
        let parts = PathParts::try_new(path.as_path())?;
        Ok(Self {
            path,
            basename: parts.basename,
            filename: parts.filename,
            parent: parts.parent,
            extension: parts.extension,
            size: metadata.len(),
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().ok(),
        })
    }

    /// Returns the vault-relative path for this file.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    /// Returns the filename without extension.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        &self.basename
    }

    /// Returns the filename with extension.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the parent folder path (empty for root).
    #[inline]
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Returns the file extension, if any.
    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Returns the file size in bytes.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the file creation timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the file modification timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }
}

/// Folder metadata tracked for a vault entry.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultFolder {
    path: VaultPath,
    basename: Box<str>,
    parent: Box<str>,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl VaultFolder {
    /// Builds a vault folder entry from a path and filesystem metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VaultPathError`] if path components are invalid or UTF-8
    /// conversion fails.
    #[inline]
    pub fn try_new(
        path: VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<Self, VaultPathError> {
        let parts = FolderParts::try_new(path.as_path())?;
        Ok(Self {
            path,
            basename: parts.basename,
            parent: parts.parent,
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().ok(),
        })
    }

    /// Returns the vault-relative path for this folder.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    /// Returns the folder name.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        &self.basename
    }

    /// Returns the parent folder path (empty for root).
    #[inline]
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Returns the folder creation timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the folder modification timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }
}

struct PathParts {
    basename: Box<str>,
    filename: Box<str>,
    parent: Box<str>,
    extension: Option<Box<str>>,
}

impl PathParts {
    fn try_new(path: &Path) -> Result<Self, VaultPathError> {
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let basename = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let extension =
            path.extension().and_then(|value| value.to_str()).map(Into::into);

        let parent =
            path.parent().and_then(|value| value.to_str()).unwrap_or("");

        Ok(Self {
            basename: basename.into(),
            filename: filename.into(),
            parent: parent.into(),
            extension,
        })
    }
}

struct FolderParts {
    basename: Box<str>,
    parent: Box<str>,
}

impl FolderParts {
    fn try_new(path: &Path) -> Result<Self, VaultPathError> {
        let basename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| VaultPathError::InvalidPathEncoding {
                path: PathBuf::from(path),
            })?;

        let parent =
            path.parent().and_then(|value| value.to_str()).unwrap_or("");

        Ok(Self {
            basename: basename.into(),
            parent: parent.into(),
        })
    }
}
