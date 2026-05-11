---
title: 02-schema-tracer-bullet-read-write
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-10
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
- **Error Type**: `struct SchemaStorageError(DbError)` — newtype wrapper over `DbError`, expandable later.

### 2. Storage Layout
- **`schema/storage_v2/`** — Temporary directory name (will rename to `storage` after legacy migration).
- `schema/storage_v2/tables.rs`:
    - Defines `const SCHEMAS: UuidTable<SchemaId, &[u8]>` — uses `&[u8]` as value type, serialized via rkyv helpers.
    - Contains `impl_redb_uuid!(SchemaId);` to keep the domain identifier pure from storage-specific trait implementations.
- `schema/storage_v2/mod.rs`: Declares the `SchemaRedbRepository` struct and re-exports modules.
- `schema/storage_v2/read.rs`: Contains `impl SchemaRepository for SchemaRedbRepository` block with `find_schema_by_id` method.
- `schema/storage_v2/write.rs`: Contains `impl SchemaRepository for SchemaRedbRepository` block with `save_schema` method.

### 3. Tracer Bullet Operations
- **Write**: `save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageError>`
- **Read**: `find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaStorageError>`

### 4. TDD Implementation Plan

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

- [ ] `trait SchemaRepository` exists in `schema/repository.rs` with `save_schema` and `find_schema_by_id`.
- [ ] `schema/storage_v2/tables.rs` exists and defines `const SCHEMAS: UuidTable<SchemaId, &[u8]>`.
- [ ] `SchemaId` implements required redb traits via `impl_redb_uuid!` within `schema/storage_v2/tables.rs` (not `identifier.rs`).
- [ ] `SchemaRedbRepository` struct defined in `schema/storage_v2/mod.rs`.
- [ ] `impl SchemaRepository for SchemaRedbRepository` in `schema/storage_v2/write.rs` with `save_schema`.
- [ ] `impl SchemaRepository for SchemaRedbRepository` in `schema/storage_v2/read.rs` with `find_schema_by_id`.
- [ ] Implementation uses `Store::read` and `Store::write` with the rkyv helpers from `db::rkyv`.
- [ ] `save_schema` correctly auto-commits and `find_schema_by_id` returns the persisted record.
- [ ] Tests validate behavior at the Interface level (exercising `SchemaRedbRepository`) and confirm basic rollback/commit semantics via `Store`.
- [ ] `mise run verify` passes with no lint warnings or test failures.

## Blocked by

- ✅ `01-lock-db-seam-and-error-classifier.md` (Completed 2026-05-11)
