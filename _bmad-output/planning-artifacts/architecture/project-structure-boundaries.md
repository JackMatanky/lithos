---
title: "Project Structure & Boundaries"
description: "Complete project directory structure and architectural boundaries for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Project Structure"
---

# Project Structure & Boundaries

## Complete Project Directory Structure

```text
lithos/
├── .gitattributes                # LF enforcement
├── .gitignore                    # standard Rust ignores
├── .mise/                        # TASK ORCHESTRATION (mise-first)
│   └── tasks/
│       ├── dev-setup.sh          # Env bootstrap (mise run setup)
│       ├── run-benchmarks.sh     # Performance validation (mise run bench)
│       └── install-hooks.sh      # Git hook setup
├── .pre-commit-config.yaml       # QUALITY GATE (miette, clippy, rustfmt)
├── Cargo.toml                    # Workspace configuration (Rust 1.92+)
├── Cargo.lock                    # Dependency lock file
├── mise.toml                     # Task definitions & tool versions
├── deny.toml                     # Dependency license & security policy
├── rustfmt.toml                  # Formatting (import sorting)
├── clippy.toml                   # Complexity limits (cognitive < 15)
├── README.md                     # Project overview
├── _bmad-output/                 # AI Agent Context
├── docs/                         # Documentation
│   ├── adr/                      # Architectural Decision Records
│   └── guides/                   # Guides and references
├── crates/
│   ├── lithos-core/              # SINGLE CORE CRATE (Logic + Infra)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            # Prelude & Public API
│   │   │   ├── db.rs             # Zero-copy Redb Database Layer
│   │   │   ├── config.rs         # Config aggregate entry point
│   │   │   ├── config/           # Config implementation details
│   │   │   │   ├── global.rs
│   │   │   │   ├── vault.rs
│   │   │   │   ├── types.rs
│   │   │   │   ├── events.rs     # Co-located events
│   │   │   │   └── error.rs      # Co-located errors
│   │   │   ├── note.rs           # Note aggregate entry point
│   │   │   ├── note/             # Note implementation details
│   │   │   │   ├── frontmatter.rs
│   │   │   │   ├── link.rs
│   │   │   │   ├── structure.rs
│   │   │   │   ├── tag.rs
│   │   │   │   ├── task.rs
│   │   │   │   ├── events.rs
│   │   │   │   └── error.rs
│   │   │   ├── schema.rs         # Schema aggregate entry point
│   │   │   ├── schema/           # Schema implementation details
│   │   │   │   ├── property.rs
│   │   │   │   ├── resolver.rs
│   │   │   │   ├── events.rs
│   │   │   │   └── error.rs
│   │   │   ├── template.rs       # Template aggregate entry point
│   │   │   ├── template/         # Template implementation details
│   │   │   │   ├── variable.rs
│   │   │   │   ├── composition.rs
│   │   │   │   ├── events.rs
│   │   │   │   └── error.rs
│   │   │   └── fs/               # Generic filesystem utilities
│   ├── lithos-cli/               # BINARY CRATE (CLI Driver)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Entry point
│   │       ├── commands/         # CLI Command definitions
│   │       └── terminal.rs       # Miette/Clap integration
├── tests/                        # Automated Tests
│   ├── integration/              # Cross-module flows
│   ├── e2e/                      # CLI-driven workflow tests
│   └── arch/                     # Dependency & Boundary enforcement
└── benches/                      # Performance Benchmarks (Criterion)
```

## Architectural Boundaries

**API Boundaries:**

- **CLI (`crates/lithos-cli`):** The primary driver. Orchestrates `lithos-core` logic and owns terminal rendering via `miette`.
- **Core (`crates/lithos-core`):** Contains all business logic, storage implementation, and file processing.

**Logical Boundaries (Module Visibility):**

- **Public API:** Only types reachable from `lithos-core/lib.rs` are public.
- **Context Isolation:** Modules (`note/`, `schema/`) rely on `pub(crate)` to enforce internal isolation. They depend on `db.rs` but not on each other (unless via public API).
- **Dependency Flow:** Infrastructure (`db.rs`, `fs/`) -> Domain (`note/`, `schema/`) -> CLI.

**Component Boundaries:**

