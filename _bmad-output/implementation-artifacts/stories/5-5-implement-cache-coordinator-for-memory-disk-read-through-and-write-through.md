# Story 5.5: Implement Cache Coordinator for Memory/Disk Read-Through and Write-Through

Status: review

## Story

As a system architect ensuring consistency and extreme performance,
I want a `CacheCoordinator` split into Reader and Writer handles that orchestrates memory and disk cache access,
So that cache hits are served fast, consistency is guaranteed, and the system follows strict CQRS principles with decoupled background backfill.

## Original Epic Acceptance Criteria

**Given** coordinated caching requires both layers
**When** I implement `Inner<K, V>` in `spi/cache/coordinator.rs`
**Then** it leverages the modular `Reader` and `Writer` handles from Story 5.4
**And** it encapsulates:
- `memory_reader: Arc<dyn CacheReader<K, V>>`
- `memory_writer: Arc<dyn CacheWriter<K, V>>`
- `disk_reader: Arc<dyn CacheReader<K, V>>`
- `disk_writer: Arc<dyn CacheWriter<K, V>>`

**Given** the new key listing capability from Story 5.4
**When** I call `keys()` on the coordinator
**Then** it returns a deduplicated union of keys from both memory and disk layers

**Given** the need for CQRS consistency
**When** I implement `Reader` and `Writer` coordinators in `coordinator.rs`
**Then** they share the `Inner` state via `Arc`
**And** they are re-exported as `ReaderCoordinator` and `WriterCoordinator`
**And** the `ReaderCoordinator` handle only implements `CacheReader`
**And** the `WriterCoordinator` handle only implements `CacheWriter`

