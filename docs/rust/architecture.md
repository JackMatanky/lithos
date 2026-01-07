---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - label: PRD
    path: docs/rust/prd.md
    category: research
  - label: UX Design Spec
    path: docs/rust/ux-design-specification.md
    category: research
  - label: Project Context (Go)
    path: _bmad-output/project-context.md
    category: project_doc
  - label: Elicitation Summary
    path: _bmad-output/planning-artifacts/discovery/elicitation_summary.md
    category: research
  - label: Project Brief
    path: _bmad-output/planning-artifacts/discovery/project_brief.md
    category: research
  - label: Original Go High Level Architecture
    path: _bmad-output/planning-artifacts/architecture/high-level-architecture.md
    category: architecture
  - label: Original Go Tech Stack
    path: _bmad-output/planning-artifacts/architecture/tech-stack.md
    category: architecture
  - label: Original Go Components
    path: _bmad-output/planning-artifacts/architecture/components.md
    category: architecture
  - label: Original Go Data Models
    path: _bmad-output/planning-artifacts/architecture/data-models.md
    category: architecture
  - label: Original Go Coding Standards
    path: _bmad-output/planning-artifacts/architecture/coding-standards.md
    category: architecture
  - label: Course Correction Comprehensive Review
    path: _bmad-output/implementation-artifacts/course_correction/2025-11-05-comprehensive-architectural-review.md
    category: implementation
  - label: Lessons Learned Phase 1
    path: _bmad-output/implementation-artifacts/course_correction/2025-10-27-lessons-learned-phase-1-archive.md
    category: implementation
  - label: Epic Impact Assessment
    path: _bmad-output/implementation-artifacts/course_correction/epic-impact-assessment-corrected.md
    category: implementation
  - label: Digest - Basalt (Rust)
    path: docs/refs/erikjuhani-basalt-digest.txt
    category: research
  - label: Digest - Markdown Oxide (TypeScript)
    path: docs/refs/feel-ix-343-markdown-oxide-digest.txt
    category: research
  - label: Digest - Mdnotes Nvim (Lua)
    path: docs/refs/ymich9963-mdnotes.nvim-digest.txt
    category: research
workflowType: 'architecture'
project_name: 'lithos'
user_name: 'Jack'
date: '2026-01-06'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The project has 50 functional requirements spanning template management (creation, execution, composition, date functions, dynamic commands, custom functions, advanced operations), schema management (definition, validation, inheritance, enums, file filtering, date formatting), interactive input (prompts, suggesters, multi-selection, help, progressive complexity), vault operations (indexing, searching, link resolution, metadata queries, consistency maintenance, large vault performance), configuration management, cross-environment compatibility, community features, security & privacy, command line interface (subcommands, help, status, operations, output formats, configuration, single-word commands), and error handling & recovery. These drive a hexagonal architecture with sophisticated template engines, comprehensive schema validation systems, rich CLI interaction patterns, and robust vault indexing capabilities. The requirements emphasize schema-driven development where schemas provide input parameters and validation rules, enabling modular template composition without manual coding, with extensive support for inheritance, enums, and complex validation.

**Non-Functional Requirements:**
Performance requirements are stringent - template operations under 500ms, vault indexing under 2 seconds for 1000+ files, memory usage under 500MB. Security requires encrypted configuration and audit logging. Scalability needs handle large vaults without degradation. Integration focuses on macOS/Linux cross-platform support with CLI-first design. Usability demands clear help, error recovery, and progressive complexity. Maintainability requires comprehensive testing and self-contained binaries. Compatibility ensures graceful handling of Obsidian vault changes and migration paths. Observability requires comprehensive logging and diagnostics. Reliability demands 99.9% uptime and zero crashes. Deployment requires fast updates with rollback capability.

**Scale & Complexity:**
- Primary domain: CLI developer tool with comprehensive template/schema management and future LSP ecosystem expansion
- Complexity level: medium-high (50 functional requirements, solo developer, ecosystem expansion)
- Estimated architectural components: 12-16 (hexagonal ports/adapters, domain services, CLI framework, storage layers, template engine, schema system, vault indexer, interactive components)
- MVP scope recommendation: Reduce initial scope to 20-25 core functional requirements focusing on template execution, schema validation, and basic vault operations to maintain solo developer velocity and achieve 6-month MVP timeline
- Success metrics: Template creation time reduction (target: 75% faster than manual), crash rate (target: 0%), schema compliance automation (target: 95%)
- Agile guardrails: Weekly sprints with demo deliverables, daily standups, monthly retrospectives despite solo development to maintain discipline

### Technical Constraints & Dependencies

**Core Language:** Rust 1.92+ for memory safety and performance, enabling zero-cost abstractions and compile-time guarantees that prevent GC pauses during complex template composition.

**Platform Support:** macOS and Linux as primary targets, with future Windows support. Single binary distribution with no external runtime dependencies.

**Vault Integration:** Must work with Obsidian vault structure (markdown files, frontmatter, wikilinks) while being app-agnostic and supporting complex schema inheritance and validation.

**Template Engine:** Need to provide powerful templating functionality (prompts, suggesters, date functions, dynamic commands, custom functions, hooks, complex operations) adapted to Rust's capabilities and CLI-first workflow.

