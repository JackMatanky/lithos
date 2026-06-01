//! Path value objects for the Note context.
//!
//! Provides validated vault-relative paths for notes and folders.
//!
//! # Validation Rules
//!
//! - Non-empty, vault-relative paths only.
//! - Reject Windows drive letters and UNC prefixes.
//! - Reject traversal (`..`) and current-dir (`.`) components.
//! - Reject hidden path segments (leading `.`).
//! - Reject non-UTF-8 segments.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::fmt;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::{NoteError, NoteFileError};
use crate::fs::{PathError, path::PathKey};

// TODO: Remove NotePath/FolderPath once centralized discovery
//       processor provides FileView with path/folder attributes.
//       The `TryFrom<PathKey>` and `From<PathKey>` impls (bridging
//       vault processor to note context pre-centrilization) must
//       also be removed — the centralized processor will own all
//       path construction, eliminating the need for these
//       conversions from the vault side.

/// Validated vault-relative path for a note.
///
/// Ensures the path follows Obsidian conventions (e.g., forward slashes,
/// `.md` extension) and prevents directory traversal attacks.
///
/// # Errors
///
/// Returns [`NoteFileError::InvalidPath`] or
/// [`NoteFileError::UnsupportedExtension`] if the path is invalid.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::paths::NotePath;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = NotePath::try_new("daily/2024-01-01.md")?;
/// assert_eq!(path.as_str(), "daily/2024-01-01.md");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NotePath(PathKey);

impl NotePath {
    /// Creates a new [`NotePath`] with validation.
    ///
    /// # Errors
    ///
    /// Returns [`NoteFileError`] if the path is invalid.
    #[inline]
    pub fn try_new(path: &str) -> Result<Self, NoteError> {
        if let [first, second, ..] = path.as_bytes()
            && first.is_ascii_alphabetic()
            && *second == b':'
        {
            return Err(NoteFileError::InvalidPath {
                path: path.into(),
                reason: "windows-style prefixes are not allowed",
            }
            .into());
        }

        let key = PathKey::try_new(path)
            .map_err(|e| path_error_to_invalid_path(path, &e))?;

        let normalized_path = std::path::Path::new(key.as_str());
        let ext = normalized_path.extension().and_then(|ext| ext.to_str());

        if !ext.is_some_and(|ext| ext.eq_ignore_ascii_case("md")) {
            return Err(NoteFileError::UnsupportedExtension {
                path: path.into(),
                found: ext.unwrap_or("").into(),
            }
            .into());
        }

        Ok(Self(key))
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns a reference to the underlying [`PathKey`].
    #[inline]
    #[must_use]
    pub fn as_path_key(&self) -> &PathKey {
        &self.0
    }
}

impl fmt::Display for NotePath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<NotePath> for PathKey {
    #[inline]
    fn from(path: NotePath) -> Self {
        path.0
    }
}

impl TryFrom<&str> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

impl TryFrom<PathKey> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(key: PathKey) -> Result<Self, Self::Error> {
        let ext = std::path::Path::new(key.as_str())
            .extension()
            .and_then(|ext| ext.to_str());
        if !ext.is_some_and(|ext| ext.eq_ignore_ascii_case("md")) {
            return Err(NoteFileError::UnsupportedExtension {
                path: key.as_str().into(),
                found: ext.unwrap_or("").into(),
            }
            .into());
        }
        Ok(Self(key))
    }
}

/// Validated folder path within the vault.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct FolderPath(PathKey);

impl FolderPath {
    /// Creates a validated folder path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteFileError::InvalidPath`] if the folder path is
    /// invalid.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        let key = PathKey::try_new(value)
            .map_err(|e| path_error_to_invalid_path(value, &e))?;
        Ok(Self(key))
    }

    /// Returns the folder path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns a reference to the underlying [`PathKey`].
    #[inline]
    #[must_use]
    pub fn as_path_key(&self) -> &PathKey {
        &self.0
    }
}

impl fmt::Display for FolderPath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<FolderPath> for PathKey {
    #[inline]
    fn from(path: FolderPath) -> Self {
        path.0
    }
}

impl From<PathKey> for FolderPath {
    #[inline]
    fn from(key: PathKey) -> Self {
        Self(key)
    }
}

fn path_error_to_invalid_path(path: &str, err: &PathError) -> NoteError {
    let reason = match err {
        PathError::Empty => "path cannot be empty",
        PathError::NotRelative(_) => "path must be relative",
        PathError::ParentTraversal(_) => "path traversal not allowed",
        PathError::CurrentDirComponent(_) => {
            "path must not include '.' components"
        }
        PathError::PlatformPrefix(_) => {
            "windows-style prefixes are not allowed"
        }
        PathError::InvalidUtf8(_) => "path contains invalid utf-8",
        _ => "invalid path",
    };
    NoteFileError::InvalidPath {
        path: path.into(),
        reason,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_path() {
        let result = NotePath::try_new("/absolute.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_curdir_components() {
        let result = NotePath::try_new("folder/./note.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_wrong_extension() {
        let result = NotePath::try_new("note.txt");
        result.unwrap_err();
    }

    #[test]
    fn rejects_windows_drive_prefix() {
        let result = NotePath::try_new("C:notes/note.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_unc_prefix() {
        let result = NotePath::try_new("//server/share/note.md");
        result.unwrap_err();
    }

    #[test]
    fn accepts_valid_vault_path() {
        let path = NotePath::try_new("folder/note.md").unwrap();
        assert_eq!(path.as_str(), "folder/note.md");
    }
}
