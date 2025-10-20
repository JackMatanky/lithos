# High Level Architecture

## Technical Summary

Lithos employs a **Hexagonal Architecture (Ports & Adapters)** pattern built as a single-binary Go CLI application. The core domain logic (template engine, schema system, validator) remains pure and framework-agnostic, depending only on port interfaces it defines. External concerns (CLI framework, file I/O, interactive UI) are implemented as adapters that plug into these ports. This architecture directly supports the PRD's "Interface-Driven Architecture" principle, enables trivial testing through adapter substitution, and naturally accommodates the post-MVP roadmap where multiple adapters (CLI, TUI, LSP server) will interact with the same core logic.

## High Level Overview

**Architectural Style:** Hexagonal Architecture (Ports & Adapters)

The system is organized around a **pure domain core** containing all business logic for template processing, schema validation, and note generation. This core defines port interfaces for external dependencies (storage, user interaction, file system). **Adapters** implement these ports using specific technologies (Cobra CLI, file-based cache, PromptUI). The core never depends on adapters; dependency arrows point inward toward the domain.

**Repository Structure:** Single Repository (Monorepo)

The project uses a **single Git repository** containing the Go application source code, test data vault (`testdata/`), and supporting documentation. This aligns with the PRD's MVP scope and solo developer resource constraint.

**Service Architecture:** Monolithic Binary

A **self-contained Go binary** with no external runtime dependencies, distributed as platform-specific executables (macOS x86_64/ARM64, Linux x86_64/ARM64, Windows x86_64). The hexagonal core compiles alongside its adapters into one binary for the MVP; the architecture supports future distribution as a library (Go module) post-MVP.

### Primary User Interaction Flow

1. **CLI Adapter** (Cobra) receives command (`lithos new`, `find`, `index`)
2. **Configuration Adapter** (Viper) loads `lithos.yaml` via FileSystem port
3. **Core Domain** orchestrates:
   - Template Engine loads template via FileSystem port
   - Executes template, calling Interactive port for prompts/suggesters
   - Query Engine retrieves vault data via Storage port
   - Validator checks output via Schema port
4. **FileSystem Adapter** writes generated note to vault
5. **CLI Adapter** displays confirmation or errors to user

### Key Architectural Decisions

- **Hexagonal Over Layered:** Chosen to isolate domain logic from framework churn (Cobra today, TUI tomorrow). Aligns with PRD Technical Assumptions: "Interface-Driven Architecture" and post-MVP vision (Phase 4: LSP, TUI). The slight upfront cost of defining ports is justified by the PRD's explicit future roadmap.

