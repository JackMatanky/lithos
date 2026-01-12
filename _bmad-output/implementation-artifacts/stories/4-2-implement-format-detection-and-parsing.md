# Story 4.2: implement-format-detection-and-parsing

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer parsing configuration files,
I want reliable format detection and parsing,
So that files are correctly interpreted regardless of their format.

## Acceptance Criteria

1. Given I have files in different formats When I implement parsing Then TOML files are parsed with toml crate, JSON with serde_json, YAML with serde_yaml

2. Given parsing is implemented When I test format detection Then it correctly identifies file types by extension (.toml, .json, .yaml, .yml)

3. Given files have parsing errors When I handle them Then errors include specific line numbers and syntax error details

4. Given security requirements When I validate parsing Then size limits are enforced and binary content is rejected before parsing

5. Given async architecture When I implement parsing Then CPU-bound parsing is properly handled without blocking threads

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [ ] Write failing unit tests for format detection by extension (.toml, .json, .yaml, .yml)
- [ ] Write failing unit tests for content-based format detection (TOML [, JSON {, YAML ---)
- [ ] Write failing unit tests for TOML parsing error handling with line numbers
- [ ] Write failing unit tests for JSON parsing error handling with positions
- [ ] Write failing unit tests for YAML parsing error handling with line numbers
- [ ] Write failing property-based tests for edge cases in format detection
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Format Detection Logic (GREEN Phase - AC: 2)
- [ ] Implement extension-based format detection mapping
- [ ] Implement content-based format detection for files without extensions
- [ ] Implement fallback logic: extension first, then content analysis
- [ ] Implement validation for supported formats (reject unsupported extensions)
- [ ] **TDD REQUIREMENT:** Make all format detection tests pass (GREEN phase complete when all tests pass)

### Task 3: Implement Parsing Infrastructure (GREEN Phase - AC: 1,3)
- [ ] Implement TOML parsing using toml crate with error mapping to domain errors
- [ ] Implement JSON parsing using serde_json with position tracking for errors
- [ ] Implement YAML parsing using serde_yaml with line number tracking for errors
- [ ] Implement unified error handling that preserves file location information
- [ ] Implement parsing dispatch based on detected format
- [ ] **TDD REQUIREMENT:** All parsing tests must pass with proper error propagation

### Task 4: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Extract common parsing error handling into reusable functions (<25 cognitive complexity)
- [ ] Optimize memory usage for large file parsing (streaming where possible)
- [ ] Ensure proper error chaining with context preservation across formats
- [ ] Add comprehensive documentation with parsing examples and error scenarios
- [ ] Implement performance optimizations (avoid unnecessary allocations)
- [ ] Verify hexagonal architecture compliance (parsing logic isolated in adapters)
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 5: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for format detection and parsing components
- [ ] Create test fixtures with valid and invalid files for each format
- [ ] Implement property-based testing for parsing edge cases and error conditions
- [ ] Add integration tests for end-to-end format detection and parsing
- [ ] Add performance benchmarks for parsing operations (<100μs target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 6: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update adapters crate documentation with parsing implementation details
- [ ] Add comprehensive doc comments with parsing examples and supported formats
- [ ] Ensure all parsing functions derive required traits for error handling
- [ ] Verify integration points with unified file loading interface (Story 4.1)
- [ ] Update Cargo.toml with any additional parsing dependencies if needed
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 8: Integrate Security Validations with Parsing (GREEN Phase - AC: 4)
- [ ] Extend FileLoaderAdapter to enforce size limits before parsing
- [ ] Implement binary content detection before parsing attempts
- [ ] Add security validation results to parsing error context
- [ ] Ensure security checks don't impact performance targets
- [ ] **TDD REQUIREMENT:** Make all security integration tests pass

### Task 9: Optimize Parsing for Async Architecture (GREEN Phase - AC: 5)
- [ ] Implement CPU-bound parsing without blocking async threads
- [ ] Add configurable parsing timeouts for large files
- [ ] Optimize memory usage for parsing operations
- [ ] Ensure parsing integrates properly with async file loading
- [ ] **TDD REQUIREMENT:** Make all async parsing tests pass

### Task 10: Comprehensive Security and Performance Testing (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for security validations and async parsing
- [ ] Create test fixtures for security edge cases in parsing (large files, binary content)
- [ ] Implement property-based testing for parsing security robustness
- [ ] Add performance benchmarks for secure parsing operations (<100μs target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all security tests pass

### Task 11: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure performance benchmarks meet <100μs targets
- [ ] **TDD VALIDATION:** Confirm comprehensive error handling covers all parsing failure modes
- [ ] **TDD VALIDATION:** Verify format detection accuracy for all supported formats and edge cases
- [ ] **TDD VALIDATION:** Confirm security validations prevent parsing of dangerous content
- [ ] **TDD VALIDATION:** Verify async parsing is thread-safe and performant
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all parsing components pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (parsing in adapters)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement format detection and parsing with security validations and async optimization`

## Dev Notes

### Developer Context
This story implements the core parsing logic for configuration file formats (TOML, JSON, YAML), building on the unified file loading interface from Story 4.1. It's critical for enabling configuration loading (Epic 5) and schema loading (Epic 6) with reliable format detection and comprehensive error reporting.

**Business Value:** Ensures all configuration files can be parsed correctly regardless of format, with clear error messages for debugging configuration issues.

**Technical Context:** Extends the FileLoaderAdapter from Story 4.1 with actual parsing implementations using toml, serde_json, and serde_yaml crates. Must maintain hexagonal architecture with parsing logic isolated in adapters layer.

**Dependencies:** Depends on Story 4.1 (unified interface), enables Story 4.3 (validation), Story 4.4 (mocks), and cross-epic functionality.

**Risks:** Format detection must handle edge cases (files without extensions, ambiguous content). Parsing errors must provide actionable feedback with line numbers and context.

### Technical Requirements
**Core Implementation Requirements:**
- **Language**: Rust 1.92+ with error handling crates
- **Architecture**: Extend FileLoaderAdapter from Story 4.1 with parsing implementations
- **Formats**: TOML via toml crate, JSON via serde_json, YAML via serde_yaml
- **Detection**: Extension-based (.toml, .json, .yaml, .yml) with content fallback
- **Error Handling**: Domain-level errors with file path, line numbers, and format context
- **Performance**: <100μs for format detection + parsing combined
- **Safety**: Zero unsafe code, comprehensive error boundaries
- **Testing**: 90%+ coverage, property-based testing for edge cases

**Format-Specific Requirements:**
- **TOML**: Full TOML 1.0 support with toml crate, line number error reporting
- **JSON**: Standard JSON with serde_json, position-based error reporting
- **YAML**: YAML 1.2 with serde_yaml, line number and context error reporting

**Error Handling Requirements:**
- Parse errors include exact file location (path, line, column where possible)
- Format-specific error context (e.g., "Invalid TOML table key at line 15")
- Graceful handling of malformed files without crashing
- Clear distinction between format detection failures and parsing failures

### Architecture Compliance Requirements
- **Hexagonal Boundary**: All parsing logic in adapters/spi/fs/loader.rs (no domain dependencies)
- **CQRS**: Parsing is read/write neutral, but follows async patterns for consistency
- **Event System**: No events generated in this story (parsing is synchronous)
- **Async Patterns**: File I/O uses spawn_blocking, parsing is CPU-bound but may be wrapped
- **Error Hierarchy**: Crate-specific errors → FileLoaderError → miette for CLI display
- **Naming Conventions**: FileLoaderAdapter, FileLoaderError, snake_case functions
- **No Unsafe Code**: All parsing uses safe Rust libraries
- **Clippy Compliance**: Cognitive complexity <25, all lint rules enforced

### Library and Framework Requirements
- **toml 0.8+**: Latest stable with serde integration for TOML parsing
- **serde_json 1.0+**: Standard JSON parsing with error position tracking
- **serde_yaml 0.9+**: YAML 1.2 parsing with line number reporting
- **serde 1.0+**: Core serialization traits
- **thiserror 2.0**: Domain error definitions
- **anyhow 1.0**: Adapter error context chaining
- **miette 7.6**: CLI error formatting with source code snippets
- **tokio 1.49**: Async runtime for file I/O isolation
Use versions from Cargo.toml workspace dependencies.

### File Structure Requirements
**Hexagonal Architecture Layout:**
```
crates/domain/src/
├── ports/
│   └── spi/
│       └── loader.rs             # FileLoaderPort trait (from Story 4.1)
├── errors.rs                     # FileLoaderError enum (extended)
└── lib.rs                        # Public API re-exports

crates/adapters/src/
├── spi/
│   └── fs/
│       └── loader.rs             # FileLoaderAdapter with parsing implementations
├── dto/
│   └── loader.rs                 # Data transfer objects (if needed)
└── lib.rs                        # Adapter crate re-exports

crates/domain/tests/
└── file_loader_integration_test.rs # Cross-crate integration tests

benches/
└── file_loader_bench.rs          # Performance benchmarks
```

**File Organization Principles:**
- **Domain**: Error types and port traits only
- **Adapters**: All parsing implementations and format detection logic
- **Tests**: Inline `#[cfg(test)]` for unit tests, separate files for integration
- **Naming**: Consistent `loader` prefix for all file-related components
- **Modularity**: One format parser per module section for maintainability

### Testing Requirements
- **Unit Tests**: Format detection logic, error mapping, individual parser validation
- **Integration Tests**: End-to-end parsing with FileLoaderAdapter
- **Property-Based Tests**: Fuzzing for malformed input detection
- **Performance Tests**: Benchmark parsing speed for different file sizes
- **Coverage Target**: 90%+ for all parsing and detection logic
- **Test Fixtures**: Sample valid/invalid files for TOML, JSON, YAML formats
- **Async Testing**: Tokio test for any async parsing operations

### Previous Story Intelligence
**Story 4-1 Critical Context:**
- **Unified Interface**: FileLoaderPort and FileLoaderAdapter established in Story 4.1
- **Error Hierarchy**: FileLoaderError enum defined with basic variants
- **Architecture Patterns**: Hexagonal boundaries established for file loading
- **Testing Foundations**: TDD framework established with domain tests first
- **File Structure**: loader.rs files established in domain ports and adapters
- **Dependencies**: This story extends the adapter with actual parsing implementations

### Git Intelligence Summary
Recent commits show focus on testing infrastructure and CQRS patterns:
- Centralized test utilities with artifact management and isolation
- CQRS testing patterns with command/query separation
- Event flow integration tests
- Async testing with tokio::test
This story should follow established testing patterns: unit tests in domain, integration tests with mocks, async tests for I/O operations.

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for file parsing, mock file systems, and performance benchmarking utilities

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Anti-Pattern Prevention (Critical Mistakes to Avoid)
**🚨 COMMON LLM DEVELOPER DISASTERS PREVENTED:**

- **❌ Wrong Libraries**: Only use toml/serde_json/serde_yaml - no custom parsers or deprecated serde versions
- **❌ Synchronous Parsing**: All parsing must be CPU-bound only; file I/O already handled by Story 4.1
- **❌ Domain Pollution**: Parsing logic stays in adapters, domain remains pure business logic
- **❌ Inconsistent Errors**: Use unified FileLoaderError for all parsing failures with consistent format
- **❌ Missing Error Context**: Always include file path, line numbers, and parsing context in errors
- **❌ Performance Regressions**: Maintain <100μs target, use efficient parsing libraries
- **❌ Security Vulnerabilities**: Validate file content before parsing, no arbitrary code execution
- **❌ Unhandled Parse Errors**: Test all error paths, provide actionable error messages
- **❌ Format Confusion**: Clear detection logic prevents misinterpreting file formats
- **❌ Memory Issues**: Handle large files gracefully, avoid loading entire files into memory if possible

**✅ CORRECT PATTERNS TO FOLLOW:**
- Hexagonal architecture: domain ports, adapter implementations
- TDD cycle: RED (failing tests) → GREEN (minimal implementation) → REFACTOR (quality)
- Error chaining: Crate errors → FileLoaderError → miette diagnostics
- Format detection: Extension first, content fallback, clear error messages
- Performance monitoring: Criterion benchmarks for regression prevention
- Comprehensive testing: Unit, integration, property-based, performance tests

### Latest Tech Information
**Library Version Rationale (2026 Ecosystem):**
- **toml 0.8.19**: Latest stable with full serde integration, excellent error reporting with line numbers
- **serde_json 1.0.133**: Standard JSON parsing with precise error positions for debugging
- **serde_yaml 0.9.34**: YAML 1.2 support with comprehensive error context and line tracking
- **thiserror 2.0**: Structured error definitions with source location tracking
- **anyhow 1.0**: Ergonomic error context without performance overhead

**Performance Benchmarks (Reference Data):**
- TOML parsing: ~50μs for 1KB files with error recovery
- JSON parsing: ~30μs for 1KB files with position tracking
- YAML parsing: ~200μs for 1KB files (acceptable for configuration)
- Format detection: <10μs for extension-based, <50μs for content analysis
- Combined detection + parsing: <100μs target for MVP

**Security Considerations:**
- All libraries use safe Rust with no unsafe code blocks
- Input validation prevents parsing of malicious content
- Error messages sanitized to prevent information leakage
- No external command execution or file system access in parsing

**Migration Considerations:**
- All libraries follow semantic versioning
- serde integration provides consistent API across formats
- Error handling patterns compatible with existing miette infrastructure

### Project Structure Notes
- Alignment with unified project structure (paths, modules, naming)
- Story 4.1 established loader.rs pattern in ports/spi/ and adapters/spi/fs/
- This story extends the adapter with parsing implementations
- Detected conflicts or variances (with rationale): None - follows established patterns

### References
- Epic 4 details: _bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md
- Architecture patterns: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions
- Project context: _bmad-output/project-context.md
- Testing standards: _bmad-output/project-context.md#Testing Rules
- Previous Story 4-1: _bmad-output/implementation-artifacts/stories/4-1-create-unified-file-loading-interface.md
- TDD Framework Examples: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Tasks-/-Subtasks
- Quality Assurance Pattern: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Task-7-Quality-Assurance-and-Commit
- Validation Report: _bmad-output/implementation-artifacts/reports/validation-report-2026-01-12-story-4-2-implement-format-detection-and-parsing.md

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List


### File List

Expected files to be created:
- crates/domain/src/ports/spi/loader.rs (extended with parsing-related methods if needed)
- crates/domain/src/errors.rs (extended FileLoaderError with parsing variants)
- crates/adapters/src/spi/fs/loader.rs (extended FileLoaderAdapter with parsing implementations)
- crates/adapters/src/dto/loader.rs (DTOs for parsed data if needed)
- crates/domain/tests/file_loader_integration_test.rs (integration tests)
- benches/file_loader_bench.rs (performance benchmarks)
