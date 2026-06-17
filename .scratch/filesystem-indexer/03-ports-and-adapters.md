---
title: 03-indexer-ports-and-adapters
category: enhancement
label: ready-for-agent
status: closed
branch:
merge_commit:
date_created: 2026-06-09
date_completed: 2026-06-17
---

# Issue 03: ScannerPort, walkdir adapter, repository ports, and redb storage adapter

## What to build

Implement the full infrastructure boundary for the Indexer: the scanner port
and its concrete walkdir adapter, the repository port traits, and the redb
storage adapter. No application service logic yet — this issue proves that
the Indexer can read from the filesystem and read/write its own persistence
tables independently.

### Scanner

- Define `ScannerPort` trait in `lithos-core::indexer::port`. The trait
  returns a lazy iterator of `ScanEntry` values from a `DirPath` + `ScanFilters`
  — no batch `ScanResult` type. The iterator yields one entry at a time; the
  caller classifies each entry inline without a second pass.
- `ScanEntry` is an enum with three variants: `File(FileNode)`, `Dir(DirNode)`,
  `Skipped(SkippedEntry)`. Entries excluded by `ScanFilters` are
  silently dropped by walkdir's `filter_entry` and never appear in the stream.
  Entries that match filters but can't be read (permission denied, unsupported
  type) are yielded as `ScanEntry::Skipped` — never escalated as hard errors.
- Implement the walkdir adapter (`WalkdirAdapter`) as a zero-size struct (no
  `vault_root` field — the adapter receives the resolved `DirPath` per call via
  the port). It translates `ScanFilters` into walkdir `filter_entry` predicates,
  wraps the walkdir iterator, and maps entries to `ScanEntry` variants.
  The existing FS context `DirScanner` is **not** used here.
- Extend `ScanFilters` with utility methods (`is_included_extension`,
  `is_excluded_name`) that the adapter calls to build `filter_entry` predicates.
  Directories are traversed unless their name is excluded; files are returned
  only when they match the extension filter, if one is configured.

### Repository ports

Define in `lithos-core::indexer::repository`:

- `ReadRepository` — lookup by `FsRecordId`, lookup by `PathKey`, listing by
  format / parent / basename, loading all persisted paths for deletion
  detection.
- `WriteRepository` — save/delete single file or dir records; atomic
  multi-record save and prune via `save_many_records` / `delete_many_records`.
- `Repository: ReadRepository + WriteRepository`.

### redb storage adapter

Implement in the standard storage-adapter layout used throughout the codebase:

```text
lithos-core/src/indexer/storage/
├── mod.rs
├── read.rs
├── write.rs
├── tables.rs
└── testing.rs
```

Tables live in `storage::tables` and are all updated atomically in one
`redb::WriteTransaction`:

| Table                  | Key                   | Value             |
|------------------------|-----------------------|-------------------|
| `FILES`                | `FsRecordId`          | rkyv `FileRecord` |
| `DIRS`                 | `FsRecordId`          | rkyv `DirRecord`  |
| `FILE_ID_BY_PATH`      | `PathKey` string      | `FsRecordId`      |
| `DIR_ID_BY_PATH`       | `PathKey` string      | `FsRecordId`      |
| `FILE_IDS_BY_BASENAME` | `&str`                | `FsRecordId`      |
| `FILE_IDS_BY_PARENT`   | `FsRecordId` (parent) | `FsRecordId` (child) |
| `FILE_IDS_BY_FORMAT`   | `&str`                | `FsRecordId`      |

redb primitives stay inside the adapter. The public repository port exposes
`IndexerRepositoryError`, not redb types.

Single-record `save_file` / `save_dir` / `delete_file` / `delete_dir` each
open their own `WriteTransaction`. `save_many_records` and
`delete_many_records` combine all primary and secondary index writes into one
`WriteTransaction` — this is the only way to guarantee cross-record atomicity.

## Concrete signatures

### Error types (`lithos-core::indexer::error`)

