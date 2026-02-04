# Story 5.6: Cache Performance & Zero-Copy Refactor

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a performance engineer optimizing the storage layer,
I want to implement a "Guard-Based Trait" design and leverage advanced crate features,
So that we achieve zero-copy reads/writes and significant performance improvements.

**Context:**
We have completed a deep analysis of `redb`, `moka`, and `rkyv` and identified significant performance gaps in the current implementation. We need a dedicated story to execute the "Guard-Based Trait" design (Level 2 architecture) and leverage advanced crate features.

## Original Epic Acceptance Criteria

**Given** the need for zero-copy reads
**When** I implement the `CacheReader` trait
**Then** it returns `Result<Option<Self::Guard>>`
**And** `redb` returns `AccessGuard` wrappers (zero-copy aligned or single-copy unaligned)
**And** `moka` returns `Arc` wrappers without allocation

**Given** the need for zero-copy writes
**When** I update the `Codec`
**Then** it supports `serialize_into` and `serialized_size`
**And** `redb` writes use `insert_reserve` to write directly to the memory-mapped file

**Given** the need for efficient batch processing
**When** I implement operations on `redb`
**Then** it supports single-transaction `get_many` and `put_many`
**And** benchmarks show ~10x speedup for batch scans

**Given** alignment requirements in `redb`
**When** I access data
**Then** the implementation handles alignment correctly (fallback to copy for unaligned reads)

**Given** Moka optimization goals
**When** I update the `moka` implementation
**Then** it stores `Entry<V>` instead of `V` to enable zero-copy timestamp checks
**And** it exposes `metrics()` and explicit maintenance hooks
**And** benchmarks show ~47x speedup for timestamp checks

**Given** architectural requirements for the Guard pattern
**When** I refactor `coordinator.rs`
**Then** it uses monomorphic generics instead of trait objects

**Given** existing functionality
**When** I run the test suite
**Then** all existing tests pass

## TDD Acceptance Criteria (Quality Gates)

**Given** I need high-performance zero-copy access
**When** I run benchmarks
**Then** significant improvements are observed in read/write operations

**Given** I am refactoring core traits
**When** I run `mise run test:unit:core`
**Then** all existing cache tests pass without regression

**Given** I modify the `Cache` trait to use Guards
**When** I implement `CacheReader::Guard`
**Then** it allows accessing the value reference `&V`

## TDD Tasks / Subtasks

### Phase 1: Codec Zero-Copy Refactor
- [ ] Task 1: Refactor `Codec` trait for Zero-Copy purity in `encoder.rs`
  - [ ] Subtask 1.1: Mark legacy methods as `#[deprecated]` (`encode_key`, `encode_value`, `decode_key`, `decode_value`, `encode_key_into`, `encode_value_into`).
  - [ ] Subtask 1.2: Add `type ArchivedKey` and `type ArchivedValue` associated types to the `Codec` trait.
  - [ ] Subtask 1.3: Define new zero-copy handshake methods (`serialized_key_size`, `serialize_key_into`, `serialized_value_size`, `serialize_value_into`).
  - [ ] Subtask 1.4: Define new pure read views (`access_key`, `access_value`).
  - [ ] Subtask 1.5: Implement `RkyvCodec` extensions for `rkyv 0.8`:
    - [ ] 1.5.1: Implement "Aligned-or-Copy" strategy: check alignment of input slices and copy to an `AlignedVec` only if the 16-byte requirement is not met.
    - [ ] 1.5.2: Implement `serialized_value_size` and `serialize_value_into` using `rkyv` 0.8 handshake.
    - [ ] 1.5.3: Ensure `MetadataMap` archives into `rkyv::collections::ArchivedHashMap`.
  - [ ] Subtask 1.6: Update `encoder.rs` unit tests to use the new zero-copy handshake.

