# Story 5.4: Refactor Cache for Modularity and CQRS

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a system architect optimizing for modularity and scalability,
I want to refactor the cache implementations to use the Handle/Inner pattern and separate Reader/Writer traits,
So that the codebase is more maintainable, supports zero-copy operations more effectively, and adheres to CQRS principles.

## Original Epic Acceptance Criteria

**Given** the need for better modularity
**When** I implement the `CacheCodec` trait in `deserializer.rs`
**Then** it provides a unified interface for serialization/deserialization
**And** `RkyvCodec` implements this trait for zero-copy Redb storage

**Given** the need for a common builder interface
**When** I create `builder.rs`
**Then** it defines a `CacheBuilder` trait with a `build()` method
**And** both Moka and Redb provide builders implementing this trait

**Given** the Handle/Inner pattern is required
**When** I refactor `MokaCache` and `RedbCache`
**Then** the implementation is split into `Inner` structs (state) and Handles (interface)
**And** Handles are cheaply cloneable `Arc` wrappers

**Given** CQRS principles
**When** I implement separate `Reader` and `Writer` handles
**Then** consumers can request read-only or write-only access to the cache
**And** the `CacheCoordinator` uses these separate handles for orchestration

**Given** the refactor must maintain quality
**When** I follow strict TDD
**Then** tests are written before implementation for each phase
**And** zero-copy verification ensures `rkyv` is used correctly in the new structure

## TDD Acceptance Criteria (Quality Gates)

**Given** I am refactoring for modularity
**When** I run `mise run test:unit:adapters`
**Then** all tests pass for the new Handle/Inner structure
**And** `CacheCodec` tests verify correct serialization/deserialization for supported backends
**And** `CacheBuilder` tests verify that both Moka and Redb can be initialized via the unified interface

**Given** CQRS enforcement
**When** I use `CacheReader` and `CacheWriter` handles
**Then** the compiler enforces that read-only handles cannot perform write operations
**And** `CacheCoordinator` tests verify that it correctly uses separate reader/writer handles

**Given** zero-copy requirements per ADR 0002
**When** I use the refactored Redb implementation
**Then** `rkyv` zero-copy deserialization is verified to work with the new `CacheCodec` abstraction

## TDD Tasks / Subtasks

### Phase 1: CacheCodec Implementation (Serialization Abstraction)
- [ ] Task 1: Define `CacheCodec` trait and implementations in `deserializer.rs`
  - [ ] Subtask 1.1: Create `crates/adapters/src/spi/cache/deserializer.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod deserializer;` to `mod.rs`
  - [ ] Subtask 1.3: Define `trait CacheCodec<K, V>` with methods for `encode_key`, `encode_value`, and `decode_value`
  - [ ] Subtask 1.4: Define `RkyvCodec` struct and encapsulate the 20+ lines of `rkyv` trait bounds here
  - [ ] Subtask 1.5: Define `IdentityCodec` for Moka (no-op serialization for in-memory objects)
  - [ ] Subtask 1.6: Write failing tests in `deserializer.rs` for `RkyvCodec` round-trip serialization
  - [ ] Subtask 1.7: Implement `RkyvCodec` logic using `rkyv::api::high`
  - [ ] Subtask 1.8: Implement `decode_value` using `rkyv::access` to preserve zero-copy potential
  - [ ] Subtask 1.9: Run `mise run test:unit:adapters deserializer` (GREEN)
  - [ ] Subtask 1.10: Run `mise run lint` and verify documentation for the codec trait

### Phase 2: Unified Builder Interface
- [ ] Task 2: Implement `CacheBuilder` trait and specialized builders in `builder.rs`
  - [ ] Subtask 2.1: Create `crates/adapters/src/spi/cache/builder.rs`
  - [ ] Subtask 2.2: Add `pub mod builder;` to `mod.rs`
  - [ ] Subtask 2.3: Define `trait CacheBuilder` with an associated `Output` type and a `build(self)` method
  - [ ] Subtask 2.4: Implement `MokaBuilder` in `builder.rs` (supporting TTL, TTI, Capacity)
  - [ ] Subtask 2.5: Implement `RedbBuilder` in `builder.rs` (supporting Path, TableName, Durability)
  - [ ] Subtask 2.6: Write failing tests ensuring `RedbBuilder` can initialize a database and table
  - [ ] Subtask 2.7: Implement builder logic to return specialized Reader/Writer handles
  - [ ] Subtask 2.8: Run `mise run test:unit:adapters builder` (GREEN)

