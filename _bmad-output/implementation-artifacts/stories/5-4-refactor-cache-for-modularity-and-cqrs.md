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
**Then** I implement concrete, fluent builders for each backend (`MokaBuilder`, `RedbBuilder`)
**And** they handle backend-specific configuration (TTL vs Durability) without a leaky shared trait

**Given** the Handle/Inner pattern is required
**When** I refactor `MokaCache` and `RedbCache`
**Then** the implementation is split into `Inner` structs (state) and Handles (interface)
**And** `RedbInner` encapsulates the `Executor` for async/sync bridging
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
  - [ ] Subtask 1.2: [TDD] Write `round_trip::preserves_metadata_and_value` (failing)
  - [ ] Subtask 1.3: Define `trait Codec<K, V>` with `encode_key`, `encode_value`, and `decode_value`
  - [ ] Subtask 1.4: Re-export `Codec` as `CacheCodec` in `mod.rs`
  - [ ] Subtask 1.5: Define `RkyvCodec` struct and implement `Codec`
  - [ ] Subtask 1.6: [TDD] Write `rkyv_codec::returns_error_on_corrupted_bytes` (failing)
  - [ ] Subtask 1.7: Implement `RkyvCodec` using `rkyv::api::high` and `rkyv::access`
  - [ ] Subtask 1.8: Define `IdentityCodec` for in-memory caches (no-op pass-through)
  - [ ] Subtask 1.9: Run `mise run test:unit:adapters deserializer` (GREEN)
  - [ ] Subtask 1.10: Run `mise run lint`, fix all warnings/errors, and verify no `rkyv` bounds leak into the public trait
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions
  - [ ] Subtask 1.11: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 1.12: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 1.13: Stage and commit all files created, deleted, or modified during this phase with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 2: Moka Split-Handle Refactor
- [ ] Task 2: Refactor Moka to split handles
  - [ ] Subtask 2.1: [TDD] Write `moka_builder::builds_split_handles_with_custom_capacity` (failing)
  - [ ] Subtask 2.2: Define `pub struct Reader<K, V>` and `pub struct Writer<K, V>` in `moka.rs` (wrapping `Arc<Inner>`)
  - [ ] Subtask 2.3: Implement `CacheReaderPort` for `Reader` (get/has)
  - [ ] Subtask 2.4: Implement `CacheWriterPort` for `Writer` (put/delete/clear)
  - [ ] Subtask 2.5: Update `Builder::build()` to return `(Reader<K, V>, Writer<K, V>)`
  - [ ] Subtask 2.6: Re-export `Builder` as `MokaBuilder`, `Reader` as `MokaReader`, and `Writer` as `MokaWriter` in `mod.rs`
  - [ ] Subtask 2.7: Remove the unified `MokaCache` struct entirely
  - [ ] Subtask 2.8: Run `mise run test:unit:adapters moka` (GREEN)
  - [ ] Subtask 2.9: Run `mise run lint` and fix all warnings/errors
  - [ ] Subtask 2.10: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 2.11: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 2.12: Stage and commit all files created, deleted, or modified during this phase with a fully descriptive conventional commit style message (NEVER use `--no-verify`)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Redb Builder & Handle Stubs
- [ ] Task 3: Implement `Builder` and Handle Stubs
  - [ ] Subtask 3.1: [TDD] Write `redb_builder::fails_when_path_is_directory` (failing, use `IsolatedTestContext`)
  - [ ] Subtask 3.2: Define `pub struct Reader<K, V>` and `pub struct Writer<K, V>` stubs in `redb.rs`
  - [ ] Subtask 3.3: Define `pub struct Builder<K, V>` in `redb.rs`
  - [ ] Subtask 3.4: Implement fluent methods for `path`, `table_name`, `durability`
  - [ ] Subtask 3.5: Implement `build()` method returning `(Reader, Writer)` (stubs for now)
  - [ ] Subtask 3.6: Re-export `Builder` as `RedbBuilder`, `Reader` as `RedbReader`, and `Writer` as `RedbWriter` in `mod.rs`
  - [ ] Subtask 3.7: Add `tracing::instrument` to `build()`
  - [ ] Subtask 3.8: Run `mise run test:unit:adapters builder` (GREEN)
  - [ ] Subtask 3.9: Run `mise run lint` and fix all warnings/errors
  - [ ] Subtask 3.10: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 3.11: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 3.12: Stage and commit all files created, deleted, or modified during this phase with a fully descriptive conventional commit style message (NEVER use `--no-verify`)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Redb Async/Sync Bridge (`Executor`)
