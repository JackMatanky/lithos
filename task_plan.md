# Paths Refactor Assessment Plan

## Goal

Assess the implications of splitting `lithos-core/src/config/paths.rs` and refactoring path domain types before implementation.

## Scope

- Understand `lithos-core/src/config/paths.rs`, `raw.rs`, and `processor.rs`.
- Analyze splitting `Paths` into `CacheConfig`, `TemplateConfig`, and `SchemaConfig`.
- Analyze converting `Cache`, `Template`, `Schema`, and `PropertyBank` into tuple structs renamed to `CacheDir`, `TemplateDir`, `SchemaDir`, and `PropertyBankFile`.
- Identify blast radius, side effects, migration questions, and testing needs.

## Phases

| Phase | Status | Notes |
| --- | --- | --- |
| Initialize planning files | complete | Root planning files created for current assessment. |
| GitNexus exploration | complete | Query, context, impact, and concrete source seams reviewed. |
| Refactor implication analysis | complete | API, archive, raw config, docs, tests, and downstream schema discovery impacts recorded. |
| Grill decision tree | in_progress | Ask one unresolved design question at a time. |

## Decisions Pending

- Whether `Paths` remains as a temporary compatibility aggregate or is removed from the resolved config API.
- Whether tuple-struct renames are intended as a public API break or should be staged.
- Whether rkyv archive compatibility matters for persisted resolved config data.

## Decisions Made

- Preferred direction: move away from `Paths` structs as long-term API boundaries because downstream contexts should consume `to_*_spec()` methods instead of full path aggregates.
- `Config` should split its resolved path storage into private `cache: CacheConfig`, `template: TemplateConfig`, and `schema: SchemaConfig` fields.

## Errors Encountered

| Error | Attempt | Resolution |
| --- | --- | --- |
| None | 1 | N/A |
