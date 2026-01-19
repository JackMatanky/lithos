# Story 3.6: Review Epic 3 Test Suite

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer maintaining the codebase,
I want an efficient test suite for Epic 3 domain models,
So that tests provide good coverage without redundancy or excessive execution time.

## Acceptance Criteria

1. **Given** all Epic 3 domain models are implemented with tests
   **When** I review the test suite
   **Then** it achieves 90%+ coverage for domain entities and validation logic

2. **Given** the test suite is implemented
   **When** I check for redundancy
   **Then** no duplicate test cases exist across domain models

3. **Given** tests are executed
   **When** I measure performance
   **Then** test execution completes in <30 seconds for the full Epic 3 suite

4. **Given** test suite is reviewed
   **When** I check maintainability
   **Then** test code follows same quality standards as production code

5. **Given** domain models evolve
   **When** I update tests
   **Then** test maintenance cost is <20% of development time

## Tasks / Subtasks

### Task 1: Establish Epic 2 Test Infrastructure Context
- [x] Read the full text of `docs/test_guide.md` and `_bmad-output/test-design-system.md` to internalize the project's testing standards, safety invariants, and quality gates
- [x] Read the full text of Story 2.4 from `_bmad-output/implementation-artifacts/stories/2-4-create-centralized-test-utilities-and-infrastructure.md` to understand available test utilities like temporary directories, fixture factories, assertion helpers
- [x] Read the full text of Story 2.5 from `_bmad-output/implementation-artifacts/stories/2-5-configure-mise-test-task-orchestration.md` to understand mise commands: `mise run test` (all tests), `mise run test:unit` (domain only), `mise run test:integration` (cross-crate), `mise run test:coverage` (tarpaulin), `mise run test:watch` (continuous)
- [x] Read the full text of Story 2.6 from `_bmad-output/implementation-artifacts/stories/2-6-establish-integration-testing-patterns-and-infrastructure.md` to understand integration testing patterns for cross-module testing, isolation strategies, external service mocking
- [x] Read the full text of Story 2.7 from `_bmad-output/implementation-artifacts/stories/2-7-create-benchmarking-infrastructure-and-performance-testing-patterns.md` to understand criterion.rs integration, performance regression detection, benchmark result storage
- [x] Create document `_bmad-output/test-utilities-reference.md` listing all available Epic 2 utilities with usage examples for Epic 3 testing, including code snippets showing how to use each utility
- [x] Run `mise run test:unit` and verify it executes successfully, confirming Epic 2 test utilities are available and functional

### Task 2: Analyze Current Epic 3 Test Implementation
- [x] Run `mise run test:coverage` to generate detailed tarpaulin HTML report in `_bmad-output/coverage-reports/` directory
- [x] Run `mise run test` with `time mise run test` to measure total execution time and record individual test category times (unit, integration, property tests)
- [x] **DOC-TEST ANALYSIS:** Audit existing public API documentation for missing or broken doc-tests, ensuring all public domain models have at least one executable example
- [x] Open the generated HTML coverage report and analyze coverage percentage for each bounded context: Note domain entities, Schema domain entities, Config domain entities, Template domain entities
- [x] Perform hexagonal compliance check for domain tests: verify `crates/domain/src/models/*/tests.rs` modules have ZERO external dependencies (no tokio, no adapters, no app layer imports) and use `#[cfg(test)]` attribute
- [x] Perform hexagonal compliance check for adapter/integration tests: verify adapter layer tests are in `tests/` directory or `adapters/*/tests/` modules and properly mock external dependencies
- [x] Review test file organization: check that domain entities have inline `#[cfg(test)] mod tests` modules and integration tests are in `crates/domain/tests/` directory
- [x] Identify specific coverage gaps in critical areas: list uncovered lines in validation logic, error paths, edge cases, boundary conditions for each bounded context
- [x] Assess coverage quality vs metrics: review test code to identify vanity coverage (tests that only exercise code without meaningful assertions) vs meaningful tests
- [x] Document coverage strategy: 80%+ target with quality focus (business logic, error cases, edge conditions over line coverage)
- [x] Document current coverage gaps, weak areas, and quality concerns

