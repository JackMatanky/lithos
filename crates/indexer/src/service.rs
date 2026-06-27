//! Indexer application service — orchestrates the index scan pipeline.
//!
//! Uses `ScannedEntry::new` (see `builder.rs`) to dispatch each `ScanEntry`
//! to the appropriate `EntryBuilder<IsFile, Scanned>` or
//! `EntryBuilder<IsDir, Scanned>`, deriving `parent_id` from the accumulated
//! `dir_ids` map via `FilePath::parent()` / `DirPath::parent()`.

use std::collections::{HashMap, HashSet};

use traces_fs::{DirPath, path::PathKey};

use crate::{
    builder::{
        Completion, CompletionKind, DirComparisonBranch, EntryBranch,
        EntryBuilder, FileComparisonBranch, Init,
    },
    entry::{DirIndexEntry, FileIndexEntry, IndexStatus},
    error::IndexerError,
    model::FsRecordId,
    port::{ScanEntry, ScannerPort},
    report::{IndexNodeFailure, IndexReport, SkippedEntry},
    repository::Repository,
    scan::{IndexOptions, IndexScope},
    summary::{DeletedNodes, IndexResult, IndexedNodes},
};

// ─── Indexer service ───────────────────────────────────────────────

/// The indexer application service.
///
/// Wires a `ScannerPort` and `Repository` into a single indexing run.
/// Call `run()` with an `IndexScope` and `IndexOptions` to produce an
/// `IndexResult`.
///
/// Uses trait-object dispatch: the two vtable calls per scan iteration are
/// noise next to disk-bound work, and a single monomorphization keeps the
/// binary smaller and compiles faster.
pub struct IndexerService {
    vault_root: DirPath,
    scanner: Box<dyn ScannerPort>,
    repo: Box<dyn Repository>,
}

impl IndexerService {
    /// Create a new indexer service.
    #[inline]
    #[must_use]
    pub fn new(
        vault_root: DirPath,
        scanner: Box<dyn ScannerPort>,
        repo: Box<dyn Repository>,
    ) -> Self {
        Self {
            vault_root,
            scanner,
            repo,
        }
    }