- **Indexer:** Not a separate actor anymore, but a logical phase in `lithos-cli`. Writes are atomic and coordinated via `db.batch_write()` for bulk operations.
- **Compliance Engine:** Located in `lithos-core/schema/compliance.rs` (or similar). Checks if Note metadata satisfies Schema rules.

**Service Boundaries:**

- **Template Designer (FR9):** Located in `lithos-core/template/designer.rs` (if interactive logic moves there) or `lithos-cli/src/commands/template.rs` for interaction. The core logic remains in `lithos-core/template/`.
- **Metrics Calculator:** Aggregates vault-wide data. Likely a submodule `lithos-core/metrics/` or helper in `lithos-core/note/stats.rs`.

**Data Boundaries:**

- **Identity (UUID v7):** We use UUID v7 (Time-ordered) instead of paths or numeric IDs.
  - **Performance:** Ensures new notes are appended to Redb B-Tree leaves sequentially, achieving O(1) insertion.
  - **Persistence:** Allows notes to be moved or renamed while preserving their logical relationships in the Knowledge Graph.
- **Zero-Copy Serialization:** **rkyv** buffers are generated in `db.rs` and returned as `ArchivedGuard`, allowing the CLI to read data without memory duplication.

**File Organization Patterns:**

- **File First, Folder Second:** Use `<module>.rs` as the entry point, with `<module>/` folder for implementation details. NO `mod.rs`.
- **Co-location:** Errors (`error.rs`), Events (`events.rs`), and Ports (`ports.rs` if needed) are co-located within the module folder.

## Requirements to Structure Mapping

**Feature/Epic Mapping:**

- **Knowledge Graph (FR20-FR25):** `lithos-core/note/` + `lithos-core/db.rs` (Links/Embeds/Tags).
- **Schema & Compliance (FR8-FR14):** `lithos-core/schema/`.
- **Template Design (FR1-FR7, FR9):** `lithos-core/template/`.
- **Interactive CLI (FR41-FR47):** `lithos-cli/src/commands/` + `lithos-core/api/ui.rs` (if abstracting UI).

**Cross-Cutting Concerns:**

- **Metadata Extraction:** Handled in `lithos-core/fs/parsers/markdown.rs`.
- **Validation Hierarchy:**
  - **Syntactic:** Structural validity of YAML/TOML/Schema JSON (in `lithos-core/fs/validator.rs`).
  - **Semantic:** Contract check between a Note and its Schema (in `lithos-core/schema/`).
- **Performance:** Monitored via `benches/`, optimized via `rkyv` byte-layouts.
- **Task Management:** Centralized in `.mise/tasks/` and orchestrated via `mise.toml`.

## Integration Points

**Internal Communication:**

- **Hybrid Bus (ADR 0007):** Minimized for Phase 1. `db.rs` handles data persistence. Events are emitted via simple callbacks or staged in `UnitOfWork` for later dispatch if needed.
- **Database:** `lithos-core/db.rs` exposes a concrete `Database` struct with zero-copy methods (`get_archived`, `put_reserve`).

**External Integrations:**

- **Obsidian Vault:** Interfaced via `lithos-core/fs` and extracted via `lithos-core/fs/parsers`.
- **Hierarchical Config:** Managed by `figment` in `lithos-core/config` (Global -> User -> Project -> Vault -> Env -> Flag).

**Data Flow:**

- **Write Path:** CLI -> Note::save(&db) -> Database::put_reserve -> Redb.
- **Read Path:** CLI -> Note::find_by_id(&db) -> Database::get_archived -> Zero-copy view.

## Development Workflow Integration

**Development Server Structure:**

- Managed via `mise run dev` which wraps `cargo-watch` for automatic rebuilding.

**Build Process Structure:**

- **Mise-first:** `mise run build` handles static linking and binary stripping to produce a zero-dependency artifact.
- **Tasks:**
  - `mise run verify`: Full quality gate (fmt, lint, test, adr).
  - `mise run test:arch`: Enforce boundary rules.

## Test Organization

- **Unit:** Inline `#[cfg(test)]` within `lithos-core` modules.
- **Integration:** `tests/integration/` for full flows (CLI -> Core -> DB).
- **Architecture:** `tests/arch/` for boundary enforcement.
- **Benchmarks:** `benches/` for zero-copy validation.
