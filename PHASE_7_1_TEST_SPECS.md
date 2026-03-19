# Phase 7.1 - Critical Test Recovery Specs

## Overview

**Goal**: Add 5 HIGH-priority integration tests to close critical coverage gaps before merging schema-refactor branch.

**Effort**: ~2-3 hours total
**Risk Mitigated**: HIGH (restart durability, corruption detection, batch atomicity)

**Context**: Comprehensive unit test mapping revealed 82% coverage (49/60 deleted tests covered). Only 11 TRUE gaps remain, 5 are HIGH-priority.

---

## Test Specifications

### 1. PropertyBank Survives Restart

**File**: `lithos-core/tests/schema_storage.rs`
**Module**: Add to existing file (or create `durability` submodule)
**Effort**: 30 minutes

**Test Name**: `property_bank_survives_restart`

**Purpose**: Verify PropertyBank data survives database close/reopen cycle (no data loss on restart).

**Steps**:
1. Create fresh TestDb
2. Load PropertyBank from fixture file
3. Save to storage using `loader.load_and_persist()`
4. Verify save succeeded
5. **Reopen database** using `db.reopen()`
6. Load PropertyBank from storage
7. **Assert**: All properties intact (count, names, IDs, specs)
8. **Assert**: Version number matches
9. **Assert**: Timestamps match

**Fixtures Needed**:
- Use existing `fixtures/property-bank.json` (minimal, 2-3 properties)

**Key Assertions**:
```rust
// After reopen
let loaded = loader.load_property_bank().await?;
assert_eq!(loaded.property_count(), original_count);
assert_eq!(loaded.version(), original_version);
// Verify specific property by name
let prop = loaded.get_by_name(&name).expect("property missing");
assert_eq!(prop.id(), original_id);
```

**Risk Mitigated**: HIGH - Silent data loss on process restart

---

### 2. Schema Survives Restart

**File**: `lithos-core/tests/schema_storage.rs`
**Module**: Same as above (durability tests)
**Effort**: 30 minutes

**Test Name**: `schema_survives_restart`

**Purpose**: Verify Schema data survives database close/reopen cycle.

**Steps**:
1. Create fresh TestDb
2. Load schema from fixture (with properties, parent ref, metadata)
3. Save to storage using `loader.load_and_persist()`
4. Verify save succeeded
5. **Reopen database** using `db.reopen()`
6. Load schema by ID
7. **Assert**: All fields intact (name, properties, parent_id, version)
8. **Assert**: Property count and order correct
9. **Assert**: Metadata (created_at, file path) correct

**Fixtures Needed**:
- Use existing `fixtures/person.json` (has inheritance)
- Use existing `fixtures/property-bank.json`

**Key Assertions**:
```rust
// After reopen
let loaded = repo.get(&schema_id).await?.expect("schema missing");
assert_eq!(loaded.name().as_str(), "person");
assert_eq!(loaded.properties().len(), expected_count);
assert_eq!(loaded.parent_id(), Some(&parent_id));
```

**Risk Mitigated**: HIGH - Silent schema loss on restart

---

### 3. Detect Corrupted Schema Bytes

**File**: `lithos-core/tests/schema_storage.rs`
**Module**: Create new `corruption_detection` submodule
**Effort**: 45 minutes

**Test Name**: `detect_corrupted_schema_bytes`

**Purpose**: Verify reading corrupted rkyv bytes returns error (not panic, not silent corruption).

**Steps**:
1. Create fresh TestDb
2. Save valid schema to storage
3. Get schema ID
4. **Manually corrupt schema bytes** in database:
   - Use `with_write_txn` to access raw redb table
   - Read schema bytes
   - Flip bits in middle of byte array
   - Write back corrupted bytes
5. Attempt to load schema by ID
6. **Assert**: Returns `Err(SchemaError::Corruption(...))` (or similar)
7. **Assert**: Does NOT panic
8. **Assert**: Error message mentions corruption/validation

**Implementation Notes**:
- Add `rkyv::access()` validation to Repository `get()` method if missing
- May need to add `SchemaError::Corruption` variant
- See AGENTS.md: "rkyv validation: Use `rkyv::access` at trust boundaries"

**Key Assertions**:
```rust
// After corruption
let result = repo.get(&schema_id).await;
assert!(result.is_err(), "Should detect corruption");
let err = result.unwrap_err();
assert!(matches!(err, SchemaError::Corruption(_)));
```

