# Story 4.3: add-basic-file-loading-validation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer loading configuration files,
I want basic validation of loaded data,
So that obviously malformed files are caught early with helpful error messages.

## Acceptance Criteria

1. Given files are loaded When I validate basic structure Then checks for required top-level structure and basic type consistency

2. Given validation fails When I provide error messages Then they include file path, line numbers, and suggested fixes

3. Given basic validation passes When I proceed with application-specific validation Then the data is ready for domain-specific processing

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [ ] Write failing unit tests for basic validation functions (structure checks, type consistency)
- [ ] Write failing unit tests for validation error types (file paths, line numbers, suggestions)
- [ ] Write failing integration tests for validation in file loading pipeline
- [ ] Write failing property-based tests for malformed data detection
- [ ] Write failing performance tests for validation overhead (<50μs target)
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Basic Structure Validation (GREEN Phase - AC: 1)
- [ ] Implement required top-level structure checks for each format (TOML tables, JSON objects, YAML documents)
- [ ] Implement basic type consistency validation (arrays vs objects, primitive types)
- [ ] Add validation for common structural issues (empty objects, malformed arrays)
- [ ] Create validation functions for each supported format
- [ ] **TDD REQUIREMENT:** Make all structure validation tests pass (GREEN phase complete when tests pass)

### Task 3: Implement Error Messages with Context (GREEN Phase - AC: 2)
- [ ] Implement error types with file path, line numbers, and column information
- [ ] Add suggested fixes for common validation failures
- [ ] Create user-friendly error messages for different validation scenarios
- [ ] Implement error context preservation through validation pipeline
- [ ] **TDD REQUIREMENT:** Make all error message tests pass with proper context

### Task 4: Integrate Validation with File Loading (GREEN Phase - AC: 1-3)
- [ ] Extend FileLoaderAdapter to include validation after parsing
- [ ] Implement validation toggle (enable/disable for performance)
- [ ] Add validation results to loading response
- [ ] Ensure validation errors integrate with existing FileLoaderError types
- [ ] **TDD REQUIREMENT:** Make all integration tests pass with validation enabled

