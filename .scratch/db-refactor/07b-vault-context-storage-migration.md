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
- Vault repository: `Repository::new(db: &'db Database)` — legacy pattern
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
   - Use new `Repository` with `Arc<Store>`
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
    fn list_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError>;
    fn list_file_paths(&self) -> Result<Vec<NormalizedPath>, VaultRepositoryError>;
    fn list_dir_views(&self) -> Result<Vec<DirView>, VaultRepositoryError>;
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
- `list_file_views()` — table scan
- `list_file_paths()` — index scan
- `list_dir_views()` — table scan
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
    - List operations (`list_file_views`, `list_file_paths`, `list_dir_views`, `list_dir_paths`)

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
  let vault_repo = Repository::new(Arc::clone(&store));
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

---

## 🚀 TDD Plan: O(1) Reverse Path Index (Post-Migration Enhancement)

**Prerequisite**: Complete Phase 1-9 above first. This is an **optimization** applied after the core migration.

### Problem Statement

**Current O(N) bottleneck** in `FileDeleteContext::load` and `DirDeleteContext::load`:

```rust
// vault/storage/write.rs (current implementation)
fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> {
    // ... load from primary table (O(1)) ...

    // ❌ O(N) SCAN: Iterate EVERY path in the vault
    let path = path_table
        .iter()?  // Scans entire FILE_ID_BY_PATH table
        .find(|res| {
            res.as_ref()
                .map(|(_, id)| id.value() == file_id)
                .unwrap_or(false)
        })
        .transpose()?
        .map(|(path, _)| path.value());

    Ok(Self { path, basename, parent_id, format })
}
```

**Why O(N)**:
- `FILE_ID_BY_PATH` indexed by **Path** (key), not **FileId** (value)
- Finding path for a FileId requires linear scan of all entries
- With 10,000 files: **10,000 comparisons per delete**
- Batch delete 100 files: **1,000,000 comparisons** (O(N×M))

### Solution: Bidirectional Path Index

Add reverse lookup tables:
- `PATH_BY_FILE_ID: UuidTable<FileId, String>`
- `PATH_BY_DIR_ID: UuidTable<DirId, String>`

**After optimization**:
```rust
fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> {
    // ✅ O(1) LOOKUP: Direct hash table access
    let reverse_path_table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
    let path = reverse_path_table.get(&file_id)?.map(|g| g.value().to_owned());

    Ok(Self { path, /* ... */ })
}
```

**Complexity improvement**:
- Single delete: O(N) → O(1)
- Batch delete M files: O(N×M) → O(M)
- Example (100 deletes in 10k vault): ~1,000,000 comparisons → ~100 lookups

---

### Phase 10: Add Reverse Path Index Tables 🔴

**Goal**: Add infrastructure without changing behavior.

#### Cycle 56: Define `PATH_BY_FILE_ID` Table
**Test**: `vault/storage/tables.rs`
```rust
#[test]
fn path_by_file_id_table_exists() {
    let (_tempdir, store) = Store::open_temp().unwrap();

    let result = store.write(|tx| {
        tx.try_open_table(PATH_BY_FILE_ID.definition())
    });

    assert!(result.is_ok(), "PATH_BY_FILE_ID table should be accessible");
}
```

**Implementation**:
```rust
// vault/storage/tables.rs
pub(crate) const PATH_BY_FILE_ID: UuidTable<FileId, String> =
    UuidTable::new("path_by_file_id");
```

**Verify**: Test passes

---

#### Cycle 57: Define `PATH_BY_DIR_ID` Table
**Test**:
```rust
#[test]
fn path_by_dir_id_table_exists() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let result = store.write(|tx| {
        tx.try_open_table(PATH_BY_DIR_ID.definition())
    });
    assert!(result.is_ok());
}
```

**Implementation**:
```rust
pub(crate) const PATH_BY_DIR_ID: UuidTable<DirId, String> =
    UuidTable::new("path_by_dir_id");
```

**Verify**: Test passes

---

### Phase 11: Populate Reverse Index on Write 🔴

**Goal**: Insert reverse index entries during `save_*` operations.

