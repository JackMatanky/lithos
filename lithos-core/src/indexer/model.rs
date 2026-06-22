//! Indexer domain model: filesystem record identity and classification types.
//!
//! Provides the foundational types for representing indexed filesystem records
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

/// Stable identifier for an indexed filesystem record (file or directory).
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
pub(crate) struct FsRecordId(pub(crate) UuidV7);

impl FsRecordId {
    /// Zero sentinel for storage keys — deterministic UUID (nil, not v7).
    /// Never represents a real record; used only as the `Root` parent sentinel
    /// in index tables.
    pub(crate) const ZERO: Self = Self(UuidV7::ZERO);

    /// Creates a new random filesystem record identifier (UUID v7).
    #[inline]
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(UuidV7::new())
    }
}

impl Default for FsRecordId {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for FsRecordId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Classification of an indexed filesystem record as either file or directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FsRecordType {
    /// A regular file.
    File,
    /// A directory.
    Dir,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FsRecord {
    File(FileRecord),
    Dir(DirRecord),
}

/// An indexed file record with identity, path, and metadata.
///
/// Represents a discovered file within the index scope, capturing all
/// information needed for downstream indexing and staleness detection.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub(crate) struct FileRecord {
    id: FsRecordId,
    parent_id: FsParentId,
    path: PathKey,
    name: FileName,
    format: FileFormat,
    metadata: FileMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl FileRecord {
    /// Creates a new file record.
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain model requires all these fields to be fully \
                  initialized"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
        id: FsRecordId,
        parent_id: FsParentId,
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

    /// Returns the record's stable identifier.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> FsRecordId {
        self.id
    }

    /// Returns the parent directory's record identifier.
    #[inline]
    #[must_use]
    pub(crate) fn parent_id(&self) -> FsParentId {
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

    /// Returns the time at which this record was written to the index.
    #[inline]
    #[must_use]
    pub(crate) fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

/// An indexed directory record with identity, path, and metadata.
///
/// Represents a discovered directory within the index scope.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub(crate) struct DirRecord {
    id: FsRecordId,
    parent_id: FsParentId,
    path: PathKey,
    name: DirName,
    metadata: DirMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl DirRecord {
    /// Creates a new directory record.
    ///
    /// `parent_id` is [`FsParentId::Root`] for the vault root directory.
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain model requires all these fields to be fully \
                  initialized"
    )]
    #[must_use]
    pub(crate) fn new(
        id: FsRecordId,
        parent_id: FsParentId,
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

    /// Returns the record's stable identifier.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> FsRecordId {
        self.id
    }

    /// Returns the parent directory's record identifier, or
    /// [`FsParentId::Root`] for the vault root.
    #[inline]
    #[must_use]
    pub(crate) fn parent_id(&self) -> FsParentId {
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

    /// Returns the time at which this record was written to the index.
    #[inline]
    #[must_use]
    pub(crate) fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

/// Identifies the parent of a filesystem node during indexing.
///
/// `Root` represents the vault root itself — used when an entry is directly
/// in the vault root (no parent directory exists). `Id(id)` represents a
/// specific indexed directory that contains this entry.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub(crate) enum FsParentId {
    /// Entry is directly under the vault root.
    Root,
    /// Entry is inside a known indexed directory.
    Id(FsRecordId),
}

impl FsParentId {
    /// Converts this parent ID to a storage key suitable for index tables.
    ///
    /// `Root` maps to the zero sentinel (nobody can collide with it since
    /// [`FsRecordId::new`] generates random `UUIDv7` values). `Id(id)` returns
    /// the inner `FsRecordId` directly.
    #[inline]
    #[must_use]
    pub(crate) fn to_storage_key(self) -> FsRecordId {
        match self {
            Self::Root => FsRecordId::default(),
            Self::Id(id) => id,
        }
    }
}

#[cfg(test)]
mod tests {
    mod id {
        mod constructor {
            use crate::indexer::model::FsRecordId;

            #[test]
            fn returns_unique_ids_on_each_call() {
                let id1 = FsRecordId::new();
                let id2 = FsRecordId::new();
                assert_ne!(id1, id2);
            }
        }

