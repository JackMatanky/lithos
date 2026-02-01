---
title: "Core Architectural Decisions"
description: "Key architectural decisions and technology choices for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Architecture Decisions"
---

# Core Architectural Decisions

## Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- **Single-Crate Architecture:** Pivot from multi-crate workspace to `lithos-core` + `lithos-cli` to enable zero-copy optimizations and reduce compilation overhead. (Proposal: `2026-01-30-rust-idiomatic-refactor`)
- **Storage Engine:** Redb + rkyv (Zero-copy structured KV) with a concrete `Database` type (no traits). [ADR 0002](docs/adr/0002-persistence-cache-infrastructure.md)
- **Serialization Strategy:** Controlled serde allowance in domain (feature-gated). [ADR 0009](docs/adr/0009-domain-serialization-strategy.md)
- **Templating:** MiniJinja (Dynamic Jinja2). [ADR 0003](docs/adr/0003-template-engine.md)
- **Markdown Parser:** pulldown-cmark (Event-streaming). [ADR 0004](docs/adr/0004-markdown-parsing.md)
- **Configuration:** Figment (Provider-based hierarchy). [ADR 0005](docs/adr/0005-configuration-management.md)
- **Error Handling:** miette + thiserror (Structured diagnostics with co-located errors). [ADR 0006](docs/adr/0006-error-handling-diagnostics.md)
- **Event Orchestration:** Minimal Foundation (Phase 1). [ADR 0007](docs/adr/0007-event-orchestration.md)

**Important Decisions (Shape Architecture):**

- **Workspace:** Single Core Crate (`lithos-core`) to maximize compiler optimizations.
- **Identity:** UUID v7 (Standardized sortable identifiers).
- **Execution Model:** Sync-First Core, Async only at CLI/LSP edges.
- **Module Structure:** File-First Modules (`<module>.rs` + `<module>/`), NO `mod.rs`.

**Deferred Decisions (Post-MVP):**

- **LSP Implementation details.**
- **Plugin architecture specifications.**

## Data Architecture

- **Engine:** Redb (Pure-Rust, ACID KV) with **rkyv** zero-copy serialization.
- **Access Pattern:** Concrete `Database` struct in `lithos-core/db.rs` exposing zero-copy primitives (`get_archived`, `put_reserve`). No repository traits for MVP.
- **Identity:** UUID v7. Decouples identity from physical path to avoid the "directory trap."
- **ADR Reference:** [ADR 0002: Persistence & Cache Infrastructure](docs/adr/0002-persistence-cache-infrastructure.md)

## Internal Communication

- **Strategy:** **Minimal Event Foundation**.
  - **Data Plane:** Direct `db` writes for Phase 1. `db.batch_write()` handles atomicity.
  - **Control Plane:** Simple callbacks or deferred dispatch via `UnitOfWork` if needed.
  - **State Plane:** Deferred to LSP phase.
- **ADR Reference:** [ADR 0007: Event Orchestration](docs/adr/0007-event-orchestration.md)

## Schema System Architecture

- **Initialization Lifecycle:** Schemas form a **Directed Acyclic Graph (DAG)** resolved at startup via topological sort.
  - **Phase 1 (Load):** Adapters load `RawSchema` definitions (unresolved).
  - **Phase 2 (Graph):** Domain `SchemaGraph` service validates acyclic lineage and determines resolution order.
  - **Phase 3 (Resolve):** Application layer drives `SchemaResolver` (Domain Service) to merge properties in order.
- **Resolution Strategy:** Separation of `RawSchema` (Input) and `Schema` (Resolved Output).
  - **RawSchema:** Contains `extends`, `excludes`, and unresolved `$ref` pointers.
  - **Schema:** Contains only final, fully resolved `properties` list.
- **Reference Handling:** Format-specific adapters (JSON Pointer, TOML Path) parse references; Domain `PropertyBank` performs key lookups only.

## Technical Preferences (Step 4 Refinement)

- **Templating:** **MiniJinja**. Selected for "Mechanical Sympathy"—minimal dependencies and VM-based rendering for user-defined Markdown templates. [ADR 0003](docs/adr/0003-template-engine.md)
- **Markdown:** **pulldown-cmark**. Enables high-speed link extraction via event streaming without building expensive ASTs. [ADR 0004](docs/adr/0004-markdown-parsing.md)
- **Configuration:** **Figment**. Uses the Provider pattern to elegantly handle the 6-layer priority hierarchy. [ADR 0005](docs/adr/0005-configuration-management.md)
- **Errors/Diagnostics:** **miette**. Provides high-fidelity terminal snippets and 1:1 mapping to LSP Diagnostic objects. [ADR 0006](docs/adr/0006-error-handling-diagnostics.md)
