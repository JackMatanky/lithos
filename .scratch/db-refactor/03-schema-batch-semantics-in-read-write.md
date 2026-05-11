---
title: 03-schema-batch-semantics-in-read-write
category: enhancement
label: ready-for-agent
status: closed
date_created: 2026-05-10
date_completed: 2026-05-11
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Add batch read and batch write semantics to `schema/storage_v2/core.rs`.

**Batch** means performing multiple operations in a single transaction (without looping), not necessarily multi-table operations.

### Phase 1: Single-table batch operations

Operations on `SCHEMAS` table only:

- **Batch write**: Save multiple schemas in one transaction (`save_many_schemas`)
- **Batch read**: Find multiple schemas by ID in one transaction (`find_many_schemas_by_id`)

### Phase 2: Multi-table batch operations (future)

Operations that join across tables in a single transaction:

- Add to tables.rs:
  - `SCHEMA_ID_BY_PATH: PathTable<&[u8]>` - path (string key) to SchemaId mapping
    - Note: Update PathTable to be generic over Path/PathBuf at construction ("parse, don't validate") before this phase
  - `RAW_SCHEMA_VIEWS: UuidTable<SchemaId, &[u8]>` - raw views by schema ID (serialized)
- Example batch operation: given `&[RelativePath]`, lookup each in SCHEMA_ID_BY_PATH, then fetch corresponding RAW_SCHEMA_VIEWS in same transaction

This requires adding tables to storage_v2 first, then implementing the cross-table batch read.

## Acceptance criteria

### Phase 1 (this issue)

**Trait methods:**

```rust
/// Save multiple schemas in a single transaction.
/// Returns error if any schema fails (atomic rollback).
fn save_many_schemas(&self, schemas: &[Schema]) -> Result<(), SchemaStorageV2Error>;

/// Find multiple schemas by ID in a single transaction.
/// Returns Vec in same order as input IDs - None for missing, Some(Schema) for found.
fn find_many_schemas_by_id(&self, ids: &[SchemaId]) -> Result<Vec<Option<Schema>>, SchemaStorageV2Error>;
```

**Implementation:**
- Uses single `store.write(|tx| ...)` for all schemas
- Atomic: if any schema fails to serialize, entire batch rolls back
- Uses single `store.read(|tx| ...)` for all lookups

**Tests to implement:**
- Empty batch (no-op, succeeds)
- All schemas found
- None found (all return None)
- Partial found (mix of Some/None)
- Rollback: failed serialization prevents any writes
- Persistence: valid batch, reopen store, schemas still retrievable

- [x] `save_many_schemas` method added to `SchemaRepository` trait
- [x] Implementation uses single write transaction for all schemas
- [x] Atomic commit/rollback: if any schema fails to serialize, no schemas are persisted
- [x] `find_many_schemas_by_id` method added to `SchemaRepository` trait
- [x] Implementation uses single read transaction for all lookups
- [x] Unit tests verify batch semantics (commit/rollback behavior)

### Phase 2 (out of scope - separate issue)

- Add SCHEMA_ID_BY_PATH and RAW_SCHEMA_VIEWS tables to tables.rs
- Implement cross-table batch read: path → SchemaId → RawSchemaView

## Blocked by

- ✅ `02-schema-tracer-bullet-read-write.md` (Completed 2026-05-11)

## Notes

- Updated paths from old `schema/storage/read.rs` to actual `schema/storage_v2/core.rs`
- Split from multi-table batch to keep scope manageable
- Multi-table batch will be a follow-up issue after this one establishes the pattern
