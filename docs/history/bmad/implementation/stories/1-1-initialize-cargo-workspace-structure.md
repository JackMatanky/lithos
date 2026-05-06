# Story 1.1: Initialize Cargo Workspace Structure

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer setting up the project foundation,
I want to create the Cargo workspace with 4 hexagonal architecture crates,
so that the project has clear separation between domain, application, infrastructure, and CLI layers.

## Acceptance Criteria

### 1. Cargo Workspace Initialization
- **Given** a new Rust project directory
- **When** I run the workspace initialization commands
- **Then** a Cargo workspace is created with the following structure:
  ```
  lithos/
  ├── Cargo.toml (workspace configuration)
  ├── crates/
  │   ├── domain/ (pure business logic, no I/O)
  │   ├── app/ (application services and orchestration)
  │   ├── adapters/ (infrastructure implementations)
  │   └── cli/ (binary entry point)
  └── Cargo.lock
  ```

### 2. Hexagonal Boundary Enforcement
- **Given** the Cargo workspace structure exists
- **When** I check the crate dependencies
- **Then** the dependencies follow hexagonal boundaries:
  - `domain` crate has no external dependencies
  - `app` crate depends only on `domain`
  - `adapters` crate depends on `domain` + external crates
  - `cli` crate depends on `app` + `adapters`

### 3. Compilation Check
- **Given** the workspace is initialized
- **When** I run `cargo check`
- **Then** all crates compile without errors

## Tasks / Subtasks

- [x] Initialize workspace Cargo.toml at root (AC: 1)
- [x] Create Tiered Configuration Files (Elite Best Practice)
  - [x] `rustfmt.toml` (Visual Identity & Import Sorting)
  - [x] `clippy.toml` (Technical Thresholds - Cognitive Complexity)
- [x] Create core crates (AC: 1)
  - [x] `crates/domain` (lib)
  - [x] `crates/app` (lib)
  - [x] `crates/adapters` (lib)
  - [x] `crates/cli` (bin)
- [x] Configure crate-level Cargo.toml dependencies (AC: 2)
  - [x] Set up hexagonal dependency graph
  - [x] Add workspace-level dependency inheritance
- [x] Implement initial lib.rs/main.rs boilerplate (AC: 3)
- [x] Verify setup with workspace-wide `cargo check` (AC: 3)

## Dev Notes

- **Architecture Compliance**: This story establishes the hexagonal foundation defined in `architecture.md`. Use Cargo workspaces to enforce these boundaries physically.
- **Dependency Strategy**: Use `[workspace.dependencies]` in the root `Cargo.toml` to centralize versions.
- **Strict Purity**: Ensure `crates/domain` is initialized with NO dependencies other than internal models.

### Specific Cargo.toml Requirements

**Root Workspace Cargo.toml (Policy & Dependencies):**
Must define the lint levels and centralize dependency versions for the entire workspace.

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Jack"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/jack/lithos-rust"
description = "A CLI-first templating and schema system for Obsidian vaults"

[workspace.dependencies]
# Core Runtime
tokio = { version = "1.49", features = ["full"] }
async-trait = "0.1"

# API & CLI
clap = { version = "4.5", features = ["derive", "env"] }
miette = { version = "7.6", features = ["fancy"] }

# Data & Persistence
serde = { version = "1.0", features = ["derive"] }
redb = "3.1"
rkyv = { version = "0.8", features = ["validation", "bytecheck_std"] }
uuid = { version = "1.19", features = ["v7", "serde"] }

# Application Logic
minijinja = "2.14"
pulldown-cmark = "0.13"
figment = { version = "0.10", features = ["toml", "env"] }
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
convert_case = "0.10"
slug = "0.1"
base64 = "0.22"
rand = "0.10"

# Internal Workspace Crates
lithos-domain = { path = "crates/domain" }
lithos-app = { path = "crates/app" }
lithos-adapters = { path = "crates/adapters" }

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
rust_2018_idioms = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
cognitive_complexity = "warn"
unwrap_used = "deny"
expect_used = "deny"
todo = "deny"
panic = "deny"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
```

**clippy.toml (Technical Guardrails):**
```toml
cognitive-complexity-threshold = 15
too-many-lines-threshold = 100
doc-valid-idents = ["Lithos", "Obsidian", "Redb", "rkyv"]
```

**rustfmt.toml (Visual Identity):**
```toml
edition = "2021"
newline_style = "Unix"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
max_width = 100
```

**Crate-Specific Dependencies:**
- **domain**: No external dependencies. Use `workspace = true` for `serde`, `thiserror`, and `uuid`.
- **app**: Depends on `lithos-domain = { workspace = true }`.
- **adapters**: Depends on `lithos-domain = { workspace = true }`, `redb`, `rkyv`, `minijinja`, etc. (all via `workspace = true`).
- **cli**: Depends on `lithos-app = { workspace = true }` and `lithos-adapters = { workspace = true }`.

### Project Structure Notes

- Alignment with the directory structure specified in `architecture.md`.
- No conflicts detected; this is a greenfield initialization.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.1]
- [Source: _bmad-output/project-context.md#Architectural Integrity]

## Change Log

- 2026-01-11: Initialized workspace structure, crates, and configuration files.

## Dev Agent Record

### Agent Model Used

Claude 3.5 Sonnet (via BMAD Dev Agent)

### Debug Log References

- Initial `cargo check` failed due to `rkyv 0.8` feature mismatch (`bytecheck_std`).
- Fixed by removing `bytecheck_std` and using `rkyv = "0.8"`.

### Completion Notes List

- Successfully initialized Cargo workspace with 4 hexagonal crates.
- Configured root `Cargo.toml` with centralized dependencies and strict lints.
- Created `clippy.toml` and `rustfmt.toml` for quality enforcement.
- Verified compilation with `cargo check`.

### File List

- `Cargo.toml`
- `clippy.toml`
- `rustfmt.toml`
- `crates/domain/Cargo.toml`
- `crates/domain/src/lib.rs`
- `crates/app/Cargo.toml`
- `crates/app/src/lib.rs`
- `crates/adapters/Cargo.toml`
- `crates/adapters/src/lib.rs`
- `crates/cli/Cargo.toml`
- `crates/cli/src/main.rs`
