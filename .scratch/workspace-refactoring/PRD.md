---
labels: ["ready-for-agent"]
---

# PRD: Workspace Refactoring and Project Rename to Trace

## Problem Statement

The `lithos` project is currently structured as a monolithic core crate (`lithos-core`) housing multiple discrete contexts (`note`, `schema`, `db`, `fs`, `config`, etc.), and a CLI crate (`lithos-cli`). As the application grows in complexity, managing dependencies, preventing circular imports, and maintaining clear architectural boundaries within a single crate becomes increasingly difficult. The single-crate approach relies entirely on manual architectural checks (like `architecture.rs`) to prevent contexts from tangling. Furthermore, the project name "Lithos" is being updated to "Trace", which requires a global rename to align all documentation, paths, and identifiers.

## Solution

The solution is a "Big Bang" architectural redesign that breaks `lithos-core` into a true multi-crate Rust workspace under a new `crates/` directory, while simultaneously executing a global project rename from "Lithos" to "Trace". Each distinct context will be extracted into its own statically-isolated crate with explicit `Cargo.toml` dependencies, enforcing an acyclic dependency graph at compile time. `config` and `discovery` will be merged into a cohesive `settings` crate, while infrastructure layers like `db` and `fs` will remain isolated to preserve Hexagonal Architecture boundaries.

## User Stories

1. As a developer, I want all contexts to be physically separated into their own Cargo crates, so that the Rust compiler enforces an acyclic dependency graph and prevents accidental domain coupling.
2. As a developer, I want the project and all documentation to be globally renamed to "Trace", so that the new project identity is consistent across the codebase.
3. As a developer, I want internal support types (e.g., Blake3Hash) to be available across the workspace but hidden from crates.io documentation, so that I can share code securely without exposing internal APIs.
4. As an architect, I want `config` and `discovery` merged into a single `trace-settings` crate, so that the configuration lifecycle is cohesive and managed as a single inbound adapter.
5. As a maintainer, I want all workspace members to use the `trace-` prefix for their Cargo package names (e.g., `trace-note`), so that we can safely publish them to crates.io in the future without name collisions.
6. As a developer, I want testing infrastructure like `TestDb` moved to `trace-db/src/testing.rs` behind a feature flag, so that multiple domain crates can spin up isolated test databases without duplicating setup code.
7. As an architect, I want integration tests spanning multiple crates to be relocated to the orchestrating `trace-app` crate, so that the entire system can be tested as a cohesive unit.
8. As a developer, I want the `architecture.rs` tests updated to reflect the new boundaries, so that I don't run tests that the compiler already guarantees natively via Cargo.
9. As a developer, I want the composition root extracted into `trace-app`, so that the CLI is solely responsible for terminal output and doesn't handle application wiring.

## Implementation Decisions

- **Execution Strategy:** We will execute a "Big Bang" migration. The codebase will undergo a single commit where `lithos` is renamed to `trace`, the `crates/` directory is populated, and all `Cargo.toml` files and `use` imports are updated simultaneously.
- **Workspace Layout:**
  - `crates/app` (`trace-app`): The orchestrator and unified facade.
  - `crates/cli` (`trace-cli`): The command-line frontend.
  - `crates/settings` (`trace-settings`): Combines previous `config` and `discovery`.
  - `crates/indexer` (`trace-indexer`)
  - `crates/note` (`trace-note`)
  - `crates/schema` (`trace-schema`)
  - `crates/template` (`trace-template`)
  - `crates/db` (`trace-db`)
  - `crates/fs` (`trace-fs`)
  - `crates/utils` (`trace-utils`)
  - `crates/support` (`trace-support`)
- **Naming and Publishing:** Crate directories will be clean (e.g., `crates/note`), but `Cargo.toml` package names will use the `trace-` prefix (`trace-note`) to prevent crates.io collisions.
- **Internal Visibility:** `trace-support` items will be made `pub` but annotated with `#[doc(hidden)]`, and `publish = false` will be set in its `Cargo.toml`. Consuming domain crates will wrap these internals in `pub(crate)` constructs to maintain their public API cleanliness.
- **Hexagonal Realignment (Step 1):** We will maintain `trace-db` and `trace-fs` as distinct outbound and inbound infrastructure crates. (Note: Moving `storage/` and `RedbRepository` out of the domain crates is reserved for a future PRD).

## Testing Decisions

- **Guiding Principle:** The restructure should not change application behavior. Existing unit and integration tests must pass without their core assertions being modified.
- **Test Infrastructure (`TestDb`):** Relocated from `tests/common/mod.rs` to `crates/db/src/testing.rs`. It will be exposed to other crates to enable RAII tempdir databases for testing.
- **Test Helpers (`setup_repository`):** Pushed down into individual domain crates (`crates/note/tests/common/mod.rs` and `crates/schema/tests/common/mod.rs`).
- **Domain Mocks (`PropertyBuilder`, `RepositoryExt`):** Relocated specifically to `crates/schema/tests/common/mod.rs` as they are tightly bound to the schema domain models.
- **Architecture Tests (`architecture.rs`):**
  - **Deleted:** `contexts_must_not_import_each_other` (Now statically enforced by Cargo).
  - **Relocated & Updated:** `ports_must_not_import_std_fs` moves to `crates/app/tests/architecture.rs` and will glob `../../crates/**/src/**/ports.rs`.
  - **Relocated:** Config vs. Discovery module tests move to `crates/settings/tests/architecture.rs` to enforce internal boundaries.
- **Domain vs. Integration Tests:** Tests residing in `lithos-core/benches/` and `lithos-core/tests/` will be routed to their respective domain crates if they only test a single domain. Tests requiring multiple systems (like the DB, Settings, and Note working together) will be routed to `crates/app/tests/`.

## Out of Scope

- Refactoring `RedbRepository` implementations out of the domain crates (`note`, `schema`, `template`) into `trace-infrastructure`. This Hexagonal architectural shift is slated for a future design change.
- Adding new feature functionality. This PRD strictly covers the structural workspace split and rename.

## Further Notes

- The project glossary and documentation (`CONTEXT-MAP.md`, `README.md`) have been updated inline during the design phase to reflect the name "Trace" and the new `crates/` pathways.
