# Story 5.2: Implement Moka In-Memory Cache Adapter

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a system architect optimizing for high concurrency,
I want an in-memory `Moka` adapter implementing the `Cache` trait with observability,
So that frequently accessed data is served with sub-millisecond latency and all operations are traced.

## Original Epic Acceptance Criteria

**Given** the `moka` crate dependency is added to `adapters/Cargo.toml`
**When** I implement the `MokaCache<K, V>` struct in `spi/cache/moka.rs`
**Then** it wraps `moka::future::Cache<K, V>` with configuration options:

- `max_capacity: usize` - maximum number of entries
- `time_to_live: Option<Duration>` - TTL for automatic expiration
- `time_to_idle: Option<Duration>` - TTI for idle eviction

**And** the struct provides a builder pattern for configuration

**Given** the adapter must implement the trait
**When** I implement `CacheReader<K, V>` and `CacheWriter<K, V>` for `MokaCache<K, V>`
**Then** all trait methods satisfy the async trait bounds
**And** `CacheReader::get()` returns `None` for cache misses, `Some(V)` for hits
**And** `CacheReader::has()` checks existence without cloning the value
**And** `CacheWriter::clear()` invalidates all entries in the Moka cache
**And** `CacheWriter::delete()` removes entries and returns true if key existed
**And** `CacheWriter::invalidate()` delegates to `delete()` for semantic clarity
**And** `CacheWriter::put()` stores values respecting TTL/TTI policies

**Given** observability is required per project standards
**When** I instrument all public methods
**Then** each method is decorated with `#[tracing::instrument(skip(self, key, value), level = "debug")]`
**And** `clear()` emits events with `cache_layer = "memory"`, `operation = "clear"`
**And** `delete()` emits events with `cache_layer = "memory"`, `operation = "delete"`, `existed = true/false`
**And** `get()` emits a `tracing::event!` with attributes:
- `cache_layer = "memory"`
- `operation = "get"`
- `hit = true/false`
**And** `has()` emits events with `cache_layer = "memory"`, `operation = "has"`, `exists = true/false`
**And** `put()` emits events with `cache_layer = "memory"`, `operation = "put"`

