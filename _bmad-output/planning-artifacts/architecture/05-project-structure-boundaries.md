---
title: "Project Structure & Boundaries"
description: "Complete project directory structure and architectural boundaries for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-03-10"
section: "Project Structure"
---

# Project Structure & Boundaries

## Complete Project Directory Structure

```text
lithos/
├── .mise/tasks/                # TASK ORCHESTRATION (mise-first)
│   ├── adr/                    # ADR management tasks (validate, metrics)
│   ├── test/                   # Specialized test suite execution tasks
│   ├── build                   # Binary build and optimization task
│   ├── clean                   # Artifact cleanup task
│   ├── dev-setup               # Environment bootstrap task
│   ├── doc                     # Documentation generation task
│   ├── fmt                     # Code formatting task
│   └── lint                    # Static analysis (clippy) task
├── _bmad-output/               # AI AGENT CONTEXT & ARTIFACTS
│   ├── planning-artifacts/     # Architecture, PRD, and Epics
│   ├── implementation-artifacts/ # Story tracking, retros, and reports
│   └── test-artifacts/         # Quality assurance design and review logs
├── docs/                       # PROJECT DOCUMENTATION
│   ├── adr/                    # Architectural Decision Records (0001-0012)
│   └── design/                 # Technical design specifications
├── lithos-core/                # CORE LIBRARY (Logic + Infrastructure)
│   ├── benches/                # Performance benchmarks (Criterion)
│   │   └── redb_rkyv.rs        # Persistence layer performance validation
│   ├── tests/                  # Integration tests (Cross-context flows)
│   └── src/
│       ├── lib.rs              # Crate root and public prelude
│       ├── patterns.rs         # Shared domain patterns
│       ├── application/        # APPLICATION LAYER (Cross-context orchestration)
│       │   ├── mod.rs          # Application service exports
│       │   ├── vault.rs        # VaultService (file discovery + note ingestion)
│       │   └── services/       # Cross-context workflow services (future)
│       ├── db/                 # PERSISTENCE INFRASTRUCTURE (redb + rkyv)
│       │   ├── mod.rs          # Database module entry, core Database type
│       │   ├── batch.rs        # Atomic write batch implementation
│       │   ├── error.rs        # Storage-specific error types
│       │   ├── reader.rs       # Zero-copy read helpers
│       │   └── writer.rs       # Batch write helpers
│       ├── note/               # NOTE CONTEXT (Knowledge Graph) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Note aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawNote (serde only, pre-validation)
│       │   ├── loader.rs       # Parse + validate + persist orchestration pipeline
│       │   ├── storage.rs      # NoteStorage trait + Redb adapter implementation
│       │   ├── view.rs         # Optional: NoteView/TaskView projections
│       │   ├── frontmatter.rs  # Metadata extraction and parsing
│       │   ├── link.rs         # Wiki-link and embed logic
│       │   ├── structure.rs    # Markdown structural analysis
│       │   ├── tag.rs          # Tag indexing logic
│       │   ├── task.rs         # Task/Todo extraction
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── schema/             # SCHEMA CONTEXT (Validation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Schema aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawSchema (serde only, pre-validation)
│       │   ├── loader.rs       # Parse + validate + persist orchestration pipeline
│       │   ├── storage.rs      # SchemaStorage trait + Redb adapter implementation
│       │   ├── view.rs         # Optional: SchemaView projection
│       │   ├── property.rs     # Individual property logic
│       │   ├── property_spec.rs # Property specification types
│       │   ├── resolver.rs     # Reference resolution
│       │   ├── graph.rs        # Schema inheritance graph
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── template/           # TEMPLATE CONTEXT (Generation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Template aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawTemplate (serde only, pre-validation, optional)
│       │   ├── loader.rs       # Parse + validate + persist orchestration pipeline
│       │   ├── storage.rs      # TemplateStorage trait + Redb adapter implementation
│       │   ├── view.rs         # Optional: TemplateView projection
│       │   ├── variable.rs     # Variable injection logic
│       │   ├── composition.rs  # Component/Partial logic
│       │   ├── syntax.rs       # Syntax highlighting/parsing
│       │   ├── validation.rs   # Template safety checks
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── config/             # CONFIG CONTEXT (System Settings) - CROSS-CUTTING
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Config aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawConfig (serde only, pre-validation)
│       │   ├── loader.rs       # Config resolution pipeline
│       │   ├── storage.rs      # ConfigStorage trait + adapter implementation
│       │   ├── view.rs         # Optional: ConfigView projection
│       │   ├── types.rs        # Shared config models
│       │   ├── global.rs       # System-wide settings
│       │   ├── vault.rs        # Vault-specific settings
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       └── fs/                 # FILESYSTEM (OS Integration)
│           ├── mod.rs          # FS module entry
│           ├── source.rs       # File discovery and reading abstractions (FsReader)
│           ├── parsers.rs      # Markdown/YAML/TOML parsers
│           ├── validator.rs    # Path and Syntax validation
│           └── error.rs        # I/O specific errors
├── lithos-cli/                 # BINARY CRATE (CLI Driver)
│   └── src/main.rs             # CLI entry point (Clap + Miette)
├── .gitattributes              # LF enforcement
├── .gitignore                  # standard Rust ignores
├── .pre-commit-config.yaml     # QUALITY GATE (miette, clippy, rustfmt)
├── mise.toml                   # Task definitions & tool versions
├── Cargo.toml                  # Workspace configuration (Rust 1.92+)
├── Cargo.lock                  # Dependency lock file
├── clippy.toml                 # Complexity limits (cognitive < 15)
├── deny.toml                   # Dependency license & security policy
├── rustfmt.toml                # Formatting (import sorting)
├── nextest.toml                # Test runner configuration
├── rust-toolchain.toml         # Toolchain version lock
└── README.md                   # Project overview
```

