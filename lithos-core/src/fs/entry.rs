//! Filesystem entry types for files and directories.
//!
//! Provides unified entry types that combine paths with metadata,
//! distinguishing files from directories at the type level.

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    metadata::{DirMetadata, FileMetadata},
    path::{DirPath, FilePath, FsPath},
};

/// Unified filesystem entry for files or directories.
///
/// Provides type-safe access to entries with variants for files and
/// directories.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FsEntry {
    /// A file entry.
    File(FsFile),
    /// A directory entry.
    Dir(FsDir),
}

impl FsEntry {
    /// Check if this entry is a file.
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Check if this entry is a directory.
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// Get file entry if this is a file.
    #[inline]
    #[must_use]
    pub const fn as_file(&self) -> Option<&FsFile> {
        match self {
            Self::File(file) => Some(file),
            Self::Dir(_) => None,
        }
    }

    /// Get directory entry if this is a directory.
    #[inline]
    #[must_use]
    pub const fn as_dir(&self) -> Option<&FsDir> {
        match self {
            Self::File(_) => None,
            Self::Dir(dir) => Some(dir),
        }
    }

    /// Get the path for this entry as an `FsPath`.
    ///
    /// This returns a unified path reference that can represent either a file
    /// or directory path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> FsPath {
        match self {
            Self::File(file) => FsPath::File(file.path().clone()),
            Self::Dir(dir) => FsPath::Dir(dir.path().clone()),
        }
    }
}

impl TryFrom<walkdir::DirEntry> for FsEntry {
    type Error = super::error::ParseError;

    #[inline]
    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        use super::{
            error::ParseError,
            metadata::{DirMetadata, FileMetadata},
        };

        // Get metadata first (before consuming entry)
        let std_metadata = entry.metadata().map_err(|e| {
            let io_err = std::io::Error::other(format!("walkdir error: {e}"));
            ParseError::Io {
                path: entry.path().to_path_buf(),
                source: io_err,
            }
        })?;

        // Now take ownership of the path
        let path = entry.into_path();

        if std_metadata.is_dir() {
            let dir_path = DirPath::new(path.clone()).map_err(|source| {
                ParseError::Io {
                    path: path.clone(), // Only clone on error path
                    source,
                }
            })?;
            let dir_metadata =
                DirMetadata::try_from(&std_metadata).map_err(|source| {
                    ParseError::Io {
                        path, // Move path (last use)
                        source,
                    }
                })?;
            Ok(Self::Dir(FsDir::new(dir_path, dir_metadata)))
        } else {
            let file_path = FilePath::new(path.clone()).map_err(|source| {
                ParseError::Io {
                    path: path.clone(), // Only clone on error path
                    source,
                }
            })?;
            let file_metadata =
                FileMetadata::try_from(&std_metadata).map_err(|source| {
                    ParseError::Io {
                        path, // Move path (last use)
                        source,
                    }
                })?;
            Ok(Self::File(FsFile::new(file_path, file_metadata)))
        }
    }
}

/// A file entry with path and metadata.
///
/// Represents a concrete file on the filesystem with its associated metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FsFile {
    /// Path to the file.
    path: FilePath,
    /// File metadata.
    metadata: FileMetadata,
}

impl FsFile {
    /// Create a new file entry.
    #[inline]
    #[must_use]
    pub const fn new(path: FilePath, metadata: FileMetadata) -> Self {
        Self {
            path,
            metadata,
        }
    }

    /// Get the file path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &FilePath {
        &self.path
    }

    /// Get the file metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }
}

/// A directory entry with path and metadata.
///
/// Represents a concrete directory on the filesystem with its associated
/// metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FsDir {
    /// Path to the directory.
    path: DirPath,
    /// Directory metadata.
    metadata: DirMetadata,
}

impl FsDir {
    /// Create a new directory entry.
    #[inline]
    #[must_use]
    pub const fn new(path: DirPath, metadata: DirMetadata) -> Self {
        Self {
            path,
            metadata,
        }
    }

