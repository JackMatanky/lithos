//! Path value objects for the Note context.
//!
//! Provides validated vault-relative paths for notes and folders.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::fmt;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::{NoteError, NoteMetadataError};

/// Validated vault-relative path for a note.
///
/// Ensures the path follows Obsidian conventions (e.g., forward slashes,
/// `.md` extension) and prevents directory traversal attacks.
///
/// # Errors
///
/// Returns [`NoteError::InvalidPath`] if:
/// - The path is absolute (starts with `/`).
/// - The extension is not `.md`.
/// - The path contains invalid characters.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::paths::NotePath;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = NotePath::new("daily/2024-01-01.md")?;
/// assert_eq!(path.as_str(), "daily/2024-01-01.md");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NotePath(RelativePath);

impl NotePath {
    /// Creates a new [`NotePath`] with validation.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::InvalidPath`] if the path is invalid.
    #[inline]
    pub fn new(path: &str) -> Result<Self, NoteError> {
        let relative = RelativePath::try_new(path)?;
        let normalized_path = std::path::Path::new(relative.as_str());

        if !normalized_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            return Err(NoteError::InvalidPath(
                "path must have .md extension".into(),
            ));
        }

        Ok(Self(relative))
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for NotePath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated folder path within the vault.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct FolderPath(RelativePath);

impl FolderPath {
    /// Creates a validated folder path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the folder is empty.
    /// Returns [`NoteError::InvalidPath`] if the folder path is invalid.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::FolderEmpty));
        }
        let relative = RelativePath::try_new(trimmed)?;
        Ok(Self(relative))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for FolderPath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
struct RelativePath(Box<str>);

impl RelativePath {
    fn try_new(path: &str) -> Result<Self, NoteError> {
        let normalized = Self::normalize_path_separators(path);
        Self::validate_relative_path(normalized.as_ref())?;
        Ok(Self(normalized.into()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    fn normalize_path_separators(path: &str) -> std::borrow::Cow<'_, str> {
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

    fn validate_relative_path(path: &str) -> Result<(), NoteError> {
        if path.is_empty() {
            return Err(NoteError::InvalidPath("path cannot be empty".into()));
        }

        if Self::has_drive_or_unc_prefix(path) {
            return Err(NoteError::InvalidPath(
                "windows-style prefixes are not allowed".into(),
            ));
        }

        if path.split('/').any(|segment| segment == ".") {
            return Err(NoteError::InvalidPath(
                "path must not include '.' components".into(),
            ));
        }

        let normalized_path = std::path::Path::new(path);
        if normalized_path.is_absolute() {
            return Err(NoteError::InvalidPath("path must be relative".into()));
        }

        for component in normalized_path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(NoteError::InvalidPath(
                        "path traversal not allowed".into(),
                    ));
                }
                std::path::Component::CurDir => {
                    return Err(NoteError::InvalidPath(
                        "path must not include '.' components".into(),
                    ));
                }
                std::path::Component::Normal(segment) => {
                    let segment = segment.to_str().ok_or_else(|| {
                        NoteError::InvalidPath(
                            "path contains invalid utf-8".into(),
                        )
                    })?;
                    if segment.starts_with('.') {
                        return Err(NoteError::InvalidPath(
                            "hidden path components not allowed".into(),
                        ));
                    }
                }
                std::path::Component::Prefix(_)
                | std::path::Component::RootDir => {
                    return Err(NoteError::InvalidPath(
                        "path must be relative".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn has_drive_or_unc_prefix(path: &str) -> bool {
        let mut chars = path.chars();
        let first = chars.next();
        let second = chars.next();
        let Some(first) = first else {
            return false;
        };
        if let Some(second) = second {
            if first.is_ascii_alphabetic() && second == ':' {
                return true;
            }
            if first == '/' && second == '/' {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_path() {
        let result = NotePath::new("/absolute.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_curdir_components() {
        let result = NotePath::new("folder/./note.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_wrong_extension() {
        let result = NotePath::new("note.txt");
        result.unwrap_err();
    }

    #[test]
    fn rejects_windows_drive_prefix() {
        let result = NotePath::new("C:notes/note.md");
        result.unwrap_err();
    }

    #[test]
    fn rejects_unc_prefix() {
        let result = NotePath::new("//server/share/note.md");
        result.unwrap_err();
    }

    #[test]
    fn accepts_valid_vault_path() -> Result<(), NoteError> {
        let path = NotePath::new("folder/note.md")?;
        assert_eq!(path.as_str(), "folder/note.md");
        Ok(())
    }
}
