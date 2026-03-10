---
name: storage-pattern-architecture
status: accepted
date_proposed: 2026-03-10
date_decided: 2026-03-10
date_implemented: pending
stakeholders: [Jack (Architect), Development Team]
---

# ADR 002: Storage Pattern Architecture

## Context

Lithos is a local, file-based CLI application where the filesystem is the ultimate source of truth, and the local database is merely an expendable read-optimized projection/cache. State mutations occur externally (e.g., a user edits Markdown in Obsidian or Vim), and the domain is inherently tied to file I/O. The database can be wiped and rebuilt at any time.

We need to manage these file-based states while achieving specific architectural goals:
- **Testability:** The ability to mock I/O boundaries and unit test business logic without standing up a real database.
- **Zero-Copy Performance:** Achieving extreme read performance using `rkyv`'s `Archived<T>` without leaking transaction lifetimes in hot paths.
- **Simplicity & Safety:** Lean, functional pipelines that favor Rust's ownership and module systems to prevent god objects, rather than relying on complex architectural patterns.

## Decision

We will adopt a **Unified Storage Trait Architecture** utilizing functional composition.

1. **Unified Storage Traits**: Each domain module defines a single `Storage` trait abstracting all interactions with the database cache (e.g., `schema::Storage`).
2. **Module Boundaries Over Trait Boundaries**: Business logic is isolated using Rust's module system (`mod schema`, `mod note`) to enforce boundaries natively.
3. **Pipeline Orchestration**: External mutations (file changes) or internal actions (CLI commands) trigger functional Iterator-based pipelines (`parse() -> validate() -> project()`), coordinated by module `Loader`s.
4. **GAT-Based Zero-Copy**: The `Storage` trait uses Generic Associated Types (GATs) with closure-scoped access (`with_archived`) to maintain zero-copy read performance.

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

Operations are composed functionally within their modules using loader orchestrators:

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

### Alternative 1: Concrete Database Coupling (No Traits)

- **Description**: Direct calls to `redb` throughout the codebase without interface abstractions.
- **Pros**: Ultimate simplicity. No abstraction overhead.
- **Cons**: Impossible to unit test business logic without standing up a real temp-file database for every test. Ties domain logic intimately to a specific third-party crate.
- **Verdict**: Rejected. We must retain testability and separation of I/O.

### Alternative 2: Full Port-Based CQRS

- **Description**: Strict write vs. read separation through `CommandPort` and `QueryPort` traits, using command objects and event sourcing orchestration.
- **Pros**: Strict write vs. read separation; high purity in domain models; familiar to DDD practitioners.
- **Cons**: Heavyweight overkill for a local file-sync tool. It creates an impedance mismatch because state changes are driven by external file edits rather than traditional CQRS commands. It requires orchestrating events that have no distributed consumers, introducing severe boilerplate overhead.
- **Verdict**: Rejected. The structural ceremony is unnecessary for a local CLI application.

## Technical Validation

### Research Findings

A deep architectural review of successful Rust file-based tools (Cargo, mdBook, Zola, rust-analyzer) reveals they rely on:
- Iterator pipelines for file ingestion.
- Functional composition returning `Result<T, E>`.
- Module privacy for boundary enforcement.
- Simple trait abstractions strictly for I/O (like Cargo's `Source` trait or testing mocks).

### Benchmarks & Prototypes

The GAT + HRTB pattern (`impl for<'a> FnOnce(Self::Archived<'a>) -> R`) in a unified `Storage` trait maintains exact nanosecond-level read latency from `redb`, allowing zero-copy capabilities without needing complex read-model query traits.

## Consequences

- **Positive**:
  - **Minimal Boilerplate**: Simple functional traits without Command/Query struct proliferation.
  - **Idiomatic Rust**: Aligns with how standard Rust tools (like Cargo) are built, making it easier for Rust developers to contribute.
  - **Simpler Mental Model**: Developers just think in terms of "Read File -> Parse -> Validate -> Cache".
- **Negative**:
  - **Less Formal Audit Trail**: Debugging relies on standard logging rather than an immutable event log.
- **Risks**:
  - If the application unexpectedly pivots to a highly concurrent distributed cloud service, this architecture will lack the strict CQRS boundaries needed to scale writes independently of reads. (Mitigation: Unlikely given the core premise is local PKM).

## References

- [Core Architectural Decisions](../../_bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md)
- [ADR 003: Domain Serialization Strategy](./003-domain-serialization.md)
- Internal Research: `RESEARCH_RUST_ARCHITECTURE_PATTERNS.md`

## Appendix

### Historical Context: Why CQRS was initially implemented

Historically, Port-Based CQRS was originally implemented in this project to prevent "god-object orchestration" and to separate read and write models based on the developer's past experience building similar systems in Go and C#. However, as the system was implemented in Rust, it became clear that Rust's ownership rules, strict type system, and module boundaries natively prevent god-objects without requiring the structural ceremony of CQRS. Consequently, that architecture was superseded by this Unified Storage Trait approach.

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
