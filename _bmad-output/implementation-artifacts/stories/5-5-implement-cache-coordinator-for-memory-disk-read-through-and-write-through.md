# Story 5.5: Implement Cache Coordinator for Memory/Disk Read-Through and Write-Through

Status: ready-for-dev

## Story

As a system architect ensuring consistency and extreme performance,
I want a `CacheCoordinator` split into Reader and Writer handles that orchestrates memory and disk cache access,
So that cache hits are served fast, consistency is guaranteed, and the system follows strict CQRS principles with decoupled background backfill.

## Original Epic Acceptance Criteria

**Given** coordinated caching requires both layers
**When** I implement `CoordinatorInner<K, V>` in `spi/cache/coordinator.rs`
**Then** it leverages the modular `Reader` and `Writer` handles from Story 5.4
**And** it encapsulates:
- `memory_reader: Arc<dyn CacheReaderPort<K, V>>`
- `memory_writer: Arc<dyn CacheWriterPort<K, V>>`
- `disk_reader: Arc<dyn CacheReaderPort<K, V>>`
- `disk_writer: Arc<dyn CacheWriterPort<K, V>>`

**Given** the need for CQRS consistency
**When** I implement `CacheCoordinatorReader` and `CacheCoordinatorWriter`
**Then** they share the `CoordinatorInner` state via `Arc`
**And** the `Reader` handle only implements `CacheReaderPort`
**And** the `Writer` handle only implements `CacheWriterPort`

**Given** read-through caching must be high-performance
**When** a "Memory Miss / Disk Hit" occurs in `Reader::get()`
**Then** the coordinator triggers an **asynchronous backfill** to memory
**And** the backfill is decoupled from the return path using an internal channel
**And** `get()` returns the value to the caller immediately without waiting for the memory write

**Given** write-through caching must ensure consistency
**When** I implement `put()` for the coordinator
**Then** it MUST write to the disk layer first to ensure persistence
**And** it MUST only write to the memory layer if the disk write succeeds
**And** it MUST return an error and PREVENT writing to memory if the disk write fails (ensuring cache consistency)

**Given** invalidation must affect both layers
**When** I implement `delete()` and `invalidate()`
**Then** both memory and disk caches are invalidated in parallel (best effort)
**And** the `Writer` handle manages this coordination logic

**Given** observability is critical for debugging
**When** I trace coordinator operations
**Then** spans nest correctly: `coordinator` → `memory operation` → `disk operation`
**And** backfill events are emitted with `operation = "backfill"` and `status = "triggered"`

## TDD Acceptance Criteria (Quality Gates)

**Given** I need a multi-layer cache coordinator
**When** I run `mise run test:unit:adapters coordinator`
**Then** all tests pass using `MockCacheReaderPort` and `MockCacheWriterPort` to verify orchestration logic
**And** `Reader` handle has NO access to `put` or `clear` methods
**And** `Writer` handle has NO access to `get` or `has` methods

**Given** a "Memory Miss / Disk Hit" scenario
**When** I call `get()` on the coordinator reader
**Then** it returns the disk value immediately
**And** a background task performs the backfill to memory without blocking the return
**And** performance tests verify `get` latency remains sub-millisecond even if memory `put` is slow

**Given** a "Write-Through" scenario
**When** I call `put()` on the coordinator writer
**Then** it verifies Disk is written BEFORE Memory
**And** it ensures Memory is NOT written if Disk fails

**Given** observability is mandatory
**When** I run tests with a tracing subscriber
**Then** spans for coordinator, memory, and disk layers are correctly nested
**And** all cache hit/miss and backfill events are emitted with correct level and attributes

## TDD Tasks / Subtasks

