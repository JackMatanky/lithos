# Story 2.8: test-suite-review-for-efficiency-and-best-practices

Status: ready-for-dev

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

## Tasks / Subtasks

- [ ] Create the test-suite review checklist file at `_bmad-output/implementation-artifacts/reports/test-suite-review-checklist.md` (AC: 1-5)
  - [ ] Add a Naming Rules section with explicit do/don’t examples (behavior-first, sentence-like names; no issue-only names)
  - [ ] Add a Test Description Template section (intent + context + rationale for ignore/only/needs flags)
  - [ ] Add a Minimal Test Content section (only code needed to prove behavior; avoid unrelated errors)

- [ ] Define scope and placement rules in the checklist (AC: 2)
  - [ ] Unit tests: colocated in module with `#[cfg(test)]`, one behavior per test
  - [ ] Integration tests: `tests/` for public API behavior only
  - [ ] Doc tests: public API examples with `no_run`/`ignore` for side effects
  - [ ] Benchmarks: `benches/` only, separate from functional tests

- [ ] Define assertion and signal-quality rules in the checklist (AC: 3)
  - [ ] Require one behavior per test with a single primary assertion
  - [ ] Require explicit risk/defect rationale for coverage (no vanity coverage)
  - [ ] Allow table-driven tests only with descriptive case labels

- [ ] Define determinism and flakiness controls in the checklist (AC: 4)
  - [ ] Require fixed UUIDs/timestamps or redactions for unstable data
  - [ ] Require async timeouts and paused time for time-based behavior
  - [ ] Require async I/O mocking via `tokio-test` or equivalent

- [ ] Define snapshot testing rules in the checklist (AC: 5)
  - [ ] Require named snapshots with small, focused payloads
  - [ ] Forbid snapshotting primitives or trivially asserted values
  - [ ] Require redactions for UUIDs, timestamps, or random values

- [ ] Update the test inventory in `docs/testing/inventory.md` (AC: 1-5)
  - [ ] Record each unit test with module path, test name, behavior statement, determinism notes, and pass/fail status
  - [ ] Record each integration test with public API focus, behavior statement, determinism notes, and pass/fail status
  - [ ] Record each doc test with target API, run mode, behavior statement, and pass/fail status
  - [ ] Record each benchmark with target area, metric focus, baseline expectations, and pass/fail status

- [ ] Audit tests against the checklist using the inventory (AC: 1-5)
  - [ ] Validate naming clarity for each test and update inventory status
  - [ ] Validate description/intent clarity for each test and update inventory status
  - [ ] Validate single-behavior assertion compliance for each test and update inventory status
  - [ ] Validate determinism compliance for each test and update inventory status
  - [ ] Validate snapshot compliance for each applicable test and update inventory status

- [ ] Populate the Remediation Plan section in this story (AC: 1-5)
  - [ ] List required renames with old/new names and rationale
  - [ ] List tests to split with new test targets and behaviors
  - [ ] List determinism fixes (fixed data, time control, redaction)
  - [ ] List tooling adjustments (doc test commands, nextest alignment)

- [ ] Populate the Review Report section in this story (AC: 1-5)
  - [ ] Summarize checklist compliance by test category
  - [ ] Summarize highest-risk gaps and recommended fixes
  - [ ] Add remediation items to the sprint backlog if required

### Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** Verify 90%+ test coverage is maintained
- [ ] **MANDATORY:** Confirm all code passes clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: review test suite for efficiency and best-practice compliance`

## Dev Notes

- **Architecture Compliance:** Follow hexagonal testing hierarchy (domain unit tests, integration in `tests/`, E2E CLI tests). Keep CQRS boundaries intact.
- **Testing Standards:** Use `nextest` for concurrent runs and `tarpaulin` for 80%+ coverage with focus on `app` and `domain` logic. Use `#[tokio::test(flavor = "multi_thread")]` for integration tests.
- **Async Determinism:** Use timeouts and paused time where appropriate for async tests; avoid blocking in async contexts.
- **Deterministic Fixtures:** Use fixed UUIDs and timestamps; avoid randomness unless redacted or seeded.

### Project Structure Notes