**Schema System:** Complex inheritance, validation, and interactive input system that must be performant, user-friendly in terminal environment, and support advanced features like file filtering and date formatting.

**CLI Complexity:** Rich interactive experiences with fuzzy finding, suggesters, multi-selection, progressive help, and single-word commands requiring sophisticated terminal interaction patterns. The CLI-first approach is viable with intelligent interfaces where schemas drive UX - enums become select lists, dates get formatters, with progressive complexity for different user expertise levels.

**Storage & Persistence:** CQRS pattern with **Redb + rkyv** for embedded persistence, separating write concerns (vault indexing, template execution, schema validation) from read concerns (queries, searches, metadata lookups) to avoid confusion between directory-like vault structures and query-optimized representations. This enables zero-copy deserialization essential for LSP performance.

**Async Runtime:** Embrace Rust's async capabilities throughout the architecture using tokio to support interactive CLI responsiveness and concurrent vault operations.

### Cross-Cutting Concerns Identified

**Performance:** Sub-millisecond response times for interactive operations, efficient vault scanning, memory-bounded operations - affects all components with 50 functional requirements. Design with <500ms targets using pre-computed read models and event-driven cache invalidation, with progress indicators for operations exceeding 500ms.

**Error Handling:** Clear, actionable error messages in CLI environment, recovery flows for failed operations, validation feedback across extensive feature set. Error messages need to be conversational and actionable, with rollback capabilities for failed template executions.

**Vault Consistency:** Alias resolution, link validation, schema compliance across entire vault with complex inheritance - requires sophisticated indexing and validation using event-driven patterns from the architectural foundation to enable decoupled services.

**Interactive UX:** Fuzzy finding, schema-driven prompts, suggesters, multi-selection, progressive help - demands advanced CLI interaction patterns for 50+ features, with contextual help and guidance during input operations.

**Cross-Platform Compatibility:** Consistent behavior across macOS/Linux, portable vault paths, terminal compatibility for comprehensive CLI interface.

**Security & Auditing:** Control access to vault data, audit template execution, encrypt sensitive configuration with logging requirements.

**Template/Schema Complexity:** Modular composition, inheritance chains, validation rules, interactive inputs - affects core business logic throughout the system.

**Configuration Management:** TOML-based configuration with extensive settings for templates, schemas, validation rules, and CLI behavior.

**Event-Driven Architecture:** Implement from day one to prevent god-object orchestrators and enable decoupled services for vault-wide operations and future LSP integration.

**Test Architecture:** Mirror hexagonal structure with pure domain tests, adapter integration tests, end-to-end CLI tests. CQRS requires separate test suites for write/read models. Performance testing with criterion benchmarks. Security testing for config encryption. TDD approach targeting 80%+ coverage.

**Documentation Strategy:** Match progressive complexity UX - power users get API docs + advanced guides, new users get quickstart tutorials + guided CLI help. Migration guides critical for adoption. Documentation as code with mdBook focusing on concrete outcomes.

**Open Source Considerations:** Personal project with community contribution potential. Design for contributor onboarding with clear examples, comprehensive documentation, and modular architecture supporting plugin ecosystem development.

## Starter Template Evaluation

### Primary Technology Domain

CLI Tool (Rust) - Complex vault templating system with 50 functional requirements requiring hexagonal architecture, CQRS patterns, and async operations.

### Technical Preferences Confirmed

Based on project requirements analysis: Rust 1.70+, async runtime, hexagonal ports/adapters, CQRS for vault operations, embedded storage. Research of Rust ecosystem patterns confirms these as optimal for complex CLI applications with performance requirements and concurrent operations.

### Starter Options Evaluated

**Generic CLI Templates**: Keats/rust-cli-template and similar provide basic clap setup but lack the sophisticated hexagonal organization, CQRS separation, async infrastructure, and domain modeling patterns required for complex vault operations.

**Custom Single-Crate Setup**: Traditional approach but doesn't scale for 50-FR requirements or enable the semi-microservices development pattern you established in Go.

**Resources Reviewed**: Rust-Trends/example_project_structure provides basic layout. Djamware guide offers organizational principles but lacks the architectural depth of your implementation. Your Go source tree demonstrates the gold standard for hexagonal organization.

### Selected Starter: Workspace-Based Hexagonal Architecture

**Rationale for Selection:**
Cargo workspaces provide the Rust-native foundation for hexagonal architecture in complex applications. This approach enables compile-time enforcement of architectural boundaries, supports parallel development through independent crate compilation, and provides natural evolution toward microservices while maintaining clean separation between domain, application, and infrastructure concerns.

**Workspace Organization Benefits:**
- **Hexagonal Enforcement**: Crate boundaries enforce ports/adapters patterns at compile time
- **Parallel Development**: Independent crate compilation matches your Go development velocity
- **CQRS Support**: Natural separation of commands/queries following your Go patterns
- **Async Native**: Tokio leverages Rust's strengths for concurrent vault operations
- **Testability**: Domain purity enables comprehensive testing like your Go implementation
- **Scalability**: Semi-microservices structure for team growth and ecosystem expansion
- **Architecture Preservation**: Maintains your established hexagonal patterns in Rust

**Initialization Commands:**
```bash
# Create workspace root
mkdir lithos && cd lithos
cargo new crates/domain --lib
cargo new crates/app --lib
cargo new crates/adapters --lib
cargo new crates/cli --bin
```