**Given** Moka's TinyLFU policy must be utilized
**When** I configure the cache
**Then** the default eviction policy is TinyLFU (Moka's default)
**And** documentation explains TinyLFU prevents scan pollution during vault indexing

**Given** the adapter must handle errors gracefully
**When** Moka operations fail (rare, but possible with listeners)
**Then** errors are mapped to `CacheError::BackendError` with descriptive context

## TDD Acceptance Criteria (Quality Gates)

**Given** I need a high-performance in-memory cache
**When** I run `mise run test:unit:core moka_cache`
**Then** all tests pass with all public components validated
**And** `get`, `put`, `delete` operations demonstrate sub-millisecond latency
**And** cache hit/miss behavior matches expected Moka semantics
**And** TTL/TTI expiration is verified through deterministic time control

**Given** observability is critical for cache performance
**When** I run tests with a tracing subscriber
**Then** all operations emit correct `tracing` events with required attributes
**And** `instrument` spans include proper skip parameters for efficiency

**Given** I need to prevent scan pollution
**When** I perform sequential scan tests
**Then** the TinyLFU policy protects the "hot" set from eviction
**And** cache hit rate for hot data remains stable during vault indexing simulation

**Given** I need documentation-driven examples
**When** I run `mise run test:unit:core --doc`
**Then** all doc tests demonstrate proper builder pattern and cache usage
**And** examples demonstrate async execution within tokio runtime

## TDD Tasks / Subtasks

### Phase 0: Dependency Management
- [x] Task 0: Add required dependencies to `crates/adapters/Cargo.toml`
  - [x] Subtask 0.1: Add `moka = { version = "0.12", features = ["future"] }` to `[dependencies]`
  - [x] Subtask 0.2: Run `cargo check -p lithos-adapters` to verify dependency resolution

### Phase 1: Test Infrastructure and Scaffolding
- [x] Task 1: Initialize implementation file and verify module linkage
  - [x] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/moka.rs`
  - [x] Subtask 1.2: Add `pub(crate) mod moka;` to `crates/adapters/src/spi/cache/mod.rs`
  - [x] Subtask 1.3: Write a unit test in `moka.rs` under `#[cfg(test)]` that fails to import `MokaCache`
  - [x] Subtask 1.4: Write a unit test in `moka.rs` that fails to import `MokaCacheBuilder`
  - [x] Subtask 1.5: Run `mise run test:unit:core moka` and verify both tests fail with "unresolved import" (RED)
  - [x] Subtask 1.6: Run `mise run lint` and ensure environment is clean

### Phase 2: Struct Definition & Configuration
- [x] Task 2: Implement minimal MokaCache and verify builder initialization
  - [x] Subtask 2.1: Write failing test expecting `MokaCache::build()` to return a builder instance
  - [x] Subtask 2.2: Implement `Cache` and `Builder` structs with minimal `build()` method
  - [x] Subtask 2.3: Write failing test requiring builder to have a `max_capacity(usize)` method
  - [x] Subtask 2.4: Implement `max_capacity` method returning `&mut Self` (fluent API)
  - [x] Subtask 2.5: Write failing test requiring builder to have a `time_to_live(Duration)` method
  - [x] Subtask 2.6: Implement `time_to_live` method in builder
  - [x] Subtask 2.7: Write failing test requiring builder to have a `time_to_idle(Duration)` method
  - [x] Subtask 2.8: Implement `time_to_idle` method in builder
  - [x] Subtask 2.9: Write failing test expecting builder `.new()` to return `Result<Cache, CacheError>`
  - [x] Subtask 2.10: Implement `new()` by initializing an internal `moka::future::Cache` with configured parameters; ensure defaults (e.g., 10,000 capacity, None for durations) are applied if builder methods were not called.
  - [x] Subtask 2.11: Run `mise run test:unit:core moka_config` and verify all configuration tests pass (GREEN)
  - [x] Subtask 2.12: Run `mise run lint` and fix all warnings/errors

### Phase 3: Trait Implementation - Core Operations
- [x] Task 3: Implement Cache trait methods and verify basic storage
  - [x] Subtask 3.1: Write failing test that implements `Cache<String, String>` for `MokaCache` and calls `get`
  - [x] Subtask 3.2: Implement `get` method using `moka_cache.get()` and verify it returns `None` for new cache
  - [x] Subtask 3.3: Write failing test that calls `put("key", "val")` then `get("key")`
  - [x] Subtask 3.4: Implement `put` method using `moka_cache.insert()`
  - [x] Subtask 3.5: Write failing test requiring `delete("key")` to return `true` if item existed
  - [x] Subtask 3.6: Implement `delete` using `moka_cache.remove()` and verify return value logic
  - [x] Subtask 3.7: Write failing test requiring `has("key")` to check existence
  - [x] Subtask 3.8: Implement `has` using `moka_cache.contains_key()`
  - [x] Subtask 3.9: Write failing test requiring `clear()` to remove all items
  - [x] Subtask 3.10: Implement `clear` using `moka_cache.invalidate_all()`
  - [x] Subtask 3.11: Write failing test requiring `invalidate("key")` to remove the item
  - [x] Subtask 3.12: Implement `invalidate` by delegating to `delete`
  - [x] Subtask 3.13: Write failing test verifying generic bounds `K: Clone + Eq + Hash + Send + Sync + 'static` and `V: Clone + Send + Sync + 'static`
  - [x] Subtask 3.14: Apply trait bounds `K: Clone + Eq + Hash + Send + Sync + 'static` and `V: Clone + Send + Sync + 'static` to the `MokaCache` struct and `Cache` implementation.
  - [x] Subtask 3.15: Run `mise run test:unit:core moka_trait` and verify all operation tests pass (GREEN)
  - [x] Subtask 3.16: Run `mise run lint` and fix all warnings/errors

### Phase 4: Observability & Tracing
- [x] Task 4: Implement tracing instrumentation and verify event emission
  - [x] Subtask 4.1: Write failing test using `tracing-test` (or similar) to expect an instrumented span for `get()`
  - [x] Subtask 4.2: Add `#[tracing::instrument(skip(self), level = "debug")]` to `get` method
  - [x] Subtask 4.3: Write failing test that expects a `tracing` event with `hit = false` on cache miss
  - [x] Subtask 4.4: Add `tracing::event!` to `get` with `cache_layer = "memory"`, `operation = "get"`, and `hit` status
  - [x] Subtask 4.5: Write failing test expecting instrumentation span for `put()` skipping the `value` field
  - [x] Subtask 4.6: Add `#[tracing::instrument(skip(self, value), level = "debug")]` to `put`
  - [x] Subtask 4.7: Write failing test expecting instrumentation for `delete()` and `invalidate()`
  - [x] Subtask 4.8: Add proper instrumentation and events to `delete` and `invalidate`
  - [x] Subtask 4.9: Run `mise run test:unit:core moka_tracing` and verify pass (GREEN)
  - [x] Subtask 4.10: Run `mise run lint` and fix all warnings/errors

### Phase 5: Eviction & Expiration
- [x] Task 5: Verify TTL/TTI and capacity eviction policies
  - [x] Subtask 5.1: Write failing test for TTL: put item with 10ms TTL, wait 20ms, verify `get` returns `None`
  - [x] Subtask 5.2: Ensure Moka initialization in `new()` correctly respects the builder's `time_to_live`
  - [x] Subtask 5.3: Write failing test for TTI: put item, wait, get item (reset TTI), wait again, verify still exists
  - [x] Subtask 5.4: Ensure builder's `time_to_idle` is correctly passed to Moka backend
  - [x] Subtask 5.5: Write failing test for `max_capacity`: put 100 items into cache with capacity 10, verify size <= 10
  - [x] Subtask 5.6: Write failing test for TinyLFU: Access 'Hot Key' 20 times, then access 100 'Scan Keys' once each; verify 'Hot Key' is not evicted by the scan (Moka's TinyLFU behavior).
  - [x] Subtask 5.7: Run `mise run test:unit:core moka_eviction` and verify pass (GREEN)
  - [x] Subtask 5.8: Run `mise run lint` and fix all warnings/errors

### Phase 6: Error Handling
- [x] Task 6: Map internal Moka states to CacheError
  - [x] Subtask 6.1: Write failing test for error mapping: Implement a test case for `From<moka::Error> for CacheError` or simulate a backend failure in `new()` to verify `CacheError::BackendError` propagation.
  - [x] Subtask 6.2: Ensure method returns `CacheError::BackendError` with descriptive message
  - [x] Subtask 6.3: Run `mise run test:unit:core moka_errors` and verify pass (GREEN)
  - [x] Subtask 6.4: Run `mise run lint` and fix all warnings/errors

### Phase 7: Documentation & Doc Testing
- [x] Task 7: Implement module documentation and executable examples
  - [x] Subtask 7.1: Write failing doc test showing basic builder setup and `get`/`put` usage
  - [x] Subtask 7.2: Implement doc comments in `moka.rs` to make the doc test pass
  - [x] Subtask 7.3: Write failing doc test showing how TinyLFU prevents scan pollution (textual explanation + example)
  - [x] Subtask 7.4: Add module-level docs explaining eviction policies and async safety
  - [x] Subtask 7.5: Run `mise run test:unit:core --doc` and verify all pass (GREEN)
  - [x] Subtask 7.6: Run `mise run lint` and fix all warnings/errors

### Phase 8: Final Quality Gate
- [x] Task 8: Comprehensive project verification
  - [x] Subtask 8.1: Run `mise run test:coverage` and verify `MokaCache` logic is fully exercised
  - [x] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [x] Subtask 8.3: Run `mise run lint` one final time to verify zero warnings/errors
  - [x] Subtask 8.4: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [x] Subtask 8.5: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 8.6: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 10: CQRS Refactor (Architectural Integrity)
- [x] Task 10: Implement split traits for MokaCache
  - [x] Subtask 10.1: Update `moka.rs` to implement `CacheReader` and `CacheWriter` separately
  - [x] Subtask 10.2: Update doc tests to use split trait imports
  - [x] Subtask 10.3: Verify all tests pass with split traits

## Dev Notes

### Architecture Compliance
- **Hexagonal Architecture**: `MokaCache` is an Adapter implementing the `Cache` Port in the adapters layer.
- **Port/Adapter Pattern**: Follows `[Subject][Technology]Adapter` naming convention (`MokaCache` -> `CacheMokaAdapter` pattern simplified to `MokaCache` for ergonomics as allowed).
- **Async Resource Safety**: Uses `moka::future::Cache` which is optimized for Tokio. Ensures no blocking I/O on async threads.
- **Idiomatic Builder Pattern**: Inside `moka.rs`, the structs are named `Cache` and `Builder`. `MokaCache::builder()` returns a `Builder`, and `Builder::build()` finalizes the construction. They are re-exported as `MokaCache` and `MokaCacheBuilder` in the parent module.

### Technical Requirements
- **High Concurrency**: Leverage Moka's lock-free read operations and high-concurrency write design.
- **Latency**: Sub-millisecond targets for `get()` operations.
- **Observability**: Mandatory `tracing` spans and events for all cache interactions.
- **Eviction**: TinyLFU policy MUST be maintained to prevent scan pollution during vault indexing.

### Library Dependencies
- **moka**: `future` feature enabled for async support.
- **tracing**: For instrumentation and event logging.
- **async-trait**: Required for implementing the `Cache` trait.
- **thiserror**: For error mapping to `CacheError`.

### File Structure Requirements
- **Location**: `crates/adapters/src/spi/cache/moka.rs`
- **Module Visibility**: `pub` mod in `cache/mod.rs` with descriptive aliases.

### Project Structure Notes
- **Alignment**: Consistent with `spi/cache/mod.rs` trait definition.
- **Conflicts**: None detected. Reuses `tracing` patterns from Epic 4.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Never write code without a failing test.
- **Co-located Tests**: All unit tests MUST live in `moka.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:core` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: project-context.md#Error--Diagnostic-Standards]
- [Source: Epic 5 Implementation Notes]
- [Source: ADR 013 (Caching - Superseded): Caching Strategy]
- [Source: Story 5.1: Define Cache Trait and Error Hierarchy]

## Dev Agent Record

### Agent Model Used
google/gemini-3-flash-preview

### Debug Log References
- Refactored internal naming to `Cache` and `Builder` for module-level idiomatic purity.
- Implemented `Default` for `Builder` with production defaults (10k capacity).
- Finalized builder API: `MokaCache::builder() -> Builder` and `Builder::build() -> Result<MokaCache, CacheError>`.
- Satisfied all strict clippy lints including alphabetical item ordering (`clear`, `delete`, `get`, `has`, `invalidate`, `put`).
- Removed `Debug` bounds from `K` and `V` to align with port trait; added `skip` to `tracing::instrument`.
- Enabled shared ownership by deriving `Clone` for `Cache` and `Builder`.
- Instrumented all methods (`clear`, `delete`, `get`, `has`, `invalidate`, `put`) with full tracing coverage.
- Added explicit TinyLFU unit test in `tests` module.

### Completion Notes List
- Applied TDD-optimized methodology with 56+ atomic subtasks.
- Preserved original Epic ACs while refining implementation details for Rust idiomaticity.
- Integrated linting requirements and mise orchestration.
- Ensured co-located tests per Rust project standards.
- Optimized for LLM developer agent consumption.

### File List
- `crates/adapters/src/spi/cache/moka.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration and re-exports.
- `_bmad-output/implementation-artifacts/stories/5-2-implement-moka-in-memory-cache-adapter.md` - Story file.
