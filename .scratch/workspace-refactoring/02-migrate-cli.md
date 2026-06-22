---
labels: ["done"]
---

## Parent

PRD: `.scratch/workspace-refactoring/PRD.md`

## What to build

Migrate the command-line interface into the new workspace structure.

Move the existing `lithos-cli` directory into `crates/cli`. Update its `Cargo.toml` so the package name is `trace-cli`. Update its dependencies to import and rely on the newly extracted `trace-app` orchestrator crate instead of the legacy `lithos-core`. Fix any broken import paths resulting from the move.

## Acceptance criteria

- [x] `lithos-cli` has been successfully moved to `crates/cli`.
- [x] `crates/cli/Cargo.toml` package name is `trace-cli`.
- [x] The CLI relies on `trace-app` for its orchestration and domain logic.
- [x] The CLI builds successfully (`cargo build --bin trace-cli` or similar).
- [x] Any CLI-specific tests pass.

## Blocked by

- `.scratch/workspace-refactoring/01-extract-core-contexts.md`
## Agent Brief

**Category:** enhancement
**Summary:** Relocate the `lithos-cli` crate into the new workspace and point it to the extracted `trace-app` facade.

**Current behavior:**
`lithos-cli` exists outside the core contexts and relies on the monolithic `lithos-core` for its business logic and orchestration.

**Desired behavior:**
The CLI lives at `crates/cli`, uses the package name `trace-cli`, and strictly consumes the `trace-app` orchestration crate instead of `lithos-core`. It successfully coordinates command-line input to application logic.

**Key interfaces:**
- `crates/cli/Cargo.toml` — package name update and dependency mappings.
- `trace-app` initialization facade — the CLI should only interact with the system via `trace-app`.

**Acceptance criteria:**
- [x] `lithos-cli` has been successfully moved to `crates/cli`.
- [x] `crates/cli/Cargo.toml` package name is `trace-cli`.
- [x] The CLI relies on `trace-app` for its orchestration and domain logic.
- [x] The CLI builds successfully (`cargo build --bin trace-cli` or similar).
- [x] Any CLI-specific tests pass.

**Out of scope:**
- Adding new CLI commands or changing the user experience.
