# Phase 2: Raw & Schema Destructuring

## Goal
Destructure the `paths` section in `RawGlobalConfig`, `RawVaultConfig`, and `config.schema.json` to match the domain split (Cache, Template, Schema) completed in Phase 1, formalizing configuration around the Three-Tier Path Taxonomy (ADR 020).

## Current State
**Status:** `completed`

## Plan

### Phase 1: Update `raw.rs` DTOs
- [x] Remove `RawPathsConfig`, `RawGlobalPaths`, `RawVaultPaths`.
- [x] Create `RawCacheConfig` with a `directory: Option<String>` field.
- [x] Create `RawTemplateConfig` with a `directory: Option<String>` field.
- [x] Create `RawSchemaConfig` with `directory: Option<String>` and `property_bank_file: Option<String>` fields.
- [x] Add `template` and `schema` fields to `RawGlobalConfig` (optional/default).
- [x] Add `cache`, `template`, and `schema` fields to `RawVaultConfig` (optional/default).

### Phase 2: Update `builder.rs` Merging
- [x] Refactor `build_from_layers` to stop using `RawPathsConfig::merge`.
- [x] Instead, resolve Option values directly at the `builder.rs` level using `.or_else()` before constructing domain types (Option B approach).
- [x] Maintain validation per ADR 020: parse using `RelativeDirPath`/`RelativeFilePath` (via their respective domain Config constructors).

### Phase 3: Update JSON Meta-schema
- [x] In `schema/config.schema.json`, remove the `paths` object property.
- [x] Add `cache` object to `VaultConfig`.
- [x] Add `template` object to `VaultConfig` and `GlobalConfig`.
- [x] Add `schema` object to `VaultConfig` and `GlobalConfig`.
- [x] Map the previous `cache_dir`, `templates_dir`, `schemas_dir`, `property_bank_file` settings into `directory` and `property_bank_file` fields on these objects.

### Phase 4: Test Updates
- [x] Update `lithos-core/src/config/raw.rs` tests (e.g., `raw_vault_config_serializes_and_roundtrips`, `raw_vault_config_supports_partial_paths`)
- [x] Update `lithos-core/src/config/merger.rs` tests (`create_test_global_config`, `create_test_vault_config`, `resolve_both_rebuild...`)
- [x] Update `lithos-core/src/config/aggregate.rs` tests (`merged_config_with_sample_overrides`, `applies_paths_fields_from_raw`, `to_schema_spec_respects_custom_paths`)
- [x] Update `lithos-core/src/config/builder.rs` tests (`load_applies_local_config_from_vault_root_marker`)

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| `ConfigField::Paths` compilation error in tests | 1 | Removed references to `ConfigField::Paths` in `processor.rs` tests. |
| Test failure in `config_field_hashes_diff_detects_new_fields` | 2 | The diffing logic was assuming `Paths` was added by default. Updated the test to use `Template` instead. |
