---
title: "Issue 01: Introduce shared ConfigSpec projection error boundary"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
date_completed: null
---

# Issue 01: Introduce shared ConfigSpec projection error boundary

Labels: `ready-for-agent`
Type: AFK

## What to build

Introduce a shared projection error contract for ConfigSpec generation so downstream contexts can depend on a narrow enum instead of importing `ConfigError`.

## Scope and rationale

This issue is intentionally separated from `pathkey-migration` because the projection error contract should apply to all ConfigSpec types, not only `SchemaConfigSpec`.

## Agent Brief

**Category:** enhancement
**Summary:** Define and adopt a dedicated projection error enum in `error.rs` for config-to-spec projection seams.

**Current behavior:**
- `Config::to_schema_spec()` currently returns `Result<SchemaConfigSpec, ConfigError>`.
- `schema::builder` stringifies projection failures at the boundary.

**Desired behavior:**
- Projection methods return a narrow projection error enum defined in `lithos-core/src/config/error.rs`.
- Downstream contexts map projection errors via typed variants, not string interpolation.
- Pattern is reusable for all ConfigSpec methods (schema, task, frontmatter, future specs).

## Key interfaces

1. **Error location (required):**
- `lithos-core/src/config/error.rs`

2. **Projection contract (initial seam):**
- `Config::to_schema_spec() -> Result<SchemaConfigSpec, ConfigSpecProjectionError>`

3. **Boundary mapping seam:**
- `lithos-core/src/schema/builder.rs` maps `ConfigSpecProjectionError` to schema ingestion errors without importing `ConfigError`.

## Initial variant set

Keep this enum minimal and projection-focused. Initial variants should cover at least:
- Invalid schema directory declaration projection.
- Invalid property bank declaration projection.

Avoid broad umbrella variants that recreate `ConfigError`.

## Acceptance criteria

- [ ] `ConfigSpecProjectionError` is defined in `lithos-core/src/config/error.rs`.
- [ ] `Config::to_schema_spec()` returns `Result<SchemaConfigSpec, ConfigSpecProjectionError>`.
- [ ] `schema::builder` uses typed mapping for projection failures and removes string-based adaptation.
- [ ] `schema::builder` does not need to import full `ConfigError` for this seam.
- [ ] Tests cover both projection failure mapping and success path behavior.
- [ ] Design/docs note this as the shared pattern for future ConfigSpec projection methods.

## Impacted symbols and files

Primary symbols:
- `Function:lithos-core/src/config/aggregate.rs:Config.to_schema_spec#0`
- `Function:lithos-core/src/schema/builder.rs:load_all`

Likely files:
- `lithos-core/src/config/error.rs`
- `lithos-core/src/config/aggregate.rs`
- `lithos-core/src/schema/builder.rs`

## TDD plan

### 1. Tracer bullet
**Behavior:** projection failure is represented as a dedicated typed error.
- **RED:** Add a failing test asserting `to_schema_spec` returns `ConfigSpecProjectionError` on invalid projection input.
- **GREEN:** Add enum in `error.rs` and update signature.

### 2. Incremental loop
**Behavior:** schema builder maps projection failure via typed conversion.
- **RED:** Add failing test in builder path verifying typed mapping behavior.
- **GREEN:** Replace string interpolation seam with typed mapping.

### 3. Refactor
- [ ] Keep enum scoped to projection concerns.
- [ ] Reuse shared conversion helpers for future spec projection methods.
- [ ] Keep context boundaries clean (no unnecessary cross-context imports).

## Verification

Run:
- `mise run fmt`
- `mise run lint`
- `mise run test:unit`
- `mise run test`