**Risk Mitigated**: HIGH - Silent data corruption

**Note**: This test requires direct redb access to corrupt bytes. May need to add helper method to TestDb.

---

### 4. Detect Corrupted Name Index

**File**: `lithos-core/tests/schema_storage.rs`
**Module**: `corruption_detection` submodule
**Effort**: 45 minutes

**Test Name**: `detect_corrupted_name_index`

**Purpose**: Verify corrupted name→ID index is detected (returns error, not wrong schema).

**Steps**:
1. Create fresh TestDb
2. Save schema A with name "person"
3. Save schema B with name "event"
4. **Manually corrupt name index** in database:
   - Use `with_write_txn` to access `SCHEMA_NAME_TO_ID` table
   - Change "person" entry to point to schema B's ID
5. Attempt to load schema by name "person"
6. **Assert**: Either returns error OR returns schema with name "person" (not "event")
7. **Assert**: Does NOT silently return wrong schema

**Implementation Notes**:
- This test verifies index integrity
- Two valid outcomes:
  - Best: Returns `Err(SchemaError::IndexCorruption)`
  - Acceptable: Returns schema A (validates name matches query)
- Worst (must fix): Returns schema B with name "event"

**Key Assertions**:
```rust
// After corruption
match repo.find_by_name("person").await {
    Ok(Some(schema)) => {
        // If returns schema, MUST be the right one
        assert_eq!(schema.name().as_str(), "person", "Index points to wrong schema!");
    }
    Err(SchemaError::IndexCorruption(_)) => {
        // Corruption detected - acceptable
    }
    Ok(None) => {
        panic!("Should not return None for existing schema");
    }
    Err(e) => {
        panic!("Unexpected error: {}", e);
    }
}
```

**Risk Mitigated**: HIGH - find_by_name could return wrong schema

**Note**: May need to add name validation to `find_by_name()` implementation.

---

### 5. Batch Save Is Atomic

**File**: `lithos-core/tests/schema_storage.rs`
**Module**: Create new `batch_operations` submodule
**Effort**: 45 minutes

**Test Name**: `batch_save_is_atomic`

**Purpose**: Verify batch save is all-or-nothing (partial failure rolls back entire batch).

**Steps**:
1. Create fresh TestDb
2. Prepare batch of 3 schemas:
   - Schema A: valid
   - Schema B: valid
   - Schema C: INVALID (duplicate name of A, or missing required field)
3. Attempt batch save
4. **Assert**: Batch save returns error
5. **Assert**: NO schemas saved (transaction rolled back)
6. Query for schema A by name
7. **Assert**: Returns None (not saved)
8. List all schemas
9. **Assert**: Empty (full rollback)

**Implementation Notes**:
- Depends on redb transaction semantics
- If current `save()` doesn't support batches, this test verifies single-transaction multi-save
- May need to add `save_batch()` method to Repository trait

**Key Assertions**:
```rust
// After failed batch save
assert!(batch_result.is_err(), "Invalid batch should fail");

// Verify rollback
let loaded_a = repo.find_by_name("person").await?;
assert_eq!(loaded_a, None, "Transaction should rollback on error");

let all_schemas = repo.list().await?;
assert_eq!(all_schemas.len(), 0, "No partial saves allowed");
```

**Risk Mitigated**: MEDIUM - Partial writes could corrupt database on error

**Note**: This test depends on Repository API design. May need to implement batch save if not present.

---

## Implementation Order

### Recommended Sequence

1. **Restart Durability Tests First** (1 hour)
   - `property_bank_survives_restart`
   - `schema_survives_restart`
   - **Rationale**: Uses existing `TestDb::reopen()` (recently fixed), straightforward

2. **Batch Atomicity Test** (45min)
   - `batch_save_is_atomic`
   - **Rationale**: Tests high-level API, no DB internals required

3. **Corruption Detection Tests** (1.5 hours)
   - `detect_corrupted_schema_bytes`
   - `detect_corrupted_name_index`
   - **Rationale**: Requires adding rkyv validation, potentially new error variants

---

## Success Criteria

### Definition of Done (per test)