## Architectural Boundaries

### API Boundaries

- **CLI (`lithos-cli`):** The primary driver. Orchestrates `lithos-core` logic and owns terminal rendering via `miette`.
- **Core (`lithos-core`):** Contains all business logic, storage implementation, and file processing.

### Logical Boundaries (Module Visibility)

- **Public API:** Only types reachable from `lithos-core/src/lib.rs` are public.
- **Context Isolation:**
  - **Business Contexts** (`note`, `schema`, `template`): Isolated from each other via Rust module boundaries.
  - **Cross-Cutting Context** (`config`): Shared business rules/settings accessible to all business contexts.
  - **Pure Infrastructure** (`db`, `fs`, ...): Generic utilities with no business rules.
  - Business contexts depend on config context and infrastructure, but NOT on each other.
- **Dependency Flow:**
  - Technical Infrastructure (db/, fs/, patterns/) → Context Storage Implementations.
  - Domain Contexts (note/, schema/, template/, config/) → Storage Traits.
  - Application Layer (application/) → Domain + Infrastructure (via dependency injection).
  - Drivers (CLI, LSP) → Application Layer.
- **Unified Storage Traits:**
  - Each context defines a single, simple `Storage` trait for all persistence operations (e.g., `SchemaStorage`, `NoteStorage`).
  - Storage adapters are implemented directly in `<context>/storage.rs`, utilizing the generic `db/` infrastructure.
  - Avoids interface bloat and complex trait bounds by utilizing single traits per context instead of split CQRS ports.

### Component Boundaries

- **Indexer/Loader:** A functional pipeline orchestrated by `<context>/loader.rs` during system bootstrap or file changes. Writes are atomic and coordinated via `db.batch_write()` for bulk operations.
- **Compliance Engine:** Located in `lithos-core/src/schema/`. Checks if Note metadata satisfies Schema rules.

### Service Boundaries

- **Template System:** The core logic remains in `lithos-core/src/template/`. Interaction is handled via `lithos-cli`.
- **Metrics & Stats:** Aggregates vault-wide data. Likely implemented as projection queries in `<context>/storage.rs` or a specialized metrics module.

### Data Boundaries

- **Identity (UUID v7):** We use UUID v7 (Time-ordered) instead of paths or numeric IDs.
  - **Performance:** Ensures new notes are appended to Redb B-Tree leaves sequentially, achieving O(1) insertion.
  - **Persistence:** Allows notes to be moved or renamed while preserving their logical relationships in the Knowledge Graph.
