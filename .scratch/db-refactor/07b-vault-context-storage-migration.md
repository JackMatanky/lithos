---
title: 07b-vault-context-storage-migration
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-25
---

## Type

Implementation

## Labels

- needs-triage
- part-of-07
- blocks-07c

## What to build

Migrate the entire Vault context to the updated db-refactor standards established in Issues 01-06 and applied in Issue 07a for Note context.

The Vault context currently uses legacy `&'db Database` patterns. This issue brings it up to the same standard as Note and Schema contexts: segregated repository traits, `Arc<Store>` transaction management, typed table wrappers, and in-memory test doubles with `InMemoryHarness`.

## Motivation

The Vault processor creates both a vault repository and a note repository. Currently:
- Vault repository: `VaultRepository::new(db: &'db Database)` — legacy pattern
- Note repository: Needs `Arc<Store>` (from 07a)

Without migrating Vault, we can't provide a clean bridge for the note repository. Both should use `Store`.

## Scope

Full vault context storage layer migration:

1. **Repository traits** (`vault/repository.rs`):
   - `ReadRepository` trait (find file/dir views, list operations)
   - `WriteRepository` trait (save/delete file/dir views)
   - `Repository` marker trait (blanket impl)

2. **Storage implementation** (`vault/storage/`):
   - `mod.rs` — `RedbRepository` struct with `Arc<Store>`
   - `tables.rs` — typed table definitions (`UuidTable`, `PathTable`)
   - `read.rs` — `ReadRepository` impl
   - `write.rs` — `WriteRepository` impl
   - `testing.rs` — `InMemoryRepository` with `InMemoryHarness`

3. **Error updates** (`vault/error.rs`):
   - Reduce to persistence-boundary variants (following Note pattern)
   - Add `From<InMemoryDbError>` impl

4. **Module cleanup**:
   - Rename current `vault/storage.rs` → `storage_legacy.rs`
   - New `storage/` submodule replaces it
   - Update `vault/mod.rs` exports

5. **Processor updates** (`vault/processor.rs`):
   - Use new `VaultRepository` with `Arc<Store>`
   - Provide bridge method: `Database::to_store() -> Store` or `From<&Database> for Store`

## Current State

File: `lithos-core/src/vault/storage.rs` (~1300 lines)

**Structure**:
- Monolithic `Repository` trait with read + write operations
- `RedbRepository<'db>` with `db: &'db Database`
- Tables: `FILES_BY_ID`, `FILE_ID_BY_PATH`, `DIRS_BY_ID`, `DIR_ID_BY_PATH`
- Mix of direct DB access and batch operations

**Needs**:
- Trait segregation (`ReadRepository` / `WriteRepository`)
- `Arc<Store>` instead of `&'db Database`
- Typed table wrappers
- InMemoryRepository with failure injection

## Dependencies

- **07a** (complete): Provides reference pattern for segregated traits + Store usage
- **06**: `db::testing` infrastructure (complete)
- **ADR 016**: Segregated Unified Repository pattern

## Acceptance Criteria

- [ ] `vault/repository.rs` defines segregated traits (Read/Write/Repository)
- [ ] `vault/storage/` submodule created with same structure as `note/storage/`
- [ ] `RedbRepository` uses `Arc<Store>` for all operations
- [ ] Typed table definitions in `tables.rs` (`UuidTable` for file/dir views, `PathTable` for lookups)
- [ ] `InMemoryRepository` in `testing.rs` with operation counters + failure injection
- [ ] Vault error enum reduced to persistence-boundary variants
- [ ] `vault/processor.rs` uses new repository
- [x] ~~`Database` → `Store` bridge available~~ (CANCELLED: all code switched directly to `Arc<Store>` per architectural decision)
- [ ] All vault tests pass (`mise run test`)
- [ ] `storage_legacy.rs` renamed and marked for future removal

## Key Decisions

