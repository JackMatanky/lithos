//! Filesystem node types for files and directories.
//!
//! Provides unified node types that combine paths with metadata,
//! distinguishing files from directories at the type level.

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    metadata::{DirMetadata, FileMetadata, FsMetadata},
    name::FileName,
    path::{DirPath, FilePath, FsPath, FsPathRef},
};

/// Unified filesystem node for files or directories.
///
/// Provides type-safe access to nodes with variants for files and
/// directories.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FsNode {
    /// A file node.
    File(FileNode),
    /// A directory node.
    Dir(DirNode),
}

impl FsNode {
    /// Check if this node is a file.
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Check if this node is a directory.
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// Get file node if this is a file.
    #[inline]
    #[must_use]
    pub const fn as_file(&self) -> Option<&FileNode> {
        match self {
            Self::File(file) => Some(file),
            Self::Dir(_) => None,
        }
    }

    /// Consume the node and return the file node if this is a file.
    #[inline]
    #[must_use]
    pub fn into_file(self) -> Option<FileNode> {
        match self {
            Self::File(file) => Some(file),
            Self::Dir(_) => None,
        }
    }

    /// Get directory node if this is a directory.
    #[inline]
    #[must_use]
    pub const fn as_dir(&self) -> Option<&DirNode> {
        match self {
            Self::File(_) => None,
            Self::Dir(dir) => Some(dir),
        }
    }

    /// Get the path for this node as an `FsPath`.
    ///
    /// This returns a unified path reference that can represent either a file
    /// or directory path.
    ///
    /// **Note**: This method clones the underlying path. For zero-copy access,
    /// use [`path_ref`](Self::path_ref) instead.
    #[inline]
    #[must_use]
    pub fn path(&self) -> FsPath {
        match self {
            Self::File(file) => FsPath::File(file.path().clone()),
            Self::Dir(dir) => FsPath::Dir(dir.path().clone()),
        }
    }

    /// Get a zero-copy reference to the path for this node.
    ///
    /// This returns a borrowed view into the path without cloning. Prefer this
    /// over [`path`](Self::path) when you only need to inspect the path.
    #[inline]
    #[must_use]
    pub fn path_ref(&self) -> FsPathRef<'_> {
        match self {
            Self::File(file) => FsPathRef::File(file.path()),
            Self::Dir(dir) => FsPathRef::Dir(dir.path()),
        }
    }

    /// Get a filename-like terminal component for this node.
    #[inline]
    #[must_use]
    pub fn filename(&self) -> Option<FileName> {
        let path = self.path_ref();
        FileName::try_from(path.as_path()).ok()
    }

    /// Get unified metadata for this node.
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> FsMetadata {
        match self {
            Self::File(file) => FsMetadata::File(file.metadata().clone()),
            Self::Dir(dir) => FsMetadata::Dir(dir.metadata().clone()),
        }
    }
}

impl TryFrom<walkdir::DirEntry> for FsNode {
    type Error = super::error::ScanError;

    #[inline]
    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        use super::error::ScanError;

        let ft = entry.file_type();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                return Err(ScanError::Traversal {
                    path: entry.into_path(),
                    source: std::io::Error::other(format!(
                        "walkdir error: {e}"
                    )),
                });
            }
        };

        let path = entry.into_path();

        if ft.is_dir() {
            let dir_path = DirPath::try_new(path)?;
            let dir_metadata = DirMetadata::from(&metadata);
            return Ok(Self::Dir(DirNode::new(dir_path, dir_metadata)));
        }

        if ft.is_file() {
            let file_path = FilePath::try_new(path)?;
            let file_metadata = FileMetadata::from(&metadata);
            return Ok(Self::File(FileNode::new(file_path, file_metadata)));
        }

        if ft.is_symlink() {
            if metadata.is_dir() {
                let dir_path = DirPath::try_new(path)?;
                let dir_metadata = DirMetadata::from(&metadata);
                return Ok(Self::Dir(DirNode::new(dir_path, dir_metadata)));
            }

            if metadata.is_file() {
                let file_path = FilePath::try_new(path)?;
                let file_metadata = FileMetadata::from(&metadata);
                return Ok(Self::File(FileNode::new(file_path, file_metadata)));
            }
        }

        Err(ScanError::UnsupportedEntryType(path))
    }
}

