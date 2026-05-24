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
   - Both are well-structured and can be reused

3. **RedbRepository Implementation** (lines 315-650):
   - Single impl block for all operations
   - Uses 3 tables: `NOTES_BY_ID`, `NOTE_ID_BY_PATH`, `LIST_VIEWS_BY_NOTE_ID`
   - Direct `Database` reference (not `Store`)

4. **Existing Integration Tests** (lines 662-749):
   - `save_persists_path()` - saves note and verifies path index
   - `delete_note_removes_note()` - deletes note and verifies removal
   - Both use tempdir + redb, must be preserved

**What's Missing**:
- ❌ No `note/repository.rs` with segregated traits
- ❌ No `note/storage/` submodule structure
- ❌ No `InMemoryRepository` implementation
- ❌ No `db::testing` infrastructure adoption (no harness, counters, failure injection)
- ❌ Uses `Database` directly instead of `Store` with transaction helpers

### Critical Gaps in Original Refactor Note

The original issue description (v1 - 2026-05-12) correctly identifies **what** to build but fails to address:

1. **Migration Strategy**: No plan for moving from monolithic `storage.rs` to `storage/` submodule
2. **Test Preservation**: No explicit strategy to keep existing tests passing
3. **Batch Adapter Reuse**: `RedbBatchNoteReader` and `RedbBatchNoteWriter` already exist—should they be preserved or refactored?
4. **Table Definition Location**: Where should `NOTES_BY_ID`, `NOTE_ID_BY_PATH`, `LIST_VIEWS_BY_NOTE_ID` constants move?
5. **Error Handling Updates**: `NoteRepositoryError` needs `From<InMemoryDbError>` impl
6. **Store vs Database**: Need to migrate from `Database` to `Store` for transaction helpers

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
2. `RedbRepository` fields are `pub(crate)` to allow child module access
3. Each impl file (`read.rs`, `write.rs`) is ~500-800 lines (maintainable)
4. `InMemoryRepository` in `testing.rs`:
   - `Arc<InMemoryHarness>` for instrumentation
   - `Arc<RwLock<HashMap<...>>>` for state
   - Uses `read_lock()` / `write_lock()` helpers
   - Supports `FailurePoint::BeforeRead` and `FailurePoint::BeforeWrite`
   - Maps `InMemoryDbError` → `SchemaStorageError`

---

## TDD Implementation Plan

### Principles (per `tdd` and `rust-best-practices` skills)

1. **Vertical Slicing (Tracer Bullets)**: One test → one implementation → repeat
2. **Behavior-Focused Tests**: Tests verify public interface behavior, not implementation details
3. **Integration-Style Tests**: Exercise real code paths, avoid mocking internals
4. **No Horizontal Slices**: Never write all tests first, then all implementation

### Test Naming Convention

Pattern: `<operation>_<condition>_<outcome>`

Examples:
- `find_by_id_returns_none_for_missing_note`
- `save_persists_note_and_path_index`
- `in_memory_injects_failure_before_read`
- `delete_note_removes_note_and_all_indices`

---

## Implementation Phases

### Phase 0: Planning & Interface Design ⏳

**Goal**: Define public interfaces and get user approval before writing code.

**Tasks**:
- [ ] Design `NoteReadRepository` trait interface
  - Methods: `find_by_id`, `find_by_path`, `list`, `with_archived_by_id`, `with_archived_by_path`, `with_batch_read`, `get_list_view`
  - Return types: `Result<Option<Note>, NoteRepositoryError>`, `Result<Vec<Note>, ...>`
  - Zero-copy methods: `with_archived_*` using closure pattern

- [ ] Design `NoteWriteRepository` trait interface
  - Methods: `save`, `delete_note`, `cache_list_view`, `invalidate_list_view`, `with_batch_write`
  - Atomicity: Multi-table writes in single transaction
  - Batch operations: Save/delete multiple notes in one transaction

- [ ] Design `InMemoryRepository` state structure
  - Indices: `notes: HashMap<NoteId, Note>`, `path_to_id: HashMap<NotePath, NoteId>`, `views: HashMap<NoteId, ListView>`
  - Harness: `Arc<InMemoryHarness>`
  - Lock strategy: `RwLock` for concurrent access

