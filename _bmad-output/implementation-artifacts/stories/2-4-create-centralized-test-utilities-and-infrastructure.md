# Story 2.4: create-centralized-test-utilities-and-infrastructure

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer writing tests across the codebase,
I want centralized test utilities for common testing needs,
So that tests are consistent, maintainable, and don't duplicate utility code.

## Acceptance Criteria

**Utility Provision:**
- **Given** researched test utility patterns in large Rust projects
- **When** reviewing centralized test utilities
- **Then** utilities provided for temporary directory creation/cleanup, test artifact output management, test data fixtures/factories, and common assertion helpers

**Temporary Directory Testing:**
- **Given** centralized test utilities exist
- **When** writing test needing temporary files
- **Then** can use standardized temporary directory utilities with automatic cleanup, cross-platform path handling, unique directory names, and proper error handling

**Test Data Fixtures:**
- **Given** centralized test utilities exist
- **When** writing test needing test data
- **Then** can use standardized fixture utilities with domain object factories, sample data generation, serialization helpers, and reusable test data

**Test Isolation:**
- **Given** researched test isolation best practices
- **When** checking utilities
- **Then** ensure proper test isolation with no shared state, resource cleanup, database/transaction isolation, and process isolation

## Tasks / Subtasks

- [ ] Research and establish temporary directory utilities **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Implement tempdir/tempfile integration with RAII cleanup patterns
  - [ ] Create unique naming strategy with timestamps and random suffixes
  - [ ] Build cross-platform path resolution utilities for test artifacts
  - [ ] Establish centralized test output directory with per-test subdirectories
- [ ] Implement test fixtures and factory framework **[Effort: 4-5 hours | Complexity: High]**
  - [ ] Integrate rstest for parameterized testing with fixture injection
  - [ ] Create builder patterns for fluent domain object construction
  - [ ] Implement fake data generation with configurable test scenarios
  - [ ] Build serialization helpers for JSON/binary persistence testing
- [ ] Develop custom assertion helpers **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Create custom derive macros for domain-specific assertion generation
  - [ ] Implement async assertion support with tokio timeout handling
  - [ ] Build structural comparison utilities for nested data structures
  - [ ] Add context-rich error reporting with field-level diff highlighting
- [ ] Establish test isolation infrastructure **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Create database transaction wrappers for automatic rollback
  - [ ] Implement process isolation utilities for system-level testing
  - [ ] Build RAII resource management for guaranteed cleanup
  - [ ] Add shared state prevention with thread-local or test-scoped variables
- [ ] Create integration testing utilities **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Develop shared test utilities for common setup/teardown patterns
  - [ ] Implement environment configuration for test databases and services
  - [ ] Build performance benchmarking with criterion integration
  - [ ] Create CI/CD integration with automated test execution and reporting
- [ ] Implement test output and artifact management **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Establish single configurable directory for all test artifacts
  - [ ] Create per-test subdirectory management to prevent conflicts
  - [ ] Implement artifact cleanup with configurable retention policies
  - [ ] Build debug artifact preservation for failed test investigation
- [ ] Add documentation and maintenance framework **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Create comprehensive usage documentation with code examples
  - [ ] Establish maintenance procedures for utility extension and modification
  - [ ] Implement performance monitoring for utility execution tracking
  - [ ] Develop deprecation strategies for safe utility evolution
- [ ] Integrate advanced testing capabilities **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Add property-based testing with QuickCheck integration
  - [ ] Implement chaos engineering for failure injection testing
  - [ ] Create contract testing utilities for API validation
  - [ ] Build performance profiling for test execution bottleneck identification

## Dev Notes

- **ADR 0010 Comprehensive Framework**: Implement complete test utilities framework from ADR 0010, covering temp management, fixtures, assertions, isolation, integration, and advanced capabilities.

- **Architecture Compliance**: Align with hexagonal architecture ports/adapters, async-first tokio design, and CQRS patterns from ADR 0001/0006/0009.