### Task 3: Identify Redundancies, Inefficiencies, and Over-Complexity
- [x] Review test fixtures for duplication across bounded contexts
- [x] Analyze property-based test patterns for consolidation opportunities
- [x] **COMPLEXITY AUDIT:** Identify "clever" or overly complex test logic that hinders readability. Ensure tests follow the KISS principle.
- [x] **RSTEST EVALUATION:** Audit usage of `rstest`. Verify that parameterized cases provide a clear benefit (e.g., testing same logic with multiple critical inputs) rather than adding unnecessary indirection for simple cases.
- [x] Check for overlapping test scenarios between domain models
- [x] Identify slow-running tests that impact the <30 second target
- [x] Document redundancy and complexity elimination opportunities

### Task 4: Optimize Test Performance and Coverage
- [x] Implement shared test utilities leveraging Epic 2 infrastructure
- [x] Consolidate duplicate fixtures into reusable modules
- [x] **DOC-TEST OPTIMIZATION:** Apply "Living Documentation" patterns to doc-tests:
    - [x] Hide setup/boilerplate imports and logic using the `#` prefix
    - [ ] Ensure examples are high-fidelity and demonstrate real-world usage of `lithos-test-utils`
    - [x] Use appropriate attributes (`no_run`, `compile_fail`, `should_panic`) to accurately reflect intended behavior
- [ ] Optimize slow tests using parallel execution and Epic 2 patterns
- [x] **COVERAGE ASSURANCE:** Add targeted tests for uncovered domain entities and validation logic
- [x] **COVERAGE ASSURANCE:** Implement property-based tests for edge cases and error paths
- [ ] **COVERAGE ASSURANCE:** Add integration tests for cross-entity validation scenarios
- [ ] **COVERAGE ASSURANCE:** Ensure coverage quality (meaningful assertions, not just line coverage)
- [ ] Configure nextest for optimal Epic 3 test execution
- [ ] Verify 80%+ coverage target achieved with `mise run test:coverage` (focus on business logic quality)

### Task 5: Establish Test Maintenance Guidelines
- [ ] Create test evolution tracking for maintenance cost monitoring (<20% target)
- [ ] Document test update patterns for future domain model changes
- [ ] Establish test quality standards aligned with production code
- [ ] Implement automated test metrics collection (coverage, execution time)
- [ ] Create maintenance cost monitoring and alerting

### Task 6: Validate Optimized Test Suite
- [ ] **COVERAGE VALIDATION:** Confirm 80%+ coverage achieved for domain entities and validation logic (prioritize quality)
- [ ] **COVERAGE VALIDATION:** Verify coverage quality - tests exercise meaningful behavior, not just lines
- [ ] **COVERAGE VALIDATION:** Ensure branch coverage for critical conditional logic
- [ ] **COVERAGE VALIDATION:** Validate edge case and error path coverage
- [ ] Verify <30 second execution time for complete Epic 3 test suite
- [ ] Ensure zero duplicate test cases across bounded contexts
- [ ] Validate test maintainability standards are met
- [ ] Document test suite efficiency improvements, coverage gains, and ROI

