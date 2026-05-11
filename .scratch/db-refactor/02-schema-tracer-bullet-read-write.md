---
title: 02-schema-tracer-bullet-read-write
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-10
date_completed: 2026-05-11
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Implement a Schema tracer-bullet vertical slice using the new seam: one complete Schema read path and one complete Schema write path through `schema/repository.rs` (trait), `schema/storage_v2/tables.rs` (table definition), `schema/storage_v2/write.rs` (save_schema impl), and `schema/storage_v2/read.rs` (find_schema_by_id impl), backed by `db::Store` and DB helpers.

This slice demonstrates the new transaction pattern, transparent error handling, and type-safe table wrappers before broader migration.

## Decisions

### 1. Seam Architecture
- **Trait**: `SchemaRepository` defined in `schema/repository.rs`.
- **Adapter**: `SchemaRedbRepository` defined in `schema/storage_v2/mod.rs`.
- **Naming**:
  - Module: `storage_v2` → rename to `storage` after legacy migration
  - Trait: `SchemaRepository` → rename to `Repository` after legacy migration
  - Adapter: `SchemaRedbRepository` → rename to `RedbRepository` after legacy migration
- **Error Type**: `struct SchemaStorageV2Error(DbError)` in `schema/repository.rs` — newtype wrapper over `DbError` to differentiate from domain errors.
- **Sync**: All operations are synchronous (not `async`).
- **Tests**: Unit tests must live in the same file as the code they are testing (standard Rust convention).

### 2. Storage Layout
- **`schema/storage_v2/`** — Temporary directory name (will rename to `storage` after legacy migration).
- `schema/storage_v2/tables.rs`:
    - Defines `const SCHEMAS: UuidTable<SchemaId, &[u8]>` — uses `&[u8]` as value type, serialized via rkyv helpers.
    - Contains `impl_redb_uuid!(SchemaId);` to keep the domain identifier pure from storage-specific trait implementations.
- `schema/storage_v2/mod.rs`: Declares the `SchemaRedbRepository` struct and re-exports modules.
- `schema/storage_v2/read.rs`: Contains `impl SchemaRepository for SchemaRedbRepository` block with `find_schema_by_id` method.
- `schema/storage_v2/write.rs`: Contains `impl SchemaRepository for SchemaRedbRepository` block with `save_schema` method.

### 3. Tracer Bullet Operations
- **Write**: `fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageV2Error>`
- **Read**: `fn find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaStorageV2Error>`

### 4. Implementation Details
- **Constructor**: `SchemaRedbRepository` must provide `pub fn new(store: Arc<Store>) -> Self`.
- **Transaction Pattern**: Logic must be inline in the `impl` block, utilizing `Store::read` and `Store::write` closures directly.

### 5. TDD Implementation Plan

**Phase 1: Define Trait Interface**
- Write test using `SchemaRepository::save_schema` and `find_schema_by_id` — fails because trait doesn't exist.

**Phase 2: Write Path**
- Create `tables.rs` with `impl_redb_uuid!(SchemaId)` and `const SCHEMAS: UuidTable<SchemaId, &[u8]>`.
- Create `mod.rs` with `SchemaRedbRepository { store: Arc<Store> }`.
- Create `write.rs` with `impl SchemaRepository for SchemaRedbRepository` block containing `save_schema` method — uses `store.write(|tx| ...)` with inline rkyv serialization and table insert.

**Phase 3: Read Path**
- Create `read.rs` with `impl SchemaRepository for SchemaRedbRepository` block containing `find_schema_by_id` method — uses `store.read(|tx| ...)` with inline table get and rkyv deserialize.
- Verify roundtrip: save → find → matches.

**Visibility Guidelines**
- Trait methods: `pub` (part of public interface).
- Adapter struct: `pub` (part of public interface).

**Phase 4: Transaction Semantics**
- Test rollback: invalid save fails, previous state preserved.
- Test auto-commit: valid save, reopen Store, find returns data.

**Phase 5: Polish**
- Run `mise run verify`, address clippy warnings, add doc comments.

## Acceptance criteria

- [x] `trait SchemaRepository` exists in `schema/repository.rs` with `save_schema` and `find_schema_by_id`.
- [x] `struct SchemaStorageV2Error(DbError)` defined in `schema/repository.rs`.
- [x] `schema/storage_v2/tables.rs` exists and defines `const SCHEMAS: UuidTable<SchemaId, &[u8]>`.
- [x] `SchemaId` implements required redb traits via `impl_redb_uuid!` within `schema/storage_v2/tables.rs` (not `identifier.rs`).
- [x] `SchemaRedbRepository` struct and `pub fn new(store: Arc<Store>) -> Self` defined in `schema/storage_v2/mod.rs`.
- [x] `impl SchemaRepository for SchemaRedbRepository` in `schema/storage_v2/core.rs` with `save_schema` and `find_schema_by_id`.
- [x] Unit tests for storage logic exist in `schema/storage_v2/core.rs`.
- [x] Implementation is synchronous (no `async` markers on trait or methods).
- [x] Implementation uses `Store::read` and `Store::write` with the rkyv helpers from `db::rkyv`.
- [x] `save_schema` correctly auto-commits and `find_schema_by_id` returns the persisted record.
- [x] Tests validate behavior at the Interface level (exercising `SchemaRedbRepository`) and confirm basic rollback/commit semantics via `Store`.
- [x] `mise run verify` passes with no lint warnings or test failures.

## Implementation Notes

### Deviation from Original Plan
- **File structure**: Instead of `read.rs` and `write.rs` as separate files, implemented `core.rs` containing both read and write implementations in a single module.
- **Reason**: Rust's E0119 error prevents splitting `impl Trait for Type` across multiple files. The delegation pattern (mod.rs re-exports core.rs) was used instead.

### Actual Implementation Structure
```
schema/storage_v2/
├── mod.rs      # Struct declaration + re-exports
├── core.rs     # Full impl block with read + write methods + tests
└── tables.rs   # Table definitions + impl_redb_uuid! macro
```

### What Was NOT Implemented (Out of Scope)
- GetRepository and SetRepository - these are separate future issues

### Verification
- All acceptance criteria met
- `mise run verify` passes with no warnings

## Blocked by

- ✅ `01-lock-db-seam-and-error-classifier.md` (Completed 2026-05-11)
