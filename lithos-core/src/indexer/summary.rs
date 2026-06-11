//! Index result and summary types.
//!
//! Captures the outcome of an indexing run: new/fresh/stale entries,
//! deleted nodes, and any failures encountered during the scan.

use super::{
    entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
    model::{FsNodeId, FsNodeType},
};

/// The set of filesystem node IDs that were present in a prior index run but
/// no longer exist on disk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeletedNodes {
    files: Box<[FsNodeId]>,
    dirs: Box<[FsNodeId]>,
}

impl DeletedNodes {
    /// Creates a new `DeletedNodes` record.
    #[inline]
    #[must_use]
    pub fn new(files: Box<[FsNodeId]>, dirs: Box<[FsNodeId]>) -> Self {
        Self {
            files,
            dirs,
        }
    }

    /// Returns the IDs of deleted file nodes.
    #[inline]
    #[must_use]
    pub fn files(&self) -> &[FsNodeId] {
        &self.files
    }

    /// Returns the IDs of deleted directory nodes.
    #[inline]
    #[must_use]
    pub fn dirs(&self) -> &[FsNodeId] {
        &self.dirs
    }

    /// Returns the total number of deleted nodes.
    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        self.files.len().saturating_add(self.dirs.len())
    }
}

/// A failure record for a single filesystem node that could not be indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexNodeFailure {
    id: FsNodeId,
    kind: FsNodeType,
    error: Box<str>,
}

impl IndexNodeFailure {
    /// Creates a new failure record.
    #[inline]
    #[must_use]
    pub fn new(id: FsNodeId, kind: FsNodeType, error: Box<str>) -> Self {
        Self {
            id,
            kind,
            error,
        }
    }

    /// Returns the node identifier.
    #[inline]
    #[must_use]
    pub fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the node type (file or directory).
    #[inline]
    #[must_use]
    pub fn kind(&self) -> FsNodeType {
        self.kind
    }

    /// Returns the error message for this failure.
    #[inline]
    #[must_use]
    pub fn error(&self) -> &str {
        &self.error
    }
}

/// The aggregate result of a single indexing run.
///
/// Captures all entries discovered during the scan, the set of nodes that
/// have been deleted, and any per-node failures.
#[derive(Debug, Clone)]
pub struct IndexResult {
    file_entries: Box<[FileIndexEntry]>,
    dir_entries: Box<[DirIndexEntry]>,
    deleted: DeletedNodes,
    failures: Box<[IndexNodeFailure]>,
}

impl IndexResult {
    /// Creates a new index result.
    #[inline]
    #[must_use]
    pub fn new(
        file_entries: Box<[FileIndexEntry]>,
        dir_entries: Box<[DirIndexEntry]>,
        deleted: DeletedNodes,
        failures: Box<[IndexNodeFailure]>,
    ) -> Self {
        Self {
            file_entries,
            dir_entries,
            deleted,
            failures,
        }
    }

    /// Returns all file entries discovered during the scan.
    #[inline]
    #[must_use]
    pub fn file_entries(&self) -> &[FileIndexEntry] {
        &self.file_entries
    }

    /// Returns all directory entries discovered during the scan.
    #[inline]
    #[must_use]
    pub fn dir_entries(&self) -> &[DirIndexEntry] {
        &self.dir_entries
    }

    /// Returns the deleted node record.
    #[inline]
    #[must_use]
    pub fn deleted(&self) -> &DeletedNodes {
        &self.deleted
    }

    /// Returns any per-node failures encountered during the scan.
    #[inline]
    #[must_use]
    pub fn failures(&self) -> &[IndexNodeFailure] {
        &self.failures
    }

    /// Returns the total number of nodes scanned (files + directories).
    #[inline]
    #[must_use]
    pub fn scanned(&self) -> usize {
        self.file_entries.len().saturating_add(self.dir_entries.len())
    }

    /// Returns the count of file entries with [`IndexStatus::New`].
    #[inline]
    #[must_use]
    pub fn new_count(&self) -> usize {
        self.file_entries
            .iter()
            .filter(|e| e.status() == IndexStatus::New)
            .count()
            .saturating_add(
                self.dir_entries
                    .iter()
                    .filter(|e| e.status() == IndexStatus::New)
                    .count(),
            )
    }

    /// Returns the count of entries with [`IndexStatus::Fresh`].
    #[inline]
    #[must_use]
    pub fn fresh_count(&self) -> usize {
        self.file_entries
            .iter()
            .filter(|e| e.status() == IndexStatus::Fresh)
            .count()
            .saturating_add(
                self.dir_entries
                    .iter()
                    .filter(|e| e.status() == IndexStatus::Fresh)
                    .count(),
            )
    }

    /// Returns the count of entries with [`IndexStatus::Stale`].
    #[inline]
    #[must_use]
    pub fn stale_count(&self) -> usize {
        self.file_entries
            .iter()
            .filter(|e| e.status() == IndexStatus::Stale)
            .count()
            .saturating_add(
                self.dir_entries
                    .iter()
                    .filter(|e| e.status() == IndexStatus::Stale)
                    .count(),
            )
    }

    /// Returns the count of deleted nodes (files + directories).
    #[inline]
    #[must_use]
    pub fn deleted_count(&self) -> usize {
        self.deleted.count()
    }

    /// Returns the count of per-node failures.
    #[inline]
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.failures.len()
    }
}

#[cfg(test)]
mod tests {
    mod deleted_nodes {
        mod constructor {
            use crate::indexer::{model::FsNodeId, summary::DeletedNodes};