**Given** read-through caching must be high-performance
**When** a "Memory Miss / Disk Hit" occurs in `Reader::get()`
**Then** the coordinator triggers an **asynchronous backfill** to memory
**And** the backfill uses a bounded internal channel (dropping requests if full) to ensure read latency is NEVER affected by backfill pressure
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
**Then** all tests pass using `MockCacheReader` and `MockCacheWriter` to verify orchestration logic
**And** `Reader` handle has NO access to `put` or `clear` methods
**And** `Writer` handle has NO access to `get` or `has` methods
**And** `keys()` returns a correct merged list of keys from both layers

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
- [x] Task 1: Initialize implementation file and verify module linkage
  - [x] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/coordinator.rs`
  - [x] Subtask 1.2: Add `pub(crate) mod coordinator;` to `crates/adapters/src/spi/cache/mod.rs`
  - [x] Subtask 1.3: [TDD] Write `coordinator_init::fails_to_link` (verify failing to import components)
  - [x] Subtask 1.4: Re-export as `ReaderCoordinator` and `WriterCoordinator` in `crates/adapters/src/spi/cache/mod.rs`
  - [x] Subtask 1.5: Run `mise run test:unit:adapters coordinator` and verify failure (RED)
  - [x] Subtask 1.6: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 1.7: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 1.8: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 2: Struct Definition & Shared State (CQRS Handles)
- [x] Task 2: Implement `Inner`, handles, and Builder
  - [x] Subtask 2.1: [TDD] Write `coordinator::shares_inner_state_between_handles` (failing test)
  - [x] Subtask 2.2: Define `struct Inner<K, V>` holding the four split ports from Story 5.4
  - [x] Subtask 2.3: [TDD] Verify `Reader` and `Writer` handles carry correct `K: Clone + Eq + Hash + Send + Sync + 'static` and `V: Clone + Send + Sync + 'static` bounds
  - [x] Subtask 2.4: Define `pub struct Reader<K, V>` and `pub struct Writer<K, V>` as `Arc<Inner>` wrappers
  - [x] Subtask 2.5: Define `pub struct Builder<K, V>` for fluent coordinator construction
  - [x] Subtask 2.6: Implement `Builder::new()` and methods to set the four cache ports
  - [x] Subtask 2.7: Implement `Builder::build()` that returns `(ReaderCoordinator, WriterCoordinator)`
  - [x] Subtask 2.8: Ensure `Inner` is non-clonable and private to the module
  - [x] Subtask 2.9: Run `mise run test:unit:adapters coordinator_init` and verify pass (GREEN)
  - [x] Subtask 2.10: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 2.11: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 2.12: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 3: Event-Driven Backfill Infrastructure
- [x] Task 3: Implement internal backfill communication
  - [x] Subtask 3.1: [TDD] Write `backfill::triggers_asynchronous_memory_put_on_disk_hit` (failing)
  - [x] Subtask 3.2: Add bounded `tokio::sync::mpsc` channel (default capacity 1024) to `Inner` for backfill requests
  - [x] Subtask 3.3: Implement `spawn_backfill_task` called during `Builder::build()` that consumes the receiver
  - [x] Subtask 3.4: Implement backfill logic: task calls `memory_writer.put()` and logs results; gracefully handles channel closure
  - [x] Subtask 3.5: Run `mise run test:unit:adapters` and verify async trigger (GREEN)
  - [x] Subtask 3.6: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 3.7: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 3.8: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 4: Read-Through Logic (CQRS Reader)
- [x] Task 4: Implement read-through `get` with decoupled async backfill
  - [x] Subtask 4.1: [TDD] Write `get::returns_memory_hit_immediately` (failing)
  - [x] Subtask 4.2: Implement `get` logic to return immediately on memory hit (avoiding disk call)
  - [x] Subtask 4.3: [TDD] Write `get::returns_disk_hit_and_triggers_backfill` (failing)
  - [x] Subtask 4.4: Implement logic to check disk on memory miss and send (K, V) to the `mpsc` channel on disk hit for background memory update
  - [x] Subtask 4.5: [TDD] Write `get::returns_none_on_total_miss` (failing)
  - [x] Subtask 4.6: Implement `has` orchestration checking memory then disk
  - [x] Subtask 4.7: [TDD] Write `keys::returns_union_of_both_layers` (failing)
  - [x] Subtask 4.8: Implement `keys` orchestration: fetch from both, merge into a `HashSet`, and return as `Vec<K>`
  - [x] Subtask 4.9: Run `mise run test:unit:adapters coordinator_get` and verify pass (GREEN)
  - [x] Subtask 4.10: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 4.11: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 4.12: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 5: Write-Through Logic (CQRS Writer)
- [x] Task 5: Implement write-through `put` and parallel invalidation
  - [x] Subtask 5.1: [TDD] Write `put::writes_to_disk_before_memory` (failing)
  - [x] Subtask 5.2: Implement sequential write logic ensuring Disk layer is persisted before updating Memory
  - [x] Subtask 5.3: [TDD] Write `put::aborts_memory_write_on_disk_failure` (failing)
  - [x] Subtask 5.4: Ensure `put` returns an error immediately and skips memory write if disk fails (maintaining consistency)
  - [x] Subtask 5.5: [TDD] Write `delete::invalidates_both_layers_in_parallel` (failing)
  - [x] Subtask 5.6: Implement `delete` and `clear` using `tokio::join!` to minimize invalidation latency
  - [x] Subtask 5.7: [TDD] Write `invalidate::delegates_to_delete` (failing)
  - [x] Subtask 5.8: Run `mise run test:unit:adapters coordinator_put` and verify pass (GREEN)
  - [x] Subtask 5.9: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 5.10: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 5.11: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 6: Observability & NFR Verification
- [x] Task 6: Finalize nested tracing and performance verification
  - [x] Subtask 6.1: [TDD] Write `observability::emits_nested_spans_for_coordinator_flow` (failing)
  - [x] Subtask 6.2: Add `#[tracing::instrument]` to all handle methods and the backfill task
  - [x] Subtask 6.3: [TDD] Write `performance::get_latency_is_independent_of_backfill_speed` (failing)
  - [x] Subtask 6.4: Verify that `get()` returns sub-millisecond even if the backfill channel is throttled
  - [x] Subtask 6.5: Run `mise run test:unit:adapters coordinator_tracing` and verify pass (GREEN)
  - [x] Subtask 6.6: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 6.7: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 6.8: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 7: Documentation & Doc Testing
