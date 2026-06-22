---
labels: ["ready-for-agent"]
---

## Parent

PRD: `.scratch/workspace-refactoring/PRD.md`

## What to build

Initialize a true multi-crate Rust workspace by extracting the contexts currently housed inside `lithos-core` into their own separate crates.

1. **Workspace Setup**: Create a `crates/` directory and a top-level workspace `Cargo.toml`.
2. **Context Extraction**: Move the contexts (`note`, `schema`, `template`, `indexer`, `db`, `fs`, `utils`, `support`, `app`) from `lithos-core/src/` into their own crate directories (e.g., `crates/note`).
3. **Cargo Configuration**: Give each extracted crate a `Cargo.toml` with the package name prefixed with `trace-` (e.g., `name = "trace-note"`). Ensure dependencies point correctly between them locally.
4. **Visibility Fixes**: Because contexts are now physical crates, `trace-support` must expose its internals as `pub`. Annotate these with `#[doc(hidden)]` and set `publish = false` in `trace-support/Cargo.toml`. Domain crates using these types should keep their usage hidden via `pub(crate)` where necessary.
5. **Test Infrastructure Splitting (Crucial Step)**:
   - Move `benches/note_parsing.rs` to `crates/note/benches/`.
   - Move `benches/schema_loader.rs.disabled` to `crates/schema/benches/`.
   - Move `benches/db_key_handling.rs` and `db_storage.rs` to `crates/db/benches/`.
   - Move `benches/string_construction.rs` to `crates/utils/benches/`.
   - Move `tests/schema_loader.rs` and `schema_storage.rs` to `crates/schema/tests/`.
   - Move `tests/note_ingest.rs`, `note_lexical_policy_integration.rs`, and `note_reader.rs` to `crates/note/tests/`.
   - Move `tests/architecture.rs` to `crates/app/tests/architecture.rs` and update globs to search `../../crates/**/src/**/ports.rs`.
   - Move `TestDb` from `tests/common/mod.rs` to `crates/db/src/testing.rs` (exposed behind a `#[cfg(any(test, feature = "testing"))]` flag) so other crates can use it.
   - Push `setup_repository` down into the `tests/common/mod.rs` of the specific domain crates (Note, Schema).
   - Move `PropertyBuilder`, `bool_property`, and `RepositoryExt` from `tests/common/mod.rs` into `crates/schema/tests/common/mod.rs`.
6. Fix all `use lithos_core::*` imports globally to use the new `trace_*` paths.

## Acceptance criteria

- [ ] The `crates/` folder contains all contexts extracted from `lithos-core`, each with a working `Cargo.toml` using the `trace-` prefix.
- [ ] `trace-support` types are `pub` but marked `#[doc(hidden)]`.
- [ ] Tests and benches are successfully moved to their specific domain crates or `trace-app/tests/`.
- [ ] `TestDb` is available in `trace-db::testing` for other crates to consume in their tests.
- [ ] The workspace compiles (`cargo check`).
- [ ] All unit and integration tests pass (`cargo test`).

## Blocked by

- None - can start immediately
## Agent Brief

**Category:** enhancement
**Summary:** Extract internal contexts into a `crates/` workspace to statically enforce architectural boundaries.

**Current behavior:**
Contexts like `note`, `db`, `schema`, and `support` are all submodules inside a single `lithos-core` crate. Boundary enforcement relies purely on integration tests (like `tests/architecture.rs`). Test infrastructure is globally mixed across domains (e.g., `TestDb` alongside domain-specific property builders in `tests/common/mod.rs`).

**Desired behavior:**
Every context becomes a standalone Cargo crate under `crates/` prefixed with `trace-` (e.g., `trace-note`). The Rust compiler natively enforces the acyclic dependency graph. `trace-support` is made globally visible across the workspace but hidden from public documentation. Tests and test utilities are pushed down into their respective domain crates or centralized in `trace-app` if they test multiple crates.

**Key interfaces:**
- Workspace and crate-level `Cargo.toml` files (must correctly map `trace-` dependencies).
- `trace-support` structs (e.g. `Blake3Hash`) — must be `pub` but marked `#[doc(hidden)]`, and the crate must set `publish = false`.
- `TestDb` — must move to `trace-db/src/testing.rs` and be exposed via a `testing` feature or `cfg(test)` so other crates can use it.
- `setup_repository` — must be pushed down into specific domain tests (e.g. `trace-schema/tests/common/mod.rs`).

**Acceptance criteria:**
- [ ] The `crates/` folder contains all contexts extracted from `lithos-core`, each with a working `Cargo.toml` using the `trace-` prefix.
- [ ] `trace-support` types are `pub` but marked `#[doc(hidden)]`.
- [ ] Tests and benches are successfully moved to their specific domain crates or `trace-app/tests/`.
- [ ] `TestDb` is available in `trace-db::testing` for other crates to consume in their tests.
- [ ] The workspace compiles (`cargo check`).
- [ ] All unit and integration tests pass (`cargo test`).

**Out of scope:**
- Merging `config` and `discovery` (handled in a separate slice).
- Renaming "Lithos" text strings globally (handled in a separate slice).
- Moving RedbRepository out of domain crates into an infrastructure crate.
