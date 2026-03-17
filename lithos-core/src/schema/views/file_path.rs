//! File path newtype for schema and property bank views.

use std::path::Path;

use rkyv::{Archive, Deserialize, Serialize};

/// File path for schema/property bank files, relative to vault root.
///
/// Provides methods to extract basename and extension without repeatedly
/// parsing the path.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
pub struct FilePath(Box<str>);

impl FilePath {
    /// Create a new file path.
    #[inline]
    #[must_use]
    pub fn new(path: Box<str>) -> Self {
        Self(path)
    }

    /// Get the full path as a string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the file basename (filename without extension).
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::views::FilePath;
    ///
    /// let path = FilePath::new("schemas/note.toml".into());
    /// assert_eq!(path.basename(), "note");
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
    /// use lithos_core::schema::views::FilePath;
    ///
    /// let path = FilePath::new("schemas/note.toml".into());
    /// assert_eq!(path.extension(), Some("toml"));
    /// ```
    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        Path::new(self.as_str()).extension().and_then(|s| s.to_str())
    }

    /// Get the underlying path as a `Path`.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl From<Box<str>> for FilePath {
    #[inline]
    fn from(path: Box<str>) -> Self {
        Self::new(path)
    }
}

impl From<String> for FilePath {
    #[inline]
    fn from(path: String) -> Self {
        Self::new(path.into_boxed_str())
    }
}

impl AsRef<str> for FilePath {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for FilePath {
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
        let path = FilePath::new("schemas/note.toml".into());
        assert_eq!(path.basename(), "note");
    }

    #[test]
    fn basename_handles_nested_paths() {
        let path = FilePath::new("vault/schemas/base-note.toml".into());
        assert_eq!(path.basename(), "base-note");
    }

    #[test]
    fn extension_returns_file_extension() {
        let path = FilePath::new("schemas/note.toml".into());
        assert_eq!(path.extension(), Some("toml"));
    }

    #[test]
    fn extension_returns_none_for_no_extension() {
        let path = FilePath::new("schemas/note".into());
        assert_eq!(path.extension(), None);
    }

    #[test]
    fn as_str_returns_full_path() {
        let path = FilePath::new("schemas/note.toml".into());
        assert_eq!(path.as_str(), "schemas/note.toml");
    }
}
