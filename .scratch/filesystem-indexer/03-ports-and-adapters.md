---
title: 03-indexer-ports-and-adapters
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-09
date_completed:
---

# Issue 03: ScannerPort, walkdir adapter, repository ports, and redb storage adapter

## What to build

Implement the full infrastructure boundary for the Indexer: the scanner port
and its concrete walkdir adapter, the repository port traits, and the redb
storage adapter. No application service logic yet — this issue proves that
the Indexer can read from the filesystem and read/write its own persistence
tables independently.

### Scanner

- Define `ScannerPort` trait in `lithos-core::indexer::scanner`. The trait
  returns a `ScanResult` (discovered FS nodes + skipped diagnostics) from an
  `IndexScope`.
- Implement the walkdir adapter (`WalkdirAdapter`) as the sole concrete
  `ScannerPort` implementation. It holds a `vault_root: DirPath`, translates
  `ScanFilters` into walkdir `filter_entry` predicates, walks the subtree, and
  produces a `ScanResult`. Per-entry errors (permission denied, unsupported
  type) are recorded in `ScanResult::skipped` — never escalated as hard errors.
  The existing FS context `DirScanner` is **not** used here.
- Extend `ScanFilters` with the minimal concrete criteria needed for the
  walkdir adapter to prove real predicate translation: included file
  extensions and excluded entry names. Directories are traversed unless their
  name is excluded; files are returned only when they match the extension
  filter, if one is configured.

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

### Scanner types (`lithos-core::indexer::scanner`)

```rust
/// Raw filesystem nodes discovered during a scan, before ID assignment or
/// index-status comparison. Uses `fs::entry::FileNode` / `fs::entry::DirNode`
/// directly — no redundant wrapper types.
pub(crate) struct ScanResult {
    pub(crate) files: Vec<fs::entry::FileNode>,
    pub(crate) dirs: Vec<fs::entry::DirNode>,
    pub(crate) skipped: Vec<SkippedEntry>,
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

pub(crate) trait ScannerPort {
    /// Walk the subtree defined by `scope`. Returns discovered FS nodes and a
    /// diagnostic report. Permission-denied and unreadable entries are
    /// collected in `ScanResult::skipped` — never propagated as `Err`.
    fn scan(&self, scope: &IndexScope) -> Result<ScanResult, ScannerError>;
}
```

### `ScanFilters` addition (`lithos-core::indexer::scan`)

Replace the current empty placeholder with minimal concrete filters:

```rust
pub(crate) struct ScanFilters {
    /// File extensions to include, without a leading dot. Empty means all files.
    included_extensions: Vec<Box<str>>,
    /// Entry names to exclude from traversal or file results.
    excluded_names: Vec<Box<str>>,
}
```

The walkdir adapter must translate these into traversal behavior without
leaking walkdir types into `ScanFilters` or `ScannerPort`.

### `IndexReport` addition (`lithos-core::indexer::summary`)

Add one field to the existing `IndexReport`:

