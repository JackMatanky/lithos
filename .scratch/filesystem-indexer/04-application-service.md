---
title: 04-indexer-application-service
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-09
date_completed:
---

# Issue 04: Indexer application service

## What to build

Implement the Indexer application service in `lithos-core::indexer::service`,
along with a per-entry typestate classifier (`IndexNode<State>`). The service
accepts an `IndexScope`, `IndexOptions`, a `ScannerPort`, and a `Repository`,
and returns an `IndexResult`.

### Architecture

The service is **procedural** (wires the pipeline) with a **scoped typestate**
(`IndexNode<State>`) used only inside the scan loop. The typestate ensures
per-entry classification always resolves `PathKey` before the entry can be
consumed — a compile-time guarantee on a single entry, not on the whole
pipeline.

**Parent tracking**: a `HashMap<PathKey, FsRecordId>` (`dir_ids`) is maintained
in the service's `run()` loop. As directories are classified, their
`FsRecordId` is inserted. Files and subdirs derive their parent's `PathKey` by
stripping the last path component and look it up in the map. When the
derivation produces empty (root-level entry), `FsParentId::Root` is returned.
An `FsParentId` enum (defined in `model.rs` alongside `FsRecordId`) clarifies
the parent relationship — `Root` for entries directly under the vault root,
`Id(FsRecordId)` for entries inside a known subdirectory.

**IndexReport in return**: `IndexResult` gains a `report: IndexReport` field.
The report is built from counters accumulated during the loop (new/fresh/stale
per entry, deleted count, skipped entries).

Data flow:

```
Service::run(scope, opts, scanner, repo)
  │
  ├─ resolve_scope(scope) → (root: DirPath, filters: ScanFilters)
  ├─ if opts.reindex() → repo.clear()
  │
  ├─ for entry in scanner.walk(&root, &filters)?
  │   │
  │   ├─ ScanEntry::Skipped(s) → skipped.push(s); continue
  │   │
  │   └─ ScanEntry::File(node) | ScanEntry::Dir(node)
  │       └─ parent_id = derive_parent_id(&key, &dir_ids)
  │       └─ IndexNode::new(node)                          // → IndexNode<Scanned>
  │          └─ .classify(&vault_root, parent_id, &repo)?
  │             ├─ seen_paths.insert(.path_key().clone())
  │             ├─ if .is_dir() → dir_ids.insert(key, .entry_id())
  │             └─ .into_entry() → accumulate in indexed_files or indexed_dirs
  │
  ├─ detect_deletions(seen_paths, &repo)?            // diff all_paths() vs seen
  ├─ persist(indexed, deleted, opts)?                // skip if dry_run
  └─ IndexResult::new(indexed, deleted, report)
```

### `FsParentId` type (in `model.rs` alongside `FsRecordId`)

```rust
/// Identifies the parent of a filesystem node during indexing.
///
/// `Root` represents the vault root itself — used when an entry is directly
/// in the vault root (no parent directory exists). `Id(id)` represents a
/// specific indexed directory that contains this entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FsParentId {
    /// Entry is directly under the vault root.
    Root,
    /// Entry is inside a known indexed directory.
    Id(FsRecordId),
}
```

### Per-entry typestate: `IndexNode<State>`

Defined inline in `service.rs`. Two-state linear typestate. State types ARE
the data carriers — no separate data wrapper:

