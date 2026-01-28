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

## Appendix: High-Performance Redb Utilities & Design Patterns

To maximize the capabilities of Redb 3.1 in the Lithos vault, the following patterns and utilities MUST be leveraged. These patterns prioritize "Mechanical Sympathy" by aligning storage operations with Rust's memory model and the underlying filesystem's behavior.

### 1. High-Performance Zero-Copy Reads
Zero-copy reads are the primary mechanism for achieving sub-50ms latency in the Lithos knowledge graph. This bypasses the traditional "read, allocate, and parse" cycle.

- **Mechanism**: `table.get(key)` returns an `AccessGuard`. This guard is a smart pointer to the memory-mapped page containing the value.
- **Pointer Stability**: The `AccessGuard` implements `Deref<Target = [u8]>`. This slice is a direct view into the OS page cache (or disk-mapped memory).
- **rkyv Integration**: Use `rkyv::access_unchecked::<T>(slice)` for raw speed or `rkyv::check_archived_root::<T>(slice)` for mandatory safety (as per Rule 26).
- **Memory Layout**: The returned `ArchivedT` is not a new struct; it is a reference to the data exactly as it exists on disk. Accessing a field (e.g., `archived_note.title`) is just a pointer offset calculation followed by a memory read.
- **Lifetime Safety**:
    - **Critical Invariant**: The lifetime of the `Archived` reference is tied to the `AccessGuard`. If the guard is dropped, the memory may be unmapped or reused.
    - **Pattern**: Never return the `Archived` type directly from an adapter. Instead, return a DTO or map it within a closure that maintains the guard's scope.
- **Performance Impact**: This approach allows Lithos to "read" complex note metadata in nanoseconds, as the only "work" performed is the B-Tree lookup.

### 2. Zero-Copy Write via `insert_reserve`
Standard `insert(key, &value)` requires Redb to allocate an internal buffer, copy your data into it, and then eventually write it to the database page. To eliminate this intermediate copy:
- **Mechanism**: Use `table.insert_reserve(key, size) -> Result<ReservedWriteGuard>`.
- **Implementation**: The `ReservedWriteGuard` provides a mutable byte slice (`&mut [u8]`) mapped directly to the database page. Use `rkyv` to serialize data directly into this slice.
- **Benefit**: Achieves true zero-copy writes from the application logic to the persistence layer, minimizing CPU cache misses and memory pressure.

### 3. Durability Tuning for Bulk Operations
Redb's default `Durability::Immediate` triggers an `fsync` on every commit, which is a significant bottleneck during vault initialization (10k+ notes).
- **Mechanism**: Set `WriteTransaction::set_durability(Durability::None)` for per-file indexing tasks.
- **The Flush Boundary**: Commits with `Durability::None` are ACID-compliant in memory but not guaranteed on disk until a subsequent `Durability::Immediate` commit occurs.
- **Strategy**: Perform bulk indexing in batches (e.g., 500 notes) using `None`, then perform a single empty commit with `Immediate` to flush the entire state to disk in one OS sync operation.

### 4. Concurrency via MVCC (Multi-Version Concurrency Control)
Redb's MVCC allows one writer and multiple concurrent readers without locks.
- **Snapshot Isolation**: Every `ReadTransaction` receives a point-in-time snapshot. Readers never block the `IndexerActor` from writing new data.
- **The Reader Starvation Risk**: Long-running `ReadTransaction`s prevent Redb from reclaiming old pages (vacuuming). This causes the database file to grow rapidly.
- **Guideline**: Keep read transactions short-lived. For LSP "hover" or "definition" lookups, open the transaction, extract the data via `rkyv` zero-copy, and drop the transaction immediately.

### 5. RAII Transaction Management & Snapshot Bloat
Manual transaction management is error-prone.
- **Pattern**: Wrap Redb transactions in a custom RAII struct (e.g., `VaultTransaction<'a>`).
- **Safety Invariant**: The `Drop` implementation must explicitly call `abort()` if the transaction wasn't committed. This prevents "zombie" snapshots from pinning old pages and causing irreversible file growth (snapshot bloat).

