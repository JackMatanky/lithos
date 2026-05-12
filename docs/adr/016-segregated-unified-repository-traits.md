---
name: segregated-unified-repository-traits
status: accepted
supersedes: [002]
date_proposed: 2026-05-12
date_decided: 2026-05-12
date_implemented:
stakeholders: [Engineering]
---

# ADR 016: Segregated Unified Repository Traits

## Context

The "Unified Repository Traits" pattern established in ADR 002 mandated a single `Repository` trait per domain context containing all persistence operations. While this effectively decoupled domain logic from infrastructure, it introduced several maintainability and architectural challenges as the codebase scaled:

1.  **Monolithic Implementation Files**: In the `redb` storage layer, implementation files for storage adapters (e.g., `SchemaRedbRepository`) grew significantly (exceeding 600 lines) even for contexts with only a few tables. This accumulated all read, write, and batch logic into a single choke point.
2.  **Rust Coherence Constraints**: Splitting a single `impl Trait for Type` across multiple files is not natively supported in Rust. This made it difficult to modularize the storage adapter implementation without introducing complex delegation patterns or making internal fields public.
3.  **Interface Segregation Violations**: Consumers that only require read access (such as query handlers or read-only background tasks) are forced to depend on a trait that also exposes destructive write operations, reducing type-level safety and intent.

## Decision

We will evolve the "Unified Repository Traits" into a **Segregated Unified Repository** pattern:

1.  **Trait Segregation**: Each domain context will define a hierarchy of three traits in its `repository.rs`:
    *   `ReadRepository`: Pure read operations (e.g., `get`, `find`, `list`, `count`).
    *   `WriteRepository`: Pure write operations (e.g., `save`, `save_many`, `delete`).
    *   `Repository`: A marker trait that extends both `ReadRepository` and `WriteRepository` to maintain the "Unified" capability where needed.
2.  **Naming Convention**:
    *   During the transition from v1 to v2 storage, traits will retain context prefixes (e.g., `SchemaReadRepository`).
    *   After legacy cleanup, these will move to generic names qualified by their module: `schema::ReadRepository`, `schema::WriteRepository`, and `schema::Repository`.
3.  **Implementation Splitting**: The implementation struct (e.g., `RedbRepository`) will have its logic split into `read.rs` and `write.rs` within the context's `storage/` directory.
    *   `read.rs` will contain the `impl ReadRepository for RedbRepository` block.
    *   `write.rs` will contain the `impl WriteRepository for RedbRepository`.
4.  **Field Accessibility**: To support this split, implementation struct fields (like `store: Arc<Store>`) will use `pub(crate)` visibility to ensure child modules can access the database handle without exposing it to the entire crate.

## Alternatives Considered

### Alternative 1: Status Quo (ADR 002 Unified Trait)
- **Pros**: Simplest interface; minimal boilerplate.
- **Cons**: Resulted in 600+ line files that are difficult to navigate; prevented implementation modularization. Rejected because it fails the maintainability requirement for complex contexts like `Schema`.

### Alternative 2: Implementation Delegation (Wrapper Structs)
We considered using separate `Reader` and `Writer` structs that the `Repository` delegates to.
- **Pros**: Stronger separation of concerns.
- **Cons**: High delegation overhead (manually forwarding every method); complex lifetime management for `redb` transactions across multiple structs. Rejected for excessive boilerplate and complexity.

## Technical Validation

### Research Findings
The decision was triggered by the implementation of `SchemaRedbRepository` in `storage_v2/core.rs`, which reached 667 lines after implementing basic batch semantics. The split into `read.rs` and `write.rs` using the segregated trait pattern was prototyped and successfully solved the file-size and navigation issues.

### Benchmarks & Prototypes
Prototypes confirmed that `impl SchemaReadRepository for SchemaRedbRepository` in a child module works correctly provided the struct fields have internal crate visibility, avoiding orphan rule violations.

## Consequences

- **Positive**:
    - **Maintainability**: Logic is naturally partitioned into read and write concerns.
    - **Readability**: Smaller files with clear operation-focused boundaries.
    - **Type Safety**: Consumers can depend on read-only interfaces, enforcing safety at the type level.
- **Negative**:
    - **Boilerplate**: Requires defining three traits instead of one per context.
    - **Discovery**: New developers must understand the trait hierarchy (Read + Write = Unified).
- **Risks**:
    - Potential for "Trait Bloat" if too many specialized interfaces are created; should be limited to the Read/Write/Unified triad.

## References

- [ADR 002: Storage Pattern](./002-storage-pattern.md) - The original unified repository decision.
- [redb documentation](https://docs.rs/redb) - Influenced transaction and table access patterns.
- [Clean Architecture (Robert C. Martin)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) - Principles of Interface Segregation.
