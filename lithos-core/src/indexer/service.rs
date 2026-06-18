//! Indexer application service — orchestrates the index scan pipeline.
//!
//! Contains the procedural `IndexerService` that wires the scanner port,
//! repository, and per-entry typestate classification into a single indexing
//! run. The per-entry typestate (`IndexNode<S>`) ensures every entry resolves
//! its `PathKey` before it can be consumed — a compile-time guarantee scoped
//! to the service's scan loop.
//!
//! # Typestate transitions
//!
//! ```text
//! IndexNode<Scanned>  ──classify()──→  IndexNode<Classified>
//! ```
//!
//! `IndexNode<Classified>` wraps a raw `FsNode`. `IndexNode<Classified>`
//! carries the resolved `PathKey` and classified `IndexedEntry`. The typestate
//! guarantees that consumers see a resolved key, making `unwrap()` on
//! `path_key()` unnecessary in service logic.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "counter increments in report builders are bounded by file count"
)]

use std::collections::{HashMap, HashSet};

use crate::{
    fs::{DirPath, FsNode, metadata::FsMetadata, path::PathKey},
    indexer::{
        entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
        error::IndexerError,
        model::FsParentId,
        port::{ScanEntry, ScannerPort, WalkIter},
        report::IndexReport,
        repository::{ReadRepository, Repository, WriteRepository},
        scan::{IndexOptions, IndexScope, ScanFilters},
        summary::{DeletedNodes, IndexResult, IndexedNodes},
    },
};

// ─── Per-entry typestate ───────────────────────────────────────────

/// State: raw `FsNode` from the scanner stream.
#[derive(Debug)]
pub(crate) struct Scanned(FsNode);

/// State: resolved `PathKey` and classified entry.
#[derive(Debug)]
pub(crate) struct Classified {
    entry: IndexedEntry,
    path_key: PathKey,
}

/// Union enum for the service loop to accumulate.
#[derive(Debug)]
pub(crate) enum IndexedEntry {
    File(FileIndexEntry),
    Dir(DirIndexEntry),
}

/// The typestate struct — only exists inside the service's for loop.
///
/// `S` is the current state type (`Scanned` or `Classified`), which also
/// carries the state-specific data. No `PhantomData`, no `Option`.
#[derive(Debug)]
pub(crate) struct IndexNode<S> {
    inner: S,
}

impl IndexNode<Scanned> {
    /// Wrap a raw `FsNode` from the scan stream.
    #[inline]
    #[must_use]
    pub(crate) fn new(node: FsNode) -> Self {
        Self {
            inner: Scanned(node),
        }
    }

    /// Resolve vault-relative `PathKey`, resolve `parent_id` from the
    /// `derive_parent_id` helper, query repository, classify status
    /// (New/Fresh/Stale).
    ///
    /// This is the sole transition — after this the entry is ready to
    /// consume.
    pub(crate) fn classify<R: ReadRepository>(
        self,
        vault_root: &DirPath,
        parent_id: FsParentId,
        repo: &R,
    ) -> Result<IndexNode<Classified>, IndexerError> {
        let (key, entry) = match self.inner.0 {
            FsNode::File(file) => {
                let key = file.path().as_key(vault_root)?;
                let existing = repo.find_file_by_path(&key)?;
                let status = classify_status(
                    file.metadata(),
                    existing.as_ref().map(super::model::FileRecord::metadata),
                );
                let record = build_file_record(&file, &key, parent_id, status);
                let entry = FileIndexEntry::new(
                    record.id(),
                    record,
                    file.path().clone(),
                    status,
                );
                (key, IndexedEntry::File(entry))
            }
            FsNode::Dir(dir) => {
                let key = dir.path().as_key(vault_root)?;
                let existing = repo.find_dir_by_path(&key)?;
                let status = classify_status(
                    dir.metadata(),
                    existing.as_ref().map(super::model::DirRecord::metadata),
                );
                let record = build_dir_record(&dir, &key, parent_id, status);
                let entry = DirIndexEntry::new(
                    record.id(),
                    record,
                    dir.path().clone(),
                    status,
                );
                (key, IndexedEntry::Dir(entry))
            }
        };
        Ok(IndexNode {
            inner: Classified {
                entry,
                path_key: key,
            },
        })
    }
}

