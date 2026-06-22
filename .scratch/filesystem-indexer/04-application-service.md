---
title: 04-indexer-application-service
category: enhancement
label: ready-for-agent
status: open
branch: 04-application-service
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

The service is **procedural** (wires the pipeline) with a **per-entry typestate**
(`EntryBuilder<S>`) used only inside the scan loop. The typestate ensures entries
flow through a 5-state pipeline: `Init → Comparison → Persistence → Indexed → Completion`,
with a branch at Comparison that short-circuits Fresh entries directly to Indexed.

The typestate lives in a dedicated `builder.rs` module, not inline in `service.rs`. State
types ARE the data carriers (no `PhantomData`, no `Option`).

**Parent tracking**: a `HashMap<PathKey, FsRecordId>` (`dir_ids`) is maintained
in the service's `run()` loop via `IndexCollector`. As directories are classified,
their `FsRecordId` is inserted. Files and subdirs derive their parent's `PathKey` by
calling `path_key.parent()` and looking it up in the map. When the derivation produces
`None` (root-level entry), `FsParentId::Root` is returned.

**IndexReport in return**: `IndexResult` gains a `report: IndexReport` field. The
report is built from counters accumulated in `IndexCollector` during the loop
(new/fresh/stale per entry, deleted count, skipped entries).

**Fresh entry short-circuit**: `FileComparison::into_comparison_branch(repo)` and
`DirComparison::into_comparison_branch(repo)` return a `*ComparisonBranch` enum:
- `Match(EntryBuilder<FileIndexed>)` — constructs Indexed state directly from the
  existing DB record, skipping Persistence entirely
- `Mismatch(EntryBuilder<FilePersistence>)` — flows through Persistence as before

Match condition uses `is_size_match` + `is_timestamp_match` for files, `FsTimes::is_match`
for dirs — partial equality rather than full `PartialEq`.

Data flow:

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
  ├─ let mut ctx = IndexCollector::default()
  │
  ├─ for result in scanner.walk(&root, &filter)?
  │   │
  │   ├─ ScanEntry → EntryBuilder::<Init>::from_scan_entry(entry)
  │   │   └─ .into_branch(&vault_root)?
  │   │      ├─ EntryBranch::Completion(b) → b.into_state() → ctx.record()
  │   │      ├─ EntryBranch::File(b) → b.into_comparison_branch(&repo)?
  │   │      │   ├─ Match(b) → b.into_completion()
  │   │      │   └─ Mismatch(b) → b.into_indexed(..., &ctx.dir_ids, ...)?
  │   │      │                   → .into_completion()
  │   │      └─ EntryBranch::Dir(b) → (same pattern as File)
  │   │
  │   └─ ctx.record(completion) → updates counters, pushes to vectors
  │
  ├─ let deleted = detect_deletions(&ctx.seen_paths)?
  ├─ if !dry_run → repo.delete_many_records(deleted.files(), deleted.dirs())?
  └─ IndexResult::new(
       IndexedNodes::new(ctx.indexed_files, ctx.indexed_dirs),
       deleted,
       IndexReport::new(..., ctx.skipped, ...),
     )
```

### `FsParentId` type (in `model.rs` alongside `FsRecordId`)

```rust
/// Identifies the parent of a filesystem node during indexing.
///
/// `Root` represents the vault root itself — used when an entry is directly
/// in the vault root (no parent directory exists). `Id(id)` represents a
/// specific indexed directory that contains this entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub(crate) enum FsParentId {
    /// Entry is directly under the vault root.
    Root,
    /// Entry is inside a known indexed directory.
    Id(FsRecordId),
}
```

`FsParentId` has a `to_storage_key()` method that maps `Root` to a zero
sentinel ([`FsRecordId::MAX`]) and `Id(id)` to `id`. This is used by
parent-index tables (which have `FsRecordId` keys) without storing an
`Option<FsRecordId>`.

### Per-entry typestate: `EntryBuilder<S>` (in `builder.rs`)

Five-state linear typestate pipeline with a branch at Comparison.

```rust
// ─── State types ──────────────────────────────────────────────────

struct Init { entry: ScanEntry }

struct FileComparison { node: FileNode, path_key: PathKey }
struct DirComparison  { node: DirNode,  path_key: PathKey }

struct FilePersistence { node: FileNode, path_key: PathKey, status: IndexStatus, id: FsRecordId }
struct DirPersistence  { node: DirNode,  path_key: PathKey, status: IndexStatus, id: FsRecordId }