Replace the current placeholder `IndexerError` entirely:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum IndexerError {
    #[error(transparent)]
    Scanner(#[from] ScannerError),
    #[error(transparent)]
    Repository(#[from] IndexerRepositoryError),
}

/// Fatal errors that prevent a scan from starting or completing.
/// Per-entry failures (permission denied, unsupported type) are NOT errors —
/// they are recorded in `ScanResult::skipped`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ScannerError {
    /// A walkdir entry or metadata read failed during traversal.
    #[error("traversal failed for {path}: {source}")]
    Traversal { path: PathBuf, source: std::io::Error },
}

/// Repository-layer errors surfaced through the port boundary.
/// redb and rkyv types never appear here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum IndexerRepositoryError {
    /// Transparent wrapper around the shared DbError (follows VaultRepositoryError pattern).
    #[error("storage error: {0}")]
    Storage(#[from] DbError),
    /// A PathKey write would create a duplicate entry.
    #[error("duplicate path: {0}")]
    DuplicatePath(PathKey),
}
```

### Scanner types (`lithos-core::indexer::port`)

```rust
/// A single item yielded by the scanner's lazy walk iterator.
///
/// Filtered entries are silently dropped by walkdir's `filter_entry` and never
/// appear in the stream. Entries that match filters but can't be read yield
/// the `Skipped` variant — the caller accumulates these into `IndexReport`.
#[derive(Debug)]
pub(crate) enum ScanEntry {
    File(fs::entry::FileNode),
    Dir(fs::entry::DirNode),
    Skipped(SkippedEntry),
}

/// An entry encountered during scanning that could not be indexed.
/// Has no `FsRecordId` because it was never persisted.
pub(crate) struct SkippedEntry {
    pub(crate) path: PathBuf,
    pub(crate) reason: SkipReason,
}

pub(crate) enum SkipReason {
    PermissionDenied,
    UnsupportedEntryType,
}

/// Interface for filesystem traversal.
///
/// Returns a lazy iterator so the caller can classify each entry inline,
/// avoiding a two-pass design (scan then classify). The `root` is always a
/// concrete `DirPath` resolved by the service layer — the adapter does not
/// know about vaults, `PathKey`, or `IndexScope`.
pub(crate) trait ScannerPort {
    fn walk<'s>(
        &'s self,
        root: &'s DirPath,
        filters: &'s ScanFilters,
    ) -> Result<Box<dyn Iterator<Item = Result<ScanEntry, ScannerError>> + 's>, ScannerError>;
}
```

### `IndexScope` change (`lithos-core::indexer::scan`)

`IndexScope` currently uses `PathKey` for the Partial variant root. This forces
a conversion circle: CLI path → `PathKey` (needs vault root) → `DirPath` (in
adapter). **Fix**: both variants carry a `DirPath` directly:

```rust
pub(crate) enum IndexScope {
    Full  { root: DirPath, filters: ScanFilters },
    Partial { root: DirPath, filters: ScanFilters },
}
```

The semantic difference between Full and Partial is deletion-detection scope
(Full checks all persisted paths; Partial checks only paths under `root`).
Both variants are concrete OS paths ready for the scanner — no roundtrip
through `PathKey`. The service constructs the appropriate variant from CLI
input (Full from vault root config, Partial by resolving user-provided path).

### `ScanFilters` addition (`lithos-core::indexer::scan`)

Replace the current empty placeholder with minimal concrete filters and add
utility methods for the adapter:

```rust
pub(crate) struct ScanFilters {
    /// File extensions to include, without a leading dot. Empty means all files.
    included_extensions: Vec<Box<str>>,
    /// Entry names to exclude from traversal or file results.
    excluded_names: Vec<Box<str>>,
}

impl ScanFilters {
    /// Returns true when `ext` matches an included extension (or no extension
    /// filter is configured).
    pub(crate) fn is_included_extension(&self, ext: &str) -> bool {
        self.included_extensions.is_empty()
            || self.included_extensions.iter().any(|e| e.as_ref() == ext)
    }

    /// Returns true when `name` matches an excluded entry name.
    pub(crate) fn is_excluded_name(&self, name: &str) -> bool {
        self.excluded_names.iter().any(|n| n.as_ref() == name)
    }
}
```

The walkdir adapter calls `filters.is_excluded_name()` in `filter_entry` and
`filters.is_included_extension()` for file entries — no inline iteration.

### `IndexReport` addition (`lithos-core::indexer::summary`)

Add one field to the existing `IndexReport`:

```rust
pub(crate) struct IndexReport {
    scanned: usize,
    new: usize,
    fresh: usize,
    stale: usize,
    deleted: usize,
    skipped: Box<[SkippedEntry]>,      // NEW — accumulated by service from ScanEntry::Skipped stream
    failures: Box<[IndexNodeFailure]>,
}
```

### Repository port traits (`lithos-core::indexer::repository`)

```rust
pub(crate) trait ReadRepository {
    fn find_file(&self, id: FsRecordId) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    fn find_dir(&self, id: FsRecordId) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    fn find_file_by_path(&self, path: &PathKey) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    fn find_dir_by_path(&self, path: &PathKey) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    fn list_files_by_parent(&self, parent_id: FsRecordId) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    fn list_dirs_by_parent(&self, parent_id: FsRecordId) -> Result<Box<[DirRecord]>, IndexerRepositoryError>;
    fn list_files_by_format(&self, format: FileFormat) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    fn list_files_by_basename(&self, basename: &str) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    /// Returns all persisted PathKeys; used by the application service for deletion detection.
    fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError>;
}

