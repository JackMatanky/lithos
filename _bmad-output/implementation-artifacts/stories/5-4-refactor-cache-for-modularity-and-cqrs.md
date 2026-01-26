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

### Phase 1: Serialization Abstraction (`deserializer.rs`)
- [ ] Task 1: Define `CacheCodec` and implementations
  - [ ] Subtask 1.1: Create `crates/adapters/src/spi/cache/deserializer.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod deserializer;` to `mod.rs`
  - [ ] Subtask 1.3: [TDD] Write failing tests for `CacheCodec::encode_value` and `decode_value`
  - [ ] Subtask 1.4: Define `trait CacheCodec<K, V>` with `encode_key`, `encode_value`, and `decode_value`
  - [ ] Subtask 1.5: Define `RkyvCodec` struct
  - [ ] Subtask 1.6: [TDD] Write failing tests for `RkyvCodec` using a complex `Entry<V>` structure
  - [ ] Subtask 1.7: Implement `RkyvCodec` with `rkyv::api::high` and `rkyv::access` for zero-copy
  - [ ] Subtask 1.8: Define `IdentityCodec` for in-memory caches (no-op)
  - [ ] Subtask 1.9: Run `mise run test:unit:adapters deserializer` (GREEN)
  - [ ] Subtask 1.10: Run `mise run lint` and verify no `rkyv` noise leaks into the trait interface

### Phase 2: Unified Construction (`builder.rs`)
- [ ] Task 2: Implement `CacheBuilder` and concrete builders
  - [ ] Subtask 2.1: Create `crates/adapters/src/spi/cache/builder.rs`
  - [ ] Subtask 2.2: Add `pub mod builder;` to `mod.rs`
  - [ ] Subtask 2.3: [TDD] Write failing tests for a generic `CacheBuilder` interface
  - [ ] Subtask 2.4: Define `trait CacheBuilder` with associated `Reader` and `Writer` types
  - [ ] Subtask 2.5: Define `MokaBuilder` with fluent methods for `max_capacity`, `ttl`, and `tti`
  - [ ] Subtask 2.6: Define `RedbBuilder` with fluent methods for `path`, `table`, and `durability`
  - [ ] Subtask 2.7: [TDD] Write failing tests ensuring `RedbBuilder` validates file permissions
  - [ ] Subtask 2.8: Implement `build()` logic for both (returning placeholders for now)
  - [ ] Subtask 2.9: Run `mise run test:unit:adapters builder` (GREEN)

### Phase 3: Redb Storage Components (`redb.rs`)
- [ ] Task 3: Implement `DatabaseManager` and `TransactionExecutor`
  - [ ] Subtask 3.1: [TDD] Write failing test for `DatabaseManager` thread-safe initialization
  - [ ] Subtask 3.2: Implement `DatabaseManager` to encapsulate `Arc<redb::Database>`
  - [ ] Subtask 3.3: [TDD] Write failing test for `TransactionExecutor` error mapping (Redb -> CacheError)
  - [ ] Subtask 3.4: Implement `TransactionExecutor` to wrap `tokio::task::spawn_blocking` and `info_span`
  - [ ] Subtask 3.5: [TDD] Write failing test for `TableHandle` isolation
  - [ ] Subtask 3.6: Implement `TableHandle` to encapsulate `TableDefinition` and repetitive string keys
  - [ ] Subtask 3.7: Run `mise run test:unit:adapters redb` (GREEN)

### Phase 4: Inner State & Encapsulation
- [ ] Task 4: Implement `Inner` structs and shareable state
  - [ ] Subtask 4.1: Define `pub(crate) struct RedbInner<K, V, C>` in `redb.rs`
  - [ ] Subtask 4.2: Move `DatabaseManager`, `TransactionExecutor`, and `TableHandle` into `RedbInner`
  - [ ] Subtask 4.3: Define `pub(crate) struct MokaInner<K, V>` in `moka.rs`
  - [ ] Subtask 4.4: [TDD] Write failing tests for `RedbInner` transaction batching logic
  - [ ] Subtask 4.5: Implement transaction batching in `RedbInner` (GREEN)
  - [ ] Subtask 4.6: Ensure `Inner` structs are non-clonable and only accessed via `Arc`

### Phase 5: Reader and Writer Handles (CQRS)
- [ ] Task 5: Implement separate handles and traits
  - [ ] Subtask 5.1: [TDD] Write failing tests for `CacheReader` and `CacheWriter` handle separation
  - [ ] Subtask 5.2: Implement `RedbReader` and `RedbWriter` as `Arc<RedbInner>` wrappers
  - [ ] Subtask 5.3: Implement `MokaReader` and `MokaWriter` as `Arc<MokaInner>` wrappers
  - [ ] Subtask 5.4: Ensure `RedbReader` returns `EntryView` (wrapping `AccessGuard`) for true zero-copy
  - [ ] Subtask 5.5: Update `Builder` implementations to return these handles correctly
  - [ ] Subtask 5.6: Run `mise run test:unit:adapters` and verify CQRS enforcement (GREEN)

### Phase 6: API Transparency & Friendly Names
- [ ] Task 6: Implement default type parameters and aliases
  - [ ] Subtask 6.1: Update `Cache` struct (if unified) to use `C = RkyvCodec` default
  - [ ] Subtask 6.2: Define `pub type RedbCache<K, V> = (RedbReader<K, V>, RedbWriter<K, V>)` aliases in `mod.rs`
  - [ ] Subtask 6.3: [TDD] Write failing test for the "Zero-Knowledge" API (user doesn't see Codec)
  - [ ] Subtask 6.4: Implement re-exports to hide `Inner` and `Executor` from the crate root (GREEN)

### Phase 7: Final Refactor & Zero-Copy Verification
- [ ] Task 7: Complete Redb and Moka refactor with performance audit
  - [ ] Subtask 7.1: Remove all legacy monolithic code from `redb.rs` and `moka.rs`
  - [ ] Subtask 7.2: [TDD] Write a "Zero-Copy Probe" test using `rkyv::access` on a large Redb value
  - [ ] Subtask 7.3: Verify no heap allocations occur in the read hot-path
  - [ ] Subtask 7.4: Verify `Durability::None` configuration actually impacts performance in a micro-benchmark
  - [ ] Subtask 7.5: Run `mise run verify` (GREEN)

### Phase 8: Coordinator Integration & Final Test Review
- [ ] Task 8: Update Coordinator and conduct project-wide verification
  - [ ] Subtask 8.1: Update `CacheCoordinator` to use the separate Reader/Writer handles
  - [ ] Subtask 8.2: [TDD] Write failing tests for coordinator "Backfill" (L2 Reader -> L1 Writer)
  - [ ] Subtask 8.3: Implement backfill orchestration using the new CQRS handles
  - [ ] Subtask 8.4: Perform a final "Trait Bound Audit" ensuring zero `rkyv` noise in `redb.rs`
  - [ ] Subtask 8.5: Run `mise run verify` and `pre-commit run --all-files`
  - [ ] Subtask 8.6: Stage and commit all changes with a descriptive conventional commit message

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
