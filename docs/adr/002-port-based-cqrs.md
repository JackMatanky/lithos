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

Defined in `<context>/adapters/`. Adapters implement the ports defined by the domain and are scoped to their respective context.

```rust
// schema/adapters/query.rs
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl schema::ports::Query for QueryAdapter<'_> {
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

**Rationale for Context-Scoped Adapters:**
- **Cohesion**: All schema-related code lives in `schema/`
- **Independence**: Context can be tested with different storage backends
- **Clarity**: Generic `db/` module contains only database primitives, not business-specific adapters
- **Prevents Circular Dependencies**: Infrastructure depends on domain ports, not vice versa
- **No Premature Nesting**: Single `adapters/` directory sufficient until multiple backends exist

### 3. Ergonomic Type Aliases

Defined in `<context>/mod.rs` to hide generic complexity from application code:

```rust
// schema/mod.rs
use crate::schema::adapters::{QueryAdapter, CommandAdapter};

pub type RedbSchemaQuery<'db> = Query<QueryAdapter<'db>>;
pub type RedbSchemaCommand<'db> = Command<CommandAdapter<'db>>;

impl<'db> RedbSchemaQuery<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(QueryAdapter::new(db))
    }
}
```

### 4. Application Layer for Orchestration

Cross-context workflows are coordinated in the **application layer** within `lithos-core`, not in CLI drivers. This enables reusability across multiple drivers (CLI, LSP, future Web API) and maintains library-first design.

```rust
// application/services/note_creation.rs
pub struct NoteCreationService<'db> {
    note_cmd: note::RedbCommand<'db>,
    template_query: template::RedbQuery<'db>,
    schema_query: schema::RedbQuery<'db>,
}

impl NoteCreationService<'_> {
    pub fn create_from_template(
        &self,
        template_name: &str,
        schema_name: &str,
        context: TemplateContext,
    ) -> Result<Note, NoteCreationError> {
        // 1. Load template
        let template = self.template_query.find_by_name(template_name)?
            .ok_or(NoteCreationError::TemplateNotFound)?;

        // 2. Load schema
        let schema = self.schema_query.find_by_name(schema_name)?
            .ok_or(NoteCreationError::SchemaNotFound)?;

        // 3. Render template with validation
        let rendered = template.render(context)?;
        schema.validate(&rendered.frontmatter)?;

        // 4. Create note
        let note = self.note_cmd.create(&rendered.path)?;

        Ok(note)
    }
}
```

**Benefits:**
- **Reusability**: Same service used by CLI and LSP
- **Context Isolation**: Business contexts don't import each other
- **Library-First**: `lithos-core` independently useful
- **Testability**: Application services easily tested with fake ports

### 5. Error Handling Strategy

To maintain decoupling while providing useful feedback, we adopt a layered error strategy:

- **Port Error (`S::Error`)**: The port trait defines an associated `Error` type.
- **CQRS Error (`QueryError<E>`)**: The CQRS layer wraps storage errors in a structured enum generic over the storage error.

```rust
#[derive(Debug, thiserror::Error)]
pub enum QueryError<E> {
    #[error("Storage error: {0}")]
    Storage(E),
    #[error("Data corruption: {0}")]
    Corruption(String),
    #[error("Validation failed: {0}")]
    Validation(String),
}
```

- **Concrete Type**: The adapter implementation defines the concrete error (e.g., `DbError`).
- **Result**: The public API returns `Result<T, QueryError<DbError>>`.
- **Application Layer**: Application services define their own error types that aggregate errors from multiple contexts.

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

## Appendix A: Operational & Security Guidelines

### Observability
- **Instrumentation**: CQRS entry points and adapter methods must be instrumented with `tracing` spans.
- **Metrics**: Hot-path queries should capture hit/miss ratios and duration.
- **Redaction**: Keys must be redacted in logs if they contain sensitive user data.

### Data Integrity & Security
- **Untrusted Input**: Archived data (rkyv bytes) must be treated as untrusted input at the adapter boundary.
- **Validation**: Adapters must perform validation (bytecheck) before exposing `Archived<'a>` views.
- **Recovery**: Corruption errors should be structured to allow upper layers to trigger recovery workflows (e.g., re-indexing).

### Risks & Mitigations

| Risk | Mitigation |
| :--- | :--- |
| **Port Bloat** | Keep ports capability-driven; only add methods for validated query shapes. |
| **Generic Proliferation** | Use type aliases (`RedbSchemaQuery`) to hide generics from 99% of call sites. |
| **Object Safety** | If `dyn` is truly needed, provide a separate `ObjectSafeStore` trait for the cold tier only. |

## Command-Side Read-for-Write

In CQRS, commands sometimes need to read current state to compute new state (e.g., allocating version numbers, computing rollback targets). This creates a tension: the command side needs read access, but CQRS emphasizes separating read and write models.

### Decision: Encapsulate Read-for-Write in Command Ports

We resolve this by encapsulating read-for-write operations as atomic methods on the Command port itself:

```rust
pub trait Command: Send + Sync {
    /// Allocates the next version number for a vault atomically.
    /// Reads current version, computes next, returns without persisting.
    fn get_next_version(&self, vault_id: VaultId) -> Result<Version, Self::Error>;

    /// Rolls back the active version by `steps` atomically.
    /// Reads current version, computes target, updates active pointer in one transaction.
    fn rollback_active_version(
        &self,
        vault_id: VaultId,
        steps: u32,
    ) -> Result<Version, Self::Error>;
}
```

### Why This Maintains CQRS Boundaries

1. **Query Model Stays Separate**: The Query port remains read-only for external read operations (LSP, UI). Command-side reads are internal implementation details, not exposed as a read API.

2. **Atomic Operations**: Using database transactions (`read_write_transaction`), these methods atomically read and write, preventing race conditions.

3. **Command Ownership**: The command model owns its state transitions. Version allocation and rollback are command responsibilities, not query responsibilities.

4. **Storage Isolation**: Both Query and Command ports can use the same storage backend (Redb) without coupling—the Command port simply exposes higher-level atomic operations.

### Implementation

- Added `Database::read_write_transaction()` for atomic read+write in one transaction
- Command adapter implements `get_next_version` and `rollback_active_version` using this API
- Command struct no longer depends on Query port—only Command port

## References
- [Core Architectural Decisions](../../_bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md)
- [ADR 003: Domain Serialization Strategy](./0003-domain-serialization.md)
- [Design Doc 012: Concrete CQRS Generic Over Port](../../docs/design/012-cqrs-concrete-over-port.md)