#### Cycle 58: `save_file_view` Creates Reverse Index (Tracer Bullet)
**Test**: `vault/storage/write.rs` → `mod upsert`
```rust
#[test]
fn file_creates_reverse_path_index() {
    // Arrange
    let (_temp, repo) = temp_vault();
    let file = sample_file(None, "test.md", FileFormat::Markdown);
    let path = NormalizedPath::try_new("notes/test.md").unwrap();

    // Act
    repo.save_file_view(&path, &file).unwrap();

    // Assert: Reverse index contains the path
    let recovered_path = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        table.get(&file.id())?.map(|g| g.value().to_owned()).transpose()
    }).unwrap();

    assert_eq!(
        recovered_path,
        Some(path.as_str().to_owned()),
        "Reverse index should map FileId → path"
    );
}
```

**Implementation**: Update `save_file_view` in `storage/write.rs`
```rust
impl WriteRepository for RedbRepository {
    fn save_file_view(&self, path: &NormalizedPath, file: &FileView) -> ... {
        self.store.write(|tx| {
            Self::remove_file_graph(tx, file.id())?;

            // Open all tables (existing 5 + new reverse index)
            let mut file_table = tx.try_open_table(FILE_VIEWS.definition())?;
            let mut path_table = tx.try_open_table(FILE_ID_BY_PATH.definition())?;
            let mut reverse_path_table =
                tx.try_open_table(PATH_BY_FILE_ID.definition())?;  // NEW
            let mut by_basename = tx.try_open_multimap(FILE_IDS_BY_BASENAME)?;
            let mut by_parent = tx.try_open_multimap(FILE_IDS_BY_PARENT.definition())?;
            let mut by_format = tx.try_open_multimap(FILE_IDS_BY_FORMAT)?;

            // Insert to all 6 locations atomically
            file_table.insert(&file.id(), file_bytes.as_ref())?;
            path_table.insert(path.as_str().to_owned(), &file.id())?;
            reverse_path_table.insert(&file.id(), path.as_str().to_owned())?;  // NEW
            by_basename.insert(basename.as_str(), &file.id())?;
            if let Some(parent_id) = file.parent_id() {
                by_parent.insert(&parent_id, &file.id())?;
            }
            by_format.insert(file.format().as_str(), &file.id())?;
            Ok(())
        })
    }
}
```

**Verify**: Test passes

---

#### Cycle 59: `save_dir_view` Creates Reverse Index
**Test**:
```rust
#[test]
fn dir_creates_reverse_path_index() {
    let (_temp, repo) = temp_vault();
    let dir = sample_dir("notes");
    let path = NormalizedPath::try_new("notes").unwrap();

    repo.save_dir_view(&path, &dir).unwrap();

    let recovered_path = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_DIR_ID.definition())?;
        table.get(&dir.id())?.map(|g| g.value().to_owned()).transpose()
    }).unwrap();

    assert_eq!(recovered_path, Some(path.as_str().to_owned()));
}
```

**Implementation**: Update `save_dir_view` similarly

**Verify**: Test passes

---

#### Cycle 60: Overwrite Updates Reverse Index
**Test**: Ensure saving same ID with different path updates reverse index
```rust
#[test]
fn file_overwrite_updates_reverse_index() {
    // Arrange
    let (_temp, repo) = temp_vault();
    let id = FileId::new();
    let first = FileView::new(
        id, None,
        FileName::new("old.md".into()),
        FileFormat::Markdown,
        FileMetadata::new(FsTimes::new(None, None), 128, false),
        [1u8; 32],
    );
    let second = FileView::new(
        id, None,
        FileName::new("new.md".into()),
        FileFormat::Markdown,
        FileMetadata::new(FsTimes::new(None, None), 256, false),
        [2u8; 32],
    );
    let old_path = NormalizedPath::try_new("notes/old.md").unwrap();
    let new_path = NormalizedPath::try_new("notes/new.md").unwrap();

    // Act: Save twice with same ID, different paths
    repo.save_file_view(&old_path, &first).unwrap();
    repo.save_file_view(&new_path, &second).unwrap();

    // Assert: Reverse index has new path only
    let recovered_path = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        table.get(&id)?.map(|g| g.value().to_owned()).transpose()
    }).unwrap();

    assert_eq!(
        recovered_path,
        Some(new_path.as_str().to_owned()),
        "Reverse index should contain updated path"
    );
}
```

**Implementation**: Existing `remove_file_graph` call before insert handles this (if updated in Cycle 61-62)

**Verify**: Test passes

---

### Phase 12: Use Reverse Index in Delete (O(1) Optimization) 🔴

**Goal**: Replace O(N) scan with O(1) reverse index lookup.