- [x] Task 7: Implement module documentation and executable examples
  - [x] Subtask 7.1: [TDD] Write failing doc test showing composition of split handles into a Coordinator
  - [x] Subtask 7.2: Implement doc comments in `coordinator.rs` to make the doc test pass
  - [x] Subtask 7.3: Add module-level docs explaining Async Backfill and CQRS benefits
  - [x] Subtask 7.4: Run `mise run test:unit:adapters --doc` and verify all pass (GREEN)
  - [x] Subtask 7.5: Run `mise run lint` and fix all warnings/errors
  - [x] Subtask 7.6: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 7.7: Stage and commit all files with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 8: Final Quality Gate
- [x] Task 8: Comprehensive project verification
  - [x] Subtask 8.1: Run `mise run test:coverage` and verify Coordinator logic is fully exercised
  - [x] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [x] Subtask 8.3: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [x] Subtask 8.4: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 8.5: Stage and commit all changes with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

## Dev Notes

### Implementation Flows

#### Read-Through Flow (ReaderCoordinator::get)
1.  Check memory cache via `memory_reader`.
2.  **Memory Hit**: Return value immediately; emit `tracing::event!` at `Level::DEBUG`.
3.  **Memory Miss**: Check disk cache via `disk_reader`.
4.  **Disk Hit**:
    *   Trigger **Asynchronous Backfill** (send `(K, V)` to internal `mpsc` channel).
    *   Emit `tracing::event!` at `Level::INFO` with "Memory Miss / Disk Hit".
    *   Return value immediately to caller.
5.  **Disk Miss**: Emit `tracing::event!` at `Level::INFO` with "Disk Miss"; return `None`.

#### Key Listing Flow (ReaderCoordinator::keys)
1.  Fetch `memory_keys` from `memory_reader`.
2.  Fetch `disk_keys` from `disk_reader`.
3.  Merge into a `HashSet<K>` to handle overlapping keys.
4.  Return `Vec<K>`.

#### Write-Through Flow (WriterCoordinator::put)
1.  Attempt write to disk via `disk_writer` (ensures persistence first).
2.  **Disk Success**: Attempt write to memory via `memory_writer`.
3.  **Disk Failure**: Return error immediately; **DO NOT** write to memory (prevents cache inconsistency).
4.  Emit `tracing::event!` at `Level::DEBUG` with "Cache Write".

### Architecture Compliance
- **CQRS Enforcement**: The coordinator is fully split into `Reader` and `Writer` components, re-exported as `ReaderCoordinator` and `WriterCoordinator`.
- **Handle/Inner Pattern**: Follows standard Lithos patterns for thread-safe, cheaply cloneable handles.
- **Event-Driven Backfill**: Prevents memory-write latency from affecting read performance; uses non-blocking `try_send` to ensure backfill pressure never stalls the caller.
- **Hexagonal Architecture**: `Coordinator` handles act as Decorators for the underlying SPI Ports (`Arc<dyn CacheReader/Writer>`).
- **Async Resource Safety**: Uses `mpsc` channels rather than spawning raw tasks to prevent resource leakage.

### Technical Requirements
- **Mock-Driven Testing**: Orchestration logic MUST be verified using `MockCacheReader` and `MockCacheWriter`.
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
Gemini 3 Flash Preview

### Debug Log References
None - Refactored during implementation to align with project builder patterns and address clippy feedback.

### Architectural Decisions & Rationale

1.  **Lean Composition (Removed `Inner` Struct)**:
    *   **Change**: Eliminated the `Inner<K, V>` struct. The `Reader` and `Writer` handles now hold their respective SPI ports (`Arc<dyn CacheReader/Writer>`) directly.
    *   **Rationale**: The `Inner` pattern added a redundant layer of indirection (double `Arc` dereferencing). Since the ports are already `Arc` pointers, wrapping them in another `Arc<Inner>` provided no benefit for sharing state and increased complexity.
