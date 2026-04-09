# Integration Test Action Plan

**Created**: After Phase 6.2 completion (822/822 tests passing)
**Purpose**: Fix 2 ignored integration tests and add 3 new incremental loading tests
**Status**: Ready to execute

---

## Summary

**Current State**:
- ✅ 8 integration tests passing
- ⚠️ 2 integration tests ignored (need investigation/fix)
- 📝 3 new integration tests needed (incremental loading scenarios)

**Goal**: 13/13 integration tests passing with comprehensive coverage

---

## Part 1: Fix Ignored Tests

### Issue 1: `schema_list` Test (INVESTIGATE)

**File**: `tests/schema_storage.rs:132`

**Current Status**: Ignored with reason "rkyv address space limitation"

**Error**:
```
Storage(Deserialization("subtree pointer overran range: ptr 0x0000000bed02c09e size 4294967295
in range 0x0000000bed02c000..0x0000000bed02c09f
trace: while checking field index 0 of tuple struct 'ArchivedTuple2'"))
```

**What the test does**:
1. Save 2 schemas to database
2. Call `repository.list_schemas()`
3. Assert returns 2 schemas

**Root Cause Hypothesis**:
- rkyv archived pointers are only valid in the memory address space where they were created
- `list_schemas()` deserializes multiple schemas in same transaction
- Pointer corruption when accessing second schema's archived data

**Action Items**:

1. **TRY**: Use `TestDb::reopen()` pattern
   ```rust
   // Save schemas
   repository.save_schemas(&[schema1, schema2])?;

   // Force database to close and reopen (new address space)
   let fresh_db = test_db.reopen()?;
   let repository2 = setup_repository(&fresh_db);

   // Try list_schemas() with fresh repository
   let all = repository2.list_schemas()?;
   ```

2. **IF STILL FAILS**: Check `list_schemas()` implementation
   - Is it deserializing correctly?
   - Does it need to clone data out of archived form?
   - Compare with `find_schema_by_id()` which DOES work

3. **INVESTIGATE**: Is this a real limitation or a test bug?
   - Check redb transaction lifetime
   - Check if rkyv requires special handling for collections
   - Look for similar issues in rkyv documentation

4. **DOCUMENT**: If it's a real limitation
   - Create ADR documenting the constraint
   - Add note to `list_schemas()` rustdoc
   - Keep test ignored with reference to ADR

5. **IF FIXABLE**: Un-ignore test and verify passes

**Priority**: HIGH - This affects core functionality (listing all schemas)

---

### Issue 2: `schema_delete` Test (IMPLEMENT OR REMOVE)

**File**: `tests/schema_storage.rs:172`

**Current Status**: Ignored with reason "delete_schema not yet implemented"

**Implementation in storage.rs**:
```rust
fn delete_schema(&self, _id: SchemaId) -> Result<(), Self::Error> {
    // Schema deletion is complex and not yet needed - requires:
    // 1. Load schema to get its name
    // 2. Delete from SCHEMA_BY_ID
    // 3. Delete from SCHEMA_ID_BY_NAME
    // 4. Delete from SCHEMA_PARENT
    // 5. Remove from SCHEMA_CHILDREN multimap entries
    unimplemented!("Schema deletion with proper cleanup of all references")
}
```

**Action Items**:

1. **CHECK PRD**: Is schema deletion a required feature?
   - Review `_bmad-output/planning-artifacts/prd.md`
   - Check if users need to delete schemas
   - Check if this is a "nice to have" vs "must have"

2. **IF REQUIRED**: Implement `delete_schema()`
   - Follow the 5-step cleanup plan in the comment
   - Add unit tests for the implementation
   - Un-ignore the integration test
   - Verify all tests pass

3. **IF NOT REQUIRED**: Remove the test
   - Delete the test from `schema_storage.rs`
   - Remove `delete_schema()` from Repository trait
   - Or: Mark as `unimplemented!()` and document "future enhancement"

4. **DECISION NEEDED**: Choose one of the above paths

**Priority**: MEDIUM - Feature completeness, not blocking core workflows

---

## Part 2: Add New Integration Tests

### Test 1: Incremental Loading with Real Filesystem Changes

**File**: `tests/schema_incremental.rs` (NEW FILE)

**Test Name**: `incremental_load_detects_file_changes`

**Purpose**: Verify that Loader detects file changes via mtime/hash when using real filesystem

