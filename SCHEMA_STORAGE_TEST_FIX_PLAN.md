# Schema Storage Test Fix Plan

## Overview

Two integration tests in `schema_storage.rs` are currently ignored and need fixing:
1. `schema_list` - rkyv deserialization error (technical limitation)
2. `schema_delete` - Not implemented (unimplemented!())

This document analyzes both issues and provides actionable fix plans.

---

## Issue 1: schema_list - rkyv Deserialization Error

### Current Status
- **Test**: `schema_list`
- **Status**: Ignored with `#[ignore = "rkyv address space limitation - requires CLI-level e2e test"]`
- **Error**: `Storage(Deserialization("subtree pointer overran range: ptr 0x00000008ccc1c19e size 4294967295 in range 0x00000008ccc1c100..0x00000008ccc1c19f"))`

### Root Cause Analysis

**The Error Message Decoded**:
- "subtree pointer overran range" = rkyv pointer validation failed
- `ptr 0x00000008ccc1c19e` = The archived pointer being accessed
- `size 4294967295` = Size field (0xFFFFFFFF) - this is suspiciously max u32!
- `range 0x00000008ccc1c100..0x00000008ccc1c19f` = Valid memory range (only 159 bytes!)

**This is NOT an address space limitation issue** - it's a data corruption issue!

The size field being `4294967295` (max u32) indicates:
1. Uninitialized memory being read as a size
2. Buffer overflow writing garbage over valid rkyv metadata
3. Wrong rkyv serialization/deserialization version
4. Memory safety issue in our rkyv usage

### Investigation Steps

1. **Check if it's a test-specific issue**:
   ```bash
   # Run the test in isolation
   cargo nextest run --package lithos-core --test schema_storage schema_list -- --include-ignored
   ```

2. **Check the actual bytes being written**:
   - Add debug logging in `save_schemas()` to see what's being serialized
   - Check if both schemas serialize correctly individually
   - Verify batch save doesn't corrupt data

3. **Check redb table definition**:
   - Verify `SCHEMA_BY_ID` table uses correct Value encoding
   - Check if there's a mismatch between write/read encoding

4. **Check rkyv alignment**:
   - Schema type has nested HashMap - check alignment requirements
   - Verify rkyv derives on Schema and all nested types

### Potential Fixes (Priority Order)

#### Fix Option A: Use TestDb::reopen() Pattern ✅ **RECOMMENDED**
**Hypothesis**: The issue is similar to the `staleness_persists_across_reopens` test - we're trying to deserialize data in the same transaction/session where it was written.

**Solution**:
```rust
#[test]
fn schema_list() -> TestResult {
    let mut test_db = TestDb::new()?;

    // FIRST SESSION: Save schemas
    {
        let repository = setup_repository(test_db.db());

        let prop1 = PropertyBuilder::new("title").build_string_default()?;
        let mut props1 = HashMap::new();
        props1.insert(prop1.name().clone(), prop1);
        let schema1 = Schema::new(/*...*/);

        let prop2 = PropertyBuilder::new("content").build_string_default()?;
        let mut props2 = HashMap::new();
        props2.insert(prop2.name().clone(), prop2);
        let schema2 = Schema::new(/*...*/);

        repository.save_schemas(&[schema1, schema2])?;
    }; // Drop repository, flush writes

    // SECOND SESSION: List schemas (fresh database handle)
    let fresh_db = test_db.reopen()?;
    let repository2 = setup_repository(&fresh_db);
    let all = repository2.list_schemas()?;

    assert_eq!(all.len(), 2, "Should have 2 schemas");
    Ok(())
}
```

**Why this might work**:
- Reopening the database forces redb to flush all writes
- Fresh database handle = fresh memory mappings
- Avoids any potential transaction/session state issues

#### Fix Option B: Use Separate Transactions
**Solution**: Save in one transaction, read in another:
```rust
// Save in one scope
{
    let repository = setup_repository(test_db.db());
    repository.save_schemas(&[schema1, schema2])?;
} // Transaction committed here

// Read in another scope
{
    let repository = setup_repository(test_db.db());
    let all = repository.list_schemas()?;
    assert_eq!(all.len(), 2);
}
```

#### Fix Option C: Save Schemas Individually
**Solution**: Save one at a time, not in batch:
```rust
repository.save_schemas(&[schema1])?;
repository.save_schemas(&[schema2])?;
let all = repository.list_schemas()?;
```

**Why this might work**: Batch save might have a bug in the iteration logic.

#### Fix Option D: Investigate Actual Bug
If all quick fixes fail, we need to debug the actual issue:
1. Add logging to `RedbRepository::save_schemas()`
2. Add logging to `RedbRepository::list_schemas()`
3. Compare serialized bytes between single-save and batch-save
4. Check if there's an off-by-one error in batch processing

### Decision: Try Fix Option A First

**Rationale**:
- Lowest risk (just test structure change, no production code change)
- Similar pattern worked for `staleness_persists_across_reopens`
- If it works, the issue was transaction/session related
- If it fails, we can try Options B, C, D in sequence

---

## Issue 2: schema_delete - Not Implemented

### Current Status
- **Test**: `schema_delete`
- **Status**: Ignored with `#[ignore = "delete_schema not yet implemented"]`
- **Implementation**: `unimplemented!("Schema deletion with proper cleanup of all references")`

### Implementation Requirements

From the comment in `storage.rs`, schema deletion requires:
1. Load schema to get its name
2. Delete from `SCHEMA_BY_ID` table
3. Delete from `SCHEMA_ID_BY_NAME` table
4. Delete from `SCHEMA_PARENT` table (inheritance parent)
5. Remove from `SCHEMA_CHILDREN` multimap entries (inheritance children)

