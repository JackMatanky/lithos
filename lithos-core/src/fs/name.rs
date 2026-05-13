//! Owned and borrowed filename components.

use std::{ffi::OsStr, path::Path};

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

    /// Get the basename as a string slice (filename without extension).
    ///
    /// Uses Obsidian terminology where "basename" means filename without
    /// extension. Returns an empty string if the basename cannot be extracted.
    ///
    /// For the owned `BaseName` type with explicit error handling, use
    /// [`basename()`](Self::basename).
    #[inline]
    #[must_use]
    pub fn basename_str(&self) -> &str {
        Path::new(self.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the basename as an owned `BaseName` (filename without extension).
    ///
    /// Uses Obsidian terminology where "basename" means filename without
    /// extension. Returns `None` if the basename would be empty or cannot
    /// be extracted.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::FileName;
    ///
    /// let name = FileName::from("my-note.md".to_owned());
    /// let base = name.basename();
    /// assert!(base.is_some());
    /// assert_eq!(base.unwrap().as_str(), "my-note");
    /// ```
    #[inline]
    #[must_use]
    pub fn basename(&self) -> Option<BaseName> {
        BaseName::try_from(Path::new(self.as_str())).ok()
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

impl From<String> for FileName {
    #[inline]
    fn from(filename: String) -> Self {
        Self::new(filename.into_boxed_str())
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

impl TryFrom<FileName> for BaseName {
    type Error = std::io::Error;

    #[inline]
    fn try_from(name: FileName) -> Result<Self, Self::Error> {
        Path::new(name.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| BaseName::new(s.into()))
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path has no stem component",
                )
            })
    }
}

impl TryFrom<&Path> for BaseName {
    type Error = std::io::Error;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| Self::new(s.into()))
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path has no stem component",
                )
            })
    }
}

impl AsRef<str> for BaseName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Borrowed basename view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BaseNameRef<'a>(pub(crate) &'a OsStr);

impl BaseNameRef<'_> {
    /// Get the basename as a string slice if it is valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
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

/// Borrowed directory name view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirNameRef<'a>(pub(crate) &'a OsStr);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    mod file_name {
        use super::*;

        mod basename {
            use super::*;

            #[test]
            fn returns_some_for_simple_filename() {
                let name = FileName::from("my-note.md".to_owned());
                let base = name.basename();
                assert!(base.is_some());
                assert_eq!(base.unwrap().as_str(), "my-note");
            }

            #[test]
            fn returns_some_for_hidden_file_with_extension() {
                let name = FileName::from(".md".to_owned());
                let base = name.basename();
                assert!(base.is_some());
                assert_eq!(base.unwrap().as_str(), ".md");
            }

            #[test]
            fn handles_double_extension() {
                let name = FileName::from("archive.tar.gz".to_owned());
                let base = name.basename();
                assert!(base.is_some());
                // Note: file_stem() on "archive.tar.gz" is "archive.tar"
                assert_eq!(base.unwrap().as_str(), "archive.tar");
            }
        }

        mod basename_str {
            use super::*;

            #[test]
            fn returns_stem_for_simple_filename() {
                let name = FileName::from("my-note.md".to_owned());
                assert_eq!(name.basename_str(), "my-note");
            }

            #[test]
            fn returns_stem_for_hidden_file() {
                let name = FileName::from(".md".to_owned());
                assert_eq!(name.basename_str(), ".md");
            }
        }
    }

    mod file_name_ref {
        use super::*;

        mod basename {
            use super::*;

            #[test]
            fn extracts_borrowed_basename_view() {
                let name = FileName::from("my-note.md".to_owned());
                let name_ref = name.as_ref();
                let base_ref = name_ref.basename();
                assert_eq!(base_ref.as_str(), Some("my-note"));
            }

            #[test]
            fn handles_double_extension() {
                let name = FileName::from("archive.tar.gz".to_owned());
                let base_ref = name.as_ref().basename();
                // Note: file_stem() on "archive.tar.gz" is "archive.tar"
                assert_eq!(base_ref.as_str(), Some("archive.tar"));
            }
        }
    }

    mod base_name {
        use super::*;

        mod try_from_path {
            use super::*;

            #[test]
            fn constructs_from_file_stem() {
                let path = PathBuf::from("readme.md");
                let base = BaseName::try_from(path.as_path()).unwrap();
                assert_eq!(base.as_str(), "readme");
            }

            #[test]
            fn extracts_from_full_path() {
                let path = PathBuf::from("notes/app.md");
                let base = BaseName::try_from(path.as_path()).unwrap();
                assert_eq!(base.as_str(), "app");
            }
        }

        mod try_from_filename {
            use super::*;

            #[test]
            fn converts_filename_with_extension() {
                let name = FileName::from("document.txt".to_owned());
                let base = BaseName::try_from(name).unwrap();
                assert_eq!(base.as_str(), "document");
            }

            #[test]
            fn returns_error_for_empty_filename() {
                let name = FileName::from(String::new());
                let base = BaseName::try_from(name);
                assert!(base.is_err());
            }
        }
    }
}