**Test Flow**:
```rust
#[test]
fn incremental_load_detects_file_changes() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    // SETUP: Write initial files
    write_file(vault_dir.path(), "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#)?;
    write_file(vault_dir.path(), "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "#property_bank/title"}}}"#)?;
    write_file(vault_dir.path(), "schemas/note.json",
        r#"{"$version": "1.0", "properties": {"content": {"type": "string"}}}"#)?;

    // FIRST LOAD: Both schemas should be NEW
    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);
    let first = loader.load()?;
    assert_eq!(first.len(), 2, "First load: 2 schemas");

    // WAIT: Ensure mtime changes (filesystem granularity)
    #[expect(clippy::disallowed_methods, reason = "Integration test needs real filesystem timing")]
    std::thread::sleep(std::time::Duration::from_millis(10));

    // MODIFY: Change only task.json
    write_file(vault_dir.path(), "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "#property_bank/title"}, "done": {"type": "bool"}}}"#)?;

    // SECOND LOAD: Only task.json should be re-resolved
    let repository2 = setup_repository(test_db.db());
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);
    let second = loader2.load()?;

    assert_eq!(second.len(), 1, "Second load: only changed schema");
    assert_eq!(second[0].name().as_ref(), "task");
    assert_eq!(second[0].properties().len(), 2, "Should have 2 properties now");

    Ok(())
}
```

**Why Integration**:
- Tests real filesystem mtime detection
- Tests Loader + Ingestor + Database coordination
- Tests RawSchemaView staleness detection with real files

**Why Not Unit**:
- Unit tests use InMemoryRepository (no real file timestamps)
- Unit tests can't test filesystem timing edge cases

---

### Test 2: Staleness Persists Across Database Sessions

**File**: `tests/schema_incremental.rs`

**Test Name**: `staleness_persists_across_database_reopens`

**Purpose**: Verify that RawSchemaView persists correctly and enables staleness detection across sessions

**Test Flow**:
```rust
#[test]
fn staleness_persists_across_database_reopens() -> TestResult {
    let vault_dir = TempDir::new()?;
    let mut test_db = TestDb::new()?;

    // SETUP: Write files
    write_file(vault_dir.path(), "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {}}"#)?;
    write_file(vault_dir.path(), "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#)?;

    // FIRST SESSION: Load schemas
    {
        let config = test_config(vault_dir.path())?;
        let repository = setup_repository(test_db.db());
        let source = FsReader::new(vault_dir.path());
        let loader = Loader::new(repository, source, &config);
        let first = loader.load()?;
        assert_eq!(first.len(), 1);
    } // Repository dropped, database still open

    // REOPEN DATABASE: Simulate fresh application start
    let fresh_db = test_db.reopen()?;

    // SECOND SESSION: Load again without file changes
    {
        let config = test_config(vault_dir.path())?;
        let repository2 = setup_repository(&fresh_db);
        let source2 = FsReader::new(vault_dir.path());
        let loader2 = Loader::new(repository2, source2, &config);
        let second = loader2.load()?;

        // Schemas should be FRESH (views were persisted)
        assert_eq!(second.len(), 0, "No schemas should be re-resolved (all fresh)");
    }

    // VERIFY: Check that RawSchemaView was persisted
    let repository3 = setup_repository(&fresh_db);
    let path = PathBuf::from("schemas/task.json");
    let view = repository3.find_raw_schema_view_by_path(&path.to_string_lossy())?;
    assert!(view.is_some(), "RawSchemaView should be persisted");

    Ok(())
}
```

**Why Integration**:
- Tests redb persistence guarantees
- Tests view serialization/deserialization across sessions
- Tests that staleness detection works after restart

**Why Not Unit**:
- InMemoryRepository doesn't test persistence across sessions
- Need to verify rkyv + redb work together correctly

---

### Test 3: Property Bank Incremental Update

**File**: `tests/schema_incremental.rs`

**Test Name**: `property_bank_incremental_update_triggers_re_resolution`

**Purpose**: Verify Phase 5.1 incremental resolution when property_bank changes

