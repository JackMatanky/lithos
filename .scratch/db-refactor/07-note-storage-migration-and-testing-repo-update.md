---
title: 07-note-storage-migration-and-testing-repo-update
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-10
---

## Type

AFK

## Labels

- needs-triage

## What to build

Migrate Note persistence to the new storage seam with `repository.rs`, `storage/read.rs`, `storage/write.rs`, and `storage/tables.rs`. Update Note `testing.rs` in-memory Repository Adapter to match the new Repository Interface and behavior.

This slice is complete when Note read/write and batch behavior are preserved end-to-end in both redb-backed and in-memory test flows.

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Migrate Note persistence to the segregated storage seam.

**Current behavior:**
Note persistence uses the legacy v1 repository and storage pattern.

**Desired behavior:**
1. Define `NoteReadRepository` and `NoteWriteRepository` traits in `note/repository.rs`.
2. Define `NoteRepository` as a marker trait extending both.
3. Implement `NoteRedbRepository` split across `note/storage/read.rs` and `note/storage/write.rs`.
4. Update `testing.rs` in-memory adapter to implement the new segregated traits.
5. Adopt the shared `db::testing` seam in Note's in-memory adapter:
   - Use `read_lock` / `write_lock` helpers
   - Embed `InMemoryHarness` for counters/failure injection
   - Map `InMemoryDbError` directly into Note storage errors

**Key interfaces:**
- `NoteReadRepository` / `NoteWriteRepository`
- `NoteRedbRepository`
- `NoteRepository` (marker)

**Acceptance criteria:**
- [ ] `NoteReadRepository` and `NoteWriteRepository` defined in `note/repository.rs`.
- [ ] `NoteRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Note `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Note behavior tests pass with new storage seam.
- [ ] Note in-memory adapter uses `db::testing::{read_lock, write_lock, InMemoryHarness}`.
- [ ] Note in-memory adapter supports failure injection (`BeforeRead`/`BeforeWrite`) and has integration tests for both paths.
- [ ] Note in-memory adapter follows naming/structure conventions from `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## Acceptance criteria

- [ ] Note Repository Adapter uses the new storage module layout and DB seam.
- [ ] Note `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Note behavior tests pass, with added coverage for changed batch/error semantics where needed.
- [ ] Cross-context adapter adoption complete for Note:
  - [ ] lock helpers use `db::testing` primitives
  - [ ] harness counters wired and verified
  - [ ] failure injection wired and verified
  - [ ] direct `InMemoryDbError` mapping in place

## Cross-context guidance reference