struct FileIndexed { record: FileRecord, path: FsFilePath, path_key: PathKey, status: IndexStatus, id: FsRecordId }
struct DirIndexed  { record: DirRecord,  path: FsDirPath,  path_key: PathKey, status: IndexStatus, id: FsRecordId }

struct Completion { kind: CompletionKind }

enum CompletionKind {
    File { entry: FileIndexEntry, path_key: PathKey },
    Dir  { entry: DirIndexEntry,  path_key: PathKey, id: FsRecordId },
    Skipped(SkippedEntry),
}
```

**Branch types** — returned by `into_comparison_branch`:

```rust
enum FileComparisonBranch {
    Match(EntryBuilder<FileIndexed>),
    Mismatch(EntryBuilder<FilePersistence>),
}

enum DirComparisonBranch {
    Match(EntryBuilder<DirIndexed>),
    Mismatch(EntryBuilder<DirPersistence>),
}
```

**Branch entry point** (from `Init`):

```rust
enum EntryBranch {
    File(EntryBuilder<FileComparison>),
    Dir(EntryBuilder<DirComparison>),
    Completion(EntryBuilder<Completion>),
}
```

**Transitions:**

- `Init::into_branch(vault_root)` → `EntryBranch`
- `FileComparison::into_comparison_branch(repo)` → `FileComparisonBranch`
- `DirComparison::into_comparison_branch(repo)` → `DirComparisonBranch`
- `FilePersistence::into_indexed(repo, dir_ids, dry_run)` → `EntryBuilder<FileIndexed>`
- `DirPersistence::into_indexed(repo, dir_ids, dry_run)` → `EntryBuilder<DirIndexed>`
- `FileIndexed::into_completion()` / `DirIndexed::into_completion()` → `EntryBuilder<Completion>`

Match condition uses partial equality for staleness:

```rust
// File
node.metadata().is_size_match(record.metadata().size())
    && node.metadata().is_timestamp_match(
        record.metadata().times().created_at(),
        record.metadata().times().modified_at(),
    )

// Dir
node.metadata().times().is_match(record.metadata().times())
```

**Parent derivation** — inlined in the builder's `into_indexed()`:

```rust
let parent_id = state
    .path_key
    .parent()
    .and_then(|pk| dir_ids.get(&pk).copied())
    .map_or(FsParentId::Root, FsParentId::Id);
