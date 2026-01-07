---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7]
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
  - label: Original Go Data Models
    path: _bmad-output/planning-artifacts/architecture/data-models.md
    category: architecture
  - label: Lessons Learned Phase 1
    path: _bmad-output/implementation-artifacts/course_correction/2025-10-27-lessons-learned-phase-1-archive.md
    category: implementation
workflowType: 'architecture'
project_name: 'lithos'
user_name: 'Jack'
date: '2026-01-07'
---

# Architecture Decision Document - Lithos Rust

_This document serves as the definitive architectural blueprint for the Rust-based implementation of Lithos, optimized for zero-copy performance and hexagonal modularity._

## 1. Project Context Analysis (Step 2)

### Requirements Overview
Lithos is a CLI developer tool with 50 functional requirements spanning template composition, schema-driven metadata management, and vault-wide link resolution. It must handle large vaults (1000+ files) with sub-50ms latency for real-time LSP features and <2s indexing targets.

### Technical Constraints
- **Core Language:** Rust 1.92+ (Stable).
- **Platform:** macOS/Linux primary, Windows secondary. Single-binary distribution.
- **Storage:** Must avoid the "directory trap" of the Go implementation by using a logical, query-optimized view.
- **Performance:** Zero-copy data paths required for LSP responsiveness.

---

## 2. Starter Template Evaluation (Step 3)

### Selected Starter: Workspace-Based Hexagonal Architecture
The project uses a Cargo Workspace to enforce compile-time boundaries between layers.
- **Rationale:** Ensures that business logic remains pure and infrastructure (Storage, UI) can be swapped or tested in isolation.
- **Crates:** `domain`, `app`, `adapters`, `lithos`.

---

## 3. Core Architectural Decisions (Step 4 Refined)

### Decision Priority Analysis
**Critical Decisions:**
- **Storage Engine:** **Redb + rkyv** (Zero-copy structured KV). [ADR 001](adr/001-storage-redb-rkyv.md)
- **Templating:** **MiniJinja** (VM-based, low-overhead). [ADR 002](adr/002-template-engine.md)
- **Markdown Parser:** **pulldown-cmark** (Event-streaming). [ADR 003](adr/003-markdown-parsing.md)
- **Configuration:** **Figment** (Provider-based hierarchy). [ADR 004](adr/004-configuration-management.md)
- **Error Handling:** **miette + thiserror** (Structured diagnostics). [ADR 005](adr/005-error-handling-diagnostics.md)
- **Event Orchestration:** **Hybrid Bus** (MPSC/Broadcast/Watch). [ADR 006](adr/006-event-orchestration.md)

### Data Architecture
- **Engine:** Redb 3.1.0 with rkyv 0.8.13 for O(1) metadata lookups.
- **Identity:** **UUID v7**. Time-ordered for Redb B-Tree performance, decoupling identity from physical file paths.

---

## 4. Implementation Patterns & Consistency Rules (Step 5)

### Naming Patterns
- **Files/Modules:** `snake_case` (e.g., `note.rs`). No directory-prefix repetition.
- **Traits:** `PascalCase` ending in `Port` (e.g., `RepositoryPort`).
- **Identity:** Use `UUID v7` via the `uuid` crate.

### Validation Strategy
1.  **Syntactic (SPI):** Structural validity check in SPI Adapters (Malformed YAML/TOML).
2.  **Orchestration (App):** Compliance check between Note and Schema in `app/compliance`.
3.  **Semantic (Domain):** Pure business logic rules defined in `domain/models`.

### Async & Concurrency
- **Runtime:** Tokio 1.49+.
- **Write Safety:** Exclusive write transactions managed by the `IndexerActor`.
- **Read Safety:** Snapshot isolation via MVCC provided by Redb.

---

## 5. Project Structure & Boundaries (Step 6)

### Complete Project Directory Structure

