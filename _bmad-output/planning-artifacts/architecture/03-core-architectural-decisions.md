---
title: "Core Architectural Decisions"
description: "Key architectural decisions and technology choices for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-15"
section: "Architecture Decisions"
---

# Core Architectural Decisions

## Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- **Workspace Shape:** `lithos-core` + `lithos-cli` single-core crate to enable zero-copy optimizations and reduce compilation overhead. (Proposal: `2026-01-30-rust-idiomatic-refactor`)
- **Storage Engine:** Redb + rkyv (zero-copy structured KV) with a concrete `Database` type (no traits). [ADR 006](docs/adr/006-persistence-cache-infrastructure.md)
- **Serialization Strategy:** Controlled serde allowance in domain (feature-gated). [ADR 003](docs/adr/003-domain-serialization.md)
- **Templating:** MiniJinja (Dynamic Jinja2). [ADR 007](docs/adr/007-template-engine.md)
- **Markdown Parser:** pulldown-cmark (event-streaming). [ADR 008](docs/adr/008-markdown-parsing.md)
- **Configuration:** Figment (provider-based hierarchy). [ADR 009](docs/adr/009-configuration-management.md)
- **Error Handling:** miette + thiserror (structured diagnostics with co-located errors). [ADR 005](docs/adr/005-error-handling.md)
- **Event Orchestration:** Minimal Foundation (Phase 1). [ADR 004](docs/adr/004-event-orchestration.md)

**Important Decisions (Shape Architecture):**

- **Identity:** UUID v7 (standardized sortable identifiers).
- **Execution Model:** Sync-first core, async only at CLI/LSP edges.
- **Module Structure:** Context modules use `<context>/mod.rs` pattern for organization.
- **CQRS Pattern:** Concrete CQRS generic over split storage ports (see implementation patterns).
- **Type-Driven Design:** Enforce invariants through type system, private fields by default, validation at construction.

**Deferred Decisions (Post-MVP):**

- **LSP Implementation details.**
- **Plugin architecture specifications.**

## Architecture Principles (Decision-Level)

- Sync-first core with async only at edges for CLI/LSP integration.
- Context isolation: business contexts (note, schema, template) do not import each other.
- Port-based CQRS with split query/command ports and generic CQRS types.
- Type-driven design: private fields by default, validation at construction, newtype wrappers for domain constraints.

## Data Architecture (High Level)

- **Engine:** Redb (pure-Rust, ACID KV) with rkyv zero-copy serialization.
- **Access Pattern:** Port-based CQRS with GAT-enabled zero-copy reads; adapters implement ports; type aliases hide generics; test fakes can substitute read/write ports.
- **Three-Shape Serialization Model (ADR 003 Appendix A):**
  - **`Raw*`:** Unvalidated input from filesystem (serde derives, tolerant parsing).
  - **Domain:** Validated entities with invariants, rkyv derives for zero-copy database operations.
  - **`Stored*`:** Optional storage-optimized representation, introduced only after profiling.
  - **Default Strategy:** Store domain types directly; keep conversions mechanical and co-located in storage adapters.
  - **Stability:** Treat `Stored*` changes as migration decisions (stable on-disk format).
- **Identity:** UUID v7 decouples identity from physical path to avoid the "directory trap."
- **References:**
  - [ADR 006: Persistence & Cache Infrastructure](../../docs/adr/006-persistence-cache-infrastructure.md)
  - [ADR 003: Domain Serialization Strategy](../../docs/adr/003-domain-serialization.md) (see Appendix A)
  - [Design Doc 012: CQRS Concrete Over Port](../../docs/design/012-cqrs-concrete-over-port.md)
  - Implementation detail: `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`

## Internal Communication

- **Strategy:** Minimal Event Foundation (Phase 1).
  - **Pattern:** Domain methods return `(Entity, Vec<Event>)` - pure functions, no side effects.
  - **Orchestration:** Application layer (CLI) collects events and dispatches synchronously.
  - **Handlers:** Simple synchronous functions for logging, tracing, basic reactions.
  - **Data Plane:** Direct `db` writes via CQRS commands. `db.batch_write()` handles atomicity.
  - **Control Plane:** Simple callbacks or deferred dispatch via `UnitOfWork` if needed.
  - **State Plane:** Deferred to LSP phase (async event bus, MPSC channels).
  - **Benefits:** Prevents god-object orchestrators while keeping Phase 1 simple.
- **ADR Reference:** [ADR 004: Event Orchestration](../../docs/adr/004-event-orchestration.md)

## Schema System Architecture

- **Initialization Lifecycle:** Schemas form a **Directed Acyclic Graph (DAG)** resolved at startup via topological sort.
  - **Phase 1 (Load):** Adapters load `RawSchema` definitions (unresolved).
  - **Phase 2 (Graph):** Domain `SchemaGraph` service validates acyclic lineage and determines resolution order.
  - **Phase 3 (Resolve):** Application layer drives `SchemaResolver` (Domain Service) to merge properties in order.
- **Resolution Strategy:** Separation of `RawSchema` (input) and `Schema` (resolved output).
  - **RawSchema:** Contains `extends`, `excludes`, and unresolved `$ref` pointers.
  - **Schema:** Contains only final, fully resolved `properties` list.
- **Reference Handling:** Format-specific adapters (JSON Pointer, TOML Path) parse references; Domain `PropertyBank` performs key lookups only.

## Technical Preferences (Step 4 Refinement)

- **Templating:** **MiniJinja**. Selected for "Mechanical Sympathy" - minimal dependencies and VM-based rendering for user-defined Markdown templates. [ADR 007](docs/adr/007-template-engine.md)
- **Markdown:** **pulldown-cmark**. Enables high-speed link extraction via event streaming without building expensive ASTs. [ADR 008](docs/adr/008-markdown-parsing.md)
- **Configuration:** **Figment**. Uses the Provider pattern to elegantly handle the 6-layer priority hierarchy. [ADR 009](docs/adr/009-configuration-management.md)
- **Errors/Diagnostics:** **miette**. Provides high-fidelity terminal snippets and 1:1 mapping to LSP Diagnostic objects. [ADR 005](docs/adr/005-error-handling.md)
