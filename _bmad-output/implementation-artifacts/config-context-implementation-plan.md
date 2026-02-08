# Config Context Implementation Plan (Progress Tracker)

Purpose: Track tasks required to implement the config design specs in
`docs/design/001-config-models.md`, `docs/design/002-config-cqrs.md`, and
`docs/design/003-config-task.md`, with test updates and frequent pre-commit
checks.

Conventions:
- Each task includes explicit test updates and a pre-commit check.
- Use `mise run <task>` for tooling consistency.
- Pre-commit hooks must pass at each checkpoint (run via `mise run verify`).

## 0) Baseline Review and Gap Map
- [ ] Review current config implementation for gaps vs specs (models, CQRS, task config, ingest).
- [ ] Record any required ADR updates or migrations (if on-disk formats change).
- [ ] Update or add review tests for identified gaps (unit tests under `lithos-core/src/config/`).
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 1) Raw Input Types + Ingest Boundary (Figment)
- [ ] Add `lithos-core/src/config/raw.rs` with `RawGlobal`, `RawVault`, and `RawTaskConfig` shapes.
- [ ] Add `lithos-core/src/config/ingest.rs` with Figment provider wiring and `ingest_global`/`ingest_vault`.
- [ ] Implement `TryFrom<Raw*>` -> validated domain types; keep Figment out of domain modules.
- [ ] Add unit tests for raw deserialization and conversion errors (missing/invalid fields, unknown keys policy).
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 2) Domain Type Refactor (Models + Newtypes)
- [ ] Replace empty-string sentinels with `Option<T>` overlays in vault overrides.
- [ ] Introduce newtypes per spec: `VaultId`, `VaultRoot`, `VaultPathKey`, `FrontmatterKey`, `LogLevel`, etc.
- [ ] Remove `SchemaVersion` deref; add `as_str()` and `Display`.
- [ ] Convert `TrustedVaults` to `enum TrustedVaults { List, Map }` with `#[serde(untagged)]`.
- [ ] Update `Schema::property_bank_path()` to return `PathBuf` using join semantics.
- [ ] Update config aggregate/build logic to use Option overlays (no empty-string checks).
- [ ] Update and add unit tests for newtypes, validation, and merge precedence.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 3) Task Config Schema (Cross-Cutting Infrastructure)
- [ ] Add `lithos-core/src/config/task.rs` with `TaskConfig`, `TaskTag`, `TaskFieldKeyword`, `StatusName`, `StatusSymbol`.
- [ ] Implement `Bounds<T>`, `DateFieldSpec`, `TaskFieldSpec` with validation + regex compile + chrono format checks.
- [ ] Add `TaskConfig::from_raw` and default config matching current checkbox behavior.
- [ ] Add validation and parsing helpers (`field_spec`, `parse_date_value`, status mapping lookups).
- [ ] Add unit tests for task tags, status mapping, bounds, regex, date parsing, and indexed fields.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 4) CQRS Refactor (Ports, Errors, Commands, Queries)
- [ ] Update `ports.rs` to split `ConfigCommandPort` and `ConfigQueryPort` with GATs.
- [ ] Add `ConfigCommandError` and `ConfigQueryError` (structured storage/domain split).
- [ ] Update `command.rs` and `query.rs` to be generic over ports and return split errors.
- [ ] Implement command-side `save_global`, `save_vault`, `load_global`, `load_vault`.
- [ ] Implement query-side `get(vault_id)` (merged read model only).
- [ ] Add unit tests for command/query behavior and error mapping.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 5) Versioned Merged Config Read Model
- [ ] Add `ConfigVersion`, `MergedConfigRecord`, `ActiveMergedConfig` types.
- [ ] Implement `rebuild_merged`, `activate_version`, and optional `rollback` in command.
- [ ] Add DB table mapping: `vault_id_by_path`, `vault_path_by_id`, `merged_config_versions`, `merged_config_active`.
- [ ] Update adapters in `lithos-core/src/db/config_adapter.rs` if needed.
- [ ] Add tests for version creation, activation, and rollback behavior.
- [ ] Run `mise run test:unit:config` and `mise run test:unit:db` if adapter changes.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 6) Aggregate Build and Merge Updates
- [ ] Update `Config::build` signature and metadata to use `VaultId` + `VaultRoot` (per spec).
- [ ] Ensure merge precedence is explicit and deterministic (vault > global > defaults).
- [ ] Ensure config events remain valid and contain structured source when required.
- [ ] Update tests that assume fixed vault path or string-based metadata.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 7) Integration Touchpoints (Note/CLI)
- [ ] Wire TaskConfig into config loading and note parsing interfaces (no context cross-imports).
- [ ] Update any CLI or adapter boundaries that depend on old config APIs.
- [ ] Add integration tests (if applicable) under `lithos-core/tests/`.
- [ ] Run `mise run test:integration` (if integration tests changed).
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

## 8) Final Quality Gate and Checkpoint Commit
- [ ] Run `mise run verify` (full pre-commit hooks + tests + adr checks).
- [ ] Confirm all updated tests pass and no clippy warnings.
- [ ] Stage and commit checkpoint with a concise message (if requested).

## Notes
- Pre-commit checks should be run frequently via `mise run verify` so hooks
  pass before each checkpoint commit.
- If any on-disk format changes are introduced, record migration notes or ADRs.
