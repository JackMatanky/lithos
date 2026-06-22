//! Entry builder — 5-state typestate pattern for constructing index entries.
//!
//! Pipeline: `Init → Comparison → Persistence → Indexed → Completion`

#![expect(
    clippy::expect_used,
    clippy::unreachable,
    reason = "Type-level API invariants guarantee these unwraps are safe"
)]

use std::{collections::HashMap, time::SystemTime};

use crate::{
    fs::{
        DirNode, DirPath, FileFormat, FileNode,
        name::{DirName, FileName},
        path::PathKey,
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

// ─── State types
// ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct Init;

#[derive(Debug)]
pub(crate) struct FileComparison {
    pub(crate) path_key: PathKey,
}

#[derive(Debug)]
pub(crate) struct DirComparison {
    pub(crate) path_key: PathKey,
}

#[derive(Debug)]
pub(crate) struct FilePersistence {
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct DirPersistence {
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
}

#[derive(Debug)]
pub(crate) struct FileIndexed {
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
    pub(crate) record: FileRecord,
}

#[derive(Debug)]
pub(crate) struct DirIndexed {
    pub(crate) path_key: PathKey,
    pub(crate) status: IndexStatus,
    pub(crate) id: FsRecordId,
    pub(crate) record: DirRecord,
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

// ─── EntryBuilder
// ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct EntryBuilder<S> {
    pub(crate) entry: ScanEntry,
    pub(crate) state: S,
}

impl<S> EntryBuilder<S> {
    #[inline]
    #[must_use]
    pub(crate) fn into_parts(self) -> (ScanEntry, S) {
        (self.entry, self.state)
    }
}

#[expect(clippy::large_enum_variant, reason = "Short-lived stack type")]
pub(crate) enum EntryBranch {
    File(EntryBuilder<FileComparison>),
    Dir(EntryBuilder<DirComparison>),
    Completion(EntryBuilder<Completion>),
}

// ─── Init ──────────────────────────────────────────────────────────────────────

impl EntryBuilder<Init> {
    #[inline]
    #[must_use]
    pub(crate) fn from_scan_entry(entry: ScanEntry) -> Self {
        Self {
            entry,
            state: Init,
        }
    }

    pub(crate) fn transition(
        self,
        vault_root: &DirPath,
    ) -> Result<EntryBranch, IndexerError> {
        let (entry, _) = self.into_parts();
        match &entry {
            ScanEntry::File(node) => {
                let path_key = node.path().as_key(vault_root)?;
                Ok(EntryBranch::File(EntryBuilder {
                    entry,
                    state: FileComparison {
                        path_key,
                    },
                }))
            }
            ScanEntry::Dir(node) => {
                let path_key = node.path().as_key(vault_root)?;
                Ok(EntryBranch::Dir(EntryBuilder {
                    entry,
                    state: DirComparison {
                        path_key,
                    },
                }))
            }
            ScanEntry::Skipped(s) => {
                Ok(EntryBranch::Completion(EntryBuilder {
                    state: Completion {
                        kind: CompletionKind::Skipped(s.clone()),
                    },
                    entry,
                }))
            }
        }
    }
}

// ─── Comparison
// ────────────────────────────────────────────────────────────────

impl EntryBuilder<FileComparison> {
    pub(crate) fn transition(
        self,
        repo: &impl ReadRepository,
    ) -> Result<EntryBuilder<FilePersistence>, IndexerError> {
        let (entry, state) = self.into_parts();
        let ScanEntry::File(node) = &entry else {
            unreachable!("FileComparison strictly holds ScanEntry::File");
        };

        let existing = repo.find_file_by_path(&state.path_key)?;
        let status = classify_status(
            node.metadata(),
            existing.as_ref().map(FileRecord::metadata),
        );
        let id = match status {
            IndexStatus::New => FsRecordId::new(),
            IndexStatus::Fresh | IndexStatus::Stale => {
                existing.expect("Fresh/Stale implies existing").id()
            }
        };
        Ok(EntryBuilder {
            entry,
            state: FilePersistence {
                path_key: state.path_key,
                status,
                id,
            },
        })
    }
}

impl EntryBuilder<DirComparison> {
    pub(crate) fn transition(
        self,
        repo: &impl ReadRepository,
    ) -> Result<EntryBuilder<DirPersistence>, IndexerError> {
        let (entry, state) = self.into_parts();
        let ScanEntry::Dir(node) = &entry else {
            unreachable!("DirComparison strictly holds ScanEntry::Dir");
        };

        let existing = repo.find_dir_by_path(&state.path_key)?;
        let status = classify_status(
            node.metadata(),
            existing.as_ref().map(DirRecord::metadata),
        );
        let id = match status {
            IndexStatus::New => FsRecordId::new(),
            IndexStatus::Fresh | IndexStatus::Stale => {
                existing.expect("Fresh/Stale implies existing").id()
            }
        };
        Ok(EntryBuilder {
            entry,
            state: DirPersistence {
                path_key: state.path_key,
                status,
                id,
            },
        })
    }
}

// ─── Persistence
// ───────────────────────────────────────────────────────────────

impl EntryBuilder<FilePersistence> {
    pub(crate) fn transition(
        self,
        repo: &impl WriteRepository,
        dir_ids: &HashMap<PathKey, FsRecordId>,
        dry_run: bool,
    ) -> Result<EntryBuilder<FileIndexed>, IndexerError> {
        let (entry, state) = self.into_parts();
        let ScanEntry::File(node) = &entry else {
            unreachable!("FilePersistence strictly holds ScanEntry::File");
        };

        let parent_id = state
            .path_key
            .parent()
            .and_then(|pk| dir_ids.get(&pk).copied())
            .map_or(FsParentId::Root, FsParentId::Id);

        let name = FileName::new(
            node.path()
                .filename()
                .map(|n| n.as_str().to_owned())
                .unwrap_or_default()
                .into(),
        );
        let format = FileFormat::from_extension(
            node.path().as_ref().extension().unwrap_or_default(),
        );
        let record = FileRecord::new(
            state.id,
            parent_id,
            state.path_key.clone(),
            name,
            format,
            node.metadata().clone(),
            SystemTime::now(),
        );

        if state.status != IndexStatus::Fresh && !dry_run {
            repo.save_file(&record)?;
        }

        Ok(EntryBuilder {
            entry,
            state: FileIndexed {
                path_key: state.path_key,
                status: state.status,
                id: state.id,
                record,
            },
        })
    }
}

impl EntryBuilder<DirPersistence> {
    pub(crate) fn transition(
        self,
        repo: &impl WriteRepository,
        dir_ids: &HashMap<PathKey, FsRecordId>,
        dry_run: bool,
    ) -> Result<EntryBuilder<DirIndexed>, IndexerError> {
        let (entry, state) = self.into_parts();
        let ScanEntry::Dir(node) = &entry else {
            unreachable!("DirPersistence strictly holds ScanEntry::Dir");
        };

        let parent_id = state
            .path_key
            .parent()
            .and_then(|pk| dir_ids.get(&pk).copied())
            .map_or(FsParentId::Root, FsParentId::Id);

        let name = DirName::new(
            node.path()
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
            node.metadata().clone(),
            SystemTime::now(),
        );

        if state.status != IndexStatus::Fresh && !dry_run {
            repo.save_dir(&record)?;
        }

        Ok(EntryBuilder {
            entry,
            state: DirIndexed {
                path_key: state.path_key,
                status: state.status,
                id: state.id,
                record,
            },
        })
    }
}

// ─── Indexed
// ───────────────────────────────────────────────────────────────────

impl EntryBuilder<FileIndexed> {
    pub(crate) fn transition(self) -> EntryBuilder<Completion> {
        let (entry, state) = self.into_parts();
        let ScanEntry::File(node) = &entry else {
            unreachable!("FileIndexed strictly holds ScanEntry::File");
        };

        let index_entry = FileIndexEntry::new(
            state.id,
            state.record,
            node.path().clone(),
            state.status,
        );

        let kind = CompletionKind::File {
            entry: index_entry,
            path_key: state.path_key,
        };

        EntryBuilder {
            entry,
            state: Completion {
                kind,
            },
        }
    }
}

impl EntryBuilder<DirIndexed> {
    pub(crate) fn transition(self) -> EntryBuilder<Completion> {
        let (entry, state) = self.into_parts();
        let ScanEntry::Dir(node) = &entry else {
            unreachable!("DirIndexed strictly holds ScanEntry::Dir");
        };

        let index_entry = DirIndexEntry::new(
            state.id,
            state.record,
            node.path().clone(),
            state.status,
        );

        let kind = CompletionKind::Dir {
            entry: index_entry,
            path_key: state.path_key,
            id: state.id,
        };

        EntryBuilder {
            entry,
            state: Completion {
                kind,
            },
        }
    }
}

// ─── Helpers
// ───────────────────────────────────────────────────────────────────

fn classify_status<T: PartialEq>(
    current: &T,
    existing: Option<&T>,
) -> IndexStatus {
    match existing {
        None => IndexStatus::New,
        Some(e) if current == e => IndexStatus::Fresh,
        Some(_) => IndexStatus::Stale,
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
        let fp = FilePath::try_new(p).unwrap();
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
        let branch = builder.transition(&vault).unwrap();

        match branch {
            EntryBranch::File(b) => {
                assert_eq!(b.state.path_key.as_str(), "doc.md");
            }
            _ => panic!("expected File branch"),
        }
    }

    #[test]
    fn test_init_to_comparison_dir() {
        let vault = make_vault_root();
        let entry = make_dir_entry("/tmp/vault/notes");

        let builder = EntryBuilder::<Init>::from_scan_entry(entry);
        let branch = builder.transition(&vault).unwrap();

        match branch {
            EntryBranch::Dir(b) => {
                assert_eq!(b.state.path_key.as_str(), "notes");
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
            .transition(&vault)
            .unwrap();
        let EntryBranch::File(b) = branch else {
            panic!()
        };

        let b = b.transition(&repo).unwrap();
        assert_eq!(b.state.status, IndexStatus::New);

        let b = b.transition(&repo, &dir_ids, false).unwrap();

        // Record should be persisted
        let existing = repo.find_file_by_path(&b.state.path_key).unwrap();
        assert!(existing.is_some());

        let b = b.transition();
        let (_, state) = b.into_parts();

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