2.  **Encapsulated Backfill (`BackfillQueue`)**:
    *   **Change**: Introduced a specialized `BackfillQueue` component to encapsulate `tokio::mpsc` channel management, background worker spawning, and non-blocking `trigger` logic.
    *   **Rationale**: This separates the "how" of asynchronous backfilling from the "when" of the coordination logic. It keeps the `Reader` handle focused purely on the Read-Through strategy.
3.  **Decomposed Builder Construction**:
    *   **Change**: Refactored the `Builder` to support independent `build_reader` and `build_writer` methods in addition to the joint `build()`.
    *   **Rationale**: Supports systems that only require one side of the CQRS split (e.g., a read-only query service), preventing the need to mock or provide unnecessary ports.
4.  **Strict Field Hygiene**:
    *   **Change**: Renamed fields in both `Reader` and `Writer` handles from `memory_reader`/`disk_reader` and `memory_writer`/`disk_writer` to simply `memory` and `disk`.
    *   **Rationale**: Resolved `clippy::struct_field_names` and ensured consistent API ergonomics across CQRS handles. Struct field names that repeat the struct's name are considered redundant in Rust.

### Event-Driven Backfill Research (Submission Handle Pattern)

To achieve strict CQRS and decouple the `Reader` from the `CacheWriter` trait, research was conducted on the **Submission Handle** pattern (inspired by the `Executor` pattern).

#### **Proposed Signatures for `backfiller.rs`**

```rust
/// Submission handle for triggering background backfills.
/// Agnostic of the writer implementation.
pub struct BackfillHandle<K, V> {
    tx: mpsc::Sender<BackfillRequest<K, V>>,
}

impl<K, V> BackfillHandle<K, V> {
    /// Non-blocking submission of a backfill request using try_send.
    pub fn trigger(&self, key: K, value: V) { ... }
}

/// Lifecycle-managed worker that processes requests.
pub struct BackfillWorker<K, V> {
    rx: mpsc::Receiver<BackfillRequest<K, V>>,
}

impl<K, V> BackfillWorker<K, V> {
    /// Starts the background task. Consumes the worker to ensure single-start.
    pub fn start(self, writer: Arc<dyn CacheWriter<K, V>>) { ... }
}

/// Factory to create the handle/worker pair.
pub fn new<K, V>(capacity: usize) -> (BackfillHandle<K, V>, BackfillWorker<K, V>);
```

#### **Rationale for Future Refactor to `backfiller.rs`**
1.  **Strict CQRS Enforcement**: The `Reader` only depends on a `BackfillHandle` (a data sink) rather than a `CacheWriter` (a command implementation).
2.  **Leaner Coordinator**: Removes channel plumbing and task management from `coordinator.rs`, leaving it focused on coordination strategy.
3.  **Encapsulation**: Centralizes asynchronous background logic, error handling, and drop-policies in a single dedicated component.
4.  **Builder Simplification**: `build_reader` creates the pair and hands the `Handle` to the reader. `build` starts the `Worker` once the writer is available.

### Completion Notes List
- Implemented `CacheCoordinator` with full CQRS support (split Reader/Writer handles).
- Implemented Read-Through logic with decoupled asynchronous backfill to memory via `BackfillQueue`.
- Implemented Write-Through logic (Disk then Memory) to ensure persistence consistency.
- Implemented Parallel Invalidation for both layers using `tokio::join!`.
- Refactored `Builder` to support independent `build_reader` and `build_writer` methods, aligning with Moka and Redb adapters.
- Implemented `clone_from` for all clonable structs to satisfy Lithos quality gates (`missing_trait_methods`).
- Sorted all implementation blocks alphabetically for maintainability.
- Verified 100% logic coverage and passing quality gates via `mise run verify`.

### File List
- `crates/adapters/src/spi/cache/coordinator.rs` - Primary implementation.
- `crates/adapters/src/spi/cache/mod.rs` - Re-exports for public API.