        mod ordering {
            use crate::indexer::model::FsRecordId;

            #[test]
            fn returns_ordered_ids_over_time() {
                let id1 = FsRecordId::new();
                let id2 = FsRecordId::new();
                assert!(id1 <= id2, "UuidV7 must be time-ordered");
            }
        }

        mod default {
            use crate::indexer::model::FsRecordId;

            #[test]
            fn default_returns_zero_sentinel() {
                let id = FsRecordId::default();
                let id2 = FsRecordId::default();
                assert_eq!(
                    id, id2,
                    "default should return deterministic sentinel"
                );
                assert_eq!(id, FsRecordId::ZERO);
            }
        }

        mod display {
            use crate::indexer::model::FsRecordId;

            #[test]
            fn formats_non_empty_string() {
                let id = FsRecordId::new();
                let s = id.to_string();
                assert!(!s.is_empty());
            }
        }
    }

    mod fs_record_type {
        mod constructor {
            use crate::indexer::model::FsRecordType;

            #[test]
            fn file_variant_is_distinguishable_from_dir() {
                assert_ne!(FsRecordType::File, FsRecordType::Dir);
            }

            #[test]
            fn clones_correctly() {
                let t = FsRecordType::File;
                assert_eq!(t, t.clone());
            }
        }
    }

    mod file {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    FileFormat,
                    metadata::{FileMetadata, FsTimes},
                    name::FileName,
                    path::PathKey,
                },
                indexer::model::{FileRecord, FsParentId, FsRecordId},
            };

            fn make_file_record() -> FileRecord {
                let id = FsRecordId::new();
                let parent_id = FsParentId::Id(FsRecordId::new());
                let path = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                FileRecord::new(
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
                let id = FsRecordId::new();
                let parent_id = FsParentId::Id(FsRecordId::new());
                let path = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let recorded_at = SystemTime::now();
                let node = FileRecord::new(
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
                let node = make_file_record();
                assert_eq!(node.path().as_str(), "notes/file.md");
            }

            #[test]
            fn returns_stored_format() {
                let node = make_file_record();
                assert_eq!(node.format(), FileFormat::Markdown);
            }
        }
    }

    mod dir {
        mod constructor {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    metadata::{DirMetadata, FsTimes},
                    name::DirName,
                    path::PathKey,
                },
                indexer::model::{DirRecord, FsParentId, FsRecordId},
            };

            #[test]
            fn returns_stored_id() {
                let id = FsRecordId::new();
                let path = PathKey::try_new("notes").unwrap();
                let name = DirName::new("notes".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let recorded_at = SystemTime::now();
                let node = DirRecord::new(
                    id,
                    FsParentId::Root,
                    path,
                    name,
                    metadata,
                    recorded_at,
                );
                assert_eq!(node.id(), id);
                assert_eq!(node.parent_id(), FsParentId::Root);
            }

            #[test]
            fn stores_parent_id_when_provided() {
                let id = FsRecordId::new();
                let parent_id = FsRecordId::new();
                let path = PathKey::try_new("notes/sub").unwrap();
                let name = DirName::new("sub".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let recorded_at = SystemTime::now();
                let node = DirRecord::new(
                    id,
                    FsParentId::Id(parent_id),
                    path,
                    name,
                    metadata,
                    recorded_at,
                );
                assert_eq!(node.parent_id(), FsParentId::Id(parent_id));
            }
        }
    }

    mod fs_parent_id {
        use crate::indexer::model::{FsParentId, FsRecordId};

        #[test]
        fn root_to_storage_key_is_deterministic() {
            let k1 = FsParentId::Root.to_storage_key();
            let k2 = FsParentId::Root.to_storage_key();
            assert_eq!(k1, k2, "Root sentinel must be deterministic");
            assert_eq!(k1, FsRecordId::ZERO);
        }

        #[test]
        fn id_to_storage_key_returns_inner_id() {
            let inner = FsRecordId::new();
            let key = FsParentId::Id(inner).to_storage_key();
            assert_eq!(key, inner);
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
