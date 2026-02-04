---
title: "Core Architectural Decisions"
description: "Key architectural decisions and technology choices for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-04"
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
- **Module Structure:** Context modules use `<context>/mod.rs` pattern for organization.
- **CQRS Pattern:** Concrete CQRS Generic Over Storage Port (see below).
- **Type-Driven Design:** Enforce invariants through type system, private fields by default, validation at construction. See [Implementation Patterns](./implementation-patterns-consistency-rules.md#type-driven-design-patterns).

**Deferred Decisions (Post-MVP):**

- **LSP Implementation details.**
- **Plugin architecture specifications.**

## Data Architecture

- **Engine:** Redb (Pure-Rust, ACID KV) with **rkyv** zero-copy serialization.
- **Access Pattern:** Port-based CQRS with GAT-enabled zero-copy reads
  - Each context defines `<Context>Store` port trait (e.g., `SchemaStore`, `NoteStore`)
  - Concrete CQRS types generic over port: `Query<S: SchemaStore>`, `Command<S: SchemaStore>`
  - Default adapter: `RedbSchemaStore<'db>` implements port with zero-copy primitives
  - Type aliases hide generic complexity: `RedbSchemaQuery<'db> = Query<RedbSchemaStore<'db>>`
  - Enables test substitution via `FakeSchemaStore` while maintaining zero-copy performance
- **Storage DTOs:** Following ADR 0009 Appendix A, introduce `Stored*` types selectively:
  - One per persisted aggregate (StoredNote, StoredSchema, StoredTemplate, StoredConfig)
  - Keep conversions mechanical and co-located in storage adapters
  - Treat changes to `Stored*` as migration decisions (stable on-disk format)
  - Domain types remain ergonomic (no rkyv derives in domain surface)
- **Identity:** UUID v7. Decouples identity from physical path to avoid the "directory trap."
- **ADR References:**
  - [ADR 0002: Persistence & Cache Infrastructure](../../docs/adr/0002-persistence-cache-infrastructure.md)
  - [ADR 0009: Domain Serialization Strategy](../../docs/adr/0009-domain-serialization-strategy.md) (see Appendix A)
  - [Design Doc 012: CQRS Concrete Over Port](../../docs/design/012-cqrs-concrete-over-port.md)

## Internal Communication

- **Strategy:** **Minimal Event Foundation** (Phase 1).
  - **Pattern:** Domain methods return `(Entity, Vec<Event>)` - pure functions, no side effects
  - **Orchestration:** Application layer (CLI) collects events and dispatches synchronously
  - **Handlers:** Simple synchronous functions for logging, tracing, basic reactions
  - **Data Plane:** Direct `db` writes via CQRS commands. `db.batch_write()` handles atomicity.
  - **Control Plane:** Simple callbacks or deferred dispatch via `UnitOfWork` if needed.
  - **State Plane:** Deferred to LSP phase (async event bus, MPSC channels).
  - **Benefits:** Prevents god-object orchestrators while keeping Phase 1 simple
- **ADR Reference:** [ADR 0007: Event Orchestration](../../docs/adr/0007-event-orchestration.md)

## CQRS Architecture Pattern

**Pattern:** Concrete CQRS Generic Over Storage Port

Each bounded context (note, schema, template, config) implements CQRS using:

### Port Trait Pattern

Storage capabilities defined via trait with GATs (Generic Associated Types) for zero-copy:

```rust
pub trait SchemaStore {
    type Error;
    type Archived<'a> where Self: 'a;  // GAT for zero-copy archived view

    // Cold tier: owned read
    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, Self::Error>;

    // Hot tier: zero-copy read with closure
    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;

    // Write operations
    fn save(&self, schema: &Schema) -> Result<(), Self::Error>;
}
```

### CQRS Layer Pattern

Concrete types generic over storage port:

```rust
pub struct Query<S> {
    store: S,
}

pub struct Command<S> {
    store: S,
}

impl<S: SchemaStore> Query<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, QueryError<S::Error>>
    {
        self.store.find_owned_by_name(name)
            .map_err(QueryError::Storage)
    }

    // Hot path helper
    pub fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(S::Archived<'a>) -> R,
    ) -> Result<Option<R>, QueryError<S::Error>>
    {
        self.store.with_archived_by_name(name, f)
            .map_err(QueryError::Storage)
    }
}
```

### Type Aliases for Ergonomics

Hide generic complexity from callers:

```rust
// In context/mod.rs
pub type RedbSchemaQuery<'db> = Query<RedbSchemaStore<'db>>;
pub type RedbSchemaCommand<'db> = Command<RedbSchemaStore<'db>>;

impl<'db> RedbSchemaQuery<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(RedbSchemaStore::new(db))
    }
}
```

### Benefits

- **Decoupling**: CQRS layer independent of concrete database implementation
- **Zero-Copy Performance**: GATs enable `Archived<'a>` without leaking transaction lifetimes
- **Testability**: Can substitute `FakeSchemaStore` implementing same port
- **Static Dispatch**: Performance benefits when using concrete type aliases
- **Future-Proof**: Can change storage backend by implementing new adapter

**Reference:** [Design Doc 012: CQRS Concrete Over Port](../../docs/design/012-cqrs-concrete-over-port.md)

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