1. **NO BRIDGE**: All code switches directly to `Arc<Store>` (no `Database` → `Store` conversion)
2. **Batch operations**: Add high-level batch methods (`save_many_file_views`, `delete_many_file_views`, etc.) to replace `with_batch_read` / `with_batch_write`
3. **Test migration**: Port 36 existing integration tests to new `storage/read.rs` and `storage/write.rs`

## Blocks

- Issue 07c: Processor and integration test migration

## Estimated Effort

~15-16 hours, 52 test cycles across 9 phases (see TDD Plan below)

---

## TDD Implementation Plan

Following vertical slicing (tracer bullets) from Issue 07a. **NO horizontal slices** — one test → one implementation → repeat.

### Current State Analysis

**Repository Interface** (`vault/storage.rs`, 1461 lines):

**Read operations** (14 methods):
- `find_file_view_by_path(path)` → `Option<FileView>`
- `find_dir_view_by_path(path)` → `Option<DirView>`
- `get_file_view(id)` → `Option<FileView>`
- `get_dir_view(id)` → `Option<DirView>`
- `get_entry(path)` → `Option<FsEntryView>` (tries file first, then dir)
- `find_file_views_by_basename(basename)` → `Vec<FileView>` (multimap)
- `find_file_views_by_parent(parent_id)` → `Vec<FileView>` (multimap)
- `list_file_views_by_format(format)` → `Vec<FileView>` (multimap)
- `list_markdown_file_views()` → `Vec<FileView>`
- `list_all_file_views()` → `Vec<FileView>`
- `list_file_paths()` → `Vec<NormalizedPath>`
- `list_all_dir_views()` → `Vec<DirView>`
- `list_dir_paths()` → `Vec<NormalizedPath>`
- `with_batch_read<F>(f)` → **REMOVE** (not used anywhere)

**Write operations** (5 methods):
- `save_file_view(path, file)` → `()`
- `save_dir_view(path, dir)` → `()`
- `delete_file_view(id)` → `()`
- `delete_dir_view(id)` → `()`
- `with_batch_write<F>(f)` → **REMOVE** (replace with high-level batch methods)

**Tables** (6 total):
- `FILE_VIEWS: UuidTable<FileId, &[u8]>` (primary)
- `DIR_VIEWS: UuidTable<DirId, &[u8]>` (primary)
- `FILE_ID_BY_PATH: PathTable<FileId>` (index)
- `DIR_ID_BY_PATH: PathTable<DirId>` (index)
- `FILE_IDS_BY_BASENAME: MultimapTableDefinition<&str, FileId>` (multimap)
- `FILE_IDS_BY_PARENT: UuidMultimap<DirId, FileId>` (multimap)
- `FILE_IDS_BY_FORMAT: MultimapTableDefinition<&str, FileId>` (multimap)

**Test Coverage**: 36 existing integration tests in `storage.rs` (lines 862-1461)

---

### Phase 0: Planning & Interface Design ⏳

**Goal**: Define traits and get user approval.

#### Trait Design

**`ReadRepository` trait** (13 methods — no batch adapter):
```rust
pub trait ReadRepository {
    // Direct lookups
    fn get_file_view(&self, id: FileId) -> Result<Option<FileView>, VaultRepositoryError>;
    fn get_dir_view(&self, id: DirId) -> Result<Option<DirView>, VaultRepositoryError>;

    // Path lookups (cross-table: path → id → view)
    fn find_file_view_by_path(&self, path: &NormalizedPath) -> Result<Option<FileView>, VaultRepositoryError>;
    fn find_dir_view_by_path(&self, path: &NormalizedPath) -> Result<Option<DirView>, VaultRepositoryError>;
    fn get_entry(&self, path: &NormalizedPath) -> Result<Option<FsEntryView>, VaultRepositoryError>;

    // Multimap index queries
    fn find_file_views_by_basename(&self, basename: &str) -> Result<Vec<FileView>, VaultRepositoryError>;
    fn find_file_views_by_parent(&self, parent_id: DirId) -> Result<Vec<FileView>, VaultRepositoryError>;
    fn list_file_views_by_format(&self, format: FileFormat) -> Result<Vec<FileView>, VaultRepositoryError>;
    fn list_markdown_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError>;

    // List operations (table/index scans)
    fn list_all_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError>;
    fn list_file_paths(&self) -> Result<Vec<NormalizedPath>, VaultRepositoryError>;
    fn list_all_dir_views(&self) -> Result<Vec<DirView>, VaultRepositoryError>;
    fn list_dir_paths(&self) -> Result<Vec<NormalizedPath>, VaultRepositoryError>;
}
```

