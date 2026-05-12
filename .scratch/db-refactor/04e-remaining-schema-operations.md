---
title: 04e-remaining-schema-operations
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Migrate remaining Schema aggregate operations that don't fit in other 04x issues.

These are operations on full `Schema` aggregates (not just indexes or views) plus schema deletion and property usage queries.

## Operations to Migrate

### Read Operations (→ `SchemaReadRepository`)
1. **`find_schemas_by_ids(ids: &[SchemaId])`** - Find multiple schemas (returns `Vec<Schema>`, skips missing)
2. **`list_schemas()`** - List all schema aggregates
3. **`find_schemas_using_properties(property_names: &[PropertyName])`** - Find schemas that reference given properties

**Already implemented:**
- ✅ `find_schema_by_id(id)` - Single schema lookup (from issue 02)
- ✅ `find_many_schemas_by_id(ids)` - Batch lookup with `Vec<Option<Schema>>` (from issue 03)

### Write Operations (→ `SchemaWriteRepository`)
4. **`delete_schema(id: SchemaId)`** - Delete schema and all its indexes

**Already implemented:**
- ✅ `save_schema(schema)` - Save single schema (from issue 02)
- ✅ `save_many_schemas(schemas)` - Batch save (from issue 03)

**Note on signatures:**
- v1 has `save_schemas(&[&Schema])`
- v2 has `save_many_schemas(&[Schema])`
- These are equivalent; v2 signature is cleaner (no double reference)

## TDD Implementation Plan

### Phase 1: Find Schemas by IDs (Skip Missing)
1. RED: Test `find_schemas_by_ids(ids)` returns empty Vec for no matches
2. GREEN: Implement in `read.rs` - filter out None from `find_many_schemas_by_id`
3. RED: Test returns only found schemas (skips missing IDs)
4. GREEN: Verify passes

### Phase 2: List All Schemas
1. RED: Test `list_schemas()` returns empty Vec when no schemas saved
2. GREEN: Implement in `read.rs` - iterate `SCHEMAS` table
3. RED: Test returns all saved schemas
4. GREEN: Verify passes

### Phase 3: Find Schemas Using Properties
1. RED: Test `find_schemas_using_properties(props)` returns empty map when no schemas use properties
2. GREEN: Implement in `read.rs`:
   - Load all schemas (via `list_schemas()`)
   - For each schema, check if it uses any of the given properties
   - Return `HashMap<SchemaId, Vec<PropertyName>>` of matches
3. RED: Test returns correct schema→property mappings
4. GREEN: Verify passes

**Note**: This is a scan operation (no index). If performance becomes an issue, can add property→schema index later.

### Phase 4: Delete Schema
1. RED: Test `delete_schema(id)` removes schema
2. GREEN: Implement in `write.rs` - delete from `SCHEMAS` table
3. RED: Test delete also removes from indexes (`SCHEMA_ID_BY_NAME`, `SCHEMA_ID_BY_PATH`)
4. GREEN: Update implementation to delete from all three tables atomically
5. RED: Test delete removes raw view (`RAW_SCHEMA_VIEWS`)
6. GREEN: Update to delete from four tables atomically
7. RED: Test atomic delete - if any delete fails, none succeed
8. GREEN: Verify transaction boundaries

### Phase 5: Delete Coordination
1. Verify delete operation removes entries from all relevant tables:
   - `SCHEMAS` (main aggregate)
   - `SCHEMA_ID_BY_NAME` (name index)
   - `SCHEMA_ID_BY_PATH` (path index)
   - `RAW_SCHEMA_VIEWS` (raw view)
2. Test that `list_schemas()` doesn't return deleted schemas
3. Test that index lookups return None after deletion

## Acceptance Criteria

- [ ] `find_schemas_by_ids(ids)` added to `SchemaReadRepository`
- [ ] `list_schemas()` added to `SchemaReadRepository`
- [ ] `find_schemas_using_properties(property_names)` added to `SchemaReadRepository`
- [ ] `delete_schema(id)` added to `SchemaWriteRepository`
- [ ] Implementation in `storage_v2/read.rs` and `storage_v2/write.rs`
- [ ] Unit tests verify:
  - `find_schemas_by_ids` skips missing schemas
  - `list_schemas` returns all schemas
  - `find_schemas_using_properties` returns correct mappings
  - `delete_schema` removes from all four tables atomically
  - Deleted schemas not returned by list/find operations
  - Index lookups return None after deletion
  - Rollback on error maintains consistency
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted

## Blocked by

- `04b-schema-index-operations.md` (delete needs to clean indexes)
- `04c-raw-view-operations.md` (delete needs to remove raw views)

## Blocks

- `04f-batch-reader-migration.md` (optional cleanup)

## Notes

- `find_schemas_by_ids` is similar to `find_many_schemas_by_id` but filters out None values
- `find_schemas_using_properties` is a scan operation; may be slow with many schemas
- Delete operation must coordinate across all tables that reference schemas
- After this issue, all core schema operations are migrated to v2
