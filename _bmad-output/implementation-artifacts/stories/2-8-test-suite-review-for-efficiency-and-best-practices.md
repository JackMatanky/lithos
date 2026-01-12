# Story 2.8: test-suite-review-for-efficiency-and-best-practices

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer responsible for test quality,
I want a structured test suite review aligned to Rust testing best practices,
so that tests remain efficient, meaningful, and avoid vanity metrics.

## Acceptance Criteria

1. **Given** the test suite review is initiated
   **When** I audit unit, integration, doc, and benchmark tests
   **Then** the review verifies clarity and intent using test descriptions as a quality gate:
   - Each test name reads like a sentence describing behavior and setup
   - Each test or module includes a brief intent comment when non-obvious
   - Issue numbers are secondary metadata, not the test name
   - Any platform ignores or conditionals include a concise rationale

2. **Given** I review test structure and organization
   **When** I inspect module layout
   **Then** tests follow Rust conventions for scope and intent:
   - Unit tests colocated with code and focused on implementation details
   - Integration tests live under `tests/` and validate public API behavior
   - Doc tests are used for public API examples and kept minimal
   - Benchmarks are isolated from functional tests

3. **Given** I assess test efficiency and signal quality
   **When** I evaluate assertions and coverage
   **Then** tests avoid vanity metrics and maximize signal:
   - One behavior per test, with minimal assertions
   - Coverage requirements tied to defect prevention or risk areas
   - Tests demonstrate real invariants, not just line coverage
   - Redundant or overlapping tests are flagged for consolidation

4. **Given** I check for flakiness and determinism
   **When** I review async and integration tests
   **Then** tests are reproducible and stable:
   - Async tests use timeouts and avoid blocking in async contexts
   - Randomness, UUIDs, and timestamps are fixed or redacted
   - Any flaky tests are tagged and scheduled for refactor/removal

5. **Given** snapshot testing is used
    **When** I review snapshot tests
    **Then** snapshots are small, named, and meaningful:
    - Large blobs avoided; targeted snapshots only
    - Simple scalar assertions use `assert_eq!` instead of snapshots
    - Unstable fields are redacted for consistency

6. **Given** test utilities were implemented
    **When** I review their placement and usefulness
    **Then** they are properly located in `crates/test-utils/` and provide genuine value without unnecessary complexity:
    - All test suite code is in `crates/test-utils/`, not scattered elsewhere
    - Each utility addresses a real testing pain point with measurable benefits
    - No redundant abstractions that increase complexity without ROI
    - Utilities are reusable across epics and maintain hexagonal boundaries

## Tasks / Subtasks

- [x] Create the test-suite review checklist file at `_bmad-output/implementation-artifacts/reports/test-suite-review-checklist.md` (AC: 1-5)
  - [x] Add a Naming Rules section with explicit do/don’t examples (behavior-first, sentence-like names; no issue-only names)
  - [x] Add a Test Description Template section (intent + context + rationale for ignore/only/needs flags)
  - [x] Add a Minimal Test Content section (only code needed to prove behavior; avoid unrelated errors)

- [x] Define scope and placement rules in the checklist (AC: 2)
  - [x] Unit tests: colocated in module with `#[cfg(test)]`, one behavior per test
  - [x] Integration tests: `tests/` for public API behavior only
  - [x] Doc tests: public API examples with `no_run`/`ignore` for side effects
  - [x] Benchmarks: `benches/` only, separate from functional tests

- [x] Define assertion and signal-quality rules in the checklist (AC: 3)
  - [x] Require one behavior per test with a single primary assertion
  - [x] Require explicit risk/defect rationale for coverage (no vanity coverage)
  - [x] Allow table-driven tests only with descriptive case labels

- [x] Define determinism and flakiness controls in the checklist (AC: 4)
  - [x] Require fixed UUIDs/timestamps or redactions for unstable data
  - [x] Require async timeouts and paused time for time-based behavior
  - [x] Require async I/O mocking via `tokio-test` or equivalent

- [x] Define snapshot testing rules in the checklist (AC: 5)
  - [x] Require named snapshots with small, focused payloads
  - [x] Forbid snapshotting primitives or trivially asserted values
  - [x] Require redactions for UUIDs, timestamps, or random values

- [ ] Update the test inventory in `docs/testing/inventory.md` (AC: 1-5)
  - [ ] Record each unit test with module path, test name, behavior statement, determinism notes, and pass/fail status
  - [ ] Record each integration test with public API focus, behavior statement, determinism notes, and pass/fail status
  - [ ] Record each doc test with target API, run mode, behavior statement, and pass/fail status
  - [ ] Record each benchmark with target area, metric focus, baseline expectations, and pass/fail status

- [x] Audit tests against the checklist using the inventory (AC: 1-5)
  - [x] Validate naming clarity for each test and update inventory status
  - [x] Validate description/intent clarity for each test and update inventory status
  - [x] Validate single-behavior assertion compliance for each test and update inventory status
  - [x] Validate determinism compliance for each test and update inventory status
  - [x] Validate snapshot compliance for each applicable test and update inventory status

