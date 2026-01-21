# Story 4.4: create-loading-strategy-mocks-for-testing

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer testing file loading functionality,
I want mocks for the loading strategy,
So that I can test file loading in isolation without actual file system operations.

## Acceptance Criteria

1. Given I need to test loading strategies When I create mocks Then mock implementations allow testing different file formats and error conditions

2. Given mocks are available When I write unit tests Then tests can verify loading logic without file system dependencies

3. Given integration tests are needed When I use mocks Then they simulate real file loading behavior for comprehensive testing

4. Given security testing requirements When I create mocks Then mocks simulate security validations and attack scenarios

5. Given async testing needs When I implement mocks Then mocks support async trait methods and proper isolation

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Mock Tests First (RED Phase - AC: All)
- [ ] Write failing tests for mock FileLoaderAdapter behavior (success/error scenarios)
- [ ] Write failing tests for mock file system interactions (permissions, not found, corrupted data)
- [ ] Write failing tests for mock format detection responses (TOML, JSON, YAML variations)
- [ ] Write failing integration tests using mocks in adapter testing scenarios
- [ ] Write failing property-based tests for mock data generation and validation per @docs/testing/developer-guide.md
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Mock FileLoaderAdapter (GREEN Phase - AC: 1,2,3)
- [ ] Create MockFileLoaderAdapter struct implementing FileLoaderAdapter trait
- [ ] Implement configurable mock behaviors for different file formats and error conditions
- [ ] Add mock state management for tracking method calls and return values
- [ ] Implement mock file system isolation (no actual I/O operations)
- [ ] **TDD REQUIREMENT:** Make all mock implementation tests pass (GREEN phase complete when tests pass)

### Task 3: Create Comprehensive Test Fixtures (GREEN Phase - AC: 1-3)
- [ ] Create mock file data fixtures for all supported formats (valid/invalid TOML, JSON, YAML)
- [ ] Implement fixture factories for generating test scenarios (permissions, corruption, large files)
- [ ] Create mock configuration objects for different adapter settings
- [ ] Add fixture validation to ensure test data integrity
- [ ] **TDD REQUIREMENT:** Make all fixture-related tests pass

### Task 4: Integrate Mocks with Existing Tests (GREEN Phase - AC: 2,3)
- [ ] Refactor existing FileLoaderAdapter tests to use mocks where appropriate
- [ ] Create integration test suites using mock adapters for isolated testing
- [ ] Implement mock verification for test assertions (call counts, parameter validation)
- [ ] Add mock-based property tests for edge cases and error conditions
- [ ] **TDD REQUIREMENT:** Make all integration tests with mocks pass

### Task 5: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Optimize mock performance for fast test execution (<1ms per mock operation)
- [ ] Add comprehensive documentation with mock usage examples
- [ ] Ensure mock implementations are thread-safe for parallel testing
- [ ] Verify mock isolation (no cross-test contamination)
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 6: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for mock implementations and fixture code
- [ ] Create extensive mock scenarios covering all FileLoaderAdapter error paths
- [ ] Implement property-based testing with mock data generation
- [ ] Add performance benchmarks for mock vs real adapter comparison
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 7: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update adapters crate documentation with mock usage patterns
- [ ] Create mock testing guide for future story implementations
- [ ] Ensure mock APIs are stable for cross-story usage
- [ ] Verify integration points with Story 4.3 validation testing
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 9: Add Security Scenario Mocks (GREEN Phase - AC: 4)
- [ ] Implement mock scenarios for security validations (path traversal, large files, binary content)
- [ ] Create mock fixtures for attack simulation (directory traversal attempts, oversized files)
- [ ] Add security error simulation in mock implementations
- [ ] Ensure security mocks accurately reflect real adapter behavior
- [ ] **TDD REQUIREMENT:** Make all security mock tests pass

### Task 10: Implement Async Mock Support (GREEN Phase - AC: 5)
- [ ] Update mock implementations to support async trait methods
- [ ] Implement proper async isolation for mock testing
- [ ] Add mock timeout simulation for async operations
- [ ] Ensure mocks are thread-safe for parallel test execution
- [ ] **TDD REQUIREMENT:** Make all async mock tests pass