### Phase 2: Timestamp & Guard Foundation
- [ ] Task 2: Define unified timestamp and guard structures
  - [ ] Subtask 2.1: Define `pub struct CacheTimestamp(u64)` in `mod.rs`.
  - [ ] Subtask 2.2: Define `pub trait CacheGuard<V>: Deref<Target = V> + Send + 'static`.
  - [ ] Subtask 2.3: Add `fn timestamp(&self) -> CacheTimestamp` to `CacheGuard`.
  - [ ] Subtask 2.4: Update `MokaReader` to store `Arc<(V, CacheTimestamp)>` to enable nanosecond staleness checks without disk access.
  - [ ] Subtask 2.5: Update `RedbReader` to return a Guard that provides zero-copy access to the `timestamp` within the archived `Entry<V>`.

### Phase 3: Read API Evolution (Stream & Prefix)
- [ ] Task 3: Implement new high-performance read APIs
  - [ ] Subtask 3.1: Add `async fn get(&self, key: &K) -> Result<Option<Self::Guard>, CacheError>` to `CacheReader`.
  - [ ] Subtask 3.2: Add `async fn timestamp(&self, key: &K) -> Result<Option<CacheTimestamp>, CacheError>` to `CacheReader`.
  - [ ] Subtask 3.3: Refactor `keys()` and `scan_prefix()` to return `BoxStream`.
  - [ ] Subtask 3.4: Add `#[deprecated] async fn get_owned(&self, key: &K) -> Result<Option<V>, CacheError> where V: Clone`.
  - [ ] Subtask 3.5: Implement `scan_prefix` in `RedbReader` using `table.range(prefix..)` for $O(log N)$ directory listing.

### Phase 4: Single Write API Evolution
- [ ] Task 4: Refactor `CacheWriter` for zero-copy and reference-based writes
  - [ ] Subtask 4.1: Update `CacheWriter::put` signature: `async fn put(&self, key: K, value: &V, timestamp: CacheTimestamp)`.
  - [ ] Subtask 4.2: Implement reference-based `put` in `MokaWriter`.
  - [ ] Subtask 4.3: Implement zero-copy `put` in `RedbWriter` using `codec.serialized_size` + `insert_reserve` + `codec.serialize_into`.

### Phase 5: Batch Operations & Directory Performance
- [ ] Task 5: Implement high-performance batch and scanning APIs
  - [ ] Subtask 5.1: Add `get_many` and `get_many_timestamps` to `CacheReader` (providing default loop implementations).
  - [ ] Subtask 5.2: Add `put_many` to `CacheWriter`.
  - [ ] Subtask 5.3: Override batch methods in `RedbReader/Writer` to use single transactions.
  - [ ] Subtask 5.4: Implement `scan_prefix` in `RedbReader` using `table.range(prefix..)` for $O(log N)$ directory-style listing.
  - [ ] Subtask 5.5: Implement `scan_prefix` in `MokaReader` (filtering keys).

### Phase 6: Moka-Specific Enhancements
- [ ] Task 6: Add Moka metrics, maintenance, and weigher APIs
  - [ ] Subtask 6.1: Add `metrics()` API returning `MokaMetrics` (entry_count, weighted_size, max_capacity).
  - [ ] Subtask 6.2: Add `run_pending_tasks()` to `MokaWriter`.
  - [ ] Subtask 6.3: Integrate `run_pending_tasks()` into `clear()` to ensure deterministic test cleanup.
  - [ ] Subtask 6.4: Add `weigher` support to `MokaBuilder` for size-based eviction policies.
  - [ ] Subtask 6.5: Refactor existing Moka unit tests to remove `sleep` workarounds in favor of explicit maintenance calls.

### Phase 7: Redb-Specific Enhancements
- [ ] Task 7: Finalize Redb performance optimizations
  - [ ] Subtask 7.1: Implement specialized `timestamp(key)` in `RedbReader` that reads only the `CacheMetadata` prefix from the memory-mapped bytes.
  - [ ] Subtask 7.2: Verify zero-copy `insert_reserve` performance in `RedbWriter`.
  - [ ] Subtask 7.3: Implement `compact()` or analyze `redb` auto-compaction settings for long-term database health.
  - [ ] Subtask 7.4: Ensure alignment fallback logic is robust and logs performance warnings when triggered.

