# Story 4.1: create-unified-file-loading-interface

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer implementing file loading across the application,
I want a unified interface for loading different file formats,
so that TOML, JSON, and YAML files can be loaded consistently with proper error handling.

## Acceptance Criteria

1. Given I need to load different configuration file formats When I create a unified loading interface Then it supports TOML, JSON, and YAML with automatic format detection

2. Given the unified interface exists When I load files Then format detection works by file extension or content analysis

3. Given file loading fails When I check error handling Then clear error messages indicate format issues and file locations

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [ ] Write failing unit tests for FileFormat enum (test format variants, validation)
- [ ] Write failing unit tests for FileLoadingPort trait (test method signatures, async contracts)
- [ ] Write failing unit tests for format detection functions (extension and content analysis)
- [ ] Write failing unit tests for domain error types (FileLoadingError variants)
- [ ] Write failing integration tests for FileLoadingAdapter port implementation
- [ ] Write failing property-based tests for edge cases (empty files, malformed extensions, binary content)
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Domain Entities and Ports (GREEN Phase - AC: 1-3)
- [ ] Implement FileFormat enum with TOML, JSON, YAML variants and validation
- [ ] Implement FileLoadingPort trait with async load_file method and comprehensive error types
- [ ] Implement format detection logic with extension mapping (.toml, .json, .yaml, .yml)
- [ ] Implement content-based detection for ambiguous cases (TOML [, JSON {, YAML ---)
- [ ] Implement FileLoadingError enum with thiserror::Error and descriptive messages
- [ ] Implement basic port validation (file existence, permissions, format support)
- [ ] **TDD REQUIREMENT:** Make all previously failing tests pass (GREEN phase complete when all tests pass)

### Task 3: Implement Adapter Layer Parsing (GREEN Phase - AC: 1,3)
- [ ] Implement TOML parsing in adapter using toml crate with serde integration
- [ ] Implement JSON parsing in adapter using serde_json with error mapping
- [ ] Implement YAML parsing in adapter using serde_yaml with error mapping
- [ ] Implement error translation from crate-specific errors to domain FileLoadingError
- [ ] Implement async file I/O using tokio::fs in spawn_blocking to avoid blocking threads
- [ ] Implement comprehensive error handling with file paths, line numbers, and context
- [ ] **TDD REQUIREMENT:** All parsing tests must pass with proper error propagation

### Task 4: Create File Loading Adapter Implementation (GREEN Phase - AC: 1,2,3)
- [ ] Implement FileLoadingAdapter struct implementing FileLoadingPort
- [ ] Implement format detection dispatch logic in adapter
- [ ] Implement parsing dispatch to appropriate format handlers
- [ ] Implement security validation (no binary files, size limits, path traversal protection)
- [ ] Implement caching for format detection results (optional performance optimization)
- [ ] Implement adapter-level validation (format consistency, data integrity)
- [ ] **TDD REQUIREMENT:** All adapter integration tests must pass

### Task 5: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Extract common parsing logic into reusable functions (<25 cognitive complexity)
- [ ] Optimize memory usage (avoid unnecessary allocations, use efficient string handling)
- [ ] Ensure proper error chaining and context preservation across layers
- [ ] Add comprehensive documentation with invariants, examples, and error conditions
- [ ] Implement performance optimizations (buffer reuse, efficient parsing strategies)
- [ ] Verify hexagonal architecture compliance (adapter depends on domain, domain pure)
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 6: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for file loading components and error handling
- [ ] Create test fixtures module with deterministic examples (valid/invalid files by format)
- [ ] Implement property-based testing with proptest for edge cases and boundary conditions
- [ ] Add integration tests for end-to-end file loading with various formats and error scenarios
- [ ] Add performance benchmarks (<500ms target validation)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 7: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update adapters crate lib.rs with proper public API surface and re-exports
- [ ] Add comprehensive doc comments following project standards (invariants, examples, errors)
- [ ] Ensure all ports and adapters derive required traits (Debug, Clone, PartialEq where applicable)
- [ ] Verify integration points with future bounded contexts (config loading, schema loading)
- [ ] Update Cargo.toml with required dependencies (toml, serde_json, serde_yaml)
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 8: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure performance benchmarks meet <500ms targets
- [ ] **TDD VALIDATION:** Confirm comprehensive error handling covers all failure modes
- [ ] **TDD VALIDATION:** Verify format detection accuracy for all supported formats
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all file loading components pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (adapter depends on domain)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement unified file loading interface with TDD validation`

## Dev Notes

### Developer Context
This story implements the foundational file loading infrastructure for the entire application, enabling consistent parsing of configuration files (TOML, JSON, YAML) across all components. It's part of Epic 4 (File Loading Strategy Foundation), which is critical for MVP core functionality as it enables configuration management (Epic 5) and schema loading (Epic 6).

**Business Value:** Provides the technical foundation for all file-based configuration in the application, ensuring reliability and consistency in loading user configurations, schemas, and templates.

**Technical Context:** Must follow hexagonal architecture patterns. The unified interface will be a Port in domain layer, implemented as an Adapter in adapters/spi layer. Use async patterns with Tokio, error handling with thiserror/anyhow/miette hierarchy.

**Dependencies:** None - this is the foundation story for Epic 4.

**Risks:** Format detection must be robust to prevent security issues with malformed files. Error messages must be user-friendly for debugging configuration issues.

### Technical Requirements
- Implement in Rust 1.92+ using Tokio for async operations
- Follow hexagonal architecture: domain port, adapters implementation
- Support TOML (toml crate), JSON (serde_json), YAML (serde_yaml)
- Format detection by file extension (.toml, .json, .yaml, .yml) or content analysis
- Error handling: clear messages with file paths, line numbers where possible
- Performance: <500ms for individual file loading operations
- Memory safety: no unsafe code, proper error propagation
- Testing: 90%+ coverage, unit tests with mocks

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
- Domain port: crates/domain/src/ports/file_loading.rs
- Adapter implementation: crates/adapters/src/spi/file_loading.rs
- DTOs: crates/adapters/src/dto/file_loading.rs
- Tests: crates/adapters/src/spi/file_loading.rs (unit tests)
- Integration tests: crates/adapters/tests/file_loading.rs
Follow naming conventions: mod.rs for submodules, lib.rs for crate roots.

### Testing Requirements
- Unit tests for domain logic (pure functions)
- Integration tests for adapter implementations with mocked filesystem
- Async tests with #[tokio::test]
- Performance tests with criterion for <500ms targets
- Coverage 90%+ with tarpaulin
- Mock implementations for testing different formats and error conditions
- Test fixtures for valid/invalid files in each format

### Git Intelligence Summary
Recent commits show focus on testing infrastructure and CQRS patterns:
- Centralized test utilities with artifact management and isolation
- CQRS testing patterns with command/query separation
- Event flow integration tests
- Async testing with tokio::test
This story should follow established testing patterns: unit tests in domain, integration tests with mocks, async tests for I/O operations.

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files (3-1, 3-5) to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Latest Tech Information
Latest stable versions of parsing libraries:
- toml: 0.8.19 (stable, no breaking changes)
- serde_json: 1.0.133 (stable)
- serde_yaml: 0.9.34 (stable)
All libraries have good performance and security. No migration issues.

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

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List


### File List

Expected files to be created:
- crates/domain/src/ports/file_loading.rs (FileLoadingPort trait, FileFormat enum, FileLoadingError)
- crates/domain/src/errors.rs (updated with FileLoadingError variants)
- crates/adapters/src/spi/file_loading.rs (FileLoadingAdapter implementation with TOML/JSON/YAML parsing)
- crates/adapters/src/dto/file_loading.rs (DTOs for adapter layer if needed)
- crates/adapters/src/spi/file_loading/tests.rs (unit tests for adapter)
- benches/file_loading.rs (performance benchmarks for <500ms target)
- crates/domain/tests/file_loading_integration.rs (integration tests)
