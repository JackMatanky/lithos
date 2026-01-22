# Epic 2: Test Architecture, Patterns & Utilities **[MVP CORE]**

Developers have comprehensive testing patterns for async code, event-driven systems, and CQRS, plus centralized test utilities for artifacts, temporary directories, and mise task orchestration that ensure 80%+ coverage and catch integration issues early.
**FRs covered:** NFR16 (test coverage), Architecture requirements (testing strategy)
**Implementation Notes:**

- Test patterns: async (tokio::test), event-driven, CQRS command/query separation
- Centralized test utilities: artifact output locations, tmp directory management, test helper functions
- mise.toml test tasks: test, test:unit, test:integration, test:coverage, test:watch, test:benchmark
- ADR creation guidelines for epics making architectural decisions

## Story 2.1: Establish Async Testing Patterns and Infrastructure

As a developer testing async code,
I want standardized patterns for testing tokio-based async operations,
So that async tests are reliable, race-condition free, and properly isolated.

**Acceptance Criteria:**

**Given** I have researched async testing best practices in Rust
**When** I review the async testing infrastructure
**Then** standardized patterns are established for:

- `#[tokio::test]` macro usage with proper runtime setup
- `spawn_blocking` for CPU-intensive operations in tests
- `CancellationToken` for graceful test shutdown
- Race condition detection and prevention

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
- Proper test runtime configuration

## Story 2.2: Create Event-Driven Testing Patterns

As a developer testing event-driven systems,
I want patterns for testing domain events and event bus interactions,
So that event-driven code is thoroughly tested with proper isolation and verification.

**Acceptance Criteria:**

**Given** I have researched event-driven testing patterns
**When** I review the event testing infrastructure
**Then** patterns are established for:

- Event publishing and subscription testing
- Event payload verification
- Event ordering and timing verification
- Mock event bus implementations for unit tests

**Given** event-driven testing patterns are established
**When** I test an event publisher
**Then** the test verifies:

- Correct events are published
- Event payloads contain expected data
- Events are published at the correct time
- Error handling for failed event publishing

**Given** event-driven testing patterns are established
**When** I test an event subscriber
**Then** the test verifies:

- Subscriber receives expected events
- Event handling logic executes correctly
- Subscriber handles malformed events gracefully
- Subscription lifecycle management

**Given** I have researched event testing in domain-driven design
**When** I check the patterns
**Then** they follow DDD testing best practices:

- Event sourcing verification patterns
- Event storming validation
- Domain event contract testing
- Integration testing for event flows

## Story 2.3: Establish CQRS Testing Patterns

As a developer testing CQRS command and query separation,
I want patterns for testing write operations and read models separately,
So that command side and query side code are tested in isolation with proper verification.

**Acceptance Criteria:**

**Given** I have researched CQRS testing patterns
**When** I review the CQRS testing infrastructure
**Then** patterns are established for:

- Command handler testing (write side)
- Query handler testing (read side)
- Command/query separation verification
- Eventual consistency testing between write and read models

**Given** CQRS testing patterns are established
**When** I test a command handler
**Then** the test verifies:

- Command validation logic
- State changes are applied correctly
- Domain events are published
- Error cases are handled appropriately

**Given** CQRS testing patterns are established
**When** I test a query handler
**Then** the test verifies:

- Query execution returns correct data
- Query performance meets requirements
- Query isolation from write operations
- Caching behavior if applicable

**Given** I have researched CQRS testing in Rust ecosystems
**When** I check the implementation
**Then** it addresses common CQRS testing challenges:

- Testing eventual consistency
- Mocking read model updates
- Verifying command/query separation
- Testing cross-aggregate consistency

## Story 2.4: Create Centralized Test Utilities and Infrastructure

As a developer writing tests across the codebase,
I want centralized test utilities for common testing needs,
So that tests are consistent, maintainable, and don't duplicate utility code.

**Acceptance Criteria:**

**Given** I have researched test utility patterns in large Rust projects
**When** I review the centralized test utilities
**Then** utilities are provided for:

- Temporary directory creation and cleanup
- Test artifact output management
- Test data fixtures and factories
- Common assertion helpers

**Given** centralized test utilities exist
**When** I write a test needing temporary files
**Then** I can use standardized temporary directory utilities:

- Automatic cleanup after test completion
- Cross-platform path handling
- Unique directory names to avoid conflicts
- Proper error handling for directory operations

**Given** centralized test utilities exist
**When** I write a test needing test data
**Then** I can use standardized fixture utilities:

- Domain object factories with valid defaults
- Sample data generation for various scenarios
- Serialization helpers for complex objects
- Reusable test data across multiple tests

**Given** I have researched test isolation best practices
**When** I check the utilities
**Then** they ensure proper test isolation:

- No shared state between tests
- Proper cleanup of resources
- Database/transaction isolation for integration tests
- Process isolation for system tests

## Story 2.5: Configure Mise Test Task Orchestration

As a developer running tests during development,
I want comprehensive mise tasks for different testing scenarios,
So that I can efficiently run tests, check coverage, and maintain code quality during development.

**Acceptance Criteria:**

**Given** I have researched mise task orchestration for Rust projects
**When** I review the mise.toml test tasks
**Then** comprehensive test tasks are configured:

- `mise run test` - Run all tests with optimal parallelization
- `mise run test:unit` - Domain layer unit tests only
- `mise run test:integration` - Cross-crate integration tests
- `mise run test:coverage` - Generate coverage report (tarpaulin)
- `mise run test:watch` - Watch mode for TDD workflow

**Given** mise test tasks are configured
**When** I run `mise run test`
**Then** tests execute with:

- Optimal parallelization for speed
- Proper output formatting
- Clear success/failure indication
- Timing information for slow tests

