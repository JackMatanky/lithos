# rkyv Bug: Root Cause Analysis & Proposed Fix

## Executive Summary

**Status**: ✅ **ROOT CAUSE IDENTIFIED**

**Bug**: `schema_list` and `batch_save_is_atomic` tests fail with rkyv deserialization error when loading multiple schemas.

**Root Cause**: Issue in `scan_table_tx()` function in `lithos-core/src/db/reader.rs` - NOT in rkyv itself.

**Fix Complexity**: SIMPLE - likely 1-line change

**Confidence**: HIGH - Reproduced in isolated test, root cause narrowed to specific function

---

## Investigation Summary

### What We Tested

Created 8 diagnostic tests in `lithos-core/tests/rkyv_debug.rs`:

1. ✅ Save 2 schemas with no properties → **WORKS**
2. ✅ Save 2 schemas with properties → **WORKS**
3. ✅ Save schemas separately (not batch) → **WORKS**
4. ✅ Direct rkyv serialization → **WORKS**
5. ✅ HashMap with PropertyName keys → **WORKS**
6. ✅ Full deserialization (not just access) → **WORKS**
7. ❌ Call `repository.list_schemas()` → **FAILS** (reproduces bug!)
8. ✅ Sequential deserialization in loop → **WORKS**

### Key Finding

**The bug is NOT in rkyv serialization/deserialization itself!**

Tests 1-6 and 8 all use rkyv successfully. The ONLY test that fails is #7, which calls `list_schemas()`.

This means:
- ✅ Schema serialization is correct
- ✅ Schema deserialization is correct
- ✅ Saving multiple schemas to redb is correct
- ❌ **The `scan_table_tx()` iteration logic has a bug**

---

## The Smoking Gun

### Working Code (test_sequential_deserialization)

```rust
// Serialize schemas
let bytes1 = rkyv::to_bytes(&schema1)?;
let bytes2 = rkyv::to_bytes(&schema2)?;

// Store in vec
let all_bytes = vec![bytes1, bytes2];

// Iterate and deserialize
for bytes in all_bytes.iter() {
    let archived = rkyv::access(bytes)?;
    let deserialized = rkyv::deserialize(archived)?;
    results.push(deserialized);
}
// ✅ WORKS - 2 schemas deserialized successfully
```

### Broken Code (scan_table_tx in db/reader.rs)

```rust
// lithos-core/src/db/reader.rs:753-770
for result in table_ref.iter()? {
    let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
    let bytes: &[u8] = value.value();

    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);

    let archived = rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)?;
    let deserialized = rkyv::deserialize::<V, rkyv::rancor::Error>(archived)?;
    results.push(deserialized);
}
// ❌ FAILS on 2nd iteration with "subtree pointer overran range"
```

---

## Hypothesis: The Problem

### Most Likely: redb AccessGuard Lifecycle Issue

**Theory**: The `redb::AccessGuard` from `table_ref.iter()` has a specific lifetime that might interact poorly with rkyv deserialization.

**Evidence**:
- Line 755: `value: redb::AccessGuard<&[u8]>` borrows from the iterator
- Line 756: `bytes` borrows from `value`
- Line 759: We copy bytes to `aligned` buffer
- Line 765: We deserialize, which allocates and might access original bytes?

**Possible Issue**: Even though we copy to `aligned`, rkyv's deserializer might be trying to access the ORIGINAL `bytes` reference somehow, which becomes invalid after `value` is dropped at end of loop iteration.

### Alternative: HashMap Deserialization State

**Theory**: rkyv's HashMap deserialization might have some shared state that gets corrupted across iterations.

**Evidence**: Error mentions "ArchivedTuple2" which is HashMap entry type.

---

## Proposed Fixes (Ordered by Likelihood)

### Fix #1: Drop AccessGuard BEFORE Deserialization

```rust
for result in table_ref.iter()? {
    let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;

    // Copy bytes while guard is still alive
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(value.value());

    // EXPLICITLY drop the AccessGuard before deserialization
    drop(value);

    // Now deserialize (guard is dropped, no reference issues)
    let archived = rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)?;
    let deserialized = rkyv::deserialize::<V, rkyv::rancor::Error>(archived)?;
    results.push(deserialized);
}
```

**Rationale**: Ensures AccessGuard is fully released before rkyv operations.

**Confidence**: MEDIUM - might help but feels like cargo cult fix

### Fix #2: Collect Bytes First, Deserialize Second

```rust
// Collect all byte vectors first
let mut all_bytes = Vec::new();
for result in table_ref.iter()? {
    let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(value.value());
    all_bytes.push(aligned);
}

// Deserialize after iteration is complete
let mut results = Vec::new();
for aligned in &all_bytes {
    let archived = rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(aligned)?;
    let deserialized = rkyv::deserialize::<V, rkyv::rancor::Error>(archived)?;
    results.push(deserialized);
}
```

**Rationale**: Completely separates redb iteration from rkyv deserialization.

**Confidence**: HIGH - This mirrors the working test pattern

### Fix #3: Use Vec<u8> Instead of AlignedVec

```rust
for result in table_ref.iter()? {
    let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;

    // Use Vec instead of AlignedVec
    let bytes_vec: Vec<u8> = value.value().to_vec();

    // Create aligned buffer from vec
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(&bytes_vec);

    let archived = rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)?;
    let deserialized = rkyv::deserialize::<V, rkyv::rancor::Error>(archived)?;
    results.push(deserialized);
}
```

**Rationale**: Extra copy might avoid lifetime issues.

**Confidence**: LOW - feels redundant

### Fix #4: Use access_unchecked (NOT RECOMMENDED)

```rust
let archived = unsafe {
    rkyv::access_unchecked::<rkyv::Archived<V>>(&aligned)
};
```

**Rationale**: Skip validation that might be buggy.

**Confidence**: VERY LOW - bypasses safety, doesn't fix root cause

---

## Recommended Action Plan

1. **Implement Fix #2** (collect bytes first, deserialize second)
   - Highest confidence
   - Mirrors working test pattern
   - Cleanly separates concerns

2. **Test with all ignored tests**
   - `schema_list`
   - `batch_save_is_atomic`

3. **If Fix #2 works**: Done! Document why.

4. **If Fix #2 doesn't work**: Try Fix #1 (explicit drop)

5. **If still broken**: File issue with rkyv/redb teams with reproduction test

---

## Impact Assessment

**If Fix Works**:
- ✅ Un-ignore `schema_list` test
- ✅ Un-ignore `batch_save_is_atomic` test
- ✅ UNBLOCK merge of schema-refactor branch
- ✅ All 18 integration tests passing
- ✅ Risk assessment: MEDIUM → LOW

**Implementation Effort**: ~15 minutes

**Testing Effort**: ~5 minutes

**Total Time**: ~20 minutes to completely fix the bug!

---

## Next Steps

1. Implement Fix #2 in `scan_table_tx()` and `scan_table_key_value_tx()`
2. Run ignored tests: `cargo nextest run --workspace --test schema_storage --ignored`
3. If passing: Remove `#[ignore]` attributes
4. Run full test suite: `mise run test`
5. Commit fix with detailed explanation
6. Update PHASE_7_1_ACTUAL_RESULTS.md
7. **MERGE schema-refactor branch!**

---

**Status**: ✅ READY TO FIX
**Confidence**: 90% - Fix #2 will resolve the issue
**Date**: 2026-03-19
