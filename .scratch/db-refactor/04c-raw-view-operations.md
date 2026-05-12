---
title: 04c-raw-view-operations
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

Migrate remaining Raw View operations for Schema staleness detection from v1 to v2.

Raw views enable staleness detection by tracking file content hashes and timestamps. The batch operation `find_raw_schema_views_by_paths` is already implemented; this issue adds the remaining single-item and write operations.

## Operations to Migrate

### Read Operations (→ `SchemaReadRepository`)
1. **`get_raw_schema_view(id: SchemaId)`** - Get raw view by schema ID
2. **`find_raw_schema_view_by_path(path: &RelativePath)`** - Get raw view by file path (cross-table lookup)

**Already implemented:**
- ✅ `find_raw_schema_views_by_paths(paths)` - Batch lookup (from issue 03)

### Write Operations (→ `SchemaWriteRepository`)
3. **`save_raw_schema_view(id: SchemaId, view: &RawSchemaView)`** - Save raw view with path index update

## Tables Required

**Already exist** in `storage_v2/tables.rs`:
- ✅ `RAW_SCHEMA_VIEWS: UuidTable<SchemaId, &[u8]>` - Views by ID
- ✅ `SCHEMA_ID_BY_PATH: PathTable<&[u8]>` - Path→ID index

## TDD Implementation Plan

### Phase 1: Read by ID
1. RED: Test `get_raw_schema_view(id)` returns None when not saved
2. GREEN: Implement in `read.rs`
3. RED: Test `get_raw_schema_view(id)` returns saved view
4. GREEN: Implement (will pass after Phase 2 write implementation)

### Phase 2: Write with Path Index
1. RED: Test `save_raw_schema_view(id, view)` persists view
2. GREEN: Implement in `write.rs` - single transaction writes to both:
   - `RAW_SCHEMA_VIEWS` (view by ID)
   - `SCHEMA_ID_BY_PATH` (path→ID index from view.path)
3. RED: Test atomic write - if either table fails, both roll back
4. GREEN: Verify transaction boundaries

### Phase 3: Read by Path (Cross-Table)
1. RED: Test `find_raw_schema_view_by_path(path)` returns None when not saved
2. GREEN: Implement in `read.rs`:
   - Lookup ID in `SCHEMA_ID_BY_PATH`
   - If found, lookup view in `RAW_SCHEMA_VIEWS`
   - Return None if either lookup fails
3. RED: Test returns saved view
4. GREEN: Verify (should pass after Phase 2)

### Phase 4: Consistency Verification
1. RED: Test that `save_raw_schema_view()` maintains consistency between path and ID indexes
2. GREEN: Verify path stored in view matches path used for `SCHEMA_ID_BY_PATH`
3. RED: Test that updating view with different path updates index
4. GREEN: Implement (may need path update logic)

## Acceptance Criteria

- [ ] `get_raw_schema_view(id)` added to `SchemaReadRepository`
- [ ] `find_raw_schema_view_by_path(path)` added to `SchemaReadRepository`
- [ ] `save_raw_schema_view(id, view)` added to `SchemaWriteRepository`
- [ ] Implementation in `storage_v2/read.rs` and `storage_v2/write.rs`
- [ ] Unit tests verify:
  - Get by ID returns correct view
  - Find by path performs cross-table lookup correctly
  - Save updates both tables atomically
  - Path in view matches path index entry
  - None returned for missing entries
  - Rollback on error maintains consistency
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md` (batch operation already done)

## Blocks

- `04b-schema-index-operations.md` (coordinates on `SCHEMA_ID_BY_PATH` usage)

## Notes

- `SCHEMA_ID_BY_PATH` is shared between raw views and schema index operations
- Both `save_schema()` (from 04b) and `save_raw_schema_view()` should write to this table
- Need to ensure consistent path format (relative paths, no duplicates)
- The batch operation `find_raw_schema_views_by_paths` already demonstrates the cross-table pattern
