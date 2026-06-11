//! Indexer domain model: filesystem node identity and classification types.
//!
//! Provides the foundational types for representing filesystem nodes
//! discovered during an index scan.

use std::{fmt, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use crate::{
    fs::{
        DirMetadata, FileFormat, FileMetadata,
        name::{DirName, FileName},
        path::PathKey,
    },
    utils::UuidV7,
};

/// Stable identifier for a filesystem node (file or directory).
///
/// Uses UUID v7 for time-ordered, collision-resistant identifiers that are
/// efficient for database indexing.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub(crate) struct FsNodeId(pub(crate) UuidV7);

impl FsNodeId {
    /// Creates a new random filesystem node identifier (UUID v7).
    #[inline]
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(UuidV7::new())
    }
}

impl Default for FsNodeId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FsNodeId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Classification of a filesystem node as either a file or a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FsNodeType {
    /// A regular file.
    File,
    /// A directory.
    Dir,
}

/// A filesystem file node with identity, path, and metadata.
///
/// Represents a discovered file within the index scope, capturing all
/// information needed for downstream indexing and staleness detection.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub(crate) struct FileNode {
    id: FsNodeId,
    parent_id: FsNodeId,
    path: PathKey,
    name: FileName,
    format: FileFormat,
    metadata: FileMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl FileNode {
    /// Creates a new file node.
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain model requires all these fields to be fully \
                  initialized"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
        id: FsNodeId,
        parent_id: FsNodeId,
        path: PathKey,
        name: FileName,
        format: FileFormat,
        metadata: FileMetadata,
        recorded_at: SystemTime,
    ) -> Self {
        Self {
            id,
            parent_id,
            path,
            name,
            format,
            metadata,
            recorded_at,
        }
    }

    /// Returns the node's stable identifier.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the parent directory's node identifier.
    #[inline]
    #[must_use]
    pub(crate) fn parent_id(&self) -> FsNodeId {
        self.parent_id
    }

    /// Returns the vault-relative storage key for this file.
    #[inline]
    #[must_use]
    pub(crate) fn path(&self) -> &PathKey {
        &self.path
    }

    /// Returns the file's name component.
    #[inline]
    #[must_use]
    pub(crate) fn name(&self) -> &FileName {
        &self.name
    }

    /// Returns the detected file format.
    #[inline]
    #[must_use]
    pub(crate) fn format(&self) -> FileFormat {
        self.format
    }

    /// Returns the file's filesystem metadata.
    #[inline]
    #[must_use]
    pub(crate) fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Returns the time at which this node was recorded in the index.
    #[inline]
    #[must_use]
    pub(crate) fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

/// A filesystem directory node with identity, path, and metadata.
///
/// Represents a discovered directory within the index scope.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub(crate) struct DirNode {
    id: FsNodeId,
    parent_id: Option<FsNodeId>,
    path: PathKey,
    name: DirName,
    metadata: DirMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl DirNode {
    /// Creates a new directory node.
    ///
    /// `parent_id` is `None` for the vault root directory.
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain model requires all these fields to be fully \
                  initialized"
    )]
    #[must_use]
    pub(crate) fn new(
        id: FsNodeId,
        parent_id: Option<FsNodeId>,
        path: PathKey,
        name: DirName,
        metadata: DirMetadata,
        recorded_at: SystemTime,
    ) -> Self {
        Self {
            id,
            parent_id,
            path,
            name,
            metadata,
            recorded_at,
        }
    }

    /// Returns the node's stable identifier.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the parent directory's node identifier, or `None` for root.
    #[inline]
    #[must_use]
    pub(crate) fn parent_id(&self) -> Option<FsNodeId> {
        self.parent_id
    }

    /// Returns the vault-relative storage key for this directory.
    #[inline]
    #[must_use]
    pub(crate) fn path(&self) -> &PathKey {
        &self.path
    }

    /// Returns the directory's name component.
    #[inline]
    #[must_use]
    pub(crate) fn name(&self) -> &DirName {
        &self.name
    }

    /// Returns the directory's filesystem metadata.
    #[inline]
    #[must_use]
    pub(crate) fn metadata(&self) -> &DirMetadata {
        &self.metadata
    }

    /// Returns the time at which this node was recorded in the index.
    #[inline]
    #[must_use]
    pub(crate) fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

#[cfg(test)]
mod tests {
    mod fs_node_id {
        mod constructor {
            use crate::indexer::model::FsNodeId;

            #[test]
            fn returns_unique_ids_on_each_call() {
                let id1 = FsNodeId::new();
                let id2 = FsNodeId::new();
                assert_ne!(id1, id2);
            }
        }

