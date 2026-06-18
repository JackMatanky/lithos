//! Entry builder — typestate pattern for constructing index entries.
//!
//! Provides `EntryBuilder<F, S>` with two type parameters:
//! - `F`: `File` or `Dir` — distinguishes file vs directory entry
//! - `S`: `Unclassified` or `Classified<E>` — scanned vs resolved state
//!
//! Only `EntryBuilder<File, Classified<FileIndexEntry>>` can `.build()` to
//! `FileIndexEntry`; only `EntryBuilder<Dir, Classified<DirIndexEntry>>` can
//! `.build()` to `DirIndexEntry`. The typestate guarantees that consumers
//! always see a resolved `PathKey` — no `Option`, no runtime match on an
//! `IndexedEntry` enum.

use std::marker::PhantomData;

use crate::{
    fs::{DirNode, DirPath, FileNode, FilePath, FsNode, path::PathKey},
    indexer::{
        entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
        error::IndexerError,
        model::{DirRecord, FileRecord, FsParentId, FsRecordId},
        repository::{ReadRepository, WriteRepository},
    },
};

// ─── Type-state markers ──────────────────────────────────

/// Marker: file entry kind.
#[derive(Debug)]
pub(crate) struct File;

/// Marker: directory entry kind.
#[derive(Debug)]
pub(crate) struct Dir;

/// State: raw `FsNode` from the scanner stream (not yet classified).
#[derive(Debug)]
pub(crate) struct Unclassified(FsNode);

/// State: resolved `PathKey` and fully built entry.
#[derive(Debug)]
pub(crate) struct Classified<E> {
    entry: E,
    path_key: PathKey,
}

// ─── EntryBuilder ────────────────────────────────────────

/// Typestate builder for index entries.
///
/// `F` is the entry kind (`File` or `Dir`), `S` is the state
/// (`Unclassified` or `Classified<E>`). The builder consumes the raw
/// `FsNode` during `classify()` and produces a fully typed entry via
/// `build()`.
#[derive(Debug)]
pub(crate) struct EntryBuilder<F, S> {
    vault_root: DirPath,
    parent_id: FsParentId,
    state: S,
    _kind: PhantomData<F>,
}

// ─── File: Unclassified → Classified ─────────────────────

impl EntryBuilder<File, Unclassified> {
    /// Wrap a `FileNode` from the scan stream.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        node: FileNode,
        vault_root: DirPath,
        parent_id: FsParentId,
    ) -> Self {
        Self {
            vault_root,
            parent_id,
            state: Unclassified(FsNode::File(node)),
            _kind: PhantomData,
        }
    }

    /// Resolve vault-relative `PathKey`, query repository, classify status
    /// (New/Fresh/Stale), and build the `FileIndexEntry`.
    #[expect(
        clippy::unreachable,
        reason = "type-level invariant: EntryBuilder<File> always holds \
                  FileNode"
    )]
    pub(crate) fn classify<R: ReadRepository>(
        self,
        repo: &R,
    ) -> Result<EntryBuilder<File, Classified<FileIndexEntry>>, IndexerError>
    {
        let FsNode::File(file) = self.state.0 else {
            unreachable!("EntryBuilder<File> always holds FileNode")
        };
        let key = file.path().as_key(&self.vault_root)?;
        let existing = repo.find_file_by_path(&key)?;
        let status = classify_status(
            file.metadata(),
            existing.as_ref().map(FileRecord::metadata),
        );
        let record = build_file_record(
            &file,
            &key,
            self.parent_id,
            status,
            existing.as_ref().map(FileRecord::id),
        );
        let entry = FileIndexEntry::new(
            record.id(),
            record,
            file.path().clone(),
            status,
        );
        Ok(EntryBuilder {
            vault_root: self.vault_root,
            parent_id: self.parent_id,
            state: Classified {
                entry,
                path_key: key,
            },
            _kind: PhantomData,
        })
    }
}

// ─── Dir: Unclassified → Classified ──────────────────────

