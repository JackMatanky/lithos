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

### Repository ports

Define in `lithos-core::indexer::repository`:

- `ReadRepository` — lookup by `FsRecordId`, lookup by `PathKey`, listing by
  format / parent / basename, loading all persisted paths for deletion
  detection.
- `WriteRepository` — save/delete single file or dir records; atomic
  multi-record save and prune via `save_many_records` / `delete_many_records`.
- `Repository: ReadRepository + WriteRepository`.

### redb storage adapter

Implement in `lithos-core::indexer::storage`. Tables (all updated atomically
in one `redb::WriteTransaction`):

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
    /// Filesystem entry is neither file nor directory (socket, fifo, etc.).
    #[error("unsupported entry type: {0}")]
    UnsupportedEntryType(PathBuf),
}

/// Repository-layer errors surfaced through the port boundary.
/// redb and rkyv types never appear here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum IndexerRepositoryError {
    /// Transparent wrapper around the shared DbError (follows VaultRepositoryError pattern).
    #[error("storage error: {0}")]
    Storage(#[from] DbError),
    /// A PathKey lookup found no matching record.
    #[error("path not found: {0}")]
    PathNotFound(PathKey),
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
    pub(crate) files: Box<[fs::entry::FileNode]>,
    pub(crate) dirs: Box<[fs::entry::DirNode]>,
    pub(crate) skipped: Box<[SkippedEntry]>,
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
    failures: Box<[IndexRecordFailure]>,
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
- [ ] An in-memory `MockScanner` implementing `ScannerPort` is provided for use
      by issue 04's application-service tests.
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
      with the exact variants above; existing placeholder variants are removed.
- [ ] `IndexReport` gains a `skipped: Box<[SkippedEntry]>` field and an
      updated constructor.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- 02-domain-model.md

---

## Triage Notes

> *This was generated by AI during triage.*

**Verdict**: `ready-for-agent` — confirmed after extended grilling session.

**What was checked (initial pass):**

- The three sub-deliverables (ScannerPort + walkdir adapter, repo port traits,
  redb adapter) are tightly coupled by type: the adapter tests need the repo
  ports, the test double (for issue 04) needs the scanner trait. Splitting
  would shift dependency complexity, not remove it.
- `ScannerPort` ownership is in the Indexer context, not re-using `DirScanner`.
  GitNexus confirms `DirScanner` has zero external callers — no blast radius.
- All 7 tables match PRD Section 10b. `PATH_BY_FILE_ID` / `PATH_BY_DIR_ID`
  dropped per PRD ("primary records carry enough data for deletion detection").
- Atomic writes across all 7 tables in a single `WriteTransaction` — enforced
  by `save_many_records` / `delete_many_records` signatures.
- In-memory `MockScanner` test double is an AC, required for issue 04.
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
   existing FS types).

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