### Phase 1: Test Infrastructure and Scaffolding
- [ ] Task 1: Initialize implementation file and verify module linkage
  - [ ] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/coordinator.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod coordinator;` to `crates/adapters/src/spi/cache/mod.rs`
  - [ ] Subtask 1.3: [TDD] Write `coordinator_init::fails_to_link` (verify failing to import components)
  - [ ] Subtask 1.4: Re-export as `CacheCoordinatorReader` and `CacheCoordinatorWriter` in `crates/adapters/src/spi/cache/mod.rs`
  - [ ] Subtask 1.5: Run `mise run test:unit:adapters coordinator` and verify failure (RED)
  - [ ] Subtask 1.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Struct Definition & Shared State (CQRS Handles)
- [ ] Task 2: Implement `CoordinatorInner` and handles
  - [ ] Subtask 2.1: [TDD] Write `coordinator::shares_inner_state_between_handles` (failing test)
  - [ ] Subtask 2.2: Define `struct CoordinatorInner<K, V>` holding the four split ports from Story 5.4
  - [ ] Subtask 2.3: Define `pub struct Reader<K, V>` and `pub struct Writer<K, V>` as `Arc<CoordinatorInner>` wrappers
  - [ ] Subtask 2.4: Implement `new` constructor that returns `(Reader, Writer)`
  - [ ] Subtask 2.5: Ensure `CoordinatorInner` is non-clonable and private to the module
  - [ ] Subtask 2.6: Run `mise run test:unit:adapters coordinator_init` and verify pass (GREEN)
  - [ ] Subtask 2.7: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Event-Driven Backfill Infrastructure
- [ ] Task 3: Implement internal backfill communication
  - [ ] Subtask 3.1: [TDD] Write `backfill::triggers_asynchronous_memory_put_on_disk_hit` (failing)
  - [ ] Subtask 3.2: Add `tokio::sync::mpsc` channel to `CoordinatorInner` for backfill requests
  - [ ] Subtask 3.3: Implement a private `spawn_backfill_task` helper that listens to the channel
  - [ ] Subtask 3.4: Implement the backfill logic: task calls `memory_writer.put()` and logs results
  - [ ] Subtask 3.5: Run `mise run test:unit:adapters` and verify async trigger (GREEN)
  - [ ] Subtask 3.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Read-Through Logic (CQRS Reader)
- [ ] Task 4: Implement read-through `get` with decoupled async backfill
  - [ ] Subtask 4.1: [TDD] Write `get::returns_memory_hit_immediately` (failing)
  - [ ] Subtask 4.2: Implement `get` logic to return immediately on memory hit (avoiding disk call)
  - [ ] Subtask 4.3: [TDD] Write `get::returns_disk_hit_and_triggers_backfill` (failing)
  - [ ] Subtask 4.4: Implement logic to check disk on memory miss and send (K, V) to the `mpsc` channel on disk hit for background memory update
  - [ ] Subtask 4.5: [TDD] Write `get::returns_none_on_total_miss` (failing)
  - [ ] Subtask 4.6: Implement `has` orchestration checking memory then disk
  - [ ] Subtask 4.7: Run `mise run test:unit:adapters coordinator_get` and verify pass (GREEN)
  - [ ] Subtask 4.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Write-Through Logic (CQRS Writer)
- [ ] Task 5: Implement write-through `put` and parallel invalidation
  - [ ] Subtask 5.1: [TDD] Write `put::writes_to_disk_before_memory` (failing)
  - [ ] Subtask 5.2: Implement sequential write logic ensuring Disk layer is persisted before updating Memory
  - [ ] Subtask 5.3: [TDD] Write `put::aborts_memory_write_on_disk_failure` (failing)
  - [ ] Subtask 5.4: Ensure `put` returns an error immediately and skips memory write if disk fails (maintaining consistency)
  - [ ] Subtask 5.5: [TDD] Write `delete::invalidates_both_layers_in_parallel` (failing)
  - [ ] Subtask 5.6: Implement `delete` and `clear` using `tokio::join!` to minimize invalidation latency
  - [ ] Subtask 5.7: [TDD] Write `invalidate::delegates_to_delete` (failing)
  - [ ] Subtask 5.8: Run `mise run test:unit:adapters coordinator_put` and verify pass (GREEN)
  - [ ] Subtask 5.9: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Observability & NFR Verification
- [ ] Task 6: Finalize nested tracing and performance verification
  - [ ] Subtask 6.1: [TDD] Write `observability::emits_nested_spans_for_coordinator_flow` (failing)
  - [ ] Subtask 6.2: Add `#[tracing::instrument]` to all handle methods and the backfill task
  - [ ] Subtask 6.3: [TDD] Write `performance::get_latency_is_independent_of_backfill_speed` (failing)
  - [ ] Subtask 6.4: Verify that `get()` returns sub-millisecond even if the backfill channel is throttled
  - [ ] Subtask 6.5: Run `mise run test:unit:adapters coordinator_tracing` and verify pass (GREEN)
  - [ ] Subtask 6.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 7: Documentation & Doc Testing