    /// Run a single indexing pass.
    ///
    /// 1. Resolve scope → root + filters.
    /// 2. If `rebuild`, clear all persisted state.
    /// 3. Fused scan + classify loop (streaming via `ScannerPort::walk`).
    /// 4. Detect deletions (diff `all_paths()` vs `seen_paths`).
    /// 5. Persist indexed and deleted records (skip if `dry_run`).
    /// 6. Build `IndexReport` and return `IndexResult`.
    /// # Errors
    /// Returns `IndexerError` if traversal or database operations fail.
    #[inline]
    pub fn run(
        &mut self,
        scope: &IndexScope,
        opts: IndexOptions,
    ) -> Result<IndexResult, IndexerError> {
        let root = scope.root();
        let filters = scope.filters();

        if opts.rebuild() {
            self.repo.clear()?;
        }

        let mut ctx = IndexCollector::default();
        for result in self.scanner.walk(root, filters)? {
            // Stream-level errors mean traversal itself broke → abort the run.
            let scan_entry = result?;
            let entry_path = scan_entry_path(&scan_entry);

            match self.classify_entry(scan_entry, &ctx.dir_ids, opts.dry_run())
            {
                Ok(completion) => ctx.record(completion),
                // Per-entry path errors are recoverable: record the failure
                // and keep scanning ("scan as much as possible").
                Err(IndexerError::Path(e)) => {
                    ctx.failures.push(IndexNodeFailure::new(
                        entry_path,
                        e.to_string().into(),
                    ));
                }
                // Repository/other errors are fatal — they will recur on every
                // entry, so abort the whole run.
                Err(other) => return Err(other),
            }
        }
        let deleted = self.detect_deletions(&ctx.seen_paths)?;

        if !opts.dry_run() {
            self.repo.delete_many_records(deleted.files(), deleted.dirs())?;
        }

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "scanned totals are bounded by the number of entries"
        )]
        let scanned_count = ctx.indexed_files.len()
            + ctx.indexed_dirs.len()
            + ctx.skipped.len();
        let report = IndexReport::new(
            scanned_count,
            ctx.new_count,
            ctx.fresh_count,
            ctx.stale_count,
            deleted.count(),
            ctx.skipped.into_boxed_slice(),
            ctx.failures.into_boxed_slice(),
        );

        Ok(IndexResult::new(
            IndexedNodes::new(
                ctx.indexed_files.into_boxed_slice(),
                ctx.indexed_dirs.into_boxed_slice(),
            ),
            deleted,
            report,
        ))
    }

    /// Drive one scan entry through the builder pipeline to a `Completion`.
    ///
    /// Returns `IndexerError::Path` for per-entry, recoverable failures (e.g.
    /// an entry outside the vault root) and `IndexerError::Repository` for
    /// fatal persistence failures; the caller decides soft-fail vs abort.
    fn classify_entry(
        &self,
        scan_entry: crate::port::ScanEntry,
        dir_ids: &HashMap<PathKey, FsRecordId>,
        dry_run: bool,
    ) -> Result<Completion, IndexerError> {
        let branch = EntryBuilder::<Init>::from_scan_entry(scan_entry)
            .into_branch(&self.vault_root)?;

        let repo = self.repo.as_ref();
        let completion = match branch {
            EntryBranch::File(b) => match b.into_comparison_branch(repo)? {
                FileComparisonBranch::Match(b) => {
                    b.into_completion().into_state()
                }
                FileComparisonBranch::Mismatch(b) => b
                    .into_indexed(repo, dir_ids, dry_run)?
                    .into_completion()
                    .into_state(),
            },
            EntryBranch::Dir(b) => match b.into_comparison_branch(repo)? {
                DirComparisonBranch::Match(b) => {
                    b.into_completion().into_state()
                }
                DirComparisonBranch::Mismatch(b) => b
                    .into_indexed(repo, dir_ids, dry_run)?
                    .into_completion()
                    .into_state(),
            },
            EntryBranch::Completion(b) => b.into_state(),
        };

        Ok(completion)
    }

    /// Detect deleted nodes: paths that exist in the repository but were not
    /// encountered during the scan.
    fn detect_deletions(
        &self,
        seen: &HashSet<PathKey>,
    ) -> Result<DeletedNodes, IndexerError> {
        let all = self.repo.all_paths()?;
        let mut file_ids: Vec<crate::model::FsRecordId> = Vec::new();
        let mut dir_ids: Vec<crate::model::FsRecordId> = Vec::new();

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
}

/// Best-effort path of a scan entry, for failure reporting.
///
/// A skipped entry already carries its path; file/dir entries expose their
/// runtime path. Used only to key an `IndexNodeFailure`.
fn scan_entry_path(entry: &ScanEntry) -> std::path::PathBuf {
    match entry {
        ScanEntry::File(node) => node.path().as_path().to_path_buf(),
        ScanEntry::Dir(node) => node.path().as_path().to_path_buf(),
        ScanEntry::Skipped(s) => s.path.clone(),
    }
}

// ─── Scan accumulator ──────────────────────────────────────────────

/// Accumulates scan results during an indexer run.
///
/// Collects indexed entries, path tracking data, and counters over the
/// streaming scan loop. Consumed at the end to build `IndexResult`.
#[derive(Debug, Default)]
pub(super) struct IndexCollector {
    pub(super) indexed_files: Vec<FileIndexEntry>,
    pub(super) indexed_dirs: Vec<DirIndexEntry>,
    pub(super) seen_paths: HashSet<PathKey>,
    pub(super) dir_ids: HashMap<PathKey, FsRecordId>,
    pub(super) skipped: Vec<SkippedEntry>,
    pub(super) failures: Vec<IndexNodeFailure>,
    pub(super) new_count: usize,
    pub(super) fresh_count: usize,
    pub(super) stale_count: usize,
}

