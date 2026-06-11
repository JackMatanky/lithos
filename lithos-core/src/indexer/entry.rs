//! Index entry types representing filesystem nodes and their index status.
//!
//! Each entry pairs a domain node (identity + metadata) with its runtime
//! filesystem path and current index classification.

use super::model::{DirNode, FileNode, FsNodeId};
use crate::fs::{DirPath, FilePath};

/// Classification of a filesystem node's index state.
///
/// Drives staleness detection and incremental re-indexing logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexStatus {
    /// The node has not been seen before; needs full indexing.
    New,
    /// The node is known and its metadata matches; no action required.
    Fresh,
    /// The node is known but its metadata has changed; needs re-indexing.
    Stale,
}

/// An indexed file entry pairing a [`FileNode`] with its runtime path and
/// current index classification.
#[derive(Debug, Clone)]
pub struct FileIndexEntry {
    id: FsNodeId,
    node: FileNode,
    path: FilePath,
    status: IndexStatus,
}

impl FileIndexEntry {
    /// Creates a new file index entry.
    #[inline]
    #[must_use]
    pub fn new(
        id: FsNodeId,
        node: FileNode,
        path: FilePath,
        status: IndexStatus,
    ) -> Self {
        Self {
            id,
            node,
            path,
            status,
        }
    }

    /// Returns the node's stable identifier.
    #[inline]
    #[must_use]
    pub fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the file node domain record.
    #[inline]
    #[must_use]
    pub fn node(&self) -> &FileNode {
        &self.node
    }

    /// Returns the runtime filesystem path for this entry.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &FilePath {
        &self.path
    }

    /// Returns the current index classification of this entry.
    #[inline]
    #[must_use]
    pub fn status(&self) -> IndexStatus {
        self.status
    }
}

/// An indexed directory entry pairing a [`DirNode`] with its runtime path and
/// current index classification.
#[derive(Debug, Clone)]
pub struct DirIndexEntry {
    id: FsNodeId,
    node: DirNode,
    path: DirPath,
    status: IndexStatus,
}

impl DirIndexEntry {
    /// Creates a new directory index entry.
    #[inline]
    #[must_use]
    pub fn new(
        id: FsNodeId,
        node: DirNode,
        path: DirPath,
        status: IndexStatus,
    ) -> Self {
        Self {
            id,
            node,
            path,
            status,
        }
    }

    /// Returns the node's stable identifier.
    #[inline]
    #[must_use]
    pub fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the directory node domain record.
    #[inline]
    #[must_use]
    pub fn node(&self) -> &DirNode {
        &self.node
    }

    /// Returns the runtime filesystem path for this entry.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &DirPath {
        &self.path
    }

    /// Returns the current index classification of this entry.
    #[inline]
    #[must_use]
    pub fn status(&self) -> IndexStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    mod index_status {
        mod constructor {
            use crate::indexer::entry::IndexStatus;

            #[test]
            fn new_is_distinguishable_from_fresh_and_stale() {
                assert_ne!(IndexStatus::New, IndexStatus::Fresh);
                assert_ne!(IndexStatus::New, IndexStatus::Stale);
                assert_ne!(IndexStatus::Fresh, IndexStatus::Stale);
            }

            #[test]
            fn clones_correctly() {
                let s = IndexStatus::New;
                assert_eq!(s, s.clone());
            }
        }
    }

    mod file_index_entry {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    FileFormat, FilePath,
                    metadata::{FileMetadata, FsTimes},
                    name::FileName,
                    path::PathKey,
                },
                indexer::{
                    entry::{FileIndexEntry, IndexStatus},
                    model::{FileNode, FsNodeId},
                },
            };

            fn make_entry() -> FileIndexEntry {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let file_path_buf = temp_dir.path().join("file.md");
                std::fs::File::create(&file_path_buf).unwrap();

                let id = FsNodeId::new();
                let parent_id = FsNodeId::new();
                let key = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                let node = FileNode::new(
                    id,
                    parent_id,
                    key,
                    name,
                    format,
                    metadata,
                    recorded_at,
                );
                let path = FilePath::try_new(file_path_buf).unwrap();
                FileIndexEntry::new(id, node, path, IndexStatus::New)
            }

            #[test]
            fn returns_stored_status() {
                let entry = make_entry();
                assert_eq!(entry.status(), IndexStatus::New);
            }

            #[test]
            fn returns_stored_id() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let file_path_buf = temp_dir.path().join("file.md");
                std::fs::File::create(&file_path_buf).unwrap();

                let id = FsNodeId::new();
                let parent_id = FsNodeId::new();
                let key = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                let node = FileNode::new(
                    id,
                    parent_id,
                    key,
                    name,
                    format,
                    metadata,
                    recorded_at,
                );
                let path = FilePath::try_new(file_path_buf).unwrap();
                let entry =
                    FileIndexEntry::new(id, node, path, IndexStatus::Fresh);
                assert_eq!(entry.id(), id);
            }
        }
    }

    mod dir_index_entry {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    DirPath,
                    metadata::{DirMetadata, FsTimes},
                    name::DirName,
                    path::PathKey,
                },
                indexer::{
                    entry::{DirIndexEntry, IndexStatus},
                    model::{DirNode, FsNodeId},
                },
            };

            #[test]
            fn returns_stored_status() {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let dir_path_buf = temp_dir.path().join("notes");
                std::fs::create_dir_all(&dir_path_buf).unwrap();

                let id = FsNodeId::new();
                let key = PathKey::try_new("notes").unwrap();
                let name = DirName::new("notes".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let recorded_at = SystemTime::now();
                let node =
                    DirNode::new(id, None, key, name, metadata, recorded_at);
                let path = DirPath::try_new(dir_path_buf).unwrap();
                let entry =
                    DirIndexEntry::new(id, node, path, IndexStatus::Stale);
                assert_eq!(entry.status(), IndexStatus::Stale);
            }
        }
    }
}