    /// Get the directory path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &DirPath {
        &self.path
    }

    /// Get the directory metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &DirMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::fs::{error::ParseError, metadata::FsTimes};

    mod fs_entry {
        use super::*;

        mod try_from {
            use super::*;

            #[test]
            fn returns_file_entry_for_walkdir_file() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let temp_path = temp_dir.path();

                std::fs::write(temp_path.join("test.md"), "content").unwrap();

                let entry = walkdir::WalkDir::new(temp_path)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .find(|e| e.file_name().to_str() == Some("test.md"))
                    .unwrap();

                let fs_entry = FsEntry::try_from(entry).unwrap();

                assert!(fs_entry.is_file(), "Expected file entry");
            }

            #[test]
            fn returns_dir_entry_for_walkdir_directory() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let temp_path = temp_dir.path();

                std::fs::create_dir(temp_path.join("subdir")).unwrap();

                let entry = walkdir::WalkDir::new(temp_path)
                    .min_depth(1)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .find(|e| e.file_name().to_str() == Some("subdir"))
                    .unwrap();

                let fs_entry = FsEntry::try_from(entry).unwrap();

                assert!(fs_entry.is_dir(), "Expected directory entry");
            }

            #[test]
            fn returns_error_when_file_missing() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let temp_path = temp_dir.path();
                let file_path = temp_path.join("missing.md");

                std::fs::write(&file_path, "content").unwrap();

                let entry = walkdir::WalkDir::new(temp_path)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .find(|e| e.file_name().to_str() == Some("missing.md"))
                    .unwrap();

                std::fs::remove_file(&file_path).unwrap();

                let result = FsEntry::try_from(entry);

                assert!(result.is_err(), "Expected error for missing file");
                let error = result.unwrap_err();
                assert!(
                    matches!(error, ParseError::Io { .. }),
                    "Expected ParseError::Io, got {error:?}"
                );
            }
        }

        mod is_file {
            use super::*;

            #[test]
            fn returns_true_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path = FilePath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FsFile::new(path, FileMetadata::new(times, 0, false));
                let entry = FsEntry::File(file);

                let result = entry.is_file();

                assert!(result, "Expected true for file variant");
            }

            #[test]
            fn returns_false_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = FsDir::new(path, DirMetadata::new(times, false));
                let entry = FsEntry::Dir(dir);

                let result = entry.is_file();

                assert!(!result, "Expected false for directory variant");
            }
        }

        mod is_dir {
            use super::*;

            #[test]
            fn returns_true_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = FsDir::new(path, DirMetadata::new(times, false));
                let entry = FsEntry::Dir(dir);

                let result = entry.is_dir();

                assert!(result, "Expected true for directory variant");
            }

            #[test]
            fn returns_false_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path = FilePath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FsFile::new(path, FileMetadata::new(times, 0, false));
                let entry = FsEntry::File(file);

                let result = entry.is_dir();

                assert!(!result, "Expected false for file variant");
            }
        }

        mod as_file {
            use super::*;

            #[test]
            fn returns_some_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path = FilePath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FsFile::new(path, FileMetadata::new(times, 512, false));
                let entry = FsEntry::File(file.clone());

                let result = entry.as_file();

                assert!(result.is_some(), "Expected Some for file variant");
                assert_eq!(
                    result.unwrap(),
                    &file,
                    "Expected same file reference"
                );
            }

            #[test]
            fn returns_none_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = FsDir::new(path, DirMetadata::new(times, false));
                let entry = FsEntry::Dir(dir);

                let result = entry.as_file();

                assert!(
                    result.is_none(),
                    "Expected None for directory variant"
                );
            }
        }

        mod as_dir {
            use super::*;

            #[test]
            fn returns_some_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = FsDir::new(path, DirMetadata::new(times, false));
                let entry = FsEntry::Dir(dir.clone());

                let result = entry.as_dir();

                assert!(
                    result.is_some(),
                    "Expected Some for directory variant"
                );
                assert_eq!(
                    result.unwrap(),
                    &dir,
                    "Expected same dir reference"
                );
            }

            #[test]
            fn returns_none_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path = FilePath::new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FsFile::new(path, FileMetadata::new(times, 512, false));
                let entry = FsEntry::File(file);

                let result = entry.as_dir();

                assert!(result.is_none(), "Expected None for file variant");
            }
        }

        mod path {
            use super::*;

            #[test]
            fn returns_file_path_for_file_entry() {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                let file_path =
                    FilePath::new(temp_file.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file_entry = FsEntry::File(FsFile::new(
                    file_path,
                    FileMetadata::new(times, 100, false),
                ));

                let result = file_entry.path();

                assert!(result.is_file(), "Expected file path type");
            }

            #[test]
            fn returns_dir_path_for_dir_entry() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let dir_path =
                    DirPath::new(temp_dir.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir_entry = FsEntry::Dir(FsDir::new(
                    dir_path,
                    DirMetadata::new(times, false),
                ));

                let result = dir_entry.path();

                assert!(result.is_dir(), "Expected directory path type");
            }
        }
    }

    mod fs_file {
        use super::*;

        #[test]
        fn returns_stored_path() {
            let temp = tempfile::NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = FileMetadata::new(times, 1024, false);

            let file = FsFile::new(path.clone(), metadata);

            assert_eq!(file.path(), &path, "Expected same path reference");
        }

        #[test]
        fn returns_stored_metadata() {
            let temp = tempfile::NamedTempFile::new().unwrap();
            let path = FilePath::new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = FileMetadata::new(times.clone(), 1024, false);

            let file = FsFile::new(path, metadata.clone());

            assert_eq!(
                file.metadata(),
                &metadata,
                "Expected same metadata reference"
            );
        }
    }

    mod fs_dir {
        use super::*;

        #[test]
        fn returns_stored_path() {
            let temp = tempfile::TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = DirMetadata::new(times, false);

            let dir = FsDir::new(path.clone(), metadata);

            assert_eq!(dir.path(), &path, "Expected same path reference");
        }

        #[test]
        fn returns_stored_metadata() {
            let temp = tempfile::TempDir::new().unwrap();
            let path = DirPath::new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = DirMetadata::new(times.clone(), false);

            let dir = FsDir::new(path, metadata.clone());

            assert_eq!(
                dir.metadata(),
                &metadata,
                "Expected same metadata reference"
            );
        }
    }
}
