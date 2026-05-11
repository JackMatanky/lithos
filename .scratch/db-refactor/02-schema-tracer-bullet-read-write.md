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

Implement a Schema tracer-bullet vertical slice using the new seam: one complete Schema read path and one complete Schema write path through `schema/repository.rs`, `schema/storage/read.rs`, `schema/storage/write.rs`, and `schema/storage/tables.rs`, backed by `db::Store` and DB helpers.

This slice demonstrates the new transaction pattern, transparent error handling, and type-safe table wrappers before broader migration.

## Decisions

### 1. Seam Architecture
- **Trait**: `SchemaRepository` defined in `schema/repository.rs`.
- **Adapter**: `SchemaRedbRepository` defined in `schema/storage/mod.rs`.
- **Naming**: Using `Schema*` prefix temporarily to avoid conflicts with existing `Repository` / `RedbRepository` in `schema/storage.rs`. These will be renamed back after the legacy implementation is deleted in Issue-09.

### 2. Storage Layout
- `schema/storage/tables.rs`:
    - Defines `const SCHEMAS: UuidTable<SchemaId, Schema>`.
    - Contains `impl_redb_uuid!(SchemaId);` to keep the domain identifier pure from storage-specific trait implementations.
- `schema/storage/read.rs`: Implements read logic using `Store::read`.
- `schema/storage/write.rs`: Implements write logic using `Store::write`.

### 3. Tracer Bullet Operations
- **Write**: `save_schema(&self, schema: &Schema) -> Result<(), Error>`
- **Read**: `find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Error>`

## Acceptance criteria

- [ ] `trait SchemaRepository` exists in `schema/repository.rs` with `save_schema` and `find_schema_by_id`.
- [ ] `schema/storage/tables.rs` exists and defines `const SCHEMAS: UuidTable<SchemaId, Schema>`.
- [ ] `SchemaId` implements required redb traits via `impl_redb_uuid!` within `schema/storage/tables.rs` (not `identifier.rs`).
- [ ] `SchemaRedbRepository` adapter exists in `schema/storage/mod.rs` and delegates to implementation modules.
- [ ] Implementation uses `Store::read` and `Store::write` with the rkyv helpers from `db::rkyv`.
- [ ] `save_schema` correctly auto-commits and `find_schema_by_id` returns the persisted record.
- [ ] Tests validate behavior at the Interface level (exercising `SchemaRedbRepository`) and confirm basic rollback/commit semantics via `Store`.
- [ ] `mise run verify` passes with no lint warnings or test failures.

## Blocked by

- ✅ `01-lock-db-seam-and-error-classifier.md` (Completed 2026-05-11)