            #[test]
            fn stores_file_and_dir_ids() {
                let f1 = FsNodeId::new();
                let d1 = FsNodeId::new();
                let deleted = DeletedNodes::new(Box::new([f1]), Box::new([d1]));
                assert_eq!(deleted.files(), &[f1]);
                assert_eq!(deleted.dirs(), &[d1]);
            }

            #[test]
            fn count_is_sum_of_files_and_dirs() {
                let f1 = FsNodeId::new();
                let f2 = FsNodeId::new();
                let d1 = FsNodeId::new();
                let deleted =
                    DeletedNodes::new(Box::new([f1, f2]), Box::new([d1]));
                assert_eq!(deleted.count(), 3);
            }

            #[test]
            fn default_is_empty() {
                let deleted = DeletedNodes::default();
                assert_eq!(deleted.count(), 0);
            }
        }
    }

    mod index_node_failure {
        mod constructor {
            use crate::indexer::{
                model::{FsNodeId, FsNodeType},
                summary::IndexNodeFailure,
            };

            #[test]
            fn stores_id_kind_and_error() {
                let id = FsNodeId::new();
                let failure = IndexNodeFailure::new(
                    id,
                    FsNodeType::File,
                    "permission denied".into(),
                );
                assert_eq!(failure.id(), id);
                assert_eq!(failure.kind(), FsNodeType::File);
                assert_eq!(failure.error(), "permission denied");
            }
        }
    }

    mod index_result {
        mod summary_counts {
            use std::time::SystemTime;

            use crate::{
                fs::{
                    DirPath, FileFormat, FilePath,
                    metadata::{DirMetadata, FileMetadata, FsTimes},
                    name::{DirName, FileName},
                    path::PathKey,
                },
                indexer::{
                    entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
                    model::{DirNode, FileNode, FsNodeId},
                    summary::{DeletedNodes, IndexResult},
                },
            };

            fn make_file_entry(status: IndexStatus) -> FileIndexEntry {
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
                let node = FileNode::new(
                    id,
                    parent_id,
                    key,
                    name,
                    format,
                    metadata,
                    SystemTime::now(),
                );
                let path = FilePath::try_new(file_path_buf).unwrap();
                FileIndexEntry::new(id, node, path, status)
            }

            fn make_dir_entry(status: IndexStatus) -> DirIndexEntry {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let dir_path_buf = temp_dir.path().join("notes");
                std::fs::create_dir_all(&dir_path_buf).unwrap();

                let id = FsNodeId::new();
                let key = PathKey::try_new("notes").unwrap();
                let name = DirName::new("notes".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let node = DirNode::new(
                    id,
                    None,
                    key,
                    name,
                    metadata,
                    SystemTime::now(),
                );
                let path = DirPath::try_new(dir_path_buf).unwrap();
                DirIndexEntry::new(id, node, path, status)
            }

            #[test]
            fn scanned_is_sum_of_file_and_dir_entries() {
                let result = IndexResult::new(
                    Box::new([
                        make_file_entry(IndexStatus::New),
                        make_file_entry(IndexStatus::Fresh),
                    ]),
                    Box::new([make_dir_entry(IndexStatus::Stale)]),
                    DeletedNodes::default(),
                    Box::new([]),
                );
                assert_eq!(result.scanned(), 3);
            }

            #[test]
            fn new_count_counts_only_new_status_entries() {
                let result = IndexResult::new(
                    Box::new([
                        make_file_entry(IndexStatus::New),
                        make_file_entry(IndexStatus::Fresh),
                        make_file_entry(IndexStatus::Stale),
                    ]),
                    Box::new([make_dir_entry(IndexStatus::New)]),
                    DeletedNodes::default(),
                    Box::new([]),
                );
                assert_eq!(result.new_count(), 2);
            }

            #[test]
            fn fresh_count_counts_only_fresh_entries() {
                let result = IndexResult::new(
                    Box::new([
                        make_file_entry(IndexStatus::Fresh),
                        make_file_entry(IndexStatus::New),
                    ]),
                    Box::new([make_dir_entry(IndexStatus::Fresh)]),
                    DeletedNodes::default(),
                    Box::new([]),
                );
                assert_eq!(result.fresh_count(), 2);
            }

            #[test]
            fn stale_count_counts_only_stale_entries() {
                let result = IndexResult::new(
                    Box::new([make_file_entry(IndexStatus::Stale)]),
                    Box::new([
                        make_dir_entry(IndexStatus::Stale),
                        make_dir_entry(IndexStatus::Fresh),
                    ]),
                    DeletedNodes::default(),
                    Box::new([]),
                );
                assert_eq!(result.stale_count(), 2);
            }

            #[test]
            fn deleted_count_delegates_to_deleted_nodes() {
                let f1 = FsNodeId::new();
                let deleted = DeletedNodes::new(Box::new([f1]), Box::new([]));
                let result = IndexResult::new(
                    Box::new([]),
                    Box::new([]),
                    deleted,
                    Box::new([]),
                );
                assert_eq!(result.deleted_count(), 1);
            }

            #[test]
            fn failed_count_returns_failure_count() {
                use crate::indexer::{
                    model::FsNodeType, summary::IndexNodeFailure,
                };
                let id = FsNodeId::new();
                let failure = IndexNodeFailure::new(
                    id,
                    FsNodeType::File,
                    "read error".into(),
                );
                let result = IndexResult::new(
                    Box::new([]),
                    Box::new([]),
                    DeletedNodes::default(),
                    Box::new([failure]),
                );
                assert_eq!(result.failed_count(), 1);
            }
        }
    }
}
