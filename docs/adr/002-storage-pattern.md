---
name: storage-pattern-architecture
status: accepted
supersedes: [0002] # Replaces the previous Port-Based CQRS decision
date_proposed: 2026-03-10
date_decided: 2026-03-10
date_implemented: pending
stakeholders: [Jack (Architect), Development Team]
---

# ADR 002: Storage Pattern Architecture

## Context

Lithos is a local, file-based CLI application where the filesystem is the ultimate source of truth, and the local database is merely an expendable read-optimized projection/cache.

Previously, the architecture mandated a "Port-Based CQRS" pattern, enforcing a rigid separation into `CommandPort` and `QueryPort` traits alongside event orchestration. While CQRS is powerful for distributed event-sourced systems where the database is authoritative, research into idiomatic Rust file-based projects (like Zola, mdBook, and Cargo) revealed that CQRS is heavyweight overkill for a tool where:

1. State mutations occur externally (user edits Markdown in Obsidian/Vim).
2. The domain is inherently tied to file I/O.
3. The database can be wiped and rebuilt at any time.

Applying traditional Hexagonal CQRS to a file-sync tool created unnecessary complexity: excessive boilerplate, misaligned mental models (file writes aren't traditional CQRS commands), and an impedance mismatch with idiomatic Rust's functional composition.

We need a pattern that supports:

- **Testability:** The ability to mock I/O boundaries without standing up a database.
- **Zero-Copy Performance:** Using `rkyv`'s `Archived<T>` without leaking transaction lifetimes in hot paths.
- **Simplicity:** Lean, functional pipelines that favor Rust's ownership and module systems over complex architectural patterns.

## Decision

We will adopt a **Unified Storage Trait Architecture** with functional composition, abandoning CQRS and explicit Command/Query separation for business logic.

1. **Unified Storage Traits**: Instead of split Command/Query ports, each domain module will define a single `Storage` trait abstracting all interactions with the database cache (e.g., `schema::Storage`).
2. **Module Boundaries Over Trait Boundaries**: Business logic will be isolated using Rust's module system (`mod schema`, `mod note`) rather than interface traits.
3. **Pipeline Orchestration**: External mutations (file changes) or internal actions (CLI commands) will trigger functional Iterator-based pipelines (`parse() -> validate() -> project()`), coordinated by module `Loader`s.
4. **GAT-Based Zero-Copy**: The `Storage` trait will continue to use Generic Associated Types (GATs) with closure-scoped access (`with_archived`) to maintain extreme zero-copy read performance.

### 1. Unified Storage Trait

Defined in `<context>/storage.rs`:

```rust
pub trait Storage {
    type Error: std::error::Error;
    type Archived<'a> where Self: 'a;  // GAT for zero-copy

    // Reads
    fn get(&self, id: &SchemaId) -> Result<Option<Schema>, Self::Error>;
    fn list(&self) -> Result<Vec<Schema>, Self::Error>;

    // Zero-copy hot path
    fn with_archived<R>(
        &self,
        id: &SchemaId,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;

    // Writes (Projection updates)
    fn save(&mut self, schema: Schema) -> Result<SchemaId, Self::Error>;
    fn delete(&mut self, id: &SchemaId) -> Result<bool, Self::Error>;
}
```

### 2. Module-Based Functional Pipelines

Instead of Command Objects and Command Handlers, operations are composed functionally within their modules:

```rust
// In schema/loader.rs
pub fn load_schema_file(
    path: &Path,
    storage: &mut impl Storage,
) -> Result<SchemaId, Error> {
    // 1. File I/O (can be abstracted via a simple fs trait if needed)
    let content = std::fs::read_to_string(path)?;

    // 2. Parse (dumb data)
    let raw = toml::from_str::<RawSchema>(&content)?;

    // 3. Validate (domain invariants)
    let schema = Schema::try_from(raw)?;

    // 4. Project to cache
    storage.save(schema)
}
```

## Alternatives Considered

### Alternative 1: Full Port-Based CQRS (Previous Architecture)

- **Pros**: Strict write vs. read separation; high purity in domain models; familiar to DDD practitioners.
- **Cons**: Severe boilerplate overhead. "Commands" conceptually clashed with the reality that users were directly editing files outside the system. Requires orchestrating events that have no distributed consumers.
- **Verdict**: Rejected. Too heavyweight for a local CLI/LSP application.

### Alternative 2: Concrete Database Coupling (No Traits)

- **Pros**: Ultimate simplicity. Direct calls to `Redb` throughout the codebase.
- **Cons**: Impossible to unit test business logic without standing up a real temp-file database for every test. Ties domain logic intimately to a specific third-party crate.
- **Verdict**: Rejected. We must retain testability and separation of I/O.

## Technical Validation

### Research Findings

A deep architectural review of successful Rust file-based tools (Cargo, mdBook, Zola, rust-analyzer) revealed that none of them use CQRS. Instead, they universally rely on:

- Iterator pipelines for file ingestion.
- Functional composition returning `Result<T, E>` (rather than emitting domain events).
- Module privacy for boundary enforcement.
- Simple trait abstractions strictly for I/O (like Cargo's `Source` trait or testing mocks).

### Benchmarks & Prototypes

The GAT + HRTB pattern (`impl for<'a> FnOnce(Self::Archived<'a>) -> R`) used previously in `QueryPort` ports perfectly identically to the new unified `Storage` trait, maintaining the exact same nanosecond-level read latency from Redb.

## Consequences

- **Positive**:
  - **Drastically Reduced Boilerplate**: Eliminates Command structs, Query structs, port splitting, and event dispatchers.
  - **Idiomatic Rust**: Aligns with how standard Rust tools (like Cargo) are built, making it easier for Rust developers to contribute.
  - **Simpler Mental Model**: Developers just think in terms of "Read File -> Parse -> Validate -> Cache".
- **Negative**:
  - **Less Formal Audit Trail**: Without explicit Command objects and Event sourcing, debugging relies on standard logging rather than an immutable event log.
- **Risks**:
  - If the application unexpectedly pivots to a highly concurrent distributed cloud service, this architecture will lack the strict CQRS boundaries needed to scale writes independently of reads. (Mitigation: Unlikely given the core premise is local PKM).

## References

- [Core Architectural Decisions](../../_bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md)
- [ADR 003: Domain Serialization Strategy](./003-domain-serialization.md)
- Internal Research: `RESEARCH_RUST_ARCHITECTURE_PATTERNS.md`

## Appendix

### Historical Context: Why CQRS was initially chosen

CQRS was originally selected to prevent "god-object orchestration" and to separate read and write models based on the developer's past experience building similar systems in Go and C#. However, as the system was implemented in Rust, it became clear that Rust's ownership rules, strict type system, and module boundaries natively prevent god-objects without requiring the structural ceremony of CQRS.

### Storage Adapter Implementation Pattern

Adapters implement the trait and are scoped to their context (e.g., `schema/adapters/storage.rs`). They map the domain `Schema` to the `SchemaView` (the database projection).

```rust
pub struct RedbSchemaStorage<'db> {
    db: &'db Database,
}

impl schema::Storage for RedbSchemaStorage<'_> {
    type Error = DbError;
    type Archived<'a> = &'a ArchivedSchemaView;

    fn get(&self, id: &SchemaId) -> Result<Option<Schema>, DbError> {
        // Redb lookup and conversion from View to Domain
    }
    // ...
}
```