```rust
/// State: raw FsNode from the scanner stream.
struct Scanned(FsNode);

/// State: resolved PathKey and classified entry.
struct Classified {
    entry: IndexedEntry,
    path_key: PathKey,
}

/// Union enum for the service loop to accumulate.
enum IndexedEntry {
    File(FileIndexEntry),
    Dir(DirIndexEntry),
}

/// The typestate struct — only exists inside the service's for loop.
/// `S` is the current state type (`Scanned` or `Classified`), which also
/// carries the state-specific data. No PhantomData, no Option.
pub(crate) struct IndexNode<S> {
    inner: S,
}

impl IndexNode<Scanned> {
    /// Wrap a raw FsNode from the scan stream.
    pub(crate) fn new(node: FsNode) -> Self {
        Self { inner: Scanned(node) }
    }

    /// Resolve vault-relative PathKey, resolve parent_id from the
    /// `derive_parent_id` helper, query repository, classify status
    /// (New/Fresh/Stale).
    /// This is the sole transition — after this the entry is ready to
    /// consume.
    pub(crate) fn classify(
        self,
        vault_root: &DirPath,
        parent_id: FsParentId,
        repo: &impl ReadRepository,
    ) -> Result<IndexNode<Classified>, IndexerError> {
        let (key, entry) = match self.inner.0 {
            FsNode::File(file) => {
                let key = file.path().as_key(vault_root)?;
                let existing = repo.find_file_by_path(&key)?;
                let status = classify_status(file.metadata(), existing.as_ref());
                let record = build_file_record(&file, &key, parent_id, status);
                let entry = FileIndexEntry::new(record.id(), record, file.path().clone(), status);
                (key, IndexedEntry::File(entry))
            }
            FsNode::Dir(dir) => {
                let key = dir.path().as_key(vault_root)?;
                let existing = repo.find_dir_by_path(&key)?;
                let status = classify_status(dir.metadata(), existing.as_ref());
                let record = build_dir_record(&dir, &key, parent_id, status);
                let entry = DirIndexEntry::new(record.id(), record, dir.path().clone(), status);
                (key, IndexedEntry::Dir(entry))
            }
        };
        Ok(IndexNode { inner: Classified { entry, path_key: key } })
    }
}

impl IndexNode<Classified> {
    /// The resolved PathKey, for the service to track seen paths.
    pub(crate) fn path_key(&self) -> &PathKey { &self.inner.path_key }

    /// The FsRecordId of the classified entry, for parent tracking.
    pub(crate) fn entry_id(&self) -> FsRecordId {
        match &self.inner.entry {
            IndexedEntry::File(f) => f.id(),
            IndexedEntry::Dir(d) => d.id(),
        }
    }

    /// True when the entry is a directory.
    pub(crate) fn is_dir(&self) -> bool {
        matches!(&self.inner.entry, IndexedEntry::Dir(_))
    }

    /// Extract the classified entry for accumulation.
    pub(crate) fn into_entry(self) -> IndexedEntry { self.inner.entry }
}
```

**`derive_parent_id` helper** (private fn in service.rs):

```rust
/// Derive the parent for an entry at `key`. Root-level entries return
/// `FsParentId::Root`. Subdirectory entries look up their parent from the
/// `dir_ids` map. Panics if the parent has not been classified yet
/// (walkdir guarantees parents before children, so this is a bug).
fn derive_parent_id(
    key: &PathKey,
    dir_ids: &HashMap<PathKey, FsRecordId>,
) -> FsParentId {
    let parent_key = key.parent_key();
    match parent_key {
        None => FsParentId::Root,
        Some(pk) => FsParentId::Id(
            dir_ids.get(&pk).copied()
                .expect("parent directory must be classified before child"),
        ),
    }
}
```

**`classify_status` helper** (private fn in service.rs):

```rust
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
```

### Procedural service: `IndexerService`

```rust
pub(crate) struct IndexerService<S: ScannerPort, R: Repository> {
    vault_root: DirPath,
    scanner: S,
    repo: R,
}

impl<S: ScannerPort, R: Repository> IndexerService<S, R> {
    pub(crate) fn new(vault_root: DirPath, scanner: S, repo: R) -> Self;

    pub(crate) fn run(
        &self,
        scope: IndexScope,
        opts: IndexOptions,
    ) -> Result<IndexResult, IndexerError> {
        let (root, filters) = (scope.root(), scope.filters());

        // 1. If reindex, discard all persisted state before scanning
        if opts.reindex() { self.repo.clear()?; }

        // 2. Scan + classify in one fused loop
        let mut indexed_files: Vec<FileIndexEntry> = Vec::new();
        let mut indexed_dirs: Vec<DirIndexEntry> = Vec::new();
        let mut seen_paths: HashSet<PathKey> = HashSet::new();
        let mut dir_ids: HashMap<PathKey, FsRecordId> = HashMap::new();
        let mut skipped: Vec<SkippedEntry> = Vec::new();
        let mut new_count = 0usize;
        let mut fresh_count = 0usize;
        let mut stale_count = 0usize;

        for entry in self.scanner.walk(&root, &filters)? {
            match entry? {
                ScanEntry::Skipped(s) => { skipped.push(s); continue; }
                ScanEntry::File(node) => {
                    let key = node.path().as_key(&self.vault_root)?;
                    let parent_id = derive_parent_id(&key, &dir_ids);
                    let scanned = IndexNode::new(FsNode::File(node));
                    let classified = scanned.classify(
                        &self.vault_root, parent_id, &self.repo,
                    )?;
                    seen_paths.insert(key);
                    if let IndexedEntry::File(f) = classified.into_entry() {
                        update_counts(f.status(), ...);
                        indexed_files.push(f);
                    }
                }
                ScanEntry::Dir(node) => {
                    let key = node.path().as_key(&self.vault_root)?;
                    let parent_id = derive_parent_id(&key, &dir_ids);
                    let scanned = IndexNode::new(FsNode::Dir(node));
                    let classified = scanned.classify(
                        &self.vault_root, parent_id, &self.repo,
                    )?;
                    let id = classified.entry_id();
                    seen_paths.insert(key.clone());
                    if let IndexedEntry::Dir(d) = classified.into_entry() {
                        update_counts(d.status(), ...);
                        dir_ids.insert(key, id);
                        indexed_dirs.push(d);
                    }
                }
            }
        }

        // 3. Detect deletions: persisted paths not in seen_paths
        let deleted = self.detect_deletions(&seen_paths)?;

        // 4. Persist (skip if dry_run)
        if !opts.dry_run() {
            let indexed = IndexedNodes::new(
                indexed_files.clone().into_boxed_slice(),
                indexed_dirs.clone().into_boxed_slice(),
            );
            self.persist(&indexed, &deleted)?;
        }

        // 5. Build report and return
        let report = IndexReport::new(
            indexed_files.len() + indexed_dirs.len(),
            new_count,
            fresh_count,
            stale_count,
            deleted.count(),
            skipped.into_boxed_slice(),
            Box::new([]),  // no failures in current impl
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

    fn detect_deletions(
        &self,
        seen: &HashSet<PathKey>,
    ) -> Result<DeletedNodes, IndexerError> { ... }

    fn persist(
        &self,
        indexed: &IndexedNodes,
        deleted: &DeletedNodes,
    ) -> Result<(), IndexerError> { ... }
}
```

