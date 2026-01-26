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

### Phase 1: CacheCodec Implementation
- [ ] Task 1: Define `CacheCodec` trait and `RkyvCodec` implementation
  - [ ] Subtask 1.1: Create `crates/adapters/src/spi/cache/deserializer.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod deserializer;` to `mod.rs`
  - [ ] Subtask 1.3: Write failing tests for `CacheCodec` trait operations
  - [ ] Subtask 1.4: Implement `CacheCodec` trait
  - [ ] Subtask 1.5: Implement `RkyvCodec` for Redb support
  - [ ] Subtask 1.6: Run `mise run test:unit:adapters deserializer` and verify pass (GREEN)
  - [ ] Subtask 1.7: Run `mise run lint` and fix all warnings/errors

### Phase 2: Unified Builder Interface
- [ ] Task 2: Implement `CacheBuilder` trait and concrete builders
  - [ ] Subtask 2.1: Create `crates/adapters/src/spi/cache/builder.rs`
  - [ ] Subtask 2.2: Add `pub mod builder;` to `mod.rs`
  - [ ] Subtask 2.3: Write failing tests for `CacheBuilder::build()` interface
  - [ ] Subtask 2.4: Implement `CacheBuilder` trait
  - [ ] Subtask 2.5: Implement Moka and Redb builders following the trait
  - [ ] Subtask 2.6: Run `mise run test:unit:adapters builder` and verify pass (GREEN)
  - [ ] Subtask 2.7: Run `mise run lint` and fix all warnings/errors

### Phase 3: Inner State Implementation
- [ ] Task 3: Implement `Inner` structs for Moka and Redb
  - [ ] Subtask 3.1: Define `MokaInner` in `moka.rs` (move existing state there)
  - [ ] Subtask 3.2: Define `RedbInner` in `redb.rs` (move existing state there)
  - [ ] Subtask 3.3: Write failing unit tests for `Inner` structs
  - [ ] Subtask 3.4: Implement logic within `Inner` structs
  - [ ] Subtask 3.5: Run `mise run test:unit:adapters` and verify `Inner` logic (GREEN)

### Phase 4: Reader and Writer Handles (CQRS)
- [ ] Task 4: Implement separate handles for both backends
  - [ ] Subtask 4.1: Define `CacheReader` and `CacheWriter` traits if not already present
  - [ ] Subtask 4.2: Implement `MokaReader` and `MokaWriter` as `Arc<MokaInner>` wrappers
  - [ ] Subtask 4.3: Implement `RedbReader` and `RedbWriter` as `Arc<RedbInner>` wrappers
  - [ ] Subtask 4.4: Write failing tests verifying CQRS separation
  - [ ] Subtask 4.5: Run `mise run test:unit:adapters` and verify handles (GREEN)

### Phase 5: Redb Refactor Completion
- [ ] Task 5: Finalize `redb.rs` refactor
  - [ ] Subtask 5.1: Update `RedbCache` to be a handle-based implementation
  - [ ] Subtask 5.2: Ensure all previous tests pass with the new structure
  - [ ] Subtask 5.3: Verify zero-copy performance remains intact
  - [ ] Subtask 5.4: Run `mise run test:unit:adapters redb` and verify (GREEN)

### Phase 6: Moka Refactor Completion
- [ ] Task 6: Finalize `moka.rs` refactor
  - [ ] Subtask 6.1: Update `MokaCache` to be a handle-based implementation
  - [ ] Subtask 6.2: Ensure all previous tests pass with the new structure
  - [ ] Subtask 6.3: Run `mise run test:unit:adapters moka` and verify (GREEN)

### Phase 7: Integration & Final Review
- [ ] Task 7: Integrate with Coordinator and conduct final review
  - [ ] Subtask 7.1: Update `CacheCoordinator` to use the new handle-based backends
  - [ ] Subtask 7.2: Conduct final TDD review of the entire refactored suite
  - [ ] Subtask 7.3: Run `mise run verify` to ensure all quality gates pass
  - [ ] Subtask 7.4: Run `pre-commit run --all-files` (NEVER use `--no-verify`)
  - [ ] Subtask 7.5: Commit changes with a descriptive message

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