- **Implementation Priority**: Follow ADR 0010 structure - temp utilities (1), fixtures (2), assertions (3), isolation (4), integration (5), output management (7), documentation (6), advanced (8).

- **Source Tree Components**: Comprehensive utilities in crates/test-utils/src/ with specialized modules, domain-specific extensions, and extensive documentation.

- **Quality Assurance**: Self-testing utilities with comprehensive validation, performance monitoring, and CI/CD integration ensuring production reliability.

### Project Structure Notes

- **Alignment with unified project structure**: Utilities follow existing crate organization and naming conventions, integrating with test-utils crate architecture.

- **Detected conflicts or variances**: None - extends existing test infrastructure without breaking changes.

### Technical Requirements

**Temporary Directory and File Utilities (ADR 0010 Decision 1):**
- tempdir/tempfile crates integration with OS-specific temp handling and automatic cleanup
- Unique naming strategy with timestamp + random suffix for parallel test safety
- RAII cleanup patterns ensuring cleanup even on test panics
- Cross-platform path utilities with joining and normalization for test artifacts
- Centralized test output with single directory and per-test subdirectories
- Configurable permissions and cleanup verification mechanisms

**Test Fixtures and Factory Framework (ADR 0010 Decision 2):**
- rstest integration with #[fixture] attributes and parameterized test matrices
- Builder pattern implementation for fluent complex object construction
- Fake data generation integration with configurable realistic scenarios
- Serialization helpers for JSON/binary data in API and persistence testing
- Fixture composition with mix-and-match combinators for edge cases

**Custom Assertion Helpers (ADR 0010 Decision 3):**
- Custom derive macros with #[derive(TestAssertions)] for automatic generation
- Async assertion support with tokio timeout and cancellation handling
- Structural comparison utilities for deep equality of nested structures
- Context-rich error reporting with field-level diffs and test context information
- Domain-specific matchers for business logic validation patterns

**Test Isolation Infrastructure (ADR 0010 Decision 4):**
- Database transaction wrappers with automatic rollback for SQL/NoSQL databases
- Process isolation utilities with separate process spawning for system tests
- RAII resource management ensuring guaranteed cleanup of files/connections/memory
- Shared state prevention with thread-local or test-scoped variable management
- Environment isolation with separate config and service instances per test

**Integration Testing Utilities (ADR 0010 Decision 5):**
- Shared test utilities providing common setup/teardown functions for service initialization
- Environment configuration managing test database, cache, and external service mocking
- Performance benchmarking with criterion integration for automated regression detection
- CI/CD integration supporting automated execution, coverage reporting, and failure notifications

**Test Output and Artifact Management (ADR 0010 Decision 7):**
- Single configurable directory for all test artifacts (files, logs, dumps)
- Automatic per-test subdirectory creation to prevent cross-test conflicts
- Consistent artifact naming with timestamps and test identifiers for traceability
- Post-test cleanup with configurable retention policies for different environments
- Debug artifact preservation specifically for failed tests requiring investigation

### File Structure Requirements

- Core utilities in crates/test-utils/src/ with modules for temp, fixtures, assertions, isolation, output management
- rstest integration in crates/test-utils/src/fixtures/ with parameterized test support
- Custom assertion macros in crates/test-utils/src/assertions/ with derive support
- Test isolation utilities in crates/test-utils/src/isolation/ with database and process helpers
- Integration testing helpers in crates/test-utils/src/integration/ with environment management
- Test output management in crates/test-utils/src/output/ with artifact collection and cleanup
- Documentation in docs/testing/utilities.md with comprehensive examples and anti-patterns
- Integration tests in crates/test-utils/tests/ validating all utility categories
- CI/CD configuration in .github/workflows/ with automated utility testing and benchmarking

### Code Examples for Complex Utilities