**Workspace Cargo.toml:**
```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.49", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
redb = "3.1"
rkyv = "0.8"
anyhow = "1.0"
thiserror = "2.0"
miette = { version = "7.6", features = ["fancy"] }
tracing = "0.1"
minijinja = "2.14"
pulldown-cmark = "0.13"
figment = { version = "0.10", features = ["toml", "env"] }
uuid = { version = "1.19", features = ["v7", "serde"] }
```

**Crate Structure (Following Rust Hexagonal Best Practices):**
```
lithos/
├── crates/
│   ├── domain/           # Core business models & logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── models/   # Entities, value objects
│   │       └── ports/    # Traits/interfaces for external dependencies
│   ├── app/              # Application services & orchestrators
│   │   ├── Cargo.toml    # Depends on domain
│   │   └── src/
│   │       ├── commands/ # CQRS command handlers
│   │       ├── queries/  # CQRS query handlers
│   │       ├── vault/    # VaultIndexer orchestrator
│   │       ├── schema/   # SchemaEngine orchestrator
│   │       └── template/ # TemplateEngine orchestrator
│   ├── adapters/         # Infrastructure implementations
│   │   ├── Cargo.toml    # Depends on domain + external crates
│   │   └── src/
│   │       ├── api/      # Driver adapters (CLI, future LSP)
│   │       ├── spi/      # Driven adapters (storage, filesystem, config)
│   │       └── dto/      # Data transfer objects
│   └── cli/              # Binary entry point
│       ├── Cargo.toml    # Depends on app + adapters
│       └── src/main.rs
└── Cargo.lock
```

**Architectural Decisions (Following Rust Ecosystem Patterns):**
- **Workspace Enforcement**: Crate boundaries enforce hexagonal dependency inversion using Rust's module system
- **Domain Purity**: Domain crate contains only business logic, no external dependencies (standard Rust practice)
- **CQRS Implementation**: App crate separates commands (writes) from queries (reads) using async patterns
- **Adapter Pattern**: Adapters crate implements domain traits with external systems using Rust's trait system
- **Semi-Microservices**: Workspace enables parallel development and future service extraction
- **Async Native**: Tokio integration across crates for concurrent vault operations leveraging Rust's async strengths
- **Testing Architecture**: Domain tests require no setup, integration tests span crates using Rust's testing framework
- **Development Velocity**: Independent crate compilation optimizes Rust's incremental compilation

**Development Benefits:**
- **Clean Boundaries**: Compile-time enforcement of hexagonal architectural rules using Rust's ownership system
- **Parallel Iteration**: Domain, application, and infrastructure developed independently with cargo's workspace features
- **Testability**: Pure domain logic tested without infrastructure complexity using Rust's unit testing
- **Scalability**: Natural evolution path for team growth and microservices using Rust's ecosystem patterns
- **Performance**: Leverages Rust's zero-cost abstractions and async runtime for high-performance CLI operations

**Note:** Project initialization using this workspace structure should be the first implementation story, establishing the hexagonal foundation following Rust ecosystem best practices for complex applications.

## Core Architectural Decisions

### Data Architecture

**Already Decided by Workspace Setup:**
- **Database Choice:** **Redb 3.1** for embedded key-value storage with ACID transactions and MVCC, paired with **rkyv 0.8** for zero-copy deserialization.
- **Data Modeling:** Domain entities in domain crate, transport DTOs in adapters crate following Rust's ownership patterns for data integrity.
- **Data Validation:** Three-phase pipeline (Syntactic -> Orchestration -> Semantic) with typed errors.
- **Migration:** Schema versioning in the Redb tables with startup validation.
- **Caching:** Event-driven invalidation using the state watch plane (ADR 006).

**Path as Identity Decision:** Use vault-relative string paths directly as Note identifiers (no NoteID wrapper) to maintain simplicity and direct filesystem correspondence. Immutable Note entities with embedded Frontmatter following Rust's ownership patterns for data integrity. This approach avoids abstraction complexity while maintaining data consistency and supports the 75% faster template performance target.

**Typed Error Hierarchy:** Implement comprehensive error types using thiserror for domain-specific enums and anyhow for ergonomic context chaining, ensuring user-friendly CLI feedback and proper error propagation across hexagonal boundaries that mirror hexagonal testing patterns.

### Authentication & Security

**Decisions for CLI Tool:**
- **Authentication:** None required - operates on local files with OS-level permissions
- **Authorization:** Respect filesystem permissions, fail gracefully on access denied
- **Security:** Optional configuration encryption for sensitive TOML sections using Rust's crypto ecosystem
- **Auditing:** Structured tracing logs with correlation IDs for template execution, vault operations, and schema validation

**Audit Trail Implementation:** Comprehensive logging of all operations with trace IDs for debugging complex vault interactions, supporting extensive observability requirements and enabling security testing for config encryption.

### API & Communication Patterns

