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
**When** I run `mise run test:unit:adapters`
**Then** all existing cache tests pass without regression

**Given** I modify the `Cache` trait to use Guards
**When** I implement `CacheReader::Guard`
**Then** it allows accessing the value reference `&V`

## TDD Tasks / Subtasks

### Phase 1: Trait Definition Updates (Zero-Copy First)
- [ ] Task 1: Define `CacheGuard` and update `CacheReader` in `mod.rs`
  - [ ] Subtask 1.1: Define `pub trait CacheGuard<V>: Deref<Target = V> + Send + 'static`:
    - [ ] 1.1.1: Add a blanket implementation: `impl<T, V> CacheGuard<V> for T where T: Deref<Target = V> + Send + 'static {}`.
    - [ ] 1.1.2: Add `fn as_bytes(&self) -> &[u8]` to the trait (or a sub-trait if needed for the blanket impl) to enable zero-copy views.
  - [ ] Subtask 1.2: Update `CacheReader` trait:
    - [ ] 1.2.1: Remove `V: Clone` requirement from the trait bounds.
    - [ ] 1.2.2: Add `type Guard: CacheGuard<V>` associated type.
    - [ ] 1.2.3: Change `async fn get(&self, key: &K) -> Result<Option<Self::Guard>, CacheError>`.
    - [ ] 1.2.4: Add `#[deprecated] async fn get_owned(&self, key: &K) -> Result<Option<V>, CacheError> where V: Clone`.
  - [ ] Subtask 1.3: Update `CacheWriter` trait to use reference-based `put(&self, key: K, value: &V)`.
  - [ ] Subtask 1.4: Fix `mockall` and tests to accommodate the `Deref`-based Guard return type.

### Phase 2: Codec Zero-Copy Refactor
- [ ] Task 2: Refactor `Codec` trait for Zero-Copy purity in `encoder.rs`
  - [ ] Subtask 2.1: Mark legacy methods as `#[deprecated]` (`encode_key`, `encode_value`, `decode_key`, `decode_value`, `encode_key_into`, `encode_value_into`).
  - [ ] Subtask 2.2: Add `type ArchivedKey` and `type ArchivedValue` associated types to the `Codec` trait.
  - [ ] Subtask 2.3: Define new zero-copy handshake methods (`serialized_key_size`, `serialize_key_into`, `serialized_value_size`, `serialize_value_into`).
  - [ ] Subtask 2.4: Define new pure read views (`access_key`, `access_value`).
  - [ ] Subtask 2.5: Implement `RkyvCodec` extensions for `rkyv 0.8`:
    - [ ] 2.5.1: Implement `serialized_value_size` by using a `SizeSerializer` or `to_bytes` length.
    - [ ] 2.5.2: Implement `serialize_value_into` using `rkyv::api::high::to_bytes_in` with `rkyv::ser::writer::Buffer`.
    - [ ] 2.5.3: Implement `access_value` with explicit `std::mem::align_of` check and `rkyv::access`.
    - [ ] 2.5.4: Repeat implementation for Key methods (`serialized_key_size`, `serialize_key_into`, `access_key`).
    - [ ] 2.5.5: Ensure `MetadataMap` archives into `rkyv::collections::ArchivedHashMap` for $O(1)$ zero-copy lookups.
    - [ ] 2.5.6: Update trait bounds to ensure compatibility with `rkyv::rancor::Error` and `CheckBytes`.
  - [ ] Subtask 2.6: Update `encoder.rs` unit tests to use the new zero-copy handshake.

### Phase 3: Redb Implementation Refactor
- [ ] Task 3: Implement `insert_reserve` and `AccessGuard` handling in `redb.rs`
  - [ ] Subtask 3.1: Refactor `RedbCache` to use `serialized_value_size` + `insert_reserve` + `serialize_value_into` (Removing legacy `encode` usage).
  - [ ] Subtask 3.2: Implement `RedbCache::get` to return a Guard wrapping `redb::AccessGuard` and providing `access_value`.
  - [ ] Subtask 3.3: Handle memory alignment checks (zero-copy if aligned, copy fallback if not).
  - [ ] Subtask 3.4: Implement `get_many` and `put_many` using single transaction.

