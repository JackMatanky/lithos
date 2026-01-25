# Story 5.6: Review Epic 5 Test Suite

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 5 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before adapter integration.

## Original Epic Acceptance Criteria

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guides during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 5 public components are implemented (Cache trait, MokaCache, RedbCache, Coordinator)
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** each `CacheError` variant has a test case ensuring proper error propagation

**Given** all Epic 5 public APIs are documented
**When** I verify doc test coverage
**Then** all public components (traits, structs, enums, methods) have runnable doc tests in `# Examples` sections demonstrating usage
**And** doc tests cover both success cases and error handling
**And** doc tests compile and pass when run via `cargo test --doc`

**Given** all Epic 5 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate:

- False positives (tests that pass but don't validate behavior)
- Redundant tests (duplicate coverage)
- Inadequate edge case coverage (error paths, boundary conditions)

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests validate actual cache behavior vs implementation details
**And** tests verify contract adherence (trait semantics) not internal state

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 5 suite

**Given** concurrency is critical per ADR 0016
**When** I test MokaCache and Coordinator
**Then** tests include concurrent read/write scenarios with 100+ spawned tasks
**And** tests verify no data races or deadlocks under load using `tokio::test` with multi-threaded runtime

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify:

- Tests use proper fixtures (test data builders, sample types)
- Tests avoid flaky behavior (no timing dependencies, no hard-coded sleep)
- Test intent is clear (descriptive names, Given/When/Then structure in comments)

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code
**And** complex test scenarios have inline documentation explaining setup

**Given** mocking is critical for isolation
**When** I review mock usage
**Then** `MockCache` is used appropriately in coordinator tests to isolate memory/disk behavior
**And** no manual mocks exist (all use `#[mockall::automock]`)

**Given** documentation quality is critical
**When** I review all doc comments
**Then** every public component has:

- Precise `///` doc comments explaining purpose and behavior
- Well-written doc tests in `# Examples` sections
- Error cases documented with `# Errors` sections where applicable
- Panic conditions documented with `# Panics` sections where applicable

**And** doc tests demonstrate realistic usage patterns
**And** doc comments follow project standards from `project-context.md`

**Given** RedbCache persistence must be validated
**When** I test persistence behavior
**Then** integration tests verify:

- Value survives process restart (create cache, write, drop, recreate, read)
- rkyv serialization round-trips correctly for complex types
- Metadata is preserved across reads/writes

## TDD Acceptance Criteria (Quality Gates)

**Given** I am performing an adversarial review
**When** I run `mise run verify`
**Then** all quality gates pass including fmt, lint, and tests
**And** `mise run test:coverage` confirms all public components in `spi/cache/` are exercised
**And** all doc tests pass across the adapters crate

**Given** I am identifying test suite gaps
**When** I perform manual code mutation or logic inversion
**Then** existing tests fail as expected (verifying test sensitivity)
**And** no "passing but empty" tests remain in the suite

**Given** I am optimizing for maintainability
**When** I check for `#[expect]` usage in tests
**Then** every instance includes a valid reason according to Section 8 of the developer guide
**And** no unnecessary `#[allow]` attributes exist

## TDD Tasks / Subtasks

### Phase 1: Standard Compliance Audit
- [ ] Task 1: Audit test suite against project standards
  - [ ] Subtask 1.1: Write failing test verifying all unit tests are co-located in `#[cfg(test)]` modules (no separate `tests/` files for units)
  - [ ] Subtask 1.2: Audit `moka.rs`, `redb.rs`, `coordinator.rs`, and `errors.rs` for test co-location
  - [ ] Subtask 1.3: Write failing test that checks for `#[tokio::test(flavor = "multi_thread")]` in concurrent test scenarios
  - [ ] Subtask 1.4: Verify all concurrent Read/Write tests use multi-threaded flavor
  - [ ] Subtask 1.5: Run `mise run lint` and fix all clippy warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Public Component & Coverage Review (Test-Driven)
- [ ] Task 2: Verify all public components have behavior-validated tests
  - [ ] Subtask 2.1: Write failing test that scans for public items without corresponding tests
  - [ ] Subtask 2.2: Identify and add missing behavior tests for any public method in `Cache`, `MokaCache`, `RedbCache`, or `CacheCoordinator`
  - [ ] Subtask 2.3: Write failing test verifying `CacheError` variants are all exercised in error propagation paths
  - [ ] Subtask 2.4: Ensure 100% variant coverage for `CacheError` in unit tests
  - [ ] Subtask 2.5: Run `mise run test:coverage` and identify any logic gaps in `spi/cache/`
  - [ ] Subtask 2.6: Implement missing tests to cover identified logic gaps
  - [ ] Subtask 2.7: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Adversarial Logic & Edge Case Verification (Test-Driven)
- [ ] Task 3: Perform adversarial logic review and edge case expansion
  - [ ] Subtask 3.1: Write failing test for `RedbCache` persistence using `IsolatedTestContext` to verify cross-restart survival
  - [ ] Subtask 3.2: Implement logic-verifying tests that use mutation (e.g. changing expected values) to ensure tests fail
  - [ ] Subtask 3.3: Write failing test for Redb table isolation: ensure writing to "table A" does not appear in "table B" even with same keys
  - [ ] Subtask 3.4: Write failing test for `MokaCache` scan resistance: verify hot data is NOT evicted during high-volume sequential reads
  - [ ] Subtask 3.5: Run `mise run test:unit:adapters` and verify all adversarial tests pass
  - [ ] Subtask 3.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Mock Usage & Orchestration Review (Test-Driven)
- [ ] Task 4: Critique and improve mock-based orchestration tests
  - [ ] Subtask 4.1: Write failing test verifying `MockCache` is used for ALL `CacheCoordinator` unit tests (no concrete adapters)
  - [ ] Subtask 4.2: Replace any manual mocks or concrete adapters in coordinator tests with `automock` expectations
  - [ ] Subtask 4.3: Write failing test verifying `put` write-through ordering: Disk FIRST, then Memory
  - [ ] Subtask 4.4: Verify `delete` invalidates both layers even if one fails (best effort)
  - [ ] Subtask 4.5: Run `mise run test:unit:adapters coordinator` and verify pass
  - [ ] Subtask 4.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Documentation & Example Validation (Test-Driven)
- [ ] Task 5: Verify doc test coverage and example quality
  - [ ] Subtask 5.1: Write failing test verifying all public traits and structs have runnable `# Examples`
  - [ ] Subtask 5.2: Audit all `///` comments for adherence to `project-context.md` standards
  - [ ] Subtask 5.3: Write failing test verifying every `Result`-returning function has an `# Errors` section
  - [ ] Subtask 5.4: Add missing doc sections and verify via `cargo test --doc`
  - [ ] Subtask 5.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Performance & Cleanliness Verification
- [ ] Task 6: Verify suite performance and RAII hygiene
  - [ ] Subtask 6.1: Write failing test verifying full Epic 5 suite executes in <30 seconds
  - [ ] Subtask 6.2: Write failing test verifying no temporary database files are left behind after Redb tests
  - [ ] Subtask 6.3: Ensure all tests use `IsolatedTestContext` or RAII guards for cleanup
  - [ ] Subtask 6.4: Run `mise run timing` and verify execution speed
  - [ ] Subtask 6.5: Run `mise run verify` one final time to confirm 100% quality gate pass
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

## Dev Notes

### Architecture Compliance
- **Adversarial Review**: This is not a checkbox exercise; it is a "brutal foundation critique" to ensure the caching layer is bulletproof.
- **Hexagonal Integrity**: Integration tests must validate contract boundaries, while unit tests focus on logic and conversions.
- **Async Resource Safety**: Mandatory multi-threaded testing for all async components to surface potential race conditions.

### Technical Requirements
- **Co-located tests**: Follow Rust idioms by keeping unit tests in `#[cfg(test)]` modules within the same file.
- **Mutation Testing Concept**: Developers should manually flip logic gates or return values to confirm that their tests actually catch regressions.
- **Persistence Validation**: `RedbCache` must surviving process drops; this is a critical NFR.

### Library Dependencies
- **mockall**: Mandatory for port isolation.
- **tracing-test**: For verifying observability events.
- **criterion**: For performance measurements.
- **tarpaulin**: For coverage auditing.

### File Structure Requirements
- **Review Scope**: `moka.rs`, `redb.rs`, `coordinator.rs`, `errors.rs`, and `mod.rs`.
- **Location**: All files in `crates/adapters/src/spi/cache/` and `spi/errors.rs`.

### Project Structure Notes
- **Alignment**: Ensures Epic 5 meets the high quality bar set in Epics 1-4.
- **Conflict Prevention**: Fixes any inconsistencies in test naming or organization established during rapid implementation.

### TDD Methodology
- **RED-GREEN-REFACTOR**: Applied to the review process itself - write tests that verify the *presence and quality* of other tests.
- **Mise Orchestration**: Use `mise run timing`, `mise run test:coverage`, and `mise run verify` exclusively.

### References
- [Source: _bmad-output/test-design-system.md]
- [Source: _bmad-output/test-developer-guide.md]
- [Source: project-context.md#Testing-Rules]
- [Source: ADR 0010: Centralized Test Utilities]
- [Source: Story 5.1 - 5.5]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology for test suite review.
- Preserved original Epic ACs for adversarial review.
- Integrated mandatory linting workflows and mise orchestration.
- Provided specific tasks for mutation testing, persistence validation, and RAII hygiene.
- Enforced Section 8 compliance for all test code.

### File List
- `crates/adapters/src/spi/cache/*.rs` - Audit targets.
- `_bmad-output/implementation-artifacts/stories/5-6-review-epic-5-test-suite.md` - Generated story.