### Task 11: Comprehensive Security and Async Testing (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for security mocks and async support
- [ ] Create test fixtures for security and async edge cases in mocking
- [ ] Implement property-based testing for mock robustness
- [ ] Add performance benchmarks for secure mock operations (<1ms target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all security/async tests pass

### Task 12: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure mock performance meets targets (<1ms per operation)
- [ ] **TDD VALIDATION:** Confirm comprehensive mock coverage for all adapter scenarios
- [ ] **TDD VALIDATION:** Verify mock isolation and thread-safety for parallel testing
- [ ] **TDD VALIDATION:** Confirm security mocks prevent testing blindspots
- [ ] **TDD VALIDATION:** Verify async mocks support proper testing isolation
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all mock components pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify mock implementations maintain hexagonal boundaries
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement loading strategy mocks with security scenarios and async support`

## Dev Notes

### Developer Context
This story creates mock implementations for the file loading strategy developed in Stories 4.1-4.3, enabling comprehensive testing without file system dependencies. It provides test doubles for isolated unit testing and integration testing of file loading functionality across all supported formats.

**Business Value:** Enables fast, reliable testing of file loading logic without external dependencies, ensuring robust error handling and format detection through comprehensive test scenarios.

**Technical Context:** Builds on the FileLoaderAdapter from previous stories, creating mock implementations that simulate all file system interactions and error conditions for thorough testing coverage.

**Dependencies:** Depends on Stories 4.1 (interface), 4.2 (parsing), 4.3 (validation), enables Story 4.5 (test suite review).

**Risks:** Mock implementations must accurately reflect real adapter behavior. Test isolation must prevent cross-contamination between test runs.

### Technical Requirements
**Core Implementation Requirements:**
- **Language**: Rust 1.92+ with mockall for trait mocking
- **Architecture**: Mock implementations following hexagonal testing patterns
- **Mock Framework**: mockall crate for comprehensive trait mocking
- **Isolation**: Complete file system isolation for deterministic testing
- **Performance**: <1ms per mock operation for fast test execution
- **Thread Safety**: Mock implementations safe for parallel test execution
- **Documentation**: Comprehensive usage examples for all mock scenarios

**Mock Design Requirements:**
- **FileLoaderAdapter Mock**: Full trait implementation with configurable behaviors
- **Error Simulation**: All error conditions from real adapter (permissions, corruption, format errors)
- **State Tracking**: Method call verification and parameter capture
- **Fixture Integration**: Pre-built test data for common scenarios

**Integration Requirements:**
- **Test Replacement**: Easy substitution for real adapters in unit tests
- **Integration Testing**: Mock-based testing for adapter validation
- **Cross-Story Usage**: Stable APIs for future story testing needs

### Architecture Compliance Requirements
- **Hexagonal Boundary**: Mocks follow same interface as real adapters
- **CQRS**: Mock support for both read/write file operations
- **Event System**: Mock event publishing if needed
- **Async Patterns**: Mock async methods matching real adapter signatures
- **Error Hierarchy**: Mock errors compatible with FileLoaderError types
- **Naming Conventions**: MockFileLoaderAdapter, mock_ prefixed test functions
- **No Unsafe Code**: Safe mock implementations
- **Clippy Compliance**: Cognitive complexity <25, all lint rules

### Library and Framework Requirements
- **mockall 0.12+**: Primary mocking framework for trait implementations
- **tokio 1.49**: For async mock method support
- **serde 1.0+**: For mock data serialization/deserialization
- **thiserror 2.0**: For mock error type definitions
- **async-trait 0.1+**: For async mock trait methods
Use versions from Cargo.toml workspace dependencies.

### File Structure Requirements
**Hexagonal Architecture Layout:**
```
crates/adapters/src/
├── spi/
│   └── fs/
│       └── loader.rs             # Extend with mock implementations inline
├── dto/
│   └── loader.rs                 # Mock DTOs if needed
└── lib.rs                        # Export mock types

crates/adapters/tests/
├── file_loader_mock_test.rs      # Mock-specific unit tests
└── file_loader_integration_mock_test.rs # Mock-based integration tests
```

**File Organization Principles:**
- **Inline Mocks**: Mock implementations in same file as real adapters
- **Test Isolation**: Separate test files for mock-specific testing
- **Fixture Organization**: Test fixtures co-located with mock implementations
- **Modularity**: Clear separation between real and mock implementations

### Testing Requirements
- **Unit Tests**: Mock behavior verification and fixture validation
- **Integration Tests**: Mock-based testing of adapter interactions
- **Property-Based Tests**: Mock data generation and validation
- **Performance Tests**: Mock operation timing verification
- **Coverage Target**: 90%+ for mock implementations and test utilities
- **Isolation Testing**: Verify mock thread-safety and parallel execution
- **Fixture Testing**: Validate test data integrity and completeness

### Previous Story Intelligence
**Story 4-3 Critical Context:**
- **Validation Integration**: Mocks must support validation testing scenarios
- **Error Handling**: Mock error simulation for validation failure testing
- **Performance Baseline**: Mock overhead must be negligible compared to real adapter

**Story 4-2 Critical Context:**
- **Parsing Integration**: Mocks must simulate format detection and parsing results
- **Error Scenarios**: Mock support for all parsing error conditions

**Story 4-1 Critical Context:**
- **Interface Compliance**: Mocks must implement FileReaderPort/FileReaderAdapter traits- **Async Contracts**: Mock async method signatures matching real implementations

### Git Intelligence Summary
Recent commits show focus on mise task configuration and testing infrastructure:
- Mise variable templating for configurable paths
- Shellcheck integration with selective disables
- Test task orchestration improvements
- Task script standardization
This story should follow established testing patterns: unit tests in adapters, integration tests with mocks, async tests for I/O operations.

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for file loading mocks, mock file systems, and performance benchmarking utilities

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Anti-Pattern Prevention (Critical Mistakes to Avoid)
**🚨 COMMON LLM DEVELOPER DISASTERS PREVENTED:**

- **❌ Incomplete Mock Coverage**: Mocks must cover all FileLoaderAdapter methods and error paths
- **❌ Mock State Contamination**: Ensure mock isolation between test runs
- **❌ Performance Issues**: Mock overhead must not slow down test execution
- **❌ Inaccurate Simulation**: Mocks must accurately reflect real adapter behavior
- **❌ Thread Safety Issues**: Mocks must be safe for parallel test execution
- **❌ Missing Fixtures**: Comprehensive test data for all supported formats and errors
- **❌ Poor Documentation**: Clear usage examples for mock implementation
- **❌ Breaking Changes**: Mock APIs must remain stable for cross-story usage

**✅ CORRECT PATTERNS TO FOLLOW:**
- Hexagonal architecture: mock implementations follow real adapter interfaces
- TDD cycle: RED (failing tests) → GREEN (minimal implementation) → REFACTOR (quality)
- Isolation first: Complete test isolation with no external dependencies
- Performance conscious: Fast mock operations for rapid test execution
- Documentation driven: Comprehensive examples for all mock usage scenarios
- Thread safe: Parallel test execution support
- Fixture rich: Extensive test data covering all edge cases

### Latest Tech Information
**Library Version Rationale (2026 Ecosystem):**
- **mockall 0.12.1**: Latest stable with comprehensive async trait mocking
- **tokio 1.49**: Async runtime for mock async method support
- **serde 1.0.133**: Data serialization for mock fixture management

**Performance Benchmarks (Reference Data):**
- Mock method dispatch: <100μs overhead vs real implementation
- Mock creation/setup: <50μs per test scenario
- Mock verification: <10μs per assertion
- Combined mock test: <1ms total execution time

**Security Considerations:**
- Mock implementations don't introduce security vulnerabilities
- Test data sanitized to prevent accidental exposure
- No external dependencies in mock code

**Migration Considerations:**
- mockall API stable across versions
- Compatible with existing tokio async patterns
- No breaking changes in serde fixture handling

### Project Structure Notes
- Alignment with unified project structure (adapters/spi/fs/ location)
- Mock implementations extend existing FileLoaderAdapter file
- Integration with Stories 4.1-4.3 for comprehensive testing
- Detected conflicts or variances (with rationale): None - follows established patterns

### References
- Epic 4 details: _bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md
- Architecture patterns: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions
- Project context: _bmad-output/project-context.md
- Testing standards: _bmad-output/project-context.md#Testing Rules
- Previous Story 4-3: _bmad-output/implementation-artifacts/stories/4-3-add-basic-file-loading-validation.md
- Previous Story 4-2: _bmad-output/implementation-artifacts/stories/4-2-implement-format-detection-and-parsing.md
- Previous Story 4-1: _bmad-output/implementation-artifacts/stories/4-1-create-unified-file-loading-interface.md
- TDD Framework Examples: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Tasks-/-Subtasks
- Quality Assurance Pattern: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Task-7-Quality-Assurance-and-Commit
- Validation Report: _bmad-output/implementation-artifacts/reports/validation-report-2026-01-12-story-4-4-create-loading-strategy-mocks-for-testing.md

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List


### File List

Expected files to be modified:
- crates/adapters/src/spi/fs/loader.rs (extend with mock implementations)
- crates/adapters/tests/file_loader_mock_test.rs (new mock unit tests)
- crates/adapters/tests/file_loader_integration_mock_test.rs (new mock integration tests)
