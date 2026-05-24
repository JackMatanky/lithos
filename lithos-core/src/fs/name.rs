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

impl TryFrom<&Path> for FileName {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let name = path
            .file_name()
            .ok_or_else(|| super::PathError::NoFileName(path.to_path_buf()))?
            .to_str()
            .ok_or_else(|| super::PathError::InvalidUtf8(path.to_path_buf()))?;

        Ok(Self::new(name.into()))
    }
}

/// Borrowed filename view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileNameRef<'a>(pub(crate) &'a OsStr);

impl FileNameRef<'_> {
    /// Get the filename as a string slice if it is valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
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

    /// Get a borrowed view of this basename.
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> BaseNameRef<'_> {
        BaseNameRef(OsStr::new(self.as_str()))
    }
}

impl TryFrom<FileName> for BaseName {
    type Error = super::PathError;

    #[inline]
    fn try_from(name: FileName) -> Result<Self, Self::Error> {
        Path::new(name.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| BaseName::new(s.into()))
            .ok_or_else(|| {
                super::PathError::NoStem(Path::new(name.as_str()).to_path_buf())
            })
    }
}

impl TryFrom<&Path> for BaseName {
    type Error = super::PathError;

    #[inline]
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| Self::new(s.into()))
            .ok_or_else(|| super::PathError::NoStem(path.to_path_buf()))
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

    /// Get a borrowed view of this directory name.
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> DirNameRef<'_> {
        DirNameRef(OsStr::new(self.as_str()))
    }
}

impl AsRef<str> for DirName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Borrowed directory name view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirNameRef<'a>(pub(crate) &'a OsStr);

impl DirNameRef<'_> {
    /// Get the directory name as a string slice if it is valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    mod file_name {
        use super::*;

        #[test]
        fn exposes_string_view_for_storage() {
            let name = FileName::from("my-note.md".to_owned());
            assert_eq!(name.as_str(), "my-note.md");
        }
    }

    mod file_name_ref {
        use super::*;

        #[test]
        fn exposes_utf8_view_when_valid() {
            let name = FileName::from("my-note.md".to_owned());
            assert_eq!(name.as_ref().as_str(), Some("my-note.md"));
        }
    }

    mod base_name {
        use super::*;

        #[test]
        fn exposes_borrowed_view() {
            let name = BaseName::new("readme".to_owned().into_boxed_str());
            assert_eq!(name.as_ref().as_str(), Some("readme"));
        }

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

    mod dir_name {
        use super::*;

        #[test]
        fn exposes_borrowed_view() {
            let name = DirName::new("notes".to_owned().into_boxed_str());
            assert_eq!(name.as_ref().as_str(), Some("notes"));
        }
    }
}
