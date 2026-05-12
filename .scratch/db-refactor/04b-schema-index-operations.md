---
title: 04b-schema-index-operations
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Migrate Schema index and lookup operations from v1 to the segregated v2 repository.

These operations provide efficient lookups by name, path, and ID without loading full schema data.

## Operations to Migrate

### Read Operations (→ `SchemaReadRepository`)
1. **`find_schema_id_by_name(name: &SchemaName)`** - Lookup ID by schema name
2. **`find_schema_id_by_path(path: &RelativePath)`** - Lookup ID by file path
3. **`find_schema_ids_by_paths(paths: &[RelativePath])`** - Batch lookup IDs by paths
4. **`list_schema_name_id_pairs()`** - List all name→ID pairs
5. **`list_schema_path_id_pairs()`** - List all path→ID pairs
6. **`get_schema_index()`** - Get unified index (combines name, path, ID lookups)

### Write Operations (→ `SchemaWriteRepository`)
- **Existing `save_schema()` must be updated** to also write to `SCHEMA_ID_BY_NAME` and `SCHEMA_ID_BY_PATH` indexes
- **Existing `save_many_schemas()` must be updated** to maintain all three tables atomically

## Tables Required

Add to `storage_v2/tables.rs`:
```rust
/// Schema name→ID index (key: schema name string, value: serialized SchemaId)
pub const SCHEMA_ID_BY_NAME: PathTable<&[u8]> =
    PathTable::new("schema_id_by_name_v2");
```

**Note**: `SCHEMA_ID_BY_PATH` already exists in `tables.rs`, but it's only used by `find_raw_schema_views_by_paths`. We need to integrate it with schema saves.

## TDD Implementation Plan

### Phase 1: Name Index (Read)
1. RED: Test `find_schema_id_by_name(name)` returns None for unknown name
2. GREEN: Implement in `read.rs` reading from `SCHEMA_ID_BY_NAME`
3. RED: Test `find_schema_id_by_name(name)` returns ID after schema saved
4. GREEN: Update `save_schema()` in `write.rs` to also write to `SCHEMA_ID_BY_NAME`

### Phase 2: Path Index (Integration)
1. RED: Test `find_schema_id_by_path(path)` returns ID after schema with path metadata saved
2. GREEN: Update `save_schema()` to write to `SCHEMA_ID_BY_PATH` (coordinate with existing raw view usage)
3. Ensure `save_raw_schema_view()` (from 04c) and `save_schema()` both write to same table consistently

### Phase 3: Batch Path Lookups
1. RED: Test `find_schema_ids_by_paths(paths)` returns HashMap of path→ID
2. GREEN: Implement in `read.rs` using single transaction
3. Verify ordering and None handling

### Phase 4: List Operations
1. RED: Test `list_schema_name_id_pairs()` returns all name→ID pairs
2. GREEN: Implement in `read.rs` using table iteration
3. RED: Test `list_schema_path_id_pairs()` returns all path→ID pairs
4. GREEN: Implement in `read.rs` using table iteration

### Phase 5: Unified Index
1. RED: Test `get_schema_index()` returns SchemaIndex with all lookups
2. GREEN: Implement by calling list operations and building index
3. Verify O(1) lookups work

### Phase 6: Atomic Updates
1. RED: Test that all three tables (SCHEMAS, SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH) updated atomically
2. GREEN: Verify single transaction in `save_schema()`
3. RED: Test rollback - if any table write fails, none persist
4. GREEN: Verify transaction boundaries

## Acceptance Criteria

- [ ] `find_schema_id_by_name()` added to `SchemaReadRepository`
- [ ] `find_schema_id_by_path()` added to `SchemaReadRepository`
- [ ] `find_schema_ids_by_paths()` added to `SchemaReadRepository`
- [ ] `list_schema_name_id_pairs()` added to `SchemaReadRepository`
- [ ] `list_schema_path_id_pairs()` added to `SchemaReadRepository`
- [ ] `get_schema_index()` added to `SchemaReadRepository`
- [ ] `SCHEMA_ID_BY_NAME` table added to `storage_v2/tables.rs`
- [ ] `save_schema()` updated to maintain all three tables atomically
- [ ] `save_many_schemas()` updated to maintain all three tables atomically
- [ ] Unit tests verify:
  - Lookups return correct IDs
  - None returned for missing entries
  - All three tables updated atomically
  - Rollback on error preserves consistency
  - List operations return complete data
  - Index provides O(1) lookups
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md`
- `04a-property-bank-migration.md` (should complete first to prove pattern)

## Blocks

- `04e-remaining-schema-operations.md`

## Notes

- This adds index maintenance to existing `save_schema()` operations
- Requires coordinating with raw view path storage (from 04c)
- The `SchemaIndex` type is a convenience wrapper; core functionality is the list operations