**`WriteRepository` trait** (8 methods — adds high-level batch operations):
```rust
pub trait WriteRepository {
    // Single operations
    fn save_file_view(&self, path: &NormalizedPath, file: &FileView) -> Result<(), VaultRepositoryError>;
    fn save_dir_view(&self, path: &NormalizedPath, dir: &DirView) -> Result<(), VaultRepositoryError>;
    fn delete_file_view(&self, id: FileId) -> Result<(), VaultRepositoryError>;
    fn delete_dir_view(&self, id: DirId) -> Result<(), VaultRepositoryError>;

    // Batch operations (replace with_batch_write)
    fn save_many_file_views(&self, entries: &[(NormalizedPath, FileView)]) -> Result<(), VaultRepositoryError>;
    fn save_many_dir_views(&self, entries: &[(NormalizedPath, DirView)]) -> Result<(), VaultRepositoryError>;
    fn delete_many_file_views(&self, ids: &[FileId]) -> Result<(), VaultRepositoryError>;
    fn delete_many_dir_views(&self, ids: &[DirId]) -> Result<(), VaultRepositoryError>;
}
```

**`Repository` marker trait**:
```rust
pub trait Repository: ReadRepository + WriteRepository {}
impl<T> Repository for T where T: ReadRepository + WriteRepository {}
```

**User approval checkpoint**: Confirm trait design (especially batch methods) before proceeding.

---

### Phase 1: Create Repository Traits (Tracer Bullet) 🔴

**Goal**: Define the contract.

#### Test Cycle 1: Trait Compilation
- **RED**: Create `vault/repository.rs` with trait definitions
- **GREEN**: Traits compile
- **Verify**: `cargo check --package lithos-core`

#### Test Cycle 2: Marker Trait Auto-Implementation
- **RED**: Define `Repository` as marker trait with blanket impl
- **GREEN**: Create dummy struct, impl both read + write, confirm `Repository` auto-implements
- **Verify**: Compiles

---

### Phase 2: Create Table Definitions Module 🔴

**Goal**: Extract table constants into `storage/tables.rs`.

#### Test Cycle 3: Table Definitions Extract
- **RED**: Create `vault/storage/tables.rs`, move 6 table constants from `vault/mod.rs`
- **GREEN**: Update imports, existing tests still pass
- **Verify**: Run existing vault storage tests

---

### Phase 3: Create RedbRepository Struct in `storage/mod.rs` 🔴

**Goal**: Set up new structure with `Arc<Store>`.

#### Test Cycle 4: RedbRepository Struct Setup
- **RED**: Create `vault/storage/mod.rs`:
  ```rust
  use std::sync::Arc;
  use crate::db::Store;

  pub struct RedbRepository {
      pub(crate) store: Arc<Store>,
  }

  impl RedbRepository {
      #[inline]
      #[must_use]
      pub fn new(store: Arc<Store>) -> Self {
          Self { store }
      }
  }
  ```
- **GREEN**: Struct compiles
- **Verify**: `cargo check`

---

### Phase 4: Implement Read Operations (`storage/read.rs`) 🔴

**Goal**: Migrate all 13 read methods.

One test → one impl per method (vertical slicing).

