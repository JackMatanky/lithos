//! Owned and borrowed filename components.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use rkyv::{Archive, Deserialize, Serialize};

/// Owned filename (UTF-8).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FileName(Box<str>);

impl FileName {
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
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        Path::new(self.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the file extension.
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

    /// Get a borrowed view of this filename.
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> FileNameRef<'_> {
        FileNameRef(OsStr::new(self.as_str()))
    }
}

impl From<Box<str>> for FileName {
    #[inline]
    fn from(filename: Box<str>) -> Self {
        Self::new(filename)
    }
}

impl From<String> for FileName {
    #[inline]
    fn from(filename: String) -> Self {
        Self::new(filename.into_boxed_str())
    }
}

impl From<FileName> for Box<str> {
    #[inline]
    fn from(filename: FileName) -> Self {
        filename.0
    }
}

impl From<FileName> for String {
    #[inline]
    fn from(filename: FileName) -> Self {
        filename.0.into_string()
    }
}

impl AsRef<str> for FileName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for FileName {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl TryFrom<&Path> for FileName {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let name = path
            .file_name()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path terminates in .. or is empty",
                )
            })?
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Path contains invalid UTF-8",
                )
            })?;

        Ok(Self::new(name.into()))
    }
}

impl TryFrom<PathBuf> for FileName {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

/// Owned directory name (UTF-8).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct DirName(Box<str>);

impl DirName {
    /// Create a new directory name.
    #[inline]
    #[must_use]
    pub fn new(name: Box<str>) -> Self {
        Self(name)
    }

    /// Get the directory name as a string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Owned basename (filename without extension).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct BaseName(Box<str>);

impl BaseName {
    /// Create a new basename.
    #[inline]
    #[must_use]
    pub fn new(name: Box<str>) -> Self {
        Self(name)
    }

    /// Get the basename as a string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Borrowed filename view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileNameRef<'a>(pub(crate) &'a OsStr);

impl<'a> FileNameRef<'a> {
    /// Get the filename as a string slice if it is valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
    }

    /// Get the basename view.
    #[inline]
    #[must_use]
    pub fn basename(&self) -> BaseNameRef<'a> {
        BaseNameRef(Path::new(self.0).file_stem().unwrap_or(self.0))
    }
}

/// Borrowed directory name view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirNameRef<'a>(pub(crate) &'a OsStr);

/// Borrowed basename view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BaseNameRef<'a>(pub(crate) &'a OsStr);

impl<'a> BaseNameRef<'a> {
    /// Get the basename as a string slice if it is valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_basename_correctly() {
        let name = FileName::from("my-note.md".to_owned());
        let name_ref = name.as_ref();
        let base_ref = name_ref.basename();

        assert_eq!(base_ref.as_str(), Some("my-note"));
    }

    #[test]
    fn should_convert_to_owned_basename() {
        let name = FileName::from("archive.tar.gz".to_owned());
        let base_ref = name.as_ref().basename();

        // Note: file_stem() on "archive.tar.gz" is "archive.tar"
        assert_eq!(base_ref.as_str(), Some("archive.tar"));
    }
}
