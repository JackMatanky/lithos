# Story 5.2: Implement Moka In-Memory Cache Adapter

Status: ready-for-dev

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
**When** I implement `Cache<K, V>` for `MokaCache<K, V>`
**Then** all trait methods satisfy the async trait bounds
**And** `get()` returns `None` for cache misses, `Some(V)` for hits
**And** `put()` stores values respecting TTL/TTI policies
**And** `delete()` removes entries and returns true if key existed
**And** `invalidate()` delegates to `delete()` for semantic clarity

**Given** observability is required per project standards
**When** I instrument all public methods
**Then** each method is decorated with `#[tracing::instrument(skip(self, value), level = "debug")]`
**And** `get()` emits a `tracing::event!` with attributes:

- `cache_layer = "memory"`
- `operation = "get"`
- `hit = true/false`
  **And** `put()` emits events with `cache_layer = "memory"`, `operation = "put"`
  **And** `delete()` emits events with `cache_layer = "memory"`, `operation = "delete"`, `existed = true/false`

**Given** Moka's TinyLFU policy must be utilized
**When** I configure the cache
**Then** the default eviction policy is TinyLFU (Moka's default)
**And** documentation explains TinyLFU prevents scan pollution during vault indexing

**Given** the adapter must handle errors gracefully
**When** Moka operations fail (rare, but possible with listeners)
**Then** errors are mapped to `CacheError::BackendError` with descriptive context

## TDD Acceptance Criteria (Quality Gates)

**Given** I need a high-performance in-memory cache
**When** I run `mise run test:unit:adapters moka_cache`
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
**When** I run `mise run test:unit:adapters --doc`
**Then** all doc tests demonstrate proper builder pattern and cache usage
**And** examples demonstrate async execution within tokio runtime

## TDD Tasks / Subtasks

### Phase 1: Test Infrastructure and Scaffolding
- [ ] Task 1: Initialize implementation file and verify module linkage
  - [ ] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/moka.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod moka;` to `crates/adapters/src/spi/cache/mod.rs`
  - [ ] Subtask 1.3: Write a unit test in `moka.rs` under `#[cfg(test)]` that fails to import `MokaCache`
  - [ ] Subtask 1.4: Write a unit test in `moka.rs` that fails to import `MokaCacheBuilder`
  - [ ] Subtask 1.5: Run `mise run test:unit:adapters moka` and verify both tests fail with "unresolved import" (RED)
  - [ ] Subtask 1.6: Run `mise run lint` and ensure environment is clean
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Struct Definition & Configuration (Test-Driven)
- [ ] Task 2: Implement minimal MokaCache and verify builder initialization
  - [ ] Subtask 2.1: Write failing test expecting `MokaCache::builder()` to return a builder instance
  - [ ] Subtask 2.2: Implement `MokaCache` and `MokaCacheBuilder` structs with minimal `builder()` method
  - [ ] Subtask 2.3: Write failing test requiring builder to have a `max_capacity(usize)` method
  - [ ] Subtask 2.4: Implement `max_capacity` method returning `&mut Self` (fluent API)
  - [ ] Subtask 2.5: Write failing test requiring builder to have a `time_to_live(Duration)` method
  - [ ] Subtask 2.6: Implement `time_to_live` method in builder
  - [ ] Subtask 2.7: Write failing test requiring builder to have a `time_to_idle(Duration)` method
  - [ ] Subtask 2.8: Implement `time_to_idle` method in builder
  - [ ] Subtask 2.9: Write failing test expecting builder `.build()` to return `Result<MokaCache, CacheError>`
  - [ ] Subtask 2.10: Implement `build()` by initializing an internal `moka::future::Cache` with configured parameters
  - [ ] Subtask 2.11: Run `mise run test:unit:adapters moka_config` and verify all configuration tests pass (GREEN)
  - [ ] Subtask 2.12: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Trait Implementation - Core Operations (Test-Driven)
- [ ] Task 3: Implement Cache trait methods and verify basic storage
  - [ ] Subtask 3.1: Write failing test that implements `Cache<String, String>` for `MokaCache` and calls `get`
  - [ ] Subtask 3.2: Implement `get` method using `moka_cache.get()` and verify it returns `None` for new cache
  - [ ] Subtask 3.3: Write failing test that calls `put("key", "val")` then `get("key")`
  - [ ] Subtask 3.4: Implement `put` method using `moka_cache.insert()`
  - [ ] Subtask 3.5: Write failing test requiring `delete("key")` to return `true` if item existed
  - [ ] Subtask 3.6: Implement `delete` using `moka_cache.remove()` and verify return value logic
  - [ ] Subtask 3.7: Write failing test requiring `invalidate("key")` to remove the item
  - [ ] Subtask 3.8: Implement `invalidate` by delegating to `delete`
  - [ ] Subtask 3.9: Write failing test verifying generic bounds `K: Clone + Eq + Hash + Send + Sync + 'static` and `V: Clone + Send + Sync + 'static`
  - [ ] Subtask 3.10: Ensure implementation correctly handles generic types with specified bounds
  - [ ] Subtask 3.11: Run `mise run test:unit:adapters moka_trait` and verify all operation tests pass (GREEN)
  - [ ] Subtask 3.12: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Observability & Tracing (Test-Driven)
- [ ] Task 4: Implement tracing instrumentation and verify event emission
  - [ ] Subtask 4.1: Write failing test using `tracing-test` (or similar) to expect an instrumented span for `get()`
  - [ ] Subtask 4.2: Add `#[tracing::instrument(skip(self), level = "debug")]` to `get` method
  - [ ] Subtask 4.3: Write failing test that expects a `tracing` event with `hit = false` on cache miss
  - [ ] Subtask 4.4: Add `tracing::event!` to `get` with `cache_layer = "memory"`, `operation = "get"`, and `hit` status
  - [ ] Subtask 4.5: Write failing test expecting instrumentation span for `put()` skipping the `value` field
  - [ ] Subtask 4.6: Add `#[tracing::instrument(skip(self, value), level = "debug")]` to `put`
  - [ ] Subtask 4.7: Write failing test expecting instrumentation for `delete()` and `invalidate()`
  - [ ] Subtask 4.8: Add proper instrumentation and events to `delete` and `invalidate`
  - [ ] Subtask 4.9: Run `mise run test:unit:adapters moka_tracing` and verify pass (GREEN)
  - [ ] Subtask 4.10: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Eviction & Expiration (Test-Driven)
- [ ] Task 5: Verify TTL/TTI and capacity eviction policies
  - [ ] Subtask 5.1: Write failing test for TTL: put item with 10ms TTL, wait 20ms, verify `get` returns `None`
  - [ ] Subtask 5.2: Ensure Moka initialization in `build()` correctly respects the builder's `time_to_live`
  - [ ] Subtask 5.3: Write failing test for TTI: put item, wait, get item (reset TTI), wait again, verify still exists
  - [ ] Subtask 5.4: Ensure builder's `time_to_idle` is correctly passed to Moka backend
  - [ ] Subtask 5.5: Write failing test for `max_capacity`: put 100 items into cache with capacity 10, verify size <= 10
  - [ ] Subtask 5.6: Write failing test for TinyLFU: simulate a scan of many items, then verify a "hot" item was not evicted
  - [ ] Subtask 5.7: Run `mise run test:unit:adapters moka_eviction` and verify pass (GREEN)
  - [ ] Subtask 5.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Error Handling (Test-Driven)
- [ ] Task 6: Map internal Moka states to CacheError
  - [ ] Subtask 6.1: Write failing test that simulates an error condition (e.g., failed resource allocation if mockable)
  - [ ] Subtask 6.2: Ensure method returns `CacheError::BackendError` with descriptive message
  - [ ] Subtask 6.3: Run `mise run test:unit:adapters moka_errors` and verify pass (GREEN)
  - [ ] Subtask 6.4: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 7: Documentation & Doc Testing (Test-Driven)
- [ ] Task 7: Implement module documentation and executable examples
  - [ ] Subtask 7.1: Write failing doc test showing basic builder setup and `get`/`put` usage
  - [ ] Subtask 7.2: Implement doc comments in `moka.rs` to make the doc test pass
  - [ ] Subtask 7.3: Write failing doc test showing how TinyLFU prevents scan pollution (textual explanation + example)
  - [ ] Subtask 7.4: Add module-level docs explaining eviction policies and async safety
  - [ ] Subtask 7.5: Run `mise run test:unit:adapters --doc` and verify all pass (GREEN)
  - [ ] Subtask 7.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 8: Final Quality Gate
- [ ] Task 8: Comprehensive project verification
  - [ ] Subtask 8.1: Run `mise run test:coverage` and verify `MokaCache` logic is fully exercised
  - [ ] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [ ] Subtask 8.3: Run `mise run lint` one final time to verify zero warnings/errors
  - [ ] Subtask 8.4: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 8.5: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 8.6: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

## Dev Notes

### Architecture Compliance
- **Hexagonal Architecture**: `MokaCache` is an Adapter implementing the `Cache` Port in the adapters layer.
- **Port/Adapter Pattern**: Follows `[Subject][Technology]Adapter` naming convention (`MokaCache` -> `CacheMokaAdapter` pattern simplified to `MokaCache` for ergonomics as allowed).
- **Async Resource Safety**: Uses `moka::future::Cache` which is optimized for Tokio. Ensures no blocking I/O on async threads.

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
- **Module Visibility**: `pub(crate)` mod in `cache/mod.rs`.

### Project Structure Notes
- **Alignment**: Consistent with `spi/cache/mod.rs` trait definition.
- **Conflicts**: None detected. Reuses `tracing` patterns from Epic 4.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Never write code without a failing test.
- **Co-located Tests**: All unit tests MUST live in `moka.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:adapters` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: project-context.md#Error--Diagnostic-Standards]
- [Source: Epic 5 Implementation Notes]
- [Source: ADR 0016: Caching Strategy]
- [Source: Story 5.1: Define Cache Trait and Error Hierarchy]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology with 51+ atomic subtasks.
- Preserved original Epic ACs.
- Integrated linting requirements and mise orchestration.
- Ensured co-located tests per Rust project standards.
- Optimized for LLM developer agent consumption.

### File List
- `crates/adapters/src/spi/cache/moka.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration.