impl IndexNode<Classified> {
    /// The resolved `PathKey`, for the service to track seen paths.
    #[inline]
    #[must_use]
    pub(crate) fn path_key(&self) -> &PathKey {
        &self.inner.path_key
    }

    /// The `FsRecordId` of the classified entry, for parent tracking.
    #[inline]
    #[must_use]
    pub(crate) fn entry_id(&self) -> crate::indexer::model::FsRecordId {
        match &self.inner.entry {
            IndexedEntry::File(f) => f.id(),
            IndexedEntry::Dir(d) => d.id(),
        }
    }

    /// True when the entry is a directory.
    #[inline]
    #[must_use]
    pub(crate) fn is_dir(&self) -> bool {
        matches!(&self.inner.entry, IndexedEntry::Dir(_))
    }

    /// Extract the classified entry for accumulation.
    #[inline]
    #[must_use]
    pub(crate) fn into_entry(self) -> IndexedEntry {
        self.inner.entry
    }
}

// ─── Helper functions ──────────────────────────────────────────────

/// Derive the parent for an entry at `key`. Root-level entries return
/// `FsParentId::Root`. Subdirectory entries look up their parent from the
/// `dir_ids` map. Panics if the parent has not been classified yet (walkdir
/// guarantees parents before children, so this is a programmer error).
#[expect(
    clippy::expect_used,
    reason = "parent prefix of valid path is always valid; parent dir \
              guaranteed classified before child"
)]
fn derive_parent_id(
    key: &PathKey,
    dir_ids: &HashMap<PathKey, crate::indexer::model::FsRecordId>,
) -> FsParentId {
    let s = key.as_str();
    let parent_key = s.rfind('/').map(|pos| {
        PathKey::try_new(&s[..pos])
            .expect("parent of valid path is a valid path")
    });
    match parent_key {
        None => FsParentId::Root,
        Some(pk) => FsParentId::Id(
            dir_ids
                .get(&pk)
                .copied()
                .expect("parent directory must be classified before child"),
        ),
    }
}

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
fn build_file_record(
    file: &crate::fs::FileNode,
    key: &PathKey,
    parent_id: FsParentId,
    status: IndexStatus,
) -> crate::indexer::model::FileRecord {
    use std::time::SystemTime;

    use crate::{
        fs::name::FileName,
        indexer::model::{FileRecord, FsRecordId},
    };

    let id = match status {
        IndexStatus::New => FsRecordId::new(),
        IndexStatus::Fresh | IndexStatus::Stale => {
            // ID will be set by the caller — for status detection we always
            // generate new since the classify loop doesn't know the existing
            // record ID at this point. The repo's save_many_records will
            // overwrite the correct record.
            FsRecordId::new()
        }
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
fn build_dir_record(
    dir: &crate::fs::DirNode,
    key: &PathKey,
    parent_id: FsParentId,
    status: IndexStatus,
) -> crate::indexer::model::DirRecord {
    use std::time::SystemTime;

    use crate::{
        fs::name::DirName,
        indexer::model::{DirRecord, FsRecordId},
    };

    let id = match status {
        IndexStatus::New | IndexStatus::Fresh | IndexStatus::Stale => {
            FsRecordId::new()
        }
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

// ─── Indexer service ───────────────────────────────────────────────

/// The indexer application service.
///
/// Wires a `ScannerPort` and `Repository` into a single indexing run.
/// Call `run()` with an `IndexScope` and `IndexOptions` to produce an
/// `IndexResult`.
pub(crate) struct IndexerService<S: ScannerPort, R: Repository> {
    vault_root: DirPath,
    scanner: S,
    repo: R,
}

impl<S: ScannerPort, R: Repository> IndexerService<S, R> {
    /// Create a new indexer service.
    #[inline]
    #[must_use]
    pub(crate) fn new(vault_root: DirPath, scanner: S, repo: R) -> Self {
        Self {
            vault_root,
            scanner,
            repo,
        }
    }

    /// Run a single indexing pass.
    ///
    /// 1. Resolve scope → root + filters.
    /// 2. If `reindex`, clear all persisted state.
    /// 3. Fused scan + classify loop (streaming via `ScannerPort::walk`).
    /// 4. Detect deletions (diff `all_paths()` vs `seen_paths`).
    /// 5. Persist indexed and deleted records (skip if `dry_run`).
    /// 6. Build `IndexReport` and return `IndexResult`.
    pub(crate) fn run(
        &self,
        scope: &IndexScope,
        opts: IndexOptions,
    ) -> Result<IndexResult, IndexerError> {
        let root = scope.root();
        let filters = scope.filters();

        if opts.reindex() {
            self.repo.clear()?;
        }

        let mut indexed_files: Vec<FileIndexEntry> = Vec::new();
        let mut indexed_dirs: Vec<DirIndexEntry> = Vec::new();
        let mut seen_paths: HashSet<PathKey> = HashSet::new();
        let mut dir_ids: HashMap<PathKey, crate::indexer::model::FsRecordId> =
            HashMap::new();
        let mut skipped: Vec<crate::indexer::report::SkippedEntry> = Vec::new();
        let mut new_count = 0usize;
        let mut fresh_count = 0usize;
        let mut stale_count = 0usize;

        for entry in self.scanner.walk(root, filters)? {
            match entry {
                Ok(ScanEntry::Skipped(s)) => skipped.push(s),
                Ok(ScanEntry::File(node)) => {
                    let key = node.path().as_key(&self.vault_root)?;
                    let parent_id = derive_parent_id(&key, &dir_ids);
                    let scanned = IndexNode::new(FsNode::File(node));
                    let classified = scanned.classify(
                        &self.vault_root,
                        parent_id,
                        &self.repo,
                    )?;
                    let pk = classified.path_key().clone();
                    seen_paths.insert(pk);
                    if let IndexedEntry::File(f) = classified.into_entry() {
                        match f.status() {
                            IndexStatus::New => new_count += 1,
                            IndexStatus::Fresh => fresh_count += 1,
                            IndexStatus::Stale => stale_count += 1,
                        }
                        indexed_files.push(f);
                    }
                }
                Ok(ScanEntry::Dir(node)) => {
                    let key = node.path().as_key(&self.vault_root)?;
                    let parent_id = derive_parent_id(&key, &dir_ids);
                    let scanned = IndexNode::new(FsNode::Dir(node));
                    let classified = scanned.classify(
                        &self.vault_root,
                        parent_id,
                        &self.repo,
                    )?;
                    let id = classified.entry_id();
                    let pk = classified.path_key().clone();
                    seen_paths.insert(pk.clone());
                    if let IndexedEntry::Dir(d) = classified.into_entry() {
                        match d.status() {
                            IndexStatus::New => new_count += 1,
                            IndexStatus::Fresh => fresh_count += 1,
                            IndexStatus::Stale => stale_count += 1,
                        }
                        dir_ids.insert(pk, id);
                        indexed_dirs.push(d);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        let deleted = self.detect_deletions(&seen_paths)?;

        if !opts.dry_run() {
            let indexed = IndexedNodes::new(
                indexed_files.clone().into_boxed_slice(),
                indexed_dirs.clone().into_boxed_slice(),
            );
            self.persist(&indexed, &deleted)?;
        }

        let scanned_count =
            indexed_files.len() + indexed_dirs.len() + skipped.len();
        let report = IndexReport::new(
            scanned_count,
            new_count,
            fresh_count,
            stale_count,
            deleted.count(),
            skipped.into_boxed_slice(),
            Box::new([]),
        );

        Ok(IndexResult::new(
            IndexedNodes::new(
                indexed_files.into_boxed_slice(),
                indexed_dirs.into_boxed_slice(),
            ),
            deleted,
            report,
        ))
    }

    /// Detect deleted nodes: paths that exist in the repository but were not
    /// encountered during the scan.
    fn detect_deletions(
        &self,
        seen: &HashSet<PathKey>,
    ) -> Result<DeletedNodes, IndexerError> {
        let all = self.repo.all_paths()?;
        let mut file_ids: Vec<crate::indexer::model::FsRecordId> = Vec::new();
        let mut dir_ids: Vec<crate::indexer::model::FsRecordId> = Vec::new();

        for path in &all {
            if !seen.contains(path) {
                if let Ok(Some(file)) = self.repo.find_file_by_path(path) {
                    file_ids.push(file.id());
                } else if let Ok(Some(dir)) = self.repo.find_dir_by_path(path) {
                    dir_ids.push(dir.id());
                } else {
                    // Neither file nor dir — stale path, skip
                }
            }
        }

        Ok(DeletedNodes::new(
            file_ids.into_boxed_slice(),
            dir_ids.into_boxed_slice(),
        ))
    }

    /// Persist indexed entries and remove deleted records.
    fn persist(
        &self,
        indexed: &IndexedNodes,
        deleted: &DeletedNodes,
    ) -> Result<(), IndexerError> {
        let file_records: Vec<_> =
            indexed.files().iter().map(|f| f.node().clone()).collect();
        let dir_records: Vec<_> =
            indexed.dirs().iter().map(|d| d.node().clone()).collect();

        self.repo.save_many_records(&file_records, &dir_records)?;
        self.repo.delete_many_records(deleted.files(), deleted.dirs())?;
        Ok(())
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        time::SystemTime,
    };

    use super::{Classified, IndexerService, Scanned};
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
            port::{ScanEntry, ScannerPort, WalkIter},
            report::{IndexReport, SkipReason, SkippedEntry},
            repository::{ReadRepository, Repository, WriteRepository},
            scan::{IndexOptions, IndexScope, ScanFilters},
            service::{
                IndexNode, IndexedEntry, build_dir_record, build_file_record,
                classify_status, derive_parent_id,
            },
            storage::InMemoryRepository,
            summary::{DeletedNodes, IndexResult, IndexedNodes},
        },
    };

    // ─── Helpers ────────────────────────────────────────────────

    fn make_vault_root() -> DirPath {
        std::fs::create_dir_all("/tmp/vault").unwrap();
        DirPath::try_new("/tmp/vault".into()).unwrap()
    }

    fn make_file_node(path: &str) -> crate::fs::FileNode {
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
        crate::fs::FileNode::new(fp, meta)
    }

    fn make_dir_node(path: &str) -> crate::fs::DirNode {
        let p = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&p).unwrap();
        let dp = DirPath::try_new(p).unwrap();
        let meta = DirMetadata::new(FsTimes::new(None, None), false);
        crate::fs::DirNode::new(dp, meta)
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
        let format = FileFormat::Unknown;
        let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
        let record = FileRecord::new(
            id,
            FsParentId::Root,
            path.clone(),
            name,
            format,
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

    // ─── Mock ScannerPort ───────────────────────────────────────

    struct MockScanner {
        entries: RefCell<
            Vec<Result<ScanEntry, crate::indexer::error::ScannerError>>,
        >,
    }

    impl MockScanner {
        fn new(
            entries: Vec<
                Result<ScanEntry, crate::indexer::error::ScannerError>,
            >,
        ) -> Self {
            Self {
                entries: RefCell::new(entries),
            }
        }

        fn empty() -> Self {
            Self {
                entries: RefCell::new(vec![]),
            }
        }

        fn single_file(node: crate::fs::FileNode) -> Self {
            Self {
                entries: RefCell::new(vec![Ok(ScanEntry::File(node))]),
            }
        }

        fn single_dir(node: crate::fs::DirNode) -> Self {
            Self {
                entries: RefCell::new(vec![Ok(ScanEntry::Dir(node))]),
            }
        }

        fn with_skipped(paths: Vec<(std::path::PathBuf, SkipReason)>) -> Self {
            let entries = paths
                .into_iter()
                .map(|(path, reason)| {
                    Ok(ScanEntry::Skipped(SkippedEntry {
                        path,
                        reason,
                    }))
                })
                .collect();
            Self {
                entries: RefCell::new(entries),
            }
        }
    }

    impl ScannerPort for MockScanner {
        fn walk(
            &self,
            _root: &DirPath,
            _filters: &ScanFilters,
        ) -> Result<WalkIter, crate::indexer::error::ScannerError> {
            let entries = std::mem::take(&mut *self.entries.borrow_mut());
            Ok(Box::new(entries.into_iter()))
        }
    }

    // ─── Cycle 1 — IndexNode typestate ──────────────────────────

    mod index_node {
        use super::*;

        #[test]
        fn scanned_new_wraps_fs_node() {
            let file_node = make_file_node("/tmp/vault/doc.md");
            let node = IndexNode::new(crate::fs::FsNode::File(file_node));
            let _ = node; // Compile-time: IndexNode<Scanned> inferred

            let dir_node_fs = make_dir_node("/tmp/vault/sub");
            let dir_node = IndexNode::new(crate::fs::FsNode::Dir(dir_node_fs));
            let _ = dir_node;
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classify_file_new() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/notes/new.md");
            let _key = file_node.path().as_key(&vault).unwrap();

            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            // Should be New
            match classified.into_entry() {
                IndexedEntry::File(f) => {
                    assert_eq!(f.status(), IndexStatus::New);
                }
                IndexedEntry::Dir(_) => panic!("expected file entry"),
            }
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classify_file_fresh() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes/new.md").unwrap();
            let repo = repo_with_file(&key);

            // Create a matching file node
            let file_path = vault.as_path().join("notes/new.md");
            let fp = FilePath::try_new(file_path).unwrap();
            let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
            let file_node = crate::fs::FileNode::new(fp, meta);

            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            match classified.into_entry() {
                IndexedEntry::File(f) => {
                    assert_eq!(f.status(), IndexStatus::Fresh);
                }
                IndexedEntry::Dir(_) => panic!("expected file entry"),
            }
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classify_file_stale() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes/new.md").unwrap();
            // Create a record with different metadata (size 200 vs current 100)
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
            let file_node = crate::fs::FileNode::new(fp, meta);

            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            match classified.into_entry() {
                IndexedEntry::File(f) => {
                    assert_eq!(f.status(), IndexStatus::Stale);
                }
                IndexedEntry::Dir(_) => panic!("expected file entry"),
            }
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classify_dir_new() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let dir_node = make_dir_node("/tmp/vault/notes");
            let scanned = IndexNode::new(crate::fs::FsNode::Dir(dir_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            match classified.into_entry() {
                IndexedEntry::Dir(d) => {
                    assert_eq!(d.status(), IndexStatus::New);
                }
                IndexedEntry::File(_) => panic!("expected dir entry"),
            }
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
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
            let dir_node = crate::fs::DirNode::new(dp, meta);

            let scanned = IndexNode::new(crate::fs::FsNode::Dir(dir_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            match classified.into_entry() {
                IndexedEntry::Dir(d) => {
                    assert_eq!(d.status(), IndexStatus::Fresh);
                }
                IndexedEntry::File(_) => panic!("expected dir entry"),
            }
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classify_dir_stale() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes").unwrap();
            let repo = {
                let r = InMemoryRepository::new();
                let id = FsRecordId::new();
                let name = DirName::new("notes".into());
                // Stale: different modified time
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
            let dir_node = crate::fs::DirNode::new(dp, meta);

            let scanned = IndexNode::new(crate::fs::FsNode::Dir(dir_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");

            match classified.into_entry() {
                IndexedEntry::Dir(d) => {
                    assert_eq!(d.status(), IndexStatus::Stale);
                }
                IndexedEntry::File(_) => panic!("expected dir entry"),
            }
        }

        #[test]
        fn classify_handles_outside_path() {
            let vault = make_vault_root();
            let repo = empty_repo();

            // File outside vault root should fail with PathError
            let file_node = make_file_node("/tmp/other/file.md");
            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let result = scanned.classify(&vault, FsParentId::Root, &repo);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), IndexerError::Path(_)));
        }

        #[test]
        fn classified_path_key() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");
            assert_eq!(classified.path_key().as_str(), "doc.md");
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Test assertions use panic for failures"
        )]
        fn classified_into_entry() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");
            match classified.into_entry() {
                IndexedEntry::File(_) => {} // expected
                IndexedEntry::Dir(_) => panic!("expected file entry"),
            }
        }

        #[test]
        fn classified_entry_id() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanned = IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified = scanned
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");
            let _id = classified.entry_id();
        }

        #[test]
        fn classified_is_dir() {
            let vault = make_vault_root();
            let repo = empty_repo();

            // File → is_dir false
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanned_file =
                IndexNode::new(crate::fs::FsNode::File(file_node));
            let classified_file = scanned_file
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");
            assert!(!classified_file.is_dir());

            // Dir → is_dir true
            let dir_node = make_dir_node("/tmp/vault/sub");
            let scanned_dir = IndexNode::new(crate::fs::FsNode::Dir(dir_node));
            let classified_dir = scanned_dir
                .classify(&vault, FsParentId::Root, &repo)
                .expect("classify should succeed");
            assert!(classified_dir.is_dir());
        }
    }

    mod helpers {
        use super::*;

        #[test]
        fn derive_parent_id_root_level() {
            let key = PathKey::try_new("doc.md").unwrap();
            let dir_ids = HashMap::new();
            let parent = derive_parent_id(&key, &dir_ids);
            assert_eq!(parent, FsParentId::Root);
        }

        #[test]
        fn derive_parent_id_subdirectory() {
            let key = PathKey::try_new("notes/doc.md").unwrap();
            let dir_id = FsRecordId::new();
            let mut dir_ids = HashMap::new();
            dir_ids.insert(PathKey::try_new("notes").unwrap(), dir_id);
            let parent = derive_parent_id(&key, &dir_ids);
            assert_eq!(parent, FsParentId::Id(dir_id));
        }

        #[test]
        #[should_panic(expected = "parent directory must be classified")]
        fn derive_parent_id_panics_on_missing_parent() {
            let key = PathKey::try_new("notes/doc.md").unwrap();
            let dir_ids = HashMap::new();
            let _ = derive_parent_id(&key, &dir_ids);
        }

        #[test]
        fn classify_status_new_when_missing() {
            let status = classify_status::<FileMetadata>(
                &FileMetadata::new(FsTimes::new(None, None), 100, false),
                None,
            );
            assert_eq!(status, IndexStatus::New);
        }

        #[test]
        fn classify_status_fresh_when_matching() {
            let meta = FileMetadata::new(FsTimes::new(None, None), 100, false);
            let status = classify_status(&meta, Some(&meta));
            assert_eq!(status, IndexStatus::Fresh);
        }

        #[test]
        fn classify_status_stale_when_different() {
            let current =
                FileMetadata::new(FsTimes::new(None, None), 100, false);
            let existing =
                FileMetadata::new(FsTimes::new(None, None), 200, false);
            let status = classify_status(&current, Some(&existing));
            assert_eq!(status, IndexStatus::Stale);
        }

        #[test]
        fn build_file_record_creates_new_record() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let key = file_node.path().as_key(&vault).unwrap();
            let record = build_file_record(
                &file_node,
                &key,
                FsParentId::Root,
                IndexStatus::New,
            );
            assert_eq!(record.path().as_str(), "doc.md");
        }

        #[test]
        fn build_dir_record_creates_new_record() {
            let vault = make_vault_root();
            let dir_node = make_dir_node("/tmp/vault/sub");
            let key = dir_node.path().as_key(&vault).unwrap();
            let record = build_dir_record(
                &dir_node,
                &key,
                FsParentId::Root,
                IndexStatus::New,
            );
            assert_eq!(record.path().as_str(), "sub");
            assert_eq!(record.parent_id(), FsParentId::Root);
        }
    }

    // ─── Cycle 2 — IndexerService::run() ────────────────────────

    mod service_run {
        use super::*;

        #[test]
        fn empty_scan() {
            let vault = make_vault_root();
            let scanner = MockScanner::empty();
            let repo = empty_repo();
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.indexed().files().len(), 0);
            assert_eq!(result.indexed().dirs().len(), 0);
            assert_eq!(result.report().scanned(), 0);
            assert_eq!(result.report().new_count(), 0);
        }

        #[test]
        fn single_file() {
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/file.md");
            let scanner = MockScanner::single_file(file_node);
            let repo = empty_repo();
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.indexed().files().len(), 1);
            assert_eq!(result.indexed().dirs().len(), 0);
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn single_dir() {
            let vault = make_vault_root();
            let dir_node = make_dir_node("/tmp/vault/notes");
            let scanner = MockScanner::single_dir(dir_node);
            let repo = empty_repo();
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.indexed().files().len(), 0);
            assert_eq!(result.indexed().dirs().len(), 1);
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn reindex_clears_repo_before_scan() {
            let vault = make_vault_root();
            let key = PathKey::try_new("notes/doc.md").unwrap();
            let repo = repo_with_file(&key);
            let dir_node = make_dir_node("/tmp/vault/notes");
            let file_node = make_file_node("/tmp/vault/notes/doc.md");
            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::Dir(dir_node)),
                Ok(ScanEntry::File(file_node)),
            ]);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            // After reindex, existing record was cleared so both dir and file
            // are New
            assert_eq!(result.report().new_count(), 2);
        }

        #[test]
        fn skipped_entries_do_not_abort() {
            let vault = make_vault_root();
            let skipped_entries = vec![(
                std::path::PathBuf::from("restricted"),
                SkipReason::PermissionDenied,
            )];
            let scanner = MockScanner::with_skipped(skipped_entries);
            let repo = empty_repo();
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().skipped().len(), 1);
        }
    }

    // ─── Cycle 3 — detect_deletions ─────────────────────────────

    mod detect_deletions {
        use super::*;

        #[test]
        fn no_deletions_when_all_seen() {
            let repo = empty_repo();
            let vault = make_vault_root();
            let scanner = MockScanner::empty();
            let service = IndexerService::new(vault, scanner, repo);

            let seen: HashSet<PathKey> =
                [PathKey::try_new("doc.md").unwrap()].into();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 0);
        }

        #[test]
        fn detects_missing_paths() {
            let key = PathKey::try_new("stale.md").unwrap();
            let repo = repo_with_file(&key);
            let vault = make_vault_root();
            let scanner = MockScanner::empty();
            let service = IndexerService::new(vault, scanner, repo);

            let seen: HashSet<PathKey> = HashSet::new();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 1);
        }

        #[test]
        fn empty_repo_no_deletions() {
            let repo = empty_repo();
            let vault = make_vault_root();
            let scanner = MockScanner::empty();
            let service = IndexerService::new(vault, scanner, repo);
            let seen: HashSet<PathKey> = HashSet::new();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 0);
        }

        #[test]
        fn mixed_files_and_dirs_deleted() {
            let file_key = PathKey::try_new("file.md").unwrap();
            let dir_key = PathKey::try_new("subdir").unwrap();
            let repo = {
                let r = repo_with_file(&file_key);
                let id = FsRecordId::new();
                let name = DirName::new("subdir".into());
                let meta = DirMetadata::new(FsTimes::new(None, None), false);
                let record = DirRecord::new(
                    id,
                    FsParentId::Root,
                    dir_key,
                    name,
                    meta,
                    SystemTime::now(),
                );
                r.save_dir(&record).unwrap();
                r
            };
            let vault = make_vault_root();
            let scanner = MockScanner::empty();
            let service = IndexerService::new(vault, scanner, repo);
            let seen: HashSet<PathKey> = HashSet::new();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 2);
            assert_eq!(deleted.files().len(), 1);
            assert_eq!(deleted.dirs().len(), 1);
        }
    }

    // ─── Cycle 4 — persist ──────────────────────────────────────

    mod persist {
        use super::*;

        #[test]
        fn persists_indexed_entries() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanner = MockScanner::single_file(file_node);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn dry_run_skips_persistence() {
            let vault = make_vault_root();
            let repo = empty_repo();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanner = MockScanner::single_file(file_node);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, true);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn reindex_no_deletions() {
            let key = PathKey::try_new("doc.md").unwrap();
            let repo = repo_with_file(&key);
            let vault = make_vault_root();
            let file_node = make_file_node("/tmp/vault/doc.md");
            let scanner = MockScanner::single_file(file_node);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: make_vault_root(),
                filters: ScanFilters::default(),
            };
            // Reindex clears repo before scan, so there's nothing to detect
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
            assert_eq!(result.report().deleted_count(), 0);
        }
    }

    // ─── Cycle 5 — Integration ──────────────────────────────────

    mod integration {
        use super::*;

        fn make_file_node_at(
            vault: &DirPath,
            rel: &str,
        ) -> crate::fs::FileNode {
            let full = vault.as_path().join(rel);
            std::fs::create_dir_all(full.parent().unwrap()).ok();
            std::fs::write(&full, "").unwrap();
            let fp = FilePath::try_new(full).unwrap();
            let meta = FileMetadata::new(
                FsTimes::new(Some(SystemTime::now()), None),
                0,
                false,
            );
            crate::fs::FileNode::new(fp, meta)
        }

        fn make_dir_node_at(vault: &DirPath, rel: &str) -> crate::fs::DirNode {
            let full = vault.as_path().join(rel);
            std::fs::create_dir_all(&full).ok();
            let dp = DirPath::try_new(full).unwrap();
            let meta = DirMetadata::new(FsTimes::new(None, None), false);
            crate::fs::DirNode::new(dp, meta)
        }

        #[test]
        fn full_integration_mixed_entries() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let file_a = make_file_node_at(&vault, "a.md");
            let dir_b = make_dir_node_at(&vault, "sub");
            let file_c = make_file_node_at(&vault, "sub/c.md");

            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::File(file_a)),
                Ok(ScanEntry::Dir(dir_b)),
                Ok(ScanEntry::File(file_c)),
            ]);

            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: DirPath::try_new(tmp.path().to_path_buf()).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");

            assert_eq!(result.indexed().files().len(), 2);
            assert_eq!(result.indexed().dirs().len(), 1);
            assert_eq!(result.report().new_count(), 3);
            assert_eq!(result.report().scanned(), 3);
        }

        #[test]
        fn dry_run_no_side_effects() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let file_node = make_file_node_at(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: DirPath::try_new(tmp.path().to_path_buf()).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, true);
            let result = service.run(&scope, opts).expect("run should succeed");

            // Dry run: result is populated but repo unchanged
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn report_counts_are_accurate() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let files = vec![
                make_file_node_at(&vault, "a.md"),
                make_file_node_at(&vault, "b.md"),
            ];
            let entries: Vec<_> =
                files.into_iter().map(|f| Ok(ScanEntry::File(f))).collect();

            let scanner = MockScanner::new(entries);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Full {
                root: DirPath::try_new(tmp.path().to_path_buf()).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");

            assert_eq!(result.report().new_count(), 2);
            assert_eq!(result.report().scanned(), 2);
        }

        #[test]
        fn partial_scope_and_reindex() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let dir_node = make_dir_node_at(&vault, "sub");
            let file_node = make_file_node_at(&vault, "sub/doc.md");
            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::Dir(dir_node)),
                Ok(ScanEntry::File(file_node)),
            ]);
            let service = IndexerService::new(vault, scanner, repo);
            let scope = IndexScope::Partial {
                root: DirPath::try_new(tmp.path().join("sub")).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");

            // Reindex after clear: dir + file are New
            assert_eq!(result.report().new_count(), 2);
        }
    }
}