**Decisions for Internal Architecture:**
- **API Design:** Domain traits as contracts between crates (matches Rust's trait system strengths)
- **Communication:** Direct function calls within hexagonal boundaries with async where needed
- **Error Handling:** Thiserror for typed errors, anyhow for ergonomic context chaining
- **Event System:** Async event bus with channels for decoupling vault operations, enabling concurrent processing and preventing god-object orchestrators

**Async Event Processing:** Implement event-driven architecture from the foundation using Tokio channels for vault-wide consistency, supporting complex async operations and future LSP integration without blocking performance. This enables phased ecosystem delivery while maintaining solo developer velocity through parallel development capabilities.

### Infrastructure & Deployment

**Decisions for Single Binary:**
- **Hosting:** Local execution only with self-contained binary
- **CI/CD:** GitHub Actions for automated testing, linting, and cross-platform builds
- **Environment:** Hierarchical TOML configuration (global → user → project) with validation using singleton pattern
- **Monitoring:** Tracing with configurable output levels and performance metrics
- **Scaling:** Optimize algorithms for large vault performance using Rust's zero-cost abstractions

**Configuration Singleton:** Use OnceCell for lazy initialization of configuration, loaded once at startup and injected across crates, supporting hot reload for development while maintaining thread safety. This supports agile workflow maintenance with weekly sprints and independent crate development.

### Decision Impact Analysis

**Implementation Sequence:**
1. Initialize workspace structure and core dependencies
2. Implement domain models (Note, Frontmatter, Schema) with ports
3. Build adapters (storage, filesystem, CLI) implementing ports
4. Create app services (VaultIndexer, QueryService, TemplateEngine)
5. Wire CLI binary and test end-to-end

**Cross-Component Dependencies:**
- Domain ports define contracts that adapters must implement
- App services orchestrate domain logic through ports
- Event system enables loose coupling between services
- Configuration singleton loaded at startup and injected

**Testing Strategy:**
- Domain crate: Pure unit tests with no external dependencies
- Integration tests: Cross-crate testing with test adapters
- Performance benchmarks: Criterion for 500ms target validation
- Async testing: Tokio::test for concurrent operation testing
- Property-based testing: QuickCheck for edge case discovery

**MVP Scope Discipline:** Focus initial implementation on 20-25 core functional requirements targeting template execution, schema validation, and basic vault operations to achieve 6-month MVP timeline with 95% schema compliance automation.

**Documentation Strategy:** Progressive documentation matching CLI complexity levels, with clear API docs for power users and guided tutorials for beginners, focusing on concrete outcomes like 75% faster template creation.

**Risk Mitigation:**
- Workspace boundaries prevent architectural drift
- Comprehensive testing catches integration issues early
- Event-driven design enables future LSP integration
- Performance benchmarks ensure NFR compliance

**Future Evolution Path:**
- Workspace structure supports microservices extraction
- Event system foundation enables distributed processing
- Configuration hierarchy supports enterprise deployment
- Testing patterns scale with team growth

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- **Storage Engine:** Redb + rkyv (Zero-copy structured KV). [ADR 001](adr/001-storage-redb-rkyv.md)
- **Templating:** MiniJinja (Dynamic Jinja2). [ADR 002](adr/002-template-engine.md)
- **Markdown Parser:** pulldown-cmark (Event-streaming). [ADR 003](adr/003-markdown-parsing.md)
- **Configuration:** Figment (Provider-based hierarchy). [ADR 004](adr/004-configuration-management.md)
- **Error Handling:** miette + thiserror (Structured diagnostics). [ADR 005](adr/005-error-handling-diagnostics.md)
- **Event Orchestration:** Hybrid MPSC/Broadcast/Watch. [ADR 006](adr/006-event-orchestration.md)

**Important Decisions (Shape Architecture):**
- **Workspace:** Cargo Workspaces for Hexagonal boundaries.
- **Identity:** UUID v7 (Standardized sortable identifiers).
- **Concurrency:** Tokio-based async runtime.

**Deferred Decisions (Post-MVP):**
- **LSP Implementation details.**
- **Plugin architecture specifications.**

### Data Architecture
- **Engine:** Redb (Pure-Rust, ACID KV) with **rkyv** zero-copy serialization for high-frequency LSP lookups.
- **Identity:** UUID v7. Decouples identity from physical path to avoid the "directory trap."
- **ADR Reference:** [ADR 001: Storage - Redb + rkyv](adr/001-storage-redb-rkyv.md)

### Internal Communication
- **Strategy:** **Hybrid Event Orchestration**. Uses a Tiered model:
    - **Data Plane (MPSC):** Reliable indexing via Actor pattern.
    - **Control Plane (Broadcast):** Global status and notifications.
    - **State Plane (Watch):** Zero-latency LSP state synchronization.
- **ADR Reference:** [ADR 006: Event Orchestration](adr/006-event-orchestration.md)

### Technical Preferences (Step 4 Refinement)
- **Templating:** **MiniJinja**. Selected for "Mechanical Sympathy"—minimal dependencies and VM-based rendering for user-defined Markdown templates. [ADR 002](adr/002-template-engine.md)
- **Markdown:** **pulldown-cmark**. Enables high-speed link extraction via event streaming without building expensive ASTs. [ADR 003](adr/003-markdown-parsing.md)
- **Configuration:** **Figment**. Uses the Provider pattern to elegantly handle the 6-layer priority hierarchy. [ADR 004](adr/004-configuration-management.md)
- **Errors/Diagnostics:** **miette**. Provides high-fidelity terminal snippets and 1:1 mapping to LSP Diagnostic objects. [ADR 005](adr/005-error-handling-diagnostics.md)

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
lithos/
├── .gitattributes                # LF enforcement
├── .gitignore                    # standard Rust ignores
├── .mise/                        # TASK ORCHESTRATION (mise-first)
│   └── scripts/
│       ├── dev-setup.sh          # Env bootstrap
│       ├── run-benchmarks.sh     # Performance validation (Criterion)
│       └── install-hooks.sh      # Git hook setup
├── .pre-commit-config.yaml       # Quality gates (miette, clippy, rustfmt)
├── Cargo.toml                    # Workspace configuration (Rust 1.92+)
├── Cargo.lock                    # Dependency lock file
├── mise.toml                     # Task definitions & tool versions
├── deny.toml                     # Dependency license & security policy
├── rustfmt.toml                  # Formatting (import sorting)
├── clippy.toml                   # Complexity limits (cognitive < 15)
├── README.md                     # Project overview
├── docs/                         # Documentation
│   ├── rust/
│   │   ├── architecture.md       # This document
│   │   ├── adr/                  # Architectural Decision Records (001-006)
│   │   └── prd.md                # Product requirements
├── crates/
│   ├── domain/                   # THE INVIOLATE CORE (Logic only)
│   │   ├── src/
│   │   │   ├── lib.rs            # Prelude & Common Types
│   │   │   ├── models/           # Identity & Unified Aggregates
│   │   │   │   ├── mod.rs
│   │   │   │   ├── identity.rs   # UUID v7 (Time-ordered keys)
│   │   │   │   ├── note.rs       # Note Root + Link, Heading, Task, Tag, Position
│   │   │   │   ├── schema.rs     # Schema + PropertyBank + PropertySpec
│   │   │   │   └── template.rs   # Template Syntax & Design models
│   │   │   ├── ports/            # HEXAGONAL INTERFACES (API/SPI)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── api/          # DRIVING PORTS
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── command.rs # Use case entry Port
│   │   │   │   │   └── ui.rs      # Prompt/Interaction Port
│   │   │   │   └── spi/          # DRIVEN PORTS
│   │   │   │       ├── mod.rs
│   │   │   │       ├── repository.rs # Storage/Graph persistence
│   │   │   │       ├── template.rs   # Rendering engine Port
│   │   │   │       ├── markdown.rs   # Content parsing Port
│   │   │   │       ├── bus.rs        # Event bus Port
│   │   │   │       └── config.rs     # Config loading Port
│   │   │   ├── events.rs         # Domain Event Enums
│   │   │   └── errors.rs         # miette + thiserror definitions
│   ├── app/                      # THE BRAIN (Orchestration)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── commands/         # WRITE USE-CASES
│   │   │   ├── queries/          # READ USE-CASES
│   │   │   ├── compliance/       # Note-Schema Compliance (Referee)
│   │   │   │   ├── mod.rs
│   │   │   │   └── engine.rs     # Compliance logic (Note vs Schema)
│   │   │   ├── template/         # Template Generation (FR9)
│   │   │   │   ├── mod.rs
│   │   │   │   └── composer.rs   # Schema-driven generation & prompts
│   │   │   ├── metrics/          # Vault-wide statistics & analysis
│   │   │   │   ├── mod.rs
│   │   │   │   └── calculator.rs # Aggregation logic
│   │   │   └── indexer/          # Indexer Actor (MPSC Mailbox)
│   ├── adapters/                 # INFRASTRUCTURE (API/SPI Split)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── api/              # DRIVER ADAPTERS (External -> App)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cli/          # Clap + miette Visual Reports
│   │   │   │   └── lsp/          # State synchronization
│   │   │   ├── spi/              # DRIVEN ADAPTERS (App -> External)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── storage/      # Redb Decomposed Tables impl
│   │   │   │   ├── schema/       # Schema SPI (Rigor from Go)
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── loader.rs   # Discovery
│   │   │   │   │   ├── resolver.rs # $ref & inheritance
│   │   │   │   │   └── validator.rs # Syntactic structure check
│   │   │   │   ├── markdown/     # Extraction & pulldown-cmark impl
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── extractor.rs # Metadata extraction (from files)
│   │   │   │   ├── template/     # MiniJinja Env & functions
│   │   │   │   ├── config/       # Figment hierarchical loader
│   │   │   │   ├── events/       # Hybrid EventBus impl
│   │   │   │   └── fs/           # OS Atomic operations
│   │   │   └── dto/              # Transport Objects
│   └── lithos/                   # BINARY ENTRY POINT
│       └── src/
│           └── main.rs           # DI Root & Runtime Init
├── tests/                        # Integration & E2E
└── benches/                      # Performance Benchmarks
```

### Architectural Boundaries

**API Boundaries:**
- **CLI (`adapters/api/cli`):** The primary driver. Maps terminal commands to `app/commands`. Renders **miette** graphical diagnostics for both syntactic (parsing) and semantic (compliance) errors.
- **LSP (`adapters/api/lsp`):** A reactive driver. Pulls Redb snapshots via the `watch` state plane to provide sub-50ms link completion and jumps.

**Component Boundaries:**
- **Indexer Actor (`app/indexer`):** The exclusive writer for Redb. It ensures knowledge graph consistency across concurrent file updates.
- **Compliance Engine (`app/compliance`):** Acts as the "Referee." It checks if Note metadata satisfies Schema rules. This is separated from domain models to keep them focused on data structure.

**Service Boundaries:**
- **Template Composer (FR9):** Orchestrates **Schema-Driven Design**. It inspects the template's linked schema to dynamically drive the **UI Port** prompts, ensuring templates are "design-correct" before they are rendered.
- **Metrics Calculator (`app/metrics`):** Aggregates graph data (Backlinks, Tag frequency, Schema usage) for observability.

**Data Boundaries:**
- **Identity (UUID v7):** We use UUID v7 (Time-ordered) instead of numeric `st_uid` or paths.
    - **Rationale:** `st_uid` identifies the OS user owning the file. UUID v7 identifies the **logical entity**. Because it is time-ordered, Redb (B-Tree) can append new notes with O(1) performance and zero fragmentation.
- **Zero-Copy Paths:** **rkyv** buffers are generated in SPI adapters and passed as `Arc<[u8]>`, allowing the `app` and `api` layers to cast them to models without copying.

### Requirements to Structure Mapping

**Feature/Epic Mapping:**
- **Knowledge Graph (FR20-FR25)** → `app/queries/`, `adapters/spi/storage/`, `domain/models/note.rs` (Link metadata).
- **Schema Compliance (FR8-FR14)** → `domain/models/schema.rs`, `app/compliance/`, `adapters/spi/schema/`.
- **Template Design (FR1-FR7, FR9)** → `domain/models/template.rs`, `app/template/composer.rs`.
- **CLI Interaction (FR41-FR47)** → `adapters/api/cli/`, `domain/ports/api/ui.rs`.

**Cross-Cutting Concerns:**
- **Metadata Extraction:** Handled in `adapters/spi/markdown/extractor.rs` (Adapter Layer).
- **Validation Layers:**
    - **Syntactic:** SPI Adapters (Config/Markdown/Schema).
    - **Semantic:** `app/compliance/engine.rs`.
- **Task Management:** Centralized in `.mise/scripts/`.

### Integration Points

**Internal Communication:**
- **Hybrid Bus (ADR 006):** Tiered channels (`mpsc`, `broadcast`, `watch`) prevent UI lag from blocking the indexing pipeline.
- **DI Container:** The `lithos` crate wires concrete SPI implementations (e.g., `RedbWriter`) to Application services via constructor injection.

**External Integrations:**
- **Obsidian Vault:** Interfaced via `adapters/spi/fs` and parsed via `adapters/spi/markdown`.
- **Hierarchical Config:** Managed by `figment` in `adapters/spi/config`.

**Data Flow:**
- **Write Path:** CLI -> App Command -> Indexer Actor -> Redb SPI -> EventBus.
- **Read Path:** CLI -> App Query -> Redb SPI (Zero-copy) -> CLI Render.

### File Organization Patterns

**Configuration Files:**
- **Centralized Root:** `Cargo.toml`, `clippy.toml`, `rustfmt.toml` ensure project-wide consistency for AI agents.

**Source Organization:**
- **Consolidated Models:** `note.rs` and `schema.rs` act as cohesive aggregates. They contain all sub-entities (Links, Properties) to maintain local reasoning until size requires split.
- **API/SPI Distinction:** `adapters/api` for drivers (Clap/LSP), `adapters/spi` for driven infra (Redb/FS).

**Test Organization:**
- **Unit:** `#[cfg(test)]` modules.
- **Integration:** `tests/integration/` (Crate boundaries).
- **E2E:** `tests/e2e/` (Binary behavior).

### Development Workflow Integration

**Development Server Structure:**
- `mise run dev`: Wraps `cargo-watch` for hot-reloading development and re-indexing.

**Build Process Structure:**
- `mise run build`: Handles static linking and stripping for a zero-dependency final binary.

**Deployment Structure:**
- Statically linked, single-binary distribution. Identity (UUID v7) ensures vaults remain portable across machines.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:** 30+ areas where AI agents could make different choices in async Rust CLI applications with hexagonal architecture, CQRS, and event-driven patterns.

### Naming Patterns

**Rust Naming Conventions:**
- **Modules & Files:** `snake_case` (e.g., `vault_indexer.rs`, `frontmatter_service.rs`)
- **Functions & Variables:** `snake_case` (e.g., `execute_template`, `vault_path`)
- **Structs & Enums:** `PascalCase` (e.g., `Note`, `DomainError`, `TemplateEngine`)
- **Traits:** `PascalCase` ending with trait name (e.g., `CacheWriter`, `VaultReader`) or `Port` (e.g., `StoragePort`)
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `MAX_VAULT_SIZE`, `DEFAULT_TIMEOUT`)
- **Crate Names:** `snake_case` matching directory (e.g., `lithos-domain`, `lithos-app`)
- **Test Functions:** `snake_case` with `test_` prefix (e.g., `test_execute_template`)
- **Macros:** `snake_case` (e.g., `my_macro!`)