```rust
pub(crate) struct IndexReport {
    scanned: usize,
    new: usize,
    fresh: usize,
    stale: usize,
    deleted: usize,
    skipped: Box<[SkippedEntry]>,      // NEW — populated from ScanResult::skipped
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

- [ ] `ScannerPort` trait and `ScanResult` / `SkippedEntry` / `SkipReason` types
      are defined in `lithos-core::indexer::scanner`.
- [ ] `WalkdirAdapter` implements `ScannerPort`; per-entry errors (permission
      denied, unsupported type) go to `ScanResult::skipped`, never `Err`.
- [ ] Scanner adapter tests prove `ScanFilters` translate into correct walkdir
      traversal without leaking walkdir types into domain contracts; test covers
      permission-denied subtree appearing in `skipped`, not as an error.
- [ ] `ScannerPort` is annotated for mockall-based tests so issue 04 can use a
      generated mock instead of a handwritten scanner double.
- [ ] `ScanFilters` includes minimal concrete filters for included file
      extensions and excluded entry names, with tests proving both are applied.
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

**Completed**:
- [x] `ScannerPort` trait and `ScanResult`/`SkippedEntry`/`SkipReason` types
- [x] `WalkdirAdapter` implementing `ScannerPort`
- [x] `ScanFilters` with extension inclusion and name exclusion (`Vec<Box<str>>`)
- [x] `IndexReport::skipped` field
- [x] `ReadRepository`, `WriteRepository`, `Repository` traits defined
- [x] `RedbRepository` implements all three traits
- [x] All 7 tables defined with typed wrappers
- [x] `save_many_records` / `delete_many_records` atomic batch operations
- [x] Single-record `save_file`/`save_dir`/`delete_file`/`delete_dir`
- [x] Upsert cleanup (load old → remove stale indexes → insert new)
- [x] Index consistency after all write operations (path, parent, basename, format)
- [x] In-memory test double (`InMemoryRepository`)
- [x] `IndexerError`, `ScannerError`, `IndexerRepositoryError` defined
- [x] Raw `&[u8]` storage via `rkyv::to_bytes`/`rkyv::from_bytes`
- [x] Serialization outside write transaction (perf)
- [x] Hexagonal boundary enforcement (port in `port.rs`, adapter in `scanner/`,
      walkdir encapsulated, Unknown variants removed, nesting flattened)

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
- Runtime scanner accumulation: `ScanResult` uses `Vec<FileNode>`,
  `Vec<DirNode>`, and `Vec<SkippedEntry>` because it is internal and built by
  pushing during traversal.
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

### Cycle 2 — Scan result model

- RED: `scanner::scan_result::stores_files_dirs_and_skipped_entries`.
- GREEN: add `ScanResult`, `SkippedEntry`, and `SkipReason` with `Vec<T>`
  collections.
- Verify direct FS context types are used: `fs::entry::FileNode` and
  `fs::entry::DirNode`.

### Cycle 3 — Scanner port and mockability

- RED: `scanner::contracts::scanner_port_can_be_mocked`.
- GREEN: define `ScannerPort` and add mockall support for test builds.
- Verify issue 04 can generate a scanner mock without a handwritten
  `MockScanner` type.

### Cycle 4 — Walkdir adapter happy path

- RED: `scanner::walkdir_adapter::returns_file_and_dir_nodes_for_full_scope`.
- GREEN: add `WalkdirAdapter { vault_root: DirPath }` and implement
  `ScannerPort::scan` for `IndexScope::Full`.
- Verify no walkdir type appears in scanner port contracts.

### Cycle 5 — Partial scope traversal

- RED: `scanner::walkdir_adapter::scans_only_partial_scope_root`.
- GREEN: map `IndexScope::Partial { root, filters }` to a subtree under
  `vault_root`.
- Verify files outside the partial root are not returned.

### Cycle 6 — Scan filter translation

- RED: `scanner::filter::excludes_files_when_extension_does_not_match`.
- GREEN: add `ScanFilters::included_extensions` behavior and apply it during
  traversal.
- RED: `scanner::filter::excludes_entries_when_name_is_excluded`.
- GREEN: add `ScanFilters::excluded_names` behavior and translate it into
  walkdir `filter_entry` traversal exclusion.
- Verify directories are traversed unless excluded by name, while files are
  returned only when extension filters allow them.

### Cycle 7 — Skipped diagnostics

- RED: `scanner::skipped::records_permission_denied_entry_as_skipped`.
- GREEN: map permission/read failures to `SkippedEntry { reason:
  PermissionDenied }` without returning `Err`.
- RED: `scanner::skipped::records_unsupported_entry_type_as_skipped`.
- GREEN: map unsupported file types to `SkipReason::UnsupportedEntryType`.

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

### Cycle 21 — IndexReport skipped field

- RED: `summary::report::stores_skipped_entries`.
- GREEN: add `skipped: Box<[SkippedEntry]>` and `skipped(&self) ->
  &[SkippedEntry]` to `IndexReport`.
- Keep existing count accessors intact.

### Cycle 22 — Module wiring

- RED: compile tests importing `crate::indexer::{ScannerPort, ReadRepository,
  WriteRepository, Repository, RedbRepository}` from internal tests.
- GREEN: wire `scanner`, `repository`, and `storage` in `indexer/mod.rs` with
  `pub(crate)` exports only.

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
