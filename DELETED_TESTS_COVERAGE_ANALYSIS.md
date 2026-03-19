# Deleted Integration Tests - Coverage Analysis

## Executive Summary

**Finding**: ~65 integration tests were deleted during CQRS → Repository refactoring.

**Status**: ✅ **CORRECTED ANALYSIS COMPLETE** (Initial assessment was WRONG)

**CRITICAL UPDATE**: Initial analysis only checked integration tests. Comprehensive unit test mapping reveals:
- ✅ **82% coverage** (49/60 tests covered by unit or integration tests)
- ❌ **18% true gap rate** (11/60 tests have no coverage)
- ⚠️ **5 HIGH-priority gaps** require integration tests before merge

**See**: [UNIT_TEST_COVERAGE_MAPPING.md](UNIT_TEST_COVERAGE_MAPPING.md) for complete analysis.

This document provides the ORIGINAL (incorrect) analysis for reference. **Use UNIT_TEST_COVERAGE_MAPPING.md for accurate gap assessment.**

---

## Deleted Test Files Inventory

### 1. schema_cqrs.rs - 42 TESTS DELETED

**Deletion**: Commit 7b91d219 (2024)
**Reason**: CQRS pattern (Command/Query) replaced by unified Repository trait
**Impact**: HIGH - Major test coverage loss

#### 1.1 PropertyBank Tests (12 deleted)

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| save_creates_singleton | ✅ property_bank_roundtrip | COVERED |
| save_updates_existing_singleton | ❌ None | **GAP** |
| version_increments_persist | ❌ None | **GAP** |
| survives_restart | ❌ None | **GAP** |
| roundtrip_preserves_all_fields | ✅ property_bank_roundtrip | COVERED |
| empty_bank_persists | ❌ None | **GAP** |
| indices_consistent_after_updates | ❌ None | **GAP** |
| iteration_order_consistent | ❌ None | **GAP** |
| get_property_by_id_returns_correct_property | ❌ None | **GAP** |
| get_property_by_id_invalid_id_returns_none | ❌ None | **GAP** |
| get_property_by_id_bank_missing_returns_none | ❌ None | **GAP** |
| get_property_by_id_roundtrip_preserves_data | ❌ None | **GAP** |

**Gap Summary**: 10/12 tests not covered (83% gap rate)

#### 1.2 Schema CRUD Tests (14 deleted)

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| save_and_load_roundtrip | ✅ schema_roundtrip | COVERED |
| find_by_name_works | ✅ schema_find_by_name | COVERED |
| find_by_name_missing_returns_none | ❌ None | **GAP** |
| batch_save_atomic | ❌ None | **GAP** |
| list_returns_all_schemas | ⏸️ schema_list (ignored) | BLOCKED |
| delete_removes_schema_and_index | ⏸️ schema_delete (ignored) | BLOCKED |
| properties_persist_sorted | ❌ None | **GAP** |
| empty_schema_persists | ❌ None | **GAP** |
| survives_restart | ❌ None | **GAP** |
| update_overwrites_existing | ❌ None | **GAP** |
| list_name_id_pairs_works | ❌ None | **GAP** |
| parent_id_persists | ❌ None | **GAP** |
| no_parent_persists_as_none | ❌ None | **GAP** |
| delete_removes_schema_metadata | ⏸️ schema_delete (ignored) | BLOCKED |

**Gap Summary**: 9/14 tests not covered (64% gap rate)

#### 1.3 Property Field Tests (2 deleted)

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| optionality_persists | ❌ None | **GAP** |
| multiplicity_persists | ❌ None | **GAP** |

**Gap Summary**: 2/2 tests not covered (100% gap rate)

#### 1.4 Cross-Aggregate Tests (9 deleted)

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| property_bank_and_schema_coexist | ✅ loads_and_persists_property_bank | PARTIAL |
| multiple_schemas_with_shared_bank | ✅ resolves_multiple_schemas | COVERED |
| versions_independent | ❌ None | **GAP** |
| property_bank_respects_version_retention_limit | ❌ None | **GAP** |
| batch_save_duplicate_names_in_batch_fails | ❌ None | **GAP** |
| save_rejects_invalid_property_references | ✅ detects_missing_property_bank_reference | COVERED |
| save_succeeds_without_property_bank | ❌ None | **GAP** |