impl IndexCollector {
    fn record(&mut self, completion: Completion) {
        match completion.kind {
            CompletionKind::File {
                entry,
                path_key,
            } => {
                self.seen_paths.insert(path_key);
                self.bump_status_counter(entry.status());
                self.indexed_files.push(entry);
            }
            CompletionKind::Dir {
                entry,
                path_key,
                id,
            } => {
                self.seen_paths.insert(path_key.clone());
                self.dir_ids.insert(path_key, id);
                self.bump_status_counter(entry.status());
                self.indexed_dirs.push(entry);
            }
            CompletionKind::Skipped(s) => self.skipped.push(s),
        }
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "status counters are bounded by the number of scanned entries"
    )]
    fn bump_status_counter(&mut self, status: IndexStatus) {
        match status {
            IndexStatus::New => self.new_count += 1,
            IndexStatus::Fresh => self.fresh_count += 1,
            IndexStatus::Stale => self.stale_count += 1,
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashSet, rc::Rc, time::SystemTime};

    use traces_fs::{
        FileFormat,
        metadata::{DirMetadata, FileMetadata, FsTimes},
        name::{DirName, FileName},
        path::{DirPath, FilePath, PathKey},
    };

    use super::IndexerService;
    use crate::*;

    // ─── Helpers ────────────────────────────────────────────────

    fn make_vault_root() -> (tempfile::TempDir, DirPath) {
        let tmp = tempfile::TempDir::new().unwrap();
        let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
        (tmp, vault)
    }

    fn make_file_node(vault: &DirPath, rel: &str) -> traces_fs::FileNode {
        let p = vault.as_path().join(rel);
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
        traces_fs::FileNode::new(fp, meta)
    }

    fn make_dir_node(vault: &DirPath, rel: &str) -> traces_fs::DirNode {
        let p = vault.as_path().join(rel);
        std::fs::create_dir_all(&p).unwrap();
        let dp = DirPath::try_new(p).unwrap();
        let meta = DirMetadata::new(FsTimes::new(None, None), false);
        traces_fs::DirNode::new(dp, meta)
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

    // ─── Mock ScannerPort ───────────────────────────────────────

    struct MockScanner {
        entries: RefCell<Vec<Result<ScanEntry, crate::error::ScannerError>>>,
    }

    impl MockScanner {
        fn new(
            entries: Vec<Result<ScanEntry, crate::error::ScannerError>>,
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

        fn single_file(node: traces_fs::FileNode) -> Self {
            Self {
                entries: RefCell::new(vec![Ok(ScanEntry::File(node))]),
            }
        }

        fn single_dir(node: traces_fs::DirNode) -> Self {
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
        ) -> Result<WalkIter, crate::error::ScannerError> {
            let entries = std::mem::take(&mut *self.entries.borrow_mut());
            Ok(Box::new(entries.into_iter()))
        }
    }

    // ─── Repository double that fails on write ──────────────────

    /// Reads succeed (returning empty), but every write fails with a
    /// repository error — used to prove fatal errors abort the run.
    struct FailingWriteRepository;

    impl ReadRepository for FailingWriteRepository {
        fn find_file(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_dir(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_file_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_dir_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn list_files_by_parent(
            &self,
            _parent_id: FsParentId,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn list_dirs_by_parent(
            &self,
            _parent_id: FsParentId,
        ) -> Result<Box<[DirRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn list_files_by_format(
            &self,
            _format: FileFormat,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn list_files_by_basename(
            &self,
            _basename: &str,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }
    }

    impl WriteRepository for FailingWriteRepository {
        fn save_file(
            &self,
            _record: &FileRecord,
        ) -> Result<(), IndexerRepositoryError> {
            Err(IndexerRepositoryError::Storage(
                traces_db::DbError::Serialization("write failed".into()),
            ))
        }

        fn save_dir(
            &self,
            _record: &DirRecord,
        ) -> Result<(), IndexerRepositoryError> {
            Err(IndexerRepositoryError::Storage(
                traces_db::DbError::Serialization("write failed".into()),
            ))
        }

        fn delete_file(
            &self,
            _id: FsRecordId,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn delete_dir(
            &self,
            _id: FsRecordId,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn save_many_records(
            &self,
            _files: &[FileRecord],
            _dirs: &[DirRecord],
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn delete_many_records(
            &self,
            _file_ids: &[FsRecordId],
            _dir_ids: &[FsRecordId],
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn clear(&self) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }
    }

    // ─── Cycle 2 — IndexerService::run() ────────────────────────

    mod service_run {
        use super::*;

        #[test]
        fn empty_scan() {
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
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
            let (_tmp, vault) = make_vault_root();
            let file_node = make_file_node(&vault, "file.md");
            let scanner = MockScanner::single_file(file_node);
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
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
            let (_tmp, vault) = make_vault_root();
            let dir_node = make_dir_node(&vault, "notes");
            let scanner = MockScanner::single_dir(dir_node);
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.indexed().files().len(), 0);
            assert_eq!(result.indexed().dirs().len(), 1);
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn rebuild_clears_repo_before_scan() {
            let (_tmp, vault) = make_vault_root();
            let key = PathKey::try_new("notes/doc.md").unwrap();
            let repo = repo_with_file(&key);
            let dir_node = make_dir_node(&vault, "notes");
            let file_node = make_file_node(&vault, "notes/doc.md");
            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::Dir(dir_node)),
                Ok(ScanEntry::File(file_node)),
            ]);
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            // After rebuild, existing record was cleared so both dir and file
            // are New
            assert_eq!(result.report().new_count(), 2);
        }

        #[test]
        fn skipped_entries_do_not_abort() {
            let (_tmp, vault) = make_vault_root();
            let skipped_entries = vec![(
                std::path::PathBuf::from("restricted"),
                SkipReason::PermissionDenied,
            )];
            let scanner = MockScanner::with_skipped(skipped_entries);
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().skipped().len(), 1);
        }

        #[test]
        fn scanner_error_aborts_run() {
            let (_tmp, vault) = make_vault_root();
            let err = crate::error::ScannerError::Traversal {
                path: "/bad".into(),
                source: std::io::Error::other("test"),
            };
            let scanner = MockScanner::new(vec![Err(err)]);
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let result = service.run(&scope, IndexOptions::new(false, false));
            assert!(matches!(
                result,
                Err(IndexerError::Scanner(
                    crate::error::ScannerError::Traversal { .. }
                ))
            ));
        }

        #[test]
        fn per_entry_path_error_records_failure_and_continues() {
            let (_tmp, vault) = make_vault_root();
            // A file outside the vault root fails `as_key` → IndexerError::Path
            // (per-entry, recoverable). A valid in-vault file still indexes.
            let outside = tempfile::TempDir::new().unwrap();
            let bad_node = {
                let p = outside.path().join("orphan.md");
                std::fs::File::create(&p).unwrap();
                let fp = FilePath::try_new(p).unwrap();
                let meta = FileMetadata::new(
                    FsTimes::new(Some(SystemTime::now()), None),
                    0,
                    false,
                );
                traces_fs::FileNode::new(fp, meta)
            };
            let good_node = make_file_node(&vault, "ok.md");
            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::File(bad_node)),
                Ok(ScanEntry::File(good_node)),
            ]);
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let result = service
                .run(&scope, IndexOptions::new(false, false))
                .expect("path error must not abort the run");
            assert_eq!(result.report().failures().len(), 1);
            assert_eq!(result.report().new_count(), 1);
        }

        #[test]
        fn repository_error_still_aborts() {
            let (_tmp, vault) = make_vault_root();
            let good_node = make_file_node(&vault, "ok.md");
            let scanner = MockScanner::single_file(good_node);
            let repo = FailingWriteRepository;
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let result = service.run(&scope, IndexOptions::new(false, false));
            assert!(matches!(result, Err(IndexerError::Repository(_))));
        }

        // ─── Capturing mock for scope delegation tests ──────

        struct CapturingMockScanner {
            root: Rc<RefCell<Option<DirPath>>>,
            filters: Rc<RefCell<Option<ScanFilters>>>,
        }

        impl CapturingMockScanner {
            #[allow(
                clippy::type_complexity,
                reason = "test helper returns shared-rc pairs"
            )]
            fn new() -> (
                Self,
                Rc<RefCell<Option<DirPath>>>,
                Rc<RefCell<Option<ScanFilters>>>,
            ) {
                let root = Rc::new(RefCell::new(None));
                let filters = Rc::new(RefCell::new(None));
                (
                    Self {
                        root: Rc::clone(&root),
                        filters: Rc::clone(&filters),
                    },
                    root,
                    filters,
                )
            }
        }

        impl ScannerPort for CapturingMockScanner {
            fn walk(
                &self,
                root: &DirPath,
                filters: &ScanFilters,
            ) -> Result<WalkIter, crate::error::ScannerError> {
                *self.root.borrow_mut() = Some(root.clone());
                *self.filters.borrow_mut() = Some(filters.clone());
                Ok(Box::new(std::iter::empty::<
                    Result<ScanEntry, crate::error::ScannerError>,
                >()))
            }
        }

        #[test]
        fn full_scope_delegates_root_and_filters() {
            let (_tmp, vault) = make_vault_root();
            let (scanner, captured_root, captured_filters) =
                CapturingMockScanner::new();
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault.clone(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let _ = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(captured_root.borrow().as_ref(), Some(&vault));
            assert_eq!(
                captured_filters.borrow().as_ref(),
                Some(&ScanFilters::default())
            );
        }

        #[test]
        fn partial_scope_delegates_root_and_filters() {
            let (_tmp, vault) = make_vault_root();
            let (scanner, captured_root, captured_filters) =
                CapturingMockScanner::new();
            let repo = empty_repo();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Partial {
                root: vault.clone(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let _ = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(captured_root.borrow().as_ref(), Some(&vault));
            assert_eq!(
                captured_filters.borrow().as_ref(),
                Some(&ScanFilters::default())
            );
        }
    }

    // ─── Cycle 3 — detect_deletions ─────────────────────────────

    mod detect_deletions {
        use super::*;

        #[test]
        fn no_deletions_when_all_seen() {
            let repo = empty_repo();
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));

            let seen: HashSet<PathKey> =
                [PathKey::try_new("doc.md").unwrap()].into();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 0);
        }

        #[test]
        fn detects_missing_paths() {
            let key = PathKey::try_new("stale.md").unwrap();
            let repo = repo_with_file(&key);
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));

            let seen: HashSet<PathKey> = HashSet::new();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 1);
        }

        #[test]
        fn empty_repo_no_deletions() {
            let repo = empty_repo();
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
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
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
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
            let (_tmp, vault) = make_vault_root();
            let repo = empty_repo();
            let repo_check = repo.clone();
            let file_node = make_file_node(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
            let key = PathKey::try_new("doc.md").unwrap();
            assert!(
                repo_check.find_file_by_path(&key).unwrap().is_some(),
                "New file should be persisted"
            );
        }

        #[test]
        fn dry_run_skips_persistence() {
            let (_tmp, vault) = make_vault_root();
            let repo = empty_repo();
            let repo_check = repo.clone();
            let file_node = make_file_node(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, true);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
            let key = PathKey::try_new("doc.md").unwrap();
            assert!(
                repo_check.find_file_by_path(&key).unwrap().is_none(),
                "dry_run must not persist"
            );
        }

        #[test]
        fn rebuild_no_deletions() {
            let key = PathKey::try_new("doc.md").unwrap();
            let repo = repo_with_file(&key);
            let (_tmp, vault) = make_vault_root();
            let file_node = make_file_node(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            // Reindex clears repo before scan, so there's nothing to detect
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 1);
            assert_eq!(result.report().deleted_count(), 0);
        }

        #[test]
        fn deletes_deleted_entries() {
            let (_tmp, vault) = make_vault_root();
            let file_key = PathKey::try_new("doc.md").unwrap();
            let dir_key = PathKey::try_new("sub").unwrap();
            let repo = repo_with_file(&file_key);
            repo.save_dir(&DirRecord::new(
                FsRecordId::new(),
                FsParentId::Root,
                dir_key.clone(),
                DirName::new("sub".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            ))
            .unwrap();
            let repo_check = repo.clone();
            let scanner = MockScanner::empty();
            let mut service = IndexerService::new(
                vault.clone(),
                Box::new(scanner),
                Box::new(repo),
            );
            let scope = IndexScope::Full {
                root: vault,
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().deleted_count(), 2);
            // Verify actual removal from repo
            assert!(repo_check.find_file_by_path(&file_key).unwrap().is_none());
            assert!(repo_check.find_dir_by_path(&dir_key).unwrap().is_none());
        }
    }

    // ─── Cycle 5 — Integration ──────────────────────────────────

    mod integration {
        use super::*;

        fn make_file_node_at(
            vault: &DirPath,
            rel: &str,
        ) -> traces_fs::FileNode {
            let full = vault.as_path().join(rel);
            std::fs::create_dir_all(full.parent().unwrap()).ok();
            std::fs::write(&full, "").unwrap();
            let fp = FilePath::try_new(full).unwrap();
            let meta = FileMetadata::new(
                FsTimes::new(Some(SystemTime::now()), None),
                0,
                false,
            );
            traces_fs::FileNode::new(fp, meta)
        }

        fn make_dir_node_at(vault: &DirPath, rel: &str) -> traces_fs::DirNode {
            let full = vault.as_path().join(rel);
            std::fs::create_dir_all(&full).ok();
            let dp = DirPath::try_new(full).unwrap();
            let meta = DirMetadata::new(FsTimes::new(None, None), false);
            traces_fs::DirNode::new(dp, meta)
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

            let mut service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
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

            // sub/c.md (files[1]) must be parented to the sub dir record.
            let sub_id = result.indexed().dirs().first().unwrap().id();
            let child = result.indexed().files().get(1).unwrap();
            assert_eq!(child.node().parent_id(), FsParentId::Id(sub_id));
        }

        #[test]
        fn dry_run_no_side_effects() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let file_node = make_file_node_at(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let mut service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
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
            let mut service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
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
        fn partial_scope_and_rebuild() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let repo = empty_repo();

            let dir_node = make_dir_node_at(&vault, "sub");
            let file_node = make_file_node_at(&vault, "sub/doc.md");
            let scanner = MockScanner::new(vec![
                Ok(ScanEntry::Dir(dir_node)),
                Ok(ScanEntry::File(file_node)),
            ]);
            let mut service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
            let scope = IndexScope::Partial {
                root: DirPath::try_new(tmp.path().join("sub")).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(true, false);
            let result = service.run(&scope, opts).expect("run should succeed");

            // Reindex after clear: dir + file are New
            assert_eq!(result.report().new_count(), 2);
        }

        #[test]
        fn scan_classify_persist_roundtrip() {
            let tmp = tempfile::TempDir::new().unwrap();
            let vault = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let file_node = make_file_node_at(&vault, "doc.md");
            let meta = file_node.metadata().clone();
            let file_key = PathKey::try_new("doc.md").unwrap();
            let repo = empty_repo();
            repo.save_file(&FileRecord::new(
                FsRecordId::new(),
                FsParentId::Root,
                file_key,
                FileName::new("doc.md".into()),
                FileFormat::Unknown,
                meta,
                SystemTime::now(),
            ))
            .unwrap();
            let scanner = MockScanner::single_file(file_node);
            let mut service =
                IndexerService::new(vault, Box::new(scanner), Box::new(repo));
            let scope = IndexScope::Full {
                root: DirPath::try_new(tmp.path().to_path_buf()).unwrap(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::new(false, false);
            let result = service.run(&scope, opts).expect("run should succeed");
            assert_eq!(result.report().new_count(), 0);
            assert_eq!(result.report().fresh_count(), 1);
            assert_eq!(result.indexed().files().len(), 1);
        }
    }
}