- **Files as Source of Truth:** User-editable vault files on the filesystem are authoritative. The database is a rebuildable, query-optimized projection cache.
- **Serialization Shapes Model:**
  - **`Raw*` (serde):** Unvalidated input from filesystem, nullable fields for better errors (e.g., `RawSchema`). Contains no behavior.
  - **Domain (rkyv + serde feature-gated):** Validated entities with **rkyv derives** for zero-copy DB operations. Validation happens in `TryFrom<Raw*>`.
  - **`Archived*` (rkyv generated):** Implicit zero-copy representation mapped directly from the DB, offering free query optimization for Domain types.
  - **`*View` (rkyv, optional):** Read-optimized database projections (e.g., `SchemaView`, `NoteView`). Only created when domain shape is inefficient for storage or when queries need a flattened format.
- **Zero-Copy Access:** **rkyv** buffers are managed in `src/db/` and returned via closure-based APIs, allowing the CLI to read data without memory duplication.

## File Organization Patterns

- **Module Folder with mod.rs:** Use `<context>/mod.rs` as the entry point for all contexts.
- **Functional Pipeline Structure:** Split logic into:
  - `aggregate.rs` (invariants, domain entities with rkyv derives).
  - `raw.rs` (unvalidated input with serde derives, dumb data DTOs).
  - `loader.rs` (Iterator-based pipelines: parse -> validate -> project).
  - `storage.rs` (Unified Storage trait and its Redb adapter implementation).
  - `view.rs` (Optional read-optimized projections for the database).
- **Co-location:** Errors (`error.rs`), Events (`events.rs`), and Storage components are co-located within the context folder.
- **Flat Context Directory:** Avoid premature nesting. Adapters and traits live together in `storage.rs`.

## Requirements to Structure Mapping

### Feature/Epic Mapping

- **Knowledge Graph (FR20-FR25):** `lithos-core/src/note/` + `lithos-core/src/db/` (Links/Embeds/Tags).
- **Schema & Compliance (FR8-FR14):** `lithos-core/src/schema/`.
- **Template Design (FR1-FR7, FR9):** `lithos-core/src/template/`.
- **Interactive CLI (FR41-FR47):** `lithos-cli/src/main.rs` (Driver) delegating to `lithos-core/src/` contexts.

### Cross-Cutting Concerns

- **Metadata Extraction:** Handled in `lithos-core/src/note/frontmatter.rs` and `lithos-core/src/fs/parsers.rs`.
- **Validation Hierarchy:**
  - **Syntactic:** Structural validity of YAML/TOML/Schema JSON (in `lithos-core/src/fs/validator.rs`).
  - **Semantic:** Contract check between a Note and its Schema (in `lithos-core/src/schema/`).
- **Performance:** Monitored via `lithos-core/benches/`, optimized via `rkyv` byte-layouts.
- **Task Management:** Centralized in `.mise/tasks/` and orchestrated via `mise.toml`.

## Integration Points

### Internal Communication

- **Functional Composition:** System state mutations and data flow occur via direct functional calls and pipeline iterators, using `Result<T, E>` for error propagation. No event bus or event sourcing is utilized.
- **Database:** `lithos-core/src/db/mod.rs` exposes a concrete `Database` struct with zero-copy methods (`get`, `put`). Context storage implementations utilize this shared infrastructure.

### External Integrations

- **Obsidian Vault:** Interfaced via `lithos-core/src/fs/` (e.g., `FsReader`) and parsed via `lithos-core/src/fs/parsers.rs`.
- **Hierarchical Config:** Managed by `figment` in `lithos-core/src/config/` (Global -> User -> Project -> Vault -> Env -> Flag).

### Data Flow

- **Ingestion Pipeline (Write Path):** File Change -> `loader` discovers via `FsReader` -> parses to `Raw*` -> validates via `TryFrom` to Domain -> projects to Redb via `storage`.
- **Read Path:** CLI -> Application Service/Loader -> Domain Storage implementation (`get`, `list`) -> Database -> Zero-copy Domain or `*View` type.

## Development Workflow Integration

### Development Server Structure

- Managed via `mise run dev` which wraps `cargo-watch` for automatic rebuilding.

### Build Process Structure

- **Mise-first:** `mise run build` handles static linking and binary stripping to produce a zero-dependency artifact.
- **Tasks:**
  - `mise run verify`: Full quality gate (fmt, lint, test, adr).

## Test Organization

- **Unit:** Inline `#[cfg(test)]` within `lithos-core` modules.
- **Integration:** `lithos-core/tests/` for full flows (CLI -> Core -> DB) when added.
- **Architecture:** Enforced via module visibility and dependency flow rules.
- **Benchmarks:** `lithos-core/benches/` for zero-copy validation.