### Pipeline details

1. **`reindex: true`** — discard all persisted state before scanning. Every
   node is treated as `New`. The clear operation deletes all records in the
   repository for this vault, so `detect_deletions` produces no deletions
   (nothing persists to compare against).
2. **Scan + classify (fused loop)** — `scanner.walk()` yields entries lazily.
   `IndexNode` resolves each entry's `PathKey` and classifies it against the
   repository inline. No intermediate `ScanResult` batch — one pass.
3. **Parent tracking via `dir_ids` map** — directories are inserted into the
   map immediately after classification, before their children are processed
   (walkdir yields parents before children). Root-level entries whose parent
   isn't in the map fall back to `FsRecordId::new()`.
4. **Deletion detection** — after the loop, query `repo.all_paths()` and
   compare against `seen_paths`. Paths in the repo but absent from the scan
   are deleted.
5. **Persist / dry_run** — `IndexedNodes` from the loop are written via
   `repo.save_many_records()`. Deleted IDs are pruned via
   `repo.delete_many_records()`. Skipped when `dry_run: true`.
6. **IndexReport** — built from counters accumulated during the loop, returned
   as part of `IndexResult`.

Hard abort conditions (return an error, do not return a partial result):
configuration errors (invalid vault root, missing config specs) and repository
initialisation failures.

The service must depend only on the `ScannerPort` and `Repository` traits —
no walkdir, no redb, no concrete adapter types in the service module.
`IndexNode` depends on `ReadRepository` for its `classify()` transition.

## File change summary

| File                       | Change                                                               |
| -------------------------- | -------------------------------------------------------------------- |
| `indexer/error.rs`           | Add `Path(#[from] PathError)` variant to `IndexerError`                  |
| `indexer/scan.rs`            | Add `IndexScope::root()` and `IndexScope::filters()`                     |
| `indexer/repository.rs`      | Add `WriteRepository::clear()` + update test `MockRepository`            |
| `indexer/storage/testing.rs` | Implement `InMemoryRepository::clear()`                                |
| `indexer/storage/write.rs`   | Implement `RedbRepository::clear()` (drain all 8 tables in write tx)   |
| `indexer/mod.rs`             | Add `mod service;`                                                     |
| `indexer/summary.rs`         | Add `report: IndexReport` to `IndexResult`; update constructor to 3 args |

## Acceptance criteria

- [ ] `IndexNode<Scanned>::classify()` classifies missing persisted nodes
      as `New` and nodes with matching metadata as `Fresh`.
- [ ] `IndexNode<Scanned>::classify()` classifies changed metadata nodes as
      `Stale`.
- [ ] `derive_parent_id()` returns `FsParentId::Root` for root-level entries,
      `FsParentId::Id(id)` for subdirectory entries found in `dir_ids`.
