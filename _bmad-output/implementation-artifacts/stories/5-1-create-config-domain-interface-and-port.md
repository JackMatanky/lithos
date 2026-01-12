# Story 5.1: create-config-domain-interface-and-port

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer implementing configuration management,
I want a clean domain interface for configuration loading,
So that configuration can be loaded through a well-defined contract following hexagonal architecture.

## Acceptance Criteria

1. Given I need to define configuration contracts When I create the Config domain interface Then it includes ConfigPort trait with async load method

2. Given ConfigPort is defined When I implement mocks for testing Then test doubles are available for isolated unit testing

3. Given the domain interface exists When I validate the design Then it follows hexagonal principles with clear separation between domain and infrastructure

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [ ] Write failing unit tests for ConfigPort trait definition (async load method, error types)
- [ ] Write failing unit tests for Config domain types (validation, construction, invariants)
- [ ] Write failing integration tests for ConfigPort mock implementations
- [ ] Write failing property-based tests for configuration loading edge cases
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Config Domain Interface (GREEN Phase - AC: 1,3)
- [ ] Implement ConfigPort trait with async_trait and comprehensive error handling
- [ ] Define configuration loading method signatures with proper type constraints
- [ ] Implement domain error types for configuration operations
- [ ] Create basic Config domain types if needed (minimal for port definition)
- [ ] **TDD REQUIREMENT:** Make all ConfigPort tests pass (GREEN phase complete when tests pass)

### Task 3: Create Configuration Mocks for Testing (GREEN Phase - AC: 2)
- [ ] Implement MockConfigPort using mockall crate for comprehensive testing
- [ ] Create test fixtures for configuration loading scenarios
- [ ] Implement mock behaviors for success and error cases
- [ ] Add mock verification capabilities for test assertions
- [ ] **TDD REQUIREMENT:** Make all mock-related tests pass

### Task 4: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Optimize async trait implementations for performance (<100μs overhead)
- [ ] Add comprehensive documentation with examples for port usage
- [ ] Ensure proper error chaining and context preservation
- [ ] Verify hexagonal architecture compliance (domain purity, no I/O)
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 5: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update domain crate documentation with ConfigPort usage examples
- [ ] Ensure integration points with future adapters (Figment, file loading)
- [ ] Add comprehensive doc comments following project standards
- [ ] Verify port traits derive required traits for testing
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 6: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure async trait implementations meet performance targets
- [ ] **TDD VALIDATION:** Confirm comprehensive error handling for configuration operations
- [ ] **TDD VALIDATION:** Verify mock implementations provide adequate test coverage
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all config domain components pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (domain port only)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement config domain interface and port with TDD validation`

## Dev Notes

### Developer Context
This story establishes the foundational domain interface for configuration management in lithos, creating the ConfigPort trait that defines how configuration will be loaded throughout the application. It's the first story in Epic 5 (Configuration Management System), providing the hexagonal architecture contract that subsequent stories will implement.

**Business Value:** Enables clean separation between configuration domain logic and infrastructure implementations, allowing flexible configuration loading strategies (Figment, custom parsers, etc.) while maintaining consistent domain contracts.

**Technical Context:** Must follow hexagonal architecture with async_trait for async port methods, comprehensive error handling with thiserror, and testability through mock implementations. This port will be used by all components that need configuration access.

**Dependencies:** Depends on Epic 4 (file loading foundation), enables all subsequent Epic 5 stories (hierarchical loading, validation, versioning).

**Risks:** Async trait design must balance ergonomics with performance. Error handling must be comprehensive for debugging configuration issues.

### Technical Requirements
**Core Implementation Requirements:**
- **Language**: Rust 1.92+ with async_trait for async port methods
- **Architecture**: Hexagonal domain port only (no I/O or infrastructure)
- **Async Patterns**: async_trait for non-blocking configuration operations
- **Error Handling**: thiserror for typed domain errors with context
- **Testing**: 90%+ coverage, mock implementations for isolated testing
- **Performance**: <100μs async trait dispatch overhead
- **Safety**: Zero unsafe code, proper error propagation
- **Documentation**: Comprehensive examples for port usage

**Port Design Requirements:**
- **ConfigPort Trait**: Async load method with generic configuration type
- **Error Types**: ConfigError enum covering loading, validation, and parsing failures
- **Type Safety**: Associated types for configuration structures
- **Mock Support**: Easy mocking for comprehensive testing

**Integration Requirements:**
- **Future Adapter Support**: Design for Figment, TOML, JSON, YAML implementations
- **Cross-Crate Usage**: Port used from app, adapters, and CLI crates
- **Configuration Types**: Generic enough for different config structures (global, user, project, vault)

### Architecture Compliance Requirements
- **Hexagonal Boundary**: Domain port only - NO I/O, NO external dependencies, NO infrastructure
- **CQRS**: Configuration is read-heavy, but port design supports both patterns
- **Event System**: No events generated (configuration is static)
- **Async Patterns**: async_trait for async methods, proper cancellation support
- **Error Hierarchy**: ConfigError → thiserror domain errors
- **Naming Conventions**: ConfigPort, ConfigError, snake_case methods
- **No Unsafe Code**: Pure Rust implementation
- **Clippy Compliance**: Cognitive complexity <25, all lint rules

### Library and Framework Requirements
- **async-trait 0.1+**: For async trait methods in domain ports
- **thiserror 2.0**: Structured error definitions for configuration operations
- **serde 1.0+**: Optional derive for configuration serialization (if domain types added)
- **mockall 0.12+**: For creating comprehensive mock implementations
- **tokio 1.49**: For async runtime support (workspace dependency)
Use versions from Cargo.toml workspace dependencies.

### File Structure Requirements
**Hexagonal Architecture Layout:**
```
crates/domain/src/
├── ports/
│   └── config.rs              # ConfigPort trait + ConfigError enum
├── errors.rs                  # Updated with ConfigError variants (if separate)
└── lib.rs                     # Public API re-exports

