# Integration Test Review - Schema Module

## Executive Summary

**Current State**: We have 10 schema integration tests across 2 files, with 2 ignored tests. After review, these tests are **properly scoped as integration tests** - they test cross-boundary interactions (filesystem + database + loader pipeline).

**Recommendation**: Keep all 10 tests, fix the 2 ignored tests, and add 3 new integration tests for incremental loading scenarios.

---

## What Makes a Test an Integration Test?

Following matklad's testing philosophy (purity vs extent):

### Integration Tests Should:
1. **Cross boundaries**: Test interactions between multiple modules/subsystems
2. **Use real implementations**: Filesystem (TempDir), Database (redb), not mocks
3. **Test end-to-end behavior**: User-visible workflows (load → resolve → persist)
4. **Be impure but valuable**: Accept filesystem/DB I/O cost for confidence

### Integration Tests Should NOT:
1. **Test single-unit behavior**: That's for unit tests (use InMemoryRepository)
2. **Duplicate unit test coverage**: Don't re-test pure logic
3. **Be unnecessarily slow**: Don't test timing/staleness (unit tests can do that)

---

## Current Integration Tests Analysis

### File: `tests/schema_storage.rs` (4 tests, 2 ignored)

**Purpose**: Test Repository persistence with **real redb database**

#### ✅ KEEP: `property_bank_roundtrip` (37 lines)
- **What it tests**: PropertyBank save → retrieve with redb
- **Cross-boundary**: Database persistence layer
- **Value**: Verifies rkyv serialization + redb storage works together
- **Integration aspect**: Tests Repository trait implementation with real DB
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `schema_roundtrip` (54 lines)
- **What it tests**: Schema save → retrieve by ID with redb
- **Cross-boundary**: Database persistence + ID lookup
- **Value**: Verifies Schema storage/retrieval works end-to-end
- **Integration aspect**: Tests Repository + redb + rkyv together
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `schema_find_by_name` (78 lines)
- **What it tests**: Schema save → retrieve by name with redb
- **Cross-boundary**: Database persistence + name index
- **Value**: Verifies name-based lookup (different code path than ID lookup)
- **Integration aspect**: Tests Repository name index with real DB
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ⚠️ FIX: `schema_list` (IGNORED - rkyv address space limitation)
- **Current state**: Ignored with reason "rkyv address space limitation"
- **What it tests**: Save 2 schemas → list all schemas
- **Problem**: "subtree pointer overran range" error when deserializing multiple schemas
- **Root cause**: Archived pointers only valid in address space where created
- **Proposed solution**:
  1. Try using `TestDb::reopen()` after save (forces new read transaction)
  2. If still fails, keep ignored but add comment linking to ADR/issue
  3. Consider if this is a real limitation or test bug
- **Verdict**: ⚠️ **INVESTIGATE AND FIX** - This is important functionality

#### ⚠️ FIX: `schema_delete` (IGNORED - not yet implemented)
- **Current state**: Ignored with reason "delete_schema not yet implemented"
- **What it tests**: Save schema → delete → verify gone
- **Problem**: `delete_schema()` returns `unimplemented!()`
- **Proposed solution**:
  1. Implement `delete_schema()` in RedbRepository
  2. Un-ignore this test
  3. OR: Remove test if deletion is not needed (confirm with requirements)
- **Verdict**: ⚠️ **IMPLEMENT OR REMOVE** - Either implement feature or remove test

---

### File: `tests/schema_resolution.rs` (7 tests, 0 ignored)

**Purpose**: Test Loader pipeline with **real filesystem + database**

#### ✅ KEEP: `resolves_property_bank_references` (62 lines)
- **What it tests**: Write files → Loader.load() → verify $ref resolution
- **Cross-boundary**: Filesystem → Ingestor → RefExpander → Loader → Database
- **Value**: End-to-end test of property_bank reference resolution
- **Integration aspect**: Full pipeline with real files and DB
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `resolves_inline_properties` (99 lines)
- **What it tests**: Write files → Loader.load() → verify inline property resolution
- **Cross-boundary**: Filesystem → Ingestor → Loader → Database
- **Value**: Tests inline properties (different code path than $ref)
- **Integration aspect**: Full pipeline with real files and DB
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `resolves_multiple_schemas` (136 lines)
- **What it tests**: Write 3 schema files → Loader.load() → verify all resolved
- **Cross-boundary**: Filesystem → Ingestor (bulk operations) → Loader → Database
- **Value**: Tests bulk processing and multi-schema coordination
- **Integration aspect**: Tests scalability of pipeline
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `resolves_schema_inheritance` (183 lines)
- **What it tests**: Write base + child schemas → Loader.load() → verify inheritance
- **Cross-boundary**: Filesystem → Ingestor → Extender → Merger → Database
- **Value**: End-to-end inheritance resolution
- **Integration aspect**: Tests Extender + Merger integration
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `loads_and_persists_property_bank` (235 lines)
- **What it tests**: Write property_bank.json → Loader.load() → verify persisted
- **Cross-boundary**: Filesystem → Ingestor → Database → Verify with new Repository
- **Value**: Tests property bank persistence across repository instances
- **Integration aspect**: Verifies data survives beyond single load session
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `detects_missing_property_bank_reference` (277 lines)
- **What it tests**: Write invalid $ref → Loader.load() → expect error
- **Cross-boundary**: Filesystem → Ingestor → RefExpander (error handling)
- **Value**: Tests error propagation through entire pipeline
- **Integration aspect**: End-to-end error handling
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

