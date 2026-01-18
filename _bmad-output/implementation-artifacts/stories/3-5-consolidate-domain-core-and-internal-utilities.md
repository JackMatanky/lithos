# Story 3.5: Consolidate Domain Core and Internal Utilities

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer maintaining the domain layer,
I want to consolidate shared logic and internal utilities into a central `lib.rs` and core modules within the domain crate,
So that the codebase remains DRY, maintainable, and architectural boundaries are respected.

## Acceptance Criteria

1. **Given** the domain crate contains existing behavior
   **When** I begin any refactoring work
   **Then** comprehensive unit tests exist for ALL domain logic to be refactored, ensuring behavior can be validated before and after changes

2. **Given** redundant utility functions and shared logic exist across Note, Schema, Config, and Template bounded contexts
   **When** I refactor the domain crate
   **Then** shared logic is moved to `crates/domain/src/lib.rs` or internal core modules AND all tests pass without modification

3. **Given** internal domain utilities are exposed publicly
   **When** I review visibility
   **Then** internal utilities use `pub(crate)` to prevent leaking into the application layer

4. **Given** the domain crate is the "inviolate core"
   **When** I consolidate utilities
   **Then** ZERO external dependencies (except justified ones like `serde` or `thiserror`) are introduced

5. **Given** common patterns like UUID v7 handling or shared error mapping are used
   **When** I implement core utilities
   **Then** they are implemented once in the core and reused across all bounded contexts

6. **Given** the models directory is already organized into bounded context subfolders (`note/`, `schema/`, `template/`)
   **When** I complete the refactor
   **Then** shared constants (like `patterns.rs`) are consolidated into `lib.rs` or dedicated validation modules, solitary models like `config.rs` remain at the root, and domain events remain centralized

7. **Given** inconsistent validation patterns across models
   **When** I consolidate utilities
   **Then** validation methods follow a standardized pattern and share common logic via the domain core

## Tasks / Subtasks

### Task 0: Pre-Refactoring Test Coverage Validation (MANDATORY FIRST TASK)
- [ ] **CRITICAL:** Run `mise run test:unit:domain` to establish baseline test coverage
- [ ] **Test Gap Analysis:** Identify ANY domain logic that lacks unit test coverage
- [ ] **Fill Test Gaps:** Write comprehensive unit tests for ALL untested domain behavior before proceeding with refactoring
- [ ] **Behavioral Documentation:** Ensure each test clearly documents the expected behavior it validates
- [ ] **Validation Checkpoint:** Run `mise run test:unit` and confirm 100% of tests pass
- [ ] **Coverage Report:** Generate coverage report with `mise run test:coverage` and document current coverage percentage
- [ ] **BLOCKING REQUIREMENT:** DO NOT PROCEED to Task 1 until all domain logic to be refactored has accompanying tests

### Task 1: Audit Domain Crate for Deep Behavioral Redundancy
- [ ] **Behavioral Audit:** Review ALL domain models and subentities across existing bounded contexts (`note/`, `schema/`, `template/`, and `config.rs`) to identify redundant logic and patterns.
- [ ] **CRITICAL:** Do not just look for identical function or struct names. Analyze the **behavior** and **intent** of logic (e.g., how paths are manipulated, how strings are sanitized, how collections are validated, how UUIDs are generated/handled) to find logically equivalent code that should be consolidated.
- [ ] **Audit Validation Patterns:** Deeply analyze all `validate()` methods and internal validation helpers across all contexts. Look for identical business rules being enforced via different implementation styles and consolidate them into the core.
- [ ] **Cross-Context Patterns:** Review `note/core.rs`, `schema/core.rs`, `template/core.rs`, `schema/validation.rs`, and `config.rs` for shared logic that can be abstracted into generic traits or shared utilities in `lib.rs`.
- [ ] **Constants Consolidation:** Identify if `schema/patterns.rs` contains constants that should be available domain-wide (not just in the schema context).
- [ ] Review error mapping patterns and identify opportunities for a unified domain error context strategy.
- [ ] Document all identified behavioral redundancies and validation inconsistencies in `_bmad-output/domain-redundancy-audit.md`

### Task 2: Consolidate into lib.rs and Core Modules
- [ ] **NOTE:** Domain models are already organized into bounded context subfolders (`note/`, `schema/`, `template/`), and `config.rs` remains at the root. This structure is complete.
- [ ] Create or update `crates/domain/src/lib.rs` to house shared traits, macros, constants (currently in `schema/patterns.rs`), and common identity logic (UUID handling)
- [ ] **Consolidate Shared Patterns:** Move `schema/patterns.rs` constants into `lib.rs` or a dedicated `crates/domain/src/validation.rs` module for cross-context reuse
- [ ] **Review Existing Structure:** Validate that current bounded context folders (`note/`, `schema/`, `template/`) have logical internal organization (e.g., `core.rs`, `validation.rs`, subentities)
- [ ] **Standardize Validation Logic:** Refactor validation methods across all contexts to use a consistent pattern (e.g., consistent naming, use of shared validation helpers in `lib.rs`, and uniform error propagation)
- [ ] **Centralized Events:** Maintain all domain events within the existing `crates/domain/src/events.rs` file
- [ ] Move shared internal utilities to `pub(crate)` modules within `crates/domain/src/`
- [ ] Refactor all domain components to use the consolidated utilities
- [ ] Ensure all consolidated code maintains the "no external I/O" hexagonal rule

### Task 3: Refactor Error Handling
- [ ] Consolidate shared error variants into a base domain error trait or common enum if appropriate
- [ ] Ensure consistent error context injection across the domain
- [ ] Verify `thiserror` is used correctly for all consolidated error types

### Task 4: Post-Refactoring Test Validation and Quality Assurance
- [ ] **CRITICAL:** Run `mise run test:unit` and confirm ALL tests pass without modification
- [ ] **Behavioral Verification:** Manually review test output to ensure no behavior changes occurred
- [ ] **Coverage Validation:** Run `mise run test:coverage` and confirm coverage percentage is equal to or greater than pre-refactoring baseline
- [ ] Run `mise run lint` to verify compliance with complexity and quality rules
- [ ] Run `mise run verify` for full gate check
- [ ] **HEXAGONAL CHECK:** Confirm `crates/domain` still has ZERO unauthorized external dependencies

### Task 5: Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] Run `mise run fmt` to format all code
- [ ] Run `mise run lint` to check for all code quality issues
- [ ] Run `mise run verify` for comprehensive verification
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS
- [ ] Stage all files
- [ ] Commit with conventional commit message: `refactor: consolidate domain internal utilities and shared core logic`

## Dev Notes

### Architectural Invariants
- **Test-First Refactoring (CRITICAL):** NO refactoring work begins until comprehensive unit tests exist for ALL domain logic to be modified. Tests are the safety net that ensures behavior preservation.
- **Behavior Preservation:** Refactoring MUST NOT change any observable behavior. All tests must pass without modification after refactoring is complete.
- **Domain Purity:** The domain crate MUST NOT have dependencies on `app`, `adapters`, or `lithos`.
- **Visibility:** Prefer `pub(crate)` for all internal utilities. Only export what is strictly necessary for the public API.
- **Deep Behavioral DRY:** Consolidation is based on **logic and intent**, not just name matches. If two different modules implement the same business rule or data manipulation logic, they must be consolidated into a single source of truth in the domain core.
- **Standardized Validation:** All bounded contexts must use the same "flavor" of validation (consistent error types, shared path logic, etc.) to ensure vault-wide consistency.
