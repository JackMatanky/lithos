---
title: "Project Structure & Boundaries"
description: "Complete project directory structure and architectural boundaries for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-05"
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
│       ├── patterns.rs         # Shared domain patterns (Aggregate, Command)
│       ├── application/        # APPLICATION LAYER (Cross-context orchestration)
│       │   ├── mod.rs          # Application service exports
│       │   ├── vault.rs        # Vault facade (high-level API)
│       │   └── services/       # Cross-context workflow services
│       │       ├── note_creation.rs     # "Create note from template" workflow
│       │       ├── vault_init.rs        # "Initialize vault" workflow
│       │       └── batch_indexing.rs    # "Batch index notes" workflow
│       ├── db/                 # PERSISTENCE INFRASTRUCTURE (redb + rkyv)
│       │   ├── mod.rs          # Database module entry, core Database type
│       │   ├── batch.rs        # Atomic write batch implementation
│       │   ├── error.rs        # Storage-specific error types
│       │   ├── reader.rs       # Zero-copy read helpers
│       │   └── writer.rs       # Batch write helpers
│       ├── note/               # NOTE CONTEXT (Knowledge Graph) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports, type aliases (NoteQuery, NoteCommand)
│       │   ├── aggregate.rs    # Note aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawNote (serde only, pre-validation, optional)
│       │   ├── command.rs      # Command<C: NoteCommandPort> write operations
│       │   ├── query.rs        # Query<Q: NoteQueryPort> read operations
│       │   ├── ports.rs        # NoteQueryPort + NoteCommandPort traits with GATs
│       │   ├── adapters/       # Storage adapters (context-scoped)
│       │   │   ├── mod.rs
│       │   │   ├── query.rs      # impl note::ports::Query
│       │   │   ├── command.rs    # impl note::ports::Command
│       │   │   └── stored.rs     # Optional: StoredNote (only if domain shape inefficient)
│       │   ├── frontmatter.rs  # Metadata extraction and parsing
│       │   ├── link.rs         # Wiki-link and embed logic
│       │   ├── structure.rs    # Markdown structural analysis
│       │   ├── tag.rs          # Tag indexing logic
│       │   ├── task.rs         # Task/Todo extraction
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── schema/             # SCHEMA CONTEXT (Validation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports, type aliases (SchemaQuery, SchemaCommand)
│       │   ├── aggregate.rs    # Schema aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawSchema (serde only, pre-validation)
│       │   ├── command.rs      # Command<C: SchemaCommandPort> lifecycle management
│       │   ├── query.rs        # Query<Q: SchemaQueryPort> lookup and resolution
│       │   ├── ports.rs        # SchemaQueryPort + SchemaCommandPort traits with GATs
│       │   ├── adapters/       # Storage adapters (context-scoped)
│       │   │   ├── mod.rs
│       │   │   ├── query.rs      # impl schema::ports::Query
│       │   │   ├── command.rs    # impl schema::ports::Command
│       │   │   └── stored.rs     # Optional: StoredSchema (only if domain shape inefficient)
│       │   ├── property.rs     # Individual property logic
│       │   ├── property_spec.rs # Property specification types
│       │   ├── resolver.rs     # Reference resolution
│       │   ├── graph.rs        # Schema inheritance graph
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── template/           # TEMPLATE CONTEXT (Generation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports, type aliases
│       │   ├── aggregate.rs    # Template aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawTemplate (serde only, pre-validation, optional)
│       │   ├── command.rs      # Command<C: TemplateCommandPort> rendering operations
│       │   ├── query.rs        # Query<Q: TemplateQueryPort> lookup and resolution
│       │   ├── ports.rs        # TemplateQueryPort + TemplateCommandPort traits with GATs
│       │   ├── adapters/       # Storage adapters (context-scoped)
│       │   │   ├── mod.rs
│       │   │   ├── query.rs      # impl template::ports::Query
│       │   │   ├── command.rs    # impl template::ports::Command
│       │   │   └── stored.rs     # Optional: StoredTemplate (only if domain shape inefficient)
│       │   ├── variable.rs     # Variable injection logic
│       │   ├── composition.rs  # Component/Partial logic
│       │   ├── syntax.rs       # Syntax highlighting/parsing
│       │   ├── validation.rs   # Template safety checks
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       ├── config/             # CONFIG CONTEXT (System Settings) - CROSS-CUTTING
│       │   ├── mod.rs          # Public API, re-exports, type aliases
│       │   ├── aggregate.rs    # Config aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawConfig (serde only, pre-validation, optional)
│       │   ├── command.rs      # Command<C: ConfigCommandPort> settings updates
│       │   ├── query.rs        # Query<Q: ConfigQueryPort> settings retrieval
│       │   ├── ports.rs        # ConfigQueryPort + ConfigCommandPort traits with GATs
│       │   ├── adapters/       # Storage adapters (context-scoped)
│       │   │   ├── mod.rs
│       │   │   ├── query.rs      # impl config::ports::Query
│       │   │   ├── command.rs    # impl config::ports::Command
│       │   │   └── stored.rs     # Optional: StoredConfig (only if domain shape inefficient)
│       │   ├── types.rs        # Shared config models
│       │   ├── global.rs       # System-wide settings
│       │   ├── vault.rs        # Vault-specific settings
│       │   ├── error.rs        # Context-specific errors
│       │   └── events.rs       # Domain events
│       └── fs/                 # FILESYSTEM (OS Integration)
│           ├── mod.rs          # FS module entry
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
  - **Business Contexts** (`note`, `schema`, `template`): Isolated from each other
  - **Cross-Cutting Context** (`config`): Shared business rules/settings accessible to all business contexts
  - **Pure Infrastructure** (`db`, `fs`, ...): Generic utilities with no business rules
  - Business contexts depend on config context and infrastructure, but NOT on each other