- [ ] Service's fused loop yields correct `IndexedNodes` and `seen_paths`
      from a mock scanner stream (no real filesystem).
- [ ] `detect_deletions` prunes persisted paths absent from `seen_paths` and
      reports them in `DeletedNodes`.
- [ ] `persist` calls `save_many_records` with indexed entries and
      `delete_many_records` with deleted IDs; no-write when `dry_run: true`.
- [ ] `ScanEntry::Skipped` entries are accumulated into `IndexReport::skipped`
      without aborting the run; `IndexResult::report().skipped()` returns them.
- [ ] Scope tests: `Full` and `Partial` scans delegate the correct `root:
      DirPath` and `filters` to `scanner.walk()`.
- [ ] `reindex: true` calls `repo.clear()` before scanning, yielding all
      entries as `New` and zero deletions.
- [ ] All application-service tests use mock `ScannerPort` and `InMemoryRepository`
      — no real disk or redb dependency.
- [ ] `IndexerError` is extended with `Path` variant for path resolution failures.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## TDD plan

### Cycle 1 — IndexNode typestate (`InMemoryRepository`)

**RED→GREEN per test:**

| Test name | What it verifies |
|---|---|
| `scanned_new_wraps_fs_node` | `IndexNode::new(file_node)` stores it; `IndexNode::new(dir_node)` stores it |
| `classify_file_new` | No existing record in repo → `IndexStatus::New`, new `FsRecordId` |
| `classify_file_fresh` | Matching metadata in repo → `IndexStatus::Fresh`, existing `FsRecordId` |
| `classify_file_stale` | Different metadata in repo → `IndexStatus::Stale`, existing `FsRecordId` |
| `classify_dir_new` | No existing dir record → `IndexStatus::New` |
| `classify_dir_fresh` | Matching dir metadata → `IndexStatus::Fresh` |
| `classify_dir_stale` | Changed dir metadata → `IndexStatus::Stale` |
| `classify_handles_outside_path` | Path outside vault root → `IndexerError::Path` |
| `classified_path_key` | `path_key()` returns the resolved `PathKey` |
| `classified_into_entry` | `into_entry()` returns the `IndexedEntry` |
| `classified_entry_id` | `entry_id()` returns the record's `FsRecordId` |
| `classified_is_dir` | `is_dir()` returns true for dir, false for file |
| `derive_parent_id_root_level` | Root-level file → `FsParentId::Root` |
| `derive_parent_id_subdirectory` | File in subdir → `FsParentId::Id(id)` from dir_ids |
| `derive_parent_id_panics_on_missing_parent` | Orphan file (parent not in map) → panic (programmer error) |

### Cycle 2 — IndexerService::run() fused loop (`MockScannerPort` + `InMemoryRepository`)

| Test name | What it verifies |
|---|---|
| `empty_scan` | No entries → empty `IndexedNodes`, zero counts in report |
| `single_file` | One `ScanEntry::File` → one file in `IndexedNodes`, report.new=1 |
| `single_dir` | One `ScanEntry::Dir` → one dir in `IndexedNodes`, report.new=1 |
| `full_scope_delegates_root_and_filters` | `IndexScope::Full` passes correct root+filters to `scanner.walk()` |
| `partial_scope_delegates_root_and_filters` | `IndexScope::Partial` passes correct root+filters |
| `reindex_clears_repo_before_scan` | `reindex: true` → `repo.clear()` called before `walk()`; all entries New |
| `skipped_entries_do_not_abort` | Stream with `ScanEntry::Skipped` → loop continues, skipped in report |

### Cycle 3 — detect_deletions (`InMemoryRepository` with pre-seeded data)

| Test name | What it verifies |
|---|---|
| `no_deletions_when_all_seen` | Repo paths all in `seen` → `DeletedNodes` empty |
| `detects_missing_paths` | Path in repo but not in `seen` → included in `DeletedNodes` |
| `empty_repo_no_deletions` | No persisted paths → empty `DeletedNodes` |
| `mixed_files_and_dirs_deleted` | Both file and dir paths missing → both IDs in `DeletedNodes` |

### Cycle 4 — persist (`InMemoryRepository`)

| Test name | What it verifies |
|---|---|
| `persists_indexed_entries` | After `run()`, repo contains saved file+dir records |
| `deletes_deleted_entries` | After `run()`, entries in `DeletedNodes` are removed from repo |
| `dry_run_skips_persistence` | `dry_run: true` → repo state unchanged from before `run()` |
| `reindex_no_deletions` | `reindex: true` → repo empty after clear → no deletions detected |

