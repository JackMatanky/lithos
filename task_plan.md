# Paths Refactor Assessment Plan

## Goal

Assess the implications of splitting `lithos-core/src/config/paths.rs` and refactoring path domain types before implementation.

## Scope

- Understand `lithos-core/src/config/paths.rs`, `raw.rs`, and `processor.rs`.
- Analyze splitting `Paths` into `CacheConfig`, `TemplateConfig`, and `SchemaConfig`.
- Analyze converting `Cache`, `Template`, `Schema`, and `PropertyBank` into tuple structs renamed to `CacheDir`, `TemplateDir`, `SchemaDir`, and `PropertyBankFile`.
- Identify blast radius, side effects, migration questions, and testing needs.
- Defer `RawPathsConfig`, `RawVaultPaths`, `RawGlobalPaths`, and `schema/config.schema.json` updates to a follow-up refactor to keep this refactor small.

## Phases

| Phase | Status | Notes |
| --- | --- | --- |
| Initialize planning files | complete | Root planning files created for current assessment. |
| GitNexus exploration | complete | Query, context, impact, and concrete source seams reviewed. |
| Refactor implication analysis | complete | API, archive, raw config, docs, tests, and downstream schema discovery impacts recorded. |
| Grill decision tree | complete | Main design decisions resolved. |

## Decisions Pending

- None for current assessment.

## Decisions Made

- Preferred direction: move away from `Paths` structs as long-term API boundaries because downstream contexts should consume `to_*_spec()` methods instead of full path aggregates.
- `Config` should split its resolved path storage into private `cache: CacheConfig`, `template: TemplateConfig`, and `schema: SchemaConfig` fields.
- No compatibility is required for previously persisted rkyv-serialized `Config` records.
- This refactor should intentionally break the current public Rust API now rather than staging deprecated `Paths` compatibility shims.
- Path config code should be split into separate `cache.rs`, `template.rs`, and `schema.rs` files.
- The split files should live directly under `lithos-core/src/config/`.
- `SchemaConfigSpec` should move from `config::paths::SchemaConfigSpec` to `config::schema::SchemaConfigSpec`.
- Existing `TemplateConfigSpec` should move from `config::paths::TemplateConfigSpec` to `config::template::TemplateConfigSpec`.
- A new `CacheConfigSpec` should be introduced.
- `CacheConfigSpec` should mirror `TemplateConfigSpec`: root plus relative directory, with absolute path and path key projection methods.
- `config::global::Paths` and `config::vault::Paths` should be removed in the same refactor; raw path DTOs remain unchanged only as a scope-control decision.
- `PropertyBankFile` should be owned by `SchemaConfig`.
- A follow-up refactor will update `RawPathsConfig`, `RawVaultPaths`, `RawGlobalPaths`, and `schema/config.schema.json`.

## Errors Encountered

| Error | Attempt | Resolution |
| --- | --- | --- |
| None | 1 | N/A |