### Task 7: Enforce Epic 2 Quality Standards
- [ ] **BDD DOCUMENTATION:** Ensure all tests include internal expressive BDD comments (e.g., `// GIVEN: a valid note with multiple links`, `// WHEN: the link resolution service is called`, `// THEN: all links are resolved to absolute vault paths`). The GIVEN-WHEN-THEN words must be followed by descriptive text explaining the context, action, and expected outcome.
- [ ] **STRICT NAMING:** Verify 100% compliance with verb-first behavioral naming across all Epic 3 tests using the formula: `unit_of_work` + `expected_behavior` + `state_under_test`
- [ ] **PARAMETERIZED TESTS:** Ensure `rstest` is used ONLY when it provides a real benefit to clarity and maintainability. Avoid using it for single scenarios or when it makes the test logic harder to follow. Always use **Named Cases**.
- [ ] **KISS COMPLIANCE:** Verify that tests are readable and maintainable. A test should be easier to understand than the code it tests. Eliminate any "test logic" that is as complex as the production logic.
- [ ] **SNAPSHOT TESTING:** Verify that complex structures (Note AST, Schema inheritance graphs) use `insta` snapshots with proper **Redactions** for UUIDs and Timestamps
- [ ] **ASYNC SAFETY:** Confirm all async tests use `#[tokio::test(flavor = "multi_thread")]` and incorporate timeouts and `spawn_blocking_test` for I/O or heavy CPU tasks
- [ ] **LINT DISCIPLINE:** Enforce `#[expect(...)]` over `#[allow(...)]` for intentional violations and verify every test module includes a `LINT_DISABLE_REASON` header
- [ ] **DOC-TESTS:** Verify mandatory doc-test coverage for ALL public domain models and utility functions as "Living Documentation", ensuring boilerplate is hidden and attributes are correctly applied
- [ ] **ERROR ASSERTIONS:** Ensure all error cases use the `assert_err_kind!` macro for standardized and readable error matching
- [ ] **OBSERVABILITY:** Verify use of `TestTracingSubscriber` where domain events or tracing spans need validation
- [ ] **TEST PLACEMENT:** Ensure no test code exists outside `tests/utils` and `tests/macros` (except for inline unit tests)
- [ ] **VIRTUAL TIME:** Confirm all time-sensitive domain logic uses the `time_test!` virtual clock infrastructure
- [ ] **PURITY GUARDIAN:** Run the Domain Purity Guardian and confirm 100% compliance for all Epic 3 domain models

### Task 8: Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] **HEXAGONAL VALIDATION:** Confirm test suite properly mirrors hexagonal architecture (domain pure, adapters integrated)
- [ ] **COVERAGE VALIDATION:** Confirm 80%+ coverage achieved and documented in coverage report (business logic focus)
- [ ] **COVERAGE VALIDATION:** Verify coverage quality - tests exercise meaningful domain behavior
- [ ] **COVERAGE VALIDATION:** Ensure critical validation logic and error paths are covered
- [ ] **VALIDATION:** Confirm Epic 3 test suite analysis is comprehensive and actionable
- [ ] **VALIDATION:** Verify Epic 2 test infrastructure integration is properly documented
- [ ] **VALIDATION:** Ensure optimization recommendations leverage available test utilities
- [ ] **VALIDATION:** Confirm maintenance cost tracking mechanisms are implemented
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all analysis artifacts pass quality gates and coverage targets are met
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `refactor: optimize epic 3 test suite for efficiency with comprehensive analysis and actionable recommendations`

## Dev Notes

### Epic 3 Test Context
- **Test Targets**: Note, Schema, Config, Template bounded contexts
- **Coverage Goal**: 90%+ for domain entities and validation logic (architecture baseline: 80%+)
- **Performance Target**: <30 seconds for full Epic 3 test suite execution
- **Quality Standards**: Same clippy, fmt, cognitive complexity rules as production code
- **Maintenance Target**: <20% of development time for test updates

### Previous Story Intelligence
**Story 3-4 Critical Architectural Fix:**
- **MAJOR CORRECTION**: MiniJinja syntax validation moved from domain to adapter layer
- **Impact**: Domain layer must remain pure with ZERO external dependencies
- **Test Implication**: Domain tests must validate business rules only, no syntax validation
- **Lesson**: Hexagonal boundary violations must be caught and corrected immediately
- **Testing Pattern**: Separate domain business rule tests from adapter integration tests

### Epic 2 Test Infrastructure Context (Available Utilities)
**Story 2.4: Centralized Test Utilities**
- Temporary directory creation and cleanup utilities
- Test artifact output management helpers
- Test data fixtures and factory patterns
- Common assertion helpers for domain testing