#### ✅ KEEP: `detects_circular_inheritance` (308 lines)
- **What it tests**: Write circular extends → Loader.load() → expect error
- **Cross-boundary**: Filesystem → Ingestor → Extender (cycle detection)
- **Value**: Tests complex error case requiring graph analysis
- **Integration aspect**: End-to-end cycle detection
- **Verdict**: ✅ **PROPER INTEGRATION TEST** - Keep as-is

---

## Tests That Belong in Unit Tests (NOT integration)

After reviewing both files, **NO tests should be moved to unit tests**. All tests are properly scoped as integration tests testing cross-boundary behavior.

However, we should ADD unit test coverage in `ingestor.rs` and `loader.rs` for:
- ✅ DONE: Staleness detection logic (using InMemoryRepository)
- ✅ DONE: Cached expansion logic (using InMemoryRepository)
- ✅ DONE: Property bank loading (using InMemoryRepository)

---

## Missing Integration Test Coverage

### Critical Gaps to Add:

#### 1. **Incremental Loading with Real Filesystem** (NEW)
**Test**: `incremental_load_detects_file_changes`
```rust
// GIVEN: Initial load with schemas
// WHEN: Modify one schema file (real file write with sleep for mtime)
// THEN: Second load returns only changed schema
```
**Why Integration**: Tests filesystem change detection with real mtime/stat
**Why Not Unit**: Unit tests use InMemoryRepository (no real filesystem)

#### 2. **Staleness Detection Across Sessions** (NEW)
**Test**: `staleness_persists_across_database_reopens`
```rust
// GIVEN: First load saves views
// WHEN: Reopen database (TestDb::reopen()), second load
// THEN: Schemas detected as Fresh (views persisted correctly)
```
**Why Integration**: Tests view persistence across database sessions
**Why Not Unit**: Unit tests don't test redb persistence guarantees

#### 3. **Property Bank Incremental Update** (NEW)
**Test**: `property_bank_incremental_update_triggers_re_resolution`
```rust
// GIVEN: Load schemas with $refs
// WHEN: Add new property to property_bank.json
// THEN: Schemas using NEW property are re-resolved
```
**Why Integration**: Tests Phase 5.1 incremental resolution with real files
**Why Not Unit**: Complex cross-boundary behavior (files + DB + loader)

---

## Test Organization Recommendations

### Proposed Structure (Better Cohesion):
```
lithos-core/tests/
├── common/mod.rs          # Shared test utilities (TestDb, builders)
├── schema_storage.rs      # Repository + Database integration (4 tests)
└── schema_loader.rs       # Loader pipeline integration (10 tests)
    ├── initial_loading    # 4 tests (first load scenarios)
    ├── inheritance        # 1 test (extends/excludes)
    ├── incremental_loading # 3 tests (staleness + caching)
    └── error_handling     # 2 tests (error detection)
```

### Rationale:
- **schema_storage.rs**: Low-level persistence (Repository trait + redb)
- **schema_loader.rs**: ALL Loader behavior (initial + incremental + errors)
  - Better cohesion: resolution and incremental loading are the same concern
  - Organized by behavior within the file (submodules)
  - Easier to find all Loader tests in one place

**Why consolidate?** Resolution and incremental loading both test the Loader's
ability to load schemas from files. Splitting them creates artificial boundaries.

See `INTEGRATION_TEST_ORGANIZATION_PROPOSAL.md` for detailed migration plan.

---

## Action Items

### Immediate (Before Phase 6.3):

1. ✅ **INVESTIGATE**: `schema_list` test - Why does rkyv fail with multiple schemas?
   - Try `TestDb::reopen()` after save
   - Check if this is a test issue or real limitation
   - Document findings in ADR if it's a real constraint

2. ✅ **DECIDE**: `schema_delete` test - Implement or remove?
   - Check if schema deletion is required by PRD
   - If yes: implement `delete_schema()` in RedbRepository
   - If no: remove test

### Phase 6.3 (Integration Test Implementation):

3. ✅ **ADD**: 3 new integration tests for incremental loading
   - `incremental_load_detects_file_changes`
   - `staleness_persists_across_database_reopens`
   - `property_bank_incremental_update_triggers_re_resolution`

4. ✅ **VERIFY**: All integration tests pass with real filesystem timing
   - Tests should use `std::thread::sleep` where needed for mtime
   - Tests should be marked with `#[expect(clippy::disallowed_methods)]`

5. ✅ **DOCUMENT**: Integration test patterns in `_bmad-output/test-developer-guide.md`
   - When to use integration tests vs unit tests
   - How to use TestDb and helpers
   - Naming conventions for integration tests

---

## Conclusion

**All 10 existing integration tests are properly scoped and should be kept.**

**Summary**:
- ✅ 8 tests passing - KEEP
- ⚠️ 2 tests ignored - INVESTIGATE/FIX
- 📝 3 new tests needed - ADD in Phase 6.3

**Next Steps**:
1. Fix/investigate the 2 ignored tests
2. Add 3 new incremental loading integration tests
3. Document integration test patterns

**Quality Gate**: All integration tests must pass before merging schema-refactor branch.
