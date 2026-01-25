# Story 5.5: Implement Performance Benchmarking Suite

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a performance engineer validating cache performance,
I want comprehensive benchmarks using `criterion`,
So that I can verify throughput, latency, and memory usage meet requirements.

## Original Epic Acceptance Criteria

**Given** benchmarking infrastructure exists per ADR 0012
**When** I create `benches/cache_benchmarks.rs` in the adapters crate
**Then** it includes benchmark suites for:

- `MokaCache` standalone operations
- `RedbCache` standalone operations
- `CacheCoordinator` full memory+disk flow

**Given** throughput is critical for LSP scenarios
**When** I benchmark `MokaCache` concurrent operations
**Then** the benchmark:

- Spawns 100 concurrent tasks performing mixed get/put operations
- Runs 1000 operations per second
- Measures p50, p95, p99 latency
- Reports ops/sec throughput

**And** p99 latency is <5ms for get() operations
**And** p99 latency is <10ms for put() operations

**Given** cold start performance matters for CLI
**When** I benchmark `RedbCache` initialization
**Then** the benchmark:

- Measures database open + table creation time
- Measures first read after cold start

**And** database open completes in <10ms

**Given** memory usage must stay within bounds
**When** I benchmark `CacheCoordinator` with large datasets
**Then** the benchmark:

- Caches 10,000 entries of typical size (e.g., 1KB each)
- Measures peak memory usage for memory + disk layers combined

**And** memory usage stays below 100MB (memory layer typically capped at 50MB, disk layer is file-backed)

**Given** scan resistance is a key Moka feature
**When** I benchmark scan scenarios
**Then** the benchmark:

- Performs 10,000 sequential reads (simulating vault scan)
- Followed by 1,000 random reads from a small "hot set"
- Measures cache hit rate for the hot set

**And** TinyLFU policy maintains >80% hit rate for hot data despite scan pollution

**Given** results must be tracked over time
**When** benchmarks are run
**Then** criterion generates statistical reports comparing against baseline
**And** reports are saved to `target/criterion/` for CI integration

## TDD Acceptance Criteria (Quality Gates)

**Given** I need to measure performance
**When** I run `mise run test:bench:adapters`
**Then** all benchmark suites execute successfully
**And** throughput and latency metrics are reported for all cache implementations
**And** performance results are compared against baselines

**Given** concurrency is a key requirement
**When** I run the concurrent Moka benchmark
**Then** it successfully coordinates 100+ tasks without deadlocks or panic
**And** reports consistent results across multiple iterations

**Given** memory bounds are critical
**When** I run the coordinator memory benchmark
**Then** it reports memory usage within the 100MB target for 10k entries

## TDD Tasks / Subtasks

### Phase 1: Infrastructure Setup
- [ ] Task 1: Initialize benchmarking structure in adapters crate
  - [ ] Subtask 1.1: Create directory `crates/adapters/benches/`
  - [ ] Subtask 1.2: Add `[[bench]]` section to `crates/adapters/Cargo.toml` for `cache_benchmarks`
  - [ ] Subtask 1.3: Create empty `crates/adapters/benches/cache_benchmarks.rs`
  - [ ] Subtask 1.4: Write a failing benchmark that cannot import `criterion` or cache types
  - [ ] Subtask 1.5: Run `mise run test:bench:adapters` and verify failure (RED)
  - [ ] Subtask 1.6: Run `mise run lint` and fix all clippy warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Moka Performance Benchmarks (Test-Driven)
- [ ] Task 2: Implement Moka throughput and latency benchmarks
  - [ ] Subtask 2.1: Write failing benchmark for `MokaCache` `get()` and `put()`
  - [ ] Subtask 2.2: Implement standalone Moka benchmarks using `criterion::black_box`
  - [ ] Subtask 2.3: Write failing benchmark for concurrent Moka operations (100 tasks)
  - [ ] Subtask 2.4: Implement concurrent benchmark using `tokio` runtime within criterion
  - [ ] Subtask 2.5: Write failing benchmark for TinyLFU scan resistance
  - [ ] Subtask 2.6: Implement scan resistance benchmark simulating 10k sequential reads
  - [ ] Subtask 2.7: Run `mise run test:bench:adapters` and verify Moka metrics are reported
  - [ ] Subtask 2.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Redb Performance Benchmarks (Test-Driven)
