# ADR 0002: High-Performance Metadata Storage with Redb and rkyv

## Status
Accepted

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

## Rationale
While SQLite is a reliable choice, the overhead of the SQL parser and the mandatory data copying between the C API and Rust strings/vectors would consume a significant portion of our 50ms LSP budget. The Redb + rkyv combination provides the "instant-access" speed required for a world-class developer tool.

## Consequences
- **Schema Evolution:** Using `rkyv` requires careful management of byte-layouts. We will need a robust versioning strategy for archived buffers.
- **Relational Complexity:** We must manually implement graph traversals (backlinks) using bidirectional adjacency lists, as we will not have SQLite's Recursive CTEs.
- **Ecosystem:** We move from "Boring Technology" (SQL) to "High-Performance Rust Idioms," which increases the initial implementation complexity but ensures the project's long-term competitive advantage.