### Phase 8: Coordinator & Monomorphism Refactor
- [ ] Task 8: Refactor `coordinator.rs` for performance and zero-copy routing
  - [ ] Subtask 8.1: Eliminate `Box<dyn ...>` trait objects. Refactor `Reader<K, V, RM, RD>` and `Writer<K, V, WM, WD>` to use monomorphic generics for memory (M) and disk (D) backends.
  - [ ] Subtask 8.2: Implement `CacheReader::get` for the Coordinator:
    - [ ] 8.2.1: Define `CoordinatorGuard` enum that can wrap either a memory-cache Guard or a disk-cache Guard.
    - [ ] 8.2.2: Implement `get` logic: Check memory, return `CoordinatorGuard::Memory`. If miss, check disk, return `CoordinatorGuard::Disk`, and trigger background backfill.
  - [ ] Subtask 8.3: Implement optimized `timestamp()` routing:
    - [ ] 8.3.1: Check memory cache first (using the lightweight `CacheTimestamp`).
    - [ ] 8.3.2: Fall back to disk if memory miss.
  - [ ] Subtask 8.4: Implement `scan_prefix` and `keys()` routing:
    - [ ] 8.4.1: Stream results primarily from disk.
    - [ ] 8.4.2: For `keys()`, merge and deduplicate streams from memory and disk using `StreamExt::merge` or similar.
  - [ ] Subtask 8.5: Refactor `put` for reference-based zero-copy writes:
    - [ ] 8.5.1: Write to Disk first using the 2-pass `Codec` handshake.
    - [ ] 8.5.2: Write to Memory using the owned `(V, CacheTimestamp)` tuple.
  - [ ] Subtask 8.6: Update `BackfillWorker` to handle the new `Entry<V>` to `(V, CacheTimestamp)` conversion.

### Phase 9: Deprecation Cleanup & Validation
- [ ] Task 9: Final cleanup and performance verification
  - [ ] Subtask 9.1: Remove all methods marked as `#[deprecated]` from `Codec` and traits.
  - [ ] Subtask 9.2: Run full benchmark suite to verify 47x/10x performance targets.

## Dev Notes

### Architecture Compliance
- **Zero-Copy**: Critical for performance. Use `rkyv`'s `Archived<T>` and `redb`'s `AccessGuard`.
- **Guard Pattern**: Allows holding a lock/reference to the underlying storage while accessing data.

### Technical Requirements
- **Redb Alignment**: `rkyv` requires aligned memory. `redb` buffers might not be aligned. Check `bytecheck` or alignment before casting.
- **Moka Entry**: Storing `Entry` allows checking metadata without cloning the value or deserializing if it was lazy.

### Zero-Copy Codec Refactor Plan
- **Aligned-or-Copy Strategy**: `redb` values are memory-mapped but not guaranteed to be 16-byte aligned. The `RkyvCodec` will now check alignment and perform a single copy into an `AlignedVec` only if necessary, ensuring `rkyv` compatibility without forcing disk-level alignment.
- **Directory Traversal**: Added `scan_prefix` to the `CacheReader` to support $O(\log N)$ directory listing in the Directory module.
- **Streaming Keys**: `keys()` and `scan_prefix()` now return `BoxStream` to prevent memory spikes during large cache enumerations.
- **Entry in Moka**: Storing `Entry<V>` in Moka enables $O(1)$ nanosecond timestamp checks by treating `timestamp` as a field access rather than a property of a deserialized object.
- **Phased Migration**: Legacy methods are marked as `#[deprecated]` in Phase 2 to allow incremental migration of backends and the coordinator. Phase 9 removes them entirely once the migration is complete.
- **Two-Pass Write**: `redb` requires knowing the size before granting a buffer (`insert_reserve`). The codec now provides `serialized_size()` followed by `serialize_into(&mut [u8])`.
- **Pure Views**: The codec only provides `access_*` methods. If an owned value is needed, the caller must explicitly call `.deserialize()` on the archived view, making the performance penalty visible.
- **Rkyv 0.8 Strategy**: Use `rkyv::api::high::to_bytes_in` for zero-copy writes into pre-allocated memory via `rkyv::ser::writer::Buffer`. Leverage `rkyv::collections::ArchivedHashMap` for $O(1)$ zero-copy metadata lookups.

