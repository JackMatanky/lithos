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
along with a per-entry typestate classifier (`ScannedNode<State>`). The service
accepts an `IndexScope`, `IndexOptions`, a `ScannerPort`, and a `Repository`,
and returns an `IndexResult`.

### Architecture

The service is **procedural** (wires the pipeline) with a **scoped typestate**
(`ScannedNode<State>`) used only inside the scan loop. The typestate ensures
per-entry classification always resolves `PathKey` before the entry can be
consumed — a compile-time guarantee on a single entry, not on the whole
pipeline.

Data flow:

```
Service::run(scope, opts, scanner, repo)
  │
  ├─ resolve_scope(scope) → (root: DirPath, filters: ScanFilters)
  │
  ├─ for entry in scanner.walk(&root, &filters)?
  │   │
  │   └─ ScannedNode::from_fsnode(entry?)         // ScanEntry → ScannedNode<Discovered>
  │      └─ node.index(&vault_root, &repo)?       // classify: resolve PathKey + compare metadata
  │         └─ node.into_entry()                   // ScannedNode<Indexed> → FileIndexEntry/DirIndexEntry
  │
  ├─ detect_deletions(seen_paths, &repo)?
  ├─ persist(indexed, deleted, opts)?              // skip if dry_run
  └─ IndexResult::new(indexed, deleted)
```

### Per-entry typestate: `IndexNode<State>`

Defined in a new module `lithos-core::indexer::classify` (or inline in
`service.rs` — TBD during implementation). Two-state linear typestate. State
types ARE the data carriers — no separate data wrapper:

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

    /// Resolve vault-relative PathKey, query repository, classify.
    /// This is the sole transition — after this the entry is ready to consume.
    pub(crate) fn classify(
        self,
        vault_root: &DirPath,
        repo: &impl ReadRepository,
    ) -> Result<IndexNode<Classified>, IndexerError> {
        let (key, entry) = match self.inner.0 {
            FsNode::File(file) => {
                let key = file.path().as_key(vault_root)?;
                let existing = repo.find_file_by_path(&key)?;
                (key, IndexedEntry::File(
                    FileIndexEntry::new(..., classify_file(&file, &existing)),
                ))
            }
            FsNode::Dir(dir) => {
                let key = dir.path().as_key(vault_root)?;
                let existing = repo.find_dir_by_path(&key)?;
                (key, IndexedEntry::Dir(
                    DirIndexEntry::new(..., classify_dir(&dir, &existing)),
                ))
            }
        };
        Ok(IndexNode { inner: Classified { entry, path_key: key } })
    }
}

impl IndexNode<Classified> {
    /// The resolved PathKey, for the service to track seen paths.
    pub(crate) fn path_key(&self) -> &PathKey { &self.inner.path_key }

    /// Extract the classified entry for accumulation.
    pub(crate) fn into_entry(self) -> IndexedEntry { self.inner.entry }
}
```

### Procedural service: `IndexerService`

```rust
pub(crate) struct IndexerServiceConfig;

pub(crate) struct IndexerService<S: ScannerPort, R: Repository> {
    vault_root: DirPath,
    config: IndexerServiceConfig,
    scanner: S,
    repo: R,
}

impl<S: ScannerPort, R: Repository> IndexerService<S, R> {
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
        let mut skipped: Vec<SkippedEntry> = Vec::new();