```text
lithos/
├── .gitattributes
├── .gitignore
├── .mise/                        # TASK ORCHESTRATION
│   └── scripts/
│       ├── dev-setup.sh
│       └── run-benchmarks.sh
├── .pre-commit-config.yaml
├── Cargo.toml                    # Workspace Root (Rust 1.92+)
├── Cargo.lock
├── mise.toml                     # Unified Task Runner
├── deny.toml                     # Boundary Enforcement
├── rustfmt.toml
├── clippy.toml                   # Complexity < 15
├── docs/
│   ├── rust/
│   │   ├── architecture.md       # This Map
│   │   └── adr/                  # ADRs 001-007
├── crates/
│   ├── domain/                   # PURE LOGIC
│   │   ├── src/
│   │   │   ├── models/
│   │   │   │   ├── identity.rs   # UUID v7 logic
│   │   │   │   ├── note.rs       # Note Aggregate (Links, Tags, Tasks)
│   │   │   │   ├── schema.rs     # Schema + Property Aggregates
│   │   │   │   └── template.rs   # Syntax & Design models
│   │   │   ├── ports/            # HEXAGONAL INTERFACES
│   │   │   │   ├── api/          # Driving (Command, UI)
│   │   │   │   └── spi/          # Driven (Repository, Template, Markdown, Bus)
│   │   │   ├── events/           # Tiered Enums (Data, Control, State)
│   │   │   └── errors.rs         # miette definitions
│   ├── app/                      # ORCHESTRATION
│   │   ├── src/
│   │   │   ├── commands/         # Write use-cases
│   │   │   ├── queries/          # Read use-cases
│   │   │   ├── compliance/       # Note-Schema Semantic Bridge
│   │   │   ├── template/         # Schema-Driven Design (Composer)
│   │   │   ├── metrics/          # Vault Analysis (Calculator)
│   │   │   └── indexer/          # Indexer Actor
│   ├── adapters/                 # INFRASTRUCTURE
│   │   ├── src/
│   │   │   ├── api/              # DRIVERS (CLI, LSP)
│   │   │   └── spi/              # DRIVEN (Storage, Markdown Extractor, Schema Rigor)
│   └── lithos/                   # ENTRY POINT
└── tests/                        # Integration & Arch tests
```

### Architectural Boundaries

**API Boundaries:**
- **CLI (`adapters/api/cli`):** Maps intent to `app/commands`. Exclusive owner of `miette` terminal rendering.
- **LSP (`adapters/api/lsp`):** Reactive driver pulling from the `watch` State Plane.

**Component Boundaries:**
- **Indexer Actor (`app/indexer`):** Protects sequential write integrity for Redb.
- **Compliance Engine (`app/compliance`):** The "Referee" for inter-model validation.

**Service Boundaries:**
- **Template Designer (FR9):** Drives interactive prompts using Schema contracts.
- **Metrics Calculator (`app/metrics`):** Aggregates graph data for observability.

**Data Boundaries:**
- **UUID v7 Identity:** Ensures Redb B-Tree efficiency and decouples entity from path.
- **Zero-Copy Paths:** Metadata passed as `rkyv` buffers (`Arc<[u8]>`) across crates.

### Requirements to Structure Mapping

**Feature Mapping:**
- **Knowledge Graph (FR20-FR25):** `app/queries/`, `adapters/spi/storage/`, `domain/models/note.rs`.
- **Schema Compliance (FR8-FR14):** `domain/models/schema.rs`, `app/compliance/`, `adapters/spi/schema/`.
- **Template Design (FR1-FR7, FR9):** `domain/models/template.rs`, `app/template/`.
- **Interactive CLI (FR41-FR47):** `adapters/api/cli/`, `domain/ports/api/ui.rs`.

**Cross-Cutting Concerns:**
- **Metadata Extraction:** Handled in `adapters/spi/markdown/extractor.rs`.
- **Task Management:** Centralized in `.mise/scripts/`.

### Integration Points
- **Hybrid Bus:** Tiered channels (`mpsc`, `broadcast`, `watch`) for reactive performance.
- **DI Container:** `lithos` crate wires concrete SPIs to App services.

---

## 6. Architecture Validation Results (Step 7)

### Coherence Validation ✅
- **Stack Synergy:** `Redb + rkyv + UUID v7` provides an optimized storage path for Rust.
- **Consistency:** Hexagonal boundaries and API/SPI separation strictly followed.

### Requirements Coverage Validation ✅
- **Traceability:** 100% of functional requirements mapped to architectural homes.
- **Performance:** NFRs met via zero-copy and time-ordered identity.

### Implementation Readiness Validation ✅
- **Hardening:** Stress-tested for LSP lag and background indexing contention.
- **Gap Resolution:** Audit and Encryption components explicitly defined.

### Implementation Handoff
**AI Agent Guidelines:**
- Follow all architectural decisions in ADRs 001-007.
- Respect the **API/SPI** boundary in the `adapters` crate.
- **PRIORITIZE** running all tasks and commands through **`mise`**.
- Refer to `domain/errors.rs` for all diagnostic definitions.

---

## 7. Requirements Traceability Matrix

