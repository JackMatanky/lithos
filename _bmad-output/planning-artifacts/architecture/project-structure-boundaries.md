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
├── _bmad-output/
│   ├── planning-artifacts/
│   │   ├── discovery/            # Project brief and corresponding elicitation summary
│   │   ├── architecture.md       # This document
│   │   ├── prd.md                # Product requirements (PRD)
│   │   └── ux-design-specification.md  # UX Design Specification
│   └── implementation-artifacts/
│       ├── course_corrections/   # Course corrections and implementation notes
│       ├── retros/               # Retrospectives and lessons learned
│       ├── stories/              # User stories and acceptance criteria
│       └── sprint-status.yaml    # Sprint status report
├── docs/                         # Documentation
│   ├── adr/                      # Architectural Decision Records
│   └── refs/
│       └── obsidian/             # Sample Obsidian vault for reference
├── crates/
│   ├── domain/                   # THE INVIOLATE CORE (Logic only, No I/O)
│   │   ├── src/
│   │   │   ├── lib.rs            # Prelude & Common Types
│   │   │   ├── config/           # Config bounded context
│   │   │   │   ├── mod.rs
│   │   │   │   ├── aggregate.rs  # Config aggregate + validation
│   │   │   │   ├── global.rs     # Global filesystem + trusted vaults
│   │   │   │   ├── vault.rs      # Vault filesystem + metadata
│   │   │   │   ├── types.rs      # Shared config value types
│   │   │   │   └── events.rs     # Config events
│   │   │   ├── note/             # Note bounded context
│   │   │   │   ├── mod.rs
│   │   │   │   ├── aggregate.rs  # Note aggregate + subentities
│   │   │   │   ├── frontmatter.rs
│   │   │   │   ├── link.rs
│   │   │   │   ├── structure.rs
│   │   │   │   ├── tag.rs
│   │   │   │   ├── task.rs
│   │   │   │   └── events.rs
│   │   │   ├── schema/           # Schema bounded context
│   │   │   │   ├── mod.rs
│   │   │   │   ├── aggregate.rs  # Schema aggregate + value objects
│   │   │   │   ├── graph.rs
│   │   │   │   ├── resolver.rs
│   │   │   │   ├── property.rs
│   │   │   │   ├── property_bank.rs
│   │   │   │   ├── property_spec.rs
│   │   │   │   ├── raw.rs
│   │   │   │   ├── patterns.rs
│   │   │   │   └── events.rs
│   │   │   ├── template/         # Template bounded context
│   │   │   │   ├── mod.rs
│   │   │   │   ├── aggregate.rs  # Template aggregate + validation
│   │   │   │   ├── composition.rs
│   │   │   │   ├── variable.rs
│   │   │   │   ├── validation.rs
│   │   │   │   ├── syntax.rs
│   │   │   │   └── events.rs
│   │   │   ├── errors.rs         # Domain errors
│   │   │   ├── patterns.rs       # Shared regex patterns
│   │   │   └── validation.rs     # Shared validation utilities
│   │   │   ├── ports/            # HEXAGONAL INTERFACES (API/SPI)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── api/          # DRIVING PORTS
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── command.rs # Command entry Port
│   │   │   │   │   └── ui.rs      # Interactive Prompt/UI Port
│   │   │   │   └── spi/          # DRIVEN PORTS
│   │   │   │       ├── mod.rs
│   │   │   │       ├── repository.rs # Storage/Graph persistence
│   │   │   │       ├── template.rs   # Rendering engine Port
│   │   │   │       ├── markdown.rs   # Content parsing & extraction Port
│   │   │   │       ├── bus.rs        # Event bus Port
│   │   │   │       ├── config.rs     # Config loading Port
│   │   │   │       ├── audit.rs      # Audit logging Port (FR40)
│   │   │   │       └── crypto.rs     # Secret management Port (FR39)
│   │   │   ├── events/           # ADR 0007: Tiered Event Planes
│   │   │   │   ├── mod.rs
│   │   │   │   ├── data.rs       # Reliable (Indexing)
│   │   │   │   ├── control.rs    # Signals (Shutdown)
│   │   │   │   └── state.rs      # Snapshots (LSP)
│   │   │   └── errors.rs         # miette + thiserror definitions
│   ├── app/                      # THE BRAIN (Orchestration & Use Cases)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── commands/         # WRITE USE-CASES (CQRS)
│   │   │   ├── queries/          # READ USE-CASES (CQRS)
│   │   │   ├── compliance/       # Note-Schema Semantic Bridge (Referee)
│   │   │   │   ├── mod.rs
│   │   │   │   └── engine.rs     # Compliance logic (Is note valid for schema?)
│   │   │   ├── template/         # Template Design & Composition (FR9)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── composer.rs   # Multi-section orchestration
│   │   │   │   └── designer.rs   # Schema-driven UI design logic
│   │   │   ├── metrics/          # Vault Analysis & Statistics
│   │   │   │   ├── mod.rs
│   │   │   │   └── calculator.rs # Aggregation logic (Backlinks, Tags)
│   │   │   └── indexer/          # Indexer Actor (MPSC Mailbox)
│   ├── adapters/                 # INFRASTRUCTURE (API/SPI Split)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── api/              # DRIVER ADAPTERS (External -> App)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cli/          # Clap + miette Visual Reports
│   │   │   │   └── lsp/          # State synchronization for IDEs
│   │   │   ├── spi/              # DRIVEN ADAPTERS (App -> External)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── storage/      # Redb Implementation (Reader/Writer split)
│   │   │   │   ├── schema/       # Schema SPI (Loader, Resolver, Validator)
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── loader.rs   # Discovery & FS access
│   │   │   │   │   ├── resolver.rs # $ref & extends logic
│   │   │   │   │   └── validator.rs # Syntactic/Structural check
│   │   │   │   ├── markdown/     # Metadata Extraction (extractor.rs)
│   │   │   │   ├── template/     # MiniJinja Env & functions
│   │   │   │   ├── config/       # Figment hierarchical loader & Encryption
│   │   │   │   ├── events/       # Hybrid EventBus & Auditor impl
│   │   │   │   └── fs/           # Local Filesystem & Atomic Ops
│   │   │   └── dto/              # Transport Objects (Serialization boundaries)
│   └── lithos/                   # BINARY ENTRY POINT
│       └── src/
│           └── main.rs           # DI Root, Runtime Setup, and Logging
├── tests/                        # Automated Tests
│   ├── integration/              # Cross-crate (SPI Mocking)
│   ├── e2e/                      # CLI-driven workflow tests
│   └── arch/                     # Dependency & Boundary enforcement
└── benches/                      # Performance Benchmarks (Criterion)
```

## Architectural Boundaries

**API Boundaries:**

- **CLI (`adapters/api/cli`):** The primary driver. It maps terminal intent to `app/commands` and is the **exclusive owner** of terminal rendering via `miette`.
- **LSP (`adapters/api/lsp`):** A reactive driver. It pulls Redb snapshots via the `watch` state plane to provide sub-50ms IDE features (completion, refactoring).

**Component Boundaries:**

- **Indexer Actor (`app/indexer`):** The single authority for Redb Write Transactions. It ensures the Knowledge Graph remains consistent across concurrent file updates via an MPSC mailbox.
- **Compliance Engine (`app/compliance`):** Acts as the "Referee." It checks if Note metadata satisfies Schema rules. This orchestration logic is separated from pure Model logic to keep the Domain layer lean.

**Service Boundaries:**

- **Template Designer (FR9):** Located in `app/template/designer.rs`. This service implements the **Schema-Driven Design** philosophy. It inspects the linked schema in a template to dynamically drive the **UI Port** prompts, ensuring templates provide a high-quality "guided" experience during creation.
- **Metrics Calculator (`app/metrics`):** Aggregates vault-wide data (backlinks, tag frequency, schema coverage) for system observability.

**Data Boundaries:**

- **Identity (UUID v7):** We use UUID v7 (Time-ordered) instead of paths or numeric IDs.
  - **Performance:** Ensures new notes are appended to Redb B-Tree leaves sequentially, achieving O(1) insertion and zero B-Tree fragmentation.
  - **Persistence:** Allows notes to be moved or renamed while preserving their logical relationships in the Knowledge Graph.
- **Zero-Copy Serialization:** **rkyv** buffers are generated in SPI adapters and passed as `Arc<[u8]>`, allowing the `app` and `api` layers to cast them to domain models without memory duplication.

## Requirements to Structure Mapping

**Feature/Epic Mapping:**

- **Knowledge Graph (FR20-FR25):** `app/queries/`, `adapters/spi/storage/`, `domain/note/aggregate.rs` (Links/Embeds/Tags).
- **Schema & Compliance (FR8-FR14):** `domain/schema/aggregate.rs`, `app/compliance/`, `adapters/spi/schema/`.
- **Template Design (FR1-FR7, FR9):** `domain/template/aggregate.rs`, `app/template/`.
- **Interactive CLI (FR41-FR47):** `adapters/api/cli/`, `domain/ports/api/ui.rs`.

**Cross-Cutting Concerns:**

- **Metadata Extraction:** Handled strictly in `adapters/spi/markdown/extractor.rs` (Adapter layer).
- **Validation Hierarchy:**
  - **Syntactic (Adapter):** Structural validity of YAML/TOML/Schema JSON.
  - **Semantic/Compliance (App):** Contract check between a Note and its Schema.
- **Performance:** Monitored via `benches/`, optimized via `rkyv` byte-layouts.
- **Task Management:** Centralized in `.mise/tasks/` and orchestrated via `mise.toml`.

## Integration Points

**Internal Communication:**

- **Hybrid Bus (ADR 0007):** Tiered channels (`mpsc`, `broadcast`, `watch`) prevent UI lag from blocking the indexing pipeline.
- **DI Container:** The `lithos` crate wires concrete SPI implementations (e.g., `RedbWriter`) to Application services via Constructor Injection.

**External Integrations:**

- **Obsidian Vault:** Interfaced via `adapters/spi/fs` and extracted via `adapters/spi/markdown`.
- **Hierarchical Config:** Managed by `figment` in `adapters/spi/config` (Global -> User -> Project -> Vault -> Env -> Flag).

**Data Flow:**

- **Write Path:** CLI -> App Command -> Indexer Actor -> Redb SPI -> EventBus Publish.
- **Read Path:** CLI -> App Query -> Redb SPI (Zero-copy via rkyv) -> CLI Render.

## File Organization Patterns

**Configuration Files:**

- **Centralized Root:** `Cargo.toml`, `clippy.toml`, `rustfmt.toml` ensure project-wide consistency for AI agents and CI/CD.

**Source Organization:**

- **Consolidated Models:** `note.rs` and `schema.rs` act as cohesive aggregates. They contain all sub-entities (Links, Properties, Specs) to maintain high cohesion and simplify imports.
- **API/SPI Distinction:** `adapters/api` for drivers (Clap/LSP), `adapters/spi` for driven infra (Redb/FS/Schema-Logic).

**Test Organization:**

- **Unit:** Inline `#[cfg(test)]` modules for logic.
- **Integration:** `tests/integration/` for crate boundary testing.
- **Architecture:** `tests/arch/` for dependency enforcement (e.g., ensuring `app` never imports `adapters`).
- **E2E:** `tests/e2e/` for CLI behavior validation.

**Asset Organization:**

- **Docs:** Centralized in `docs/` using `mdBook` layout.
- **Scripts:** All shell logic encapsulated in `.mise/tasks/`.

## Development Workflow Integration

**Development Server Structure:**

- Managed via `mise run dev` which wraps `cargo-watch` for automatic rebuilding and vault re-indexing.

**Build Process Structure:**

- **Mise-first:** `mise run build` handles static linking and binary stripping to produce a zero-dependency artifact.

**Deployment Structure:**

- Statically linked, single-binary distribution. Identity (UUID v7) ensures that notes remain logically consistent even when synced across different filesystems.