#### Test Cycle 5: `get_file_view()` - Tracer Bullet
- **RED**: Write integration test:
  ```rust
  #[test]
  fn get_file_view_returns_none_for_missing_id() {
      let (_tempdir, store) = Store::open_temp().unwrap();
      let repo = RedbRepository::new(Arc::new(store));
      let result = repo.get_file_view(FileId::new());
      assert!(result.unwrap().is_none());
  }
  ```
- **GREEN**: Implement using `store.read(|tx| ...)` pattern
- **Verify**: Test passes

#### Test Cycles 6-17: Implement Remaining Read Methods

Repeat for each read method (following Note pattern):
- `get_file_view()` — returns stored view
- `get_dir_view()` — direct lookup
- `find_file_view_by_path()` — cross-table (path → id → view)
- `find_dir_view_by_path()` — cross-table
- `get_entry()` — tries file first, then dir
- `find_file_views_by_basename()` — multimap scan
- `find_file_views_by_parent()` — multimap scan
- `list_file_views_by_format()` — multimap scan
- `list_markdown_file_views()` — special case (FileFormat::Markdown)
- `list_all_file_views()` — table scan
- `list_file_paths()` — index scan
- `list_all_dir_views()` — table scan
- `list_dir_paths()` — index scan

**Test naming**: `<method>_<condition>_<outcome>`

Examples:
- `find_file_view_by_path_returns_none_for_missing_path`
- `find_file_view_by_path_performs_cross_table_lookup`
- `find_file_views_by_basename_returns_all_matches`
- `list_markdown_file_views_filters_by_markdown_format`

**Estimated cycles**: 13 tests

---

### Phase 5: Implement Write Operations (`storage/write.rs`) 🔴

**Goal**: Migrate all 8 write methods (4 single + 4 batch).

#### Test Cycle 18: `save_file_view()` - Persists View and Indexes
- **RED**: Write test:
  ```rust
  #[test]
  fn save_file_view_persists_view_and_all_indexes() {
      let (_tempdir, store) = Store::open_temp().unwrap();
      let repo = RedbRepository::new(Arc::new(store));
      let path = NormalizedPath::try_new("test.md").unwrap();
      let file = FileView::new(...);

      repo.save_file_view(&path, &file).unwrap();

      // Verify via read operations
      let retrieved = repo.find_file_view_by_path(&path).unwrap().unwrap();
      assert_eq!(retrieved.id(), file.id());
  }
  ```
- **GREEN**: Implement using `store.write(|tx| ...)` — atomically write to:
  - `FILE_VIEWS` (primary)
  - `FILE_ID_BY_PATH` (path index)
  - `FILE_IDS_BY_BASENAME` (multimap)
  - `FILE_IDS_BY_PARENT` (multimap)
  - `FILE_IDS_BY_FORMAT` (multimap)
- **Verify**: Test passes

#### Test Cycle 19: `save_file_view()` - Atomicity
- **RED**: Test verifies rollback if index write fails
- **GREEN**: `store.write()` auto-rolls back on error
- **Verify**: Test passes

#### Test Cycle 20: `delete_file_view()` - Removes View and Indexes
- **RED**: Write test (migrate existing `delete_file_view_removes_primary_and_all_indexes`)
- **GREEN**: Implement removal from all 5 locations atomically
- **Verify**: Test passes

#### Test Cycle 21: `save_dir_view()` - Persists Dir and Path Index
- **RED**: Write test for dir view persistence
- **GREEN**: Implement using `DIR_VIEWS` + `DIR_ID_BY_PATH`
- **Verify**: Test passes

#### Test Cycle 22: `delete_dir_view()` - Removes Dir and Index
- **RED**: Migrate existing `delete_dir_view_removes_primary_and_path_index`
- **GREEN**: Implement removal
- **Verify**: Test passes

#### Test Cycle 23: `save_many_file_views()` - Batch Save
- **RED**: Write test saving multiple file views in one transaction
- **GREEN**: Implement using `store.write(|tx| ...)` — loop over entries, save each
- **Verify**: Test passes, all entries persisted

