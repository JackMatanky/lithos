# Story 5.4: Implement Cache Coordinator for Memory/Disk Read-Through and Write-Through

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a system architect ensuring consistency,
I want a `CacheCoordinator` struct that orchestrates memory and disk cache access,
So that cache hits are served fast from memory, misses fall back to disk, and consistency is guaranteed.

## Original Epic Acceptance Criteria

**Given** coordinated caching requires both layers
**When** I implement `CacheCoordinator<K, V>` in `spi/cache/coordinator.rs`
**Then** it wraps:

- `memory: Box<dyn Cache<K, V>>` - fast in-memory cache (typically MokaCache)
- `disk: Box<dyn Cache<K, V>>` - persistent disk cache (typically RedbCache)

**And** constructor `new(memory: Box<dyn Cache<K, V>>, disk: Box<dyn Cache<K, V>>)` accepts pre-configured caches

**Given** read-through caching must be implemented
**When** I implement `get()` for the coordinator
**Then** the flow is:

1. Check memory cache
2. If memory hit: Return value immediately, emit `tracing::event!` at `Level::DEBUG` with "Memory Hit"
3. If memory miss: Check disk cache
4. If disk hit: Backfill memory with the value, emit `tracing::event!` at `Level::INFO` with "Memory Miss / Disk Hit", return value
5. If disk miss: Emit `tracing::event!` at `Level::INFO` with "Disk Miss", return None

**Given** write-through caching must ensure consistency
**When** I implement `put()` for the coordinator
**Then** the flow is:

1. Write to disk first (persistence)
2. If disk write succeeds: Write to memory (in-memory cache)
3. If disk write fails: Return error WITHOUT writing to memory (prevent inconsistency)
4. Emit `tracing::event!` at `Level::DEBUG` with "Cache Write" including key (if serializable)

**And** both layers must succeed or neither is modified (consistency coordination)

**Given** invalidation must affect both layers
**When** I implement `delete()` and `invalidate()`
**Then** both memory and disk caches are invalidated
**And** if either fails, the error is logged but both operations attempt to complete (best effort)
**And** returns true if key existed in either layer

**Given** the coordinator must implement the trait
**When** I implement `Cache<K, V>` for `CacheCoordinator<K, V>`
**Then** all trait methods are satisfied
**And** errors from underlying caches are propagated with layer context

**Given** observability is critical for debugging
**When** I trace coordinator operations
**Then** spans nest correctly: `coordinator` → `memory operation` → `disk operation`
**And** each span includes `cache_layer`, `operation`, and `result` attributes

## TDD Acceptance Criteria (Quality Gates)

**Given** I need a multi-layer cache coordinator
**When** I run `mise run test:unit:adapters coordinator`
**Then** all tests pass using `MockCache` to verify orchestration logic
**And** "Memory Hit" scenario avoids calling Disk layer entirely
**And** "Memory Miss / Disk Hit" scenario verifies memory backfill occurs
**And** "Write-Through" scenario verifies Disk is written before Memory
**And** "Write-Through Failure" scenario verifies Memory is NOT written if Disk fails

**Given** observability is mandatory
**When** I run tests with a tracing subscriber
**Then** spans for coordinator, memory, and disk layers are correctly nested
**And** all cache hit/miss events are emitted with correct level and attributes

**Given** the system must be resilient
**When** underlying caches return errors
**Then** the coordinator propagates them with clear context identifying the failing layer

**Given** I need documentation-driven examples
**When** I run `mise run test:unit:adapters --doc`
**Then** all doc tests demonstrate composing Moka and Redb adapters into a Coordinator

## TDD Tasks / Subtasks

### Phase 1: Test Infrastructure and Scaffolding
- [ ] Task 1: Initialize implementation file and verify module linkage
  - [ ] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/coordinator.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod coordinator;` to `crates/adapters/src/spi/cache/mod.rs`
  - [ ] Subtask 1.3: Write a unit test in `coordinator.rs` under `#[cfg(test)]` that fails to import `CacheCoordinator`
  - [ ] Subtask 1.4: Run `mise run test:unit:adapters coordinator` and verify failure (RED)
  - [ ] Subtask 1.5: Run `mise run lint` and ensure environment is clean
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Struct Definition & Initialization (Test-Driven)
- [ ] Task 2: Implement CacheCoordinator and constructor
  - [ ] Subtask 2.1: Write failing test for `CacheCoordinator::new(memory, disk)` requiring `Box<dyn Cache>` types
  - [ ] Subtask 2.2: Implement `CacheCoordinator` struct with `memory` and `disk` fields
  - [ ] Subtask 2.3: Implement `new` constructor
  - [ ] Subtask 2.4: Run `mise run test:unit:adapters coordinator_init` and verify pass (GREEN)
  - [ ] Subtask 2.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Read-Through Logic (Test-Driven)