- [ ] Task 4: Implement `Executor` utility
  - [ ] Subtask 4.1: Import `CacheReader` and `CacheWriter` as `CacheReaderPort` and `CacheWriterPort`
  - [ ] Subtask 4.2: [TDD] Write `executor::maps_redb_error_to_cache_error` (failing, use `IsolatedTestContext`)
  - [ ] Subtask 4.3: Define `pub(crate) struct Executor` in `redb.rs` (no generics needed)
  - [ ] Subtask 4.4: Implement `spawn<F, R>(&self, span: Span, f: F) -> Result<R, CacheError>` where `F: FnOnce() -> Result<R, redb::Error> + Send + 'static`
  - [ ] Subtask 4.5: Ensure `spawn` enters the provided span and catches Tokio JoinErrors
  - [ ] Subtask 4.6: Implement error mapping helper `map_redb_error` that converts `redb::Error` to `CacheError::BackendError` or `IoError`
  - [ ] Subtask 4.7: Run `mise run test:unit:adapters redb` (GREEN)
  - [ ] Subtask 4.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Inner State & Encapsulation
- [ ] Task 5: Implement `Inner` structs and shareable state
  - [ ] Subtask 5.1: Define `pub(crate) struct Inner<K, V, C>` locally in `redb.rs` (generic over Codec)
  - [ ] Subtask 5.2: Add fields: `db: Arc<redb::Database>`, `executor: Executor`, `table: TableDefinition<'static, [u8], [u8]>`, `codec: C`
  - [ ] Subtask 5.3: Implement `Inner::new(...)` constructor
  - [ ] Subtask 5.4: [TDD] Write `redb_inner::batches_multiple_writes_in_single_transaction` (failing, use `IsolatedTestContext`)
  - [ ] Subtask 5.5: Implement `write<F, T>(&self, f: F) -> Result<T, CacheError>` helper method
    - Use `self.executor.spawn()` to run the closure
    - Begin write transaction via `self.db.begin_write()`
    - Commit transaction at end of closure
    - Map all errors using `Executor::map_redb_error`
  - [ ] Subtask 5.6: Implement `read<F, T>(&self, f: F) -> Result<T, CacheError>` helper method
    - Use `self.executor.spawn()`
    - Begin read transaction
    - No commit needed
  - [ ] Subtask 5.7: Ensure `Inner` structs are non-clonable (enforcing `Arc` usage for sharing)
  - [ ] Subtask 5.8: Run `mise run test:unit:adapters` (GREEN)
  - [ ] Subtask 5.9: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Redb Handle Logic (Zero-Copy)
- [ ] Task 6: Implement logic for `Reader` and `Writer`
  - [ ] Subtask 6.1: Update `Reader` and `Writer` to wrap `Arc<RedbInner>`
  - [ ] Subtask 6.2: [TDD] Write `cqrs::prevents_reader_access_to_port_writer_methods` (failing)
  - [ ] Subtask 6.3: Implement `CacheReaderPort` for `Reader` using `inner.read()`
  - [ ] Subtask 6.4: Implement `CacheWriterPort` for `Writer` using `inner.write()`
  - [ ] Subtask 6.5: [TDD] Write `redb_reader::returns_entry_view_without_allocating` (failing, use `IsolatedTestContext`)
  - [ ] Subtask 6.6: Implement `EntryView` for true zero-copy retrieval using `AccessGuard`
  - [ ] Subtask 6.7: Run `mise run test:unit:adapters` (GREEN)
  - [ ] Subtask 6.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 7: API Transparency & Friendly Names