- **Ports Defined by Core:** The domain defines what it needs (StoragePort, InteractivePort, FileSystemPort), not what adapters provide. This prevents core logic from coupling to adapter implementation details (e.g., core doesn't know about Cobra flags or PromptUI styling).

- **Single Binary for MVP:** While hexagonal architecture supports microservices, the PRD requires "single standalone binary" (NFR2). All adapters compile with the core. Future: Core could be extracted as a Go module consumed by multiple binaries (CLI, LSP server, TUI).

- **Test Adapters as First-Class Citizens:** Epic 1.1 test vault and Epic 4.1 interactive test harness are implemented as test adapters. Production code and test code use the same ports, eliminating the need for mocking frameworks.

- **CQRS Applied to Storage Layer Only:** CQRS (Command Query Responsibility Segregation) pattern is applied exclusively to storage layer ports and adapters (Cache and FileSystem) for independent read/write optimization. Commands write data (indexing, file creation), queries read data (lookups, searches). Domain services coordinate CQRS storage but use single unified models (Note, Schema, Template) - not separate read/write models. This pragmatic approach provides CQRS benefits (scalability, independent optimization) without model proliferation complexity. Post-MVP: Consider true CQRS with event sourcing (separate event store for writes, materialized views for reads) if performance requirements demand it.

## High Level Project Diagram

```mermaid
graph TB
    User[User] -->|CLI commands| Lithos[Lithos CLI<br/>Single Go Binary]

    Lithos -->|reads/writes| Vault[(Obsidian Vault<br/>Local Filesystem)]
    Lithos -->|loads| Config[lithos.yaml<br/>Configuration]

    Vault -->|notes| Notes[notes/*.md]
    Vault -->|schemas| Schemas[schemas/*.json]
    Vault -->|templates| Templates[templates/*.md]
    Vault -->|field banks| FieldBanks[schemas/properties/*.json]
    Vault -->|cache| Cache[.lithos/cache/*.json]

    Lithos -.->|future| TUI[TUI Interface]
    Lithos -.->|future| LSP[LSP Server]

    style Lithos fill:#e8f4f8
    style Vault fill:#fff4e1
    style TUI fill:#f0f0f0
    style LSP fill:#f0f0f0
```

**System Boundaries:**

- **Lithos CLI:** Single standalone Go binary, runs locally on user's machine
- **Obsidian Vault:** Local directory containing markdown notes, JSON schemas, and templates
- **No External Services:** Entirely local operation, no network dependencies for MVP

**Data Flow:**

1. User invokes CLI commands (`lithos new`, `lithos index`, `lithos find`)
2. Lithos reads configuration from `lithos.yaml` (with defaults if not present)
3. Lithos reads templates, schemas, and field banks from vault
4. Lithos generates notes with user interaction (prompts, fuzzy finding)
5. Lithos writes generated notes and cache to vault
6. Lithos validates notes against schemas

**Future Integrations (Post-MVP):**

- TUI interface for terminal-based knowledge management
- LSP server for IDE integration with VS Code, NeoVim

## Architectural and Design Patterns

**1. Hexagonal Architecture (Ports & Adapters)**

*Core domain defines port interfaces; external adapters implement them*

- **Rationale:** Isolates business logic from framework changes. When Phase 4 adds TUI (post-MVP), only a new primary adapter is needed—core remains untouched. Aligns with PRD principle: "Interface-Driven Architecture" and enables trivial testing (swap production adapters for test doubles). → *(Supports Post-MVP Vision: TUI, LSP, Logseq integration)*

*Note: Each port naturally enables the Strategy Pattern—multiple adapter implementations can be swapped at runtime (e.g., production PromptUIAdapter vs. test MockInteractiveAdapter for the InteractivePort).*

**2. Repository Pattern (via StoragePort)**

*StoragePort interface abstracts vault indexing and cache access*

- **Rationale:** Decouples query/indexing logic from storage implementation. MVP uses FileCache adapter; post-MVP can add BoltDB adapter without changing core. PRD Technical Assumptions explicitly require: "Storage must be implemented behind interface." → *(Epic 3, Story 3.1: "Define Storage interface")*

**3. Dependency Injection (Constructor-Based)**

*CLI adapter layer constructs concrete adapter instances and injects them into core components via constructors*

- **Rationale:** Core remains framework-agnostic and doesn't import adapter packages. Enables test seams—unit tests construct core components with mock adapters instead of production ones. No DI framework needed; Go's constructor pattern (`NewTemplateEngine(storage StoragePort, interactive InteractivePort)`) is sufficient for MVP scope. → *(Epic 1.1/4.1: Test harness uses injected mock adapters)*

**4. Builder Pattern (Schema Inheritance Resolution)**

*Schema system resolves inheritance chains (C extends B extends A) into flattened schemas*

- **Rationale:** Simplifies multi-level inheritance while detecting circular dependencies at load time (fail-fast). Immutable source schemas remain unchanged; builder creates resolved copies. → *(Epic 2, Stories 2.5-2.6: Multi-level inheritance + circular detection)*

**5. CQRS (Command Query Responsibility Segregation)**

*Separation of read and write operations for vault indexing and querying*

- **Rationale:** Vault indexing (write) and note querying (read) have fundamentally different optimization needs. Write operations optimize for batch processing, validation, and persistence - scans vault, parses frontmatter, builds complete index. Read operations optimize for fast O(1) lookups with in-memory indices and concurrent reads. The separation is in **operations** (dedicated logic for read vs write), not models (single Note model shared). This provides CQRS benefits (scalability, clear responsibilities, independent optimization paths) without model proliferation complexity. MVP uses single `.lithos/index/vault.json` file for persistence; post-MVP can migrate write side to append-only event store and read side to denormalized query database (e.g., BoltDB) without changing domain or interfaces. → *(Epic 3: Vault indexing requires different concerns than Epic 5: Template queries)*

*Note: The CQRS pattern future-proofs for larger vaults (>10k notes). Write side could batch and optimize filesystem scans, while read side maintains multiple specialized indices (by fileClass, by tag, by link graph) without impacting write performance.*

## Design Principles

**Dependency Inversion Principle (DIP):** High-level domain modules depend on abstractions (ports), not concrete adapters. Adapters import core packages and implement port interfaces; core never imports adapters. Enables independent evolution—replace Cobra with another CLI framework by swapping one adapter. Prevents Go import cycles (mandatory for clean hexagonal architecture).

**Lean Ports:** Ports have 2-4 methods representing service needs. Adapters handle complexity (file I/O, parsing, validation algorithms). Example: SchemaEnginePort has 2 methods (LoadSchemas, Validate), not 9. Prevents God Object ports.

**YAGNI Over Premature Optimization:** Single Note model for MVP (not separate read/write models). Config struct passed directly (no ConfigPort). Cache adapters use os package directly (no FileRead/WritePort abstraction). Add abstraction when compelling need emerges.

**ISP Compliance:** PromptPort (5 methods for prompts) separated from FuzzyFinderPort (2 methods for fuzzy finding). Template Service depends on PromptPort only. Prevents unused method dependencies.

**CQRS in Operations, Not Models:** Separate Command/Query services and ports for writes/reads. Both use single Note model. Provides CQRS benefits (independent optimization, scalability) without model proliferation complexity.

**Dependency Injection via Constructors:** CLI adapter constructs concrete types, injects via constructors. No DI framework—Go constructors sufficient. Config struct passed to components needing configuration.

**Ports Optimize for Common Case:** FilterByFrontmatter() for 80% of queries (frontmatter field equality), generic Filter() for remaining 20%. Enables secondary index optimization without interface bloat.