        for entry in self.scanner.walk(&root, &filters)? {
            match entry? {
                ScanEntry::Skipped(s) => { skipped.push(s); continue; }
                ScanEntry::File(node) | ScanEntry::Dir(node) => {
                    let scanned = IndexNode::new(node);
                    let classified = scanned.classify(&self.vault_root, &self.repo)?;
                    seen_paths.insert(classified.path_key().clone());
                    match classified.into_entry() {
                        IndexedEntry::File(f) => indexed_files.push(f),
                        IndexedEntry::Dir(d) => indexed_dirs.push(d),
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

        Ok(IndexResult::new(
            IndexedNodes::new(
                indexed_files.into_boxed_slice(),
                indexed_dirs.into_boxed_slice(),
            ),
            deleted,
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
   `ScannedNode` resolves each entry's `PathKey` and classifies it against the
   repository inline. No intermediate `ScanResult` batch — one pass.
3. **Deletion detection** — after the loop, query `repo.all_paths()` and
   compare against `seen_paths`. Paths in the repo but absent from the scan
   are deleted.
4. **Persist / dry_run** — `IndexedNodes` from the loop are written via
   `repo.save_many_records()`. Deleted IDs are pruned via
   `repo.delete_many_records()`. Skipped when `dry_run: true`.

Hard abort conditions (return an error, do not return a partial result):
configuration errors (invalid vault root, missing config specs) and repository
initialisation failures.

The service must depend only on the `ScannerPort` and `Repository` traits —
no walkdir, no redb, no concrete adapter types in the service module.
`IndexNode` depends on `ReadRepository` for its `classify()` transition.

## Acceptance criteria

- [ ] `IndexNode<Scanned>::classify()` classifies missing persisted nodes
      as `New` and nodes with matching metadata as `Fresh`.
- [ ] `IndexNode<Scanned>::classify()` classifies changed metadata nodes as
      `Stale`.
- [ ] `IndexNode<Scanned>::classify()` classifies all nodes as `New` when
      `IndexOptions { reindex: true }` is set, regardless of stored metadata.
- [ ] Service's fused loop yields correct `IndexedNodes` and `seen_paths`
      from a mock scanner stream (no real filesystem).
- [ ] `detect_deletions` prunes persisted paths absent from `seen_paths` and
      reports them in `DeletedNodes`.
- [ ] `persist` calls `save_many_records` with indexed entries and
      `delete_many_records` with deleted IDs; no-write when `dry_run: true`.
- [ ] `ScanEntry::Skipped` entries are accumulated into `IndexReport::skipped`
      without aborting the run.
- [ ] Scope tests: `Full` and `Partial` scans delegate the correct `root:
      DirPath` and `filters` to `scanner.walk()`.
- [ ] All application-service tests use mock `ScannerPort` and mock `Repository`
      — no real disk or redb dependency.
- [ ] `IndexerError` is extended with variants needed for service orchestration
      (configuration errors, repository initialization, scanning failures).
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- 03-ports-and-adapters.md

---

## Changelog

### 2026-06-17 — Session 5: Fused loop + per-entry typestate redesign

**No commits yet** — design session.

Redesigned the application service from a typestate processor (`IndexerProcessor<P>`)
to a procedural service with scoped per-entry typestate (`ScannedNode<State>`).

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

> *These notes were updated during Session 5 to reflect the redesigned architecture.*

**Verdict**: `ready-for-agent` (pending issue 03 streaming-scanner implementation).

**What was checked:**

- Fused scan+classify loop eliminates the two-pass problem identified during
  adversarial review of the initial typestate-processor design.
- `ScannedNode<State>` provides compile-time classification ordering without
  the boilerplate of a full pipeline typestate processor.
- `reindex: true` now calls `repo.clear()` before scanning — simpler than the
  per-entry "treat as New" logic from the original design. This matches the PRD
  ("discard all persisted state") and works identically for both Full and
  Partial scope (clear clears everything; subsequent scan only repopulates
  seen paths).
- `dry_run: true` skips `persist()` after classification — same as original design.
- Per-node `ScanEntry::Skipped` accumulation matches PRD Section 7 (non-fatal
  I/O failures).
- Service depends only on `ScannerPort` and `Repository` traits — correct
  hexagonal boundary.
- All AC items test through the service's `run()` method with mock ports.

**Minor observation (not a blocker):**

- The `reindex: true` + `IndexScope::Partial` question is resolved by the new
  design: `repo.clear()` discards ALL state regardless of scope. The subsequent
  scan only discovers nodes within the Partial root, so state outside the
  partial root stays deleted until a Full scan restores it. This is technically
  correct per the PRD ("discard all persisted state") but could surprise users.
  Document this behavior in a code comment on the `reindex` path.