```

Uses `PathKey::parent()` (strips last component via `/`) instead of a standalone
`derive_parent_id` function.

### Scan accumulator: `IndexCollector` (in `service.rs`)

```rust
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
            CompletionKind::File { entry, path_key } => {
                self.seen_paths.insert(path_key);
                self.increment(entry.status());
                self.indexed_files.push(entry);
            }
            CompletionKind::Dir { entry, path_key, id } => {
                self.seen_paths.insert(path_key.clone());
                self.dir_ids.insert(path_key, id);
                self.increment(entry.status());
                self.indexed_dirs.push(entry);
            }
            CompletionKind::Skipped(s) => self.skipped.push(s),
        }
    }

    fn increment(&mut self, status: IndexStatus) {
        match status {
            IndexStatus::New => self.new_count += 1,
            IndexStatus::Fresh => self.fresh_count += 1,
            IndexStatus::Stale => self.stale_count += 1,
        }
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
        scope: &IndexScope,
        opts: IndexOptions,
    ) -> Result<IndexResult, IndexerError> {
        let root = scope.root();
        let filters = scope.filters();

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

        for entry in self.scanner.walk(root, filters)? {
            match entry {
                Ok(ScanEntry::Skipped(s)) => skipped.push(s),
                Ok(ScanEntry::File(node)) => {
                    let key = node.path().as_key(&self.vault_root)?;
                    let parent_id = derive_parent_id(&key, &dir_ids);
                    let scanned = IndexNode::new(FsNode::File(node));
                    let classified = scanned.classify(
                        &self.vault_root, parent_id, &self.repo,
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
                        &self.vault_root, parent_id, &self.repo,
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

        // 3. Detect deletions: persisted paths not in seen_paths
        let deleted = self.detect_deletions(&seen_paths)?;

        // 4. Persist (skip if dry_run)
        if !opts.dry_run() {
            self.persist(&seen_paths, &deleted)?;
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
   repository via `delete_table`+`open_table` on each of the 8 tables within
   a single write transaction (atomic). Afterwards, `detect_deletions` produces
   no deletions (nothing persists to compare against).
2. **Scan + classify (fused loop)** — `scanner.walk()` yields entries lazily.
   Each entry flows through the `EntryBuilder` pipeline via the `into_branch` /
   `into_comparison_branch` chain — one pass, no intermediate batch.
3. **Parent tracking via `dir_ids` map** — `path_key.parent()` extracts the parent
   key via `PathKey`'s internal path normalization. Root-level entries (no `/`)
   return `FsParentId::Root`. Subdirectory entries look up their parent in the
   `dir_ids` map (walkdir guarantees parents before children).
4. **Fresh short-circuit** — `FileComparison::into_comparison_branch` checks
   `is_size_match` + `is_timestamp_match` for files, `FsTimes::is_match` for dirs.
   Match constructs `FileIndexed`/`DirIndexed` directly from the existing DB record
   with `IndexStatus::Fresh`, skipping Persistence entirely.
5. **Deletion detection** — after the loop, query `repo.all_paths()` and compare
   against `seen_paths`. Paths in the repo but absent from the scan are deleted.
6. **Persist / dry_run** — Mismatch entries persist via `into_indexed()`, which
   calls `repo.save_file()`/`repo.save_dir()` unless `dry_run` is true. Deletions
   are removed via `repo.delete_many_records()`. Both skip when `dry_run: true`.
7. **IndexReport** — built from `IndexCollector` counters after the loop (including
   deleted count from `detect_deletions`), returned as part of `IndexResult`.

Hard abort conditions: path resolution errors (`PathKey::as_key` fails), repo
errors, and scanner errors (fatal). `ScanEntry::Skipped` entries are non-fatal —
accumulated into `ctx.skipped` and appear in `IndexReport::skipped()`.

The service depends only on the `ScannerPort` and `Repository` traits —
no walkdir, no redb, no concrete adapter types in the service module.
`EntryBuilder` depends on `ReadRepository` / `WriteRepository` for its
transitions.

## File change summary

| File                       | Change                                                               |
| -------------------------- | -------------------------------------------------------------------- |
| `indexer/service.rs`       | NEW — IndexerService, IndexCollector, 16 tests (run, deletions, persist, integration) |
| `indexer/builder.rs`       | NEW — EntryBuilder<S> 5-state typestate, ComparisonBranch enums, 3 tests |
| `indexer/error.rs`         | Add `Path(#[from] PathError)` variant to `IndexerError`              |
| `indexer/scan.rs`          | Add `IndexScope::root()` and `IndexScope::filters()`                 |
| `indexer/repository.rs`    | Add `FsParentId` to trait signatures, `WriteRepository::clear()`     |
| `indexer/model.rs`         | Add `FsParentId` enum with rkyv derives + `to_storage_key()`         |
| `indexer/storage/testing.rs` | Implement `InMemoryRepository::clear()` + parent-index table methods |
| `indexer/storage/write.rs` | `clear()` uses `delete_table`+`open_table` (not iterate+remove)      |
| `indexer/storage/read.rs`  | Parent-index lookups accept `FsParentId`                              |
| `indexer/entry.rs`         | Use `FsParentId` in `FileIndexEntry`/`DirIndexEntry` constructors    |
| `indexer/summary.rs`       | Add `report: IndexReport` to `IndexResult`; update constructor       |
| `indexer/mod.rs`           | Add `mod service; mod builder;`                                      |

## Acceptance criteria

- [x] `EntryBuilder<FileComparison>::into_comparison_branch` classifies missing persisted
      nodes as `New` (Mismatch), matching metadata as `Fresh` (Match), and changed
      metadata as `Stale` (Mismatch). Match uses `is_size_match` + `is_timestamp_match`
      for files, `FsTimes::is_match` for dirs.
- [x] Parent derivation (`path_key.parent()`) returns `FsParentId::Root` for root-level
      entries, `FsParentId::Id(id)` for subdirectory entries found in `dir_ids`.
- [x] Service's fused loop yields correct `IndexedNodes` and `seen_paths` from a mock
      scanner stream (no real filesystem). Loop uses `IndexCollector` to accumulate
      entries, counters, and skipped entries.
- [x] `detect_deletions` prunes persisted paths absent from `seen_paths` and reports
      them in `DeletedNodes`.
- [x] Fresh entries skip Persistence entirely (Match branch constructs `FileIndexed`/
      `DirIndexed` directly from existing DB record). Mismatch entries flow through
      `Persistence → Indexed`. `dry_run` skips save calls in `into_indexed`.
- [x] `ScanEntry::Skipped` entries are accumulated into `ctx.skipped` via `CompletionKind::Skipped`
      without aborting the run. Appear in `IndexReport::skipped()`.
- [ ] Scope tests: `Full` and `Partial` scans delegate the correct `root: DirPath` and
      `filters` to `scanner.walk()`. — **No dedicated delegation test; covered indirectly
      by integration tests.**
- [x] `reindex: true` calls `repo.clear()` before scanning, yielding all entries as `New`.
- [x] All application-service tests use `MockScanner` (mock `ScannerPort`) and
      `InMemoryRepository` — no real disk or redb dependency.
- [x] `IndexerError` is extended with `Path(#[from] PathError)` variant for path
      resolution failures.
- [x] All existing tests pass (`mise run test`).
- [x] No clippy warnings (`mise run lint`).

## Tests implemented

### Builder tests (`lithos-core/src/indexer/builder.rs`)

| Test name | What it verifies |
|---|---|
| `test_init_to_comparison_file` | `Init::into_branch` wraps `FileNode` in `FileComparison` |
| `test_init_to_comparison_dir` | `Init::into_branch` wraps `DirNode` in `DirComparison` |
| `test_full_pipeline_file_new` | New file: `Init→FileComparison→Mismatch→FilePersistence→FileIndexed→Completion`, report matches |

### Service tests (`lithos-core/src/indexer/service.rs`)

**Cycle 1 — `IndexerService::run()` basic:**

| Test name | What it verifies |
|---|---|
| `empty_scan` | No entries → empty `IndexedNodes`, zero counts in report |
| `single_file` | One `ScanEntry::File` → one file in `IndexedNodes`, report.new=1 |
| `single_dir` | One `ScanEntry::Dir` → one dir in `IndexedNodes`, report.new=1 |
| `reindex_clears_repo_before_scan` | `reindex: true` → `repo.clear()` called before `walk()`; all entries New |
| `skipped_entries_do_not_abort` | Stream with `ScanEntry::Skipped` → loop continues, skipped in report |

**Cycle 2 — detect_deletions:**

| Test name | What it verifies |
|---|---|
| `no_deletions_when_all_seen` | Repo paths all in `seen` → `DeletedNodes` empty |
| `detects_missing_paths` | Path in repo but not in `seen` → included in `DeletedNodes` |
| `empty_repo_no_deletions` | No persisted paths → empty `DeletedNodes` |
| `mixed_files_and_dirs_deleted` | Both file and dir paths missing → both IDs in `DeletedNodes` |

**Cycle 3 — persist:**

| Test name | What it verifies |
|---|---|
| `persists_indexed_entries` | After `run()`, repo has saved file record |
| `dry_run_skips_persistence` | `dry_run: true` → repo state unchanged |
| `reindex_no_deletions` | `reindex: true` → repo empty after clear → no deletions detected |

**Cycle 4 — Integration:**

| Test name | What it verifies |
|---|---|
| `full_integration_mixed_entries` | Files + dirs → correct `IndexedNodes`, `IndexReport` |
| `dry_run_no_side_effects` | Full scan with `dry_run` → result populated, repo unchanged |
| `report_counts_are_accurate` | New/fresh/stale/deleted/skipped counts match actual entries |
| `partial_scope_and_reindex` | Partial scope + reindex → clear all, scan only partial root |

### Missing from original TDD plan

| Planned test | Status | Notes |
|---|---|---|
| `full_scope_delegates_root_and_filters` | ❌ Not implemented | Coverage gap AC-8 |
| `partial_scope_delegates_root_and_filters` | ❌ Not implemented | Coverage gap AC-8 |
| `deletes_deleted_entries` | ❌ Not implemented | No test asserting deleted IDs removed from repo |
| `scan_classify_persist_roundtrip` | ❌ Not implemented | No test asserting second run shows Fresh |
| IndexNode-specific tests (14) | N/A | Replaced by EntryBuilder in builder.rs |

## Blocked by

- 03-ports-and-adapters.md

---

## Changelog

### 2026-06-22 — Session 10: Short-circuit + IndexCollector + builder refinements

**Commit**: `38da2cd0` on `04-application-service`

**Implemented:**
1. `FsMetadata::is_match()` — cross-variant metadata comparison (files use `PartialEq`,
   dirs use `PartialEq`). Available for downstream consumers.
2. `FileComparison::into_comparison_branch(repo)` / `DirComparison::into_comparison_branch(repo)` —
   replaces `into_persistence`. Returns Match/Mismatch enum. Match constructs
   `FileIndexed`/`DirIndexed` directly from existing DB record, skipping Persistence.
3. Match condition uses `is_size_match` + `is_timestamp_match` for files,
   `FsTimes::is_match` for dirs (partial equality, not full `PartialEq`).
4. `IndexCollector` — named struct encapsulating the 8 mutable accumulators
   (indexed_files, indexed_dirs, seen_paths, dir_ids, skipped, counters).
   `record(&mut self, Completion)` method replaces the inline match block.
5. `state` field on `EntryBuilder` made private; `state()` and `into_state()` accessors.
6. Method renames: all `transition()` → `into_branch`, `into_comparison_branch`,
   `into_indexed`, `into_completion`.

**Deviations from spec:**
- `IndexNode<State>` typestate → `EntryBuilder<S>` in dedicated `builder.rs` module.
- `classify()` → `into_branch()` / `into_comparison_branch()` chain with 5 states.
- `derive_parent_id` helper → inlined `path_key.parent().and_then(...)` in builder.
- `persist()` method → inline in builder pipeline (`into_indexed` calls
  `repo.save_file()`/`repo.save_dir()` directly).
- `IndexedEntry` enum removed entirely — `CompletionKind` serves the same role.
- No separate `persist()` method — deletions handled via
  `repo.delete_many_records()` in `run()`, saves happen per-entry in `into_indexed`.
- 16 service tests + 3 builder tests (original spec planned 36+).

**Test infrastructure:**
- `MockScanner` uses `RefCell<Vec<...>>` for interior mutability.
- `make_vault_root()` creates `/tmp/vault` on disk.
- `make_file_node()` / `make_dir_node()` create real files/dirs on disk.
- 2002 total tests pass (no regressions).

**Quality:**
- Zero clippy warnings (all targets).
- All pre-commit hooks pass (gitleaks, conventional-commits, fmt, clippy, tests).

**Coverage gaps:**
- `full_scope_delegates_root_and_filters` / `partial_scope_delegates_root_and_filters` —
  no test directly verifies scope→walk parameter delegation (AC-8).
- `deletes_deleted_entries` — no test asserting deleted IDs are removed from repo.
- `scan_classify_persist_roundtrip` — no test asserting second run shows Fresh.

### 2026-06-18 — Session 7: Implementation (committed)

**Commit**: `00a0f45f` on `04-application-service`

**Implemented:**
1. `service.rs` (1407 lines) — `IndexerService` with `IndexNode<S>` typestate,
   `run()` fused loop, `detect_deletions()`, `persist()`, and 36 tests across
   5 cycles (typestate, service_run, detect_deletions, persist, integration).
2. `FsParentId` enum in `model.rs` with `to_storage_key()` mapping `Root` to
   zero sentinel. Derives `Archive+Serialize+Deserialize` for rkyv embedding.
3. Parent-index tables in read/write/testing backends — `list_files_by_parent`
   and `list_dirs_by_parent` accept `FsParentId`.
4. `clear()` uses `delete_table`+`open_table` within a single `WriteTransaction`
   instead of iterating all keys — simpler and allocation-free.

**Deviation from spec:**
- `classify()` takes `&self` not `self` (state types behind `&`).
- `derive_parent_id` uses `s.rfind('/')` on `PathKey::as_str()` (no `parent_key()` method).
- `run()` takes `scope: &IndexScope` to satisfy `clippy::needless_pass_by_value`.
- `FsParentId` has rkyv derives for redb storage (spec didn't mention rkyv).
- `Scope::root()` returns `&DirPath` (not owned); `scope.root()` renamed from
  `Scope::root()` to `IndexScope::root()` after scan module restructure.

**Test infrastructure:**
- `MockScanner` uses `RefCell<Vec<...>>` for interior mutability (avoids Clone
  bound on `ScanEntry`/`ScannerError`).
- `make_vault_root()` creates `/tmp/vault` on disk (required by `DirPath::try_new`).
- `make_file_node()` / `make_dir_node()` create real files/dirs before construction.
- `repo_with_file()` / `repo_with_dir()` helpers for pre-seeded InMemoryRepository.
- 36 tests all pass, 2067 total (no regressions).

**Quality:**
- Zero clippy warnings (all targets).
- `cargo fmt --check` clean.
- All pre-push hooks pass (gitleaks, conventional-commits, fmt, clippy, tests).

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