### 6. Database Configuration & Resource Tuning
Tuning the `DatabaseBuilder` is critical for scaling to 100,000+ notes:
- **`set_cache_size(bytes)`**: Redb uses an internal page cache. For Lithos, set this to ~20% of the total vault size or at least 128MB. This ensures the B-Tree's internal nodes and "hot" metadata remain in RAM, reducing IOPS.
- **`set_page_size(bytes)`**: The default 4KB is optimal for small metadata. However, if using large multimaps for backlinks, increasing to 8KB or 16KB can reduce B-Tree height (fewer disk seeks) and fragmentation, at the cost of slightly higher "slack" space.
- **`set_region_size(bytes)`**: Controls the growth increments of the database file. Larger regions reduce filesystem fragmentation on modern SSDs.

### 7. Maintenance, Integrity & Recovery
- **`MultimapTable`**: Essential for 1:N relations (e.g., `#tag -> [note_ids]`). It uses a specialized B-Tree structure that is significantly faster than storing a `Vec<Uuid>` inside a standard `Table` value, as it avoids serializing the entire list for every insertion.
- **`db.compact()`**: Perform this during the "Clean Up" phase. It relocates active pages to the start of the file and truncates the remainder. Requires NO active read transactions to be effective.
- **`db.check_integrity()`**: A deep-scan utility that re-calculates all B-Tree checksums. Should be exposed as a `lithos diagnostic` command to help users recover from suspected hardware failure or disk corruption.
- **Migration Strategy**: Since Redb doesn't store schemas, use a `VersionTable` (Key: "schema_version", Value: u32) to manage backward-compatible changes to `rkyv` structs.

## Appendix: High-Performance rkyv Patterns

To achieve "Mechanical Sympathy" and maximum throughput with rkyv 0.8, the following patterns MUST be used in the Lithos implementation.

### 1. Niche Optimization for Optional Metadata
Default `Option<T>` in rkyv adds a tag byte. For high-frequency metadata, use niching to reduce storage footprint and CPU branch pressure.
- **Mechanism**: Use `#[rkyv(niche)]` or `#[rkyv(with = NicheInto<NaN>)]`.
- **Implementation**: For `Option<NonZeroU32>`, niching uses the zero value as `None`. For `Option<bool>`, it uses invalid bit patterns.
- **Benefit**: Reduces the archived size of `NoteMetadata` and increases CPU cache density.

### 2. Validated Access Strategy (bytecheck)
As per **Rule 26**, all storage types MUST use validation. However, validation has a cost.
- **Pattern**: Use `rkyv::access::<T, _>(bytes)` for safe entry points.
- **Optimization**: For large read-heavy operations (e.g., building the knowledge graph), validate once and then use `access_unchecked::<T>` for subsequent field lookups within the same scope.
- **Security**: Never use `access_unchecked` on data provided directly by the user or from an untrusted source without a prior `check_bytes` pass.

### 3. Memory Alignment & AlignedVec
rkyv requires data to be aligned in memory according to the type's `align_of`.
- **Issue**: Standard `Vec<u8>` or memory-mapped slices from Redb may not be 8-byte or 16-byte aligned.
- **Pattern**: When serializing to an intermediate buffer, use `rkyv::util::AlignedVec`.
- **Redb Alignment**: Redb pages are naturally page-aligned (4KB), which satisfies most Rust types. However, always verify alignment if using `AccessGuard` with types requiring high alignment (e.g., SIMD types).

### 4. Zero-Copy Traversal (Knowledge Graph)
When traversing backlinks or tags, do not deserialize into owned types.
- **Mechanism**: Perform all logic on the `ArchivedNote` type.
- **Pattern**: `let archived = rkyv::access::<ArchivedNote>(bytes)?; if archived.tags.contains("#rust") { ... }`.
- **Benefit**: Bypasses the allocator entirely. The CPU performs direct memory reads from the memory-mapped Redb page.

### 5. Inline Value Updates
For high-frequency counters (e.g., note view counts or link weights), avoid the full "read-modify-write" cycle.
- **Mechanism**: Use `MutInPlaceValue` where applicable.
- **Implementation**: Combine with Redb's `AccessGuardMutInPlace` to modify bytes directly within the database page without re-serializing the entire record.