crates/domain/tests/
└── config_port_integration_test.rs  # Cross-crate integration tests
```

**File Organization Principles:**
- **Domain Ports**: All driven ports in ports/ directory with clear naming
- **Error Types**: Either in ports file or separate errors.rs based on size
- **Test Placement**: Integration tests in tests/ directory, unit tests inline
- **Modularity**: Keep ports focused on single responsibility
- **Documentation**: Extensive examples for adapter implementers

### Testing Requirements
- **Unit Tests**: Port trait definitions, error type validation, mock behavior
- **Integration Tests**: Cross-crate usage of ConfigPort implementations
- **Property-Based Tests**: Configuration loading edge cases and invariants
- **Performance Tests**: Async trait dispatch overhead measurement
- **Coverage Target**: 90%+ for domain port and error handling logic
- **Mock Testing**: Comprehensive mock implementations for all port methods
- **Async Testing**: Tokio test for async port operations

### Git Intelligence Summary
Recent commits show focus on testing infrastructure and CQRS patterns:
- Centralized test utilities with artifact management and isolation
- CQRS testing patterns with command/query separation
- Event flow integration tests
- Async testing with tokio::test
This story should follow established testing patterns: unit tests in domain, integration tests with mocks, async tests for I/O operations.

### Story Quality Improvements from Epic 3 Review
Reviewed Epic 3 story files to adopt proven TDD patterns:
- **Task Structure**: RED (Define Tests First) → GREEN (Implement) → REFACTOR → Testing Coverage → Documentation → Quality Assurance
- **Atomic Subtasks**: Each subtask represents single TDD cycle with clear acceptance criteria
- **Mandatory Quality Assurance Task**: Includes mise run verify, pre-commit hooks, coverage validation, and conventional commits
- **Comprehensive Documentation**: Invariants, examples, error conditions in all public APIs
- **Performance Validation**: Benchmarks and coverage targets with measurable criteria

### Anti-Pattern Prevention (Critical Mistakes to Avoid)
**🚨 COMMON LLM DEVELOPER DISASTERS PREVENTED:**

- **❌ Wrong Port Design**: ConfigPort must be async and generic enough for all config types
- **❌ Domain Pollution**: NO I/O, NO file loading, NO external crates beyond async-trait/thiserror
- **❌ Inconsistent Errors**: Unified ConfigError enum with comprehensive variants
- **❌ Missing Mock Support**: Easy mocking critical for hexagonal testing
- **❌ Performance Issues**: Async trait overhead must be minimal (<100μs)
- **❌ Poor Documentation**: Extensive examples needed for adapter implementers
- **❌ Type Safety Issues**: Strong typing for configuration structures
- **❌ Blocking Operations**: All operations must be async-ready

**✅ CORRECT PATTERNS TO FOLLOW:**
- Hexagonal architecture: domain ports, adapter implementations
- TDD cycle: RED (failing tests) → GREEN (minimal implementation) → REFACTOR (quality)
- Async safety: async_trait for all port methods, proper cancellation
- Error chaining: ConfigError → domain error hierarchy
- Mock-first design: Comprehensive mocking from day one
- Performance monitoring: Criterion benchmarks for trait dispatch
- Documentation-driven: Examples for every public method

### Latest Tech Information
**Library Version Rationale (2026 Ecosystem):**
- **async-trait 0.1.80**: Latest stable with zero-cost async trait implementation
- **thiserror 2.0.3**: Standard error handling with excellent ergonomics
- **mockall 0.12.1**: Comprehensive mocking with async trait support
- **serde 1.0.133**: Optional for configuration serialization (domain may not need)

**Performance Benchmarks (Reference Data):**
- async-trait dispatch: <50μs overhead for simple methods
- Mock creation: <10μs for basic mock setup
- Error allocation: <5μs for typical domain errors
- Combined port dispatch + error: <100μs target

**Security Considerations:**
- No external I/O in domain (security handled in adapters)
- Type safety prevents malformed configuration structures
- Error messages sanitized for debugging (no sensitive data exposure)

**Migration Considerations:**
- async-trait stable and widely adopted
- thiserror standard in Rust ecosystem
- Compatible with existing tokio and serde versions

### Project Structure Notes
- Alignment with unified project structure (ports/ directory, hexagonal layout)
- ConfigPort follows established port naming (Verb + Port)
- Integration with Epic 4 file loading for configuration files
- Detected conflicts or variances (with rationale): None - follows architecture.md patterns

### References
- Epic 5 details: _bmad-output/planning-artifacts/epics/epic-5-configuration-management-system-phase-15.md
- Architecture patterns: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns
- Project context: _bmad-output/project-context.md
- Testing standards: _bmad-output/project-context.md#Testing Rules
- Port Design Examples: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns
- Quality Assurance Pattern: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md#Task-7-Quality-Assurance-and-Commit
- Validation Report: _bmad-output/implementation-artifacts/reports/validation-report-2026-01-12-story-5-1-create-config-domain-interface-and-port.md

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References


### Completion Notes List


### File List

Expected files to be created:
- crates/domain/src/ports/config.rs (ConfigPort trait, ConfigError enum)
- crates/domain/src/errors.rs (updated with ConfigError variants if separate)
- crates/domain/tests/config_port_integration_test.rs (integration tests)
- crates/domain/src/lib.rs (updated with public re-exports)