**Story 2.5: Mise Test Orchestration**
- `mise run test` - All tests with optimal parallelization
- `mise run test:unit` - Domain layer unit tests only
- `mise run test:integration` - Cross-crate integration tests
- `mise run test:coverage` - Tarpaulin coverage reports
- `mise run test:watch` - Watch mode for continuous testing

**Story 2.6: Integration Testing Patterns**
- Cross-module API contract testing infrastructure
- Database/transaction isolation for integration tests
- External service mocking utilities
- Integration test data fixtures and setup patterns

**Story 2.7: Benchmarking Infrastructure**
- criterion.rs integration for micro-benchmarks
- Performance regression detection
- Benchmark result storage and comparison
- CI/CD integration for performance gates

### Previous Story Intelligence
**Story 3-4 Architectural Corrections:**
- CRITICAL FIX: Moved MiniJinja syntax validation from domain to adapter layer
- Impact: Template domain now has ZERO external dependencies (justified exception: serde for persistence)
- Lesson: Hexagonal architecture violations must be caught and fixed immediately
- Pattern: Domain entities store content as opaque strings, validation happens in adapters

**Story 3-3 Implementation Patterns:**
- Config domain with hierarchical validation (Global → User → Project → Vault)
- Semantic validation integrated with domain business rules
- Error handling using thiserror::Error with proper error chaining

**Story 3-2 Schema Complexity:**
- PropertyBank singleton registry pattern
- PropertySpec trait with multiple variants (String, Number, Bool, Date, File)
- Inheritance resolution with Extends/Excludes patterns
- Deterministic ID generation from content hash

**Story 3-1 Note Foundations:**
- Rich aggregate with subentities (Frontmatter, Links, Embeds, Tags, Headings, Tasks, Sections)
- Wiki-link resolution and vault-relative paths
- Semantic validation for internal consistency

### Git Intelligence Summary
**Recent Commit Patterns:**
- Architectural violation fixes (domain purity enforcement)
- Comprehensive story file creation with TDD frameworks
- Quality assurance task additions
- Epic completion and preparation workflows

**Code Patterns Established:**
- Hexagonal boundary enforcement (domain zero dependencies except justified cases)
- TDD Red-Green-Refactor implementation cycles
- Quality gates with mandatory pre-commit hook compliance
- Conventional commit messages with proper scope and type

### Latest Tech Information
**Rust Testing Ecosystem (2026):**
- **nextest**: Primary test runner for parallel execution and optimized performance
- **tarpaulin**: Code coverage analysis with 80%+ target enforcement
- **criterion**: Micro-benchmarking for performance regression detection
- **proptest**: Property-based testing for edge case discovery
- **mockall**: Mock generation for adapter layer testing

**Performance Testing Standards:**
- <100μs for individual domain operations
- <500ms for template operations (Story 3-4)
- <2 seconds for vault indexing (NFR2)
- Statistical analysis with criterion for reliable benchmarks

## Technical Requirements

### Domain Model Test Coverage Requirements

**Coverage Targets by Bounded Context:**

**Note Bounded Context (Story 3-1):**
- Note aggregate: 80%+ coverage (entity creation, validation, relationships)
- Subentities: Frontmatter, Links, Embeds, Tags, Headings, Tasks, Sections (all 80%+)
- Business rules: Wiki-link resolution, vault path validation, semantic consistency
- Error cases: Invalid relationships, malformed content, constraint violations

**Schema Bounded Context (Story 3-2):**
- Schema entity: 80%+ coverage (creation, inheritance, validation)
- PropertyBank: 80%+ coverage (registry operations, lookup methods)
- Property entity: 80%+ coverage (ID generation, constraint validation)
- PropertySpec variants: 80%+ coverage (String, Number, Bool, Date, File specs)
- Inheritance resolution: 80%+ coverage (Extends, Excludes, property merging)

**Config Bounded Context (Story 3-3):**
- Config entity: 80%+ coverage (hierarchical structure, validation)
- Configuration layers: 80%+ coverage (Global, User, Project, Vault merging)
- Business rules: 80%+ coverage (configuration integrity, type safety)
- Error handling: 80%+ coverage (validation failures, merge conflicts)

