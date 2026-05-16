---
name: explicit-redb-adapter-seam
status: accepted
supersedes: []
date_proposed: 2026-05-16
date_decided: 2026-05-16
date_implemented:
stakeholders: [Engineering]
---

# ADR 018: Explicit Redb Adapter Seam

## Context

The `db` module was originally designed as a domain-agnostic persistence layer that attempted to hide both `redb` transaction mechanics and `rkyv` serialization details. This manifested in monolithic wrapper modules (`reader.rs` and `writer.rs`) containing dozens of highly duplicated methods (e.g., `get`, `get_owned`, `get_by_uuid`, `get_owned_by_uuid`, `scan_range`, etc.).

This approach proved incompatible with Rust's type system and created severe architectural friction:
1. **Method Explosion**: The `db` module became a "god object" trying to anticipate every possible domain query.
2. **Leaky Abstractions**: To support `rkyv` zero-copy access safely, massive and unreadable trait bounds (`V::Archived: rkyv::Portable + for<'archived> rkyv::bytecheck::CheckBytes...`) leaked into every public method signature in the `db` API.
3. **Rigid Adapters**: Context storage adapters (like `SchemaRedbRepository`) were forced into N+1 query anti-patterns because the `db` module hid the `redb` transactions.
4. **Implementation Bloat**: Attempting to extract table patterns into `TableOps` traits recreated a shallow wrapper over `redb` that hid its capabilities without adding semantic value.

## Decision

We will transition the `db` module from a **shallow transaction wrapper** to a **deep codec provider**. We will explicitly expose `redb` table operations to the context adapters, but hide serialization safety behind a new `DbEntity` (Codec) trait.

Specifically, we will:
1. **Expose `redb` Primitives**: Context storage adapters (e.g., `SchemaRedbRepository`) will interact directly with `redb` transaction primitives (e.g., `tx.inner.open_table(SCHEMAS.definition())?`) instead of calling helper methods on the transaction wrappers.
2. **Introduce `DbEntity` (Codec) Trait**: We will define a `DbEntity` trait that encapsulates all `rkyv` serialization, alignment (`AlignedVec`), and validation (`bytecheck`) logic.
3. **Blanket Implement `DbEntity`**: Since our domain entities (like `Schema`) already pragmatically derive `rkyv::Archive`, we will provide a blanket implementation of `DbEntity` for any type that satisfies the required `rkyv` bounds.
4. **GAT-Powered Zero-Copy**: The `DbEntity` trait will use a Generic Associated Type (GAT) to represent the zero-copy view (`type View<'a>`), allowing context adapters to execute zero-copy reads without leaking `rkyv::Archived` types into their signatures.
5. **Retain Compile-Time Wrappers**: We will keep the `UuidTable` and `PathTable` compile-time wrappers to enforce type safety on table definitions, but remove all runtime extension traits (e.g., `UuidTableReadExt`).

## Alternatives Considered

### Alternative 1: Table-Specific Operation Traits (`UuidTableOps`)
We considered extracting the table operations into segregated traits implemented directly on the table wrappers (e.g., `UuidTableOps` on `UuidTable`).
- **Pros**: Strong encapsulation of table-specific logic.
- **Cons**: This recreates the "shallow wrapper" anti-pattern. We would have to continuously add methods (`scan_multimap`, `iter_rev`) to these traits to support new domain queries, effectively rebuilding the entire `redb` API surface just to hide it. This directly contradicts the need for adapters to understand and optimize the data flow to `redb`.

### Alternative 2: Implementing `redb::Value` directly on Domain Entities
We considered implementing `redb::Value` directly on types like `Schema`.
- **Pros**: Eliminates the need for a separate serialization abstraction entirely.
- **Cons**: The `redb::Value::from_bytes` signature is infallible (`fn from_bytes(data: &[u8]) -> Self::SelfType`). Since `rkyv` requires byte-level validation for memory safety, an invalid byte slice would force a thread panic rather than returning a `Result`. Additionally, `redb::Value` forces a choice between owned vs zero-copy return types, whereas we need both.

## Technical Validation

### Research Findings
The decision relies on the interaction between `redb` and `rkyv` where safe zero-copy access requires alignment checks and data validation (`bytecheck`). Extracting this specifically into a `DbEntity` trait encapsulates the 5-line trait bounds currently infecting `reader.rs` and `writer.rs`.

### Benchmarks & Prototypes
A prototype implementation of the `DbEntity` trait using GATs (`type View<'a> where Self: 'a`) confirmed that zero-copy references to `Archived` types can be returned safely without exposing the `rkyv` machinery to the caller's interface. By exposing `tx.inner.open_table` directly, adapters can safely manage their own `AccessGuard` lifetimes, resolving known `rkyv`/`redb` lifetime footguns.

## Consequences

- **Positive**:
    - Massive reduction in boilerplate (`reader.rs` and `writer.rs` can be mostly deleted).
    - Perfect locality for serialization safety bugs (all in the `DbEntity` impl).
    - High visibility and control of transaction boundaries within context adapters.
    - Zero `rkyv` trait bounds in public API signatures.
- **Negative**:
    - Context adapters must manually call `tx.inner.open_table` and orchestrate the `redb` transaction mechanics directly.
- **Risks**:
    - The pragmatic leak of `rkyv` derives onto domain entities remains, tying domain models permanently to `rkyv`'s macro requirements.

## References

- [PRD: Refactor DB Module for Maintainability and Type Safety](../.scratch/db-refactor/PRD.md)
- [Clean Architecture (Robert C. Martin)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