- [ ] Task 3: Implement Redb persistence and initialization benchmarks
  - [ ] Subtask 3.1: Write failing benchmark for Redb initialization (open + table creation)
  - [ ] Subtask 3.2: Implement Redb init benchmark with temporary file cleanup
  - [ ] Subtask 3.3: Write failing benchmark for Redb `get()` with `rkyv` zero-copy
  - [ ] Subtask 3.4: Implement Redb read benchmark verifying zero-copy performance
  - [ ] Subtask 3.5: Write failing benchmark for Redb `put()` transactions
  - [ ] Subtask 3.6: Implement Redb write benchmark with ACID guarantees
  - [ ] Subtask 3.7: Run `mise run test:bench:adapters` and verify Redb metrics are reported
  - [ ] Subtask 3.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Coordinator Benchmarks & Memory Usage (Test-Driven)
- [ ] Task 4: Implement Coordinator flow and memory usage benchmarks
  - [ ] Subtask 4.1: Write failing benchmark for `CacheCoordinator` read-through flow (memory miss/disk hit)
  - [ ] Subtask 4.2: Implement full coordinator flow benchmark
  - [ ] Subtask 4.3: Write failing benchmark for memory usage with 10,000 entries
  - [ ] Subtask 4.4: Implement memory profiling benchmark (using `dhat` or similar if available, otherwise heap estimation)
  - [ ] Subtask 4.5: Verify memory usage stays below 100MB for 10k 1KB entries
  - [ ] Subtask 4.6: Run `mise run test:bench:adapters` and verify Coordinator metrics
  - [ ] Subtask 4.7: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Final Verification & Baseline Generation
- [ ] Task 5: Generate performance baselines and verify quality
  - [ ] Subtask 5.1: Run `mise run test:bench:adapters` to generate initial baselines in `target/criterion/`
  - [ ] Subtask 5.2: Run `mise run lint` one final time
  - [ ] Subtask 5.3: Run `mise run fmt` and verify formatting
  - [ ] Subtask 5.4: Run `mise run verify` to ensure all Lithos quality gates are satisfied
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

## Dev Notes

### Architecture Compliance
- **Performance First**: Benchmarks are first-class citizens in Lithos, following [Source: ADR 0012: Benchmarking Strategy].
- **NFR Verification**: These benchmarks directly validate the Non-Functional Requirements specified in the PRD (under 500ms operations, scale to 1000+ files).
- **Tooling parity**: Use `criterion` as the standard benchmarking tool for all Rust components.

### Technical Requirements
- **Criterion**: Use for statistical analysis and baseline comparison.
- **Tokio Runtime**: Use `tokio` runtime within benchmarks for async cache operations.
- **Black Box**: Use `criterion::black_box` to prevent compiler optimizations from skewing results.
- **Deterministic Setup**: Ensure benchmarks use fixed data sizes and patterns for reproducibility.

### Library Dependencies
- **criterion**: Primary benchmarking framework.
- **tokio**: Async runtime for benchmarks.
- **dhat** (optional): For memory profiling if detailed analysis is needed.

### File Structure Requirements
- **Location**: `crates/adapters/benches/cache_benchmarks.rs`
- **Config**: Updated `Cargo.toml` in adapters crate.

### Project Structure Notes
- **Alignment**: Complements implementation stories 5.1-5.4.
- **No detected conflicts**: Benchmarks live in a separate directory from source code.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Applied to benchmarks - write the measurement logic first, then ensure it reports valid metrics.
- **Mise Orchestration**: Use `mise run test:bench:adapters` for execution.

### References
- [Source: project-context.md#Performance-Benchmarking]
- [Source: ADR 0012: Benchmarking Strategy]
- [Source: PRD Performance Requirements]
- [Source: Story 5.2, 5.3, 5.4 for implementation details]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology for benchmarking code.
- Preserved original Epic ACs for performance targets.
- Integrated mandatory linting workflows and mise orchestration.
- Provided specific tasks for concurrent and memory usage measurements.

### File List
- `crates/adapters/benches/cache_benchmarks.rs` - Benchmark suite.
- `crates/adapters/Cargo.toml` - Configuration.
