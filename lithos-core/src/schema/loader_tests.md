# Loader Unit Tests - Implementation Plan

## Critical Test Scenarios

### 1. Three-Way Partitioning Logic
**Test:** Verify stale schemas are correctly categorized
- NEW schema (not in name_to_id) → new_schemas
- EXISTING + file_changed → schemas_for_full_resolution
- EXISTING + file_unchanged → existing_file_unchanged

**Assertion:** Each category gets the right schemas

### 2. Incremental Resolution Path
**Test:** Existing schema with unchanged file + bank property change
**Given:**
- Schema exists in DB
- File hasn't changed (not in file_changed_ids)
- Property bank has changed properties
- Schema references one of the changed properties

**Expected:**
- Schema goes to `existing_file_unchanged`
- `find_schemas_using_properties` called
- `Resolver::resolve_affected_properties` called
- Result includes incrementally resolved schema

### 3. Full Resolution Path - New Schema
**Test:** Brand new schema file
**Given:**
- Schema NOT in name_to_id
- Raw schema from filesystem

**Expected:**
- Schema goes to `new_schemas`
- Full pipeline: RefExpander → Extender → Resolver
- Result includes fully resolved schema

### 4. Full Resolution Path - File Changed
**Test:** Existing schema with file modification
**Given:**
- Schema EXISTS in name_to_id
- File timestamp/hash changed (in file_changed_ids)

**Expected:**
- Schema goes to `schemas_for_full_resolution`
- Full pipeline executed
- Result includes fully resolved schema

### 5. No Incremental When No Property Changes
**Test:** Bank stale but no changed_properties
**Given:**
- Existing schemas with unchanged files
- `bank_stale = true`
- `changed_properties = []` (empty)

**Expected:**
- Incremental path skipped (line 183 check)
- No calls to `resolve_affected_properties`
- Schemas remain in existing_file_unchanged but not processed

### 6. Mixed Scenario
**Test:** Multiple schemas with different states
**Given:**
- 1 new schema
- 1 existing with file change
- 1 existing with unchanged file + property change
- 1 existing completely fresh

**Expected:**
- New → full resolution
- File changed → full resolution
- Unchanged + property ref → incremental resolution
- Fresh → not in stale list at all

## Implementation Strategy

Since creating a full in-memory Repository is complex, we have two options:

### Option A: Integration Tests (Recommended)
- Use actual `RedbRepository` with temporary database
- Real filesystem with `TempDir`
- End-to-end validation
- **Downside:** Slower, more setup

### Option B: Unit Tests with Mocks
- Mock Repository implementation
- Mock FsReader
- Faster execution
- **Downside:** More boilerplate, doesn't test actual integrations

**Recommendation:** Start with Option A (integration-style unit tests) since:
1. We already have `TestDb` helper in common module
2. More confidence in actual behavior
3. Can reuse disabled test infrastructure

## Next Steps

1. Create test helper to setup Loader with temp DB + filesystem
2. Implement 6 test scenarios above
3. Run and verify all pass
4. Document coverage gaps
