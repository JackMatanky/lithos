# Test Reorganization Plan - Phase 6.2

## Objective
Reorganize ALL tests in `ingestor.rs` and `loader.rs` into proper submodules to enhance readability and coverage analysis.

## Current State Analysis

### ingestor.rs Tests (28 tests total)

**Standalone tests (10 tests - NEED ORGANIZATION)**:
1. `property_bank_parses_valid_json` → Move to `property_bank_loading_tests`
2. `property_bank_parses_valid_yaml` → Move to `property_bank_loading_tests`
3. `property_bank_parses_valid_toml` → Move to `property_bank_loading_tests`
4. `property_bank_returns_error_for_invalid_json` → Move to `property_bank_loading_tests`
5. `property_bank_returns_error_for_unsupported_format` → Move to `property_bank_loading_tests`
6. `property_bank_defaults_version_when_omitted` → Move to `property_bank_loading_tests`
7. `ingest_all_returns_both_property_bank_and_schemas` → Already in `ingest_all_tests` (MOVE)
8. `ingest_all_supports_toml_format` → Already in `ingest_all_tests` (MOVE)
9. `ingest_all_separates_property_bank_from_schemas` → Already in `ingest_all_tests` (MOVE)
10. `ingest_all_defaults_schema_version_when_omitted` → Already in `ingest_all_tests` (MOVE)

**Existing modules (18 tests - ALREADY ORGANIZED)**:
- `staleness_tests` (8 tests) ✅
- `property_bank_result_tests` (5 tests) ✅
- `schema_ingest_result_tests` (3 tests) ✅
- `ingest_all_tests` (3 tests) ✅ BUT has duplicate tests as standalone!

### loader.rs Tests (7 tests total)

**Standalone tests (5 tests - NEED ORGANIZATION)**:
1. `new_schema_uses_full_resolution` → Move to `pipeline_tests`
2. `existing_schema_file_change_uses_full_resolution` → Move to `pipeline_tests`
3. `existing_schema_bank_change_uses_incremental` → Move to `incremental_resolution_tests`
4. `no_incremental_when_property_unchanged` → Move to `incremental_resolution_tests`
5. `mixed_scenario_handles_all_three_paths` → Move to `pipeline_tests`

**Existing modules (2 tests - ALREADY ORGANIZED)**:
- `cached_expansion_tests` (2 tests) ✅

## Target Organization

### ingestor.rs - Final Structure

```
#[cfg(test)]
mod tests {
    // Test helpers (keep at top level)
    fn write_file()
    fn test_config()
    fn test_repository()

    // NEW MODULE: Property bank loading tests
    mod property_bank_loading_tests {
        use super::*;

        #[test] fn parses_valid_json()
        #[test] fn parses_valid_yaml()
        #[test] fn parses_valid_toml()
        #[test] fn returns_error_for_invalid_json()
        #[test] fn returns_error_for_unsupported_format()
        #[test] fn defaults_version_when_omitted()
    }

    // EXISTING MODULE: Keep as-is
    mod property_bank_result_tests { ... }

    // CONSOLIDATE: Move standalone tests here
    mod ingest_all_tests {
        use super::*;

        // Existing tests
        #[test] fn ingest_all_with_new_files()
        #[test] fn ingest_all_with_empty_schemas_dir()
        #[test] fn ingest_all_with_multiple_schemas()

        // MOVE from standalone
        #[test] fn returns_both_property_bank_and_schemas()
        #[test] fn supports_toml_format()
        #[test] fn separates_property_bank_from_schemas()
        #[test] fn defaults_schema_version_when_omitted()
    }

    // EXISTING MODULE: Keep as-is
    mod schema_ingest_result_tests { ... }

    // EXISTING MODULE: Keep as-is
    mod staleness_tests { ... }
}
```

### loader.rs - Final Structure

```
#[cfg(test)]
mod tests {
    // Test helpers (keep at top level)
    struct TestDbContext { ... }
    fn write_file()
    fn test_config()

    // NEW MODULE: Pipeline integration tests
    mod pipeline_tests {
        use super::*;

        #[test] fn new_schema_uses_full_resolution()
        #[test] fn existing_schema_file_change_uses_full_resolution()
        #[test] fn mixed_scenario_handles_all_three_paths()
    }

    // NEW MODULE: Incremental resolution tests
    mod incremental_resolution_tests {
        use super::*;

        #[test] fn existing_schema_bank_change_uses_incremental()
        #[test] fn no_incremental_when_property_unchanged()
    }

    // EXISTING MODULE: Keep as-is
    mod cached_expansion_tests { ... }
}
```

## Implementation Steps

1. ✅ **Create new module `property_bank_loading_tests` in ingestor.rs**
   - Move 6 standalone property_bank tests into this module
   - Remove original standalone test functions

2. ✅ **Consolidate `ingest_all` tests in ingestor.rs**
   - Move 4 standalone ingest_all tests into existing `ingest_all_tests` module
   - Remove original standalone test functions

3. ✅ **Create new module `pipeline_tests` in loader.rs**
   - Move 3 pipeline tests into this module
   - Remove original standalone test functions

4. ✅ **Create new module `incremental_resolution_tests` in loader.rs**
   - Move 2 incremental tests into this module
   - Remove original standalone test functions

5. ✅ **Verify all tests still pass**
   - Run `cargo nextest run --lib -p lithos-core -E 'test(schema)'`

6. ✅ **Commit changes**

## Benefits

- **Readability**: Related tests are visually grouped
- **Coverage analysis**: Easy to see what behaviors are tested
- **Navigation**: Jump to test category quickly
- **Naming**: Shorter test names (module provides context)
- **Consistency**: Matches matklad's testing best practices

## Test Count Verification

Before: 28 tests in ingestor.rs, 7 tests in loader.rs (35 total)
After: 28 tests in ingestor.rs, 7 tests in loader.rs (35 total)

Must maintain exact same test count!
