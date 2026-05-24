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
    use std::{
        ffi::OsStr,
        os::unix::ffi::OsStrExt,
        path::{Path, PathBuf},
    };

    use super::*;

    mod file {
        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_inner_str_for_as_str() {
                let name = FileName::from("my-note.md".to_owned());
                assert_eq!(name.as_str(), "my-note.md");
            }

            #[test]
            fn returns_inner_str_for_as_ref_str() {
                let name = FileName::from("my-note.md".to_owned());
                assert_eq!(
                    <FileName as AsRef<str>>::as_ref(&name),
                    "my-note.md"
                );
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn returns_borrowed_view_from_as_ref() {
                let name = FileName::from("my-note.md".to_owned());
                assert_eq!(name.as_ref().as_str(), Some("my-note.md"));
            }

            #[test]
            fn roundtrips_into_string() {
                let name = FileName::from("my-note.md".to_owned());
                let result: String = name.into();
                assert_eq!(result, "my-note.md");
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn constructs_when_path_has_valid_utf8_filename() {
                let path = Path::new("notes/app.md");
                let result = FileName::try_from(path);
                assert!(
                    result.is_ok(),
                    "expected valid filename, got {result:?}"
                );
            }

            #[test]
            fn returns_error_when_path_has_no_filename() {
                let path = Path::new("/");
                let result = FileName::try_from(path);
                assert!(matches!(
                    result,
                    Err(crate::fs::PathError::NoFileName(_))
                ));
            }

            #[test]
            fn returns_error_when_filename_is_not_utf8() {
                let path = Path::new(OsStr::from_bytes(b"\xff.md"));
                let result = FileName::try_from(path);
                assert!(matches!(
                    result,
                    Err(crate::fs::PathError::InvalidUtf8(_))
                ));
            }
        }

        mod borrowing {
            use super::*;

            #[test]
            fn returns_some_when_utf8() {
                let name = FileNameRef(OsStr::new("my-note.md"));
                assert_eq!(name.as_str(), Some("my-note.md"));
            }

            #[test]
            fn returns_none_when_not_utf8() {
                let name = FileNameRef(OsStr::from_bytes(b"\xff.md"));
                assert_eq!(name.as_str(), None);
            }
        }
    }

    mod base {
        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_inner_str_for_as_str() {
                let name = BaseName::new("readme".to_owned().into_boxed_str());
                assert_eq!(name.as_str(), "readme");
            }

            #[test]
            fn returns_inner_str_for_as_ref_str() {
                let name = BaseName::new("readme".to_owned().into_boxed_str());
                assert_eq!(<BaseName as AsRef<str>>::as_ref(&name), "readme");
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn constructs_from_filename_with_extension() {
                let name = FileName::from("document.txt".to_owned());
                let result = BaseName::try_from(name);
                assert!(
                    result.is_ok(),
                    "expected basename conversion, got {result:?}"
                );
            }

            #[test]
            fn returns_error_when_filename_has_no_stem() {
                let name = FileName::from(String::new());
                let result = BaseName::try_from(name);
                assert!(matches!(result, Err(crate::fs::PathError::NoStem(_))));
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn constructs_from_path_with_stem() {
                let path = PathBuf::from("readme.md");
                let result = BaseName::try_from(path.as_path());
                assert!(
                    result.is_ok(),
                    "expected stem from path, got {result:?}"
                );
            }

            #[test]
            fn returns_error_when_path_has_no_stem() {
                let path = Path::new("..");
                let result = BaseName::try_from(path);
                assert!(matches!(result, Err(crate::fs::PathError::NoStem(_))));
            }

            #[test]
            fn returns_error_when_stem_is_not_utf8() {
                let path = Path::new(OsStr::from_bytes(b"\xff.md"));
                let result = BaseName::try_from(path);
                assert!(matches!(result, Err(crate::fs::PathError::NoStem(_))));
            }
        }

        mod borrowing {
            use super::*;

            #[test]
            fn returns_some_when_utf8() {
                let name = BaseName::new("readme".to_owned().into_boxed_str());
                assert_eq!(name.as_ref().as_str(), Some("readme"));
            }

            #[test]
            fn returns_none_when_not_utf8() {
                let name = BaseNameRef(OsStr::from_bytes(b"\xff"));
                assert_eq!(name.as_str(), None);
            }
        }
    }

    mod dir {
        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_inner_str_for_as_str() {
                let name = DirName::new("notes".to_owned().into_boxed_str());
                assert_eq!(name.as_str(), "notes");
            }

            #[test]
            fn returns_inner_str_for_as_ref_str() {
                let name = DirName::new("notes".to_owned().into_boxed_str());
                assert_eq!(<DirName as AsRef<str>>::as_ref(&name), "notes");
            }
        }

        mod borrowing {
            use super::*;

            #[test]
            fn returns_some_when_utf8() {
                let name = DirName::new("notes".to_owned().into_boxed_str());
                assert_eq!(name.as_ref().as_str(), Some("notes"));
            }

            #[test]
            fn returns_none_when_not_utf8() {
                let name = DirNameRef(OsStr::from_bytes(b"\xff"));
                assert_eq!(name.as_str(), None);
            }
        }
    }
}
