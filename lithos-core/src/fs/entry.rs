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
            let dir_path =
                DirPath::new(path.clone()).map_err(|e| ParseError::Io {
                    path: path.clone(),
                    source: e,
                })?;
            let dir_metadata =
                DirMetadata::try_from(&std_metadata).map_err(|e| {
                    ParseError::Io {
                        path,
                        source: e,
                    }
                })?;
            Ok(Self::Dir(FsDir::new(dir_path, dir_metadata)))
        } else {
            let file_path =
                FilePath::new(path.clone()).map_err(|e| ParseError::Io {
                    path: path.clone(),
                    source: e,
                })?;
            let file_metadata =
                FileMetadata::try_from(&std_metadata).map_err(|e| {
                    ParseError::Io {
                        path,
                        source: e,
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
    use crate::fs::metadata::FsTimes;

    #[test]
    fn fs_entry_try_from_walkdir_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a test file
        std::fs::write(temp_path.join("test.md"), "content").unwrap();

        // Use walkdir to get a DirEntry
        let entry = walkdir::WalkDir::new(temp_path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .find(|e| e.file_name().to_str() == Some("test.md"))
            .unwrap();

        // Convert to FsEntry
        let fs_entry = FsEntry::try_from(entry).unwrap();

        // Verify it's a file entry
        assert!(fs_entry.is_file());
        assert!(!fs_entry.is_dir());

        // Verify path contains the filename
        let path = fs_entry.path();
        assert!(path.is_file());
    }

    #[test]
    fn fs_entry_try_from_walkdir_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a test subdirectory
        std::fs::create_dir(temp_path.join("subdir")).unwrap();

        // Use walkdir to get a DirEntry for the directory
        let entry = walkdir::WalkDir::new(temp_path)
            .min_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .find(|e| e.file_name().to_str() == Some("subdir"))
            .unwrap();

        // Convert to FsEntry
        let fs_entry = FsEntry::try_from(entry).unwrap();

        // Verify it's a directory entry
        assert!(!fs_entry.is_file());
        assert!(fs_entry.is_dir());

        // Verify path
        let path = fs_entry.path();
        assert!(path.is_dir());
    }

    #[test]
    fn fs_file_stores_path_and_metadata() {
        let path = FilePath::try_from("vault/note.md").unwrap();
        let times = FsTimes::new(Some(SystemTime::now()), None);
        let metadata = FileMetadata::new(times.clone(), 1024, false);

        let file = FsFile::new(path.clone(), metadata.clone());

        assert_eq!(file.path(), &path);
        assert_eq!(file.metadata(), &metadata);
    }

    #[test]
    fn fs_dir_stores_path_and_metadata() {
        let path = DirPath::try_from("vault/schemas").unwrap();
        let times = FsTimes::new(Some(SystemTime::now()), None);
        let metadata = DirMetadata::new(times.clone(), false);

        let dir = FsDir::new(path.clone(), metadata.clone());

        assert_eq!(dir.path(), &path);
        assert_eq!(dir.metadata(), &metadata);
    }

    #[test]
    fn fs_entry_path_returns_unified_fs_path() {
        let file_path = FilePath::try_from("note.md").unwrap();
        let dir_path = DirPath::try_from("schemas").unwrap();
        let times = FsTimes::new(None, None);

        let file_entry = FsEntry::File(FsFile::new(
            file_path.clone(),
            FileMetadata::new(times.clone(), 100, false),
        ));
        let dir_entry = FsEntry::Dir(FsDir::new(
            dir_path.clone(),
            DirMetadata::new(times, false),
        ));

        assert!(file_entry.path().is_file());
        assert!(dir_entry.path().is_dir());
    }

    #[test]
    fn fs_entry_as_file_returns_some_for_file_variant() {
        let path = FilePath::try_from("note.md").unwrap();
        let times = FsTimes::new(None, None);
        let file = FsFile::new(path, FileMetadata::new(times, 512, false));
        let entry = FsEntry::File(file.clone());

        let retrieved = entry.as_file();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &file);
    }

    #[test]
    fn fs_entry_as_file_returns_none_for_dir_variant() {
        let path = DirPath::try_from("schemas").unwrap();
        let times = FsTimes::new(None, None);
        let dir = FsDir::new(path, DirMetadata::new(times, false));
        let entry = FsEntry::Dir(dir);

        assert!(entry.as_file().is_none());
    }

    #[test]
    fn fs_entry_as_dir_returns_some_for_dir_variant() {
        let path = DirPath::try_from("schemas").unwrap();
        let times = FsTimes::new(None, None);
        let dir = FsDir::new(path, DirMetadata::new(times, false));
        let entry = FsEntry::Dir(dir.clone());

        let retrieved = entry.as_dir();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &dir);
    }

    #[test]
    fn fs_entry_as_dir_returns_none_for_file_variant() {
        let path = FilePath::try_from("note.md").unwrap();
        let times = FsTimes::new(None, None);
        let file = FsFile::new(path, FileMetadata::new(times, 512, false));
        let entry = FsEntry::File(file);

        assert!(entry.as_dir().is_none());
    }
}