**API Contract Naming:**
- **Trait Methods:** `snake_case` with clear action verbs (e.g., `persist_note`, `find_templates`)
- **Port Traits:** Descriptive names ending with `Port` (e.g., `CacheWriterPort`, `VaultReaderPort`)
- **DTO Structs:** `PascalCase` ending with `Dto` (e.g., `VaultFileDto`, `FileDatesDto`)
- **Event Names:** `PascalCase` with past tense (e.g., `NoteIndexed`, `TemplateExecuted`)

### Structure Patterns

**Workspace Organization:**
- **Crate Separation:** Strict hexagonal boundaries - domain depends on nothing, app depends only on domain, adapters depend on domain + external crates, cli depends on app + adapters
- **Module Organization:** Within crates, use `mod.rs` for submodules, keep related functionality together; avoid deep nesting (max 3 levels)
- **Test Placement:** Unit tests in same file as implementation (`#[cfg(test)]`), integration tests in `tests/` directory at crate root, performance tests in `benches/`
- **Binary Organization:** CLI crate should be minimal, delegating to library crates

**File Structure Standards:**
- **Domain Crate:** `src/models/`, `src/ports/`, `src/services/` (for pure domain services)
- **App Crate:** `src/commands/`, `src/queries/`, `src/vault/`, `src/schema/`, `src/template/`
- **Adapters Crate:** `src/api/`, `src/spi/`, `src/dto/`
- **CLI Crate:** `src/main.rs` only, all logic in other crates
- **Common Patterns:** Group related items, use `prelude.rs` for common imports, keep files focused