impl EntryBuilder<Dir, Unclassified> {
    /// Wrap a `DirNode` from the scan stream.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        node: DirNode,
        vault_root: DirPath,
        parent_id: FsParentId,
    ) -> Self {
        Self {
            vault_root,
            parent_id,
            state: Unclassified(FsNode::Dir(node)),
            _kind: PhantomData,
        }
    }

    /// Resolve vault-relative `PathKey`, query repository, classify status,
    /// and build the `DirIndexEntry`.
    #[expect(
        clippy::unreachable,
        reason = "type-level invariant: EntryBuilder<Dir> always holds DirNode"
    )]
    pub(crate) fn classify<R: ReadRepository>(
        self,
        repo: &R,
    ) -> Result<EntryBuilder<Dir, Classified<DirIndexEntry>>, IndexerError>
    {
        let FsNode::Dir(dir) = self.state.0 else {
            unreachable!("EntryBuilder<Dir> always holds DirNode")
        };
        let key = dir.path().as_key(&self.vault_root)?;
        let existing = repo.find_dir_by_path(&key)?;
        let status = classify_status(
            dir.metadata(),
            existing.as_ref().map(DirRecord::metadata),
        );
        let record = build_dir_record(
            &dir,
            &key,
            self.parent_id,
            status,
            existing.as_ref().map(DirRecord::id),
        );
        let entry =
            DirIndexEntry::new(record.id(), record, dir.path().clone(), status);
        Ok(EntryBuilder {
            vault_root: self.vault_root,
            parent_id: self.parent_id,
            state: Classified {
                entry,
                path_key: key,
            },
            _kind: PhantomData,
        })
    }
}

// ─── File: Classified — accessors and finaliser ──────────

impl EntryBuilder<File, Classified<FileIndexEntry>> {
    /// The resolved vault-relative `PathKey`.
    #[inline]
    #[must_use]
    pub(crate) fn path_key(&self) -> &PathKey {
        &self.state.path_key
    }

    /// The `FsRecordId` of the classified entry.
    #[inline]
    #[must_use]
    pub(crate) fn entry_id(&self) -> FsRecordId {
        self.state.entry.id()
    }

    /// Always `false` for file entries.
    #[inline]
    #[must_use]
    #[allow(
        clippy::unused_self,
        reason = "type-level API: EntryBuilder<File> is_dir always false"
    )]
    pub(crate) fn is_dir(&self) -> bool {
        false
    }

    /// The index classification status.
    #[inline]
    #[must_use]
    pub(crate) fn status(&self) -> IndexStatus {
        self.state.entry.status()
    }

    /// Finalise and return the `FileIndexEntry`.
    #[inline]
    #[must_use]
    pub(crate) fn build(self) -> FileIndexEntry {
        self.state.entry
    }
}

// ─── Dir: Classified — accessors and finaliser ───────────

impl EntryBuilder<Dir, Classified<DirIndexEntry>> {
    /// The resolved vault-relative `PathKey`.
    #[inline]
    #[must_use]
    pub(crate) fn path_key(&self) -> &PathKey {
        &self.state.path_key
    }

    /// The `FsRecordId` of the classified entry.
    #[inline]
    #[must_use]
    pub(crate) fn entry_id(&self) -> FsRecordId {
        self.state.entry.id()
    }

    /// Always `true` for directory entries.
    #[inline]
    #[must_use]
    #[allow(
        clippy::unused_self,
        reason = "type-level API: EntryBuilder<Dir> is_dir always true"
    )]
    pub(crate) fn is_dir(&self) -> bool {
        true
    }

    /// The index classification status.
    #[inline]
    #[must_use]
    pub(crate) fn status(&self) -> IndexStatus {
        self.state.entry.status()
    }

    /// Finalise and return the `DirIndexEntry`.
    #[inline]
    #[must_use]
    pub(crate) fn build(self) -> DirIndexEntry {
        self.state.entry
    }
}

// ─── Helper functions ────────────────────────────────────

/// Classify an entry's status by comparing current metadata against the
/// persisted record (if any).
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

