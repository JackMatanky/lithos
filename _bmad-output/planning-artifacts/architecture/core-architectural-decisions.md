---
title: "Core Architectural Decisions"
description: "Key architectural decisions and technology choices for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-05"
section: "Architecture Decisions"
---

# Core Architectural Decisions

## Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- **Single-Crate Architecture:** Pivot from multi-crate workspace to `lithos-core` + `lithos-cli` to enable zero-copy optimizations and reduce compilation overhead. (Proposal: `2026-01-30-rust-idiomatic-refactor`)
- **Storage Engine:** Redb + rkyv (Zero-copy structured KV) with a concrete `Database` type (no traits). [ADR 006](docs/adr/006-persistence-cache-infrastructure.md)
- **Serialization Strategy:** Controlled serde allowance in domain (feature-gated). [ADR 003](docs/adr/003-domain-serialization.md)
- **Templating:** MiniJinja (Dynamic Jinja2). [ADR 007](docs/adr/007-template-engine.md)
- **Markdown Parser:** pulldown-cmark (Event-streaming). [ADR 008](docs/adr/008-markdown-parsing.md)
- **Configuration:** Figment (Provider-based hierarchy). [ADR 009](docs/adr/009-configuration-management.md)
- **Error Handling:** miette + thiserror (Structured diagnostics with co-located errors). [ADR 005](docs/adr/005-error-handling.md)
- **Event Orchestration:** Minimal Foundation (Phase 1). [ADR 004](docs/adr/004-event-orchestration.md)

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
  - Each context defines **split storage ports**: `<Context>::ports::Query` and `<Context>::ports::Command` (e.g., `schema::ports::Query`, `schema::ports::Command`)
  - Concrete CQRS types generic over respective ports: `Query<Q: SchemaQueryPort>`, `Command<C: SchemaCommandPort>`
  - Default adapters: `RedbSchemaQueryAdapter<'db>` and `RedbSchemaCommandAdapter<'db>` implement ports with zero-copy primitives
  - Type aliases hide generic complexity: `RedbSchemaQuery<'db> = Query<RedbSchemaQueryAdapter<'db>>`
  - Enables test substitution via `FakeSchemaQueryPort` while maintaining zero-copy performance
  - **Port Split Benefits:** Read-only test fakes don't implement writes, prevents interface bloat, enables future flexibility (cache reads, DB writes)
- **Three-Shape Serialization Model:** Following ADR 003 Appendix A pattern:
  - **`Raw*` (serde derives):** Unvalidated input from filesystem (YAML/JSON), tolerant parsing with nullable fields for better error messages
  - **Domain (rkyv + serde feature-gated):** Validated entities with invariants, **has rkyv derives** for zero-copy database operations, used throughout application
  - **`Stored*` (rkyv derives, optional):** Storage-optimized representation, only created when domain shape inefficient (wrapper newtypes, deep nesting, Arc sharing issues)
  - **Default Strategy:** Store domain types directly (they already have rkyv derives); only introduce `Stored*` when performance profiling reveals inefficiency
  - Keep conversions mechanical and co-located in storage adapters
  - Treat changes to `Stored*` as migration decisions (stable on-disk format)
- **Identity:** UUID v7. Decouples identity from physical path to avoid the "directory trap."
- **ADR References:**
  - [ADR 006: Persistence & Cache Infrastructure](../../docs/adr/006-persistence-cache-infrastructure.md)
  - [ADR 003: Domain Serialization Strategy](../../docs/adr/003-domain-serialization.md) (see Appendix A for three-shape model)
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
- **ADR Reference:** [ADR 004: Event Orchestration](../../docs/adr/004-event-orchestration.md)

## CQRS Architecture Pattern

**Pattern:** Concrete CQRS Generic Over Split Storage Ports

Each bounded context (note, schema, template, config) implements CQRS using split query and command ports to prevent interface bloat and enable independent read/write backends.

### Split Port Trait Pattern

Storage capabilities defined via **separate traits** with GATs (Generic Associated Types) for zero-copy:

#### Query Port Pattern

```rust
// Defined in <context>/ports.rs
pub trait Query {
    type Error: std::error::Error;
    type Archived<'a> where Self: 'a;  // GAT for zero-copy archived view

    // COLD TIER: Owned reads for mutations/complex operations
    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, Self::Error>;

    fn list_all_owned(&self) -> Result<Vec<Schema>, Self::Error>;

    // HOT TIER: Zero-copy closure-scoped reads (LSP hot path)
    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;
}
```

#### Command Port Pattern

```rust
// Defined in <context>/ports.rs
pub trait Command {
    type Error: std::error::Error;

    fn save(&self, schema: &Schema) -> Result<(), Self::Error>;
    fn delete(&self, name: &SchemaName) -> Result<bool, Self::Error>;
    fn batch_save(&self, schemas: &[Schema]) -> Result<(), Self::Error>;
}
```

