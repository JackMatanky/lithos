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

### Phase 1: Trait Definition Updates
- [ ] Task 1: Add `CacheGuard` trait and `CacheReader::Guard` associated type in `mod.rs`
  - [ ] Subtask 1.1: Define `pub trait CacheGuard<V> { fn deref(&self) -> &V; }` or similar generic usage
  - [ ] Subtask 1.2: Update `CacheReader` trait to include `type Guard: CacheGuard<V>`
  - [ ] Subtask 1.3: Update `CacheReader::get` signature to return `Result<Option<Self::Guard>, ...>`

### Phase 2: Codec Enhancements
- [ ] Task 2: Add `serialized_size` and `serialize_into` to `Codec` trait
  - [ ] Subtask 2.1: Update `Codec` trait in `encoder.rs`
  - [ ] Subtask 2.2: Implement methods for `RkyvCodec` to support zero-copy sizing and writing

### Phase 3: Redb Implementation Refactor
- [ ] Task 3: Implement `insert_reserve` and `AccessGuard` handling in `redb.rs`
  - [ ] Subtask 3.1: Refactor `RedbCache` to use `insert_reserve` for zero-copy writes
  - [ ] Subtask 3.2: Implement `RedbCache::get` to return a Guard wrapping `redb::AccessGuard`
  - [ ] Subtask 3.3: Handle memory alignment checks (zero-copy if aligned, copy fallback if not)
  - [ ] Subtask 3.4: Implement `get_many` and `put_many` using single transaction

### Phase 4: Moka Implementation Refactor
- [ ] Task 4: Change Moka storage to `Entry<V>` and add metrics
  - [ ] Subtask 4.1: Update `MokaCache` to store `Entry<V>` (wrapping value + timestamp)
  - [ ] Subtask 4.2: Implement zero-copy timestamp checks by accessing `Entry` metadata
  - [ ] Subtask 4.3: Expose `metrics()` and `run_pending_tasks()`
  - [ ] Subtask 4.4: Update `MokaCache::get` to return `Arc` wrapper as Guard

### Phase 5: Coordinator Refactor
- [ ] Task 5: Refactor `coordinator.rs` to use monomorphic generics
  - [ ] Subtask 5.1: Remove `Box<dyn ...>` or `&dyn ...` dispatch where possible
  - [ ] Subtask 5.2: Use generics for `Reader` and `Writer` implementations
  - [ ] Subtask 5.3: Ensure `Coordinator` correctly composes new Guard-based readers

### Phase 6: Validation & Benchmarking
- [ ] Task 6: Verify performance and correctness
  - [ ] Subtask 6.1: Run existing tests to ensure no regressions
  - [ ] Subtask 6.2: Create/Run benchmarks to verify performance gains (47x timestamp, 10x batch)

## Dev Notes

### Architecture Compliance
- **Zero-Copy**: Critical for performance. Use `rkyv`'s `Archived<T>` and `redb`'s `AccessGuard`.
- **Guard Pattern**: Allows holding a lock/reference to the underlying storage while accessing data.

### Technical Requirements
- **Redb Alignment**: `rkyv` requires aligned memory. `redb` buffers might not be aligned. Check `bytecheck` or alignment before casting.
- **Moka Entry**: Storing `Entry` allows checking metadata without cloning the value or deserializing if it was lazy.

### References
- [Source: Epic 5 Story 5.6]
- [Source: ADR 0002 Redb + rkyv]