**Template Bounded Context (Story 3-4):**
- Template entity: 80%+ coverage (structure validation, composition)
- VariableDefinition enum: 80%+ coverage (type safety, constraints, defaults)
- TemplateComposition: 80%+ coverage (modular assembly, dependency resolution)
- Business rules: 80%+ coverage (variable naming, composition cycles, semantic validation)

### Test Efficiency Standards

**Performance Requirements:**
- **Execution Time**: <30 seconds for complete Epic 3 test suite
- **Individual Tests**: <100ms per test (parallel execution encouraged)
- **Setup Time**: <5 seconds for test fixtures and data preparation
- **Coverage Analysis**: <10 seconds for tarpaulin report generation

**Redundancy Elimination:**
- **Shared Fixtures**: Common test data in `domain::test_utils` module
- **Test Factories**: Reusable entity creation patterns
- **Property Test Sharing**: Common property test utilities across bounded contexts
- **Validation Test Patterns**: Standardized validation error testing

**Maintenance Cost Control:**
- **Test-to-Code Ratio**: Maintain <3:1 test lines per production line
- **Update Frequency**: <20% of development time spent on test maintenance
- **Change Detection**: Automated detection of test maintenance cost increases
- **Evolution Tracking**: Metrics collection for test suite health monitoring

### Architecture Compliance - MANDATORY READING

**Hexagonal Testing Architecture:**
- **Domain Layer Tests**: Pure unit tests with ZERO external dependencies
- **Adapter Layer Tests**: Integration tests with real implementations and mocks
- **Cross-Crate Tests**: Integration tests spanning domain/app/adapters boundaries
- **E2E Tests**: CLI-driven workflow tests (separate from domain testing)

**Test Organization Standards:**
```rust
// Domain crate structure
crates/domain/
├── src/
│   ├── entities/          // Domain entities
│   ├── value_objects/     // Value objects
│   ├── services/          // Domain services
│   └── lib.rs
├── tests/                 // Integration tests
│   └── integration_tests.rs
└── benches/              // Performance benchmarks
    └── domain_benchmarks.rs

// Test placement rules
#[cfg(test)]              // Unit tests in same file as implementation
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() { /* ... */ }
}

// Integration tests in tests/ directory
#[cfg(test)]
mod integration_tests {
    use domain::*;

    #[tokio::test]
    async fn test_cross_entity_validation() { /* ... */ }
}
```

**Test Quality Standards:**
- **Naming**: `snake_case` with verb-first behavioral naming formula: `unit_of_work` + `expected_behavior` + `state_under_test`
- **Documentation**: Comprehensive doc comments explaining test purpose and scenarios; mandatory internal expressive BDD comments (e.g., `// GIVEN: [context]`, `// WHEN: [action]`, `// THEN: [expectation]`) for all test bodies to ensure the test logic is self-explanatory.
- **Isolation**: Each test independent, no shared state between tests
- **Deterministic**: Fixed seeds for property-based tests, no flaky behavior
- **Performance**: Sub-100ms execution time per test

**Coverage Analysis Tools:**
- **tarpaulin**: Primary coverage tool with branch coverage analysis
- **Minimum Thresholds**: 80%+ line coverage, 75%+ branch coverage for Epic 3 (quality over quantity)
- **Exclusion Rules**: Generated code, test utilities excluded from coverage
- **Reporting**: HTML reports for detailed analysis, CI integration for enforcement

### Library Framework Requirements

**Core Testing Dependencies:**
```toml
# Cargo.toml test dependencies
[dev-dependencies]
tokio = { version = "1.49", features = ["full", "test-util"] }
nextest = "0.9"           # Primary test runner
tarpaulin = "0.27"        # Code coverage analysis
criterion = "0.5"         # Performance benchmarking
proptest = "1.5"          # Property-based testing
mockall = "0.12"          # Mock generation for adapters
```