#### Test Cycle 24: `save_many_dir_views()` - Batch Save
- **RED**: Write test saving multiple dir views
- **GREEN**: Implement batch save
- **Verify**: Test passes

#### Test Cycle 25: `delete_many_file_views()` - Batch Delete
- **RED**: Write test deleting multiple file views
- **GREEN**: Implement batch delete (idempotent for missing IDs)
- **Verify**: Test passes

#### Test Cycle 26: `delete_many_dir_views()` - Batch Delete
- **RED**: Write test deleting multiple dir views
- **GREEN**: Implement batch delete
- **Verify**: Test passes

**Estimated cycles**: 9 tests

---

### Phase 6: Build InMemoryRepository (`storage/testing.rs`) 🔴

**Goal**: Adopt `db::testing` infrastructure (following Note pattern exactly).

#### Test Cycle 27: InMemoryRepository - Basic Structure
- **RED**: Create `InMemoryRepository` in `storage/testing.rs`:
  ```rust
  #[derive(Clone)]
  pub(crate) struct InMemoryRepository {
      harness: Arc<InMemoryHarness>,
      file_views: Arc<RwLock<HashMap<FileId, FileView>>>,
      dir_views: Arc<RwLock<HashMap<DirId, DirView>>>,
      file_path_to_id: Arc<RwLock<HashMap<NormalizedPath, FileId>>>,
      dir_path_to_id: Arc<RwLock<HashMap<NormalizedPath, DirId>>>,
      // Multimap indexes
      files_by_basename: Arc<RwLock<HashMap<String, Vec<FileId>>>>,
      files_by_parent: Arc<RwLock<HashMap<DirId, Vec<FileId>>>>,
      files_by_format: Arc<RwLock<HashMap<FileFormat, Vec<FileId>>>>,
  }
  ```
- **GREEN**: Struct compiles with all fields
- **Verify**: `cargo check`

#### Test Cycles 28-50: Implement InMemoryRepository Methods

Following Note pattern — organize tests into modules:
```rust
#[cfg(test)]
mod tests {
    mod defaults { ... }      // new() creates empty repo
    mod lookup { ... }         // get_*, find_* methods
    mod list { ... }           // list_* methods
    mod indexes { ... }        // multimap queries
    mod update { ... }         // save_* methods
    mod delete { ... }         // delete_* methods
    mod counters { ... }       // operation counting
    mod injection { ... }      // failure injection (BeforeRead/BeforeWrite)
}
```

**Read methods** (~13 tests):
- Basic lookups (`get_file_view`, `get_dir_view`)
- Path lookups (`find_file_view_by_path`, `find_dir_view_by_path`, `get_entry`)
- Multimap queries (`find_file_views_by_basename`, `find_file_views_by_parent`, `list_file_views_by_format`, `list_markdown_file_views`)
- List operations (`list_all_file_views`, `list_file_paths`, `list_all_dir_views`, `list_dir_paths`)

**Write methods** (~8 tests):
- Single operations (`save_file_view`, `save_dir_view`, `delete_file_view`, `delete_dir_view`)
- Batch operations (`save_many_file_views`, `save_many_dir_views`, `delete_many_file_views`, `delete_many_dir_views`)

**Instrumentation** (~3 tests):
- Operation counters (`harness.counters().snapshot()` — reads/writes increment)
- Failure injection `BeforeRead` (using `SelectiveFailureInjector`)
- Failure injection `BeforeWrite`

**Estimated cycles**: ~24 tests

---

### Phase 7: Update Error Handling 🔴

**Goal**: `VaultRepositoryError` converts `InMemoryDbError`.