**Test Isolation with Database Transactions:**
```rust
#[test]
async fn test_user_creation_isolation() {
    let transaction = TestTransaction::begin().await;

    // Create user within isolated transaction
    let user_id = user_service.create(create_test_user()).await.unwrap();

    // Verify in transaction scope
    let user = user_repository.find_by_id(&user_id).await.unwrap();
    assert_eq!(user.email, "test@example.com");

    // Transaction auto-rolls back - no test data pollution
}
```

**Centralized Test Output Management:**
```rust
#[test]
fn test_with_artifact_output() {
    let test_output = TestOutput::new("data_processing_test");

    // All artifacts go to centralized location
    let input_path = test_output.path().join("input.json");
    let output_path = test_output.path().join("result.txt");
    let log_path = test_output.path().join("processing.log");

    // Write test data
    std::fs::write(&input_path, r#"{"data": "test"}"#).unwrap();

    // Process and generate outputs
    process_data(&input_path, &output_path, &log_path);

    // Verify outputs exist in centralized location
    assert!(output_path.exists());
    assert!(log_path.exists());

    // Automatic cleanup unless test failed
}
```

### Testing Requirements

- Unit tests for all utility functions, macros, and framework components
- Integration tests validating utility interactions across different test scenarios
- Cross-platform testing for temp directory and path resolution utilities
- Async testing validation for tokio-compatible utilities and timeout handling
- Performance tests with benchmarking to ensure utilities don't impact test execution
- Memory leak tests using automated tools for resource management utilities
- Property-based tests for fixture generators and data validation utilities
- Chaos engineering tests for isolation and failure recovery mechanisms
- CI/CD validation tests ensuring utilities work in automated environments

### Previous Story Intelligence

- Story 2.3 established CQRS testing patterns - integrate utilities to support ADR 0009 testing framework with mock repositories and query stores
- Story 2.2 established event testing infrastructure - provide utilities that support event verification and async testing patterns
- Story 2.1 established async testing infrastructure - build upon tokio integration for async utility functions

### Git Intelligence Summary

- Recent commits show CQRS testing framework development - utilities support ADR 0009 implementation
- Test infrastructure evolution follows established patterns - maintain consistency with existing test utilities

### References

- [ADR 0010: Centralized Test Utilities](docs/adr/0010-centralized-test-utilities.md) - Comprehensive test utilities framework and implementation patterns
- [ADR 0009: CQRS Testing Patterns](docs/adr/0009-cqrs-testing-patterns.md) - CQRS-specific testing patterns integration
- [ADR 0008: Event-Driven Testing Patterns](docs/adr/0008-event-driven-testing-patterns.md) - Event testing foundation
- [Research: Rust Testing Best Practices - https://www.shuttle.dev/blog/2024/03/21/testing-in-rust]
- [Research: Test Fixtures in Rust - https://dawchihliou.github.io/articles/testing-with-fixtures-in-rust]

### Latest Tech Information

- Rust testing ecosystem: tempdir/tempfile crates with RAII cleanup, rstest for fixtures, custom derives for assertions
- Factory patterns: Builder crates with fluent APIs, fake data generation for realistic scenarios
- Assertion libraries: pretty_assertions for diffs, custom macros for domain-specific validations
- Isolation techniques: Database transactions, process isolation, RAII resource management patterns

### Project Context Reference

- Lithos requires consistent test utilities across hexagonal architecture layers
- CQRS patterns need specialized utilities for command/query separation testing
- Async-first design demands tokio-compatible utility functions
- Event-driven architecture requires utilities for event verification and flow testing

### Story Completion Status

- Status: ready-for-dev
- All acceptance criteria defined with comprehensive test utility requirements covering temp dirs, fixtures, assertions, and isolation
- Technical requirements complete with detailed implementation specifications for each utility category
- Integration points identified with existing test infrastructure and CQRS testing patterns
- Risk assessment: Low risk, builds on established testing foundations with clear extension points
- Execution Optimization: Follow ADR 0010's 8-decision framework for systematic implementation and maximum test infrastructure reliability
