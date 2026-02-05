---
name: port-based-cqrs-architecture-with-split-ports
status: accepted
supersedes: []
date_proposed: 2026-02-05
date_decided: 2026-02-05
date_implemented: pending
date_updated: 2026-02-05
stakeholders: [Jack (Architect), Development Team]
---

# ADR 002: Port-Based CQRS Architecture with Split Ports

## Context

The Lithos architecture requires a clean separation between business logic and infrastructure while maintaining high performance (zero-copy reads). Traditional "Hexagonal Architecture" often implies:
1. Object-safe trait objects (`Box<dyn Store>`) which kill zero-copy performance and generic flexibility.
2. Single "Store" interfaces that violate the Interface Segregation Principle (read-only components forced to depend on write methods).
3. Ambiguity about where "Storage DTOs" live vs Domain entities.

We need a pattern that supports:
- **Zero-Copy Performance**: Using rkyv's `Archived<T>` without leaking transaction lifetimes.
- **Testability**: Enabling read-only test fakes without implementing dummy write methods.
- **Future Flexibility**: Allowing different backends for reads (e.g., cache) and writes (e.g., durable DB).
- **Interface Segregation**: Clients should only depend on what they use.

## Decision

We will adopt a **Port-Based CQRS Architecture** with the following key characteristics:

1. **Split Storage Ports**: Instead of a single `Store` trait, each context defines separate `QueryPort` and `CommandPort` traits.
2. **Generic CQRS Types**: `Query<Q>` and `Command<C>` structs are generic over these ports, enabling static dispatch (monomorphization) and zero-copy performance.
3. **GAT-Based Zero-Copy**: Port traits use Generic Associated Types (GATs) to expose `Archived<'a>` views that are tied to the `&self` lifetime of the port adapter (which holds the transaction).
4. **Closure-Scoped Access**: Hot-path reads use Higher-Ranked Trait Bounds (HRTBs) to enforce that archived views never escape the transaction scope.

### 1. Split Port Pattern

Defined in `<context>/ports.rs`:

```rust
// Read-only capability
pub trait SchemaQueryPort {
    type Error: std::error::Error;
    type Archived<'a> where Self: 'a;  // GAT for zero-copy

    // Cold tier: Owned reads
    fn find_owned(&self, id: &SchemaId) -> Result<Option<Schema>, Self::Error>;

    // Hot tier: Zero-copy closure-scoped reads
    fn with_archived<R>(
        &self,
        id: &SchemaId,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;
}

// Write-only capability
pub trait SchemaCommandPort {
    type Error: std::error::Error;

    fn save(&self, schema: &Schema) -> Result<(), Self::Error>;
    fn delete(&self, id: &SchemaId) -> Result<bool, Self::Error>;
}
```

### 2. Infrastructure Adapters

Defined in `db/<context>_adapter.rs`. Adapters implement the ports defined by the domain.

```rust
pub struct RedbSchemaQueryAdapter<'db> {
    db: &'db Database,
}

impl SchemaQueryPort for RedbSchemaQueryAdapter<'_> {
    type Error = DbError;
    type Archived<'a> = &'a ArchivedSchema; // or &'a ArchivedStoredSchema

    fn with_archived<R>(
        &self,
        id: &SchemaId,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R
    ) -> Result<Option<R>, DbError> {
        // ... open transaction, get byte slice, verify, call f ...
    }
}
```

### 3. Ergonomic Type Aliases

Defined in `<context>/mod.rs` to hide generic complexity from application code:

```rust
pub type RedbSchemaQuery<'db> = Query<RedbSchemaQueryAdapter<'db>>;
pub type RedbSchemaCommand<'db> = Command<RedbSchemaCommandAdapter<'db>>;
```

## Alternatives Considered

### Alternative 1: Single "Store" Trait (Traditional Repository Pattern)
- **Pros**: Simpler to define (one trait per context).
- **Cons**: Violates Interface Segregation Principle. Read-only test fakes must implement `save/delete` methods (usually with `unimplemented!()`), reducing type safety.
- **Verdict**: Rejected. Split ports provide better testing ergonomics and architectural clarity.

### Alternative 2: Object-Safe Traits (`dyn Store`)
- **Pros**: Dynamic dispatch, smaller binary size (no monomorphization).
- **Cons**: Cannot use GATs effectively for zero-copy. `Archived<'a>` types become very difficult to express without leaking implementation details. Performance cost of vtable lookups in hot paths.
- **Verdict**: Rejected. Zero-copy performance is a core NFR.

### Alternative 3: Direct Database Dependency
- **Pros**: Simplest implementation, no traits.
- **Cons**: Tightly couples domain to Redb/rkyv. Impossible to unit test domain logic without a real database instance. Hard to swap backends later.
- **Verdict**: Rejected. We need testability and loose coupling.

## Technical Validation

### Zero-Copy Feasibility
The GAT + HRTB pattern (`impl for<'a> FnOnce(Self::Archived<'a>) -> R`) has been validated to work with Rust's borrow checker. It correctly prevents `Archived` references from escaping the closure while allowing the adapter to manage the transaction lifetime.

### Benchmark Impact
Initial benchmarks of this pattern (vs direct DB usage) show negligible overhead due to static dispatch (monomorphization) effectively inlining the calls.

## Consequences

- **Positive**:
  - **Type Safety**: Test fakes can be strictly read-only or write-only.
  - **Performance**: Zero-copy hot paths preserved via GATs.
  - **Decoupling**: Domain logic depends on capabilities (traits), not implementations (Redb).
  - **Testability**: Trivial to write in-memory fakes for unit tests.

- **Negative**:
  - **Boilerplate**: Requires defining traits, implementing generic structs, and type aliases.
  - **Complexity**: GATs and HRTBs are advanced Rust features that may be harder for new contributors to understand initially.

## References
- [Core Architectural Decisions](../../_bmad-output/planning-artifacts/architecture/core-architectural-decisions.md)
- [ADR 003: Domain Serialization Strategy](./0003-domain-serialization.md)
