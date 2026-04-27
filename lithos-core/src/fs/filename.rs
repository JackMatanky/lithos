//! Filename wrapper for vault-scoped file operations.
//!
//! Provides a type-safe wrapper for filenames that ensures consistent
//! extension handling and path validation.

use std::path::Path;

use rkyv::{Archive, Deserialize, Serialize};

/// Filename for vault-scoped files (schemas, notes, templates).
///
/// Stores only the filename with its extension (e.g., "note.toml").
/// Vault directories are typically assumed to be flat or managed via
/// configuration. This type provides methods to extract the stem and
/// extension without repeatedly parsing the underlying string.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq))]
pub struct Filename(Box<str>);

impl Filename {
    /// Create a new filename from a boxed string.
    #[inline]
    #[must_use]
    pub fn new(filename: Box<str>) -> Self {
        Self(filename)
    }

    /// Get the filename as a string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the basename (filename without extension).
    ///
    /// Uses Obsidian terminology where "basename" means filename without
    /// extension.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::fs::Filename;
    ///
    /// let filename = Filename::new("note.toml".into());
    /// assert_eq!(filename.basename(), "note");
    /// ```
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        Path::new(self.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the file extension.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::fs::Filename;
    ///
    /// let filename = Filename::new("note.toml".into());
    /// assert_eq!(filename.extension(), Some("toml"));
    /// ```
    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        Path::new(self.as_str()).extension().and_then(|s| s.to_str())
    }

    /// Get the underlying filename as a `Path`.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl From<Box<str>> for Filename {
    #[inline]
    fn from(filename: Box<str>) -> Self {
        Self::new(filename)
    }
}

impl From<String> for Filename {
    #[inline]
    fn from(filename: String) -> Self {
        Self::new(filename.into_boxed_str())
    }
}

impl AsRef<str> for Filename {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Filename {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_extracts_filename_without_extension() {
        let filename = Filename::new("note.toml".into());
        assert_eq!(filename.basename(), "note");
    }

    #[test]
    fn basename_handles_hyphens() {
        let filename = Filename::new("base-note.toml".into());
        assert_eq!(filename.basename(), "base-note");
    }

    #[test]
    fn extension_returns_file_extension() {
        let filename = Filename::new("note.toml".into());
        assert_eq!(filename.extension(), Some("toml"));
    }

    #[test]
    fn extension_returns_none_for_no_extension() {
        let filename = Filename::new("note".into());
        assert_eq!(filename.extension(), None);
    }

    #[test]
    fn as_str_returns_full_filename() {
        let filename = Filename::new("note.toml".into());
        assert_eq!(filename.as_str(), "note.toml");
    }
}