- This issue must apply the shared adapter guidance established by Issue 06
  (DB seam foundation) and keep adapter behavior local to Note context.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`

---

## Analysis & Implementation Status (2026-05-25)

### Design Decisions (Approved 2026-05-25)

#### 1. Error Enum Design ✅

**Final `NoteRepositoryError` variants** (4 only):
```rust
pub enum NoteRepositoryError {
    /// Wraps all DB-layer errors (redb, transactions, deserialization)
    #[error("storage error: {0}")]
    Storage(#[from] crate::db::DbError),

    /// Note not found by ID
    #[error("note not found: {id}")]
    NotFoundById { id: NoteId },

    /// Note not found by path
    #[error("note not found at path: {path}")]
    NotFoundByPath { path: NotePath },

    /// Duplicate path constraint violation
    #[error("duplicate path: note already exists at {path}")]
    DuplicatePath { path: NotePath },
}
```

**Removed variants** (layer boundary violations):
- ❌ `Corruption` → Use `Storage(DbError::Deserialization)`
- ❌ `ResourceLimitExceeded` → Use `Storage(DbError)` (wraps redb size errors)
- ❌ `ConstraintViolation` → Too generic, replaced with `DuplicatePath`
- ❌ `IdentityConflict` → Duplicate concept, merged into `DuplicatePath`
- ❌ `InvalidNoteData` → Wrong layer (domain = `NoteError`, deserialization = `DbError`)

**Rationale**: Repository errors focus on persistence concerns only. Domain validation → `NoteError`, infrastructure failures → `DbError`.

#### 2. Batch Operations Pattern ✅

**Remove `with_batch_*` methods** (exposes transactions):
```rust
// ❌ OLD: Exposes transaction control to caller
fn with_batch_write<F, R>(&self, f: F) -> Result<R>
where F: FnOnce(&mut BatchWriter) -> Result<R>
```

**Add high-level batch methods** (manages transactions internally):
```rust
// ✅ NEW: Transaction is implementation detail
fn save_many(&self, notes: &[Note]) -> Result<()> {
    self.store.write(|tx| {
        for note in notes {
            // insert in single transaction
        }
    })
}

fn find_many_by_id(&self, ids: &[NoteId]) -> Result<Vec<Note>>
fn delete_many(&self, ids: &[NoteId]) -> Result<()>
```

**Rationale**: Follow Schema's pattern. No special "batch operations" in redb—just multiple operations in one transaction. Callers don't need transaction control.

#### 3. Table Type Migration ✅

**Use typed table wrappers** (not raw `TableDefinition`):
```rust
// ✅ NEW: Typed wrappers from db/tables.rs
pub const NOTES: UuidTable<NoteId> = UuidTable::new("notes");
pub const LIST_VIEWS: UuidTable<Uuid> = UuidTable::new("list_views");
pub const NOTE_ID_BY_PATH: PathTable<NoteId> = PathTable::new("note_id_by_path");
```

**Rationale**: Type safety, consistent with Schema pattern, leverages `db::tables` infrastructure.

---

### Implementation Progress (2026-05-25)

**Status**: 🟡 In Progress (6/9 phases complete)

**Completed**:
- ✅ Phase 0: Design decisions approved (error enum, batch operations, table types)
- ✅ Phase 1: Module structure created (`note/storage/` + typed tables)
- ✅ Phase 2: Repository traits created (`note/repository.rs`)
- ✅ Phase 3: `RedbRepository` created in `storage/mod.rs` with `Arc<Store>`
- ✅ Phase 4: `ReadRepository` implemented in `storage/read.rs`
- ✅ Phase 5: `WriteRepository` implemented in `storage/write.rs`
- ✅ Phase 7: `NoteRepositoryError` reduced to approved boundary variants

**In Progress**:
- 🟡 Phase 6: In-memory adapter (`storage/testing.rs`)

**Remaining**:
- ⬜ Phase 8: Migrate existing integration tests
- ⬜ Phase 9: Remove legacy `storage_legacy.rs` and cleanup

**Files Created**:
- `lithos-core/src/note/repository.rs`
- `lithos-core/src/note/storage/tables.rs`
- `lithos-core/src/note/storage/read.rs`
- `lithos-core/src/note/storage/write.rs`

**Files Modified**:
- `lithos-core/src/note/mod.rs`
- `lithos-core/src/note/storage/mod.rs`
- `lithos-core/src/note/error.rs`
- `lithos-core/src/note/storage_legacy.rs`
- `lithos-core/src/note/storage/write.rs` (WriteRepository impl)
- `lithos-core/src/note/storage/testing.rs` (InMemoryRepository)

---

### Current State Assessment

**File**: `lithos-core/src/note/storage.rs` (750 lines)

**What Exists**:
1. **Monolithic Unified `Repository` Trait** (lines 30-135):
   - Single trait mixing read + write operations (violates ADR 016)
   - Contains: `find_by_id`, `find_by_path`, `list`, `save`, `delete_note`, `cache_list_view`, batch operations
   - Associated types: `BatchReader`, `BatchWriter`, `Error`, `NoteArchived`

2. **Batch Adapters** (lines 137-313):
   - `RedbBatchNoteReader` - read-only batch operations
   - `RedbBatchNoteWriter` - write-capable batch operations
   - **INCORRECT**: These should be deleted, not reused. Note must follow Schema pattern.

3. **RedbRepository Implementation** (lines 315-650):
   - Single impl block for all operations
   - Uses 3 tables: `NOTES_BY_ID`, `NOTE_ID_BY_PATH`, `LIST_VIEWS_BY_NOTE_ID`
   - **INCORRECT**: Uses legacy `&'db Database` instead of `Arc<Store>`

4. **Existing Integration Tests** (lines 662-749):
   - `save_persists_path()` - saves note and verifies path index
   - `delete_note_removes_note()` - deletes note and verifies removal
   - Both use tempdir + redb, must be preserved

**What's Missing**:
- ❌ No `note/repository.rs` with segregated traits
- ❌ No `note/storage/` submodule structure
- ❌ No `InMemoryRepository` implementation
- ❌ No `db::testing` infrastructure adoption (no harness, counters, failure injection)
- ❌ Uses legacy `Database` API instead of `Store` with per-method transactions

### Architectural Analysis

#### Current State (Note - Legacy)
- **Adapter**: `RedbRepository<'db>` with `&'db Database`
- **Transaction API**: `db.batch_read(|reader| ...)` / `db.batch_write(|writer| ...)`
- **Tables**: Raw `redb::TableDefinition<&str, &[u8]>` constants
- **Batch Adapters**: `RedbBatchNoteReader` and `RedbBatchNoteWriter` wrapper structs
- **Transaction Scope**: Controlled by caller via `with_batch_read/write` trait methods

#### Target State (Following Schema Pattern)
- **Adapter**: `RedbRepository` with `Arc<Store>`
- **Transaction API**: `store.read(|tx| ...)` / `store.write(|tx| ...)`
- **Tables**: Typed wrappers (`UuidTable`, `PathTable`)
- **Batch Adapters**: **None** (call transaction APIs directly)
- **Transaction Scope**: Per-method (each repository method manages its own transaction)

#### Note-Specific Features

The Note context has these additional requirements beyond Schema:

1. **ListView Caching**: Three methods for materialized view management:
   - `cache_list_view(&self, view: &ListView) -> Result<(), E>`
   - `invalidate_list_view(&self, note_id: NoteId) -> Result<(), E>`
   - `get_list_view(&self, note_id: NoteId) -> Result<ListView, E>` (or `Result<Option<ListView>, E>`?)

2. **Path Index Management**: Must maintain `NOTE_ID_BY_PATH` atomically with note saves/deletes

3. **Unique Path Constraint**: Two notes cannot have the same path (enforced in `save`)

#### Naming Violations (docs/naming-taxonomy.md)

Current violations to fix:

1. ❌ `delete_note(id)` → ✅ `delete(id)` (Repository pattern rule)
2. ❌ `get_list_view(id) -> Result<ListView, E>` → ✅ `find_list_view(id) -> Result<Option<ListView>, E>` (or keep as `get_*` if cache MUST exist?)
3. ✅ `find_by_id`, `find_by_path` - correct (optional lookups)
4. ✅ `save`, `list` - correct (repository pattern)
5. ✅ `cache_list_view`, `invalidate_list_view` - acceptable (cache-specific operations)

### Table Type Migration

#### Current (lithos-core/src/note/mod.rs lines 89-105)
```rust
pub(crate) const NOTES_BY_ID: redb::TableDefinition<&str, &[u8]> = ...;
pub(crate) const NOTE_ID_BY_PATH: redb::TableDefinition<&str, &[u8]> = ...;
pub(crate) const LIST_VIEWS_BY_NOTE_ID: redb::TableDefinition<&str, &[u8]> = ...;
```

#### Target (new file: lithos-core/src/note/storage/tables.rs)
```rust
use crate::{
    db::{PathTable, UuidTable},
    impl_redb_uuid,
    note::aggregate::NoteId,
};

impl_redb_uuid!(NoteId);

/// Note aggregates indexed by UUID
pub const NOTES: UuidTable<NoteId, &[u8]> = UuidTable::new("notes");

/// Materialized list views indexed by note UUID
pub const LIST_VIEWS: UuidTable<NoteId, &[u8]> = UuidTable::new("list_views");

/// Path-to-NoteId index for fast path-based lookup
pub const NOTE_ID_BY_PATH: PathTable<&[u8]> = PathTable::new("note_id_by_path");
```

### Reference Implementation: Schema Context

The Schema context provides the reference pattern:

**Schema Structure** (`lithos-core/src/schema/`):
```
schema/
├── repository.rs          # Segregated traits (ReadRepository, WriteRepository, Repository)
└── storage/
    ├── mod.rs             # RedbRepository struct with pub(crate) store: Arc<Store>
    ├── read.rs            # impl ReadRepository for RedbRepository
    ├── write.rs           # impl WriteRepository for RedbRepository
    ├── tables.rs          # Table definitions (SCHEMAS, SCHEMA_ID_BY_NAME, etc.)
    └── testing.rs         # InMemoryRepository with db::testing adoption
```

**Key Patterns from Schema**:
1. Traits use generic names (`ReadRepository`, not `SchemaReadRepository`)
2. `RedbRepository` uses `Arc<Store>`, not `&Database`
3. Each impl file (`read.rs`, `write.rs`) uses `store.read(|tx| ...)` / `store.write(|tx| ...)`
4. No batch adapter structs - call transaction APIs directly
5. Typed table wrappers (`UuidTable`, `PathTable`) instead of raw definitions
6. Per-method transaction boundaries (each method opens/closes its own transaction)
7. `InMemoryRepository` in `testing.rs`:
   - `Arc<InMemoryHarness>` for instrumentation
   - `Arc<RwLock<HashMap<...>>>` for state
   - Uses `read_lock()` / `write_lock()` helpers
   - Supports `FailurePoint::BeforeRead` and `FailurePoint::BeforeWrite`
   - Maps `InMemoryDbError` → `SchemaStorageError`

---

## TDD Implementation Plan

### Planning Phase

**Before writing any code:**

1. Confirm with user:
   - Should `get_list_view` return `Result<Option<ListView>>` or `Result<ListView>` (error if cache missing)?
   - Should we rename `delete_note` → `delete`?
   - Any other behavioral changes needed?

2. Identify test priorities (from most to least critical):
   - **Critical**: `save` preserves note + path index atomically
   - **Critical**: `delete` removes note + all indexes atomically
   - **Critical**: Path uniqueness constraint enforced in `save`
   - **High**: `find_by_id`, `find_by_path`, `list` basic retrieval
   - **High**: ListView cache/invalidate/get round-trip
   - **Medium**: Zero-copy `with_archived_*` methods
   - **Medium**: In-memory adapter parity with failure injection
   - **Low**: Batch operations (may remove if not needed)

### Phase 1: Create Module Structure (Tracer Bullet)

**Goal**: Establish new file layout without breaking existing code.

#### Cycle 1: Create empty storage module
- **Test**: `cargo check` passes after creating empty module structure
- **Implementation**:
  1. Create `lithos-core/src/note/storage/` directory
  2. Create `mod.rs`, `read.rs`, `write.rs`, `tables.rs`, `testing.rs` (empty)
  3. Add `pub mod storage;` to `lithos-core/src/note/mod.rs`
- **Verify**: `cargo check` passes

#### Cycle 2: Define typed table constants
- **Test**: Tables compile and export correctly
- **Implementation**:
  1. Implement `tables.rs` with `UuidTable` and `PathTable` wrappers
  2. Add `impl_redb_uuid!(NoteId)` macro
  3. Export tables from `storage/mod.rs`
- **Verify**: `cargo check`, tables visible in module tree

#### Cycle 3: Create RedbRepository struct with Arc<Store>
- **Test**: Struct compiles with correct field visibility
- **Implementation**:
  1. Define `RedbRepository { store: Arc<Store> }` in `storage/mod.rs`
  2. Add constructor `pub fn new(store: Arc<Store>) -> Self`
  3. Mark `store` field as `pub(crate)` for child module access
- **Verify**: `cargo check`

### Phase 2: Define Repository Traits

**Goal**: Create segregated trait interfaces matching Schema pattern.

#### Cycle 4: Define ReadRepository trait ✅ COMPLETED
- **Test**: Trait compiles with all read method signatures
- **Implementation** (`lithos-core/src/note/repository.rs`):
```rust
pub trait ReadRepository {
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteRepositoryError>;
    fn find_by_path(&self, path: &NotePath) -> Result<Option<Note>, NoteRepositoryError>;
    fn find_many_by_id(&self, ids: &[NoteId]) -> Result<Vec<Note>, NoteRepositoryError>;
    fn list(&self) -> Result<Vec<Note>, NoteRepositoryError>;
}
```
- **Verify**: ✅ `cargo check` passes
- **Status**: ✅ Complete (2026-05-25)

#### Cycle 5: Define WriteRepository trait ✅ COMPLETED
- **Test**: Trait compiles with write operations
- **Implementation**:
```rust
pub trait WriteRepository {
    fn save(&self, note: &Note) -> Result<NoteId, NoteRepositoryError>;
    fn save_many(&self, notes: &[Note]) -> Result<Vec<NoteId>, NoteRepositoryError>;
    fn delete(&self, id: NoteId) -> Result<(), NoteRepositoryError>;
    fn delete_many(&self, ids: &[NoteId]) -> Result<(), NoteRepositoryError>;
}
```
- **Verify**: ✅ `cargo check` passes
- **Status**: ✅ Complete (2026-05-25)

**Note**: ListView methods (`cache_list_view`, `invalidate_list_view`, `find_list_view`) will be added in a separate cycle after core CRUD operations are working.

#### Cycle 6: Define Repository marker trait ✅ COMPLETED
- **Test**: Blanket impl auto-implements for types with both traits
- **Implementation**:
```rust
pub trait Repository: ReadRepository + WriteRepository {}
impl<T> Repository for T where T: ReadRepository + WriteRepository {}
```
- **Verify**: ✅ `cargo check` passes
- **Status**: ✅ Complete (2026-05-25)

### Phase 3: Implement Read Operations (Vertical Slices)

**Goal**: Implement one read method at a time, test → implement → verify.

#### Cycle 7: Implement find_by_id
- **Test** (`lithos-core/src/note/storage/read.rs` tests):
```rust
#[test]
fn find_by_id_returns_some_when_note_exists() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let note = create_test_note("test.md");
    let id = repo.save(&note).unwrap();

    let found = repo.find_by_id(id).unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), id);
}

#[test]
fn find_by_id_returns_none_when_note_missing() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let missing_id = NoteId::new();

    let found = repo.find_by_id(missing_id).unwrap();

    assert!(found.is_none());
}
```
- **Implementation** (`lithos-core/src/note/storage/read.rs`):
```rust
impl ReadRepository for RedbRepository {
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let table = tx.open_table(NOTES.definition())?;
                let Some(guard) = table.get(id.to_string().as_str())? else {
                    return Ok(None);
                };
                let archived = rkyv::check_archived_root::<Note>(guard.value())
                    .map_err(|e| DbError::Deserialization(e.into()))?;
                let note: Note = archived.deserialize(&mut rkyv::Infallible)?;
                Ok(Some(note))
            })
            .map_err(NoteRepositoryError::from)
    }

    // Stub other methods temporarily
}
```
- **Verify**: `cargo test note::storage::read::tests::find_by_id`
- **Status**: ✅ Complete

#### Cycle 8: Implement find_by_path
- **Test**:
```rust
#[test]
fn find_by_path_returns_some_when_path_exists() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let path = NotePath::try_new("notes/test.md").unwrap();
    let note = create_test_note_with_path(path.clone());
    repo.save(&note).unwrap();

    let found = repo.find_by_path(&path).unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().path(), &path);
}
```
- **Implementation**: Use `NOTE_ID_BY_PATH` index, then lookup by ID
- **Verify**: Test passes
- **Status**: ✅ Complete

#### Cycle 9-11: Implement list, with_archived_by_id, with_archived_by_path
- `list` and `find_many_by_id` implemented + tested (`note/storage/read.rs`)
- `with_archived_*` deferred pending trait boundary decision for this migration slice
- **Status**: ✅ Partial Complete (list/find_many done; archived methods deferred)

#### Cycle 12: Implement find_list_view
- **Test**:
```rust
#[test]
fn find_list_view_returns_none_when_not_cached() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let note_id = NoteId::new();

    let found = repo.find_list_view(note_id).unwrap();

    assert!(found.is_none());
}
```
- **Implementation**: Lookup from `LIST_VIEWS_BY_NOTE_ID` table
- **Verify**: Test passes
- **Status**: ✅ Complete

### Phase 4: Implement Write Operations (Critical Path First)

**Goal**: Implement write methods with atomic index maintenance.

#### Cycle 13: Implement save (upsert with path uniqueness)
- **Test**:
```rust
#[test]
fn save_persists_note_and_path_index() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let path = NotePath::try_new("test.md").unwrap();
    let note = create_test_note_with_path(path.clone());

    let id = repo.save(&note).unwrap();

    let found_by_id = repo.find_by_id(id).unwrap();
    let found_by_path = repo.find_by_path(&path).unwrap();
    assert!(found_by_id.is_some());
    assert!(found_by_path.is_some());
    assert_eq!(found_by_id.unwrap().id(), id);
}

#[test]
fn save_enforces_unique_path_constraint() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let path = NotePath::try_new("test.md").unwrap();
    let note1 = create_test_note_with_path(path.clone());
    let note2 = create_test_note_with_path(path.clone());

    repo.save(&note1).unwrap();
    let result = repo.save(&note2);

    assert!(matches!(result, Err(NoteRepositoryError::DuplicatePath(_))));
}

#[test]
fn save_updates_existing_note_at_same_path() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let path = NotePath::try_new("test.md").unwrap();
    let note1 = create_test_note_with_path(path.clone());
    let id = repo.save(&note1).unwrap();

    let note2 = note1.clone().with_id(id); // Same ID, same path
    let id2 = repo.save(&note2).unwrap();

    assert_eq!(id, id2);
    let found = repo.find_by_id(id).unwrap();
    assert!(found.is_some());
}
```
- **Implementation**:
```rust
fn save(&self, note: &Note) -> Result<NoteId, Self::Error> {
    let path = note.path();

    self.store.write(|tx| {
        // Check for existing note at this path
        let existing_id = {
            let path_table = tx.try_open_table(NOTE_ID_BY_PATH.definition())?;
            path_table.and_then(|t| t.get_path(path)).transpose()?
        };

        // Determine note ID (existing or new)
        let note_id = existing_id.unwrap_or_else(NoteId::new);

        // Enforce uniqueness: if path exists with different ID, error
        if let Some(existing) = existing_id {
            if existing != note.id() && note.id() != NoteId::default() {
                return Err(NoteRepositoryError::DuplicatePath(path.clone()).into());
            }
        }

        // Ensure note uses determined ID
        let stored_note = if note_id == note.id() {
            Cow::Borrowed(note)
        } else {
            Cow::Owned(note.clone().with_id(note_id))
        };

        // Atomic write: note + path index
        let mut note_table = tx.open_table(NOTES_BY_ID.definition())?;
        let mut path_table = tx.open_table(NOTE_ID_BY_PATH.definition())?;

        note_table.insert(note_id, stored_note.as_ref())?;
        path_table.insert_path(path, &note_id)?;

        Ok(note_id)
    }).map_err(NoteRepositoryError::from)
}
```
- **Verify**: All three tests pass
- **Status**: ✅ Complete

#### Cycle 14: Implement delete (atomic cleanup)
- **Test**:
```rust
#[test]
fn delete_removes_note_and_all_indexes() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let path = NotePath::try_new("test.md").unwrap();
    let note = create_test_note_with_path(path.clone());
    let id = repo.save(&note).unwrap();

    repo.delete(id).unwrap();

    let found_by_id = repo.find_by_id(id).unwrap();
    let found_by_path = repo.find_by_path(&path).unwrap();
    assert!(found_by_id.is_none());
    assert!(found_by_path.is_none());
}

#[test]
fn delete_is_idempotent() {
    let (_tempdir, store) = Store::open_temp().unwrap();
    let repo = RedbRepository::new(store);
    let id = NoteId::new();

    repo.delete(id).unwrap();
    let result = repo.delete(id);

    assert!(result.is_ok()); // No error on missing note
}
```
- **Implementation**: Atomic delete from NOTES_BY_ID + NOTE_ID_BY_PATH
- **Verify**: Tests pass
- **Status**: ✅ Complete

#### Cycle 15-16: Implement cache_list_view, invalidate_list_view
- Follow RED → GREEN pattern
- Use `LIST_VIEWS_BY_NOTE_ID` UuidTable
- Test round-trip: cache → find → invalidate → find returns None
- **Status**: ✅ Complete

### Phase 5: In-Memory Adapter (Test Double)

**Goal**: Implement `InMemoryRepository` matching Schema's pattern.

#### Cycle 17: Create InMemoryRepository structure
- **Test**: Struct compiles with correct fields
- **Implementation**:
```rust
pub struct InMemoryRepository {
    harness: Arc<InMemoryHarness>,
    notes: Arc<RwLock<HashMap<NoteId, Note>>>,
    path_index: Arc<RwLock<HashMap<NotePath, NoteId>>>,
    list_views: Arc<RwLock<HashMap<NoteId, ListView>>>,
}
```
- **Verify**: `cargo check`

#### Cycle 18-22: Implement NoteReadRepository for InMemoryRepository
- One method per cycle (find_by_id, find_by_path, list, etc.)
- Use `read_lock()` helper from `db::testing`
- Inject failure points via harness
- Tests verify behavior + instrumentation (counters, failures)

#### Cycle 23-25: Implement NoteWriteRepository for InMemoryRepository
- Implement save, delete, cache/invalidate methods
- Use `write_lock()` helper
- Maintain atomicity semantics (all indexes update or none)
- Tests verify failure injection works

### Phase 6: Migration & Cleanup

**Goal**: Remove old code, update call sites, preserve existing tests.

#### Cycle 26: Update existing integration tests
- Migrate tests in `lithos-core/tests/note_*.rs` to use new repository
- Change `Database` → `Store` in test setup
- Verify all existing tests still pass

#### Cycle 27: Update VaultProcessor call sites
- Run GitNexus impact analysis on `VaultProcessor`
- Update to use new `NoteRepository` trait
- Verify downstream tests pass

#### Cycle 28: Remove old storage.rs
- Delete `lithos-core/src/note/storage.rs` (old monolithic file)
- Remove old table constants from `note/mod.rs`
- Remove batch adapter references
- Verify `cargo build` succeeds

#### Cycle 29: Remove deprecated Repository trait
- Delete backwards-compat `Repository` marker trait if no longer needed
- Update any remaining call sites to use `NoteRepository`
- Verify full test suite passes

### Phase 7: Final Verification

**Goal**: Ensure all quality gates pass.

- [ ] `cargo test -p lithos-core` - all tests pass
- [ ] `cargo clippy -p lithos-core` - no warnings
- [ ] `cargo fmt --check` - formatted
- [ ] No `unwrap()`/`expect()` in production code
- [ ] All public APIs have doc comments
- [ ] ADR updated if needed (architecture decision change)

## Decisions (2026-05-25) ✅ ALL APPROVED

1. ✅ **ListView retrieval**: `find_list_view() -> Result<Option<ListView>>` - cache may not exist
2. ✅ **Method naming**: `delete_note()` → `delete()` - removes note + all indexes atomically
3. ✅ **Batch operations**: Follow Schema pattern (approved decision reversal)
   - ❌ Remove `with_batch_read/write` traits (exposes transaction control)
   - ✅ Add high-level batch methods: `save_many()`, `find_many_by_id()`, `delete_many()`
   - Rationale: No special "batch operations" in redb - just multiple ops in one transaction. Follow Schema's cleaner pattern.
4. ✅ **Transaction scope**: Per-method (approved)
   - Each repository method manages its own transaction internally
   - Callers never manage transactions directly
5. ✅ **Path index atomicity**: Atomic - both updates in single `store.write()` transaction
6. ✅ **Table names**: `NOTES`, `LIST_VIEWS`, `NOTE_ID_BY_PATH` (not `*_BY_ID`)
7. ✅ **Table types**: Use typed wrappers (approved)
   - `UuidTable<NoteId>` for `NOTES`
   - `UuidTable<Uuid>` for `LIST_VIEWS`
   - `PathTable<NoteId>` for `NOTE_ID_BY_PATH`
8. ✅ **Error handling**: Approved cleanup (see Design Decisions section at top)
   - Remove: `Corruption`, `ResourceLimitExceeded`, `ConstraintViolation`, `IdentityConflict`
   - Keep: `Storage(DbError)`, `NotFoundById`, `NotFoundByPath`, `DuplicatePath`
   - Rationale: Repository errors = persistence concerns only. Domain validation → `NoteError`, infrastructure → `DbError`.

## References

- Schema storage: `lithos-core/src/schema/storage/`
- DB Store API: `lithos-core/src/db/core.rs`
- Naming taxonomy: `docs/naming-taxonomy.md`
- Rust best practices: `.agents/skills/rust-best-practices/`
- TDD workflow: `.agents/skills/tdd/`
- **GREEN**: Implement `find_by_id()` in `read.rs` using `Store::read()` transaction
- **Verify**: Test passes
- **Status**: ⬜

#### Test 6: find_by_id() - Returns Stored Note
- **RED**: Write test saving a note (using write method), then retrieving it
- **GREEN**: Minimal implementation (reuse existing logic from monolithic impl)
- **Verify**: Test passes
- **Status**: ⬜

#### Test 7: find_by_path() - Cross-Table Lookup
- **RED**: Write test:
  ```rust
  #[test]
  fn find_by_path_performs_cross_table_lookup() {
      // Save note with path, retrieve by path
      // Verify path → id → note lookup works
  }
  ```
- **GREEN**: Implement using `NOTE_ID_BY_PATH` → `NOTES_BY_ID` lookup
- **Verify**: Test passes
- **Status**: ⬜

#### Test 8: list() - Scans All Notes
- **RED**: Write test with multiple notes, verify all returned
- **GREEN**: Implement table scan over `NOTES_BY_ID`
- **Verify**: Test passes
- **Status**: ⬜

#### Test 9: find_many_by_id() - Batch Read
- **RED**: Write test requesting multiple notes by ID in one call
  ```rust
  #[test]
  fn find_many_by_id_returns_notes_in_order() {
      let store = Arc::new(Store::open_temp().unwrap());
      let repo = RedbRepository::new(Arc::clone(&store));
      let note1 = create_test_note("note1.md");
      let note2 = create_test_note("note2.md");
      let id1 = repo.save(&note1).unwrap();
      let id2 = repo.save(&note2).unwrap();

      let found = repo.find_many_by_id(&[id1, id2]).unwrap();

      assert_eq!(found.len(), 2);
      assert_eq!(found[0].id(), id1);
      assert_eq!(found[1].id(), id2);
  }
  ```
- **GREEN**: Implement `find_many_by_id()` using single `store.read()` transaction
- **Verify**: Test passes
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 5: Implement Write Operations (`storage/write.rs`)

**Goal**: Migrate all write operations to `write.rs`.

#### Test 10: save() - Persists Note
- **RED**: Write test (migrate existing `save_persists_path()` logic):
  ```rust
  #[test]
  fn save_persists_note_and_path_index() {
      let store = Arc::new(Store::open_temp().unwrap());
      let repo = RedbRepository::new(Arc::clone(&store));
      let note = /* ... */;

      let note_id = repo.save(&note).unwrap();

      // Verify both note and path index were written atomically
      let retrieved = repo.find_by_id(note_id).unwrap().unwrap();
      assert_eq!(retrieved.path(), note.path());
  }
  ```
- **GREEN**: Implement `save()` in `write.rs` using `Store::write()` transaction
- **Verify**: Test passes
- **Status**: ⬜

#### Test 11: save() - Atomicity (Multi-Table)
- **RED**: Write test that verifies rollback if second table write fails
- **GREEN**: Ensure `Store::write()` auto-rolls back on error
- **Verify**: Test passes
- **Status**: ⬜

#### Test 12: delete_note() - Removes Note
- **RED**: Migrate existing `delete_note_removes_note()` logic:
  ```rust
  #[test]
  fn delete_note_removes_note_and_all_indices() {
      // Save note, delete it, verify removal from all tables
  }
  ```
- **GREEN**: Implement `delete_note()` removing from `NOTES_BY_ID`, `NOTE_ID_BY_PATH`, `LIST_VIEWS_BY_NOTE_ID`
- **Verify**: Test passes
- **Status**: ⬜

#### Test 13: save_many() - Batch Write
- **RED**: Write test saving multiple notes in one call
  ```rust
  #[test]
  fn save_many_persists_all_notes_atomically() {
      let store = Arc::new(Store::open_temp().unwrap());
      let repo = RedbRepository::new(Arc::clone(&store));
      let note1 = create_test_note("note1.md");
      let note2 = create_test_note("note2.md");

      let ids = repo.save_many(&[note1, note2]).unwrap();

      assert_eq!(ids.len(), 2);
      let found1 = repo.find_by_id(ids[0]).unwrap();
      let found2 = repo.find_by_id(ids[1]).unwrap();
      assert!(found1.is_some());
      assert!(found2.is_some());
  }
  ```
- **GREEN**: Implement `save_many()` using single `store.write()` transaction
- **Verify**: Test passes
- **Status**: ⬜

#### Test 13b: delete_many() - Batch Delete
- **RED**: Write test deleting multiple notes in one call
  ```rust
  #[test]
  fn delete_many_removes_all_notes_atomically() {
      let store = Arc::new(Store::open_temp().unwrap());
      let repo = RedbRepository::new(Arc::clone(&store));
      let note1 = create_test_note("note1.md");
      let note2 = create_test_note("note2.md");
      let id1 = repo.save(&note1).unwrap();
      let id2 = repo.save(&note2).unwrap();

      repo.delete_many(&[id1, id2]).unwrap();

      assert!(repo.find_by_id(id1).unwrap().is_none());
      assert!(repo.find_by_id(id2).unwrap().is_none());
  }
  ```
- **GREEN**: Implement `delete_many()` using single `store.write()` transaction
- **Verify**: Test passes
- **Status**: ⬜

#### Test 14: cache_list_view() / invalidate_list_view()
- **RED**: Write tests for view caching:
  ```rust
  #[test]
  fn cache_list_view_persists_view() { /* ... */ }

  #[test]
  fn invalidate_list_view_removes_view() { /* ... */ }
  ```
- **GREEN**: Implement view operations on `LIST_VIEWS_BY_NOTE_ID` table
- **Verify**: Tests pass
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 6: Build InMemoryRepository (`storage/testing.rs`)

**Goal**: Adopt `db::testing` infrastructure for in-memory testing.

#### Test 15: InMemoryRepository - Basic Structure
- **RED**: Create `InMemoryRepository` in `storage/testing.rs`:
  ```rust
  #[derive(Clone)]
  pub(crate) struct InMemoryRepository {
      harness: Arc<InMemoryHarness>,
      notes: Arc<RwLock<HashMap<NoteId, Note>>>,
      path_to_id: Arc<RwLock<HashMap<NotePath, NoteId>>>,
      views: Arc<RwLock<HashMap<NoteId, ListView>>>,
  }
  ```
- **GREEN**: Struct compiles with `Arc<RwLock<_>>` fields
- **Verify**: `cargo check` passes
- **Status**: ⬜

#### Test 16: InMemoryRepository::find_by_id() - Basic Read
- **RED**: Write test:
  ```rust
  #[test]
  fn in_memory_find_by_id_returns_stored_note() {
      let repo = InMemoryRepository::new();
      let note = /* ... */;
      // Manually insert into repo.notes via write_lock
      let result = repo.find_by_id(note.id());
      assert_eq!(result.unwrap().unwrap().id(), note.id());
  }
  ```
- **GREEN**: Implement using `read_lock(&self.notes, "find_by_id")` helper
- **Verify**: Test passes
- **Status**: ⬜

#### Test 17: InMemoryRepository::save() - Basic Write
- **RED**: Write test saving a note, verify it's stored
- **GREEN**: Implement using `write_lock(&self.notes, "save")` helper, insert into HashMap
- **Verify**: Test passes
- **Status**: ⬜

#### Test 18: InMemoryRepository - Operation Counters
- **RED**: Write test:
  ```rust
  #[test]
  fn in_memory_increments_read_counter() {
      let repo = InMemoryRepository::new();
      repo.find_by_id(NoteId::new()).unwrap();
      let snapshot = repo.harness().counters().snapshot();
      assert_eq!(snapshot.reads, 1);
  }
  ```
- **GREEN**: Call `self.harness.counters().inc_read()` in read methods
- **Verify**: Test passes
- **Status**: ⬜

#### Test 19: InMemoryRepository - Failure Injection (BeforeRead)
- **RED**: Write test (following Schema pattern from `schema/storage/testing.rs:779-792`):
  ```rust
  #[test]
  fn in_memory_injects_failure_before_read() {
      use crate::db::testing::{FailurePoint, InMemoryHarness, SelectiveFailureInjector};

      let injector = SelectiveFailureInjector::new(FailurePoint::BeforeRead);
      let repo = InMemoryRepository::with_harness(
          InMemoryHarness::with_injector(Box::new(injector))
      );

      let result = repo.find_by_id(NoteId::new());
      assert!(matches!(result, Err(NoteRepositoryError::Storage(_))));
  }
  ```
- **GREEN**: Call `self.harness.fail_at(FailurePoint::BeforeRead)?` before read operations
- **Verify**: Test passes
- **Status**: ⬜

#### Test 20: InMemoryRepository - Failure Injection (BeforeWrite)
- **RED**: Write test for write failure injection:
  ```rust
  #[test]
  fn in_memory_injects_failure_before_write() {
      let injector = SelectiveFailureInjector::new(FailurePoint::BeforeWrite);
      let repo = InMemoryRepository::with_harness(
          InMemoryHarness::with_injector(Box::new(injector))
      );

      let note = /* ... */;
      let result = repo.save(&note);
      assert!(matches!(result, Err(NoteRepositoryError::Storage(_))));
  }
  ```
- **GREEN**: Call `self.harness.fail_at(FailurePoint::BeforeWrite)?` before write operations
- **Verify**: Test passes
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 7: Update Error Handling

**Goal**: Ensure `NoteRepositoryError` properly converts `InMemoryDbError`.

#### Test 21: InMemoryDbError Conversion
- **RED**: Write test in `note/error.rs`:
  ```rust
  #[test]
  fn in_memory_db_error_converts_to_repository_error() {
      use crate::db::testing::{InMemoryDbError, FailurePoint};

      let err = InMemoryDbError::InjectedFailure {
          point: FailurePoint::BeforeRead,
          reason: "test".into(),
      };
      let repo_err: NoteRepositoryError = err.into();
      assert!(matches!(repo_err, NoteRepositoryError::Storage(_)));
  }
  ```
- **GREEN**: Add to `note/error.rs`:
  ```rust
  #[cfg(test)]
  impl From<crate::db::testing::InMemoryDbError> for NoteRepositoryError {
      fn from(err: crate::db::testing::InMemoryDbError) -> Self {
          Self::Storage(crate::db::DbError::from(err))
      }
  }
  ```
- **Verify**: Test passes
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 8: Integration & Cleanup

**Goal**: Ensure all existing behavior is preserved, remove old code.

#### Test 22: Migrate Existing Integration Tests
- **RED**: Copy `save_persists_path()` and `delete_note_removes_note()` to use new traits
- **GREEN**: Update imports, change from `Database` to `Store`, update trait bounds
- **Verify**: Tests pass with new structure
- **Status**: ⬜

#### Test 23: Delete Old Monolithic Implementation
- **RED**: Remove old `storage.rs` implementation (keep tests in separate file or `storage/tests.rs`)
- **GREEN**: All references now point to `storage/` submodule
- **Verify**: `cargo test` passes, no compiler warnings
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 9: Refactor & Deep Modules

**Goal**: Apply Rust best practices and refactor for maintainability.

#### Refactor Candidates (from `tdd/refactoring.md`):

- [ ] **Extract Duplication**:
  - UUID encoding pattern appears in multiple methods (both read and write)
  - Error mapping from `DbError` → `NoteRepositoryError` repeated
  - Consider helper functions in `storage/mod.rs`

- [ ] **Deepen Modules** (from `tdd/deep-modules.md`):
  - Move batch adapter logic into private helper functions
  - Simplify public API surface (small interface, deep implementation)
  - Hide rkyv details behind repository interface

- [ ] **Apply SOLID Principles**:
  - **SRP**: Ensure each method does one thing (e.g., `save()` only persists, doesn't validate)
  - **OCP**: Repository traits are open for extension (can add new impls) but closed for modification
  - **LSP**: Verify `InMemoryRepository` fully substitutes for `RedbRepository` in all tests
  - **ISP**: Read/Write segregation already satisfies Interface Segregation
  - **DIP**: Domain code depends on `Repository` traits, not concrete implementations

- [ ] **Performance Checks** (from `rust-best-practices`):
  - Run `cargo clippy -- -D clippy::perf` on new code
  - Profile hot paths for unnecessary allocations
  - Verify zero-copy reads (`with_archived_*`) don't allocate
  - Check iterator chains for intermediate `.collect()` calls

**Phase Status**: 🔴 Not Started

---

## Success Criteria (Definition of Done)

Per project standards in `AGENTS.md`:

- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public APIs have tests (trait methods covered)
- [ ] Tests cover critical paths (save/find/delete cycles, batch operations)
- [ ] No `unwrap()`/`panic!` in production code (only in tests)
- [ ] Context boundaries respected (Note storage doesn't import Schema internals)
- [ ] Unified Repository pattern followed (Read + Write traits, marker trait)
- [ ] Type-driven design applied (private fields in structs, validated constructors)
- [ ] Documentation updated (doc comments for all public trait methods)
- [ ] No string allocation anti-patterns (no `.to_owned().into()`, no unnecessary `.to_string()` in hot paths)
- [ ] ADR 016 compliance verified (segregated traits, split implementations)

---

## Risk Assessment & Mitigation

### High Risks

1. **Transaction Boundary Changes → Data Loss**
   - Risk: Moving from `Database` to `Store` changes transaction semantics
   - Mitigation: Run existing integration tests after each phase, add explicit rollback tests

2. **Lock Poisoning in InMemoryRepository**
   - Risk: Concurrent access to `RwLock` under panic conditions
   - Mitigation: Use `read_lock()` / `write_lock()` helpers that centralize lock handling and map poisoned locks to errors

3. **Batch Operation Semantics Drift**
   - Risk: Batch adapters (`RedbBatchNoteReader`/`Writer`) might behave differently when wrapped in new transaction pattern
   - Mitigation: Add tests for batch operations in both redb and in-memory implementations

4. **Zero-Copy Read Lifetime Violations**
   - Risk: `with_archived_*` methods use closure pattern with complex lifetimes
   - Mitigation: Follow Schema's exact pattern, verify with clippy, test with multiple closure types

### Medium Risks

1. **Error Conversion Complexity**
   - Risk: `InMemoryDbError` → `NoteRepositoryError` might lose error context
   - Mitigation: Add tests verifying error messages are preserved, check Schema's error conversion

2. **Path Index Consistency**
   - Risk: Multi-table writes might leave orphaned path → id mappings
   - Mitigation: Add tests verifying atomic updates, check rollback behavior

---

## Dependencies & Blockers

**Blocked By** (must complete first):
- ✅ Issue 06: DB Testing Seam (completed - `db::testing` infrastructure exists)
- ✅ Schema migration (completed - provides reference pattern)
- ✅ ADR 016 (accepted - defines segregated trait pattern)

**Blocks** (waiting on this issue):
- Issue 08: Template storage migration (if planned)
- Issue 09: Config storage migration (if planned)
- Issue 10: Cross-context verification (needs all contexts migrated)

---

## Implementation Notes

### Key Decisions Made

1. **Preserve Batch Adapters**: `RedbBatchNoteReader` and `RedbBatchNoteWriter` will be kept and moved to `storage/mod.rs` (they're well-structured)

2. **Table Definitions to `tables.rs`**: All table constants move to `storage/tables.rs` following Schema pattern

3. **Store vs Database**: Migrate from direct `Database` usage to `Store` with transaction helpers (`read()`, `write()`)

4. **Test Organization**:
   - Integration tests stay in existing test files (e.g., `tests/note_storage.rs`)
   - Unit tests for `InMemoryRepository` go in `storage/testing.rs` with `#[cfg(test)]` modules

### Questions for Clarification

- [ ] Should existing integration tests move to `storage/tests.rs` or stay at crate root?
- [ ] Should `RedbBatchNoteReader`/`Writer` be public or `pub(crate)`?
- [ ] Does `NoteId` already implement `UuidV7DbType` trait?

---

## Progress Tracking

**Overall Status**: 🟡 In Progress (6/9 phases complete)

**Completed Phases**:
- ✅ Phase 0: Design decisions approved (2026-05-25)
- ✅ Phase 1: Module structure + tables (`note/storage/`)
- ✅ Phase 2: Repository traits (`note/repository.rs`)
- ✅ Phase 3: RedbRepository struct (`note/storage/mod.rs`)
- ✅ Phase 4: ReadRepository implementation (`note/storage/read.rs`)
- ✅ Phase 5: WriteRepository implementation (`note/storage/write.rs`)
- ✅ Phase 7: Repository error cleanup (`note/error.rs`)

**Current Phase**:
- 🟡 Phase 6: In-memory adapter (`note/storage/testing.rs`)

**Estimated Effort**:
- Phase 0-5: ✅ ~6 hours (design + traits + redb read/write)
- Phase 6-7: ⏳ ~3 hours (in-memory adapter + final error boundary polish)
- Phase 8-9: ⬜ ~2 hours (integration migration + cleanup)
- **Total**: ~11 hours (24 test cycles)

**Last Updated**: 2026-05-25 (after read/write + list-view cycles)

---

## References

- **ADR 016**: Segregated Unified Repository Traits (`docs/adr/016-segregated-unified-repository-traits.md`)
- **ADR 018**: Explicit Redb Adapter Seam (`docs/adr/018-explicit-redb-adapter-seam.md`)
- **Schema Reference**: `lithos-core/src/schema/storage/` (reference implementation)
- **DB Testing Seam**: `lithos-core/src/db/testing.rs` (infrastructure primitives)
- **TDD Skill**: `.agents/skills/tdd/SKILL.md` (test-first methodology)
- **Rust Best Practices**: `.agents/skills/rust-best-practices/SKILL.md` (Apollo handbook patterns)
