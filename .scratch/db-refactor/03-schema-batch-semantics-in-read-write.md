---
title: 03-schema-batch-semantics-in-read-write
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-10
date_completed: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Add batch read and batch write semantics to `schema/storage_v2/`.

**REOPENED (2026-05-12)**: Following the segregation of `SchemaRepository` into `SchemaReadRepository` and `SchemaWriteRepository`, batch operations must now be implemented in their respective segregated interfaces within `read.rs` and `write.rs`.

## Agent Brief (v2 - 2026-05-12)

**Category:** enhancement
**Summary:** Implement batch operations in segregated Read/Write repository traits.

**Current behavior:**
Batch operations were implemented in a unified `SchemaRepository` trait within a monolithic `core.rs`.

**Desired behavior:**
1. `SchemaReadRepository` gains `find_many_schemas_by_id` and `find_raw_schema_views_by_paths`.
2. `SchemaWriteRepository` gains `save_many_schemas`.
3. Implementations must reside in `read.rs` and `write.rs` respectively.

**Key interfaces:**
- `SchemaReadRepository` / `SchemaWriteRepository` - where methods are added.
- `SchemaRedbRepository` - the implementation struct.

**Acceptance criteria:**
- [x] `save_many_schemas` added to `SchemaWriteRepository` in `repository.rs`.
- [x] `find_many_schemas_by_id` added to `SchemaReadRepository` in `repository.rs`.
- [x] `find_raw_schema_views_by_paths` added to `SchemaReadRepository` in `repository.rs`.
- [x] Implementations moved to `storage_v2/read.rs` and `storage_v2/write.rs`.
- [x] Tests in `read.rs` and `write.rs` verify batch semantics and atomicity.

**Refactor Reason:**
To maintain consistency with the new segregated trait pattern and to ensure that batch operations reside with their corresponding read/write implementations in the split file structure.

## Implementation Notes (v2 - Segregated Traits, 2026-05-12)

### What Was Implemented

**Batch Write Operations** (`storage_v2/write.rs`):
- `save_many_schemas(&self, schemas: &[Schema])` - atomic batch write in single transaction
- Tests verify: empty batch succeeds, all-or-nothing atomicity, persistence after reopen
- Implementation uses single `store.write(|tx| ...)` for all schemas with rollback on any error

**Batch Read Operations** (`storage_v2/read.rs`):
- `find_many_schemas_by_id(&self, ids: &[SchemaId])` - batch lookup in single transaction
  - Returns `Vec<Option<Schema>>` in same order as input IDs
  - Tests verify: empty batch, all found, none found, partial found (mix of Some/None)
- `find_raw_schema_views_by_paths(&self, paths: &[RelativePath])` - cross-table batch read
  - Single transaction: path → SchemaId → RawSchemaView lookup
  - Returns `Vec<Option<RawSchemaView>>` in same order as input paths
  - Uses `SCHEMA_ID_BY_PATH` and `RAW_SCHEMA_VIEWS` tables

**Key Decisions:**
- All batch operations use single transaction for atomicity
- Return type is `Vec<Option<T>>` (not HashMap) to preserve input ordering
- Empty batch operations are no-ops that succeed
- Rollback behavior tested: failed serialization prevents any writes

### Verification
- All batch tests pass with comprehensive coverage of edge cases
- Atomicity verified via rollback tests
- Cross-table batch read correctly chains lookups in single transaction
- Same commit as issue 02: `9f4cb8da` includes all batch operations

---

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

### Phase 2 (multi-table batch operations)

**Tables to add:**
- `SCHEMA_ID_BY_PATH: PathTable<&[u8]>` - path (string key) to SchemaId mapping
  - Note: Update PathTable to be generic over Path/PathBuf at construction ("parse, don't validate")
- `RAW_SCHEMA_VIEWS: UuidTable<SchemaId, &[u8]>` - raw views by schema ID (serialized)

**Batch operation to implement:**
- Given `&[RelativePath]`, lookup each in SCHEMA_ID_BY_PATH, then fetch corresponding RAW_SCHEMA_VIEWS in same transaction

**Acceptance criteria:**
- [x] PathTable updated to use `String` keys (not `&'static str`) for runtime paths
- [x] SCHEMA_ID_BY_PATH table added to tables.rs
- [x] RAW_SCHEMA_VIEWS table added to tables.rs
- [x] Batch method for cross-table read: paths → schema IDs → raw views
- [x] Tests verify single transaction, atomic behavior, correct ordering
- [x] Return type changed from HashMap to Vec<Option<T>> for consistency with Phase 1

## Blocked by

- ✅ `02-schema-tracer-bullet-read-write.md` (Completed 2026-05-11)

## Notes

- Updated paths from old `schema/storage/read.rs` to actual `schema/storage_v2/core.rs`
- Split from multi-table batch to keep scope manageable
- Multi-table batch will be a follow-up issue after this one establishes the pattern
