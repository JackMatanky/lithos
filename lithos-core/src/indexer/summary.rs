//! Index result and summary types.
//!
//! Captures the outcome of an indexing run: new/fresh/stale entries,
//! deleted nodes, and any failures encountered during the scan.

use super::{
    entry::{DirIndexEntry, FileIndexEntry},
    model::{FsNodeId, FsNodeType},
};

/// The aggregate result of a single indexing run.
///
/// Contains the payload of indexed entries and deleted identifiers.
#[derive(Debug, Clone)]
pub(crate) struct IndexResult {
    indexed: IndexedNodes,
    deleted: DeletedNodes,
}

impl IndexResult {
    /// Creates a new index result.
    #[inline]
    #[must_use]
    pub(crate) fn new(indexed: IndexedNodes, deleted: DeletedNodes) -> Self {
        Self {
            indexed,
            deleted,
        }
    }

    /// Returns the indexed nodes.
    #[inline]
    #[must_use]
    pub(crate) fn indexed(&self) -> &IndexedNodes {
        &self.indexed
    }

    /// Returns the deleted node record.
    #[inline]
    #[must_use]
    pub(crate) fn deleted(&self) -> &DeletedNodes {
        &self.deleted
    }
}

/// A summary report containing metrics and failures from an indexer run.
#[derive(Debug, Clone)]
pub(crate) struct IndexReport {
    scanned: usize,
    new: usize,
    fresh: usize,
    stale: usize,
    deleted: usize,
    failures: Box<[IndexNodeFailure]>,
}

impl IndexReport {
    /// Creates a new index report.
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain report encapsulates all summary counters"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
        scanned: usize,
        new: usize,
        fresh: usize,
        stale: usize,
        deleted: usize,
        failures: Box<[IndexNodeFailure]>,
    ) -> Self {
        Self {
            scanned,
            new,
            fresh,
            stale,
            deleted,
            failures,
        }
    }

    /// Returns the total number of nodes scanned.
    #[inline]
    #[must_use]
    pub(crate) fn scanned(&self) -> usize {
        self.scanned
    }

    /// Returns the count of new nodes.
    #[inline]
    #[must_use]
    pub(crate) fn new_count(&self) -> usize {
        self.new
    }

    /// Returns the count of fresh nodes.
    #[inline]
    #[must_use]
    pub(crate) fn fresh_count(&self) -> usize {
        self.fresh
    }

    /// Returns the count of stale nodes.
    #[inline]
    #[must_use]
    pub(crate) fn stale_count(&self) -> usize {
        self.stale
    }

    /// Returns the count of deleted nodes.
    #[inline]
    #[must_use]
    pub(crate) fn deleted_count(&self) -> usize {
        self.deleted
    }

    /// Returns the failures encountered during the scan.
    #[inline]
    #[must_use]
    pub(crate) fn failures(&self) -> &[IndexNodeFailure] {
        &self.failures
    }
}

/// The set of successfully indexed filesystem nodes.
#[derive(Debug, Clone)]
pub(crate) struct IndexedNodes {
    files: Box<[FileIndexEntry]>,
    dirs: Box<[DirIndexEntry]>,
}

impl IndexedNodes {
    /// Creates a new `IndexedNodes` collection.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        files: Box<[FileIndexEntry]>,
        dirs: Box<[DirIndexEntry]>,
    ) -> Self {
        Self {
            files,
            dirs,
        }
    }

    /// Returns the indexed file entries.
    #[inline]
    #[must_use]
    pub(crate) fn files(&self) -> &[FileIndexEntry] {
        &self.files
    }

    /// Returns the indexed directory entries.
    #[inline]
    #[must_use]
    pub(crate) fn dirs(&self) -> &[DirIndexEntry] {
        &self.dirs
    }
}

/// The set of filesystem node IDs that were present in a prior index run but
/// no longer exist on disk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DeletedNodes {
    files: Box<[FsNodeId]>,
    dirs: Box<[FsNodeId]>,
}

impl DeletedNodes {
    /// Creates a new `DeletedNodes` record.
    #[inline]
    #[must_use]
    pub(crate) fn new(files: Box<[FsNodeId]>, dirs: Box<[FsNodeId]>) -> Self {
        Self {
            files,
            dirs,
        }
    }

    /// Returns the IDs of deleted file nodes.
    #[inline]
    #[must_use]
    pub(crate) fn files(&self) -> &[FsNodeId] {
        &self.files
    }

    /// Returns the IDs of deleted directory nodes.
    #[inline]
    #[must_use]
    pub(crate) fn dirs(&self) -> &[FsNodeId] {
        &self.dirs
    }

    /// Returns the total number of deleted nodes.
    #[inline]
    #[must_use]
    pub(crate) fn count(&self) -> usize {
        self.files.len().saturating_add(self.dirs.len())
    }
}

/// A failure record for a single filesystem node that could not be indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexNodeFailure {
    id: FsNodeId,
    kind: FsNodeType,
    error: Box<str>,
}

impl IndexNodeFailure {
    /// Creates a new failure record.
    #[inline]
    #[must_use]
    pub(crate) fn new(id: FsNodeId, kind: FsNodeType, error: Box<str>) -> Self {
        Self {
            id,
            kind,
            error,
        }
    }

    /// Returns the node identifier.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> FsNodeId {
        self.id
    }

    /// Returns the node type (file or directory).
    #[inline]
    #[must_use]
    pub(crate) fn kind(&self) -> FsNodeType {
        self.kind
    }

    /// Returns the error message for this failure.
    #[inline]
    #[must_use]
    pub(crate) fn error(&self) -> &str {
        &self.error
    }
}

#[cfg(test)]
mod tests {
    mod indexed_nodes {
        mod constructor {
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
                    summary::IndexedNodes,
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
            fn stores_files_and_dirs() {
                let file = make_file_entry(IndexStatus::New);
                let dir = make_dir_entry(IndexStatus::New);

                let indexed = IndexedNodes::new(
                    Box::new([file.clone()]),
                    Box::new([dir.clone()]),
                );

                assert_eq!(indexed.files().len(), 1);
                assert_eq!(indexed.dirs().len(), 1);
            }
        }
    }

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

    mod index_report {
        mod constructor {
            use crate::indexer::summary::IndexReport;

            #[test]
            fn stores_counts_and_failures() {
                let report = IndexReport::new(10, 2, 5, 3, 1, Box::new([]));
                assert_eq!(report.scanned(), 10);
                assert_eq!(report.new_count(), 2);
                assert_eq!(report.fresh_count(), 5);
                assert_eq!(report.stale_count(), 3);
                assert_eq!(report.deleted_count(), 1);
                assert_eq!(report.failures().len(), 0);
            }
        }
    }

    mod index_result {
        mod constructor {
            use crate::indexer::summary::{
                DeletedNodes, IndexResult, IndexedNodes,
            };

            #[test]
            fn stores_indexed_and_deleted() {
                let indexed = IndexedNodes::new(Box::new([]), Box::new([]));
                let deleted = DeletedNodes::default();
                let result = IndexResult::new(indexed, deleted);
                assert_eq!(result.indexed().files().len(), 0);
                assert_eq!(result.deleted().count(), 0);
            }
        }
    }
}