#### Cycle 61: `FileDeleteContext::load` Uses Reverse Index
**Test**: `vault/storage/write.rs` → `mod delete`
```rust
#[test]
fn file_delete_removes_reverse_index_entry() {
    // Arrange
    let (_temp, repo) = temp_vault();
    let file = sample_file(None, "delete.md", FileFormat::Markdown);
    let path = NormalizedPath::try_new("notes/delete.md").unwrap();
    repo.save_file_view(&path, &file).unwrap();

    // Act: Delete the file
    repo.delete_file_view(file.id()).unwrap();

    // Assert: Reverse index entry is cleaned
    let recovered_path = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        table.get(&file.id())?.map(|g| g.value().to_owned()).transpose()
    }).unwrap();

    assert!(
        recovered_path.is_none(),
        "Reverse index should be cleaned on delete"
    );
}
```

**Implementation**: Update `FileDeleteContext::load`
```rust
impl FileDeleteContext {
    fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> {
        let file_table = tx.try_open_table(FILE_VIEWS.definition())?;

        // Load from primary table (unchanged)
        let (basename, parent_id, format) = if let Some(file) = file_table
            .get(&file_id)?
            .map(|g| FileView::from_bytes(g.value()))
            .transpose()?
        {
            (
                Some(BaseName::try_from(file.name().clone())
                    .map_err(|e| DbError::Deserialization(e.to_string()))?),
                file.parent_id(),
                Some(file.format()),
            )
        } else {
            (None, None, None)
        };

        // ✅ O(1) reverse index lookup (REPLACES O(N) SCAN)
        let reverse_path_table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        let path = reverse_path_table
            .get(&file_id)?
            .map(|g| g.value().to_owned());

        Ok(Self { path, basename, parent_id, format })
    }
}
```

**Also update** `remove_file_graph` to clean reverse index:
```rust
fn remove_file_graph(tx: &WriteTx, file_id: FileId) -> Result<(), DbError> {
    let ctx = FileDeleteContext::load(tx, file_id)?;
    Self::remove_file_path_index(tx, ctx.path.as_deref())?;
    Self::remove_file_basename_index(tx, ctx.basename.as_ref(), file_id)?;
    Self::remove_file_parent_index(tx, ctx.parent_id, file_id)?;
    Self::remove_file_format_index(tx, ctx.format, file_id)?;

    // NEW: Remove reverse index entry
    let mut reverse_path_table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
    reverse_path_table.remove(&file_id)?;

    Self::remove_file_primary(tx, file_id)
}
```

**Verify**: Test passes

---

#### Cycle 62: `DirDeleteContext::load` Uses Reverse Index
**Test**:
```rust
#[test]
fn dir_delete_removes_reverse_index_entry() {
    let (_temp, repo) = temp_vault();
    let dir = sample_dir("notes");
    let path = NormalizedPath::try_new("notes").unwrap();
    repo.save_dir_view(&path, &dir).unwrap();

    repo.delete_dir_view(dir.id()).unwrap();

    let recovered_path = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_DIR_ID.definition())?;
        table.get(&dir.id())?.map(|g| g.value().to_owned()).transpose()
    }).unwrap();

    assert!(recovered_path.is_none());
}
```

**Implementation**: Update `DirDeleteContext::load` and `remove_dir_graph` similarly

**Verify**: Test passes

---

#### Cycle 63: Batch Delete Cleans All Reverse Index Entries
**Test**: Verify batch operations maintain reverse index consistency
```rust
#[test]
fn batch_file_delete_removes_all_reverse_index_entries() {
    // Arrange
    let (_temp, repo) = temp_vault();
    let a = sample_file(None, "a.md", FileFormat::Markdown);
    let b = sample_file(None, "b.md", FileFormat::Markdown);
    repo.save_file_view(&NormalizedPath::try_new("a.md").unwrap(), &a).unwrap();
    repo.save_file_view(&NormalizedPath::try_new("b.md").unwrap(), &b).unwrap();

    // Act: Batch delete
    repo.delete_many_file_views(&[a.id(), b.id()]).unwrap();

    // Assert: Both reverse index entries gone
    let paths = repo.store.read(|tx| {
        let table = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        Ok::<_, DbError>((
            table.get(&a.id())?.is_some(),
            table.get(&b.id())?.is_some()
        ))
    }).unwrap();

    assert_eq!(
        paths,
        (false, false),
        "All reverse index entries should be removed"
    );
}
```

**Implementation**: Existing `delete_many_file_views` calls `remove_file_graph` per ID, which now cleans reverse index

