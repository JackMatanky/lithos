---
title: 07c-note-processor-and-tests-migration
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
- blocked-by-07a-07b

## What to build

Migrate all call sites from `storage_legacy::Repository` to the new segregated `note::repository::Repository` trait. This completes the Note context storage migration by updating the processor and integration tests.

## Scope

1. **note/processor.rs**:
   - Change import from `storage_legacy::Repository` → `repository::Repository`
   - Update generic bounds from `Repository<Error = NoteRepositoryError>` → `Repository`
   - Rename method call: `repository.delete_note(id)` → `repository.delete(id)`

2. **vault/processor.rs**:
   - Change import from `storage_legacy::RedbRepository` → `note::storage::RedbRepository`
   - Use `Arc<Store>` for note repository construction (via bridge from 07b)

3. **tests/note_ingest.rs**:
   - Use `note::storage::RedbRepository` instead of `storage_legacy::RedbRepository`
   - Create `Store` (via bridge) instead of using legacy struct

4. **tests/note_reader.rs**:
   - Same as `note_ingest.rs`

5. **Cleanup**:
   - Remove `note/storage_legacy.rs`
   - Remove `pub mod storage_legacy` from `note/mod.rs`
   - Remove legacy table constants from `note/mod.rs` (replaced by `storage/tables.rs`)

## Dependencies

- **07a** (complete): New repository traits + storage implementation exist
- **07b** (open): Provides `Database` → `Store` bridge for vault processor

## Current State

### note/processor.rs (11 lines affected)

**Import** (line 36):
```rust
use crate::note::storage_legacy::Repository;
```

**Generic bounds** (8 locations):
```rust
Repository<Error = NoteRepositoryError>  // OLD
Repository                                // NEW (no associated Error type)
```

**Method call** (line 340):
```rust
repository.delete_note(note.id())?;  // OLD
repository.delete(note.id())?;        // NEW
```

All other method names are identical between legacy and new traits:
- `find_by_path(path)` ✅
- `save(note)` ✅
- `save_list_view(view)` ✅
- `delete_list_view(note_id)` ✅

### vault/processor.rs (5 occurrences)

**Import** (line 25):
```rust
use crate::note::storage_legacy::RedbRepository as NoteRepository;
```

**Construction** (lines 322, 348):
```rust
let note_repository = NoteRepository::new(db);  // db: &Database
```

After 07b, becomes:
```rust
let store = Arc::new(db.to_store());  // Or From<&Database> bridge
let note_repository = note::storage::RedbRepository::new(store);
```

### tests/note_ingest.rs

**Import** (line 11):
```rust
use lithos_core::note::storage_legacy::{RedbRepository, Repository as _};
```

**Usage** (line 45):
```rust
let db = Database::open(&db_path)?;
let repository = RedbRepository::new(&db);
```

After migration:
```rust
let store = Arc::new(Store::open(&db_path)?);  // Or bridge from Database
let repository = note::storage::RedbRepository::new(Arc::clone(&store));
```

### tests/note_reader.rs

Same pattern as `note_ingest.rs` (lines 43, 93, 148, 305, 336, 387).

## Acceptance Criteria

- [ ] `note/processor.rs` uses `repository::Repository` trait
- [ ] `note/processor.rs` calls `repository.delete(id)` instead of `delete_note(id)`
- [ ] Generic bounds changed from `Repository<Error = NoteRepositoryError>` → `Repository`
- [ ] `vault/processor.rs` uses `note::storage::RedbRepository` with `Arc<Store>`
- [ ] `tests/note_ingest.rs` uses new repository
- [ ] `tests/note_reader.rs` uses new repository
- [ ] All tests pass (`mise run test`)
- [ ] `note/storage_legacy.rs` removed
- [ ] `note/mod.rs` cleaned up (no `storage_legacy` export, no legacy table constants)
- [ ] No clippy warnings (`mise run lint`)

## TDD Approach

Since the processor and tests are already thoroughly tested via integration tests, this is a pure refactor. The workflow:

1. **Green baseline**: Verify all tests pass with legacy code
2. **Refactor**: Make the changes (import, bounds, method rename)
3. **Green verification**: Verify all tests still pass
4. **Cleanup**: Remove legacy module

No new tests needed — existing integration tests verify behavior is preserved.

## Blocked By

- **07b**: Must complete first (provides `Database` → `Store` bridge)

## Estimated Effort

~2 hours (straightforward refactor once 07b provides the bridge)

---

**Status**: 🔴 Blocked by 07b