- [ ] Task 3: Implement read-through `get` with backfill
  - [ ] Subtask 3.1: Write failing test verifying Memory Hit scenario: Mock Memory returns `Some`, verify Disk is NOT called
  - [ ] Subtask 3.2: Implement `get` logic for Memory Hit
  - [ ] Subtask 3.3: Write failing test for Memory Miss / Disk Hit: verify Disk is called and Memory `put` is triggered (backfill)
  - [ ] Subtask 3.4: Implement backfill logic in `get`
  - [ ] Subtask 3.5: Write failing test for Memory Miss / Disk Miss: verify both are called and result is `None`
  - [ ] Subtask 3.6: Complete `get` implementation
  - [ ] Subtask 3.7: Run `mise run test:unit:adapters coordinator_get` and verify pass (GREEN)
  - [ ] Subtask 3.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Write-Through Logic (Test-Driven)
- [ ] Task 4: Implement write-through `put` with consistency
  - [ ] Subtask 4.1: Write failing test verifying Disk is written BEFORE Memory in `put`
  - [ ] Subtask 4.2: Implement sequential `put` logic
  - [ ] Subtask 4.3: Write failing test for Disk Write Failure: verify Memory is NOT written if Disk fails
  - [ ] Subtask 4.4: Ensure `put` returns error immediately on Disk failure
  - [ ] Subtask 4.5: Run `mise run test:unit:adapters coordinator_put` and verify pass (GREEN)
  - [ ] Subtask 4.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Invalidation Logic (Test-Driven)
- [ ] Task 5: Implement `delete` and `invalidate`
  - [ ] Subtask 5.1: Write failing test verifying both layers are called in `delete`
  - [ ] Subtask 5.2: Implement `delete` calling both layers and returning `true` if either existed
  - [ ] Subtask 5.3: Write failing test for best-effort invalidation: verify both layers are attempted even if one fails
  - [ ] Subtask 5.4: Implement error logging but continued execution for invalidation
  - [ ] Subtask 5.5: Write failing test for `invalidate` delegating to `delete`
  - [ ] Subtask 5.6: Implement `invalidate`
  - [ ] Subtask 5.7: Run `mise run test:unit:adapters coordinator_delete` and verify pass (GREEN)
  - [ ] Subtask 5.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Observability & Tracing (Test-Driven)
- [ ] Task 6: Implement nested tracing and events
  - [ ] Subtask 6.1: Write failing test expecting `"coordinator"` span for all methods
  - [ ] Subtask 6.2: Add `#[tracing::instrument]` to coordinator implementation
  - [ ] Subtask 6.3: Write failing test expecting "Memory Hit" / "Memory Miss" events with correct levels
  - [ ] Subtask 6.4: Add `tracing::event!` calls to the orchestration flow
  - [ ] Subtask 6.5: Run `mise run test:unit:adapters coordinator_tracing` and verify pass (GREEN)
  - [ ] Subtask 6.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 7: Documentation & Doc Testing (Test-Driven)
- [ ] Task 7: Implement module documentation and executable examples
  - [ ] Subtask 7.1: Write failing doc test showing composition of Moka and Redb adapters
  - [ ] Subtask 7.2: Implement doc comments in `coordinator.rs` to make the doc test pass
  - [ ] Subtask 7.3: Add module-level docs explaining Read-Through/Write-Through strategies
  - [ ] Subtask 7.4: Run `mise run test:unit:adapters --doc` and verify all pass (GREEN)
  - [ ] Subtask 7.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 8: Final Quality Gate
- [ ] Task 8: Comprehensive project verification
  - [ ] Subtask 8.1: Run `mise run test:coverage` and verify Coordinator logic is fully exercised
  - [ ] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [ ] Subtask 8.3: Run `mise run lint` one final time
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
- **Hexagonal Architecture**: `CacheCoordinator` is a Domain Service (or Adapter depending on perspective, but acts as a decorator for Ports).
- **Read-Through/Write-Through**: Standard caching patterns implemented to ensure data consistency between volatile and non-volatile layers.
- **Async Safety**: Ensures that async methods correctly coordinate underlying async ports without blocking.

### Technical Requirements
- **Mock-Driven Testing**: Orchestration logic MUST be verified using mocks (`MockCache`) to avoid dependency on concrete implementations.
- **Consistency**: The coordinator is the primary authority for ensuring memory and disk layers remain synchronized.
- **Error Handling**: Must distinguish between transient memory errors and persistent disk errors when possible.

### Library Dependencies
- **async-trait**: For trait implementation.
- **tracing**: For nested instrumentation.
- **mockall**: Mandatory for testing orchestration flows.

### File Structure Requirements
- **Location**: `crates/adapters/src/spi/cache/coordinator.rs`
- **Module Visibility**: `pub(crate)` mod in `cache/mod.rs`.

### Project Structure Notes
- **Alignment**: Complements Moka (5.2) and Redb (5.3) implementations.
- **Conflicts**: None detected. Reuses existing `Cache` trait from 5.1.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Strict adherence.
- **Co-located Tests**: Unit tests live in `coordinator.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:adapters` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: ADR 0016: Caching Strategy]
- [Source: Story 5.1: Define Cache Trait and Error Hierarchy]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology with atomic RED-GREEN subtasks.
- Preserved original Epic ACs.
- Integrated mandatory linting workflows and mise orchestration.
- Ensured co-located tests per Rust project standards.
- Detailed Read-Through/Write-Through orchestration logic for developer.

### File List
- `crates/adapters/src/spi/cache/coordinator.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration.