**Verify**: Test passes

---

### Phase 13: Edge Cases 🔴

**Goal**: Ensure robustness for missing/invalid data.

#### Cycle 64: Delete Non-Existent File Is Idempotent
**Test**:
```rust
#[test]
fn delete_missing_file_is_idempotent() {
    let (_temp, repo) = temp_vault();
    let missing_id = FileId::new();

    let result = repo.delete_file_view(missing_id);

    assert!(
        result.is_ok(),
        "Delete of non-existent file should succeed"
    );
}
```

**Implementation**: `FileDeleteContext::load` returns `path: None` for missing entries, `remove()` on non-existent key is no-op

**Verify**: Test passes

---

#### Cycle 65: Batch Delete With Mixed IDs Succeeds
**Test**:
```rust
#[test]
fn batch_delete_with_missing_ids_succeeds() {
    let (_temp, repo) = temp_vault();
    let file = sample_file(None, "exists.md", FileFormat::Markdown);
    let path = NormalizedPath::try_new("exists.md").unwrap();
    repo.save_file_view(&path, &file).unwrap();

    let missing = FileId::new();

    let result = repo.delete_many_file_views(&[file.id(), missing]);

    assert!(
        result.is_ok(),
        "Batch delete with missing IDs should succeed"
    );
}
```

**Implementation**: Already handled by idempotent delete

**Verify**: Test passes

---

### Phase 14: Performance Verification (Benchmark) 📊

**Goal**: Prove O(1) characteristic via benchmarking.

Per **rust-best-practices Chapter 3**: "Don't guess, measure."

#### Cycle 66: Benchmark — Delete Time Independent of Vault Size
**After all tests pass**, create `benches/vault_delete_performance.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lithos_core::{
    db::Store,
    vault::{
        model::{FileId, FileView},
        repository::WriteRepository,
        storage::RedbRepository,
    },
};
use std::sync::Arc;

fn setup_vault_with_n_files(n: usize) -> (tempfile::TempDir, RedbRepository, Vec<FileId>) {
    let (tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(Arc::new(store));
    let mut ids = Vec::new();

    for i in 0..n {
        let file = /* create test file */;
        let path = NormalizedPath::try_new(&format!("file_{i}.md")).unwrap();
        repo.save_file_view(&path, &file).unwrap();
        ids.push(file.id());
    }

    (tempdir, repo, ids)
}

fn bench_delete_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete_scaling");

    // Test with vaults of different sizes
    for vault_size in [1_000, 10_000, 100_000] {
        let (_temp, repo, ids) = setup_vault_with_n_files(vault_size);
        let target_id = ids[vault_size / 2]; // Middle file

        group.bench_with_input(
            BenchmarkId::new("single_delete", vault_size),
            &vault_size,
            |b, _| {
                b.iter(|| {
                    // Note: This will fail after first iteration (file deleted)
                    // Use setup/teardown or measure FileDeleteContext::load directly
                    repo.delete_file_view(black_box(target_id))
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_delete_scaling);
criterion_main!(benches);
```

**Expected Results** (O(1) proof):
```
delete_scaling/single_delete/1000     time:   [12.5 µs 12.8 µs 13.1 µs]
delete_scaling/single_delete/10000    time:   [12.7 µs 13.0 µs 13.3 µs]  ← Similar!
delete_scaling/single_delete/100000   time:   [12.9 µs 13.2 µs 13.5 µs]  ← Still similar!
```

**Baseline (without reverse index)** would show linear growth:
```
delete_scaling/single_delete/1000     time:   [15 µs   ...]
delete_scaling/single_delete/10000    time:   [150 µs  ...]  ← 10× slower
delete_scaling/single_delete/100000   time:   [1500 µs ...]  ← 100× slower
```

**Acceptance**: Times remain **roughly constant** across vault sizes (proves O(1))

**Run**: `cargo bench --bench vault_delete_performance`

---

### Phase 15: Documentation & Refactor 📝

**Goal**: Update docs to reflect new design.

#### Cycle 67: Update Module Doc
**Implementation**: Update `vault/storage/write.rs` module doc:
```rust
//! Write operations for vault files and directories.
//!
//! ...existing content...
//!
//! ## Performance Optimization: Reverse Path Index
//!
//! The module maintains bidirectional path indexes:
//! - `FILE_ID_BY_PATH`: Path → FileId (forward)
//! - `PATH_BY_FILE_ID`: FileId → Path (reverse)
//!
//! The reverse index enables O(1) path lookup during delete operations.
//! Previously, [`FileDeleteContext::load`] performed an O(N) scan of the
//! path table. With the reverse index, deletion time is **independent of
//! vault size**.
//!
//! **Trade-off**: Write operations maintain 2 path indexes (slightly higher
//! write cost) in exchange for guaranteed O(1) delete performance.
```