- [ ] Task 7: Implement module documentation and executable examples
  - [ ] Subtask 7.1: [TDD] Write failing doc test showing composition of split handles into a Coordinator
  - [ ] Subtask 7.2: Implement doc comments in `coordinator.rs` to make the doc test pass
  - [ ] Subtask 7.3: Add module-level docs explaining Async Backfill and CQRS benefits
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
  - [ ] Subtask 8.3: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 8.4: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 8.5: Stage and commit all changes with a descriptive conventional commit style message

## Dev Notes

### Implementation Flows

#### Read-Through Flow (CoordinatorReader::get)
1.  Check memory cache via `memory_reader`.
2.  **Memory Hit**: Return value immediately; emit `tracing::event!` at `Level::DEBUG`.
3.  **Memory Miss**: Check disk cache via `disk_reader`.
4.  **Disk Hit**:
    *   Trigger **Asynchronous Backfill** (send `(K, V)` to internal `mpsc` channel).
    *   Emit `tracing::event!` at `Level::INFO` with "Memory Miss / Disk Hit".
    *   Return value immediately to caller.
5.  **Disk Miss**: Emit `tracing::event!` at `Level::INFO` with "Disk Miss"; return `None`.

#### Write-Through Flow (CoordinatorWriter::put)
1.  Attempt write to disk via `disk_writer` (ensures persistence first).
2.  **Disk Success**: Attempt write to memory via `memory_writer`.
3.  **Disk Failure**: Return error immediately; **DO NOT** write to memory (prevents cache inconsistency).
4.  Emit `tracing::event!` at `Level::DEBUG` with "Cache Write".

### Architecture Compliance
- **CQRS Enforcement**: The coordinator is fully split into Reader and Writer components.
- **Event-Driven Backfill**: Prevents memory-write latency from affecting read performance.
- **Hexagonal Architecture**: `Coordinator` handles act as Decorators for the underlying SPI Ports.
- **Async Resource Safety**: Uses `mpsc` channels rather than spawning raw tasks to prevent resource leakage.

### Technical Requirements
- **Mock-Driven Testing**: Orchestration logic MUST be verified using `MockCacheReaderPort` and `MockCacheWriterPort`.
- **Zero-Blocking**: Ensure the backfill task does not block the return path of the Reader.
- **Error Handling**: Must distinguish between transient memory errors and persistent disk errors when possible.

### Library Dependencies
- **async-trait**: For trait implementation.
- **tracing**: For nested instrumentation and backfill tracking.
- **mockall**: Mandatory for testing orchestration flows.
- **tokio**: For `mpsc` and background task management.

### File Structure Requirements
- **Location**: `crates/adapters/src/spi/cache/coordinator.rs`
- **Module Visibility**: `pub(crate)` mod in `cache/mod.rs`.

### Project Structure Notes
- **Alignment**: Complements Moka (5.2) and Redb (5.3) implementations as refactored in 5.4.
- **Conflicts**: None detected. Reuses split ports established in 5.4.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Strict adherence.
- **Co-located Tests**: Unit tests live in `coordinator.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:adapters` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: ADR 0016: Caching Strategy]
- [Source: Story 5.4: Refactor Cache for Modularity and CQRS]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology with atomic RED-GREEN subtasks.
- Preserved original Epic ACs while aligning with the new CQRS/Modular architecture.
- Integrated mandatory linting workflows and mise orchestration with explicit fix requirements.
- Ensured co-located tests per Rust project standards.
- Detailed Read-Through/Write-Through orchestration logic with decoupled async backfill.

### File List
- `crates/adapters/src/spi/cache/coordinator.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration.