### Implementation Plan

#### Step 1: Implement `delete_schema()` in RedbRepository

```rust
fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error> {
    // 1. Load schema to get its name and inheritance info
    let schema = self.find_schema_by_id(id)?
        .ok_or_else(|| SchemaError::NotFound(id))?;

    let name = schema.name();
    let parent_id = schema.extends();

    // 2. Delete from SCHEMA_BY_ID
    Writer::delete(&self.db, SCHEMA_BY_ID, id.as_uuid())?;

    // 3. Delete from SCHEMA_ID_BY_NAME
    Writer::delete(&self.db, SCHEMA_ID_BY_NAME, name.as_ref())?;

    // 4. Delete from SCHEMA_PARENT (if has parent)
    if parent_id.is_some() {
        Writer::delete(&self.db, SCHEMA_PARENT, id.as_uuid())?;
    }

    // 5. Remove from SCHEMA_CHILDREN multimap entries
    // Note: This only removes THIS schema as a child of its parent
    // It does NOT handle schemas that inherit from THIS schema
    if let Some(parent) = parent_id {
        Writer::multimap_remove(&self.db, SCHEMA_CHILDREN, parent.as_uuid(), id.as_uuid())?;
    }

    Ok(())
}
```

#### Step 2: Handle Cascading Deletes (Optional)

**Question**: What should happen if you delete a schema that has children?

**Option A**: Reject deletion (safest)
```rust
// Before deletion, check if schema has children
let children = self.list_inheritance_children(id)?;
if !children.is_empty() {
    return Err(SchemaError::CannotDeleteSchemaWithChildren {
        schema_id: id,
        child_count: children.len()
    });
}
```

**Option B**: Cascade delete (dangerous)
- Delete all children recursively
- Could accidentally delete many schemas

**Option C**: Orphan children (make them standalone)
- Remove parent reference from all children
- Children become standalone schemas

**Recommendation**: Start with **Option A** (reject) for safety. We can add cascade/orphan options later if needed.

#### Step 3: Add Error Variant to SchemaError

```rust
// In schema/error.rs
pub enum SchemaError {
    // ... existing variants ...

    /// Schema not found by ID.
    #[error("Schema not found: {0}")]
    NotFound(SchemaId),

    /// Cannot delete schema with children.
    #[error("Cannot delete schema {schema_id} - it has {child_count} child schema(s)")]
    CannotDeleteSchemaWithChildren {
        schema_id: SchemaId,
        child_count: usize,
    },
}
```

#### Step 4: Delete RawSchemaView (Staleness Tracking)

Don't forget to clean up the staleness tracking metadata:

```rust
fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error> {
    // ... existing deletion logic ...

    // Delete RawSchemaView (for staleness tracking)
    Writer::delete(&self.db, RAW_SCHEMA_VIEW_BY_ID, id.as_uuid())?;

    Ok(())
}
```

#### Step 5: Update Test

The test is already correct! We just need to:
1. Remove the `#[ignore]` attribute
2. Run the test to verify it works

---

## Implementation Order

### Phase 1: Fix schema_list (Quick Win)
1. Try Fix Option A (reopen pattern) - **5 minutes**
2. If fails, try Fix Option B (separate transactions) - **5 minutes**
3. If fails, try Fix Option C (individual saves) - **5 minutes**
4. If all fail, investigate actual bug - **30-60 minutes**

### Phase 2: Implement delete_schema (Core Feature)
1. Add `NotFound` error variant - **2 minutes**
2. Implement basic deletion (steps 1-4) - **15 minutes**
3. Add children check (reject if has children) - **10 minutes**
4. Delete RawSchemaView - **2 minutes**
5. Remove `#[ignore]` from test - **1 minute**
6. Verify test passes - **2 minutes**

**Total Estimated Time**: 45-90 minutes

---

## Success Criteria

### schema_list Test
- ✅ Test runs without `#[ignore]` attribute
- ✅ Test passes consistently
- ✅ No rkyv deserialization errors
- ✅ Returns correct number of schemas (2)

### schema_delete Test
- ✅ Test runs without `#[ignore]` attribute
- ✅ Test passes consistently
- ✅ Schema is deleted from all tables
- ✅ Find by ID returns None after deletion
- ✅ Find by name returns None after deletion

### Overall
- ✅ **15/15 integration tests passing** (13 current + 2 fixed)
- ✅ **0 ignored tests**
- ✅ **100% integration test pass rate**

---

## Risk Assessment

### schema_list Fix
- **Risk**: LOW if using reopen pattern (just test structure change)
- **Risk**: MEDIUM if bug is in production code (requires careful investigation)

### delete_schema Implementation
- **Risk**: LOW (straightforward CRUD operation)
- **Complexity**: MEDIUM (multiple tables to update)
- **Testing**: Already have test case, easy to verify

---

## Rollback Plan

If fixes fail or introduce new issues:

### schema_list
- **Revert**: Keep test ignored, document as "requires separate process"
- **Alternative**: Move to CLI e2e test suite (separate repo/process)

### schema_delete
- **Revert**: Keep test ignored, remove implementation
- **Impact**: Delete is not a critical feature for initial release
- **Alternative**: Implement in future release when needed

---

## Next Steps

1. **Immediate**: Try schema_list Fix Option A (reopen pattern)
2. **Next**: Implement delete_schema with safety checks
3. **Verify**: Run full test suite (should be 15/15 passing)
4. **Document**: Update PHASE_6_3_STATUS.md with final results
5. **Commit**: "fix(schema): enable schema_list and implement delete_schema (15/15 tests)"
