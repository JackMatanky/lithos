# Findings - Property Bank Processor Test Normalization

## Unit Test Suite Design (Structure A)

The `PropertyBankProcessor` unit tests will be organized into focused submodules by unit of work, following `unit-naming.md`.

### Module Tree
```text
mod tests
  ├─ mod fixtures (Setup helpers)
  ├─ mod constructor (from_discovery)
  ├─ mod run (High-level orchestration)
  ├─ mod comparison (check_timestamps, check_content)
  ├─ mod parse (parsing logic for Path A and Path B)
  ├─ mod analysis (analyze logic: Empty, Delta, Corrupt)
  ├─ mod refresh (sync_metadata)
  └─ mod construction (create, update, fetch)
```

### Specific Test Cases

#### mod constructor
- `from_discovery_returns_processor_with_unknown_status`
- `from_discovery_returns_error_when_path_key_fails`

#### mod run (Happy Paths & Regressions)
- `run_executes_missing_path_for_initial_load`
- `run_executes_fresh_path_when_timestamps_match`
- `run_executes_refresh_path_when_content_matches`
- `run_executes_delta_path_when_properties_change`
- `run_executes_corrupt_path_when_view_is_malformed`

#### mod comparison
- `check_timestamps_returns_match_when_identical`
- `check_timestamps_returns_mismatch_when_mtime_drifted`
- `check_content_returns_match_when_hash_identical`
- `check_content_returns_mismatch_when_hash_different`

#### mod parse
- `parse_missing_returns_new_processor_with_raw_bank`
- `parse_stale_returns_analysis_processor_with_raw_bank`
- `parse_returns_error_when_syntax_invalid`

#### mod analysis
- `analyze_returns_empty_when_properties_unchanged`
- `analyze_returns_delta_when_properties_differ`
- `analyze_returns_corrupt_when_view_missing_version`
- `analyze_returns_corrupt_when_delta_engine_fails`

#### mod refresh
- `sync_metadata_updates_timestamps_only_for_stale_ts`
- `sync_metadata_updates_hash_and_ts_for_stale_content`

#### mod construction
- `create_persists_bank_and_view`
- `update_applies_deltas_and_persists`
- `fetch_retrieves_existing_bank_from_repo`
- `fetch_returns_error_when_bank_missing`

## Integration Test Consolidation
- `tests/property_bank_processor.rs` is redundant with `tests/schema_loader.rs`.
- Unique check: `repository2.get_raw_property_bank_view(&bank_path)?` should be added to `schema_loader.rs`.

## Visibility Reduction
- All Stage markers (`Comparison`, `Analysis`, etc.) can be private.
- All Status structs (`Suspect`, `Stale`, etc.) can be private.
- All transition methods and branch enums can be private.
- `PropertyBankProcessor<P, S>`, `PropertyBankResolution`, `Init`, and `Unknown` remain `pub(crate)`.