### Phase 3: Inner State Implementation (Encapsulation)
- [ ] Task 3: Implement `Inner` structs to hold shared database/cache state
  - [ ] Subtask 3.1: Define `RedbInner<K, V, C>` in `redb.rs` as `pub(crate)`
  - [ ] Subtask 3.2: Move `Arc<redb::Database>` and `table_name` into `RedbInner`
  - [ ] Subtask 3.3: Implement a `run_blocking` helper on `RedbInner` to unify `spawn_blocking` and span management
  - [ ] Subtask 3.4: Define `MokaInner<K, V>` in `moka.rs` wrapping the `moka` cache instance
  - [ ] Subtask 3.5: Write failing tests for `RedbInner` transaction isolation
  - [ ] Subtask 3.6: Implement transaction logic within `RedbInner` (GREEN)

### Phase 4: Reader and Writer Handles (CQRS Split)
- [ ] Task 4: Implement distinct handles that share the same `Inner` state
  - [ ] Subtask 4.1: Create `RedbReader<K, V, C = RkyvCodec>` and `RedbWriter<K, V, C = RkyvCodec>`
  - [ ] Subtask 4.2: Implement `CacheReader` for `RedbReader` using zero-copy `rkyv::access`
  - [ ] Subtask 4.3: Implement `CacheWriter` for `RedbWriter`
  - [ ] Subtask 4.4: Repeat for `MokaReader` and `MokaWriter`
  - [ ] Subtask 4.5: Write failing tests verifying that a `Reader` cannot be cast to a `Writer`
  - [ ] Subtask 4.6: Implement handle instantiation in builders (GREEN)

### Phase 5: Redb Refactor Completion & Zero-Copy Verification
- [ ] Task 5: Finalize `redb.rs` and verify performance constraints
  - [ ] Subtask 5.1: Clean up `redb.rs` to remove the old monolithic `Cache` struct
  - [ ] Subtask 5.2: Implement `pub type RedbCache<K, V> = (RedbReader<K, V>, RedbWriter<K, V>)` alias if needed, or similar friendly naming
  - [ ] Subtask 5.3: Add a specific "Zero-Copy Probe" test to verify no allocations occur during `RedbReader::get`
  - [ ] Subtask 5.4: Verify `Durability` settings are correctly passed from Builder to Inner
  - [ ] Subtask 5.5: Run `mise run test:unit:adapters redb` (GREEN)

### Phase 6: Moka Refactor Completion
- [ ] Task 6: Finalize `moka.rs` refactor
  - [ ] Subtask 6.1: Remove old monolithic `MokaCache` implementation
  - [ ] Subtask 6.2: Ensure `MokaReader/Writer` handles are correctly re-exported
  - [ ] Subtask 6.3: Run `mise run test:unit:adapters moka` (GREEN)

### Phase 7: Coordinator Integration & Final Review
- [ ] Task 7: Update Coordinator and project-wide verification
  - [ ] Subtask 7.1: Update `CacheCoordinator` to accept `Arc<dyn CacheReader>` and `Arc<dyn CacheWriter>` components
  - [ ] Subtask 7.2: Verify that `CacheCoordinator` correctly orchestrates backfill (DiskReader -> MemoryWriter)
  - [ ] Subtask 7.3: Perform a final "Trait Bound Audit" to ensure `redb.rs` is free of `rkyv` boilerplate
  - [ ] Subtask 7.4: Run `mise run verify` and `pre-commit run --all-files`
  - [ ] Subtask 7.5: Stage and commit all changes with a descriptive conventional commit message

## Dev Notes

### Architecture Compliance
- **Handle/Inner Pattern**: Follows standard Rust patterns for cheaply cloneable state handles.
- **CQRS**: Separates read and write interfaces to allow for more granular access control.
- **SPI Modularity**: Improves the utility of the caching layer for various adapters.

### Technical Requirements
- **Zero-Copy**: Must maintain `rkyv` zero-copy benefits through the `CacheCodec` abstraction.
- **Thread Safety**: All handles and inner states must remain `Send + Sync`.

### References
- [Source: project-context.md#Handle-Inner-Pattern]
- [Source: ADR 0002: Redb + rkyv storage]
- [Source: Story 5.1, 5.2, 5.3]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None

### Completion Notes List
- Refactored for modularity and CQRS.
- Implemented CacheCodec for abstraction.
- Implemented Handle/Inner pattern.
- Maintained zero-copy requirements.
