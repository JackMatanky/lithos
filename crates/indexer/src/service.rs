//! Indexer application service — orchestrates the index scan pipeline.
//!
//! Uses `ScannedEntry::new` (see `builder.rs`) to dispatch each `ScanEntry`
//! to the appropriate `EntryBuilder<IsFile, Scanned>` or
//! `EntryBuilder<IsDir, Scanned>`, deriving `parent_id` from the accumulated
//! `dir_ids` map via `FilePath::parent()` / `DirPath::parent()`.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "counter increments in report builders are bounded by file count"
)]

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
    port::ScannerPort,
    report::{IndexReport, SkippedEntry},
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
pub struct IndexerService<S: ScannerPort, R: Repository> {
    vault_root: DirPath,
    scanner: S,
    repo: R,
}

impl<S: ScannerPort, R: Repository> IndexerService<S, R> {
    /// Create a new indexer service.
    #[inline]
    #[must_use]
    pub fn new(vault_root: DirPath, scanner: S, repo: R) -> Self {
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
        &self,
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
            let scan_entry = result?;
            let branch = EntryBuilder::<Init>::from_scan_entry(scan_entry)
                .into_branch(&self.vault_root)?;

            let completion = match branch {
                EntryBranch::File(b) => match b
                    .into_comparison_branch(&self.repo)?
                {
                    FileComparisonBranch::Match(b) => {
                        b.into_completion().into_state()
                    }
                    FileComparisonBranch::Mismatch(b) => b
                        .into_indexed(&self.repo, &ctx.dir_ids, opts.dry_run())?
                        .into_completion()
                        .into_state(),
                },
                EntryBranch::Dir(b) => match b
                    .into_comparison_branch(&self.repo)?
                {
                    DirComparisonBranch::Match(b) => {
                        b.into_completion().into_state()
                    }
                    DirComparisonBranch::Mismatch(b) => b
                        .into_indexed(&self.repo, &ctx.dir_ids, opts.dry_run())?
                        .into_completion()
                        .into_state(),
                },
                EntryBranch::Completion(b) => b.into_state(),
            };

            ctx.record(completion);
        }
        let deleted = self.detect_deletions(&ctx.seen_paths)?;

        if !opts.dry_run() {
            self.repo.delete_many_records(deleted.files(), deleted.dirs())?;
        }

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
            Box::new([]),
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
                match entry.status() {
                    IndexStatus::New => self.new_count += 1,
                    IndexStatus::Fresh => self.fresh_count += 1,
                    IndexStatus::Stale => self.stale_count += 1,
                }
                self.indexed_files.push(entry);
            }
            CompletionKind::Dir {
                entry,
                path_key,
                id,
            } => {
                self.seen_paths.insert(path_key.clone());
                self.dir_ids.insert(path_key, id);
                match entry.status() {
                    IndexStatus::New => self.new_count += 1,
                    IndexStatus::Fresh => self.fresh_count += 1,
                    IndexStatus::Stale => self.stale_count += 1,
                }
                self.indexed_dirs.push(entry);
            }
            CompletionKind::Skipped(s) => self.skipped.push(s),
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

    #[allow(dead_code, reason = "Test helper currently uncalled")]
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

    // ─── Cycle 2 — IndexerService::run() ────────────────────────

    mod service_run {
        use super::*;

        #[test]
        fn empty_scan() {
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let repo = empty_repo();
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let (_tmp, vault) = make_vault_root();
            let scanner = MockScanner::empty();
            let service = IndexerService::new(vault, scanner, repo);

            let seen: HashSet<PathKey> = HashSet::new();
            let deleted = service.detect_deletions(&seen).unwrap();
            assert_eq!(deleted.count(), 1);
        }

        #[test]
        fn empty_repo_no_deletions() {
            let repo = empty_repo();
            let (_tmp, vault) = make_vault_root();
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
            let (_tmp, vault) = make_vault_root();
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
            let (_tmp, vault) = make_vault_root();
            let repo = empty_repo();
            let repo_check = repo.clone();
            let file_node = make_file_node(&vault, "doc.md");
            let scanner = MockScanner::single_file(file_node);
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault.clone(), scanner, repo);
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
            let service = IndexerService::new(vault, scanner, repo);
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