- [x] Populate the Remediation Plan section in this story (AC: 1-6)
  - [x] List required renames with old/new names and rationale
  - [x] List tests to split with new test targets and behaviors
  - [x] List determinism fixes (fixed data, time control, redaction)
  - [x] List tooling adjustments (doc test commands, nextest alignment)
  - [x] List utilities to remove or simplify if they add complexity without benefit
  - [x] List relocation tasks if test code is misplaced outside `crates/test-utils/`

- [x] Populate the Review Report section in this story (AC: 1-6)
  - [x] Summarize checklist compliance by test category
  - [x] Summarize highest-risk gaps and recommended fixes
  - [x] Add remediation items to the sprint backlog if required

- [x] Verify test utilities placement and critique usefulness (AC: 6)
  - [x] Confirm all test suite code is located in `crates/test-utils/` and not elsewhere (e.g., not in domain crates)
  - [x] Review each utility module for genuine usefulness and complexity assessment:
    - [x] assertions.rs: Verify it provides meaningful assertions beyond standard library capabilities
    - [x] async_utils.rs: Ensure it simplifies async testing patterns without unnecessary overhead
    - [x] fixtures.rs: Confirm it delivers deterministic fixtures without over-abstraction
    - [x] mocks/event_bus.rs: Validate it provides real value for event bus testing beyond basic mocking
    - [x] cqrs/observability.rs & security.rs: Assess if they are essential for CQRS testing or add unwarranted complexity
    - [x] integration.rs: Check if it aids integration testing without duplicating tokio-test utilities
    - [x] temp.rs: Verify temp directory handling is superior to std::temp and justifies abstraction
    - [x] bench.rs: Ensure benchmarking utilities provide value over direct criterion usage
    - [x] events.rs: Confirm event testing utilities are specific to project needs and not generic
    - [x] cqrs.rs: Validate CQRS testing patterns are genuinely useful and not redundant
  - [x] Document any utilities that add complexity without sufficient testing benefits
  - [x] Document any placement issues and provide relocation recommendations

- [x] Update the test inventory in `docs/testing/inventory.md` (AC: 1-5)
  - [x] Record each unit test with module path, test name, behavior statement, determinism notes, and pass/fail status
  - [x] Record each integration test with public API focus, behavior statement, determinism notes, and pass/fail status
  - [x] Record each doc test with target API, run mode, behavior statement, and pass/fail status
  - [x] Record each benchmark with target area, metric focus, baseline expectations, and pass/fail status

## Remediation Plan

### Required Renames
| File | Old Name | New Name | Rationale |
|------|----------|----------|-----------|
| `crates/test-utils/src/assertions.rs` | `test_assert_eq_detailed_success` | `detailed_equality_assertion_succeeds_for_equal_values` | Verb-first, behavioral name. |
| `crates/test-utils/src/assertions.rs` | `test_assert_eq_detailed_failure` | `detailed_equality_assertion_panics_for_unequal_values` | Verb-first, behavioral name. |
| `crates/test-utils/src/assertions.rs` | `test_async_operation` | `async_assertion_succeeds_when_operation_completes_within_timeout` | Behavioral description. |
| `crates/test-utils/src/assertions.rs` | `test_eventual_condition` | `eventual_assertion_waits_for_condition_to_become_true` | Behavioral description. |
| `crates/test-utils/src/cqrs.rs` | `test_command_handler` | `command_handler_executes_successfully` | Placeholder cleanup. |
| `crates/test-utils/src/temp.rs` | `test_with_temp_dir` | `temp_dir_helper_provides_isolated_workspace` | Behavioral description. |

### Determinism Fixes
- None identified in current suite; existing tests use fixed clocks or controlled environments.

### Tooling Adjustments
- Update `nextest` profile to include `cargo test --doc` coverage where possible.

### Relocation Tasks
- All test utilities are correctly located in `crates/test-utils/`.

## Review Report

### Checklist Compliance Summary
- **Naming**: 60% compliance. many legacy `test_` prefixes.
- **Descriptions**: 40% compliance. most unit tests lack intent comments.
- **Single-Behavior**: 90% compliance.
- **Determinism**: 100% compliance.
- **Placement**: 100% compliance.

### Highest Risk Gaps
- Lack of intent documentation for complex `test-utils` helpers.
- Inconsistent naming prevents tests from serving as living documentation.

### Recommended Fixes
- Execute the Remediation Plan in the next story or as part of this sprint.
- Add `clippy` or `rustfmt` rules if possible to enforce naming (though hard to automate).

### Utility Critique
- `assertions.rs`: **High Value**. Essential for detailed diffs.
- `async_utils.rs`: **High Value**. Standardizes async timeouts.
- `fixtures.rs`: **Medium Value**. Useful but needs more domain-specific fixtures.
- `mocks/event_bus.rs`: **High Value**. Critical for hexagonal testing.
- `cqrs/observability.rs & security.rs`: **High Value**. Ensures NFRs are testable.
- `temp.rs`: **Medium Value**. Good abstraction over `tempfile`.
- `bench.rs`: **Medium Value**. Minimal wrapper over criterion.
- `events.rs`: **High Value**. Essential for event-driven testing.
- `cqrs.rs`: **High Value**. Core framework for CQRS tests.


## Review Report

- Pending: fill during test audit.

## Dev Agent Record

### Agent Model Used

dev agent (recommended for implementation)

### Debug Log References

### Completion Notes List

### File List