- [ ] **User Approval Checkpoint**: Review trait designs, confirm critical behaviors to test, agree on migration strategy

**Status**: 🔴 Not Started

---

### Phase 1: Create Repository Traits (Tracer Bullet)

**Goal**: Define the contract before any implementation.

#### Test 1: Trait Compilation
- **RED**: Create `note/repository.rs` with trait definitions
- **GREEN**: Traits compile, no implementations yet
- **Verify**: `cargo check` passes
- **Status**: ⬜

#### Test 2: Marker Trait Auto-Implementation
- **RED**: Define `NoteRepository` as `trait NoteRepository: NoteReadRepository + NoteWriteRepository {}`
- **GREEN**: Any type implementing both read + write gets `NoteRepository` via blanket impl
- **Verify**: Create dummy struct, impl both, confirm `NoteRepository` compiles
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 2: Migrate Table Definitions

**Goal**: Extract table constants into `storage/tables.rs` without changing behavior.

#### Test 3: Table Definitions Extract
- **RED**: Create `note/storage/tables.rs`, move `NOTES_BY_ID`, `NOTE_ID_BY_PATH`, `LIST_VIEWS_BY_NOTE_ID`
- **GREEN**: Update imports in existing `storage.rs`, tests still pass
- **Verify**: Run existing `save_persists_path()` test
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 3: Create RedbRepository Struct in `storage/mod.rs`

**Goal**: Set up the new structure with `pub(crate)` field visibility.

#### Test 4: RedbRepository Struct Setup
- **RED**: Create `note/storage/mod.rs` with:
  ```rust
  use std::sync::Arc;
  use crate::db::Store;

  pub struct RedbRepository {
      pub(crate) store: Arc<Store>,
  }

  impl RedbRepository {
      pub fn new(store: Arc<Store>) -> Self {
          Self { store }
      }
  }
  ```
- **GREEN**: Struct compiles with `pub(crate)` fields
- **Verify**: `cargo check` passes
- **Status**: ⬜

**Phase Status**: 🔴 Not Started

---

### Phase 4: Implement Read Operations (`storage/read.rs`)

**Goal**: Migrate all read operations from monolithic `storage.rs` to segregated `read.rs`.

#### Test 5: find_by_id() - Public Interface
- **RED**: Write test:
  ```rust
  #[test]
  fn find_by_id_returns_none_for_missing_note() {
      let store = Arc::new(Store::open_temp().unwrap());
      let repo = RedbRepository::new(store);
      let result = repo.find_by_id(NoteId::new());
      assert!(result.unwrap().is_none());
  }
  ```
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

#### Test 9: with_batch_read() - Batch Reader
- **RED**: Write test using batch reader to access multiple notes in one transaction
- **GREEN**: Implement using existing `RedbBatchNoteReader`
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

#### Test 13: with_batch_write() - Batch Writer
- **RED**: Write test using batch writer to save multiple notes in one transaction
- **GREEN**: Implement using existing `RedbBatchNoteWriter`
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

**Overall Status**: 🔴 Not Started (0/9 phases complete)

**Estimated Effort**:
- Phase 0-3: ~2 hours (setup + trait design)
- Phase 4-5: ~4 hours (read/write implementations)
- Phase 6-7: ~3 hours (in-memory adapter + error handling)
- Phase 8-9: ~2 hours (cleanup + refactor)
- **Total**: ~11 hours (23 test cycles)

**Last Updated**: 2026-05-25

---

## References

- **ADR 016**: Segregated Unified Repository Traits (`docs/adr/016-segregated-unified-repository-traits.md`)
- **ADR 018**: Explicit Redb Adapter Seam (`docs/adr/018-explicit-redb-adapter-seam.md`)
- **Schema Reference**: `lithos-core/src/schema/storage/` (reference implementation)
- **DB Testing Seam**: `lithos-core/src/db/testing.rs` (infrastructure primitives)
- **TDD Skill**: `.agents/skills/tdd/SKILL.md` (test-first methodology)
- **Rust Best Practices**: `.agents/skills/rust-best-practices/SKILL.md` (Apollo handbook patterns)
