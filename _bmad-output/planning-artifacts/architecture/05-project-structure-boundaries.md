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

> **Note**: This structure represents the **target state** after refactoring to the unified Storage pattern. Files marked with:
> - `✅ CREATED` - Already implemented with new pattern
> - `[TODO: create]` - Needs to be created during refactor
> - `[TODO: assess]` - Evaluate if needed (optional View pattern)
> - `[TODO: refactor]` - Exists but needs updating to new pattern
> - `[LEGACY - REMOVE]` - Old CQRS/port/event patterns to be deleted

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
│       ├── patterns.rs         # Shared domain patterns (legacy - migrate to context modules)
│       ├── bounds.rs           # Generic bounds types for range validation (cross-cutting utility)
│       ├── application/        # APPLICATION LAYER (Cross-context orchestration)
│       │   ├── mod.rs          # Application service exports
│       │   ├── vault.rs        # VaultService (file discovery + note ingestion)
│       │   └── services/       # Cross-context workflow services (future)
│       ├── db/                 # PERSISTENCE INFRASTRUCTURE (redb + rkyv)
│       │   ├── mod.rs          # Database module entry, core Database type
│       │   ├── error.rs        # Storage-specific error types (DbError)
│       │   ├── reader.rs       # Zero-copy read operations with closure-based API
│       │   ├── writer.rs       # Write/batch operations (BatchWriter)
│       │   └── retry.rs        # Retry logic for transient errors (RetryConfig)
│       ├── note/               # NOTE CONTEXT (Knowledge Graph) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Note aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawNote (serde only, pre-validation) [TODO: create]
│       │   ├── loader.rs       # Parse + validate + persist orchestration pipeline
│       │   ├── storage.rs      # note::Storage trait + Redb/InMemory/Fake implementations [TODO: create]
│       │   ├── view.rs         # Optional: NoteView/TaskView projections (only if needed) [TODO: assess]
│       │   ├── frontmatter.rs  # Metadata extraction and parsing
│       │   ├── link.rs         # Wiki-link and embed logic
│       │   ├── structure.rs    # Markdown structural analysis
│       │   ├── heading.rs      # Heading extraction and navigation
│       │   ├── tag.rs          # Tag indexing logic
│       │   ├── task.rs         # Task/Todo extraction
│       │   ├── identity.rs     # Note identity (UUID, paths, aliases)
│       │   ├── paths.rs        # Path resolution and canonicalization
│       │   ├── position.rs     # Position/offset tracking in markdown
│       │   ├── value.rs        # Value types for note metadata
│       │   ├── error.rs        # Context-specific errors
│       │   ├── reader/         # Note parsing and frontmatter extraction utilities
│       │   ├── db_command.rs   # [LEGACY - REMOVE] Old CQRS command pattern
│       │   ├── db_query.rs     # [LEGACY - REMOVE] Old CQRS query pattern
│       │   ├── ports.rs        # [LEGACY - REMOVE] Old port-based pattern
│       │   ├── stored.rs       # [LEGACY - REMOVE] Old DTO pattern
│       │   └── events.rs       # [LEGACY - REMOVE] Old event pattern
│       ├── schema/             # SCHEMA CONTEXT (Validation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Schema aggregate root (domain, has rkyv derives) [TODO: create]
│       │   ├── raw.rs          # RawSchema (serde only, pre-validation)
│       │   ├── loader.rs       # 8-phase pipeline: Discover→Parse→Validate→Dereference→Graph→Sort→Resolve→Project
│       │   ├── storage.rs      # schema::Storage trait + Redb/InMemory/Fake implementations ✅ CREATED
│       │   ├── ingestor.rs     # File parsing logic (File → RawSchema)
│       │   ├── view.rs         # Optional: SchemaView projection (only if needed) [TODO: assess]
│       │   ├── id.rs           # Schema identity types
│       │   ├── property.rs     # Individual property domain logic
│       │   ├── property_spec.rs # Property specification types
│       │   ├── property_spec/  # Property spec submodules
│       │   ├── bank.rs         # PropertyBank for $ref expansion
│       │   ├── dereferencer.rs # $ref pointer expansion (Phase 3)
│       │   ├── extender.rs     # Inheritance merging logic
│       │   ├── resolver.rs     # Reference resolution and graph operations
│       │   ├── error.rs        # Context-specific errors
│       │   ├── db_command.rs   # [LEGACY - REMOVE] Old CQRS command pattern
│       │   ├── db_query.rs     # [LEGACY - REMOVE] Old CQRS query pattern
│       │   ├── ports.rs        # [LEGACY - REMOVE] Old port-based pattern
│       │   └── events.rs       # [LEGACY - REMOVE] Old event pattern
│       ├── template/           # TEMPLATE CONTEXT (Generation) - BUSINESS
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Template aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawTemplate (serde only, pre-validation, optional)
│       │   ├── loader.rs       # Parse + validate + persist orchestration pipeline [TODO: create]
│       │   ├── storage.rs      # template::Storage trait + Redb/InMemory/Fake implementations [TODO: create]
│       │   ├── view.rs         # Optional: TemplateView projection (only if needed) [TODO: assess]
│       │   ├── block.rs        # Template block structures
│       │   ├── catalog.rs      # Template catalog and discovery
│       │   ├── value.rs        # Value types for template variables
│       │   ├── adapter/        # Template rendering adapters
│       │   ├── error.rs        # Context-specific errors
│       │   ├── command.rs      # [LEGACY - REMOVE] Old CQRS command pattern
│       │   ├── query.rs        # [LEGACY - REMOVE] Old CQRS query pattern
│       │   ├── ports.rs        # [LEGACY - REMOVE] Old port-based pattern
│       │   └── events.rs       # [LEGACY - REMOVE] Old event pattern
│       ├── config/             # CONFIG CONTEXT (System Settings) - CROSS-CUTTING
│       │   ├── mod.rs          # Public API, re-exports
│       │   ├── aggregate.rs    # Config aggregate root (domain, has rkyv derives)
│       │   ├── raw.rs          # RawConfig (serde only, pre-validation)
│       │   ├── loader.rs       # Config resolution pipeline with hybrid caching [TODO: refactor]
│       │   ├── storage.rs      # config::Storage trait + adapter implementation [TODO: create]
│       │   ├── view.rs         # Optional: ConfigView projection (only if needed) [TODO: assess]
│       │   ├── frontmatter.rs  # Frontmatter-specific config
│       │   ├── global.rs       # System-wide settings
│       │   ├── vault.rs        # Vault-specific settings
│       │   ├── paths.rs        # Path configuration
│       │   ├── logging.rs      # Logging configuration
│       │   ├── task.rs         # Task-specific configuration
│       │   ├── value.rs        # Value types for config
│       │   ├── adapter/        # Config source adapters (Figment integration)
│       │   ├── error.rs        # Context-specific errors
│       │   ├── command.rs      # [LEGACY - REMOVE] Old CQRS command pattern
│       │   ├── query.rs        # [LEGACY - REMOVE] Old CQRS query pattern
│       │   ├── ports.rs        # [LEGACY - REMOVE] Old port-based pattern
│       │   └── events.rs       # [LEGACY - REMOVE] Old event pattern
│       └── fs/                 # FILESYSTEM (OS Integration)
│           ├── mod.rs          # FS module entry and public type aliases
│           ├── error.rs        # I/O specific errors (ParseError, PathValidationError)
│           ├── reader.rs       # Root-scoped file reader (FsReader) with validation
│           ├── writer.rs       # Root-scoped file writer (FsWriter) with atomic-replace
│           ├── validator.rs    # Security-critical path validation (PathValidator)
│           └── types.rs        # File type markers and parsing helpers (module-internal)
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
