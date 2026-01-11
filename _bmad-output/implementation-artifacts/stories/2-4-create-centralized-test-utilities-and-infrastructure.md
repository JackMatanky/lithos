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

- [ ] Research test utility patterns and establish framework **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Analyze existing test utilities in crates/test-utils/ for extension points
  - [ ] Study temporary directory patterns in Rust testing ecosystem
  - [ ] Review fixture and factory patterns for domain objects
  - [ ] Examine assertion helper libraries and best practices
- [ ] Create temporary directory and file utilities **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Implement TempDir utility with automatic cleanup and cross-platform support
  - [ ] Create unique directory naming to prevent test conflicts
  - [ ] Add error handling for directory operations and cleanup failures
  - [ ] Build test artifact output management with path resolution
- [ ] Develop test data fixtures and factories **[Effort: 4-5 hours | Complexity: High]**
  - [ ] Create domain object factories with valid defaults and configuration
  - [ ] Implement sample data generators for various test scenarios
  - [ ] Build serialization helpers for complex object persistence
  - [ ] Develop reusable test data builders and combinators
- [ ] Establish common assertion helpers **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Create domain-specific assertion macros and functions
  - [ ] Implement async assertion helpers for tokio-based tests
  - [ ] Build comparison utilities for complex data structures
  - [ ] Add custom failure messages and debugging helpers
- [ ] Implement test isolation utilities **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Create database transaction isolation helpers for integration tests
  - [ ] Implement process isolation utilities for system tests
  - [ ] Build resource cleanup managers with RAII patterns
  - [ ] Add shared state prevention mechanisms
- [ ] Integrate utilities with existing test framework **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Update test configuration to include new utilities
  - [ ] Create documentation and usage examples
  - [ ] Add CI/CD integration for utility validation
  - [ ] Establish maintenance and extension guidelines

## Dev Notes

- **ADR 0009 CQRS Testing Integration**: Build utilities that support ADR 0009 CQRS testing patterns, providing mock repositories, query stores, and event verification foundations.

- **Architecture Compliance**: Create utilities that align with hexagonal architecture (ports/adapters), async-first design, and Lithos' CQRS patterns from ADR 001 and ADR 006.

- **Implementation Priority**: Start with temporary directory utilities (Priority 1), then fixtures/factories (Priority 2), followed by assertion helpers and isolation (Priority 3-4).

- **Source Tree Components**: Core utilities in crates/test-utils/src/, domain-specific helpers in domain crates, documentation in docs/testing/utilities.md.

- **Quality Assurance**: Utilities include comprehensive testing themselves, with integration tests ensuring reliability across different test scenarios.

### Project Structure Notes

- **Alignment with unified project structure**: Utilities follow existing crate organization and naming conventions, integrating with test-utils crate architecture.

- **Detected conflicts or variances**: None - extends existing test infrastructure without breaking changes.

### Technical Requirements

**Temporary Directory Management:**
- Cross-platform temp directory creation with automatic cleanup
- Unique naming to prevent test interference
- Path resolution utilities for test artifacts
- Error handling and recovery mechanisms

**Test Data Fixtures:**
- Domain object factories with builder patterns
- Configurable test data generators
- Serialization helpers for persistence scenarios
- Reusable fixture combinators and modifiers

**Assertion Helpers:**
- Domain-specific assertion functions
- Async assertion support for tokio tests
- Structural comparison utilities
- Enhanced error reporting and debugging

**Test Isolation:**
- Database transaction wrappers for integration tests
- Process isolation helpers for system tests
- Resource management with RAII patterns
- Shared state prevention mechanisms

### File Structure Requirements

- Core utilities in crates/test-utils/src/utilities/ with modules for temp, fixtures, assertions, isolation
- Domain-specific factories in crates/test-utils/src/fixtures/ organized by bounded context
- Documentation in docs/testing/utilities.md with usage examples and best practices
- Integration tests in crates/test-utils/tests/ validating utility functionality
- CI/CD configuration in .github/workflows/ for utility validation

### Testing Requirements

- Unit tests for all utility functions and classes
- Integration tests validating utility interactions
- Cross-platform testing for temp directory utilities
- Performance tests ensuring utilities don't impact test execution speed
- Memory leak tests for resource management utilities

### Previous Story Intelligence

- Story 2.3 established CQRS testing patterns - integrate utilities to support ADR 0009 testing framework with mock repositories and query stores
- Story 2.2 established event testing infrastructure - provide utilities that support event verification and async testing patterns
- Story 2.1 established async testing infrastructure - build upon tokio integration for async utility functions

### Git Intelligence Summary

- Recent commits show CQRS testing framework development - utilities support ADR 0009 implementation
- Test infrastructure evolution follows established patterns - maintain consistency with existing test utilities

### Latest Tech Information

- Rust testing ecosystem: tempdir and tempfile crates provide foundation for temporary directory management
- Factory patterns: builder crates offer ergonomic test data construction
- Assertion libraries: custom derive macros enable domain-specific test assertions
- Isolation techniques: RAII patterns and tokio utilities support clean test isolation

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
- Execution Optimization: Follow structured priority approach with comprehensive utility coverage for maximum developer productivity