/// Build a `FileRecord` from the scanned data, parent info, and status.
///
/// For `New` entries a fresh `FsRecordId` is generated. For `Fresh`/`Stale`
/// the existing ID is reused so the record identity is preserved across runs.
#[expect(
    clippy::expect_used,
    reason = "existing_id is always Some for Fresh/Stale — validated by caller"
)]
fn build_file_record(
    file: &FileNode,
    key: &PathKey,
    parent_id: FsParentId,
    status: IndexStatus,
    existing_id: Option<FsRecordId>,
) -> FileRecord {
    use std::time::SystemTime;

    use crate::{fs::name::FileName, indexer::model::FileRecord};

    let id = match status {
        IndexStatus::New => FsRecordId::new(),
        IndexStatus::Fresh | IndexStatus::Stale => existing_id
            .expect("existing_id must be provided for Fresh/Stale entries"),
    };
    let name = FileName::new(
        file.path()
            .filename()
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default()
            .into(),
    );
    let format = crate::fs::FileFormat::from_extension(
        file.path().as_ref().extension().unwrap_or_default(),
    );

    FileRecord::new(
        id,
        parent_id,
        key.clone(),
        name,
        format,
        file.metadata().clone(),
        SystemTime::now(),
    )
}

/// Build a `DirRecord` from the scanned data, parent info, and status.
///
/// For `New` entries a fresh `FsRecordId` is generated. For `Fresh`/`Stale`
/// the existing ID is reused so the record identity is preserved across runs.
#[expect(
    clippy::expect_used,
    reason = "existing_id is always Some for Fresh/Stale — validated by caller"
)]
fn build_dir_record(
    dir: &DirNode,
    key: &PathKey,
    parent_id: FsParentId,
    status: IndexStatus,
    existing_id: Option<FsRecordId>,
) -> DirRecord {
    use std::time::SystemTime;

    use crate::{fs::name::DirName, indexer::model::DirRecord};

    let id = match status {
        IndexStatus::New => FsRecordId::new(),
        IndexStatus::Fresh | IndexStatus::Stale => existing_id
            .expect("existing_id must be provided for Fresh/Stale entries"),
    };
    let name = DirName::new(
        dir.path()
            .as_ref()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
            .into(),
    );

    DirRecord::new(
        id,
        parent_id,
        key.clone(),
        name,
        dir.metadata().clone(),
        SystemTime::now(),
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, time::SystemTime};

    use super::*;
    use crate::{
        fs::{
            DirPath, FileFormat, FilePath,
            metadata::{DirMetadata, FileMetadata, FsTimes},
            name::{DirName, FileName},
            path::PathKey,
        },
        indexer::{
            entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
            error::IndexerError,
            model::{DirRecord, FileRecord, FsParentId, FsRecordId},
            repository::WriteRepository,
            scan::{IndexOptions, IndexScope, ScanFilters},
            storage::InMemoryRepository,
        },
    };

    // ─── Helpers ────────────────────────────────────────────────

    fn make_vault_root() -> DirPath {
        std::fs::create_dir_all("/tmp/vault").unwrap();
        DirPath::try_new("/tmp/vault".into()).unwrap()
    }

    fn make_file_node(path: &str) -> FileNode {
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
        FileNode::new(fp, meta)
    }

    fn make_dir_node(path: &str) -> DirNode {
        let p = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&p).unwrap();
        let dp = DirPath::try_new(p).unwrap();
        let meta = DirMetadata::new(FsTimes::new(None, None), false);
        DirNode::new(dp, meta)
    }

    fn empty_repo() -> InMemoryRepository {
        InMemoryRepository::new()
    }

    fn repo_with_file(path: &PathKey) -> InMemoryRepository {
        let repo = InMemoryRepository::new();
        let id = FsRecordId::new();
        let name = FileName::new(
            path.as_str().rsplit('/').next().unwrap_or("file").into(),
        );
        let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
        let record = FileRecord::new(
            id,
            FsParentId::Root,
            path.clone(),
            name,
            FileFormat::Unknown,
            meta.clone(),
            SystemTime::now(),
        );
        repo.save_file(&record).unwrap();
        repo
    }

    fn repo_with_dir(path: &PathKey) -> InMemoryRepository {
        let repo = InMemoryRepository::new();
        let id = FsRecordId::new();
        let name = DirName::new(
            path.as_str().rsplit('/').next().unwrap_or("dir").into(),
        );
        let meta = DirMetadata::new(FsTimes::new(None, None), false);
        let record = DirRecord::new(
            id,
            FsParentId::Root,
            path.clone(),
            name,
            meta.clone(),
            SystemTime::now(),
        );
        repo.save_dir(&record).unwrap();
        repo
    }

    // ─── EntryBuilder tests ──────────────────────────────────

    mod entry_builder {
        use super::*;

        #[test]
        fn file_new_wraps_file_node() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let _ = builder;
        }

        #[test]
        fn dir_new_wraps_dir_node() {
            let vault = make_vault_root();
            let dir_node = make_dir_node("/tmp/vault/sub");
            let builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let _ = builder;
        }

        #[test]
        fn classify_file_new() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/notes/new.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::New);
        }

        #[test]
        fn classify_file_fresh() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes/new.md").unwrap();
            let repo = repo_with_file(&key);
            let file_path = vault.as_path().join("notes/new.md");
            let fp = FilePath::try_new(file_path).unwrap();
            let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
            let file_node = FileNode::new(fp, meta);
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::Fresh);
        }

        #[test]
        fn classify_file_stale() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes/new.md").unwrap();
            let repo = {
                let r = InMemoryRepository::new();
                let id = FsRecordId::new();
                let name = FileName::new("new.md".into());
                let meta =
                    FileMetadata::new(FsTimes::new(None, None), 200, false);
                let record = FileRecord::new(
                    id,
                    FsParentId::Root,
                    key.clone(),
                    name,
                    FileFormat::Unknown,
                    meta,
                    SystemTime::now(),
                );
                r.save_file(&record).unwrap();
                r
            };
            let file_path = vault.as_path().join("notes/new.md");
            let fp = FilePath::try_new(file_path).unwrap();
            let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
            let file_node = FileNode::new(fp, meta);
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::Stale);
        }

        #[test]
        fn classify_dir_new() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let dir_node = make_dir_node("/tmp/vault/notes");
            let builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::New);
        }

        #[test]
        fn classify_dir_fresh() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes").unwrap();
            let repo = {
                let r = InMemoryRepository::new();
                let id = FsRecordId::new();
                let name = DirName::new("notes".into());
                let meta = DirMetadata::new(FsTimes::new(None, None), false);
                let record = DirRecord::new(
                    id,
                    FsParentId::Root,
                    key,
                    name,
                    meta.clone(),
                    SystemTime::now(),
                );
                r.save_dir(&record).unwrap();
                r
            };
            let dir_path = vault.as_path().join("notes");
            let dp = DirPath::try_new(dir_path).unwrap();
            let meta = DirMetadata::new(FsTimes::new(None, None), false);
            let dir_node = DirNode::new(dp, meta);
            let builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::Fresh);
        }

        #[test]
        fn classify_dir_stale() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes").unwrap();
            let repo = {
                let r = InMemoryRepository::new();
                let id = FsRecordId::new();
                let name = DirName::new("notes".into());
                let meta = DirMetadata::new(
                    FsTimes::new(Some(SystemTime::now()), None),
                    false,
                );
                let record = DirRecord::new(
                    id,
                    FsParentId::Root,
                    key,
                    name,
                    meta,
                    SystemTime::now(),
                );
                r.save_dir(&record).unwrap();
                r
            };
            let dir_path = vault.as_path().join("notes");
            let dp = DirPath::try_new(dir_path).unwrap();
            let meta = DirMetadata::new(FsTimes::new(None, None), false);
            let dir_node = DirNode::new(dp, meta);
            let builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.status(), IndexStatus::Stale);
        }

        #[test]
        fn classify_handles_outside_path() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/other/file.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let result = builder.classify(&repo);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), IndexerError::Path(_)));
        }

        #[test]
        fn classified_path_key() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            assert_eq!(classified.path_key().as_str(), "doc.md");
        }

        #[test]
        fn classified_is_dir() {
            let vault = make_vault_root();
            let repo = empty_repo();

            let file_node = make_file_node("/tmp/vault/doc.md");
            let file_builder = EntryBuilder::<File, _>::new(
                file_node,
                vault.clone(),
                FsParentId::Root,
            );
            let file_classified =
                file_builder.classify(&repo).expect("classify should succeed");
            assert!(!file_classified.is_dir());

            let dir_node = make_dir_node("/tmp/vault/sub");
            let dir_builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let dir_classified =
                dir_builder.classify(&repo).expect("classify should succeed");
            assert!(dir_classified.is_dir());
        }

        #[test]
        fn classified_entry_id() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            let _id = classified.entry_id();
        }

        #[test]
        fn build_file_entry() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let builder = EntryBuilder::<File, _>::new(
                file_node,
                vault,
                FsParentId::Root,
            );
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            let entry: FileIndexEntry = classified.build();
            assert_eq!(entry.status(), IndexStatus::New);
        }

        #[test]
        fn build_dir_entry() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let dir_node = make_dir_node("/tmp/vault/sub");
            let builder =
                EntryBuilder::<Dir, _>::new(dir_node, vault, FsParentId::Root);
            let classified =
                builder.classify(&repo).expect("classify should succeed");
            let entry: DirIndexEntry = classified.build();
            assert_eq!(entry.status(), IndexStatus::New);
        }
    }

    // ─── Helper tests ────────────────────────────────────────

    mod helpers {
        use super::*;

        #[test]
        fn classify_status_new_when_missing() {
            let status = super::classify_status::<FileMetadata>(
                &FileMetadata::new(FsTimes::new(None, None), 100, false),
                None,
            );
            assert_eq!(status, IndexStatus::New);
        }

        #[test]
        fn classify_status_fresh_when_matching() {
            let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
            let status = super::classify_status(&meta, Some(&meta));
            assert_eq!(status, IndexStatus::Fresh);
        }

        #[test]
        fn classify_status_stale_when_different() {
            let current =
                FileMetadata::new(FsTimes::new(None, None), 100, false);
            let existing =
                FileMetadata::new(FsTimes::new(None, None), 200, false);
            let status = super::classify_status(&current, Some(&existing));
            assert_eq!(status, IndexStatus::Stale);
        }

        #[test]
        fn build_file_record_creates_new_record() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let key = file_node.path().as_key(&vault).unwrap();
            let record = super::build_file_record(
                &file_node,
                &key,
                FsParentId::Root,
                IndexStatus::New,
                None,
            );
            assert_eq!(record.path().as_str(), "doc.md");
        }

        #[test]
        fn build_file_record_reuses_id_for_fresh() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let key = file_node.path().as_key(&vault).unwrap();
            let existing_id = FsRecordId::new();
            let record = super::build_file_record(
                &file_node,
                &key,
                FsParentId::Root,
                IndexStatus::Fresh,
                Some(existing_id),
            );
            assert_eq!(record.id(), existing_id);
        }

        #[test]
        fn build_file_record_reuses_id_for_stale() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/stale.md");
            let key = file_node.path().as_key(&vault).unwrap();
            let existing_id = FsRecordId::new();
            let record = super::build_file_record(
                &file_node,
                &key,
                FsParentId::Root,
                IndexStatus::Stale,
                Some(existing_id),
            );
            assert_eq!(record.id(), existing_id);
        }

        #[test]
        fn build_dir_record_creates_new_record() {
            let vault = make_vault_root();
            let dir_node = make_dir_node("/tmp/vault/sub");
            let key = dir_node.path().as_key(&vault).unwrap();
            let record = super::build_dir_record(
                &dir_node,
                &key,
                FsParentId::Root,
                IndexStatus::New,
                None,
            );
            assert_eq!(record.path().as_str(), "sub");
            assert_eq!(record.parent_id(), FsParentId::Root);
        }
    }
}