#### Test Cycle 51: InMemoryDbError Conversion
- **RED**: Write test in `vault/error.rs`:
  ```rust
  #[test]
  fn in_memory_db_error_converts_to_repository_error() {
      use crate::db::testing::{InMemoryDbError, FailurePoint};

      let err = InMemoryDbError::InjectedFailure {
          point: FailurePoint::BeforeRead,
          reason: "test".into(),
      };
      let repo_err: VaultRepositoryError = err.into();
      assert!(matches!(repo_err, VaultRepositoryError::Storage(_)));
  }
  ```
- **GREEN**: Add `#[cfg(test)] impl From<InMemoryDbError> for VaultRepositoryError`
- **Verify**: Test passes

---

### Phase 8: Migrate VaultProcessor 🔴

**Goal**: Update `vault/processor.rs` to use `Arc<Store>` (**NO BRIDGE**).

#### Current Signature
```rust
pub fn process_full(
    self,
    db: &Database,
    config: &Config,
) -> Result<VaultProcessReport, VaultProcessError>
```

#### New Signature
```rust
pub fn process_full(
    self,
    store: Arc<Store>,
    config: &Config,
) -> Result<VaultProcessReport, VaultProcessError>
```

#### Test Cycle 52: VaultProcessor Uses Store
- **RED**: Change signature, update internal repository construction:
  ```rust
  let vault_repo = VaultRepository::new(Arc::clone(&store));
  let note_repo = note::storage::RedbRepository::new(Arc::clone(&store));
  ```
- **GREEN**: Update all callers (tests) to create `Arc<Store>` instead of `Database`
- **Verify**: All vault processor tests pass

---

### Phase 9: Integration & Cleanup 🔴

**Goal**: Preserve behavior, remove legacy.

#### Test Cycle 53: Migrate Existing Integration Tests
- **RED**: Port 36 existing tests from `storage.rs` to `storage/read.rs` and `storage/write.rs`
- **GREEN**: Update to use `Store::open_temp()` instead of `Database::open()`
- **Verify**: All tests pass

#### Test Cycle 54: Delete Old Monolithic Implementation
- **RED**: Rename `storage.rs` → `storage_legacy.rs`, mark for future removal
- **GREEN**: All references point to `storage/` submodule
- **Verify**: `cargo test` passes

#### Test Cycle 55: Update Module Exports
- **RED**: Update `vault/mod.rs`:
  - Remove old table constants (now in `storage/tables.rs`)
  - Export new `storage` submodule
  - Mark `storage_legacy` for removal
- **GREEN**: Imports resolve
- **Verify**: `cargo check`

---

## Success Criteria (Definition of Done)

Per project standards in `AGENTS.md`:

- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public APIs have tests (21 trait methods covered)
- [ ] Tests cover critical paths (CRUD, multimap queries, batch operations)
- [ ] No `unwrap()`/`panic!` in production code
- [ ] Context boundaries respected
- [ ] Unified Repository pattern (Read + Write traits, marker)
- [ ] Type-driven design (private fields, validated constructors)
- [ ] Documentation updated (doc comments for all trait methods)
- [ ] No string allocation anti-patterns
- [ ] `VaultProcessor` uses `Arc<Store>` (NO `Database` references anywhere)

---

## Risk Assessment

### High Risks

1. **VaultProcessor Caller Changes** — Breaking change to public API
   - All tests that call `process_full` need `Arc<Store>` instead of `&Database`
   - Mitigation: Identify all callers first (gitnexus query), update atomically

2. **Multimap Index Consistency** — 3 multimap indexes must stay in sync
   - `save_file_view` writes to 5 locations atomically
   - `delete_file_view` removes from 5 locations atomically
   - Mitigation: Single transaction, comprehensive tests

3. **Test Migration Volume** — 36 existing tests to port
   - Risk: Introducing subtle behavior changes
   - Mitigation: Port in batches, verify subset passes before continuing

---

## Summary

**Phases**: 9 total
**Test Cycles**: 55 total
**Estimated Effort**: 15-16 hours

**Next Step**: User approval to proceed with Phase 1 (create repository traits)

---

**Status**: 🟡 Awaiting user approval