**Verify**: `cargo doc --open`, review updated docs

---

#### Cycle 68: Update `FileDeleteContext::load` Doc
**Implementation**: Add `# Performance` section
```rust
/// Loads the index metadata for a given file.
///
/// Reads the primary [`FileView`] record to extract basename, parent ID,
/// and format. Performs an O(1) lookup in the reverse path index
/// ([`PATH_BY_FILE_ID`]) to retrieve the path.
///
/// # Parameters
///
/// * `tx` — An open write transaction containing the vault tables.
/// * `file_id` — The unique identifier of the file to look up.
///
/// # Returns
///
/// A [`FileDeleteContext`] with populated index fields. Fields for
/// entries that do not exist in the database are `None`.
///
/// # Errors
///
/// Returns [`DbError`] if the table access fails or if a stored record
/// cannot be deserialized.
///
/// # Performance
///
/// This method performs **O(1) lookups** via hash table access, regardless
/// of vault size. Prior to the reverse index optimization, this method
/// performed an O(N) scan of the forward path index.
fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> { ... }
```

**Verify**: `cargo doc`, check doc comments render correctly

---

#### Cycle 69: Extract Helpers (Optional Refactor)
**After all tests GREEN**, consider extracting common patterns:

```rust
impl RedbRepository {
    /// Insert path into both forward and reverse indexes atomically.
    fn insert_file_path_indexes(
        tx: &WriteTx,
        file_id: FileId,
        path: &NormalizedPath,
    ) -> Result<(), DbError> {
        let mut forward = tx.try_open_table(FILE_ID_BY_PATH.definition())?;
        let mut reverse = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        forward.insert(path.as_str().to_owned(), &file_id)?;
        reverse.insert(&file_id, path.as_str().to_owned())?;
        Ok(())
    }

    /// Remove path from both forward and reverse indexes atomically.
    fn remove_file_path_indexes(
        tx: &WriteTx,
        file_id: FileId,
        path: Option<&str>,
    ) -> Result<(), DbError> {
        if let Some(path) = path {
            let mut forward = tx.try_open_table(FILE_ID_BY_PATH.definition())?;
            forward.remove(path.to_owned())?;
        }
        let mut reverse = tx.try_open_table(PATH_BY_FILE_ID.definition())?;
        reverse.remove(&file_id)?;
        Ok(())
    }
}
```

**Trade-off**: Adds indirection but reduces duplication. Only refactor if `save_file_view` and `save_dir_view` have significant overlap.

**Verify**: All tests still pass after refactor

---

## Summary: Reverse Index Optimization

**Phases Added**: 6 (Phase 10-15)
**Test Cycles Added**: 14 (Cycles 56-69)
**Estimated Additional Effort**: 4-5 hours

**Storage Impact**:
- **Before**: 1 path index (Path → ID)
- **After**: 2 path indexes (Path ↔ ID bidirectional)
- **Cost**: ~2× path index storage (~100 bytes/file × 10k files = ~1MB)

**Performance Impact**:
- **Delete**: O(N) → O(1)
- **Batch delete (M files)**: O(N×M) → O(M)
- **Write**: Negligible overhead (one extra index insert per save)

**Complexity Table**:

| Operation                 | Before (Forward Only)    | After (Bidirectional) |
| ------------------------- | ------------------------ | --------------------- |
| **Single delete**             | O(N) — scan all paths    | O(1) — hash lookup    |
| **Batch delete M files**      | O(N×M) — scan per file   | O(M) — one lookup/file   |
| **Save file**                 | O(1)                     | O(1) (same)           |
| **100 deletes in 10k vault** | ~1,000,000 comparisons   | ~100 lookups          |

**Next Steps After Core Migration**:
1. Complete Phases 1-9 (core migration)
2. Verify all tests pass
3. Proceed with Phases 10-15 (reverse index optimization)
4. Run benchmark (Cycle 66) to prove O(1) characteristic

---

**Status**: 🟡 Awaiting user approval for core migration (Phases 1-9) before adding reverse index optimization (Phases 10-15)
