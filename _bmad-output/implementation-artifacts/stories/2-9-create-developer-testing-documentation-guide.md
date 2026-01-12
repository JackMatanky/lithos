# Story 2.9: create-developer-testing-documentation-guide

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer onboarding to Lithos testing patterns,
I want a consolidated testing documentation guide that references project rules and ADRs,
so that I can apply the approved patterns consistently and avoid ambiguity.

## Acceptance Criteria

1. **Given** I need to understand Lithos testing standards
   **When** I open the developer testing guide
   **Then** it documents:
   - Hexagonal testing hierarchy (domain, integration, E2E)
   - Async test requirements and timeouts
   - Event-driven and CQRS testing patterns
   - Integration testing rules and isolation requirements

2. **Given** I need to run tests locally or in CI
   **When** I follow the guide
   **Then** it provides:
   - `mise run` commands for unit, integration, coverage, and benchmarks
   - `nextest` usage and doc test requirements
   - Coverage expectations and tarpaulin usage

3. **Given** I am authoring new tests
   **When** I follow the guidance
   **Then** the guide includes:
   - Naming and description conventions (clarity checks)
   - Deterministic fixture rules (fixed UUIDs/timestamps)
   - Snapshot testing rules and redaction guidance
   - Checklist for test isolation and anti-patterns

## Tasks / Subtasks

- [ ] Review and validate existing test documentation accuracy (AC: 1-3)
  - [ ] Compare `docs/testing/async.md` section by section against Rust Book Ch.11, Tokio docs, and project context async rules; flag any outdated or missing patterns
  - [ ] Compare `docs/testing/event.md` against ADR 0008 (event patterns) and ADR 0009 (CQRS); note any gaps in timing, payload verification, or mock usage
  - [ ] Compare ADR 0010 (utilities) and ADR 0011 (integration) against reviewed sources; identify any misalignments with Rust testing best practices
  - [ ] Document all misalignments in a gap analysis file at `_bmad-output/implementation-artifacts/reports/test-docs-gap-analysis.md` with specific references and required updates

- [ ] Update and align existing documentation (AC: 1-3)
  - [ ] For each identified gap, research current Rust testing best practices via authoritative sources (Rust Book, Tokio docs, rustc-dev-guide)
  - [ ] Update `docs/testing/async.md` with precise clarifications on timeouts, blocking avoidance, and multi-threaded testing; ensure examples are minimal and correct
  - [ ] Update `docs/testing/event.md` with explicit CQRS patterns, timing assertions, and malformed event handling; add code examples for each pattern
  - [ ] Update ADRs 0010 and 0011 if they reference outdated practices; ensure they align with current Rust ecosystem standards

- [ ] Create comprehensive developer testing guide at `docs/testing/developer-guide.md` (AC: 1-3)
  - [ ] Write a 2-paragraph overview: define scope (Lithos testing standards), audience (developers onboarding or writing tests), and usage (quick reference for patterns and commands)
  - [ ] Add "Testing Hierarchy" section: describe hexagonal layers (domain unit, integration public APIs, E2E CLI), with when to use each and file locations
  - [ ] Add "Async Testing" section: summarize mandatory `#[tokio::test(flavor = "multi_thread")]` usage, blocking avoidance, timeouts; reference `docs/testing/async.md` for details
  - [ ] Add "Event & CQRS Testing" section: outline Given-When-Then for aggregates, mock event buses, payload verification; reference ADR 0008 and `docs/testing/event.md`
  - [ ] Add "Integration Testing" section: describe `tests/` structure, testcontainers usage, trait mocking; reference ADR 0011 for patterns

- [ ] Add precise "Running Tests" section (AC: 2)
  - [ ] List exact `mise run` commands: `mise run test` (all), `mise run test:unit` (domain+app), `mise run test:integration` (external APIs), `mise run test:coverage` (tarpaulin HTML report), `mise run test:benchmark` (criterion)
  - [ ] Document `cargo nextest` usage: parallel execution, filtering; note `cargo test --doc` for doc tests; explain when to use each
  - [ ] Specify coverage expectations: 80%+ via tarpaulin, focus on `crates/app` and `crates/domain`; report location and how to interpret

- [ ] Add detailed "Test Authoring Standards" section (AC: 3)
  - [ ] Document naming: tests must read like sentences (e.g., `test_user_creation_succeeds_with_valid_data`); forbid issue-only names; use descriptive case names in tables
  - [ ] Document descriptions: each test must have a brief intent comment when non-obvious; rationalize ignores/conditionals
  - [ ] Document fixtures: use fixed UUIDs/timestamps for determinism; seed randomness; avoid unstable data in snapshots
  - [ ] Document snapshots: use `insta` with redactions for UUIDs/timestamps; avoid snapshotting primitives; name snapshots meaningfully
  - [ ] Document isolation: one behavior per test; no shared state; use temp dirs for file tests; automatic cleanup

- [ ] Add actionable "Common Pitfalls" section (AC: 3)
  - [ ] List async pitfalls: never block threads without `spawn_blocking_test`; use timeouts to prevent hangs; avoid single-threaded for concurrent code
  - [ ] List flakiness causes: time-based tests without paused time; unseeded randomness; shared state between tests
  - [ ] List anti-patterns: multi-behavior tests (split them); vanity coverage (require defect-prevention rationale); issue-only names (use descriptive names)

- [ ] Implement cross-linking and index updates (AC: 1-3)
  - [ ] Add precise links to `docs/testing/async.md`, `docs/testing/event.md`, ADRs 0008-0011, and `_bmad-output/project-context.md`
  - [ ] Update `docs/index.md` to include "Testing Guide" link under documentation section
  - [ ] Update any existing TOC or testing overview to reference the new guide

### Documentation Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] Verify all links and references are valid and point to correct locations
- [ ] Ensure consistent formatting and terminology across all updated documents
- [ ] Confirm all documentation follows project writing standards (clear, concise, actionable)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks (documentation-focused)
- [ ] **CRITICAL:** Fix ALL documentation issues - NO EXCEPTIONS, NO BYPASSING
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `docs: add developer testing guide and update test documentation for accuracy`

## Dev Notes

- **Primary Sources:** `docs/testing/async.md`, `docs/testing/event.md`, ADR 0008-0011, `_bmad-output/project-context.md`.
- **Testing Rules:** Use `#[tokio::test(flavor = "multi_thread")]` for integration tests and enforce deterministic fixtures.
- **Coverage:** Maintain 80%+ coverage via tarpaulin; focus on `app` and `domain`.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story-2.9]
- [Source: docs/testing/async.md]
- [Source: docs/testing/event.md]
- [Source: docs/adr/0008-event-driven-testing-patterns.md]
- [Source: docs/adr/0009-cqrs-testing-patterns.md]
- [Source: docs/adr/0010-centralized-test-utilities.md]
- [Source: docs/adr/0011-integration-testing-patterns.md]
- [Source: _bmad-output/project-context.md]

## Dev Agent Record

### Agent Model Used

dev agent (recommended for implementation)

### Debug Log References

### Completion Notes List

### File List
