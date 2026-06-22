//! Index result and summary types.
//!
//! Captures the outcome of an indexing run: the top-level `IndexResult`
//! along with its `IndexedNodes` and `DeletedNodes` aggregates. Scan-level
//! failures and skipped-entry records live in the report submodule.

use super::{
    entry::{DirIndexEntry, FileIndexEntry},
    model::FsRecordId,
    report::IndexReport,
};

/// The aggregate result of a single indexing run.
///
/// Contains the payload of indexed entries and deleted identifiers.
#[derive(Debug, Clone)]
pub(crate) struct IndexResult {
    indexed: IndexedNodes,
    deleted: DeletedNodes,
    report: IndexReport,
}

impl IndexResult {
    /// Creates a new index result.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        indexed: IndexedNodes,
        deleted: DeletedNodes,
        report: IndexReport,
    ) -> Self {
        Self {
            indexed,
            deleted,
            report,
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

    /// Returns the index report.
    #[inline]
    #[must_use]
    pub(crate) fn report(&self) -> &IndexReport {
        &self.report
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
    files: Box<[FsRecordId]>,
    dirs: Box<[FsRecordId]>,
}

impl DeletedNodes {
    /// Creates a new `DeletedNodes` record.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        files: Box<[FsRecordId]>,
        dirs: Box<[FsRecordId]>,
    ) -> Self {
        Self {
            files,
            dirs,
        }
    }

    /// Returns the IDs of deleted file nodes.
    #[inline]
    #[must_use]
    pub(crate) fn files(&self) -> &[FsRecordId] {
        &self.files
    }

    /// Returns the IDs of deleted directory nodes.
    #[inline]
    #[must_use]
    pub(crate) fn dirs(&self) -> &[FsRecordId] {
        &self.dirs
    }

    /// Returns the total number of deleted nodes.
    #[inline]
    #[must_use]
    pub(crate) fn count(&self) -> usize {
        self.files.len().saturating_add(self.dirs.len())
    }
}

#[cfg(test)]
mod tests {
    mod indexed_nodes {
        mod constructor {
            use std::time::SystemTime;

            use trace_fs::{
                FileFormat,
                metadata::{DirMetadata, FileMetadata, FsTimes},
                name::{DirName, FileName},
                path::{DirPath, FilePath, PathKey},
            };

            use crate::{
                entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
                model::{DirRecord, FileRecord, FsParentId, FsRecordId},
                summary::IndexedNodes,
            };

            fn make_file_entry(status: IndexStatus) -> FileIndexEntry {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let file_path_buf = temp_dir.path().join("file.md");
                std::fs::File::create(&file_path_buf).unwrap();

                let id = FsRecordId::new();
                let parent_id = FsParentId::Id(FsRecordId::new());
                let key = PathKey::try_new("notes/file.md").unwrap();
                let name = FileName::new("file.md".into());
                let format = FileFormat::Markdown;
                let metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let node = FileRecord::new(
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

                let id = FsRecordId::new();
                let key = PathKey::try_new("notes").unwrap();
                let name = DirName::new("notes".into());
                let metadata =
                    DirMetadata::new(FsTimes::new(None, None), false);
                let node = DirRecord::new(
                    id,
                    FsParentId::Root,
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

    mod deleted {
        mod constructor {
            use crate::{model::FsRecordId, summary::DeletedNodes};

            #[test]
            fn stores_file_and_dir_ids() {
                let f1 = FsRecordId::new();
                let d1 = FsRecordId::new();
                let deleted = DeletedNodes::new(Box::new([f1]), Box::new([d1]));
                assert_eq!(deleted.files(), &[f1]);
                assert_eq!(deleted.dirs(), &[d1]);
            }

            #[test]
            fn count_is_sum_of_files_and_dirs() {
                let f1 = FsRecordId::new();
                let f2 = FsRecordId::new();
                let d1 = FsRecordId::new();
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

    mod result {
        mod constructor {
            use crate::summary::{DeletedNodes, IndexResult, IndexedNodes};

            #[test]
            fn stores_indexed_and_deleted() {
                use crate::report::IndexReport;
                let indexed = IndexedNodes::new(Box::new([]), Box::new([]));
                let deleted = DeletedNodes::default();
                let report =
                    IndexReport::new(0, 0, 0, 0, 0, Box::new([]), Box::new([]));
                let result = IndexResult::new(indexed, deleted, report);
                assert_eq!(result.indexed().files().len(), 0);
                assert_eq!(result.deleted().count(), 0);
            }
        }
    }
}
