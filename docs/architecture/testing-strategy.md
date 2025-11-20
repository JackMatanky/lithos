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
- **Mocking Library:** Custom mocks in `tests/utils/mocks.go` (MockCacheWriter, MockCacheReader, MockVaultReader, MockVaultWriter, MockConfigPort)
- **Coverage Requirement:** ≥85% for `internal/app`, ≥80% for `internal/adapters`

**AI Agent Requirements:**
- Generate tests for all public methods and critical private methods
- Cover happy path, edge cases, and error conditions
- Follow table-driven test patterns for multiple scenarios
- Mock all external dependencies (cache, vault, configuration, external APIs)
- Use table-driven tests for multiple scenarios where applicable
- Ensure tests are isolated and can run in parallel
- Assert domain side effects and error conditions
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

### Production Test Data

This section describes the production-scale test data available for Epic 3 vault indexing engine validation and performance testing.

**Real Obsidian Vault**: `docs/refs/obsidian/`
- **Size**: 70+ MB of real-world data (gitignored)
- **Content**: Jack's personal Obsidian vault with production frontmatter patterns
- **Structure**: Templates, projects, tools, PKM notes with diverse file classes
- **Usage**: Extract subsets for performance validation and realistic testing
- **Access**: Use `ls docs/refs/obsidian/` and `find docs/refs/obsidian/ -name "*.md"` commands
- **Guide**: See `docs/refs/obsidian-vault-guide.md` for detailed usage instructions

**Vault Structure:**
- **00_system/**: Obsidian configuration, templates, and automation scripts
  - **07_templates/**: 50+ Templater templates for various note types
  - **scripts/**: Dataview queries and Templater JavaScript functions
- **44_work/**: Work-related projects and documentation
- **70_pkm/**: Personal Knowledge Management system
  - **00_tools_and_skills/**: Technical documentation and learning materials
  - **python/**: Programming notes, scripts, and code examples

**Test Data Generation**:
```bash
# Count available notes
find docs/refs/obsidian/ -name "*.md" | wc -l

# Create large test vault (500+ notes)
mkdir -p testdata/vault-large/
find docs/refs/obsidian/ -name "*.md" | head -500 | while read file; do
  mkdir -p "testdata/vault-large/$(dirname "${file#docs/refs/obsidian/}")"
  cp "$file" "testdata/vault-large/$(dirname "${file#docs/refs/obsidian/}")/"
done

# Create diverse test vault (representative sample)
mkdir -p testdata/vault-diverse/
# Templates (diverse note types)
find docs/refs/obsidian/00_system/07_templates/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done
# Work projects
find docs/refs/obsidian/44_work/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done
# PKM notes
find docs/refs/obsidian/70_pkm/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done
```

**Expected File Classes:**
Based on vault exploration, the following file classes are commonly used:
- **Templates**: Various note type templates
- **Projects**: Work and personal projects
- **Tools**: Technical documentation
- **Knowledge**: Learning and PKM materials
- **Scripts**: Automation and utility documentation
- **Contacts**: People and organization records
- **Tasks**: Action items and workflows

**Configuration Testing:**
The vault uses various frontmatter key naming patterns:
- **Snake case**: `file_class`, `created_date`, `due_date`
- **Camel case**: `fileClass`, `createdDate`, `dueDate`
- **Mixed patterns**: Different templates use different conventions

This diversity is perfect for testing the configurable `file_class_key` feature.

**Performance Benchmarks:**
Use this data to validate Epic 3 performance targets:

- **BoltDB Hot Cache (Path Lookups)**
  - Target: <1ms average
  - Test: Path-based queries on 500+ notes
  - Command: Time individual `ByPath()` calls

- **SQLite Deep Storage (Complex Queries)**
  - Target: <50ms average
  - Test: Frontmatter property searches
  - Command: Time `ByFrontmatter()` calls with various criteria

- **Template Rendering (End-to-End)**
  - Target: <100ms total
  - Test: Full template rendering pipeline
  - Command: Time complete template execution with vault queries

- **Full Vault Indexing**
  - Target: <5s for 1000+ notes
  - Test: Complete vault indexing from scratch
  - Command: Time full vault index rebuild

- **Incremental Updates**
  - Target: <1s for typical change sets
  - Test: Staleness detection and incremental indexing
  - Command: Time incremental updates after file modifications

**Data Privacy and Usage:**
- **Content**: Personal/work vault data is included for realistic testing
- **Anonymization**: Sensitive content should be reviewed before sharing test results
- **Scope**: Use only for Lithos development and testing purposes
- **Access**: Data is gitignored and only available in local development environment

**Integration with Epic 3 Stories:**

- **Story 3.19 (BoltDB Hot Cache)**: Use vault data to test BoltDB bucket structures and validate secondary index performance
- **Story 3.20 (SQLite Deep Storage)**: Validate SQLite schema with real frontmatter patterns and test JSON_EXTRACT queries
- **Story 3.23 (Hybrid Query Service)**: Performance test query routing with realistic note distribution and validate smart routing decisions
- **Story 3.29 (FileClassKey Configuration)**: Test configurable file_class_key with actual note variations

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
  - `MockCacheWriter`, `MockCacheReader`, `MockVaultReader`, `MockVaultWriter`: In-memory implementations with configurable error injection capabilities
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
    - `templates/`: Template files for testing (`static_template.md`, `basic_note.md`, `integration_test_template.txt`, etc.)
    - `schemas/`: JSON schema fixtures organised into `valid/`, `invalid/`, `duplicate/`, and `properties/` subdirectories
    - `vault/`: Configuration samples such as `lithos.json`
    - `golden/`: Expected output artefacts kept separate from template inputs
    - Top-level snake_case fixtures retained for backward compatibility (e.g., `basic_note.md`)
  - **Golden Files:** `testdata/golden/` for expected output comparisons (`static_template_expected.md`, etc.)
  - **Schema Data:** Canonically served from `testdata/schemas/`
- **Data Loading:** Centralized loading utilities via `tests/utils/testdata.go`
  - Path resolution via `tests/utils/testdata.go` helpers for reliable discovery and naming validation
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