**Test Framework Configuration:**
```toml
# nextest.toml
[profile.default]
retries = 1
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.ci]
retries = 3
slow-timeout = { period = "120s", terminate-after = 3 }
```

### File Structure Requirements

**Test File Organization:**
```
crates/
├── domain/
│   ├── src/
│   │   ├── note.rs           # #[cfg(test)] mod tests {}
│   │   ├── schema.rs         # #[cfg(test)] mod tests {}
│   │   ├── config.rs         # #[cfg(test)] mod tests {}
│   │   ├── template.rs       # #[cfg(test)] mod tests {}
│   │   └── lib.rs
│   ├── tests/
│   │   ├── integration_tests.rs    # Cross-entity integration
│   │   └── property_tests.rs       # Property-based tests
│   └── benches/
│       └── domain_benchmarks.rs    # Performance tests
```

**Test Utility Modules:**
- `test_utils.rs`: Shared fixtures, factories, and helper functions
- `fixtures/`: Predefined test data files
- `property_tests.rs`: Common property test utilities
- `performance_tests.rs`: Benchmark configuration and helpers

### Testing Infrastructure Integration

**Hexagonal Testing Architecture (MANDATORY - Architecture Requirement):**
- **Domain Layer Tests**: Pure unit tests with ZERO external dependencies (`#[cfg(test)]` modules)
- **Adapter Layer Tests**: Integration tests with real implementations and mocking
- **Cross-Crate Tests**: Integration tests spanning domain/app/adapters boundaries
- **E2E Tests**: CLI-driven workflow tests (separate from domain testing)
- **CQRS Test Separation**: Prepare for future write model vs read model test suites

**Leveraging Epic 2 Test Utilities (Stories 2.4-2.7):**

**Centralized Test Utilities (Story 2.4):**
- **Temporary Directory Management**: Cross-platform temp dir creation/cleanup for isolated testing
- **Test Artifact Helpers**: Standardized output location management for test artifacts
- **Fixture Factories**: Reusable domain object creation with valid defaults and edge cases
- **Assertion Helpers**: Common test assertions for domain validation patterns

**Mise Test Orchestration (Story 2.5):**
- **Parallel Execution**: `nextest` integration for optimal test parallelization
- **Coverage Analysis**: `tarpaulin` integration for detailed coverage reporting (80%+ baseline, 90%+ target)
- **Watch Mode**: Continuous testing during development workflow
- **Test Categorization**: Separate unit, integration, and performance test execution

**Integration Testing Patterns (Story 2.6):**
- **Cross-Module Testing**: Established patterns for testing bounded context interactions
- **Transaction Isolation**: Proper database/transaction management for integration tests
- **Mock Infrastructure**: External service mocking utilities for isolated testing
- **Contract Testing**: API contract verification between modules

**Benchmarking Infrastructure (Story 2.7):**
- **Performance Baselines**: criterion.rs benchmarks for establishing performance expectations
- **Regression Detection**: Automated performance regression alerts
- **Statistical Analysis**: Confidence intervals and statistical significance testing
- **CI/CD Integration**: Performance gates in automated pipelines

### Test Coverage Assurance Framework

**Coverage Measurement and Analysis:**

**Tools Integration (Epic 2.5):**
- **Primary Tool**: `tarpaulin` via `mise run test:coverage` for detailed HTML reports
- **Baseline Reports**: Generate coverage reports before and after optimizations
- **File-by-File Analysis**: Identify specific modules and functions needing coverage
- **Branch Coverage**: Ensure conditional logic and error paths are tested
- **Exclusion Rules**: Properly exclude generated code, test utilities, and boilerplate
- **Coverage Trends**: Track coverage improvements over time with baseline comparisons

**Advanced Testing Patterns:**
- **Property-Based Testing**: Leverage `proptest` with custom strategies for const generic types
- **Phantom Type Testing**: Ensure compile-time safety is validated through type-level tests
- **Associated Type Testing**: Test repository port implementations with concrete associated types

