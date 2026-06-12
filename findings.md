# Paths Refactor Findings

## Domain Context

- Config context language distinguishes `Resolved Config`, `Config Spec`, and `Declarative Paths`.
- Config invariants require downstream contexts to consume narrowed Config Specs rather than full resolved config.
- Declarative paths should remain `RelativeDirPath` / `RelativeFilePath` until resolved against a vault root.

## Initial File Review

- `paths.rs` currently mixes resolved path aggregate types, declarative path newtypes, `SchemaConfigSpec`, rkyv archive helpers, conversion from raw path DTOs, and tests.
- `raw.rs` owns serde DTOs and preserves external TOML field names: `cache_dir`, `templates_dir`, `schemas_dir`, `property_bank_file`.
- `processor.rs` hashes raw config top-level sections. Path refactor in resolved types should not affect field-level hash behavior unless raw DTOs or serialization change.

## GitNexus Findings

- Relevant execution flows include `applies_paths_fields_from_raw` and `to_schema_spec_respects_custom_paths` in `lithos-core/src/config/aggregate.rs`.
- `Paths` and `SchemaConfigSpec` struct-level upstream impact returned LOW risk with 0 direct graph dependents, but this likely undercounts field access/import fallout for Rust type refactors.
- GitNexus query surfaced config tests, `schema_loader` integration tests, config storage tests, and schema storage paths as related definitions to review.
- Exact type impact for `Cache`, `Template`, `Schema`, and `PropertyBank` also returned LOW risk with 0 graph dependents, but text search shows concrete imports in `config/global.rs`, `config/vault.rs`, and `config/aggregate.rs`.

## Refactor Surface

- `config::aggregate::Config` stores resolved `paths::Paths`, exposes `paths()`, and projects to `SchemaConfigSpec` via `to_schema_spec()`.
- `config::global::Paths` and `config::vault::Paths` are partial override structs that reuse `paths::{Template, Schema, PropertyBank}` and `paths::Cache`.
- `schema::discovery::DiscoveryEngine::run()` depends on `config::paths::SchemaConfigSpec`, not on the full resolved `Paths` aggregate.
- `raw.rs` external config DTO field names should remain unchanged unless the TOML schema is intentionally changed.
- `config::builder::build_from_layers()` merges raw global/vault path DTOs into one `RawPathsConfig`, then validates/defaults through `config::paths::Paths::try_from()`.
- `config::merger` operates on raw config layer outcomes only and should not need changes unless raw path DTOs or `ConfigField::Paths` semantics change.

## Likely Side Effects

- Public API imports and docs must change from `Cache`, `Template`, `Schema`, `PropertyBank` to `CacheDir`, `TemplateDir`, `SchemaDir`, `PropertyBankFile`.
- `ArchivedCache` and `ArchivedPaths` helper methods are generated from rkyv archive names; tuple struct conversion and renames will change archived type names and field accessors.
- Existing tests assert direct field chains such as `config.paths().schema.schemas_dir()` and `config.paths().cache.cache_dir()`; those would need accessor-based replacements.
- Renaming `Schema` and `Template` config path types reduces ambiguity with schema/template context aggregates.
- Splitting `Paths` into `CacheConfig`, `TemplateConfig`, and `SchemaConfig` can be source-compatible only if `Paths` remains as the resolved aggregate with renamed fields/accessors. Removing `Paths` would affect `Config`, `Vault`, docs, storage archives, and callers.
- If `PropertyBankFile` moves under `SchemaConfig`, `Config::to_schema_spec()` can become simpler: schema config can own both schema directory and property bank filename, preserving the downstream `SchemaConfigSpec` contract.

## Recommended Direction

- Keep raw DTOs unchanged: they are the external config file schema.
- Treat `Paths` as transitional if kept at all; the desired end-state is moving downstream usage to narrowed `to_*_spec()` methods rather than exposing full path aggregates.
- Put `SchemaDir` and `PropertyBankFile` inside `SchemaConfig`; this matches the domain relationship used by `to_schema_spec()`.
- Convert `CacheDir`, `TemplateDir`, `SchemaDir`, and `PropertyBankFile` to tuple newtypes with accessor methods rather than public fields.
- Preserve or intentionally replace `SchemaConfigSpec`; it is already a downstream context-facing Config Spec and should not be conflated with internal resolved `SchemaConfig`.
- Replace `Config`'s private `paths: Paths` storage with private `cache`, `template`, and `schema` resolved config fields.
- Add or preserve narrowed projection methods (`to_cache_spec()`, `to_template_spec()`, `to_schema_spec()`); keep a transitional `paths()` only if public API breakage needs staging.

## Open Questions

- Does persisted rkyv data need backward compatibility across this type rename and struct-shape change?
- Should `Paths` continue to expose fields named `cache`, `template`, `schema`, and `property_bank`, or should the public resolved config shape change?