### Cycle 5 — Integration (`MockScannerPort` + `InMemoryRepository`)

| Test name | What it verifies |
|---|---|
| `full_integration_mixed_entries` | Files + dirs + skipped → correct `IndexedNodes`, `DeletedNodes`, `IndexReport` |
| `partial_scope_and_reindex` | Partial scope + reindex → clear all, scan only partial root |
| `dry_run_no_side_effects` | Full scan with `dry_run` → repo unchanged, result still populated |
| `scan_classify_persist_roundtrip` | First run persists; second run (no changes) shows all Fresh |
| `report_counts_are_accurate` | New/fresh/stale/deleted/skipped counts match actual entries |

## Blocked by

- 03-ports-and-adapters.md

---

## Changelog

### 2026-06-18 — Session 6: TDD plan + design refinements

**No commits yet** — planning session.

**Key changes from Session 5 design:**
1. **HashMap parent tracking** — `classify()` now takes `dir_ids: &HashMap<PathKey, FsRecordId>` to resolve `parent_id`. Dirs inserted into map immediately after classification so children can find them.
2. **IndexReport in IndexResult** — `IndexResult` gains `report: IndexReport` field. Report built from loop counters (new/fresh/stale/deleted/skipped).
3. **IndexNode<Classified> gains `entry_id()` and `is_dir()`** — needed by service loop for dir map insertion and status tracking.
4. **5-cycle TDD plan** — IndexNode typestate → fused loop → detect_deletions → persist → integration.
5. **7 existing files modified** — see File change summary above.

### 2026-06-17 — Session 5: Fused loop + per-entry typestate redesign

**No commits yet** — design session.

Redesigned the application service from a typestate processor (`IndexerProcessor<P>`)
to a procedural service with scoped per-entry typestate (`IndexNode<State>`).

**Key changes**:
1. **Fused scan+classify loop**: `scanner.walk()` yields entries lazily;
   `IndexNode<Scanned>::classify()` classifies each entry against the repo inline. No
    batch `ScanResult`, no second pass.
2. **Per-entry typestate**: `IndexNode<Scanned> → IndexNode<Classified>`.
    No `PhantomData`, no `Option` — state types ARE the data carriers. The
    typestate is scoped to the `for` loop — guarantees every entry resolves
    `PathKey` before consumption without infecting the service's orchestration.
3. **Procedural service**: `IndexerService::run()` orchestrates the loop,
   deletion detection, persist, and report building. Each is a private helper
   method.
4. **`IndexScope` uses `DirPath`**, `ScannerPort` is streaming, `WalkdirAdapter`
   is zero-size — all from the issue-03 redesign.

**Status**: Design agreed with stakeholder. Awaiting issue 03 implementation
before issue 04 can proceed.

---

## Triage Notes

> *These notes were updated during Session 6 to reflect parent-tracking and IndexReport refinements.*

**Verdict**: `ready-for-agent` (pending issue 03 streaming-scanner implementation).

**What was checked:**

- Fused scan+classify loop eliminates the two-pass problem identified during
  adversarial review of the initial typestate-processor design.
- `IndexNode<State>` provides compile-time classification ordering without
  the boilerplate of a full pipeline typestate processor.
- `reindex: true` now calls `repo.clear()` before scanning — simpler than the
  per-entry "treat as New" logic from the original design. This matches the PRD
  ("discard all persisted state") and works identically for both Full and
  Partial scope (clear clears everything; subsequent scan only repopulates
  seen paths).
- `dry_run: true` skips `persist()` after classification — same as original design.
- Per-node `ScanEntry::Skipped` accumulation matches PRD Section 7 (non-fatal
  I/O failures); skipped entries appear in `IndexReport::skipped` on the returned
  `IndexResult`.
- Service depends only on `ScannerPort` and `Repository` traits — correct
  hexagonal boundary.
- All AC items test through the service's `run()` method with mock ports.
- **New**: `HashMap<PathKey, FsRecordId>` parent tracking ensures `parent_id`
  correctness for non-root entries; root-level entries use `FsRecordId::new()`
  fallback (documented known gap).
- **New**: `IndexResult` carries `IndexReport` for downstream consumers.

**Known gaps (deferred):**
- The vault root itself has no persisted `DirRecord`. `FsParentId::Root` maps
  to `FsRecordId::new()` for `FileRecord.parent_id` and `None` for
  `DirRecord.parent_id` — these values are transient per run. A future
  enhancement could persist a root `DirRecord` for consistency.