### Format Patterns

**Error Handling Standards:**
- **Domain Errors:** `thiserror::Error` for typed error enums with descriptive messages and `#[from]` for conversions
- **Context Addition:** `anyhow::Result` for ergonomic error chaining in application code with `context!` macro
- **CLI Output:** User-friendly error messages with actionable guidance using `color-eyre` for pretty printing
- **Logging:** `tracing` with structured spans, consistent log levels, and subscriber setup in CLI crate
- **Panic Avoidance:** Never use `unwrap()`, `expect()`, `panic!()` in library code; prefer `Result`

**Async Patterns:**
- **Runtime:** Tokio as the async runtime throughout all crates with `#[tokio::main]` in CLI
- **Trait Methods:** Use `async_trait` for async trait methods with `Send + Sync` bounds
- **Error Propagation:** `?` operator for clean error bubbling, `map_err` for context addition
- **Cancellation:** Accept `CancellationToken` in long-running operations with `select!` for graceful shutdown
- **Channels:** Use `tokio::sync::mpsc` for event buses, bounded channels to prevent memory issues
- **Futures:** Prefer `async fn` over manual `Future` implementations, use `async move` for owned data

**Documentation Standards:**
- **Item Documentation:** Use `///` for functions, structs, traits, and other items
- **Module Documentation:** Use `//!` for module-level documentation
- **Examples:** Include code examples in documentation where helpful
- **Error Documentation:** Document error conditions and panic scenarios
- **Formatting:** Use markdown formatting in doc comments
- **Links:** Reference related items and external documentation

