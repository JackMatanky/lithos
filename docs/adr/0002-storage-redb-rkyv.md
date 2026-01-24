# ADR 0002: High-Performance Metadata Storage with Redb and rkyv

- **Status**: Accepted
- **Date**: 2026-01-11
- **Stakeholders**: Jack (Developer), Architects

## Context

The Lithos project requires a high-performance metadata index to support real-time features in a Command Line Interface (CLI) and a future Language Server Protocol (LSP).
Key performance requirements include:

- Sub-50ms latency for link suggestions and resolution.
- Concurrent background indexing that does not block user queries.
- Scaling from small vaults to 100,000+ notes without performance degradation.

The previous Go implementation used a "directory-like" structure that was rigid and suffered from serialization overhead. In Rust, we need to leverage "Mechanical Sympathy"—aligning our storage strategy with the language's memory safety and zero-cost abstraction capabilities.

## Decision

We will use **Redb** as the primary storage engine, with values serialized using the **rkyv** zero-copy framework.

### 1. Storage Engine: Redb

- **Pure Rust:** Eliminates C-toolchain dependencies and FFI overhead.
- **ACID Transactions:** Ensures data integrity for the knowledge graph.
- **MVCC Concurrency:** Provides non-blocking snapshots for readers, essential for background indexing in the LSP.

### 2. Serialization: rkyv

- **Zero-Copy:** Maps bytes directly from the database disk/cache into Rust structs without allocation or parsing.
- **Performance:** Achieves CPU-cache speeds for "hot path" lookups like suggestions and backlinks.

### 3. Identity: UUID v7

- **Standardized:** Follows RFC 9562 for 128-bit sortable identifiers.
- **B-Tree Optimized:** Lexicographically sortable keys ensure efficient insertion and retrieval in Redb.
- **Decoupled Identity:** Decouples note identity from physical filesystem paths to avoid the "directory trap."

## Alternatives Considered

### SQLite

- **Pros**: Industry standard, robust SQL support, recursive CTEs for graph traversal.
- **Cons**: The overhead of the SQL parser and the mandatory data copying between the C API and Rust strings/vectors would consume a significant portion of our 50ms LSP budget.

### Sled

- **Pros**: Pure Rust, high performance.
- **Cons**: Beta status, history of database corruption in some versions, less mature transaction model than Redb.

## Technical Validation

### Research Findings

- **Zero-Copy Serialization**: Research into `rkyv` vs `serde_json` or `bincode` shows 10-100x speedups for large data structures as it avoids the "parse and allocate" step.
- **Embedded KVs**: `Redb` emerged as the most stable pure-Rust B-Tree implementation with MVCC support, critical for our async requirements.
- **Mechanical Sympathy**: This selection aligns our storage strategy with Rust's memory safety and zero-cost abstraction capabilities, moving from the rigid "directory-like" structure of the previous Go implementation to a high-performance Rust idiom.

### Compatibility & Performance

- **Hexagonal Alignment**: Isolated in the `adapters` layer, protecting the `domain` from storage specifics.
- **Performance Impact**: Critical for achieving the sub-50ms latency target for link suggestions and resolution, and scaling to 100,000+ notes.

## Consequences

- **Positive**:
  - **Extreme Performance**: Sub-millisecond data access for hot paths.
  - **Concurrency**: MVCC allows non-blocking background indexing.
  - **Stability**: ACID transactions protect the knowledge graph integrity.
- **Negative**:
  - **Schema Evolution**: Using `rkyv` requires careful management of byte-layouts and a robust versioning strategy.
  - **Relational Complexity**: We must manually implement graph traversals (backlinks) using bidirectional adjacency lists.
  - **Ecosystem**: Higher initial implementation complexity compared to SQL.

## Status Tracking

- **Proposed**: 2026-01-08
- **Accepted**: 2026-01-11
- **Implemented**: 2026-01-11