/// A file node with path and metadata.
///
/// Represents a concrete file on the filesystem with its associated metadata.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FileNode {
    /// Path to the file.
    path: FilePath,
    /// File metadata.
    metadata: FileMetadata,
}

impl FileNode {
    /// Create a new file node.
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

    /// Consume the node and return its metadata.
    #[inline]
    #[must_use]
    pub fn into_metadata(self) -> FileMetadata {
        self.metadata
    }
}

/// A directory node with path and metadata.
///
/// Represents a concrete directory on the filesystem with its associated
/// metadata.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct DirNode {
    /// Path to the directory.
    path: DirPath,
    /// Directory metadata.
    metadata: DirMetadata,
}

impl DirNode {
    /// Create a new directory node.
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

    /// Consume the node and return its metadata.
    #[inline]
    #[must_use]
    pub fn into_metadata(self) -> DirMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::fs::{error::ScanError, metadata::FsTimes};

    mod fs_node {
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

                let fs_entry = FsNode::try_from(entry).unwrap();

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

                let fs_entry = FsNode::try_from(entry).unwrap();

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

                let result = FsNode::try_from(entry);

                assert!(result.is_err(), "Expected error for missing file");
                let error = result.unwrap_err();
                assert!(
                    matches!(error, ScanError::Traversal { .. }),
                    "Expected ScanError::Traversal, got {error:?}"
                );
            }
        }

        mod is_file {
            use super::*;

            #[test]
            fn returns_true_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path =
                    FilePath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FileNode::new(path, FileMetadata::new(times, 0, false));
                let entry = FsNode::File(file);

                let result = entry.is_file();

                assert!(result, "Expected true for file variant");
            }

            #[test]
            fn returns_false_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = DirNode::new(path, DirMetadata::new(times, false));
                let entry = FsNode::Dir(dir);

                let result = entry.is_file();

                assert!(!result, "Expected false for directory variant");
            }
        }

        mod is_dir {
            use super::*;

            #[test]
            fn returns_true_for_dir_variant() {
                let temp = tempfile::TempDir::new().unwrap();
                let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = DirNode::new(path, DirMetadata::new(times, false));
                let entry = FsNode::Dir(dir);

                let result = entry.is_dir();

                assert!(result, "Expected true for directory variant");
            }

            #[test]
            fn returns_false_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path =
                    FilePath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FileNode::new(path, FileMetadata::new(times, 0, false));
                let entry = FsNode::File(file);

                let result = entry.is_dir();

                assert!(!result, "Expected false for file variant");
            }
        }

        mod as_file {
            use super::*;

            #[test]
            fn returns_some_for_file_variant() {
                let temp = tempfile::NamedTempFile::new().unwrap();
                let path =
                    FilePath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FileNode::new(path, FileMetadata::new(times, 512, false));
                let entry = FsNode::File(file.clone());

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
                let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = DirNode::new(path, DirMetadata::new(times, false));
                let entry = FsNode::Dir(dir);

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
                let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir = DirNode::new(path, DirMetadata::new(times, false));
                let entry = FsNode::Dir(dir.clone());

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
                let path =
                    FilePath::try_new(temp.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file =
                    FileNode::new(path, FileMetadata::new(times, 512, false));
                let entry = FsNode::File(file);

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
                    FilePath::try_new(temp_file.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file_entry = FsNode::File(FileNode::new(
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
                    DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir_entry = FsNode::Dir(DirNode::new(
                    dir_path,
                    DirMetadata::new(times, false),
                ));

                let result = dir_entry.path();

                assert!(result.is_dir(), "Expected directory path type");
            }
        }

        mod path_ref {
            use super::*;

            #[test]
            fn returns_file_path_ref_for_file_entry() {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                let file_path =
                    FilePath::try_new(temp_file.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file_entry = FsNode::File(FileNode::new(
                    file_path.clone(),
                    FileMetadata::new(times, 100, false),
                ));

                let result = file_entry.path_ref();

                assert!(result.is_file(), "Expected file path type");
                assert_eq!(result.as_path(), file_path.as_path());
            }

            #[test]
            fn returns_dir_path_ref_for_dir_entry() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let dir_path =
                    DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let dir_entry = FsNode::Dir(DirNode::new(
                    dir_path.clone(),
                    DirMetadata::new(times, false),
                ));

                let result = dir_entry.path_ref();

                assert!(result.is_dir(), "Expected directory path type");
                assert_eq!(result.as_path(), dir_path.as_path());
            }

            #[test]
            fn borrows_without_cloning() {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                let file_path =
                    FilePath::try_new(temp_file.path().to_path_buf()).unwrap();
                let times = FsTimes::new(None, None);
                let file_entry = FsNode::File(FileNode::new(
                    file_path.clone(),
                    FileMetadata::new(times, 100, false),
                ));

                // This should compile and not move file_entry
                let _ref1 = file_entry.path_ref();
                let _ref2 = file_entry.path_ref();

                // We can still use file_entry
                assert!(file_entry.is_file());
            }
        }

        mod filename_and_metadata {
            use super::*;

            #[test]
            fn filename_returns_terminal_component_for_file() {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                let file_path =
                    FilePath::try_new(temp_file.path().to_path_buf()).unwrap();
                let file_entry = FsNode::File(FileNode::new(
                    file_path,
                    FileMetadata::new(FsTimes::new(None, None), 7, false),
                ));

                let filename = file_entry.filename().unwrap();
                assert!(!filename.as_str().is_empty());
            }

            #[test]
            fn metadata_preserves_variant() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let dir_path =
                    DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();
                let entry = FsNode::Dir(DirNode::new(
                    dir_path,
                    DirMetadata::new(FsTimes::new(None, None), false),
                ));

                let metadata = entry.metadata();
                assert!(metadata.is_dir());
            }
        }
    }

    mod file {
        use super::*;

        #[test]
        fn returns_stored_path() {
            let temp = tempfile::NamedTempFile::new().unwrap();
            let path = FilePath::try_new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = FileMetadata::new(times, 1024, false);

            let file = FileNode::new(path.clone(), metadata);

            assert_eq!(file.path(), &path, "Expected same path reference");
        }

        #[test]
        fn returns_stored_metadata() {
            let temp = tempfile::NamedTempFile::new().unwrap();
            let path = FilePath::try_new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = FileMetadata::new(times.clone(), 1024, false);

            let file = FileNode::new(path, metadata.clone());

            assert_eq!(
                file.metadata(),
                &metadata,
                "Expected same metadata reference"
            );
        }
    }

    mod dir {
        use super::*;

        #[test]
        fn returns_stored_path() {
            let temp = tempfile::TempDir::new().unwrap();
            let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = DirMetadata::new(times, false);

            let dir = DirNode::new(path.clone(), metadata);

            assert_eq!(dir.path(), &path, "Expected same path reference");
        }

        #[test]
        fn returns_stored_metadata() {
            let temp = tempfile::TempDir::new().unwrap();
            let path = DirPath::try_new(temp.path().to_path_buf()).unwrap();
            let times = FsTimes::new(Some(SystemTime::now()), None);
            let metadata = DirMetadata::new(times.clone(), false);

            let dir = DirNode::new(path, metadata.clone());

            assert_eq!(
                dir.metadata(),
                &metadata,
                "Expected same metadata reference"
            );
        }
    }
}