### Communication Patterns

**Event System Standards:**
- **Event Naming:** `PascalCase` with past tense (e.g., `NoteIndexed`, `VaultIndexingCompleted`)
- **Event Payloads:** Immutable structs with clear field names and derive macros for serialization
- **Event Bus:** Async channels with bounded capacity and weak subscriber references to prevent leaks
- **Subscriber Patterns:** Handler functions with proper error isolation and logging

**Inter-Crate Communication:**
- **Dependency Injection:** Constructor injection of port implementations with `Arc<dyn Trait>` for shared ownership
- **Trait Objects:** Use `Box<dyn Trait>` for runtime polymorphism with `Send + Sync` bounds
- **Type Safety:** Leverage Rust's compile-time guarantees over runtime checks
- **Configuration Passing:** Use `Arc<Config>` for shared immutable configuration across crates

### Process Patterns

**Testing Standards:**
- **Unit Tests:** Pure domain logic with no external dependencies using `#[cfg(test)]`; test both success and error cases
- **Integration Tests:** Cross-crate testing with test adapters using `tokio::test`; test real implementations
- **Performance Tests:** Criterion benchmarks for 500ms targets with statistical analysis
- **Async Testing:** `tokio::test` for concurrent operation testing with proper setup/teardown
- **Mocking:** Test doubles for ports using `mockall` crate or manual implementations
- **Property-Based Testing:** `proptest` for edge case discovery in domain logic
- **Test Organization:** Group tests by functionality, use descriptive names, avoid flaky tests; include doc tests

**Configuration Management:**
- **Hierarchy:** CLI flags > Environment variables > Config file > Defaults with precedence documentation
- **Validation:** Compile-time validation using Serde derive with custom validators
- **Singleton Pattern:** `OnceCell` for lazy initialization with thread-safe access
- **Hot Reload:** File watching with `notify` crate for development (optional)
- **Environment-Specific:** Different config files for dev/staging/prod

**Build & Development:**
- **Cargo Profiles:** Separate debug/release profiles with appropriate optimizations
- **Linting:** Clippy with all pedantic and nursery lints enabled; deny complexity violations
- **Formatting:** Rustfmt with standard configuration and import sorting
- **CI/CD:** GitHub Actions with cross-platform testing, coverage reporting, and security auditing
- **Development Tools:** `cargo-watch` for auto-rebuild, `cargo-expand` for macro debugging, **mise 2026.1.0** for tool version management and task execution via `.mise/scripts/`
- **Pre-commit Hooks:** Use pre-commit framework to run clippy, rustfmt, and tests before commits for maximum visibility and clean git history

