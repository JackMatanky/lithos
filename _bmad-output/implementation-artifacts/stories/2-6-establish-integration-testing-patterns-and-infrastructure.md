# Story 2.6: establish-integration-testing-patterns-and-infrastructure

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer testing cross-module interactions,
I want patterns for integration testing,
So that integration issues are caught early with proper isolation and mocking.

## Acceptance Criteria

1. **Given** I have researched integration testing patterns in large Rust projects
   **When** I review the integration testing infrastructure
   **Then** patterns are established for:
   - Cross-module API contract testing
   - Database state management in integration tests
   - External service mocking for isolated testing
   - Integration test data fixtures and setup

2. **Given** integration testing patterns are established
   **When** I test interactions between bounded contexts
   **Then** the test verifies:
   - API contracts between modules are maintained
   - Data flows correctly across boundaries
   - Error handling works end-to-end
   - Performance meets integration requirements

3. **Given** integration tests are running
   **When** I check for data consistency
   **Then** tests use proper transaction management and rollback

4. **Given** I have researched integration testing best practices
   **When** I check the test setup
   **Then** integration tests:
   - Run in isolated environments (test containers if needed)
   - Use realistic test data without production dependencies
   - Execute in parallel where possible
   - Provide clear failure diagnostics

## Tasks / Subtasks

- [ ] Research integration testing patterns in Rust ecosystems
  - [ ] Analyze existing async/event-driven/CQRS test patterns for extension
  - [ ] Identify cross-module testing needs in bounded context architecture
  - [ ] Review testcontainers vs mocking alternatives (fidelity vs speed trade-off)
  - [ ] Evaluate trait-based mocking with mockall for external services

- [ ] Establish integration test infrastructure (Phase 1: 1 week)
  - [ ] Create tests/integration/ directory structure
  - [ ] Add testcontainers dependency (v0.26.3) for external service mocking with Docker Compose support
  - [ ] Implement trait-based mocking framework for ports/adapters
  - [ ] Set up test database isolation with testcontainers or sqlx transactions
  - [ ] Create shared test fixtures for bounded context interactions

- [ ] Implement cross-module testing patterns (Phase 2: 1 week)
  - [ ] Define API contract testing between bounded contexts (hexagonal validation)
  - [ ] Create end-to-end data flow verification tests across modules
  - [ ] Implement error propagation testing across boundaries
  - [ ] Add performance validation for integration scenarios (2-3x slower expected)

- [ ] Configure mise test tasks and CI integration (Phase 3: 2 weeks)
  - [ ] Add mise run test:integration task with --test-threads for parallelism
  - [ ] Configure test container management and cleanup
  - [ ] Integrate with existing test suite (separate execution from unit tests)
  - [ ] Establish performance baselines for integration test execution

## Dev Notes

- **Architecture Compliance:** Follow established async testing patterns, extend for cross-boundary scenarios. Use existing test utilities for fixtures and isolation.
- **Testing Standards:** Integration tests should run separately from unit tests, with clear isolation. Aim for 80% coverage on integration paths.
- **Source Tree Components:** Add integration test modules in `tests/` directory, leverage existing `lithos-core` test utilities.
- **Dependencies:** May need test containers (testcontainers crate) for database isolation, existing tokio for async testing.

### Project Structure Notes

- Align with existing test structure in `tests/` directory
- Follow naming conventions established in Epic 1 (kebab-case for modules)
- Integration tests in separate `tests/integration/` subdirectory for isolation

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story-2.6]
- [Source: _bmad-output/planning-artifacts/architecture.md] - for bounded context interactions
- [Source: _bmad-output/implementation-artifacts/stories/2-1-establish-async-testing-patterns-and-infrastructure.md] - base async patterns
- [Source: _bmad-output/implementation-artifacts/stories/2-4-create-centralized-test-utilities-and-infrastructure.md] - test utilities to extend
- [ADR: docs/adr/0011-integration-testing-patterns.md] - research and decision framework

## Dev Agent Record

### Agent Model Used

dev agent (recommended for implementation)

### Debug Log References

### Completion Notes List

### File List