**Test Flow**:
```rust
#[test]
fn property_bank_incremental_update_triggers_re_resolution() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    // SETUP: Initial files
    write_file(vault_dir.path(), "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#)?;
    write_file(vault_dir.path(), "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "#property_bank/title"}}}"#)?;

    // FIRST LOAD
    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);
    let first = loader.load()?;
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].properties().len(), 1, "Should have 1 property");

    // WAIT for filesystem timing
    #[expect(clippy::disallowed_methods, reason = "Integration test needs real filesystem timing")]
    std::thread::sleep(std::time::Duration::from_millis(10));

    // MODIFY: Add new property to property_bank
    write_file(vault_dir.path(), "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string"},
            "status": {"type": "string"}
        }}"#)?;

    // UPDATE: task.json to use new property
    write_file(vault_dir.path(), "schemas/task.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"$ref": "#property_bank/title"},
            "status": {"$ref": "#property_bank/status"}
        }}"#)?;

    // SECOND LOAD: Schema should be re-resolved with new property
    let repository2 = setup_repository(test_db.db());
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);
    let second = loader2.load()?;

    assert_eq!(second.len(), 1, "Should re-resolve schema");
    assert_eq!(second[0].properties().len(), 2, "Should have 2 properties now");

    // VERIFY: Check that both properties resolved correctly
    let task = &second[0];
    assert!(task.properties().contains_key(&"title".try_into()?));
    assert!(task.properties().contains_key(&"status".try_into()?));

    Ok(())
}
```

**Why Integration**:
- Tests Phase 5.1 incremental property bank resolution
- Tests coordination between property_bank changes and schema re-resolution
- Tests expanded properties caching behavior

**Why Not Unit**:
- Complex cross-boundary behavior (files + ingestor + loader + expander)
- Need to verify full pipeline works together

---

## Test File Template: `schema_incremental.rs`

```rust
//! Integration tests for incremental schema loading.
//!
//! Tests the Loader's ability to detect changes and perform incremental
//! resolution across multiple load cycles with real filesystem and database.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use std::path::{Path, PathBuf};

use common::*;
use lithos_core::{
    config::{aggregate::Config, raw::RawConfig, vault::{VaultId, VaultRoot}},
    fs::FsReader,
    schema::{loader::Loader, storage::Repository as _},
};
use tempfile::TempDir;

/// Write a file to the test directory.
fn write_file(root: &Path, relative: &str, content: &str) -> TestResult {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a test config for a vault root.
fn test_config(root: &Path) -> TestResult<Config> {
    let raw = RawConfig::default();
    let root = VaultRoot::try_new(root.to_path_buf())?;
    let config = Config::build(
        &raw,
        VaultId::new(),
        root,
        lithos_core::config::aggregate::Version::initial(),
    )?;
    Ok(config)
}

// Tests go here...
```

---

## Execution Plan

### Step 1: Investigate `schema_list` (Day 1)

1. Try `TestDb::reopen()` pattern
2. If still fails, investigate redb/rkyv interaction
3. Document findings
4. Either fix or create ADR documenting limitation

### Step 2: Decide on `schema_delete` (Day 1)

1. Review PRD for deletion requirement
2. If required: implement + test
3. If not required: remove test or document as future work

### Step 3: Create `schema_incremental.rs` (Day 2)

1. Create new test file with template
2. Implement Test 1: `incremental_load_detects_file_changes`
3. Implement Test 2: `staleness_persists_across_database_reopens`
4. Implement Test 3: `property_bank_incremental_update_triggers_re_resolution`
5. Run all integration tests: `cargo nextest run --test schema_incremental`

### Step 4: Verify All Tests Pass (Day 2)

1. Run full integration test suite
2. Fix any failures
3. Document any new patterns in test-developer-guide.md

### Step 5: Update Documentation (Day 2)

1. Update `INTEGRATION_TEST_REVIEW.md` with results
2. Update `_bmad-output/test-developer-guide.md` with integration test patterns
3. Update `PHASE_6_2_STATUS.md` → `PHASE_6_3_STATUS.md`

---

## Success Criteria

✅ All integration tests passing (13/13)
✅ No ignored integration tests (or documented ADRs for unavoidable ignores)
✅ Comprehensive coverage of incremental loading scenarios
✅ Documentation updated with integration test patterns

---

## Related Documents

- `INTEGRATION_TEST_REVIEW.md` - Detailed analysis of current tests
- `unit-test-review-analysis-REVISED.md` - Unit test coverage analysis
- `loader-ingestor-refactoring-implementation-plan.md` - Phase 6 plan
- `_bmad-output/test-developer-guide.md` - Test naming conventions