**Given** mise test tasks are configured
**When** I run `mise run test:coverage`
**Then** coverage report is generated:

- HTML report for browser viewing
- Coverage percentage calculation
- File-by-file coverage breakdown
- Integration with CI/CD pipelines

**Given** I have researched continuous testing workflows
**When** I check the mise configuration
**Then** tasks support modern development workflows: - Watch mode for automatic test re-running - Fast feedback for TDD cycles - Integration with IDEs and editors - Remote development environment support

## Story 2.6: Establish Integration Testing Patterns and Infrastructure

As a developer testing cross-module interactions,
I want patterns for integration testing,
So that integration issues are caught early with proper isolation and mocking.

**Acceptance Criteria:**

**Given** I have researched integration testing patterns in large Rust projects
**When** I review the integration testing infrastructure
**Then** patterns are established for:

- Cross-module API contract testing
- Database state management in integration tests
- External service mocking for isolated testing
- Integration test data fixtures and setup

**Given** integration testing patterns are established
**When** I test interactions between bounded contexts
**Then** the test verifies:

- API contracts between modules are maintained
- Data flows correctly across boundaries
- Error handling works end-to-end
- Performance meets integration requirements

**Given** integration tests are running
**When** I check for data consistency
**Then** tests use proper transaction management and rollback

**Given** I have researched integration testing best practices
**When** I check the test setup
**Then** integration tests:

- Run in isolated environments (test containers if needed)
- Use realistic test data without production dependencies
- Execute in parallel where possible
- Provide clear failure diagnostics

## Story 2.7: Create Benchmarking Infrastructure and Performance Testing Patterns

As a developer measuring and preventing performance regressions,
I want benchmarking patterns and infrastructure,
So that performance is monitored and regressions are caught early.

**Acceptance Criteria:**

**Given** I have researched benchmarking in Rust ecosystems
**When** I review the benchmarking infrastructure
**Then** patterns are established for:

- criterion.rs integration for micro-benchmarks
- Performance regression detection
- Benchmark result storage and comparison
- CI/CD integration for performance gates

**Given** benchmarking patterns are established
**When** I create a performance benchmark
**Then** the benchmark:

- Uses criterion for statistical accuracy
- Measures relevant performance metrics
- Includes baseline comparisons
- Runs in CI/CD pipeline

**Given** performance tests are running
**When** I check for regressions
**Then** the system:

- Compares against historical baselines
- Alerts on significant performance drops
- Provides detailed performance reports
- Supports multiple benchmark categories

**Given** I have researched performance testing best practices
**When** I check the implementation
**Then** it addresses common performance testing challenges:

- Warm-up periods for JIT optimization
- Statistical significance in measurements
- Environment consistency across runs
- Memory usage tracking alongside timing

## Story 2.8: Review Epic 2 Test Suite

As a developer responsible for test quality,
I want a structured test suite review aligned to Rust testing best practices,
So that tests remain efficient, meaningful, and avoid vanity metrics.

**Acceptance Criteria:**

**Given** the test suite review is initiated
**When** I audit unit, integration, doc, and benchmark tests
**Then** the review verifies clarity and intent using test descriptions as a quality gate:

- Each test name reads like a sentence describing behavior and setup
- Each test or module includes a brief intent comment when non-obvious
- Issue numbers are secondary metadata, not the test name
- Any platform ignores or conditionals include a concise rationale

**Given** I review test structure and organization
**When** I inspect module layout
**Then** tests follow Rust conventions for scope and intent:

- Unit tests colocated with code and focused on implementation details
- Integration tests live under `tests/` and validate public API behavior
- Doc tests are used for public API examples and kept minimal
- Benchmarks are isolated from functional tests

**Given** I assess test efficiency and signal quality
**When** I evaluate assertions and coverage
**Then** tests avoid vanity metrics and maximize signal:

- One behavior per test, with minimal assertions
- Coverage requirements tied to defect prevention or risk areas
- Tests demonstrate real invariants, not just line coverage
- Redundant or overlapping tests are flagged for consolidation

**Given** I check for flakiness and determinism
**When** I review async and integration tests
**Then** tests are reproducible and stable:

- Async tests use timeouts and avoid blocking in async contexts
- Randomness, UUIDs, and timestamps are fixed or redacted
- Any flaky tests are tagged and scheduled for refactor/removal

**Given** snapshot testing is used
**When** I review snapshot tests
**Then** snapshots are small, named, and meaningful:

- Large blobs avoided; targeted snapshots only
- Simple scalar assertions use `assert_eq!` instead of snapshots
- Unstable fields are redacted for consistency

## Story 2.9: Create Developer Testing Documentation Guide

As a developer onboarding to Lithos testing patterns,
I want a consolidated testing documentation guide that references project rules and ADRs,
So that I can apply the approved patterns consistently and avoid ambiguity.

**Acceptance Criteria:**

**Given** I need to understand Lithos testing standards
**When** I open the developer testing guide
**Then** it documents:

- Hexagonal testing hierarchy (domain, integration, E2E)
- Async test requirements and timeouts
- Event-driven and CQRS testing patterns
- Integration testing rules and isolation requirements

**Given** I need to run tests locally or in CI
**When** I follow the guide
**Then** it provides:

- `mise run` commands for unit, integration, coverage, and benchmarks
- `nextest` usage and doc test requirements
- Coverage expectations and tarpaulin usage

**Given** I am authoring new tests
**When** I follow the guidance
**Then** the guide includes:

- Naming and description conventions (clarity checks)
- Deterministic fixture rules (fixed UUIDs/timestamps)
- Snapshot testing rules and redaction guidance
- Checklist for test isolation and anti-patterns