#### Planned `CacheReader` and `CacheGuard` Trait Definition
```rust
#[async_trait]
pub trait CacheReader<K, V>: Send + Sync
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Guard: CacheGuard<V>;

    /// Retrieve a guard providing zero-copy access to the value and timestamp.
    async fn get(&self, key: &K) -> Result<Option<Self::Guard>, CacheError>;

    /// Batch retrieval of values.
    async fn get_many(&self, keys: &[K]) -> Result<Vec<Option<V>>, CacheError> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get_owned(key).await?);
        }
        Ok(results)
    }

    /// Retrieve only the timestamp for a key (Zero-copy optimization).
    async fn timestamp(&self, key: &K) -> Result<Option<CacheTimestamp>, CacheError>;

    /// Retrieve all keys currently present in the cache as a stream.
    fn keys(&self) -> BoxStream<'_, Result<K, CacheError>>;

    /// Scan keys starting with a specific prefix (Directory traversal optimization).
    fn scan_prefix(&self, prefix: &str) -> BoxStream<'_, Result<K, CacheError>>;

    /// Batch retrieval of timestamps.
    async fn get_many_timestamps(&self, keys: &[K]) -> Result<Vec<Option<u64>>, CacheError> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.timestamp(key).await?.map(|t| t.0));
        }
        Ok(results)
    }

    /// Existing: Backward compatibility (Forces cloning).
    async fn get_owned(&self, key: &K) -> Result<Option<V>, CacheError>
    where V: Clone
    {
        Ok(self.get(key).await?.map(|g| (*g).clone()))
    }

    /// Existing: Performance optimization (Uses get internally).
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        Ok(self.get(key).await?.is_some())
    }
}

/// Guard type that provides deref access to cached values.
pub trait CacheGuard<V>: Deref<Target = V> + Send + 'static {
    /// Access raw bytes of the cached entry (for Codec::access).
    fn as_bytes(&self) -> &[u8];

    /// Access the lightweight timestamp.
    fn timestamp(&self) -> CacheTimestamp;
}
```



#### Planned `Codec` Trait Definition
```rust
pub trait Codec<K, V>: Send + Sync {
    /// The archived representation for zero-copy access.
    type ArchivedKey: ?Sized;
    type ArchivedValue: ?Sized;

    // --- READ PATH: Zero-Copy Views ---

    /// Provide zero-copy access to the archived key.
    fn access_key<'a>(&self, bytes: &'a [u8]) -> Result<&'a Self::ArchivedKey, CacheError>;

    /// Provide zero-copy access to the archived value.
    fn access_value<'a>(&self, bytes: &'a [u8]) -> Result<&'a Self::ArchivedValue, CacheError>;

    // --- WRITE PATH: The Handshake ---

    /// Pass 1: How much space do we need for the key?
    fn serialized_key_size(&self, key: &K) -> Result<usize, CacheError>;

    /// Pass 2: Write key directly into storage-provided memory.
    fn serialize_key_into(&self, key: &K, target: &mut [u8]) -> Result<usize, CacheError>;

    /// Pass 1: How much space do we need for the value?
    fn serialized_value_size(&self, value: &V) -> Result<usize, CacheError>;

    /// Pass 2: Write value directly into storage-provided memory.
    fn serialize_value_into(&self, value: &V, target: &mut [u8]) -> Result<usize, CacheError>;

    // --- DEPRECATED METHODS (To be removed in Phase 7) ---
    #[deprecated(note = "Use access_key instead")]
    fn decode_key(&self, encoded: &[u8]) -> Result<K, CacheError>;
    // ... others ...
}
```

### References
- [Source: Epic 5 Story 5.6]
- [Source: ADR 0002 Redb + rkyv]