pub(crate) trait WriteRepository {
    fn save_file(&self, record: &FileRecord) -> Result<(), IndexerRepositoryError>;
    fn save_dir(&self, record: &DirRecord) -> Result<(), IndexerRepositoryError>;
    fn delete_file(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError>;
    fn delete_dir(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError>;
    /// Atomically persist files and dirs in one WriteTransaction.
    /// All primary records and all secondary indexes are written together or not at all.
    fn save_many_records(&self, files: &[FileRecord], dirs: &[DirRecord]) -> Result<(), IndexerRepositoryError>;
    /// Atomically prune file and dir records in one WriteTransaction.
    /// All primary records and all secondary indexes are removed together or not at all.
    fn delete_many_records(&self, file_ids: &[FsRecordId], dir_ids: &[FsRecordId]) -> Result<(), IndexerRepositoryError>;
}

pub(crate) trait Repository: ReadRepository + WriteRepository {}
```

## Acceptance criteria

- [ ] `ScannerPort` trait, `ScanEntry`, `SkippedEntry`, and `SkipReason` types
      are defined in `lithos-core::indexer::port`.
- [ ] `ScannerPort::walk` returns a lazy iterator (not a batch `ScanResult`).
- [ ] `WalkdirAdapter` is a zero-size struct (no `vault_root` field). It
      implements `ScannerPort`; per-entry errors (permission denied,
      unsupported type) are yielded as `ScanEntry::Skipped`, never `Err`.
- [ ] Scanner adapter tests prove `ScanFilters` translate into correct walkdir
      traversal without leaking walkdir types into domain contracts; test covers
      permission-denied subtree appearing as `ScanEntry::Skipped`, not as an error.
- [ ] `ScannerPort` is annotated for mockall-based tests so issue 04 can use a
      generated mock instead of a handwritten scanner double.
- [ ] `ScanFilters` includes minimal concrete filters for included file
      extensions and excluded entry names, with utility methods
      (`is_included_extension`, `is_excluded_name`) and tests proving both
      are applied.
- [ ] `IndexScope::Full` and `IndexScope::Partial` both carry `root: DirPath`
      (not `PathKey`). `Partial` resolution no longer roundtrips through
      `PathKey`.
- [ ] `ReadRepository`, `WriteRepository`, and `Repository` traits are defined
      with the exact signatures above.
- [ ] `RedbRepository` implements all three traits.
- [ ] All seven tables are defined; `save_many_records` and
      `delete_many_records` perform all writes in a single `WriteTransaction`
      (primary records and all secondary indexes commit atomically).
- [ ] Single-record methods (`save_file`, `save_dir`, `delete_file`,
      `delete_dir`) each open and commit their own `WriteTransaction`.
- [ ] Repository contract tests prove all secondary indexes (`path`, `parent`,
      `basename`, `format`) stay consistent after every write operation,
      including `save_many_records` and `delete_many_records`.
- [ ] `IndexerError`, `ScannerError`, and `IndexerRepositoryError` are defined
      with the exact variants above; existing placeholder variants are removed;
      per-entry unsupported types remain skipped diagnostics, not
      `ScannerError` variants.
- [ ] `IndexReport` gains a `skipped: Box<[SkippedEntry]>` field and an
      updated constructor.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Changlog

### 2026-06-17 — Session 5: Streaming scanner redesign (adversarial review)

**No commits yet** — design session.

Redesigned the scanner port from batch to streaming, informed by adversarial
review against the two-pass problem (scan then classify).

**Design changes**:
1. `ScannerPort::scan(&self, &IndexScope) -> Result<ScanResult, ScannerError>`
   → `ScannerPort::walk(&self, &DirPath, &ScanFilters) -> Result<Box<dyn
   Iterator<Item = Result<ScanEntry, ScannerError>>>, ScannerError>`.
2. `ScanResult` removed — replaced by `ScanEntry` enum (lazy stream).
3. `WalkdirAdapter` becomes zero-size (no `vault_root`). Receives `DirPath`
   per call.
4. `IndexScope::Partial { root: PathKey }` → `IndexScope::Partial { root:
   DirPath }`. Breaks the OS-path → PathKey → DirPath conversion circle.
5. `ScanFilters` gains utility methods `is_included_extension()` and
   `is_excluded_name()`.
6. `SkippedEntry` values now flow through `ScanEntry::Skipped` in the stream
   instead of being batched in `ScanResult::skipped`.

**Status**: Design agreed with stakeholder. Implementation deferred to issue 03
work session.

### 2026-06-17 — Session 6: Implementation (scanner port streaming redesign)

**Commits**: (pending commit after review)

Implemented the scanner streaming redesign across 4 files. All 2030 tests pass.

**Changes per TDD cycle**:
- **Cycle 2**: `ScanEntry` enum with `File(FileNode)`, `Dir(DirNode)`,
  `Skipped(SkippedEntry)` variants; removed batch `ScanResult` type.
- **Cycle 3**: `ScannerPort` trait with `walk()` returning
  `Box<dyn Iterator<Item = Result<ScanEntry, ScannerError>>>` (`WalkIter`
  type alias for clippy hygiene). Mock via `mockall::mock!` macro (not
  `#[automock]`) following the `discovery/port.rs` pattern — needed because
  mockall can't handle lifetime-parameterized return types.
- **Cycle 4/5**: `WalkdirAdapter` zero-size (no `vault_root`, no constructor).
  `walk()` clones `ScanFilters` + root `PathBuf` into `move` closures so the
  returned iterator is `'static`. `filter_entry` uses `ScanFilters` utility
  methods.
- **Cycle 6**: `ScanFilters::is_included_extension()` and
  `is_excluded_name()` added. `IndexScope::Partial { root: DirPath }` — both
  variants carry `DirPath`, breaking the PathKey conversion circle.
- **Cycle 7**: Permission-denied and unsupported-entry-type errors yield
  `ScanEntry::Skipped` (never `Err`). `try_map_entry` returns `Option` to
  silently skip the root directory itself.
- **Cycle 22**: `mod.rs` exports `ScanEntry` instead of `ScanResult`.

**Key decisions**:
- `'static` iterator return (not borrowed). Adapter clones `DirPath` and
  `ScanFilters` into `move` closures. Cheap — `ScanFilters` is small
  `Vec<Box<str>>`, `DirPath` is a `PathBuf`. Avoids mockall lifetime issues.
- `Box<dyn Iterator>` with `WalkIter` type alias satisfies clippy's
  `type_complexity` lint.
- `filter_entry` + `filter_map` pattern: `filter_entry` drops excluded
  entries before the walkdir inner loop; `filter_map` converts remaining
  entries while skipping the root dir via `None`.
- `scanner/mod.rs` unchanged — already `pub(crate) mod scanner;` with
  `mod walkdir;`.
- `report.rs` unchanged — `SkippedEntry`, `SkipReason`, `IndexReport::skipped`
  were already correct from session 4.

**GitNexus impact**: LOW risk (0 affected processes, 33 symbols changed across
4 files, all `pub(crate)` within indexer, zero external consumers).

### 2026-06-15 — Session 4: Adversarial review — hexagonal boundary enforcement

**Commits**: (pending)

Fixed 7 architectural violations identified during adversarial review against
`docs/refs/rust/guides/hexagonal_architecture.md`:

1. **Port defined inside adapter module**: Moved `ScannerPort`, `ScanResult`,
   `SkippedEntry`, `SkipReason` from `scanner/` into new `indexer/port.rs`.
   ScannerError went to `indexer/error.rs` alongside other error types.
   `scanner/mod.rs` now purely declares the adapter submodule.

2. **Walkdir dependency not encapsulated**: `mod walkdir` made private (was
   `pub(crate)`). `pub mod scanner` changed to `pub(crate) mod scanner`.
   `WalkdirAdapter` no longer re-exported from `indexer/mod.rs`. No walkdir
   types visible outside the adapter.

3. **`ScannerError::Unknown(String)` removed**: dead code — all traversal
   errors have a concrete `Traversal { path, source }` path.

4. **`SkipReason::Unknown(String)` removed**: dead code — all skipped entries
   are `PermissionDenied` or `UnsupportedEntryType`.

5. **`#[allow(clippy::excessive_nesting)]` removed**: extracted
   `WalkdirAdapter::handle_entry` to flatten the walkdir loop body.

6. **`ScanFilters` type mismatch**: changed `Vec<String>` to `Vec<Box<str>>`
   on both `included_extensions` and `excluded_names` fields, matching the
   issue spec.

7. **Module doc comments**: added `//!` doc to `repository.rs`, `port.rs`,
   `report.rs`; updated `scanner/mod.rs` and `walkdir.rs` docs.

Structural changes:
- `report.rs` created: `IndexReport`, `IndexNodeFailure`, `SkippedEntry`,
  `SkipReason` — report/skipped types from `summary.rs` + `port.rs`
- `port.rs` shrinks to just `ScannerPort` + `ScanResult` (the pure port contract)
- `summary.rs` trimmed to `IndexResult`, `IndexedNodes`, `DeletedNodes`

### 2026-06-15 — Session 3: Serialization before write transaction (perf)

**Commits**: `ea4646bd`

Moved `rkyv::to_bytes` out of the redb write lock on all save paths. Serialization
is CPU-bound allocation; holding it inside `store.write()` blocks concurrent readers.

- `save_file_in_tx` / `save_dir_in_tx` accept pre-serialized `&[u8]` instead of
  calling `rkyv::to_bytes` internally.
- `save_file` / `save_dir` serialize before `store.write()`, then pass bytes in.
- `save_many_records` serializes all files+dirs upfront into `Vec<AlignedVec>`,
  then batch-writes in one transaction.
- Deserialization (load-delete-context for upsert cleanup) stays inside the
  transaction — rkyv `from_bytes` is zero-copy pointer validation, negligible cost.

**Key decisions**:
- `_in_tx` helpers still receive the `&FileRecord` / `&DirRecord` reference for
  secondary index field access (path, name, format, parent_id). Only the primary
  table insert uses pre-serialized bytes.
- Clippy lint: `.map(|f| to_bytes(f))` → `.map(to_bytes)` (redundant closure).
- No new tests needed — behavior is identical, existing 1933 test suite passes.

### 2026-06-15 — Session 2: Remove `impl_rkyv_redb_value!` macro (refactor)

**Commits**: `847640f4`

Removed the `impl_rkyv_redb_value!` macro and switched from entity-typed tables
to raw `&[u8]` storage, matching all 5 other contexts (vault, template, schema,
config, note).

- `tables.rs`: Removed `impl_rkyv_redb_value!` macro and its `FileRecord`/`DirRecord`
  invocations. Changed `FILES`/`DIRS` from `UuidTable<FsRecordId, FileRecord>` to
  `UuidTable<FsRecordId, &[u8]>`.
- `read.rs`: Updated all 8 read methods to deserialize via `rkyv::from_bytes` with
  `DbError::Deserialization`. Added `deserialize_file`/`deserialize_dir` helpers.
  Updated 12 test insert sites.
- `write.rs`: Updated load/delete/save helpers to serialize via `rkyv::to_bytes`
  with `DbError::Serialization`.
- `ArchivedEntity` trait not used — raw `rkyv::from_bytes::<T, rkyv::rancor::Error>`
  and `rkyv::to_bytes::<rkyv::rancor::Error>` called directly at each read/write site.
- Helper functions return `Result<_, DbError>` (not panic) so the `store.read()`
  closure can use `?` to propagate errors.
- Zero-copy via `rkyv::access` deferred — available locally in hot paths but not
  wired into read methods since `ReadRepository` trait contract returns owned values.
- `impl_redb_uuid!(FsRecordId)` kept unchanged — still needed for table keys.

### 2026-06-09 — Session 1: Ports and adapters refinement

**Commits**: `fb34fe7c`

Replaced raw table definitions with typed wrappers (`UuidTable`, `PathUuidTable`,
`UuidMultimap`), removed `PathKey` TODO, added `DIR_IDS_BY_PARENT` multimap.

---

## Status

**Label**: `ready-for-agent` (awaiting issue 04 — scanner and adapters
complete).

**Completed (repository ports + storage — unchanged by redesign)**:
- [x] `ReadRepository`, `WriteRepository`, `Repository` traits defined
- [x] `RedbRepository` implements all three traits
- [x] All 7 tables defined with typed wrappers
- [x] `save_many_records` / `delete_many_records` atomic batch operations
- [x] Single-record `save_file`/`save_dir`/`delete_file`/`delete_dir`
- [x] Upsert cleanup (load old → remove stale indexes → insert new)
- [x] Index consistency after all write operations (path, parent, basename, format)
- [x] In-memory test double (`InMemoryRepository`)
- [x] `IndexerRepositoryError` defined
- [x] Raw `&[u8]` storage via `rkyv::to_bytes`/`rkyv::from_bytes`
- [x] Serialization outside write transaction (perf)

**Needs redesign (scanner port — Session 5 redesign)**: COMPLETED by Session 6
- [x] `ScannerPort` — switch from batch `scan(&self, &IndexScope)` to
      streaming `walk(&self, &DirPath, &ScanFilters)` yielding `ScanEntry`
- [x] `WalkdirAdapter` — remove `vault_root` field, make zero-size
- [x] `ScanEntry` enum — replace `ScanResult` batch type
- [x] `ScanFilters::is_included_extension()` / `is_excluded_name()` — utility methods
- [x] `IndexScope` — change `Partial { root: PathKey }` to `Partial { root: DirPath }`
- [x] `IndexReport::skipped` — accumulated by service, not by adapter batch
- [x] `ScannerError` — defined; `ScanResult` removed from module exports

---

## Blocked by

- 02-domain-model.md

---

## Triage Notes

> *This was generated by AI during triage.*

**Verdict**: `ready-for-agent` — confirmed after extended grilling session.

**What was checked (initial pass):**

- The three sub-deliverables (ScannerPort + walkdir adapter, repo port traits,
  redb adapter) are tightly coupled by type: the adapter tests need the repo
  ports, and issue 04's mockall-based service tests need the scanner trait.
  Splitting would shift dependency complexity, not remove it.
- `ScannerPort` ownership is in the Indexer context, not re-using `DirScanner`.
  GitNexus confirms `DirScanner` has zero external callers — no blast radius.
- All 7 tables match PRD Section 10b. `PATH_BY_FILE_ID` / `PATH_BY_DIR_ID`
  dropped per PRD ("primary records carry enough data for deletion detection").
- Atomic writes across all 7 tables in a single `WriteTransaction` — enforced
  by `save_many_records` / `delete_many_records` signatures.
- `ScannerPort` must be mockable with mockall for issue 04 application-service
  tests; a handwritten `MockScanner` is not required.
- redb primitives stay inside the adapter; ports expose `IndexerRepositoryError`.
- Blocker chain (issue-02) is correct.
- GitNexus impact on vault module: **LOW risk** — zero cross-imports; issue 03
  creates parallel tables without touching vault.

**Design decisions resolved during grilling:**

1. **`PermissionDenied` is a diagnostic, not an error.** Per-entry traversal
   failures (permission denied, unsupported entry type) do not abort the scan.
   They are collected in `ScanResult::skipped` and ultimately surfaced in
   `IndexReport::skipped`. Only fatal conditions that prevent the scan from
   starting or completing are `ScannerError` variants.

2. **`ScanResult` not `ScanBatch`.** Named to match `IndexResult`. Returns
   `fs::entry::FileNode` / `fs::entry::DirNode` directly (FS context types) —
   no redundant wrapper types (`ScannedFile` / `ScannedDir` were duplicates of
   existing FS types). Because `ScanResult` is internal to the Indexer module
   and built incrementally during traversal, its collections are `Vec<T>` rather
   than `Box<[T]>`.

3. **`ScanResult` is not `IndexResult`.** `IndexResult` contains Indexer domain
   types (`FileRecord`, `DirRecord`, `FsRecordId`, `IndexStatus`) computed by
   the application service after comparing against the repository. `ScanResult`
   contains raw FS context types from the walkdir traversal. These are distinct
   layers.

4. **`IndexerRepositoryError` is transparent to `DbError`**, following the
   `VaultRepositoryError::Storage(#[from] DbError)` pattern already established
   in this codebase. A redundant `StorageError` layer was rejected — `DbError`
   already wraps all redb error variants and provides `DbErrorKind` for stable
   classification without backend coupling.

5. **`find_*` returns `Option`, not direct `T`.** Per naming taxonomy:
   `find_*` is for optional lookup; `get_*` for required singletons.
   `FileNotFound` / `DirNotFound` error variants were dropped — unused at
   the port boundary when all ID lookups return `Option`.

6. **`save_many_records` / `delete_many_records` naming.** The `_records`
   suffix distinguishes these from single-type bulk operations. Both take
   mixed file+dir slices in one call; the suffix signals the atomicity
   contract spans both record types. Per naming taxonomy: `batch_*` prefix
   is not used in this codebase.

7. **Visibility stays `pub(crate)` throughout.** All consumers of these ports
   live inside `lithos-core`. No cross-crate visibility is needed; promoting
   to `pub` was rejected as premature.

8. **Storage adapter follows the standard module layout.** The redb adapter is
   implemented as `storage::{mod, read, write, tables, testing}` to match the
   existing codebase pattern used by context storage adapters.

9. **`ScanFilters` gets minimal real behavior now.** Extension inclusion and
   entry-name exclusion are enough to prove translation into walkdir traversal
   predicates without designing the full future filtering language.

10. **Mocking uses mockall, not a handwritten scanner.** `ScannerPort` should be
    annotated/configured so issue 04 can generate a mock scanner for service
    tests.

## TDD Implementation Plan

This plan follows vertical red-green-refactor slices. Do not write all tests
first. Each cycle adds one behavior, implements the minimum code to pass, then
refactors only after green.

### Architecture targets

- Hexagonal boundary: `ScannerPort`, `ReadRepository`, `WriteRepository`, and
  `Repository` are owned by `indexer`; walkdir and redb remain adapter details.
- Storage layout: `lithos-core/src/indexer/storage/{mod.rs,read.rs,write.rs,tables.rs,testing.rs}`.
- Streaming scanner design: `ScannerPort::walk` returns a lazy iterator of
  `ScanEntry` values. No batch `ScanResult` type — the service classifies each
  entry inline. `SkippedEntry` values are yielded as `ScanEntry::Skipped` and
  accumulated by the service into `IndexReport`.
- Stable report boundary: `IndexReport::skipped` uses `Box<[SkippedEntry]>`,
  matching existing report/result collection conventions.
- Test naming follows `docs/engineering/testing/unit.md` and
  `docs/engineering/testing/unit-naming.md`: Structure A modules, verb-first
  behavior names, one concern per test.

### Cycle 1 — Error boundary tracer bullet

- RED: `error::conversions::converts_scanner_error_to_indexer_error`.
- GREEN: replace placeholder `IndexerError::{Internal, Io}` with
  `IndexerError::{Scanner, Repository}`, add `ScannerError` and
  `IndexerRepositoryError`.
- Verify `DbError` wraps through `IndexerRepositoryError::Storage` and Display
  output remains actionable.
- Keep per-entry unsupported types out of `ScannerError`; they are skipped
  diagnostics.

### Cycle 2 — Stream entry model

- RED: `scanner::walk_entry::yields_file_dir_and_skipped_variants`.
- GREEN: add `ScanEntry`, `SkippedEntry`, and `SkipReason`.
- Verify direct FS context types are used: `fs::entry::FileNode` and
  `fs::entry::DirNode`. No batch `ScanResult` type.

### Cycle 3 — Scanner port and mockability

- RED: `scanner::contracts::scanner_port_can_be_mocked`.
- GREEN: define `ScannerPort` and add mockall support for test builds.
- Verify issue 04 can generate a scanner mock without a handwritten
  `MockScanner` type.

### Cycle 4 — Walkdir adapter happy path

- RED: `scanner::walkdir_adapter::returns_file_and_dir_nodes_for_full_scope`.
- GREEN: add zero-size `WalkdirAdapter` and implement `ScannerPort::walk`.
  Adapter receives `DirPath` per call — no `vault_root` field.
- Verify no walkdir type appears in scanner port contracts.

### Cycle 5 — Walkdir adapter test with different roots

- RED: `scanner::walkdir_adapter::scans_different_root_on_each_call`.
- GREEN: adapter accepts any `DirPath` per call — no vault coupling.
- Verify files outside the specified root are not returned.

### Cycle 6 — Scan filter translation

- RED: `scanner::filter::excludes_files_when_extension_does_not_match`.
- GREEN: add `ScanFilters::is_included_extension()` and apply it in the
  adapter's `filter_entry`.
- RED: `scanner::filter::excludes_entries_when_name_is_excluded`.
- GREEN: add `ScanFilters::is_excluded_name()` and apply it in the adapter's
  `filter_entry`.
- Verify directories are traversed unless excluded by name, while files are
  returned only when extension filters allow them.

### Cycle 7 — Skipped diagnostics in stream

- RED: `scanner::walk::yields_permission_denied_as_skipped_in_stream`.
- GREEN: map permission/read failures to `ScanEntry::Skipped` — never `Err`.
- RED: `scanner::walk::yields_unsupported_entry_type_as_skipped_in_stream`.
- GREEN: map unsupported file types to `ScanEntry::Skipped`.

### Cycle 8 — Repository port contract

- RED: `repository::contracts::blanket_repository_impl_accepts_read_write_type`.
- GREEN: add `ReadRepository`, `WriteRepository`, `Repository`, and the blanket
  `impl<T> Repository for T where T: ReadRepository + WriteRepository`.
- Verify signatures match the issue and use `find_* -> Result<Option<T>, E>`.

### Cycle 9 — Storage table definitions

- RED: `storage::tables::opens_all_indexer_tables`.
- GREEN: add all seven table definitions in `storage/tables.rs` and implement
  redb key support for `FsRecordId`.
- Verify table constants stay `pub(crate)` and redb types do not leave storage.

### Cycle 10 — RedbRepository construction

- RED: `storage::constructor::stores_shared_store_handle`.
- GREEN: add `RedbRepository` in `storage/mod.rs` wrapping `Arc<Store>`.
- Keep constructor and adapter visibility `pub(crate)`.

### Cycle 11 — Read repository lookups

- RED/GREEN one behavior at a time:
  - `lookup::find_file_returns_none_when_missing`.
  - `lookup::find_file_returns_record_when_present`.
  - `lookup::find_dir_returns_none_when_missing`.
  - `lookup::find_dir_returns_record_when_present`.
  - `lookup::find_file_by_path_returns_record_when_path_exists`.
  - `lookup::find_dir_by_path_returns_record_when_path_exists`.
- Seed through repository writes where possible; use direct table seeding only
  to isolate read behavior before writes exist.

### Cycle 12 — Read repository index queries

- RED/GREEN one behavior at a time:
  - `list::returns_files_for_parent`.
  - `list::returns_dirs_for_parent`.
  - `filter::returns_files_for_format`.
  - `lookup::returns_files_for_basename`.
  - `list::returns_all_paths`.
- `list_dirs_by_parent` uses the primary `DIRS` table and filters by
  `DirRecord::parent_id()` unless an eighth table is explicitly approved later.

### Cycle 13 — Single-record file writes

- RED: `create::save_file_persists_primary_and_indexes`.
- GREEN: implement `save_file`.
- Assert through `ReadRepository`: ID lookup, path lookup, basename lookup,
  parent lookup, and format lookup.

### Cycle 14 — Single-record directory writes

- RED: `create::save_dir_persists_primary_and_path_index`.
- GREEN: implement `save_dir`.
- Assert through `find_dir`, `find_dir_by_path`, and `list_dirs_by_parent`.

### Cycle 15 — File delete behavior

- RED: `delete::delete_file_removes_primary_and_indexes`.
- GREEN: implement `delete_file`.
- Because `PATH_BY_FILE_ID` is intentionally not part of the seven-table design,
  load the primary `FileRecord` before removal and derive path, basename,
  parent, and format from it to clean secondary indexes.
- RED: `delete::delete_file_is_idempotent_when_missing`.
- GREEN: missing IDs remain `Ok(())`.

### Cycle 16 — Directory delete behavior

- RED: `delete::delete_dir_removes_primary_and_path_index`.
- GREEN: implement `delete_dir` by loading the primary `DirRecord` before
  removal to recover its `PathKey`.
- RED: `delete::delete_dir_is_idempotent_when_missing`.
- GREEN: missing IDs remain `Ok(())`.

### Cycle 17 — Upsert cleanup

- RED: `update::save_file_cleans_stale_indexes_when_record_changes`.
- GREEN: remove the existing file graph by ID before inserting replacement
  primary and secondary indexes.
- RED: `update::save_dir_cleans_stale_path_index_when_path_changes`.
- GREEN: same pattern for directories.

### Cycle 18 — Atomic mixed batch save

- RED: `transactions::save_many_records_persists_files_and_dirs_together`.
- GREEN: implement `save_many_records(&[FileRecord], &[DirRecord])` with one
  `Store::write` transaction.
- Verify all primary records and secondary indexes are visible after commit.

### Cycle 19 — Atomic mixed batch delete

- RED: `transactions::delete_many_records_removes_files_and_dirs_together`.
- GREEN: implement `delete_many_records(&[FsRecordId], &[FsRecordId])` with one
  `Store::write` transaction.
- Verify all primary records and secondary indexes are removed for both kinds.

### Cycle 20 — Storage testing support

- RED: `storage::testing::repository_double_supports_save_and_find`.
- GREEN: add a test-only in-memory repository in `storage/testing.rs` only if
  issue 04 needs repository behavior that mockall cannot express ergonomically.
- Prefer mockall-generated mocks for ports; do not add handwritten doubles by
  default.

### Cycle 21 — IndexReport skipped field (stream-aware)

- RED: `summary::report::stores_skipped_entries`.
- GREEN: add `skipped: Box<[SkippedEntry]>` and `skipped(&self) ->
  &[SkippedEntry]` to `IndexReport`. Skipped entries are accumulated from
  `ScanEntry::Skipped` stream values, not from a batch `ScanResult`.
- Keep existing count accessors intact.

### Cycle 22 — Module wiring

- RED: compile tests importing `crate::indexer::{ScannerPort, ScanEntry,
  ReadRepository, WriteRepository, Repository, RedbRepository}`.
- GREEN: wire `scanner`, `repository`, and `storage` in `indexer/mod.rs` with
  `pub(crate)` exports only. `ScanEntry` exported from `port.rs`.

### Cycle 23 — Refactor after green [DONE]

- Extract private storage helpers only after tests pass:
  `save_file_in_tx`, `remove_file_graph`, `load_file_delete_context`,
  `save_dir_in_tx`, and `remove_dir_graph`.
- Do not prematurely genericize file and directory storage helpers; their index
  sets differ.
- Run tests after each refactor step.

### Cycle 24 — Serialization before write transaction [DONE]

Moved `rkyv::to_bytes` outside `store.write()` on all save paths. CPU-bound
encoding no longer holds the redb write lock that blocks concurrent readers.

- `save_file_in_tx` / `save_dir_in_tx` accept `&[u8]` param type.
- `save_many_records` serializes all records upfront into `Vec<AlignedVec>`.
- The `record` ref is still passed for secondary-index field access.
- Deserialization stays inside transactions (zero-copy, negligible).

### Cycle 25 — Verification gates

- Run targeted module tests during each cycle.
- Before completion: `mise run fmt`, `mise run test`, and `mise run lint`.
- Before any commit: run GitNexus change detection to verify affected scope.