**Clippy Complexity Limits:**
- **Cyclomatic Complexity:** `clippy::cognitive_complexity` threshold set to 15 (warn) and 25 (deny) to prevent overly complex functions
- **Function Length:** `clippy::too_many_lines` with limit of 100 lines per function
- **Arguments:** `clippy::too_many_arguments` with max 7 arguments
- **Nesting:** `clippy::nested_if_else` and `clippy::too_many_nested_loops` limits enforced
- **Code Quality:** Deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`
- **Performance:** Enable `clippy::inefficient_to_string`, `clippy::redundant_clone`, `clippy::needless_collect`
- **Style:** Enforce `clippy::implicit_return`, `clippy::single_match_else`, `clippy::redundant_else`

### Enforcement Guidelines

**All AI Agents MUST:**
- Follow established naming conventions without exception
- Maintain hexagonal architecture boundaries (no domain → adapters dependencies)
- Use async/await consistently throughout the codebase with proper error handling
- Implement comprehensive error handling with typed errors and context
- Write tests for all public APIs and critical paths including async operations
- Document public traits and complex business logic with examples following Rust doc standards
- Use tracing for all logging with structured spans and consistent levels
- Keep cyclomatic complexity under 15 and cognitive complexity under 25 per function
- Never use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code
- Run clippy on all code with complexity limits enforced before commits via pre-commit hooks

**Pattern Enforcement:**
- **Pre-commit Hooks:** Run clippy, rustfmt, and tests before commits to maintain clean git history and catch issues early
- **Code Reviews:** Automated checks for naming violations, dependency rules, architectural boundaries, and complexity metrics; manual review for logic and API design
- **CI Pipeline:** Clippy with complexity limits, rustfmt, and custom lint enforcement with failure on violations; require green CI for merges
- **Architecture Tests:** Integration tests verifying crate boundaries and hexagonal rules
- **Documentation:** Pattern violations documented in commit messages with remediation steps
- **Quality Gates:** Minimum test coverage (80%), no clippy warnings, performance regression checks, security audit passing

**Advanced Enforcement:**
- **Dependency Analysis:** Use `cargo deny` to prevent unwanted dependency introductions
- **Security Auditing:** Regular `cargo audit` runs to catch vulnerabilities
- **Performance Regression:** Automated benchmark comparisons to prevent performance degradation
- **Code Coverage:** Minimum coverage thresholds enforced in CI with `tarpaulin`
- **Style Consistency:** Automated import sorting and formatting checks; use `cargo fmt --check` in CI

### Pattern Examples

**Good Examples:**
```rust
/// A note in the vault with its metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Vault-relative path serving as the unique identifier
    pub path: String,
    /// Parsed frontmatter metadata
    pub frontmatter: Frontmatter,
}

impl Note {
    /// Creates a new note with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::Note;
    /// let note = Note::new("path.md".to_string(), frontmatter)?;
    /// ```
    pub fn new(path: String, frontmatter: Frontmatter) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::InvalidPath);
        }
        Ok(Self { path, frontmatter })
    }
}

#[async_trait]
pub trait VaultWriterPort: Send + Sync {
    /// Persists a note to the vault storage.
    ///
    /// # Errors
    /// Returns `DomainError` if persistence fails.
    async fn persist_note(&self, note: Note) -> Result<(), DomainError>;
}

pub struct VaultIndexerService {
    vault_writer: Arc<dyn VaultWriterPort>,
    event_bus: Arc<dyn EventBus>,
}

impl VaultIndexerService {
    /// Indexes the vault and publishes completion events.
    ///
    /// This function maintains low complexity by delegating to helper methods.
    pub async fn index_vault(&self) -> Result<IndexStats, DomainError> {
        self.event_bus.publish(DomainEvent::VaultIndexingStarted).await?;

        let stats = self.perform_indexing().await?;

        self.event_bus.publish(DomainEvent::VaultIndexingCompleted { stats: stats.clone() }).await?;

        Ok(stats)
    }

    async fn perform_indexing(&self) -> Result<IndexStats, DomainError> {
        Ok(IndexStats::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Fired when vault indexing begins
    VaultIndexingStarted,
    /// Fired when vault indexing completes
    VaultIndexingCompleted { stats: IndexStats },
    /// Fired when a note is indexed
    NoteIndexed { note_path: String, indexed_at: DateTime<Utc> },
    /// Fired when a template is executed
    TemplateExecuted { template_id: String, success: bool },
}

#[tokio::test]
async fn test_vault_indexing_success() {
    let mock_writer = Arc::new(MockVaultWriter::new());
    let mock_bus = Arc::new(MockEventBus::new());
    let service = VaultIndexerService::new(mock_writer, mock_bus);

    let result = service.index_vault().await;
    assert!(result.is_ok());

    let stats = result.unwrap();
    assert_eq!(stats.total_notes, 0);
}
```

**Anti-Patterns:**
- Functions exceeding 15 cyclomatic complexity or 100 lines
- Using `unwrap()` or `expect()` in production code
- Deeply nested control structures
- Inconsistent naming or missing documentation
- Blocking operations in async functions without `spawn_blocking`
- Tests that don't cover error cases or async behavior
- Missing doc examples for public APIs
- Not running clippy or ignoring warnings

**Resource References:**
- Rust Official Documentation
- Clippy Lints Reference
- Tokio Async Patterns
