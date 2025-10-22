# Testing Strategy

## Testing Philosophy

- **Approach:** Test-driven development with table-driven tests for comprehensive scenario coverage
- **Coverage Goals:** ≥85% for `internal/app` and `internal/adapters` components, ≥80% overall project coverage
- **Test Pyramid:** Unit tests (70%), Integration tests (25%), E2E/Smoke tests (5%)
- **Quality Gates:** All tests must pass, minimum coverage thresholds enforced, no critical linting violations

## Test Types and Organization

### Unit Tests

- **Framework:** Go `testing` package (built-in)
- **File Convention:** `*_test.go` files co-located with implementation
- **Location:** Co-located `*_test.go` files near implementation code
- **Mocking Strategy:** In-memory mock implementations for all external dependencies
- **Mocking Library:** Custom mocks in `tests/utils/mocks.go` (MockFileSystemPort, MockConfigPort)
- **Coverage Requirement:** ≥85% for `internal/app`, ≥80% for `internal/adapters`

**AI Agent Requirements:**
- Generate tests for all public methods and critical private methods
- Cover happy path, edge cases, and error conditions
- Follow table-driven test patterns for multiple scenarios
- Mock all external dependencies (filesystem, configuration, external APIs)
- Use table-driven tests for multiple scenarios where applicable
- Ensure tests are isolated and can run in parallel
- Assert both `Result[T]` states and domain side effects
- Verify atomic write behavior via temp directories

### Integration Tests

- **Scope:** Cobra CLI command flows and complete template processing pipeline interactions
- **Location:** `tests/integration/` directory
- **Build Tags:** None currently used, but available for conditional execution
- **Test Infrastructure:**
  - **Filesystem:** Real filesystem operations with temporary directories and automatic cleanup
  - **Template Processing:** Full template parsing, execution, and output generation pipeline
  - **CLI Commands:** Direct service calls and Cobra command execution testing
- **Environment Setup:** Project root discovery and relative path management
- **Cleanup Strategy:** `t.TempDir()` for temporary files, `t.Cleanup()` for test isolation

**Current Tests:**
- `new_command_test.go`: CLI command integration testing
- `template_pipeline_test.go`: End-to-end template processing pipeline

**Requirements:** Ensure CLI exit codes map correctly to success/warning/error, template functions (toUpper, toLower, now) work correctly, and atomic file writes behave properly.

### End-to-End / Smoke Tests

- **Framework:** Go `testing` package with real binary execution capability
- **Scope:** Critical template processing workflows and complete CLI command execution
- **Location:** `tests/e2e/` directory
- **Environment:** Local development environment with ability to extend to CI/CD pipelines
- **Test Data:** Sample templates and expected golden file outputs
- **Execution Trigger:** Part of `just verify` workflow, extendable for release validation

**Current Implementation:** Integrated within existing integration tests but designed for expansion to dedicated release validation.

## Test Utilities and Shared Infrastructure

- **Shared Utilities Location:** `tests/utils/` package
- **Test Data Helpers:**
  - Path management utilities for consistent testdata access via `TestDataPaths` struct
  - Data loading functions (`LoadTestData`, `LoadSchemaTestData`) for fixtures and golden files
  - Runtime path resolution with `GetTestDataPath` functions
- **Mock Implementations:**
  - `MockFileSystemPort`: In-memory filesystem with configurable error injection capabilities
  - `MockConfigPort`: Mock configuration for testing different vault configurations
  - Reusable test doubles for external dependencies
- **Test Infrastructure:**
  - Project root discovery utilities (`findProjectRoot`)
  - Template processing pipeline setup and teardown
  - Golden file comparison with dynamic content handling (`compareTemplateOutputs`)
  - Temporary directory management for safe test isolation

### TestDataPaths Structure
```go
type TestDataPaths struct {
    SchemaValid      string // "schema/valid/"
    SchemaInvalid    string // "schema/invalid/"
    SchemaProperties string // "schema/properties/"
    Golden          string // "golden/"
    Templates       string // "templates/"
    Notes           string // "notes/"
}
```

### Key Utility Functions
- `LoadTestData(filename)`: Load any testdata file as string
- `LoadSchemaTestData(filename)`: Convenience wrapper for schema files
- `GetTestDataPath(filename)`: Get absolute path to testdata file
- `NewTestDataPaths()`: Create path constants for organized testdata access

## Test Data Management

- **Strategy:** Immutable fixtures with temporary copies for mutation testing
- **Test Data Location:** `testdata/` directory with organized subdirectories
- **Data Organization:**
  - **Fixtures:** `testdata/` with immutable test data organized by type
    - `templates/`: Template files for testing (static-template.md, integration-test-template.txt, etc.)
    - `schema/`: JSON schema test files organized by validity (valid/, invalid/, properties/)
    - `notes/`: Sample note files for testing
  - **Golden Files:** `testdata/golden/` for expected output comparisons (static-template-expected.md, etc.)
  - **Schema Data:** `testdata/schema/` with validity-based organization for validation test cases
- **Data Loading:** Centralized loading utilities via `tests/utils/testdata.go`
  - Runtime path resolution using `runtime.Caller()` for reliable path discovery
  - Immutable fixture handling with copy-to-temp patterns for safe mutation testing
- **Factories:** `NewTestDataPaths()` provides structured access to test data categories
- **Cleanup:** `t.Cleanup()` and `t.TempDir()` for automatic temporary file management
- **Versioning:** Test data is version-controlled alongside source code for consistency

## Continuous Testing

- **CI Integration:**
  - Unit tests on every commit via `go test ./...`
  - Integration tests as part of full test suite
  - Quality checks via `golangci-lint run` and other linting tools
- **Test Commands:** (via `justfile`)
  - **Unit:** `just test` - Run all unit tests
  - **Integration:** `just test-integration` - Run integration tests with build tags
  - **Coverage:** `just test-coverage` - Run tests with inline coverage reporting
  - **Artifacts:** `just test-artifacts` - Generate detailed HTML coverage report
  - **All:** `just verify` - Run format, lint, and test in sequence
  - **Benchmarks:** `just bench` - Run benchmark tests with memory allocation stats
- **Performance Tests:**
  - **Benchmark tests:** `go test -bench=. -benchmem ./...` for template rendering performance
  - **Load testing:** Not currently implemented, available for future expansion
  - **Performance regression detection:** Benchmark results recorded but non-blocking
- **Security Tests:**
  - **Static analysis:** `golangci-lint` with security-focused linters
  - **Dependency scanning:** Available via `gitleaks detect` for secrets detection
  - **Secret detection:** Integrated into pre-commit hooks and CI pipeline
- **Quality Gates:**
  - Minimum coverage thresholds: ≥85% for `internal/app`, ≥70% overall
  - Test pass rate requirements: 100% pass rate required for merging
  - Performance benchmark limits: Benchmarks recorded for trend analysis
- **Reporting:**
  - **Coverage reports:** HTML reports generated in `coverage/coverage.html`
  - **Test result artifacts:** JUnit XML and coverage profiles available
  - **Failure notifications:** CI pipeline integration for failure alerts

---