- **Dependency Flow:**
  - Technical Infrastructure (db/, fs/, patterns/) → Adapters
  - Adapters (<context>/adapters/) → Domain Ports + Technical Infrastructure
  - Domain Contexts (note/, schema/, template/, config/) → Ports only (not adapters)
  - Application Layer (application/) → Domain + Adapters (via dependency injection)
  - Drivers (CLI, LSP) → Application Layer
- **Port-Based CQRS:**
  - Each context defines **split storage ports** with GATs (e.g., `schema::ports::Query`, `schema::ports::Command`)
  - CQRS types generic over respective ports: `Query<Q: SchemaQueryPort>`, `Command<C: SchemaCommandPort>`
  - Context-scoped adapters: `schema::adapters::QueryAdapter<'db>` and `schema::adapters::CommandAdapter<'db>` implement ports
  - Type aliases hide complexity: `SchemaQuery<'db> = Query<QueryAdapter<'db>>` (storage-agnostic names)
  - Port split prevents interface bloat and enables read-only test fakes
  - Adapters scoped to context (not in generic `db/`) for cohesion and independence

### Component Boundaries

- **Indexer:** Not a separate actor anymore, but a logical phase in `lithos-cli`. Writes are atomic and coordinated via `db.batch_write()` for bulk operations.
- **Compliance Engine:** Located in `lithos-core/src/schema/`. Checks if Note metadata satisfies Schema rules.

### Service Boundaries

- **Template System:** The core logic remains in `lithos-core/src/template/`. Interaction is handled via `lithos-cli`.
- **Metrics & Stats:** Aggregates vault-wide data. Likely implemented as queries in `lithos-core/src/note/query.rs` or a specialized metrics module.

### Data Boundaries

- **Identity (UUID v7):** We use UUID v7 (Time-ordered) instead of paths or numeric IDs.
  - **Performance:** Ensures new notes are appended to Redb B-Tree leaves sequentially, achieving O(1) insertion.
  - **Persistence:** Allows notes to be moved or renamed while preserving their logical relationships in the Knowledge Graph.
- **Three-Shape Serialization Model:** **rkyv** buffers are managed in `src/db/` and returned via closure-based APIs, allowing the CLI to read data without memory duplication.
  - **Raw\* (serde):** Unvalidated input from filesystem, nullable fields for better errors (e.g., `RawSchema`)
  - **Domain (rkyv + serde feature-gated):** Validated entities with **rkyv derives** for zero-copy DB operations, used throughout application
  - **Stored\* (rkyv, optional):** Storage-optimized representation, only created when domain shape inefficient for storage
  - Port traits with GATs enable closure-based archived reads
  - **Default:** Store domain types directly (they have rkyv derives); only use `Stored*` for optimization
- **Storage DTOs (Optional):**
  - Create `Stored*` only when: wrapper newtypes complicate indexing, deep nesting causes performance issues, Arc sharing doesn't serialize well
  - Mechanical conversions at storage boundary when `Stored*` exists
  - Treat `Stored*` changes as migration decisions (persisted format contract)

## File Organization Patterns

- **Module Folder with mod.rs:** Use `<context>/mod.rs` as the entry point for all contexts.
- **CQRS Structure:** Split logic into:
  - `aggregate.rs` (invariants, domain entities with rkyv derives)
  - `raw.rs` (unvalidated input with serde derives, optional per context)
  - `command.rs` (Command<C: CommandPort> generic over command port)
  - `query.rs` (Query<Q: QueryPort> generic over query port)
  - `ports.rs` (QueryPort + CommandPort traits with GATs, single file per context)
- **Co-location:** Errors (`error.rs`), Events (`events.rs`), Ports (`ports.rs`), and Raw types (`raw.rs`) are co-located within the context folder.
- **Storage Adapters:** Adapters and optional `Stored*` types are context-scoped:
  - `<context>/adapters/query.rs` - QueryPort implementation
  - `<context>/adapters/command.rs` - CommandPort implementation
  - `<context>/adapters/stored.rs` - Optional StoredType (only if profiling shows need)
  - Adapters scoped to context, import `db/` utilities and implement context ports
  - No premature nesting (flat `adapters/` until multiple backends exist)

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

- **Hybrid Bus (ADR 004):** Minimized for Phase 1. `src/db/` handles data persistence. Events are emitted via simple callbacks or staged in `UnitOfWork` for later dispatch if needed.
- **Database:** `lithos-core/src/db/mod.rs` exposes a concrete `Database` struct with zero-copy methods (`get`, `put`).

### External Integrations

- **Obsidian Vault:** Interfaced via `lithos-core/src/fs/` and extracted via `lithos-core/src/fs/parsers.rs`.
- **Hierarchical Config:** Managed by `figment` in `lithos-core/src/config/` (Global -> User -> Project -> Vault -> Env -> Flag).

### Data Flow

- **Write Path:** CLI -> Note::command::save(&db) -> Database::put -> Redb.
- **Read Path:** CLI -> Note::query::find_by_id(&db) -> Database::get -> Zero-copy view.

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