**Gap Summary**: 4/9 tests not covered (44% gap rate)

#### 1.5 Staleness/Corruption Tests (5 deleted)

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| is_schema_stale_reports_missing_schema_as_stale | ✅ detects_file_changes | COVERED |
| is_schema_stale_returns_false_for_fresh_schema | ✅ staleness_persists_across_reopens | COVERED |
| is_schema_stale_with_asymmetric_created_at | ❌ None | **GAP** |
| query_detects_corrupted_schema_data | ❌ None | **GAP** |
| query_detects_missing_metadata_corruption | ❌ None | **GAP** |
| query_detects_corrupted_property_bank_metadata | ❌ None | **GAP** |
| query_detects_corrupted_name_index | ❌ None | **GAP** |

**Gap Summary**: 5/7 tests not covered (71% gap rate)

---

### 2. schema_ingestion.rs - 11 TESTS DELETED

**Deletion**: Commit 7b91d219
**Reason**: File ingestion moved to Loader integration tests
**Impact**: MEDIUM

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| property_bank_loads_from_json | ✅ loads_and_persists_property_bank | COVERED |
| property_bank_loads_from_toml | ❌ None | **GAP** |
| property_bank_loads_from_yaml | ❌ None | **GAP** |
| schema_scanner_finds_all_files | ✅ resolves_multiple_schemas | COVERED |
| schema_scanner_preserves_timestamps | ❌ None | **GAP** |
| full_pipeline_loads_schemas | ✅ loads_and_persists_property_bank | COVERED |
| full_pipeline_resolves_properties | ✅ resolves_property_bank_references | COVERED |
| full_pipeline_incremental_updates | ✅ property_bank_update_triggers_re_resolution | COVERED |
| pipeline_handles_missing_property_bank | ❌ None | **GAP** |
| pipeline_handles_malformed_property_bank | ❌ None | **GAP** |
| schema_name_derived_from_filename | ❌ None | **GAP** |

**Gap Summary**: 7/11 tests not covered (64% gap rate)

---

### 3. schema_incremental_resolution.rs - 5 TESTS DELETED

**Deletion**: Commit 08dce24a
**Reason**: Consolidated into schema_loader.rs incremental_loading tests
**Impact**: LOW

| Deleted Test | Current Coverage | Status |
|-------------|------------------|---------|
| new_schema_uses_full_resolution | ✅ detects_file_changes | COVERED |
| existing_schema_file_change_uses_full_resolution | ✅ detects_file_changes | COVERED |
| existing_schema_bank_change_uses_incremental | ✅ property_bank_update_triggers_re_resolution | COVERED |
| no_resolution_when_property_unchanged | ❌ None | **GAP** |
| mixed_scenario_handles_all_three_paths | ❌ None | **GAP** |

**Gap Summary**: 2/5 tests not covered (40% gap rate)

---

### 4. schema_property_bank.rs - 7 TESTS DELETED

**Deletion**: Commit 137c7ce9
**Reason**: Consolidated into schema_cqrs.rs (then both deleted)
**Impact**: Covered by schema_cqrs.rs gaps above

---

## Overall Gap Analysis

### By Category

| Category | Deleted | Covered | Blocked | Gap | Gap Rate |
|----------|---------|---------|---------|-----|----------|
| PropertyBank | 12 | 2 | 0 | 10 | 83% |
| Schema CRUD | 14 | 2 | 3 | 9 | 64% |
| Property Fields | 2 | 0 | 0 | 2 | 100% |
| Cross-Aggregate | 9 | 4 | 0 | 5 | 56% |
| Staleness | 7 | 2 | 0 | 5 | 71% |
| File Ingestion | 11 | 4 | 0 | 7 | 64% |
| Incremental | 5 | 3 | 0 | 2 | 40% |

