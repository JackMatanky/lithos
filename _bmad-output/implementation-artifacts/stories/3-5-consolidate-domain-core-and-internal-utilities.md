# Story 3.5: Consolidate Domain Core and Internal Utilities

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer maintaining the domain layer,
I want to consolidate shared logic and internal utilities into a central `lib.rs` and core modules within the domain crate,
So that the codebase remains DRY, maintainable, and architectural boundaries are respected.

## Acceptance Criteria

1. **Given** redundant utility functions and shared logic exist across Note, Schema, Config, and Template bounded contexts
   **When** I refactor the domain crate
   **Then** shared logic is moved to `crates/domain/src/lib.rs` or internal core modules

2. **Given** internal domain utilities are exposed publicly
   **When** I review visibility
   **Then** internal utilities use `pub(crate)` to prevent leaking into the application layer

3. **Given** the domain crate is the "inviolate core"
   **When** I consolidate utilities
   **Then** ZERO external dependencies (except justified ones like `serde` or `thiserror`) are introduced

4. **Given** common patterns like UUID v7 handling or shared error mapping are used
   **When** I implement core utilities
   **Then** they are implemented once in the core and reused across all bounded contexts

5. **Given** the models directory is flattened
   **When** I complete the refactor
   **Then** models are organized into bounded context subfolders (Note, Schema, Template), solitary models like `config.rs` remain at the root, and shared constants/utilities are consolidated in `lib.rs`, with domain events remaining centralized

6. **Given** inconsistent validation patterns across models
   **When** I consolidate utilities
   **Then** validation methods follow a standardized pattern and share common logic via the domain core

## Tasks / Subtasks

### Task 1: Audit Domain Crate for Redundancy
- [ ] Scan `crates/domain/src/models/` for duplicate utility functions (path handling, string manipulation, etc.)
- [ ] **Audit Validation Patterns:** Identify all `validate()` methods and internal validation helpers across models to identify inconsistent patterns or redundant logic
- [ ] Identify shared logic between Note, Schema, Config, and Template contexts
- [ ] Review error mapping patterns and identify opportunities for consolidation
- [ ] Document all identified redundancies and validation inconsistencies in `_bmad-output/domain-redundancy-audit.md`

### Task 2: Consolidate into lib.rs and Core Modules
- [ ] Create or update `crates/domain/src/lib.rs` to house shared traits, macros, constants (from `patterns.rs`), and common identity logic (UUID handling)
- [ ] **Restructure Domain Models:** Organize `crates/domain/src/models/` into bounded context subfolders:
    - `note/`: Note aggregate, Tag, Task, Link, Frontmatter, Structure
    - `schema/`: Schema aggregate, PropertyBank, PropertySpec, Property
    - `template/`: Template aggregate, Syntax, Variable, Composition
- [ ] **Root-Level Models:** Keep `config.rs` at the `models/` root level as it remains a solitary file
- [ ] **Modularize Large Models:** Split complex aggregates like `Note` into focused files within their respective subfolders (e.g., `note/mod.rs` for aggregate and `note/validation.rs` for logic)
- [ ] **Standardize Validation Logic:** Refactor validation methods to use a consistent pattern (e.g., consistent naming, use of shared validation helpers in `lib.rs`, and uniform error propagation)
- [ ] **Centralized Events:** Maintain all domain events within the existing `crates/domain/src/events.rs` file
- [ ] Move shared internal utilities to `pub(crate)` modules within `crates/domain/src/`
- [ ] Refactor all domain components to use the consolidated utilities and new module paths
- [ ] Ensure all consolidated code maintains the "no external I/O" hexagonal rule

### Task 3: Refactor Error Handling
- [ ] Consolidate shared error variants into a base domain error trait or common enum if appropriate
- [ ] Ensure consistent error context injection across the domain
- [ ] Verify `thiserror` is used correctly for all consolidated error types

### Task 4: Verification and Quality Assurance
- [ ] Run `mise run test:unit` to ensure no regressions in domain logic
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
- **Domain Purity:** The domain crate MUST NOT have dependencies on `app`, `adapters`, or `lithos`.
- **Visibility:** Prefer `pub(crate)` for all internal utilities. Only export what is strictly necessary for the public API.
- **DRY Logic:** If a piece of logic is used in more than two bounded contexts, it belongs in the domain core.
