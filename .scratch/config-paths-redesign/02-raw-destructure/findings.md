# Findings: Config Paths Redesign Phase 2

## GitNexus Impact Analysis Results

Removing/changing `RawPathsConfig`, `RawGlobalPaths`, and `RawVaultPaths` will affect:

1. **`builder.rs`**:
   - `build_from_layers` calls `RawPathsConfig::merge` and consumes the merged paths object.
   - `load_applies_local_config_from_vault_root_marker` test creates raw string toml configuration containing `[paths] templates_dir`.
2. **`raw.rs`**:
   - `raw_vault_config_serializes_and_roundtrips` and other internal parsing tests.
3. **`merger.rs`**:
   - `create_test_global_config` (d=1, WILL BREAK)
   - `create_test_vault_config` (d=1, WILL BREAK)
   - `resolve_both_rebuild_returns_rebuild_with_both_layers` (d=2)
4. **`aggregate.rs`**:
   - `merged_config_with_sample_overrides` (d=1, WILL BREAK)
   - `applies_paths_fields_from_raw` (d=1, WILL BREAK)
   - `to_schema_spec_respects_custom_paths` (d=1, WILL BREAK)
   - Note: Some of these look like test names, confirming testing blast radius is large but contained to config.

## Domain Model
From `CONTEXT.md` and ADR 020, configuration should be Declarative Paths (`RelativeDirPath`, `RelativeFilePath`).

The proposed structure will map TOML like:
```toml
[cache]
dir = ".cache"

[template]
dir = "custom-templates"

[schema]
dir = "schemas"
property_bank_file = "bank.json"
```
instead of the previous monolithic `[paths]` section.
