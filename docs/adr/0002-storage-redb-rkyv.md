---
name: high-performance-metadata-storage-with-redb-and-rkyv
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-08
date_decided: 2026-01-11
date_implemented: 2026-01-11
---

# ADR 0002: High-Performance Metadata Storage with Redb and rkyv

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

## Appendix: Deep Dive into rkyv Mechanics & Philosophy

rkyv 0.8 is not just a serialization library; it is a **zero-copy data architecture**. To use it effectively, developers must move past the "parse-and-allocate" mindset of `serde` and embrace the following core principles.

### 1. The Philosophy: Total Memory Separation
Traditional serialization (like `serde_json` or `bincode`) aims to transform bytes into a native Rust struct. rkyv explicitly rejects this.
- **The Split**: Every type `T` that implements `Archive` is associated with a distinct `Archived<T>` type.
- **Ownership**: The native `T` is used for **construction and mutation** (owning the data). The `Archived<T>` is used for **zero-copy access** (viewing the data).
- **Result**: You never "deserialize" a vault index into memory. You open a memory-mapped file and access the `ArchivedVault` directly, treating the disk as extended memory.

### 2. The "Secret Sauce": Relative Pointers (`RelPtr`)
The primary reason zero-copy data usually fails is **ASLR (Address Space Layout Randomization)**. If a buffer contains a standard pointer (an absolute 64-bit address), that pointer becomes invalid the moment the buffer is loaded at a different address.
- **The Mechanic**: rkyv replaces all pointers with `RelPtr`. Instead of an absolute address, a `RelPtr` stores a **signed offset** relative to its own position in the buffer.
- **Position Independence**: Because the *distance* between the pointer and its target remains constant even if the entire buffer moves, the pointer remains valid.
- **Performance**: Accessing a `RelPtr` is just an `offset_from_this + *this` calculation, which is effectively free on modern CPUs.

### 3. The 3-Step Archival Pipeline
rkyv uses a sophisticated trait system to build archived data while maintaining safety:
1.  **`Serialize`**: The native type is traversed. Any "off-line" data (like the contents of a `String` or `Vec`) is written to the end of the buffer.
2.  **`Resolver`**: As data is written, a "Resolver" object is created. This holds the metadata (like the distance to the newly written string) needed to build the pointer.
3.  **`Resolve`**: The `Archive::resolve` method is called. It uses the `Resolver` to write the final `Archived<T>` into the buffer. This "backfilling" ensures that pointers can only point to data that has already been reliably laid out.

### 4. Layout Stability: The `Portable` Trait
For a zero-copy buffer to be shared across processes or stored on disk, it must have a **stable memory layout**.
- **The Problem**: Rust's default `#[repr(Rust)]` allows the compiler to reorder fields or change padding at will, which would break the archive.
- **The Solution**: rkyv enforces the `Portable` trait. This guarantees that the archived type has a stable, well-defined layout (effectively `#[repr(C)]` with strict alignment rules) that is consistent across different compiler versions and target architectures.

### 5. Safety & Integrity: `bytecheck`
Accessing raw bytes as a Rust struct is inherently `unsafe`. A single bit-flip could create an invalid `enum` variant or a null reference.
- **Mechanism**: rkyv integrates with the `bytecheck` crate.
- **The Pass**: Before you get an `&Archived<T>`, the `access::<T>` function performs a **recursive validation pass**. It traverses the entire byte buffer, verifying that every field, tag, and relative pointer is mathematically valid for the target type.
- **Efficiency**: This validation is done once at the "access" boundary. Once a buffer is validated, all subsequent field accesses are 100% safe and zero-overhead.

### 6. Niche Optimization & Layout Density
rkyv is optimized for **mechanical sympathy**—keeping data compact to maximize CPU cache hits.
- **Niching**: rkyv supports niche optimization (e.g., `#[rkyv(niche)]`). It can store an `Option<NonZeroU32>` in the same 4 bytes as a `u32` by using the zero bit-pattern to represent `None`.
- **Impact**: In a vault with millions of optional fields (like tags or metadata keys), niching can reduce the index size by 30-50%, drastically reducing I/O and cache pressure.

### 7. Shared Pointer Deduplication
The knowledge graph often has many notes referencing the same tag or user.
- **Mechanism**: By using the `Sharing` strategy, rkyv detects if a shared pointer (like `Arc<T>`) has already been serialized.
- **The Result**: Instead of duplicating the data, it writes a `RelPtr` to the *existing* location of that data in the buffer. This transforms the archive into a **Directed Acyclic Graph (DAG)**, ensuring that "hot" shared metadata is only stored once.
