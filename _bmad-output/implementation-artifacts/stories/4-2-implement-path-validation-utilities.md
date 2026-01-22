# Story 4.2: implement-path-validation-utilities

Status: ready-for-dev

<!-- Note: Validation completed. Quality standards enforced. -->

## Story

As a developer implementing adapters,
I want shared path validation utilities,
So that I can prevent path traversal and enforce security rules without duplicating logic in every adapter.

## Acceptance Criteria

**Given** the architecture requires clean SPI
**When** I implement the module
**Then** it resides in `adapters/src/spi/fs/validator.rs`

**Given** a raw file path string containing `..` components (e.g., `../../etc/passwd`)
**When** I use the validation utility
**Then** it returns a specific `PathTraversalError` and rejects the path

**Given** an absolute path (e.g., `/etc/hosts`) provided to a relative-only validator
**When** I use the validation utility
**Then** it returns an `AbsolutePathError` ensuring only vault-relative paths are accepted

**Given** a path containing hidden components (e.g., `.git/config`, `.env`)
**When** I use the validation utility with default settings
**Then** it returns a `RestrictedPathError` to protect sensitive files

**Given** a symbolic link path
**When** I use the safe resolution utility with a **Validator configured as Strict (with root)**
**Then** it ensures the resolved target path resides within the intended root directory, returning a `SymlinkEscapeError` if it escapes (e.g., for Vault security)

**Given** a symbolic link path
**When** I use the safe resolution utility with **Validator configured as Flexible (no root)**
**Then** it follows the symlink safely (allowing dotfiles) while still enforcing traversal checks on the input path

**Given** valid inputs
**When** validation succeeds
**Then** it returns a sanitized `Cow<'a, Path>` ready for safe I/O operations

**Given** I am running on any platform (Windows/Unix)
**When** I run the tests
**Then** all path logic handles platform-specific separators correctly

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Test Cases (RED Phase - AC: All)
- [ ] Write failing unit tests for path traversal (`..`) rejection
- [ ] Write failing unit tests for absolute path rejection
- [ ] Write failing unit tests for restricted (hidden) file detection
- [ ] Write failing async unit tests for **Strict** symlink resolution security (escaped link rejection)
- [ ] Write failing async unit tests for **Flexible** symlink resolution (external link acceptance)
- [ ] Write failing unit tests for valid path acceptance
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Path Validation Utilities (GREEN Phase - AC: All)
- [ ] Implement `adapters/src/spi/fs/validator.rs` module structure
- [ ] Implement `Validator` struct pattern
  - [ ] internal configuration (strict vs flexible modes)
  - [ ] `new_flexible()` factory method
  - [ ] `new_strict(root: PathBuf)` factory method
- [ ] Implement validation logic
  - [ ] traversal + absolute check (Synchronous)
  - [ ] restricted/hidden file check (Synchronous)
  - [ ] symlink loop verification
- [ ] Implement `resolve_safe_symlink` method
  - **CRITICAL:** Must use `tokio::fs::read_link` or `tokio::fs::canonicalize`.
  - **CRITICAL:** Do NOT use `std::fs` to avoid blocking the runtime.
- [ ] Update return types to use `Cow<'a, Path>` to avoid unnecessary allocations
- [ ] Define `PathValidationError` enum in the same module using `thiserror`
- [ ] **TDD REQUIREMENT:** Make all validation tests pass (GREEN phase complete when all tests pass)

### Task 3: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Ensure `PathValidationError` implements `thiserror::Error` for proper context
- [ ] Optimize string checking to avoid unnecessary allocations (use `AsRef<Path>` and components iteration)
- [ ] Ensure all public functions have proper doc comments
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 4: Comprehensive Documentation (REFACTOR Phase - AC: All)
- [ ] Add module-level documentation `//!` explaining security guarantees and "Strict vs Flexible" modes
- [ ] Add example usage in doc comments for all public functions
- [ ] Ensure doctests run and pass
- [ ] **TDD REQUIREMENT:** `cargo test --doc` passes successfully

### Task 5: Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] **TDD VALIDATION:** Confirm all tests pass
- [ ] Run `mise run fmt`
- [ ] Run `mise run lint`
- [ ] Run `mise run verify`
- [ ] **CRITICAL:** Fix ALL linter warnings
- [ ] Commit with conventional commit message: `feat(adapters): implement secure path validation utilities`

## Dev Notes

### Developer Context
This story implements the security foundation for all file-based adapters. Instead of repeating `..` checks in every adapter, we centralized this logic in a configurable `Validator` struct. This ensures consistent security policy enforcement across the application.

**Business Value:** Prevents security vulnerabilities (Path Traversal, Arbitrary File Access) in all file operations.

**Technical Context:**
- Location: `crates/adapters/src/spi/fs/validator.rs`
- Usage: Will be used by ConfigAdapter (Flexible mode), SchemaAdapter (Strict/Flexible mixed), and NoteAdapter (Strict mode).
- Dependencies: `std::path`, `thiserror`, `tokio` (for async symlink resolution).

### Anti-Pattern Prevention
- **❌ Domain Pollution**: Do NOT put this in `crates/domain`. This is an infrastructure concern (file system security).
- **❌ Partial Checks**: Do NOT just check for `..` string. Use `Path::components()` to handle normalization correctly.
- **❌ Symlink Blindness**: Do NOT ignore symlinks. They are a primary vector for escaping jail roots.
- **❌ Blocking I/O**: Do NOT use `std::fs` for symlink resolution. Use `tokio::fs` as this utility will be called from async adapters.
- **❌ Over-Strictness**: Do NOT blindly reject all symlinks. Allow dotfiles support via the `Validator` configuration.

### References
- Epic 4: `_bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md`
- Project Context: `_bmad-output/project-context.md`
- ADR 0015: `docs/adr/0015-file-loading-port-boundary.md`