**Port Organization:** Both traits defined in single `<context>/ports.rs` file (Rust's preferred flatter structure).

### CQRS Layer Pattern

Concrete types generic over respective storage ports:

```rust
// In <context>/query.rs
pub struct Query<Q> {
    query_port: Q,
}

impl<Q: SchemaQueryPort> Query<Q> {
    pub fn new(query_port: Q) -> Self {
        Self { query_port }
    }

    pub fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, QueryError<Q::Error>>
    {
        self.query_port.find_owned_by_name(name)
            .map_err(QueryError::Storage)
    }

    // Hot path helper
    pub fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Q::Archived<'a>) -> R,
    ) -> Result<Option<R>, QueryError<Q::Error>>
    {
        self.query_port.with_archived_by_name(name, f)
            .map_err(QueryError::Storage)
    }
}

// In <context>/command.rs
pub struct Command<C> {
    command_port: C,
}

impl<C: SchemaCommandPort> Command<C> {
    pub fn new(command_port: C) -> Self {
        Self { command_port }
    }

    pub fn save(&self, schema: &Schema)
        -> Result<(), CommandError<C::Error>>
    {
        self.command_port.save(schema)
            .map_err(CommandError::Storage)
    }
}
```

### Adapter Implementation Pattern

Adapters live in `db/<context>_adapter.rs` and implement port traits:

```rust
// In db/schema_adapter.rs
pub struct RedbSchemaQueryAdapter<'db> {
    db: &'db Database,
}

impl SchemaQueryPort for RedbSchemaQueryAdapter<'_> {
    type Error = DbError;
    type Archived<'a> = &'a ArchivedSchema;  // Domain type directly, or ArchivedStoredSchema if Stored* exists

    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, DbError>
    {
        // Default: Store domain type directly (has rkyv derives)
        self.db.get_owned::<Schema>("schemas", name.as_ref())

        // Optional: If Stored* exists for optimization
        // let stored: Option<StoredSchema> = self.db.get_owned("schemas", name.as_ref())?;
        // Ok(stored.map(Schema::from))
    }

    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(&'a ArchivedSchema) -> R,
    ) -> Result<Option<R>, DbError> {
        self.db.get::<Schema, _, _>("schemas", name.as_ref(), f)
    }
}

pub struct RedbSchemaCommandAdapter<'db> {
    db: &'db Database,
}

impl SchemaCommandPort for RedbSchemaCommandAdapter<'_> {
    type Error = DbError;

    fn save(&self, schema: &Schema) -> Result<(), DbError> {
        // Default: Store domain type directly
        self.db.put("schemas", schema.name().as_ref(), schema)

        // Optional: If Stored* exists for optimization
        // let stored = StoredSchema::from(schema);
        // self.db.put("schemas", schema.name().as_ref(), &stored)
    }
}
```

### Type Aliases for Ergonomics

Hide generic complexity from callers:

```rust
// In <context>/mod.rs
pub type RedbSchemaQuery<'db> = Query<RedbSchemaQueryAdapter<'db>>;
pub type RedbSchemaCommand<'db> = Command<RedbSchemaCommandAdapter<'db>>;

impl<'db> RedbSchemaQuery<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(RedbSchemaQueryAdapter::new(db))
    }
}

impl<'db> RedbSchemaCommand<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(RedbSchemaCommandAdapter::new(db))
    }
}
```

### Benefits

- **Decoupling**: CQRS layer independent of concrete database implementation
- **Zero-Copy Performance**: GATs enable `Archived<'a>` without leaking transaction lifetimes
- **Testability**: Can substitute `FakeSchemaQueryPort` or `FakeSchemaCommandPort` implementing respective ports
- **Interface Segregation**: Read-only test fakes don't implement write operations
- **Static Dispatch**: Performance benefits when using concrete type aliases (monomorphization)
- **Future-Proof**: Can change storage backend by implementing new adapters, or use different backends for reads vs writes
- **Lean Ports**: Each port trait contains only methods relevant to its responsibility

### Three-Shape Serialization Flow

```
┌─────────────────────────────────────────┐
│ File System (YAML/JSON)                 │
│ - User-editable vault files             │
└─────────────────┬───────────────────────┘
                  │
                  ▼ parse (serde)
┌─────────────────────────────────────────┐
│ Raw* (serde derives)                    │
│ - Unvalidated input representation      │
│ - Location: <context>/raw.rs            │
│ - Nullable fields for better errors     │
└─────────────────┬───────────────────────┘
                  │
                  ▼ validate & compile
┌─────────────────────────────────────────┐
│ Domain (rkyv + serde feature-gated)     │
│ - Validated, invariant-preserving       │
│ - Location: <context>/aggregate.rs      │
│ - Used throughout application           │
│ - Has rkyv derives for zero-copy DB     │
└─────────────────┬───────────────────────┘
                  │
                  ▼ project/adapt (optional, only when needed)
┌─────────────────────────────────────────┐
│ Stored* (rkyv derives, optional)        │
│ - Storage-optimized representation      │
│ - Location: db/stored/<context>.rs      │
│ - Only when domain shape inefficient    │
└─────────────────┬───────────────────────┘
                  │
                  ▼ serialize (rkyv)
┌─────────────────────────────────────────┐
│ Database (redb)                         │
│ - Zero-copy archived access             │
└─────────────────────────────────────────┘
```

**When to Create `Stored*` Types:**

Only introduce `Stored*` when profiling reveals:
- Wrapper newtypes (SchemaName) complicate database indexing
- Deep nesting causes excessive alignment copy overhead
- `Arc<T>` sharing doesn't serialize efficiently
- Storage layout differs significantly from domain representation

**Default Strategy:** Store domain types directly (they already have rkyv derives for zero-copy).

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

- **Templating:** **MiniJinja**. Selected for "Mechanical Sympathy"—minimal dependencies and VM-based rendering for user-defined Markdown templates. [ADR 007](docs/adr/007-template-engine.md)
- **Markdown:** **pulldown-cmark**. Enables high-speed link extraction via event streaming without building expensive ASTs. [ADR 008](docs/adr/008-markdown-parsing.md)
- **Configuration:** **Figment**. Uses the Provider pattern to elegantly handle the 6-layer priority hierarchy. [ADR 009](docs/adr/009-configuration-management.md)
- **Errors/Diagnostics:** **miette**. Provides high-fidelity terminal snippets and 1:1 mapping to LSP Diagnostic objects. [ADR 005](docs/adr/005-error-handling.md)