### Task 5: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Optimize validation performance (<50μs overhead for basic checks)
- [ ] Add comprehensive documentation with validation examples and error scenarios
- [ ] Ensure proper error chaining and context preservation across layers
- [ ] Verify validation doesn't break existing file loading functionality
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 6: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for validation components and error handling
- [ ] Create test fixtures with valid/invalid files for all formats and validation scenarios
- [ ] Implement property-based testing for validation edge cases and error detection
- [ ] Add integration tests for validation in complete file loading workflows
- [ ] Add performance benchmarks for validation overhead (<50μs target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 7: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update adapters crate documentation with validation capabilities
- [ ] Add comprehensive doc comments with validation examples and error cases
- [ ] Ensure validation integration points are clearly documented
- [ ] Verify compatibility with future domain-specific validation layers
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 8: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch validation edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure validation performance meets <50μs targets
- [ ] **TDD VALIDATION:** Confirm comprehensive error messages with file locations and suggestions
- [ ] **TDD VALIDATION:** Verify validation doesn't break existing file loading functionality
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all validation components pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify validation integrates properly with existing FileLoaderError hierarchy
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement basic file loading validation with comprehensive error handling`

## Dev Notes

### Developer Context
This story adds basic validation to the file loading system implemented in Stories 4.1 and 4.2, ensuring that obviously malformed configuration files are caught early with helpful error messages. It extends the FileLoaderAdapter with validation capabilities while maintaining the performance and reliability established in previous stories.

**Business Value:** Prevents runtime failures from malformed configuration files by catching structural issues early, with actionable error messages that help users fix problems quickly.

**Technical Context:** Builds on the parsing infrastructure from Story 4.2, adding validation logic in the adapter layer. Must maintain <50μs validation overhead while providing comprehensive error context.

**Dependencies:** Depends on Stories 4.1 (interface) and 4.2 (parsing), enables Story 4.4 (mocks) and cross-epic configuration loading.

**Future Integration Points:**
- **Epic 5 (Configuration)**: Validation results feed into hierarchical config loading
- **Epic 6 (Schema)**: JSON schema validation builds on this basic structure validation
- **Epic 9 (Vault)**: File validation prevents corrupted note imports
- **Epic 13 (CLI)**: Validation errors provide user-friendly CLI feedback

**Risks:** Validation performance must not impact file loading speed. Error messages must be user-friendly while providing technical debugging information.

### Technical Requirements
**Core Implementation Requirements:**
- **Language**: Rust 1.92+ with comprehensive error handling
- **Architecture**: Extend FileLoaderAdapter from Stories 4.1/4.2 with validation
- **Validation Scope**: Basic structural checks, not domain-specific validation
- **Error Handling**: Enhanced FileLoaderError with validation context and suggestions
- **Performance**: <50μs validation overhead per file
- **Safety**: Zero unsafe code, comprehensive error boundaries
- **Testing**: 90%+ coverage, property-based testing for edge cases

**Validation Requirements:**
- **Structure Checks**: Required top-level elements for each format
- **Type Consistency**: Basic type validation (objects, arrays, primitives)
- **Error Context**: File paths, line numbers, column positions where possible
- **User-Friendly Messages**: Clear explanations with suggested fixes

**Integration Requirements:**
- **Adapter Extension**: Add validation to existing FileLoaderAdapter
- **Optional Validation**: Configurable validation (can be disabled for performance)
- **Error Integration**: Validation errors use existing FileLoaderError types
- **Pipeline Integration**: Validation occurs after parsing, before return

**Example Validation Error Scenarios:**
- **TOML Structure**: "Missing required top-level table in 'config.toml' - add [config] section"
- **JSON Structure**: "Root must be object in 'config.json', found array - wrap in object or use different file"
- **YAML Structure**: "Invalid YAML structure in 'config.yaml' line 1 - document must start with mapping or sequence"
- **Type Consistency**: "Expected string value in 'config.toml' line 12, found integer - use quotes for string values"
- **Required Elements**: "Missing required 'version' field in 'config.json' - add version field with semantic version string"

### Architecture Compliance Requirements
- **Hexagonal Boundary**: Validation in adapter layer, domain remains pure
- **CQRS**: Validation fits read path (file loading), no write concerns
- **Event System**: No events generated (validation is synchronous)
- **Async Patterns**: Validation integrated into existing async file loading
- **Error Hierarchy**: Validation errors extend FileLoaderError from Story 4.1
- **Naming Conventions**: Validation functions follow snake_case, error types PascalCase
- **No Unsafe Code**: Safe validation logic only
- **Clippy Compliance**: Cognitive complexity <25, all lint rules

### Library and Framework Requirements
- **Existing Dependencies**: Uses parsing libraries from Story 4.2 (toml, serde_json, serde_yaml)
- **Error Handling**: Extends thiserror usage from previous stories
- **Async Runtime**: Integrates with existing tokio patterns
- **Testing**: mockall for adapter testing, proptest for validation edge cases
Use versions from Cargo.toml workspace dependencies.

### File Structure Requirements
**Hexagonal Architecture Layout:**
```
crates/adapters/src/
├── spi/
│   └── fs/
│       └── loader.rs             # Extended FileLoaderAdapter with validation
├── dto/
│   └── loader.rs                 # Extended DTOs if needed for validation results
└── lib.rs                        # Updated re-exports

crates/domain/src/
├── ports/
│   └── spi/
│       └── loader.rs             # Extended FileLoaderError with validation variants
└── errors.rs                     # Updated error types if needed

crates/adapters/tests/
└── file_loader_validation_test.rs # Validation-specific integration tests
```

**File Organization Principles:**
- **Adapter Extension**: Validation logic added to existing loader.rs file
- **Error Extensions**: Validation errors added to existing FileLoaderError enum
- **Test Isolation**: Validation tests separate from basic loading tests
- **Modularity**: Validation functions clearly separated from parsing logic

### Testing Requirements
- **Unit Tests**: Individual validation functions for each format
- **Integration Tests**: Validation in complete file loading workflows
- **Property-Based Tests**: Edge cases in malformed file detection
- **Performance Tests**: Validation overhead measurement
- **Coverage Target**: 90%+ for validation logic and error handling
- **Test Fixtures**: Comprehensive set of valid/invalid files for testing
- **Error Testing**: All error paths and message generation

### Previous Story Intelligence
**Story 4-2 Critical Context:**
- **Parsing Infrastructure**: FileLoaderAdapter with format detection and parsing
- **Error Types**: FileLoaderError enum established for loading failures
- **Performance Baseline**: <100μs target for detection + parsing
- **Integration Points**: This story extends adapter with validation
- **Testing Patterns**: TDD framework established for adapter testing

**Story 4-1 Critical Context:**
- **Interface Contract**: FileLoaderPort defines loading contract
- **Error Hierarchy**: FileLoaderError base types for extension
- **Mock Infrastructure**: Mock implementations for testing
- **Hexagonal Boundaries**: Validation must stay in adapter layer

### Git Intelligence Summary
Recent commits show focus on testing infrastructure and CQRS patterns:
- Centralized test utilities with artifact management and isolation
- CQRS testing patterns with command/query separation
- Event flow integration tests
- Async testing with tokio::test
This story should follow established testing patterns: unit tests in adapters, integration tests with mocks, async tests for validation.

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for file validation, mock file systems, and performance benchmarking utilities

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Anti-Pattern Prevention (Critical Mistakes to Avoid)
**🚨 COMMON LLM DEVELOPER DISASTERS PREVENTED:**

- **❌ Validation in Domain**: Keep validation in adapter layer, domain stays pure
- **❌ Blocking Validation**: All validation must be CPU-bound, no I/O operations
- **❌ Generic Error Messages**: Provide specific file locations and actionable suggestions
- **❌ Performance Impact**: Maintain <50μs validation overhead, make it optional
- **❌ Incomplete Coverage**: Test all validation paths and error scenarios
- **❌ Breaking Changes**: Extend existing interfaces, don't break Story 4.1/4.2 contracts
- **❌ Over-Validation**: Basic structural checks only, leave domain validation to other layers
- **❌ Poor Error Context**: Always include file paths, line numbers, format information

**✅ CORRECT PATTERNS TO FOLLOW:**
- Hexagonal architecture: validation in adapters, domain contracts unchanged
- TDD cycle: RED (failing tests) → GREEN (minimal implementation) → REFACTOR (quality)
- Error excellence: Rich context with file locations and user-friendly messages
- Performance consciousness: Optional validation, benchmarked overhead
- Integration safety: Extend existing FileLoaderAdapter without breaking changes
- Testing thoroughness: Cover all error paths and edge cases
- Documentation clarity: Examples of validation failures and fixes

### Latest Tech Information
**Library Version Rationale (2026 Ecosystem):**
- **Existing Libraries**: Uses toml/serde_json/serde_yaml from Story 4.2
- **Error Enhancement**: Extends thiserror patterns for validation context
- **Performance Monitoring**: Criterion benchmarks for validation overhead

**Performance Benchmarks (Reference Data):**
- Basic structure validation: <10μs for typical config files
- Type consistency checks: <20μs for complex nested structures
- Error context generation: <5μs for typical validation failures
- Combined validation overhead: <50μs target
- **Performance Impact**: Validation adds ~15-25% overhead to parsing time, acceptable for config loading
- **Memory Usage**: Validation uses minimal additional memory (<1KB per file)

**Security Considerations:**
- Validation prevents processing of dangerously malformed files
- Error messages don't expose sensitive file content
- Path validation inherited from file loading security

**Migration Considerations:**
- Backward compatible: Validation is additive, can be disabled
- Error types extend existing hierarchy without breaking changes
- Performance impact is minimal and optional

### Project Structure Notes
- Alignment with unified project structure (adapters/spi/fs/ location)
- Extends Story 4.2 FileLoaderAdapter with validation capabilities
- Integration with Epic 4 file loading infrastructure
- Detected conflicts or variances (with rationale): None - follows established patterns

### References
- Epic 4 details: _bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md
- Architecture patterns: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions
- Project context: _bmad-output/project-context.md
- Testing standards: _bmad-output/project-context.md#Testing Rules
- Previous Story 4-2: _bmad-output/implementation-artifacts/stories/4-2-implement-format-detection-and-parsing.md
- Previous Story 4-1: _bmad-output/implementation-artifacts/stories/4-1-create-unified-file-loading-interface.md
- TDD Framework Examples: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Tasks-/-Subtasks
- Quality Assurance Pattern: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Task-7-Quality-Assurance-and-Commit
- Validation Report: _bmad-output/implementation-artifacts/reports/validation-report-2026-01-12-story-4-3-add-basic-file-loading-validation.md

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List


### File List

Expected files to be modified:
- crates/adapters/src/spi/fs/loader.rs (extend FileLoaderAdapter with validation)
- crates/domain/src/ports/spi/loader.rs (extend FileLoaderError with validation variants)
- crates/adapters/tests/file_loader_validation_test.rs (new validation integration tests)