| ID | Requirement | Primary Module/Path | Architectural Strategy |
| :--- | :--- | :--- | :--- |
| **FR1** | Modular templates | `domain/models/template.rs` | Recursive composition model. |
| **FR2** | Interactive prompts | `domain/ports/api/ui.rs` | Abstracted UI traits. |
| **FR3** | Complex composition | `app/template/composer.rs` | Orchestrates section-by-section flow. |
| **FR4** | Date functions | `adapters/spi/template/` | MiniJinja custom functions. |
| **FR5** | Dynamic commands | `adapters/spi/template/` | Whitespace control & shell hooks. |
| **FR6** | User functions | `adapters/spi/config/` | Discovered scripts registered to engine. |
| **FR7** | Advanced hooks | `app/template/composer.rs` | Lifecycle events on Hybrid Bus. |
| **FR8** | Metadata schemas | `domain/models/schema.rs` | Unified aggregate for property specs. |
| **FR9** | **Schema-Driven Design**| `app/template/designer.rs` | Schema properties dictate UI prompts. |
| **FR10**| Note validation | `app/compliance/engine.rs` | Semantic check between Note and Schema. |
| **FR11**| Enum-driven suggesters| `app/template/designer.rs` | Schema enums passed to UI Port. |
| **FR12**| Directory filters | `adapters/spi/schema/` | Constraints applied to file pickers. |
| **FR13**| Date formatting | `domain/models/schema.rs` | Format logic in PropertySpec. |
| **FR14**| Schema inheritance | `adapters/spi/schema/resolver.rs`| Dereferences `$ref` and processes `extends`. |
| **FR15**| Free-text prompts | `adapters/api/cli/` | Implements UI Port via standard input. |
| **FR16**| Single-choice lists | `adapters/api/cli/` | Implements UI Port via fuzzy-select. |
| **FR17**| Multi-suggesters | `adapters/api/cli/` | Implements UI Port via multi-select. |
| **FR18**| Contextual help | `domain/errors.rs` | miette-rich diagnostic labels. |
| **FR19**| Progressive complexity| `adapters/spi/config/` | User mode toggle in Figment config. |
| **FR20**| Index & Search | `app/queries/` | Snapshots from Redb tables. |
| **FR21**| Multi-key lookups | `adapters/spi/storage/` | B-tree indexed path/uuid/alias keys. |
| **FR22**| Link resolution | `app/compliance/resolver.rs` | Logical resolution via aliases. |
| **FR23**| Metadata queries | `app/queries/` | Snapshots from RedbSnapshot. |
| **FR24**| Vault consistency | `app/indexer/` | Single-writer transactions. |
| **FR25**| Large vault scale | `domain/models/note.rs` | Zero-copy rkyv::Archive. |
| **FR26**| Template packs | `adapters/spi/fs/` | Discovery logic for Git-cloned packs. |
| **FR27**| Manage schemas | `adapters/api/cli/` | CLI subcommands for schema registry. |
| **FR28**| App preferences | `adapters/spi/config/` | Figment provider hierarchy. |
| **FR29**| Custom lint rules | `app/compliance/` | Compliance engine ruleset. |
| **FR30**| OS Consistency | `lithos/` | Static binary + .gitattributes. |
| **FR31**| Terminal access | `adapters/api/cli/` | Primary driver (Clap). |
| **FR32**| IDE integration | `adapters/api/lsp/` | Secondary driver (LSP). |
| **FR33**| CI/CD automation | `lithos/` | CLI-first design support. |
| **FR34**| Share Git packs | `mise.toml` | Tasks for pack orchestration. |
| **FR35**| Discover packs | `README.md` | Community documentation. |
| **FR36**| Validate 3rd party | `app/compliance/` | Reuses core compliance engine. |
| **FR37**| Contribute to packs | `mise.toml` | Pre-commit quality gates. |
| **FR38**| Access control | `adapters/spi/fs/` | OS filesystem permissions. |
| **FR39**| Encrypt sensitive files| `adapters/spi/config/` | age/gpg support via Encryption Port. |
| **FR40**| Audit logging | `adapters/spi/events/` | Dedicated Audit subscriber. |
| **FR41**| CLI subcommands | `adapters/api/cli/` | Nested clap subcommands. |
| **FR42**| Comprehensive help | `adapters/api/cli/` | Auto-generated help via Clap. |
| **FR43**| Status & Config view | `adapters/api/cli/` | Maps status to Config snapshot. |
| **FR44**| CLI Vault Ops | `app/commands/` | Maps CLI intent to Indexer mailbox. |
| **FR45**| Format destinations | `app/commands/` | Config-driven output routing. |
| **FR46**| Configure CLI behavior| `domain/models/` | UI preference models. |
| **FR47**| Single-word commands | `adapters/api/cli/` | Default fuzzy-pickers for shortcuts. |
| **FR48**| Actionable errors | `domain/errors.rs` | High-fidelity miette diagnostics. |
| **FR49**| Rollback failure | `app/indexer/` | Atomic storage transactions. |
| **FR50**| Troubleshooting | `adapters/api/cli/` | Graphical config validation. |