**Coverage Quality vs Quantity:**
- **Meaningful Coverage**: Focus on testing business logic, validation rules, and error conditions
- **Avoid Vanity Metrics**: Don't write tests just to increase percentages - quality over quantity
- **Risk-Based Testing**: Prioritize coverage for critical domain operations and edge cases
- **Hexagonal Coverage Structure**: Domain (80%+), Adapters (integration), E2E (critical paths)
- **Quality Gates**: 80%+ line coverage, 75%+ branch coverage for Epic 3 domain entities (quality-focused)
- **Coverage Assurance**: Regular reviews to ensure tests validate behavior, not just execution paths

**Coverage Assurance Techniques:**

**Domain Entity Coverage:**
- **Constructor Validation**: Test all entity creation paths and validation rules
- **Business Rule Testing**: Cover all domain invariants and constraints
- **Error Path Coverage**: Test validation failures and error conditions
- **Property-Based Testing**: Use proptest for mathematical properties and edge cases

**Validation Logic Coverage:**
- **Semantic Validation**: Test domain business rules and consistency checks
- **Type Safety**: Verify enum variants, option handling, and type constraints
- **Boundary Conditions**: Test limits, ranges, and constraint enforcement
- **Integration Points**: Test interactions between related domain entities

**Test Coverage Implementation Strategy:**

**Gap Analysis Process:**
1. **Run Coverage Report**: `mise run test:coverage` to identify uncovered lines
2. **Categorize Gaps**: Business logic vs boilerplate vs error handling
3. **Priority Assessment**: Critical validation logic > edge cases > happy paths
4. **Test Planning**: Design specific tests to close identified gaps

**Targeted Test Addition:**
- **Unit Tests**: Pure domain logic with no external dependencies
- **Property Tests**: Edge cases and mathematical properties using proptest
- **Integration Tests**: Cross-entity validation scenarios
- **Error Path Tests**: Exception conditions and failure modes

**Coverage Quality Assurance:**
- **Test Intent Clarity**: Each test should have clear purpose and assertions
- **Assertion Quality**: Verify meaningful behavior, not just execution
- **Test Isolation**: Independent tests with proper setup/teardown
- **Maintenance Cost**: Tests should be as maintainable as production code

**Performance Optimization Strategy:**
- **Baseline Measurement**: Establish current execution time using `mise run test`
- **Bottleneck Analysis**: Identify slow-running tests and optimization opportunities
- **Parallel Execution**: Leverage nextest for optimal test parallelization
- **Caching Strategies**: Implement fixture reuse and test result caching where appropriate

**Redundancy Elimination Process:**
- **Pattern Recognition**: Automated detection of duplicate test scenarios
- **Consolidation Planning**: Create shared utilities for common test patterns
- **Refactoring Roadmap**: Systematic elimination of redundant test code
- **Maintenance Tracking**: Monitor impact on test maintenance costs

**Maintenance Cost Management:**
- **Evolution Tracking**: Automated monitoring of test-to-code ratios
- **Change Impact Analysis**: Predict maintenance cost for domain model changes
- **Optimization Metrics**: Track time spent on test maintenance vs feature development
- **Continuous Improvement**: Regular review and optimization of test maintenance processes

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test utilities for test suite analysis, mock repositories, and performance benchmarking infrastructure

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

- 2026-01-19: Read Epic 2 test standards (docs/test_guide.md, _bmad-output/test-design-system.md) and Epic 2 story artifacts (2.4-2.7).
- 2026-01-19: Generated Epic 2 test utilities reference doc with usage examples.
- 2026-01-19: Ran `mise run test:unit` to validate test utilities and doc-tests.
- 2026-01-19: Ran `time mise run test` for full test timing; unit, integration, and E2E suites passed.
- 2026-01-19: Ran `cargo test -p lithos-domain --doc -- --list` to audit doc-tests in domain models.
- 2026-01-19: Reviewed domain test modules for hexagonal compliance (no external deps; inline `#[cfg(test)]`), integration tests in tests/suite.
- 2026-01-19: Ran `mise run test:coverage --package domain --skip-e2e` to generate tarpaulin HTML report and per-file coverage stats.
- 2026-01-19: Audited fixture duplication, property-based test usage, `rstest` usage, overlapping scenarios, and slow-test contributors for Task 3.
- 2026-01-19: Added shared proptest identifier strategies to test utils and reused in domain/template tests.
- 2026-01-19: Ran `mise run test:unit --package domain` after proptest strategy updates.
- 2026-01-19: Consolidated note fixtures in-domain and added integration fixture module under tests/suite/common.
- 2026-01-19: Optimized doc-tests with hidden boilerplate and updated resolver example; ran domain doctests.