- [ ] Test implemented in `lithos-core/tests/schema_storage.rs`
- [ ] Test passes (`mise run test:integration`)
- [ ] Test covers exact gap concern from UNIT_TEST_COVERAGE_MAPPING.md
- [ ] Test uses existing fixtures (no new test data needed)
- [ ] Test properly cleans up resources (uses `TestDb` RAII)
- [ ] Test has clear doc comment explaining purpose
- [ ] Test assertions are thorough (not just "doesn't panic")

### Phase 7.1 Complete When

- [ ] All 5 tests implemented and passing
- [ ] `mise run verify` passes (all quality gates green)
- [ ] PHASE_6_3_STATUS.md updated with Phase 7.1 completion
- [ ] Risk assessment updated (mark gaps as CLOSED)
- [ ] Final review document created (merge readiness checklist)

---

## File Organization

### Proposed Structure for `schema_storage.rs`

```rust
// lithos-core/tests/schema_storage.rs

mod roundtrip_tests {
    // Existing tests
    #[test] fn property_bank_roundtrip() { ... }
    #[test] fn schema_roundtrip() { ... }
}

mod lookup_tests {
    // Existing tests
    #[test] fn schema_find_by_name() { ... }
    #[test] #[ignore] fn schema_list() { ... }  // Blocked
    #[test] #[ignore] fn schema_delete() { ... }  // Blocked
}

mod durability_tests {  // NEW
    #[test] fn property_bank_survives_restart() { ... }  // NEW
    #[test] fn schema_survives_restart() { ... }  // NEW
}

mod corruption_detection {  // NEW
    #[test] fn detect_corrupted_schema_bytes() { ... }  // NEW
    #[test] fn detect_corrupted_name_index() { ... }  // NEW
}

mod batch_operations {  // NEW
    #[test] fn batch_save_is_atomic() { ... }  // NEW
}
```

---

## Potential Blockers

### Known Risks

1. **rkyv validation API**
   - Risk: May need to change Repository trait signatures to use `rkyv::access()`
   - Mitigation: Check existing storage.rs implementation first
   - Escalation: If breaks API, create ADR for validation strategy

2. **Batch save API**
   - Risk: Repository trait may not support batch operations
   - Mitigation: Test with individual saves in single transaction
   - Escalation: If not atomic, add `save_many()` method to trait

3. **Manual corruption requires DB internals**
   - Risk: May need to expose redb tables for testing
   - Mitigation: Add `TestDb::corrupt_bytes()` helper method
   - Escalation: If not possible, skip corruption tests and document as KNOWN_ISSUE

---

## Post-Implementation

### After Phase 7.1 Complete

1. **Update documentation**:
   - Mark 5 gaps as CLOSED in UNIT_TEST_COVERAGE_MAPPING.md
   - Update PHASE_6_3_STATUS.md with Phase 7.1 results
   - Update risk assessment (downgrade corruption risk if tests pass)

2. **Run full verification**:
   - `mise run verify` (must be 100% green)
   - Check integration test count: Should be 18/18 passing (13 + 5 new)

3. **Create merge readiness checklist**:
   - All quality gates passing
   - All critical gaps closed
   - No known HIGH-risk issues
   - Documentation updated
   - ADRs current

4. **Merge decision**:
   - If Phase 7.1 complete: MERGE schema-refactor → main
   - If blockers found: Document in KNOWN_ISSUES.md, reassess risk

---

## Estimated Timeline

| Task | Duration | Dependencies |
|------|----------|--------------|
| Setup test structure | 15min | None |
| Test 1: property_bank_survives_restart | 30min | TestDb::reopen() |
| Test 2: schema_survives_restart | 30min | Test 1 |
| Test 3: batch_save_is_atomic | 45min | None |
| Test 4: detect_corrupted_schema_bytes | 45min | rkyv::access() |
| Test 5: detect_corrupted_name_index | 45min | Test 4 |
| Documentation updates | 15min | All tests complete |
| **TOTAL** | **~3 hours** | |

**Best Case**: 2 hours (if no API changes needed)
**Worst Case**: 4 hours (if must add validation layer + batch API)

---

## Next Steps

1. ✅ Phase 7.1 specs complete (this document)
2. Implement tests in order (durability → batch → corruption)
3. Run `mise run test:integration` after each test
4. Update documentation when all 5 tests pass
5. Run `mise run verify` before merge
6. Create final merge checklist
7. Merge schema-refactor branch

---

**Status**: ✅ **SPECS COMPLETE** - Ready for implementation
**Reviewed by**: AI Agent (bmad-master)
**Date**: 2026-03-19
