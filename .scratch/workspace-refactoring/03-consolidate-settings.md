---
labels: ["ready-for-agent"]
---

## Parent

PRD: `.scratch/workspace-refactoring/PRD.md`

## What to build

Merge the isolated configuration and discovery mechanisms into a single, cohesive Settings adapter.

1. Create a new crate: `crates/settings` (Package name: `trace-settings`).
2. Move the contents of `crates/config` and `crates/discovery` (which now includes `dirs.rs` and `env.rs`) into this unified `trace-settings` crate.
3. Update `trace-settings` internal modules to expose the necessary APIs.
4. Update `trace-app` and any other dependent crates to use `trace-settings` instead of the disparate config/discovery crates.
5. Update `crates/settings/tests/architecture.rs` to ensure any internal module boundaries between the config parser and the discovery engine are maintained.
6. Delete the old `crates/config` and `crates/discovery` folders.

## Acceptance criteria

- [ ] `crates/settings` exists and contains the logic for config, discovery, dirs, and env.
- [ ] The old `config` and `discovery` crates are deleted.
- [ ] `trace-app` successfully orchestrates application workflows using `trace-settings`.
- [ ] The workspace compiles and all tests pass.

## Blocked by

- `.scratch/workspace-refactoring/01-extract-core-contexts.md`
## Agent Brief

**Category:** enhancement
**Summary:** Merge configuration and discovery modules into a unified `trace-settings` crate.

**Current behavior:**
Discovery (finding vault/config files) and Configuration (parsing and prioritizing them) are separated into disparate modules (`lithos-core/src/config/`, `lithos-core/src/discovery/`, `dirs.rs`, `env.rs`). This splits the inbound settings lifecycle across too many boundaries.

**Desired behavior:**
A single `trace-settings` crate houses everything related to discovering paths, reading environments, and parsing configuration. The application bootstrapper (in `trace-app`) consumes this unified adapter to load its settings.

**Key interfaces:**
- `trace-settings` root module structure — must cohesively export both the config parsing types and the discovery engine results.
- `trace-app` bootstrapper (`crates/app/src/bootstrap.rs` or similar) — must be updated to use `trace-settings`.
- `crates/settings/tests/architecture.rs` — updated to ensure internal module boundaries between the config parser and the discovery engine are maintained.

**Acceptance criteria:**
- [ ] `crates/settings` exists and contains the logic for config, discovery, dirs, and env.
- [ ] The old `config` and `discovery` crates are deleted.
- [ ] `trace-app` successfully orchestrates application workflows using `trace-settings`.
- [ ] The workspace compiles and all tests pass.

**Out of scope:**
- Changing configuration formats or adding new settings.