**TOTAL**: 60 tests deleted, 17 covered (integration), 3 blocked, **40 gaps (67% gap rate)**

⚠️ **CORRECTED FINDING**: After checking unit tests, TRUE gap rate is **18%** (11/60 tests). See [UNIT_TEST_COVERAGE_MAPPING.md](UNIT_TEST_COVERAGE_MAPPING.md).

---

## Critical Gaps Requiring New Tests

### HIGH Priority (Core Functionality)

1. **PropertyBank Operations** (10 tests needed)
   - ❌ Update existing singleton
   - ❌ Version increments persist
   - ❌ Survives database restart
   - ❌ Empty bank persists
   - ❌ Indices stay consistent after updates
   - ❌ Iteration order is consistent
   - ❌ Get property by ID (valid/invalid/missing bank)
   - ❌ Roundtrip preserves all data

2. **Schema CRUD** (9 tests needed)
   - ❌ Find by name returns none for missing
   - ❌ Batch save is atomic
   - ❌ Properties persist in sorted order
   - ❌ Empty schema persists
   - ❌ Survives database restart
   - ❌ Update overwrites existing
   - ❌ List name-ID pairs works
   - ❌ Parent ID persists (with/without parent)

3. **Property Field Persistence** (2 tests needed)
   - ❌ Optionality field persists correctly
   - ❌ Multiplicity field persists correctly

### MEDIUM Priority (Edge Cases)

4. **Cross-Aggregate Behavior** (4 tests needed)
   - ❌ Versions independent between bank and schema
   - ❌ Version retention limit respected
   - ❌ Batch save rejects duplicate names
   - ❌ Save succeeds without property bank

5. **Error Handling** (5 tests needed)
   - ❌ Detect corrupted schema data
   - ❌ Detect missing metadata corruption
   - ❌ Detect corrupted property bank metadata
   - ❌ Detect corrupted name index
   - ❌ Handle asymmetric created_at timestamps

6. **File Format Support** (7 tests needed)
   - ❌ Property bank loads from TOML
   - ❌ Property bank loads from YAML
   - ❌ Scanner preserves file timestamps
   - ❌ Handle missing property bank file
   - ❌ Handle malformed property bank
   - ❌ Schema name derived from filename

### LOW Priority (Optimization)

7. **Incremental Resolution** (2 tests needed)
   - ❌ No resolution when property unchanged
   - ❌ Mixed scenario handles all three resolution paths

---

## Recommendations

### Immediate Action Required

**Phase 7 MUST include**:

1. **Add 10 PropertyBank integration tests** (HIGH)
   - Critical gap: 83% of PropertyBank behavior untested
   - Risk: Silent data corruption, version conflicts
   - Effort: ~4 hours

2. **Add 9 Schema CRUD tests** (HIGH)
   - Critical gap: Find operations, persistence, restart behavior
   - Risk: Data loss, index corruption
   - Effort: ~4 hours

3. **Add 2 Property Field tests** (HIGH)
   - Critical gap: 100% of field persistence untested
   - Risk: Silent data loss for optionality/multiplicity
   - Effort: ~1 hour

4. **Add 5 Error Handling tests** (MEDIUM)
   - Gap: No corruption detection testing
   - Risk: Silent data corruption goes undetected
   - Effort: ~2 hours

5. **Add 7 File Format tests** (MEDIUM)
   - Gap: Only JSON tested, TOML/YAML untested
   - Risk: Multi-format support broken
   - Effort: ~2 hours

**Total Effort**: ~13 hours to restore critical coverage

### Long-Term Strategy

1. **Test Coverage Target**: 90% of deleted tests restored or proven obsolete
2. **Test Organization**: Group by functionality, not API surface
3. **CI Integration**: Block PRs with <80% integration test coverage
4. **Documentation**: Maintain test coverage matrix in docs/

---

## Test Restoration Priority Matrix