### Completion Notes List

- Created Epic 2 test utilities reference at `_bmad-output/test-utilities-reference.md`, covering core async helpers, fixtures, assertions, CQRS/event tools, integration fixtures, benchmarks, observability, and mocks.
- Confirmed `mise run test:unit` passes (193 unit tests, doc tests across domain and test-utils).
- Full test run timing: total ~42.6s; unit 193 tests (~0.59s), integration 217 tests (~23.6s), E2E 2 tests (~0.44s).
- Doc-test audit: lithos-domain lists 53 doc-tests (all current); public domain models show executable examples, with ports using `ignore` for trait API snippets.
- Domain tests are inline under `#[cfg(test)]` with no adapter/app imports; integration and E2E tests live under tests/suite.
- Tarpaulin coverage (domain-only, skip E2E): 51.13% overall (676/1322). HTML report at `target/tarpaulin/tarpaulin-report.html`.
- Coverage gaps are concentrated in note frontmatter/link aggregate paths, schema resolver/property_spec, template variable validation, and template aggregate composition helper paths. Coverage quality is strong for validation modules and core invariants but thin for complex branches and error paths.
- Coverage strategy documented: prioritize business logic, error paths, and edge conditions over raw line metrics; use property tests and targeted unit coverage for complex branches.
- Fixture duplication exists between note fixtures (`crates/domain/src/note/aggregate.rs`) and schema/property fixtures (`crates/domain/src/schema/aggregate.rs`, `crates/domain/src/schema/property.rs`). Consolidate into shared `tests/utils` factories or a domain-level fixture module.
- Property-based tests appear in schema graph, config aggregate, template aggregate, and property name validation; several overlapping valid-name strategies could share a common proptest strategy helper.
- Complexity hotspots: schema graph proptest logic uses nested collections, manual sorting, and index math; could be replaced with helper builder or shared `SchemaGraphFixture` to reduce cognitive load. Config aggregate proptest uses deep nested object setup that could leverage shared fixture builders. These are readable but at risk for KISS violations if expanded.
- `rstest` usage is limited to config aggregate validation and error message table tests; both use named cases and are justified. No unnecessary parameterization found.
- Overlapping scenarios: PropertyName validation and SchemaName validation follow similar constraints; property name tests and schema name tests could share a generic name validation harness to reduce redundancy.
- Slow-test risk: Full suite at ~42.6s misses 30s target; integration tests (~23.6s) are the dominant contributor. Domain-only tests are fast. Focus optimization on integration suites and E2E concurrency to meet performance target.
- Redundancy elimination opportunities: unify fixture builders, centralize common validation assertions (e.g., name-format error cases) and consider moving repeated setup into `lithos_test_utils` helpers.
- Added shared proptest strategies in `tests/utils/src/data/properties.rs` for valid/invalid identifiers and reused them in schema/property and template tests.
- `mise run test:unit --package domain` passes (105 unit tests, 49 doc tests, 4 ignored).
- Consolidated note fixtures to stay with domain unit tests and added integration fixture location under `tests/suite/common/mod.rs`.
- Cleaned doc-test examples by hiding boilerplate imports and aligning schema resolver example with actual API.
- `cargo test -p lithos-domain --doc` passes (49 doctests, 4 ignored).

### File List

- _bmad-output/test-utilities-reference.md
- target/tarpaulin/tarpaulin-report.html