- Unit tests: colocated in module files with `#[cfg(test)]`.
- Integration tests: `tests/integration/` for cross-crate public API coverage.
- E2E tests: `tests/e2e/` for CLI behavior.
- Architecture tests: `tests/arch/` for boundary enforcement.
- Benchmarks: `benches/` for criterion-based performance checks.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story-2.8]
- [Source: _bmad-output/planning-artifacts/architecture.md] - testing standards, structure, and constraints
- [Source: _bmad-output/project-context.md] - async testing rules, nextest, tarpaulin targets
- https://doc.rust-lang.org/book/ch11-00-testing.html
- https://doc.rust-lang.org/cargo/guide/tests.html
- https://rustc-dev-guide.rust-lang.org/tests/best-practices.html
- https://raw.githubusercontent.com/apollographql/rust-best-practices/main/book/chapter_05.md
- https://tokio.rs/tokio/topics/testing

## Developer Context

- Establish a single, repeatable review checklist that enforces clarity checks and rejects vanity metrics.
- Treat test descriptions as a first-class quality gate (clear names, intent comments, and rationale for ignores).
- Keep tests minimal, deterministic, and scoped to one behavior per test.

## Technical Requirements

- Define naming conventions for tests that read like behavior statements.
- Require intent comments for non-obvious tests and context for ignores/platform gates.
- Enforce minimal assertions and one-behavior-per-test patterns.
- Require deterministic fixtures (fixed UUIDs/timestamps, redactions for unstable data).
- Ensure review outputs include a remediation plan and backlog updates.

## Architecture Compliance

- Respect hexagonal test hierarchy: domain unit tests, integration tests for cross-crate behavior, E2E CLI tests, and criterion benchmarks.
- Validate CQRS boundaries in tests and avoid crossing write/read responsibilities.
- Ensure tests and benchmarks align with project performance NFRs and CI expectations.

## Library and Framework Requirements

- Tokio async tests should use `#[tokio::test]` with timeouts; use paused time when appropriate.
- Use doc tests for public API examples; remember `cargo test --doc` when running `nextest`.
- Snapshot tests should use `insta` with redactions for unstable fields.
- Property-based testing should use `proptest` where it yields stronger invariants.

## File Structure Requirements

- Unit tests colocated with modules under `crates/*/src`.
- Integration tests under `tests/integration/` and E2E tests under `tests/e2e/`.
- Benchmark suites in `benches/` using criterion.
- Test inventory stored in `docs/testing/inventory.md`; review reports and remediation recorded in this story file.

## Testing Requirements

- Every test must state behavior and setup clearly in name and description.
- No vanity coverage: coverage targets must map to risk or defect prevention.
- Split tests that assert multiple behaviors; keep assertions minimal.
- Identify flaky tests; stabilize or quarantine with explicit rationale.
- Use redaction for snapshots of unstable output.

## Previous Story Intelligence

- Story 2.7 established criterion-based benchmarking with CI gates; align review findings with benchmark categories and performance targets.
- Benchmark infrastructure expects HTML reports, baseline comparisons, and mise tasks integration.

## Git Intelligence Summary

- Recent commits emphasize event-driven testing patterns and async test stability.
- Toolchain pinning via mise is in place; ensure test review respects that task orchestration.

## Latest Tech Information

- Rust Book + Cargo testing guides reinforce unit vs integration placement and doc test execution.
- rustc-dev-guide mandates descriptive test names, minimal test content, and intent comments.
- Apollo best practices recommend single-behavior tests with minimal assertions and clear organization.
- Tokio testing guidance recommends paused time and async I/O mocking via `tokio-test` utilities.

## Project Context Reference

- `nextest` is the default high-performance runner; `tarpaulin` targets 80%+ coverage.
- Integration tests must use `#[tokio::test(flavor = "multi_thread")]` for race detection.
- Deterministic fixtures are mandatory (fixed UUIDs/timestamps).

## Story Completion Status

- Status set to `ready-for-dev`.
- Completion note: comprehensive test suite review criteria defined for efficient, non-vanity testing.

## Remediation Plan

- Pending: fill during test audit.

## Review Report

- Pending: fill during test audit.

## Dev Agent Record

### Agent Model Used

dev agent (recommended for implementation)

### Debug Log References

### Completion Notes List

### File List
