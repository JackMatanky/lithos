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
**And** the handles are compatible with multi-layer coordination requirements

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
- [ ] Task 1: Define `Codec` and implementations
  - [ ] Subtask 1.1: Create `crates/adapters/src/spi/cache/deserializer.rs` and register in `mod.rs`
  - [ ] Subtask 1.2: [TDD] Write `round_trip::should_preserve_metadata_and_value` (failing)
  - [ ] Subtask 1.3: Define `trait Codec<K, V>` with `encode_key`, `encode_value`, and `decode_value`
  - [ ] Subtask 1.4: Re-export `Codec` as `CacheCodec` in `mod.rs`
  - [ ] Subtask 1.5: Define `RkyvCodec` struct and implement `Codec`
  - [ ] Subtask 1.6: [TDD] Write `rkyv_codec::should_return_error_on_corrupted_bytes` (failing)
  - [ ] Subtask 1.7: Implement `RkyvCodec` using `rkyv::api::high` and `rkyv::access`
  - [ ] Subtask 1.8: Define `IdentityCodec` for in-memory caches (no-op pass-through)
  - [ ] Subtask 1.9: Run `mise run test:unit:adapters deserializer` (GREEN)
  - [ ] Subtask 1.10: Run `mise run lint` and verify no `rkyv` bounds leak into the public trait

### Phase 2: Unified Construction (`builder.rs`)
- [ ] Task 2: Implement `Builder` and specialized builders
  - [ ] Subtask 2.1: Create `crates/adapters/src/spi/cache/builder.rs` and register in `mod.rs`
  - [ ] Subtask 2.2: [TDD] Write `redb_builder::should_fail_when_path_is_directory` (failing)
  - [ ] Subtask 2.3: Define `trait Builder` with associated `Reader` and `Writer` types
  - [ ] Subtask 2.4: Re-export `Builder` as `CacheBuilder` in `mod.rs`
  - [ ] Subtask 2.5: Implement `MokaBuilder` with fluent methods for `max_capacity`, `ttl`, `tti`
  - [ ] Subtask 2.6: [TDD] Write `redb_builder::should_initialize_db_with_correct_table` (failing)
  - [ ] Subtask 2.7: Implement `RedbBuilder` supporting `path`, `table`, and `Durability`
  - [ ] Subtask 2.8: Ensure all builders have `tracing::instrument` on `build()`
  - [ ] Subtask 2.9: Run `mise run test:unit:adapters builder` (GREEN)

### Phase 3: Redb Storage Components (`redb.rs`)
- [ ] Task 3: Implement `DatabaseManager` and `TransactionExecutor`
  - [ ] Subtask 3.1: Import `CacheReader` and `CacheWriter` as `CacheReaderPort` and `CacheWriterPort`
  - [ ] Subtask 3.2: [TDD] Write `database_manager::should_share_same_instance_across_clones` (failing)
  - [ ] Subtask 3.3: Implement `DatabaseManager` to encapsulate `Arc<redb::Database>` with error logging
  - [ ] Subtask 3.4: [TDD] Write `transaction_executor::should_map_redb_error_to_cache_error` (failing)
  - [ ] Subtask 3.5: Implement `TransactionExecutor` to wrap `tokio::task::spawn_blocking` and instrument with `info_span` and `tracing::error!` mapping
  - [ ] Subtask 3.6: [TDD] Write `table_handle::should_prevent_table_name_collisions` (failing)
  - [ ] Subtask 3.7: Implement `TableHandle` to encapsulate `TableDefinition` logic
  - [ ] Subtask 3.8: Run `mise run test:unit:adapters redb` (GREEN)

### Phase 4: Inner State & Encapsulation
- [ ] Task 4: Implement `Inner` structs and shareable state
  - [ ] Subtask 4.1: Define `pub(crate) struct Inner<K, V, C>` locally in `redb.rs` and `moka.rs`
  - [ ] Subtask 4.2: [TDD] Write `redb_inner::should_batch_multiple_writes_in_single_transaction` (failing)
  - [ ] Subtask 4.3: Implement write batching logic in `Inner` with `tracing` instrumentation
  - [ ] Subtask 4.4: Ensure `Inner` structs are non-clonable and only accessed via `Arc`
  - [ ] Subtask 4.5: Implement shared helpers for logging backend-specific stats (e.g., table size)
  - [ ] Subtask 4.6: Run `mise run test:unit:adapters` (GREEN)

### Phase 5: Reader and Writer Handles (CQRS Split)
- [ ] Task 5: Implement `Reader` and `Writer` handles
  - [ ] Subtask 5.1: Define `pub struct Reader<K, V, C>` and `pub struct Writer<K, V, C>` locally in `redb.rs` and `moka.rs`
  - [ ] Subtask 5.2: Re-export as `RedbReader/RedbWriter` and `MokaReader/MokaWriter` in `mod.rs`
  - [ ] Subtask 5.3: [TDD] Write `cqrs::reader_should_not_have_access_to_port_writer_methods` (failing)
  - [ ] Subtask 5.4: Implement `CacheReaderPort` for `Reader` handles with instrumentation
  - [ ] Subtask 5.5: Implement `CacheWriterPort` for `Writer` handles with instrumentation
  - [ ] Subtask 5.6: [TDD] Write `redb_reader::should_return_entry_view_without_allocating` (failing)
  - [ ] Subtask 5.7: Implement `EntryView` for true zero-copy retrieval
  - [ ] Subtask 5.8: Run `mise run test:unit:adapters` (GREEN)

### Phase 6: API Transparency & Friendly Names
- [ ] Task 6: Implement default type parameters and aliases
  - [ ] Subtask 6.1: Update `Reader/Writer` handles to use `C = RkyvCodec` default in `redb.rs`
  - [ ] Subtask 6.2: Define `pub type RedbCache<K, V> = (RedbReader<K, V>, RedbWriter<K, V>)` aliases in `mod.rs`
  - [ ] Subtask 6.3: [TDD] Write `api::should_allow_usage_without_specifying_codec` (failing)
  - [ ] Subtask 6.4: Implement re-exports to hide `Inner` and `TransactionExecutor` from the public SPI
  - [ ] Subtask 6.5: Run `mise run test:unit:adapters` (GREEN)

### Phase 7: Final Refactor & NFR Verification
- [ ] Task 7: Complete refactor with performance and observability audit
  - [ ] Subtask 7.1: Remove all legacy monolithic code from `redb.rs` and `moka.rs`
  - [ ] Subtask 7.2: [TDD] Write `observability::should_emit_nested_spans_for_transactions` (failing)
  - [ ] Subtask 7.3: Verify nested spans using `TestTracingSubscriber` across all new components
  - [ ] Subtask 7.4: [TDD] Write `nfr::zero_copy_probe::should_verify_direct_pointer_access` (failing)
  - [ ] Subtask 7.5: Verify no heap allocations occur in the read hot-path
  - [ ] Subtask 7.6: Run `mise run verify` (GREEN)

### Phase 8: Final Review & Quality Gate
- [ ] Task 8: Comprehensive project verification
  - [ ] Subtask 8.1: Perform a final "Trait Bound Audit" ensuring zero `rkyv` noise in backend files
  - [ ] Subtask 8.2: Verify that `RedbReader` and `RedbWriter` correctly use `tracing` for all I/O
  - [ ] Subtask 8.3: Run `mise run verify` and `pre-commit run --all-files`
  - [ ] Subtask 8.4: Stage and commit all changes with a descriptive conventional commit message

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