        mod ordering {
            use crate::indexer::model::FsNodeId;

            #[test]
            fn returns_ordered_ids_over_time() {
                let id1 = FsNodeId::new();
                let id2 = FsNodeId::new();
                assert!(id1 <= id2, "UuidV7 must be time-ordered");
            }
        }

        mod default {
            use crate::indexer::model::FsNodeId;

            #[test]
            fn default_creates_valid_id() {
                let id = FsNodeId::default();
                let id2 = FsNodeId::default();
                assert_ne!(id, id2, "default should create unique IDs");
            }
        }

        mod display {
            use crate::indexer::model::FsNodeId;

            #[test]
            fn formats_non_empty_string() {
                let id = FsNodeId::new();
                let s = id.to_string();
                assert!(!s.is_empty());
            }
        }
    }

    mod fs_node_type {
        mod constructor {
            use crate::indexer::model::FsNodeType;

            #[test]
            fn file_variant_is_distinguishable_from_dir() {
                assert_ne!(FsNodeType::File, FsNodeType::Dir);
            }

            #[test]
            fn clones_correctly() {
                let t = FsNodeType::File;
                assert_eq!(t, t.clone());
            }
        }
    }

    mod file_node {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    FileFormat,
                    metadata::{FileMetadata, FsTimes},
                    name::FileName,
                    path::PathKey,
                },
                indexer::model::{FileNode, FsNodeId},
            };

            fn make_file_node() -> FileNode {
                let id = FsNodeId::new();
                let parent_id = FsNodeId::new();
                let path = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                FileNode::new(
                    id,
                    parent_id,
                    path,
                    name,
                    format,
                    metadata,
                    recorded_at,
                )
            }

            #[test]
            fn returns_stored_id() {
                let id = FsNodeId::new();
                let parent_id = FsNodeId::new();
                let path = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                let node = FileNode::new(
                    id,
                    parent_id,
                    path,
                    name,
                    format,
                    metadata,
                    recorded_at,
                );
                assert_eq!(node.id(), id);
                assert_eq!(node.parent_id(), parent_id);
            }

            #[test]
            fn returns_stored_path() {
                let node = make_file_node();
                assert_eq!(node.path().as_str(), "notes/file.md");
            }

            #[test]
            fn returns_stored_format() {
                let node = make_file_node();
                assert_eq!(node.format(), FileFormat::Markdown);
            }
        }
    }

    mod dir_node {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    metadata::{DirMetadata, FsTimes},
                    name::DirName,
                    path::PathKey,
                },
                indexer::model::{DirNode, FsNodeId},
            };

            #[test]
            fn returns_stored_id() {
                let id = FsNodeId::new();
                let path = PathKey::try_new("notes").unwrap();
                let name = DirName::new("notes".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let recorded_at = SystemTime::now();
                let node =
                    DirNode::new(id, None, path, name, metadata, recorded_at);
                assert_eq!(node.id(), id);
                assert_eq!(node.parent_id(), None);
            }

            #[test]
            fn stores_parent_id_when_provided() {
                let id = FsNodeId::new();
                let parent_id = FsNodeId::new();
                let path = PathKey::try_new("notes/sub").unwrap();
                let name = DirName::new("sub".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let recorded_at = SystemTime::now();
                let node = DirNode::new(
                    id,
                    Some(parent_id),
                    path,
                    name,
                    metadata,
                    recorded_at,
                );
                assert_eq!(node.parent_id(), Some(parent_id));
            }
        }
    }

    mod path_conversions {

        use crate::fs::{DirPath, FilePath};

        #[test]
        fn file_path_requires_vault_root_to_produce_path_key() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let vault_root =
                DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();
            let file_path_buf = temp_dir.path().join("notes").join("file.md");
            std::fs::create_dir_all(file_path_buf.parent().unwrap()).unwrap();
            std::fs::File::create(&file_path_buf).unwrap();
            let file_path = FilePath::try_new(file_path_buf).unwrap();
            let key = file_path.as_key(&vault_root);
            assert!(key.is_ok());
            assert_eq!(key.unwrap().as_str(), "notes/file.md");
        }

        #[test]
        fn file_path_outside_vault_root_is_rejected() {
            let vault_dir = tempfile::TempDir::new().unwrap();
            let vault_root =
                DirPath::try_new(vault_dir.path().to_path_buf()).unwrap();

            let outside_dir = tempfile::TempDir::new().unwrap();
            let file_path_buf = outside_dir.path().join("file.md");
            std::fs::File::create(&file_path_buf).unwrap();
            let file_path = FilePath::try_new(file_path_buf).unwrap();

            let key = file_path.as_key(&vault_root);
            assert!(key.is_err());
        }
    }
}
