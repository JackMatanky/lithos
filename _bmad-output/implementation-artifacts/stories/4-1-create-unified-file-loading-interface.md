# Story 4.1: create-unified-file-loading-interface

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer implementing file loading across the application,
I want a unified interface for loading different file formats,
so that TOML, JSON, and YAML files can be loaded consistently with proper error handling.

## Acceptance Criteria

1. Given I need to load different configuration file formats When I create a unified loading interface Then it supports TOML, JSON, and YAML with automatic format detection

2. Given the unified interface exists When I load files Then format detection works by file extension or content analysis

3. Given file loading fails When I check error handling Then clear error messages indicate format issues and file locations

4. Given security requirements When I validate file loading Then path traversal is prevented, binary files are rejected, size limits are enforced, and symlinks are allowed for configuration flexibility

5. Given async architecture When I implement loading Then all I/O operations use tokio::fs in spawn_blocking for thread safety

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [x] Write failing unit tests for FileReaderPort trait (test method signatures, async contracts per docs/testing/async.md)
- [x] Write failing unit tests for domain error types (FileLoaderError variants)
- [x] Write failing unit tests for adapter format detection functions (extension and content analysis)
- [x] Write failing integration tests for FileReaderAdapter port implementation (async testing patterns per docs/testing/async.md)
- [x] Write failing property-based tests for edge cases per docs/testing/async.md (empty files, malformed extensions, binary content)
- [x] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 2: Implement Domain Entities and Ports (GREEN Phase - AC: 1-3)
- [x] Implement FileReaderPort trait with async read method returning String and comprehensive error types
- [x] Implement FileLoaderError enum with thiserror::Error and descriptive messages
- [x] Implement adapter format detection logic with extension mapping (.toml, .json, .yaml, .yml)
- [x] Implement adapter content-based detection for ambiguous cases (TOML [, JSON {, YAML ---)
- [x] Implement basic port validation (file existence, permissions, format support)
- [x] **TDD REQUIREMENT:** Make all previously failing tests pass (GREEN phase complete when all tests pass)
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 3: Implement Adapter Layer Parsing (GREEN Phase - AC: 1,3)
- [x] Implement TOML parsing in adapter using toml crate with serde integration
- [x] Implement JSON parsing in adapter using serde_json with error mapping
- [x] Implement YAML parsing in adapter using serde_yaml with error mapping
- [x] Implement error translation from crate-specific errors to domain FileLoaderError
- [x] Implement async file I/O using tokio::fs in spawn_blocking to avoid blocking threads
- [x] Implement comprehensive error handling with file paths, line numbers, and context
- [x] **TDD REQUIREMENT:** All parsing tests must pass with proper error propagation
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 4: Create File Loading Adapter Implementation (GREEN Phase - AC: 1,2,3)
- [x] Implement FileReaderAdapter struct implementing FileReaderPort trait
- [x] Implement format detection dispatch logic in adapter
- [x] Implement parsing dispatch to appropriate format handlers
- [x] Implement security validation (no binary files, size limits, path traversal protection)
- [x] Implement caching for format detection results (optional performance optimization)
- [x] Implement adapter-level validation (format consistency, data integrity)
- [x] **TDD REQUIREMENT:** All adapter integration tests must pass
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 5: Refactor for Quality (REFACTOR Phase - AC: All)
- [x] Extract common parsing logic into reusable functions (<25 cognitive complexity)
- [x] Optimize memory usage (avoid unnecessary allocations, use efficient string handling)
- [x] Ensure proper error chaining and context preservation across layers
- [x] Add comprehensive documentation with invariants, examples, and error conditions
- [x] Implement performance optimizations (buffer reuse, efficient parsing strategies)
- [x] Verify hexagonal architecture compliance (adapter depends on domain, domain pure)
- [x] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 6: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [x] Achieve sufficient test coverage for file loading components and error handling
- [x] Create test fixtures module with deterministic examples (valid/invalid files by format)
- [x] Implement property-based testing with proptest for edge cases and boundary conditions
- [x] Add integration tests for end-to-end file loading with various formats and error scenarios
- [x] Add performance benchmarks (<500ms target validation)
- [x] **TDD REQUIREMENT:** Coverage reports show sufficient coverage, all property-based tests pass
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 7: Documentation and Integration (REFACTOR Phase - AC: All)
- [x] Update adapters crate lib.rs with proper public API surface and re-exports
- [x] Add comprehensive doc comments following project standards (invariants, examples, errors)
- [x] Ensure all ports and adapters derive required traits (Debug, Clone, PartialEq where applicable)
- [x] Verify integration points with future bounded contexts (config loading, schema loading)
- [x] Update Cargo.toml with required dependencies (toml, serde_json, serde_yaml)
- [x] **TDD REQUIREMENT:** All documentation examples compile and run successfully
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 9: Implement Security Validations (GREEN Phase - AC: 4)
- [x] Implement path sanitization (reject absolute paths, path traversal attempts via .. components)
- [x] Allow symlinks for dotfile flexibility (documented in ADR 0015 - symlinks followed transparently)
- [x] Add binary file detection and rejection (check for null bytes, non-UTF-8 content)
- [x] Implement configurable size limits (default 10MB, configurable via adapter settings)
- [x] Add security validation to FileReaderPort contract
- [x] **TDD REQUIREMENT:** Make all security validation tests pass
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 10: Add Async I/O Support (GREEN Phase - AC: 5)
- [x] Update FileReaderPort to use async trait methods
- [x] Implement tokio::fs operations in spawn_blocking for thread safety
- [x] Add async timeout handling for file operations
- [x] Ensure async error propagation maintains context
- [x] **TDD REQUIREMENT:** Make all async I/O tests pass
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 11: Comprehensive Security and Performance Testing (RED-GREEN-REFACTOR - AC: All)
- [x] Achieve sufficient test coverage for security validations and async operations
- [x] Create test fixtures for security edge cases (path traversal, binary files, large files)
- [x] Implement property-based testing for security validation robustness
- [x] Add performance benchmarks for async file loading (<500ms target)
- [x] **TDD REQUIREMENT:** Coverage reports show sufficient coverage, all security tests pass
- [x] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise

### Task 12: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [x] **TDD VALIDATION:** Confirm all tests pass and coverage meets sufficient requirement
- [x] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [x] **TDD VALIDATION:** Ensure performance benchmarks meet <500ms targets
- [x] **TDD VALIDATION:** Confirm comprehensive error handling covers all failure modes
- [x] **TDD VALIDATION:** Verify format detection accuracy for all supported formats
- [x] **TDD VALIDATION:** Confirm security validations prevent all identified attack vectors
- [x] **TDD VALIDATION:** Verify async operations are thread-safe and properly isolated
- [x] Run `mise run fmt` to format all code according to project standards
- [x] Run `mise run lint` to check for all code quality issues and anti-patterns
- [x] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [x] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [x] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [x] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [x] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [x] **MANDATORY:** Confirm all file loading components pass clippy cognitive complexity limits (<25)
- [x] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [x] **MANDATORY:** Verify hexagonal architecture boundaries maintained (adapter depends on domain)
- [x] Stage all files created or modified during story development
- [x] Commit with conventional commit message: `feat: implement unified file loading interface with security validations and async I/O`

## Dev Notes

### Developer Context
This story implements the foundational file loading infrastructure for the entire application, enabling consistent parsing of configuration files (TOML, JSON, YAML) across all components. It's part of Epic 4 (File Loading Strategy Foundation), which is critical for MVP core functionality as it enables configuration management (Epic 5) and schema loading (Epic 6).

**Business Value:** Provides the technical foundation for all file-based configuration in the application, ensuring reliability and consistency in loading user configurations, schemas, and templates.

**Technical Context:** Must follow hexagonal architecture patterns. The unified interface will be a Port in domain layer, implemented as an Adapter in adapters/spi layer. Use async patterns with Tokio, error handling with thiserror/anyhow/miette hierarchy.

**Dependencies:** None - this is the foundation story for Epic 4.

**Risks:** Format detection must be robust to prevent security issues with malformed files. Error messages must be user-friendly for debugging configuration issues.

### Technical Requirements
**Core Implementation Requirements:**
- **Language**: Rust 1.92+ with Tokio 1.49 for async runtime
- **Architecture**: Hexagonal pattern - domain ports, adapter implementations
- **Formats**: TOML, JSON, YAML with automatic format detection (adapter-owned)
- **Detection Logic**: File extension first (.toml, .json, .yaml, .yml), content analysis fallback (adapter-owned)
- **Port Contract**: FileReaderPort returns UTF-8 text (String), not FileFormat
- **Error Handling**: Hierarchical errors (domain → adapter → CLI) with rich diagnostics
- **Performance**: <500ms for individual file loading, <100μs for format detection + parsing
- **Safety**: Zero unsafe code, no unwrap/expect, comprehensive error propagation
- **Testing**: Sufficient coverage, TDD framework, property-based testing

**Format-Specific Requirements:**
- **TOML**: Use toml crate, support full TOML 1.0 specification
- **JSON**: Use serde_json, support standard JSON with comments (if needed)
- **YAML**: Use serde_yaml, support YAML 1.2 with anchors/aliases

**Security Requirements:**
- Path validation: Reject absolute paths, path traversal attempts via .. components
- Symlink handling: Allow symlinks for dotfile flexibility (ADR 0015 - security via content validation)
- Content validation: Reject binary files, enforce reasonable size limits
- Error sanitization: No sensitive information in error messages

**Example Error Scenarios to Handle:**
- File not found: "Configuration file 'config.toml' not found at /path/to/file"
- Permission denied: "Cannot read configuration file 'config.json' - permission denied"
- Malformed content: "Invalid TOML syntax in 'config.toml' at line 15, column 23: expected table key"
- Unsupported format: "Unknown file format for 'config.xyz' - supported: .toml, .json, .yaml"
- Empty file: "Configuration file 'config.toml' is empty"

### Architecture Compliance Requirements
- Hexagonal boundary: domain contains only business logic, no I/O
- CQRS: if this involves data operations, separate read/write concerns
- Event system: use hybrid bus if events are generated
- Async patterns: all I/O in spawn_blocking, no blocking in async fns
- Error hierarchy: thiserror in domain, anyhow in adapters, miette in cli
- Naming conventions: snake_case functions, PascalCase structs, etc.
- No unsafe code
- Clippy compliance: cognitive complexity <25

### Library and Framework Requirements
- toml: 0.8+ (latest stable)
- serde_json: 1.0+
- serde_yaml: 0.9+
- serde: 1.0+ for derive
- thiserror: 2.0
- anyhow: 1.0
- tokio: 1.49 full
- miette: 7.6
Use versions from Cargo.toml workspace dependencies.

### File Structure Requirements
**Hexagonal Architecture Layout:**
```
crates/domain/src/
├── ports/
│   └── spi/
│       └── fs.rs                 # FileReader trait (aliased as FileReaderPort) + FileLoaderError
├── errors.rs                     # Updated with FileLoaderError variants
└── lib.rs                        # Public API re-exports

crates/adapters/src/
├── spi/
│   └── fs/
│       └── loader.rs             # FileReader implementation with inline #[cfg(test)] unit tests
└── lib.rs                        # Adapter crate re-exports

crates/domain/tests/
└── file_loader_integration.rs    # Cross-crate integration tests (moved to unit tests)

benches/
└── file_loader.rs                # Criterion performance benchmarks (to be implemented)
```

**File Organization Principles:**
- Domain: Pure business logic, zero I/O, zero external dependencies
- Adapters: Infrastructure implementations, error translation, async I/O
- Tests: Inline `#[cfg(test)]` for domain, separate files for integration
- Naming: snake_case files, PascalCase types, SCREAMING_SNAKE_CASE constants
- Modularity: One concept per file, clear module boundaries

### Testing Requirements
- Unit tests for domain logic (pure functions)
- Integration tests for adapter implementations with mocked filesystem
- Async tests with #[tokio::test]
- Performance tests with criterion for <500ms targets
- Coverage sufficient with tarpaulin
- Mock implementations for testing different formats and error conditions
- Test fixtures for valid/invalid files in each format

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
- **Integration Points**: Use shared test fixtures, mock repositories, and performance benchmarking utilities

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files (3-1, 3-5) to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Anti-Pattern Prevention (Critical Mistakes to Avoid)
**🚨 COMMON LLM DEVELOPER DISASTERS PREVENTED:**

- **❌ Wrong Libraries**: Only use toml/serde_json/serde_yaml - no custom parsers or deprecated crates
- **❌ Synchronous I/O**: All file operations must use `tokio::fs` in `spawn_blocking` - never block async threads
- **❌ Domain Pollution**: File I/O logic stays in adapters, domain remains pure business logic; domain port returns UTF-8 text only
- **❌ Inconsistent Error Handling**: Use thiserror in domain, anyhow in adapters, miette in CLI
- **❌ Missing Format Detection**: Always check file extension first, fall back to content analysis (adapter)
- **❌ No Error Context**: Include file paths, line numbers, and format type in all error messages
- **❌ Performance Regressions**: Maintain <500ms target, use benchmarks to prevent degradation
- **❌ Security Vulnerabilities**: Validate file paths, reject binary files, prevent path traversal
- **❌ Unhandled Edge Cases**: Test empty files, malformed content, permission errors, large files

**✅ CORRECT PATTERNS TO FOLLOW:**
- Hexagonal architecture: domain ports, adapter implementations
- TDD cycle: RED (failing tests) → GREEN (minimal implementation) → REFACTOR (quality)
- Async safety: `spawn_blocking` for I/O, no blocking in async functions
- Error chaining: Domain errors → Adapter translation → CLI presentation
- Performance monitoring: Criterion benchmarks for regression prevention

### Latest Tech Information
**Library Version Rationale (2026 Ecosystem):**
- **toml 0.8.19**: Latest stable with full serde integration, no breaking changes since 0.7, excellent performance for configuration parsing
- **serde_json 1.0.133**: Stable 1.0 release with zero-copy parsing capabilities, proven security track record
- **serde_yaml 0.9.34**: Latest stable with comprehensive YAML 1.2 support, maintains compatibility with existing schemas

**Performance Benchmarks (Reference Data):**
- toml parsing: ~50μs for 1KB config files
- serde_json: ~30μs for 1KB JSON files
- serde_yaml: ~200μs for 1KB YAML files (acceptable for config loading)
- Combined format detection + parsing: <100μs target for MVP

**Security Considerations:**
- All libraries use safe Rust with no unsafe code blocks
- No known CVEs in current versions
- Input validation prevents malformed file attacks
- Path traversal protection implemented at adapter level

**Migration Considerations:**
- No breaking changes required from current versions
- serde integration provides consistent API across all formats
- Error handling patterns established in architecture ADR 0006

### Project Structure Notes
- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### References
- Epic 4 details: _bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md
- Architecture patterns: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions
- Project context: _bmad-output/project-context.md
- Testing standards: _bmad-output/project-context.md#Testing Rules
- TDD Framework Examples: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Tasks-/-Subtasks
- Quality Assurance Pattern: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Task-7-Quality-Assurance-and-Commit
- Validation Report: _bmad-output/implementation-artifacts/reports/validation-report-2026-01-12-story-4-1-create-unified-file-loading-interface.md

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List

#### Architectural Pivot (2026-01-22)

**Realization:**
During the implementation of Task 2 ("Implement Domain Entities and Ports"), we realized that defining a generic `FileReaderPort` in the domain layer (Story 4.1) created an unnecessary abstraction that conflated two distinct responsibilities:
1.  **File I/O**: Reading bytes from a file system (an infrastructure concern).
2.  **Parsing**: Interpreting those bytes as structured data (TOML/JSON/YAML).

The initial plan forced all file reading to go through this single domain port, which would require the domain to know about "files" and "paths" more than necessary. It also created a rigid coupling where every file read *had* to use this specific port, rather than allowing adapters to use standard `tokio::fs` or other sources.

**Factors Leading to Decision:**
1.  **Single Responsibility Principle (SRP)**: Parsing strategy is a distinct logical operation from the mechanism of retrieving bytes (I/O). Bundling them made the "Reader" responsible for too much.
2.  **Hexagonal Architecture Purity**: The domain should define *what* data it needs (e.g., `ConfigQuery::load_global()`), not *how* that data is retrieved from a file system. A `FileReaderPort` leaked the "how" (file system) into the domain interface definitions.
3.  **Adapter Responsibility**: In Hexagonal Architecture, adapters are responsible for the "dirty" work of I/O. It is more idiomatic for a `ConfigAdapter` to directly use `tokio::fs` (for I/O) and then delegate to a `ParserStrategy` (for interpretation) than to go through an intermediate `FileReaderPort`.
4.  **Testability**: Testing a pure `ParserStrategy` (string -> struct) is significantly easier and faster than testing a `FileReader` (path -> struct) because it removes the file system dependency entirely from the unit tests.

**Decision:**
We pivoted to implementing a **Parser Utility** (`ParserStrategy` enum + `Dispatcher`) in the `adapters/spi` layer. This utility handles the complexity of format detection and parsing but leaves the I/O to the specific adapters that need it. This simplifies the domain (removing the `fs` port) and makes the parsing logic reusable without binding it to a specific I/O implementation.

### File List

Expected files to be created:
- crates/adapters/src/spi/parsers.rs (Parser strategies and Dispatcher)
- crates/adapters/src/spi/errors.rs (ParseError implementation)
- crates/adapters/src/spi/mod.rs (re-exports)
- crates/adapters/src/lib.rs (crate exports)
- crates/adapters/Cargo.toml (dependencies)