### Phase 4: Moka Implementation Refactor
- [ ] Task 4: Change Moka storage to `Entry<V>` and add metrics
  - [ ] Subtask 4.1: Update `MokaCache` to store `Entry<V>` (wrapping value + timestamp).
  - [ ] Subtask 4.2: Implement zero-copy timestamp and metadata checks by accessing `Entry` metadata via `ArchivedHashMap`.
  - [ ] Subtask 4.3: Expose `metrics()` and `run_pending_tasks()`.
  - [ ] Subtask 4.4: Update `MokaCache::get` to use `access_value` (Removing legacy `decode` usage).

### Phase 5: Coordinator Refactor
- [ ] Task 5: Refactor `coordinator.rs` to use monomorphic generics
  - [ ] Subtask 5.1: Remove `Box<dyn ...>` or `&dyn ...` dispatch where possible.
  - [ ] Subtask 5.2: Use generics for `Reader` and `Writer` implementations.
  - [ ] Subtask 5.3: Ensure `Coordinator` correctly composes new Guard-based readers.
  - [ ] Subtask 5.4: Migrate all remaining logic from `encode/decode` to `access/serialize` (Manual `.deserialize()` where owned types are required).

### Phase 6: Validation & Benchmarking
- [ ] Task 6: Verify performance and correctness
  - [ ] Subtask 6.1: Run existing tests to ensure no regressions.
  - [ ] Subtask 6.2: Create/Run benchmarks to verify performance gains (47x timestamp, 10x batch).

### Phase 7: Deprecation Cleanup
- [ ] Task 7: Remove legacy Codec support
  - [ ] Subtask 7.1: Remove all methods marked as `#[deprecated]` from `Codec` trait.
  - [ ] Subtask 7.2: Remove corresponding implementations from `RkyvCodec`.
  - [ ] Subtask 7.3: Verify the codebase contains zero references to `encode_key`, `decode_key`, etc.

## Dev Notes

### Architecture Compliance
- **Zero-Copy**: Critical for performance. Use `rkyv`'s `Archived<T>` and `redb`'s `AccessGuard`.
- **Guard Pattern**: Allows holding a lock/reference to the underlying storage while accessing data.

### Technical Requirements
- **Redb Alignment**: `rkyv` requires aligned memory. `redb` buffers might not be aligned. Check `bytecheck` or alignment before casting.
- **Moka Entry**: Storing `Entry` allows checking metadata without cloning the value or deserializing if it was lazy.

### Zero-Copy Codec Refactor Plan
- **Phased Migration**: Legacy methods are marked as `#[deprecated]` in Phase 2 to allow incremental migration of backends and the coordinator. Phase 7 removes them entirely once the migration is complete.
- **Two-Pass Write**: `redb` requires knowing the size before granting a buffer (`insert_reserve`). The codec now provides `serialized_size()` followed by `serialize_into(&mut [u8])`.
- **Pure Views**: The codec only provides `access_*` methods. If an owned value is needed, the caller must explicitly call `.deserialize()` on the archived view, making the performance penalty visible.
- **Rkyv 0.8 Strategy**: Use `rkyv::api::high::to_bytes_in` for zero-copy writes into pre-allocated memory via `rkyv::ser::writer::Buffer`. Leverage `rkyv::collections::ArchivedHashMap` for $O(1)$ zero-copy metadata lookups.

#### Planned `CacheReader` and `CacheGuard` Trait Definition
```rust
#[async_trait]
pub trait CacheReader<K, V>: Send + Sync
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Send + Sync + 'static, // Note: Clone no longer required!
{
    type Guard: CacheGuard<V>;

    /// Retrieve a guard providing zero-copy access to the value.
    async fn get(&self, key: &K) -> Result<Option<Self::Guard>, CacheError>;

    /// Deprecated: Forces cloning/deserialization. Use `get` instead.
    #[deprecated(note = "Use get() and access the view via the guard")]
    async fn get_owned(&self, key: &K) -> Result<Option<V>, CacheError>
    where V: Clone
    {
        self.get(key).await?.map(|g| g.clone()) // Error: requires Deref + Clone
    }
}

/// Guard type that provides deref access to cached values.
pub trait CacheGuard<V>: Deref<Target = V> + Send + 'static {
    /// Access raw bytes of the cached entry (for Codec::access).
    fn as_bytes(&self) -> &[u8];
}

// Blanket implementation for types like Arc<V>
impl<T, V> CacheGuard<V> for T
where
    T: Deref<Target = V> + Send + 'static,
    T: AsRef<[u8]> // Potential requirement for as_bytes()
{}
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
