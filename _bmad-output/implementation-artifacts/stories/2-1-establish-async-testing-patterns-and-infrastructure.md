# Story 2.1: establish-async-testing-patterns-and-infrastructure

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer testing async code,
I want standardized patterns for testing tokio-based async operations,
So that async tests are reliable, race-condition free, and properly isolated.

## Acceptance Criteria

**Given** I have researched async testing best practices in Rust
**When** I review the async testing infrastructure
**Then** standardized patterns are established for:
- `#[tokio::test]` macro usage with proper runtime setup
- `spawn_blocking` for CPU-intensive operations in tests
- `CancellationToken` for graceful test shutdown
- Race condition detection and prevention
- Timeout handling to prevent hanging tests

**Given** async testing patterns are established
**When** I write an async unit test
**Then** the test follows the established patterns:
- Proper `#[tokio::test]` attribute usage
- No blocking operations without `spawn_blocking`
- Proper error handling for async operations
- Test isolation without shared state

**Given** async tests are running
**When** I check for race conditions
**Then** tests use proper synchronization primitives and avoid flaky behavior

**Given** I have researched tokio testing ecosystem
**When** I check test dependencies
**Then** optimal crates are selected:
- `tokio::test` for basic async testing
- `tokio::time::timeout` for preventing hanging tests
- `tokio-test` for additional testing utilities
- Proper test runtime configuration

## Tasks / Subtasks

- [ ] Research comprehensive tokio async testing patterns and best practices
   - [ ] Analyze tokio::test macro usage and runtime configuration
   - [ ] Review spawn_blocking patterns for CPU-intensive test operations
   - [ ] Study CancellationToken and timeout patterns for test lifecycle
   - [ ] Examine race condition prevention techniques in async tests
- [ ] Create async testing infrastructure and utilities
   - [ ] Set up tokio test runtime configuration for consistent behavior
   - [ ] Implement test utilities for spawn_blocking operations
   - [ ] Create timeout helpers to prevent hanging tests
   - [ ] Develop cancellation token patterns for graceful test shutdown
- [ ] Establish async testing patterns and guidelines
   - [ ] Define #[tokio::test] usage standards and best practices
   - [ ] Create patterns for testing async functions and futures
   - [ ] Implement synchronization primitives for race-free testing
   - [ ] Develop error handling patterns for async test failures
- [ ] Test async testing infrastructure
   - [ ] Validate tokio test runtime configuration
   - [ ] Test spawn_blocking and timeout utilities
   - [ ] Verify cancellation token patterns work correctly
   - [ ] Ensure all patterns prevent race conditions and flakiness

## Dev Notes

- **Architecture Compliance**: Establishes async testing foundations that support the hexagonal architecture's async domain operations and event-driven patterns in Epic 2.

- **Technical Requirements**: Create standardized tokio-based async testing patterns with proper runtime setup, spawn_blocking utilities, timeout handling, and race condition prevention.

- **Source Tree Components**: Async test utilities in test-utils crate, tokio test configuration in Cargo.toml, testing guidelines in docs/testing/async-testing.md.

- **Testing Standards Summary**: Async tests follow established patterns ensuring reliability, proper isolation, and race-free execution with comprehensive coverage.

### Project Structure Notes

- **Alignment with unified project structure**: Async testing patterns integrate with existing test infrastructure and quality pipelines.

- **Detected conflicts or variances**: None - builds upon established tokio runtime usage patterns.

### Technical Requirements

- Create tokio test runtime configuration with proper setup and teardown
- Implement spawn_blocking utilities for CPU-intensive test operations
- Develop timeout helpers using tokio::time::timeout to prevent hanging tests
- Establish CancellationToken patterns for graceful async test shutdown
- Create synchronization primitives for race-free async testing

### File Structure Requirements

- Async test utilities in crates/test-utils/src/async.rs
- Tokio test configuration in Cargo.toml workspace [profile.dev] and [profile.test]
- Async testing guidelines in docs/testing/async-testing.md
- Example async test files in crates/*/tests/async_*.rs

### Testing Requirements

- Unit tests for async testing utilities themselves
- Integration tests verifying tokio runtime configuration
- Performance tests ensuring async tests don't introduce overhead
- Flakiness detection tests for race condition prevention

### Previous Story Intelligence

- Story 1.3 established tokio runtime usage - build async testing on existing runtime setup
- Story 1.2 established testing infrastructure - extend with async-specific patterns

### Git Intelligence Summary

- Recent commits show tokio integration patterns
- Async testing patterns align with established async code conventions

### Latest Tech Information

- Tokio 1.x async testing patterns stable and well-established
- Focus on runtime configuration and timeout prevention
- Integration with tokio-test utilities for enhanced testing

### Project Context Reference

- Lithos project uses tokio for async operations throughout hexagonal architecture
- Testing strategy emphasizes async reliability and race-free execution
- Domain layer requires async testing for event-driven operations

### Story Completion Status

- Status: ready-for-dev
- All acceptance criteria defined with testable async testing requirements
- Technical requirements complete with tokio-specific implementations
- Integration points identified with existing async infrastructure
- Risk assessment: Low risk, builds on established tokio patterns

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Async Architecture Patterns]
- [Source: _bmad-output/planning-artifacts/epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story 2.1]
- [Source: Tokio Testing Documentation (https://tokio.rs/tokio/topics/testing)]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