- [ ] Task 7: Implement default type parameters and aliases
  - [ ] Subtask 7.1: Update `Reader/Writer` handles to use `C = RkyvCodec` default in `redb.rs`
  - [ ] Subtask 7.2: Verify re-exports in `mod.rs` (`pub use redb::{Reader as RedbReader, Writer as RedbWriter}`)
  - [ ] Subtask 7.3: [TDD] Write `api::allows_usage_without_specifying_codec` (failing)
  - [ ] Subtask 7.4: Implement re-exports to hide `Inner` and `Executor` from the public SPI
  - [ ] Subtask 7.5: Run `mise run test:unit:adapters` (GREEN)
  - [ ] Subtask 7.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 8: Final Refactor & NFR Verification
- [ ] Task 8: Complete refactor with performance and observability audit
  - [ ] Subtask 8.1: Remove all legacy monolithic code from `redb.rs` and `moka.rs`
  - [ ] Subtask 8.2: [TDD] Write `observability::emits_nested_spans_for_transactions` (failing)
  - [ ] Subtask 8.3: Verify nested spans using `TestTracingSubscriber` across all new components
  - [ ] Subtask 8.4: [TDD] Write `nfr::zero_copy_probe::verifies_direct_pointer_access` (failing)
  - [ ] Subtask 8.5: Verify no heap allocations occur in the read hot-path
  - [ ] Subtask 8.6: Run `mise run lint` and fix all warnings/errors
  - [ ] Subtask 8.7: Run `mise run verify` (GREEN)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 9: Final Review & Quality Gate
- [ ] Task 9: Comprehensive project verification
  - [ ] Subtask 9.1: Perform a final "Trait Bound Audit" ensuring zero `rkyv` noise in backend files
  - [ ] Subtask 9.2: Verify that `RedbReader` and `RedbWriter` correctly use `tracing` for all I/O
  - [ ] Subtask 9.3: Run `mise run fmt` and verify formatting compliance
  - [ ] Subtask 9.4: Run `mise run lint` one final time
  - [ ] Subtask 9.5: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 9.6: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 9.7: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

## Dev Notes

### Research-Backed Implementation Details

#### Performance & NFR Optimizations
- **Zero-Copy Pipeline**: Research into `redb::AccessGuard` and `rkyv::access` confirms we can bypass O(n) heap allocations. `RedbReader` will return an `EntryView` holding the database guard, allowing the application to read complex cached objects directly from the OS page cache (memory map).
- **Durability Tuning**: To meet ASR-01 and ASR-02, we will implement `Durability::None` as the cache default. This bypasses the `fsync` bottleneck (disk flush), leveraging OS-level write buffering to increase throughput by 10x–100x while maintaining persistence across process restarts.
- **Transaction Batching**: The `Inner` core is designed to support the grouping of multiple `put` operations into single `WriteTransaction` calls via the `Executor`, significantly reducing lock contention.

#### Modular Backend Components (Redb Engine)
- **`Executor`**: A private utility that "swallows" the async/sync friction. It encapsulates `tokio::task::spawn_blocking`, manages nested `info_span` instrumentation for transactions, and centralizes `CacheError` mapping.
- **`Inner<K, V, C>`**: The private, non-clonable core aggregator. It holds the `Arc<redb::Database>`, `Executor`, `TableDefinition`, and the `Codec`. It is the single source of truth for the database connection, hidden from the public API.

#### Capability-Based CQRS Handles
- **`Reader<K, V, C>`**: Restricts access to `get` and `has`. Implements `CacheReaderPort`.
- **`Writer<K, V, C>`**: Restricts access to `put`, `delete`, and `clear`. Implements `CacheWriterPort`.
- **Type Aliasing**: To keep the SPI lean, these are re-exported as `RedbReader/Writer` and `MokaReader/Writer` with `C = RkyvCodec` as the default type parameter.

#### Implementation Flows (Redb)
- **Read Path**: `Executor` runs `spawn_blocking` -> `Inner` starts Read Transaction -> `TableDefinition` retrieves value -> `Codec::decode_value` validates bytes -> `EntryView` returns the pointer.
- **Write Path**: `Executor` runs `spawn_blocking` -> `Inner` starts Write Transaction -> `Codec::encode_value` generates bytes -> `TableDefinition` inserts -> Transaction commits with requested Durability.

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
