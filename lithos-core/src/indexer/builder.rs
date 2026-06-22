//! Entry builder — 5-state typestate pattern for constructing index entries.
//!
//! Pipeline: `Init → Comparison → Persistence → Indexed → Completion`

use std::{collections::HashMap, time::SystemTime};

use crate::{
    fs::{
        DirNode, DirPath, FileFormat, FileNode,
        name::{DirName, FileName},
        path::{DirPath as FsDirPath, FilePath as FsFilePath, PathKey},
    },
    indexer::{
        entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
        error::IndexerError,
        model::{DirRecord, FileRecord, FsParentId, FsRecordId},
        port::ScanEntry,
        report::SkippedEntry,
        repository::{ReadRepository, WriteRepository},
    },
};

// ─── State types ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct Init {
    pub(crate) entry: ScanEntry,
}

#[derive(Debug)]
pub(crate) struct FileComparison {
    pub(crate) node: FileNode,
    pub(crate) path_key: PathKey,
}

#[derive(Debug)]
pub(crate) struct DirComparison {
    pub(crate) node: DirNode,
    pub(crate) path_key: PathKey,
}

#[derive(Debug)]
pub(crate) struct FilePersistence {
    pub(crate) node: FileNode,
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct DirPersistence {
    pub(crate) node: DirNode,
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct FileIndexed {
    pub(crate) record: FileRecord,
    pub(crate) path: FsFilePath,
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct DirIndexed {
    pub(crate) record: DirRecord,
    pub(crate) path: FsDirPath,
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct Completion {
    pub(crate) kind: CompletionKind,
}

#[derive(Debug)]
pub(crate) enum CompletionKind {
    File {
        entry: FileIndexEntry,
        path_key: PathKey,
    },
    Dir {
        entry: DirIndexEntry,
        path_key: PathKey,
        id: FsRecordId,
    },
    Skipped(SkippedEntry),
}

// ─── EntryBuilder ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct EntryBuilder<S> {
    state: S,
}

impl<S> EntryBuilder<S> {
    #[inline]
    #[must_use]
    pub(crate) fn state(&self) -> &S {
        &self.state
    }

    #[inline]
    #[must_use]
    pub(crate) fn into_state(self) -> S {
        self.state
    }
}

pub(crate) enum EntryBranch {
    File(EntryBuilder<FileComparison>),
    Dir(EntryBuilder<DirComparison>),
    Completion(EntryBuilder<Completion>),
}

pub(crate) enum FileComparisonBranch {
    Match(EntryBuilder<FileIndexed>),
    Mismatch(EntryBuilder<FilePersistence>),
}

pub(crate) enum DirComparisonBranch {
    Match(EntryBuilder<DirIndexed>),
    Mismatch(EntryBuilder<DirPersistence>),
}

// ─── Init ────────────────────────────────────────────────────────────────────

impl EntryBuilder<Init> {
    #[inline]
    #[must_use]
    pub(crate) fn from_scan_entry(entry: ScanEntry) -> Self {
        Self {
            state: Init {
                entry,
            },
        }
    }

    pub(crate) fn into_branch(
        self,
        vault_root: &DirPath,
    ) -> Result<EntryBranch, IndexerError> {
        let state = self.state;
        match state.entry {
            ScanEntry::File(node) => {
                let path_key = node.path().as_key(vault_root)?;
                Ok(EntryBranch::File(EntryBuilder {
                    state: FileComparison {
                        node,
                        path_key,
                    },
                }))
            }
            ScanEntry::Dir(node) => {
                let path_key = node.path().as_key(vault_root)?;
                Ok(EntryBranch::Dir(EntryBuilder {
                    state: DirComparison {
                        node,
                        path_key,
                    },
                }))
            }
            ScanEntry::Skipped(s) => {
                Ok(EntryBranch::Completion(EntryBuilder {
                    state: Completion {
                        kind: CompletionKind::Skipped(s),
                    },
                }))
            }
        }
    }
}

// ─── Comparison ──────────────────────────────────────────────────────────────

impl EntryBuilder<FileComparison> {
    pub(crate) fn into_comparison_branch(
        self,
        repo: &impl ReadRepository,
    ) -> Result<FileComparisonBranch, IndexerError> {
        let state = self.state;
        let existing = repo.find_file_by_path(&state.path_key)?;

        match existing {
            Some(record)
                if state
                    .node
                    .metadata()
                    .is_size_match(record.metadata().size())
                    && state.node.metadata().is_timestamp_match(
                        record.metadata().times().created_at(),
                        record.metadata().times().modified_at(),
                    ) =>
            {
                let id = record.id();
                Ok(FileComparisonBranch::Match(EntryBuilder {
                    state: FileIndexed {
                        record,
                        path: state.node.path().clone(),
                        path_key: state.path_key,
                        status: IndexStatus::Fresh,
                        id,
                    },
                }))
            }
            existing => {
                let (status, id) = match existing {
                    None => (IndexStatus::New, FsRecordId::new()),
                    Some(e) => (IndexStatus::Stale, e.id()),
                };
                Ok(FileComparisonBranch::Mismatch(EntryBuilder {
                    state: FilePersistence {
                        node: state.node,
                        path_key: state.path_key,
                        status,
                        id,
                    },
                }))
            }
        }
    }
}

impl EntryBuilder<DirComparison> {
    pub(crate) fn into_comparison_branch(
        self,
        repo: &impl ReadRepository,
    ) -> Result<DirComparisonBranch, IndexerError> {
        let state = self.state;
        let existing = repo.find_dir_by_path(&state.path_key)?;

        match existing {
            Some(record)
                if state
                    .node
                    .metadata()
                    .times()
                    .is_match(record.metadata().times()) =>
            {
                let id = record.id();
                Ok(DirComparisonBranch::Match(EntryBuilder {
                    state: DirIndexed {
                        record,
                        path: state.node.path().clone(),
                        path_key: state.path_key,
                        status: IndexStatus::Fresh,
                        id,
                    },
                }))
            }
            existing => {
                let (status, id) = match existing {
                    None => (IndexStatus::New, FsRecordId::new()),
                    Some(e) => (IndexStatus::Stale, e.id()),
                };
                Ok(DirComparisonBranch::Mismatch(EntryBuilder {
                    state: DirPersistence {
                        node: state.node,
                        path_key: state.path_key,
                        status,
                        id,
                    },
                }))
            }
        }
    }
}

// ─── Persistence ─────────────────────────────────────────────────────────────

impl EntryBuilder<FilePersistence> {
    pub(crate) fn into_indexed(
        self,
        repo: &impl WriteRepository,
        dir_ids: &HashMap<PathKey, FsRecordId>,
        dry_run: bool,
    ) -> Result<EntryBuilder<FileIndexed>, IndexerError> {
        let state = self.state;

        let parent_id = state
            .path_key
            .parent()
            .and_then(|pk| dir_ids.get(&pk).copied())
            .map_or(FsParentId::Root, FsParentId::Id);

        let name = FileName::new(
            state
                .node
                .path()
                .filename()
                .map(|n| n.as_str().to_owned())
                .unwrap_or_default()
                .into(),
        );
        let format = FileFormat::from_extension(
            state.node.path().as_ref().extension().unwrap_or_default(),
        );
        let record = FileRecord::new(
            state.id,
            parent_id,
            state.path_key.clone(),
            name,
            format,
            state.node.metadata().clone(),
            SystemTime::now(),
        );

        if state.status != IndexStatus::Fresh && !dry_run {
            repo.save_file(&record)?;
        }

        let path = state.node.path().clone();

        Ok(EntryBuilder {
            state: FileIndexed {
                record,
                path,
                path_key: state.path_key,
                status: state.status,
                id: state.id,
            },
        })
    }
}

impl EntryBuilder<DirPersistence> {
    pub(crate) fn into_indexed(
        self,
        repo: &impl WriteRepository,
        dir_ids: &HashMap<PathKey, FsRecordId>,
        dry_run: bool,
    ) -> Result<EntryBuilder<DirIndexed>, IndexerError> {
        let state = self.state;

        let parent_id = state
            .path_key
            .parent()
            .and_then(|pk| dir_ids.get(&pk).copied())
            .map_or(FsParentId::Root, FsParentId::Id);

        let name = DirName::new(
            state
                .node
                .path()
                .as_ref()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
                .into(),
        );
        let record = DirRecord::new(
            state.id,
            parent_id,
            state.path_key.clone(),
            name,
            state.node.metadata().clone(),
            SystemTime::now(),
        );

        if state.status != IndexStatus::Fresh && !dry_run {
            repo.save_dir(&record)?;
        }

        let path = state.node.path().clone();

        Ok(EntryBuilder {
            state: DirIndexed {
                record,
                path,
                path_key: state.path_key,
                status: state.status,
                id: state.id,
            },
        })
    }
}

// ─── Indexed
// ───────────────────────────────────────────────────────────────────

impl EntryBuilder<FileIndexed> {
    pub(crate) fn into_completion(self) -> EntryBuilder<Completion> {
        let state = self.state;
        let entry = FileIndexEntry::new(
            state.id,
            state.record,
            state.path,
            state.status,
        );
        let kind = CompletionKind::File {
            entry,
            path_key: state.path_key,
        };
        EntryBuilder {
            state: Completion {
                kind,
            },
        }
    }
}

impl EntryBuilder<DirIndexed> {
    pub(crate) fn into_completion(self) -> EntryBuilder<Completion> {
        let state = self.state;
        let entry = DirIndexEntry::new(
            state.id,
            state.record,
            state.path,
            state.status,
        );
        let kind = CompletionKind::Dir {
            entry,
            path_key: state.path_key,
            id: state.id,
        };
        EntryBuilder {
            state: Completion {
                kind,
            },
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic,
        clippy::shadow_unrelated,
        reason = "Test code often panics and shadows variables safely"
    )]

    use super::*;
    use crate::{
        fs::{
            metadata::{DirMetadata, FileMetadata, FsTimes},
            path::FilePath,
        },
        indexer::storage::InMemoryRepository,
    };

    fn make_vault_root() -> DirPath {
        std::fs::create_dir_all("/tmp/vault").unwrap();
        DirPath::try_new("/tmp/vault".into()).unwrap()
    }

    fn make_file_entry(path: &str) -> ScanEntry {
        let p = std::path::PathBuf::from(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::File::create(&p).unwrap();
        let fp = FsFilePath::try_new(p).unwrap();
        let meta = FileMetadata::new(
            FsTimes::new(Some(SystemTime::now()), None),
            100,
            false,
        );
        ScanEntry::File(FileNode::new(fp, meta))
    }

    fn make_dir_entry(path: &str) -> ScanEntry {
        let p = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&p).unwrap();
        let dp = DirPath::try_new(p).unwrap();
        let meta = DirMetadata::new(FsTimes::new(None, None), false);
        ScanEntry::Dir(DirNode::new(dp, meta))
    }

    #[test]
    fn test_init_to_comparison_file() {
        let vault = make_vault_root();
        let entry = make_file_entry("/tmp/vault/doc.md");

        let builder = EntryBuilder::<Init>::from_scan_entry(entry);
        let branch = builder.into_branch(&vault).unwrap();

        match branch {
            EntryBranch::File(b) => {
                assert_eq!(b.state().path_key.as_str(), "doc.md");
            }
            _ => panic!("expected File branch"),
        }
    }

    #[test]
    fn test_init_to_comparison_dir() {
        let vault = make_vault_root();
        let entry = make_dir_entry("/tmp/vault/notes");

        let builder = EntryBuilder::<Init>::from_scan_entry(entry);
        let branch = builder.into_branch(&vault).unwrap();

        match branch {
            EntryBranch::Dir(b) => {
                assert_eq!(b.state().path_key.as_str(), "notes");
            }
            _ => panic!("expected Dir branch"),
        }
    }

    #[test]
    fn test_full_pipeline_file_new() {
        let vault = make_vault_root();
        let dir_ids = HashMap::new();
        let repo = InMemoryRepository::new();
        let entry = make_file_entry("/tmp/vault/new.md");

        let branch = EntryBuilder::<Init>::from_scan_entry(entry)
            .into_branch(&vault)
            .unwrap();
        let EntryBranch::File(b) = branch else {
            panic!()
        };

        let FileComparisonBranch::Mismatch(b) =
            b.into_comparison_branch(&repo).unwrap()
        else {
            panic!("expected Mismatch")
        };
        assert_eq!(b.state().status, IndexStatus::New);

        let b = b.into_indexed(&repo, &dir_ids, false).unwrap();

        // Record should be persisted
        let existing = repo.find_file_by_path(&b.state().path_key).unwrap();
        assert!(existing.is_some());

        let b = b.into_completion();
        let state = b.into_state();

        match state.kind {
            CompletionKind::File {
                entry,
                path_key,
            } => {
                assert_eq!(entry.status(), IndexStatus::New);
                assert_eq!(path_key.as_str(), "new.md");
            }
            _ => panic!("Expected File completion"),
        }
    }
}