### Phase 7.1 - Critical Recovery (Must Have)
- [ ] PropertyBank: save_updates_existing_singleton
- [ ] PropertyBank: version_increments_persist
- [ ] PropertyBank: survives_restart
- [ ] Schema: find_by_name_missing_returns_none
- [ ] Schema: survives_restart
- [ ] Schema: parent_id_persists
- [ ] Property Fields: optionality_persists
- [ ] Property Fields: multiplicity_persists
- [ ] Cross-Aggregate: save_succeeds_without_property_bank
- [ ] File Formats: property_bank_loads_from_toml

**Estimated**: 10 tests, ~4 hours

### Phase 7.2 - Important Recovery (Should Have)
- [ ] PropertyBank: empty_bank_persists
- [ ] PropertyBank: indices_consistent_after_updates
- [ ] PropertyBank: get_property_by_id_* (3 tests)
- [ ] Schema: batch_save_atomic
- [ ] Schema: properties_persist_sorted
- [ ] Schema: update_overwrites_existing
- [ ] Error: query_detects_corrupted_* (4 tests)

**Estimated**: 13 tests, ~5 hours

### Phase 7.3 - Nice to Have
- [ ] PropertyBank: iteration_order_consistent
- [ ] Schema: empty_schema_persists
- [ ] Schema: list_name_id_pairs_works
- [ ] Cross-Aggregate: versions_independent
- [ ] Cross-Aggregate: version_retention_limit
- [ ] File Formats: all remaining (6 tests)
- [ ] Incremental: remaining (2 tests)

**Estimated**: 17 tests, ~4 hours

---

## Current Test Suite Quality Assessment

### Strengths
✅ Loader pipeline end-to-end coverage (10 tests)
✅ Incremental loading well tested (3 new tests in Phase 6.3)
✅ Error detection for missing refs and circular deps
✅ Basic CRUD operations covered

### Critical Weaknesses
❌ **67% gap rate** from deleted tests
❌ PropertyBank operations barely tested (2/12 = 17%)
❌ Schema persistence barely tested (2/14 = 14%)
❌ Zero property field persistence tests
❌ Zero corruption detection tests
❌ Zero multi-format tests (only JSON)
❌ Zero restart/durability tests

### Risk Assessment

**Data Loss Risk**: 🔴 **HIGH**
- PropertyBank update/version behavior untested
- Schema update/overwrite behavior untested
- Property field persistence untested
- Database restart behavior untested

**Corruption Risk**: 🔴 **HIGH**
- Zero corruption detection tests
- Zero index consistency tests
- Zero metadata validation tests

**Multi-Format Risk**: 🟡 **MEDIUM**
- TOML/YAML support untested
- Could silently break with library updates

**Overall Risk**: 🔴 **HIGH** - Production deployment NOT recommended without Phase 7.1 tests

---

## Sign-Off

**Review Status**: ✅ **COMPLETE** - Comprehensive unit test mapping finished

**Original Assessment (WRONG)**: Stated "no valuable tests lost"
**Second Assessment (WRONG)**: Stated "67% gap rate" (only checked integration tests)
**Final Assessment (CORRECT)**: **18% true gap rate** (11/60 tests) - See [UNIT_TEST_COVERAGE_MAPPING.md](UNIT_TEST_COVERAGE_MAPPING.md)

**Key Discoveries**:
- ✅ 266 schema unit tests exist (far more than expected!)
- ✅ TOML/YAML parsing fully tested (unit tests)
- ✅ PropertyBank operations well-tested (12/13 covered)
- ✅ Incremental resolution well-tested (4/5 covered)
- ❌ 5 HIGH-priority gaps (restart durability, corruption detection)

**Recommendation**: **MERGE-READY AFTER PHASE 7.1** (5 tests, ~2-3 hours)

**Next Steps**:
1. ✅ Complete unit test coverage mapping (DONE)
2. Create Phase 7.1 test implementation specs (5 HIGH-priority tests)
3. Implement 5 critical tests (~2-3 hours)
4. Update risk assessment
5. Merge schema-refactor branch

---

**Reviewed by**: AI Agent (bmad-master)
**Date**: 2026-03-19
**Status**: ✅ **ANALYSIS CORRECTED** - Phase 7.1 required before merge (5 tests, not 40)
