# Story 4.2: implement-path-validation-utilities

Status: review

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
- [x] Write failing unit tests for path traversal (`..`) rejection
- [x] Write failing unit tests for absolute path rejection
- [x] Write failing unit tests for restricted (hidden) file detection
- [x] Write failing async unit tests for **Strict** symlink resolution security (escaped link rejection)
- [x] Write failing async unit tests for **Flexible** symlink resolution (external link acceptance)
- [x] Write failing unit tests for valid path acceptance
- [x] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Path Validation Utilities (GREEN Phase - AC: All)
- [x] Implement `adapters/src/spi/fs/validator.rs` module structure
- [x] Implement `Validator` struct pattern
  - [x] internal configuration (strict vs flexible modes)
  - [x] `new_flexible()` factory method
  - [x] `new_strict(root: PathBuf)` factory method
- [x] Implement validation logic
  - [x] traversal + absolute check (Synchronous)
  - [x] restricted/hidden file check (Synchronous)
  - [x] symlink loop verification
- [x] Implement `resolve_safe_symlink` method
  - **CRITICAL:** Must use `tokio::fs::read_link` or `tokio::fs::canonicalize`.
  - **CRITICAL:** Do NOT use `std::fs` to avoid blocking the runtime.
- [x] Update return types to use `Cow<'a, Path>` to avoid unnecessary allocations
- [x] Define `PathValidationError` enum in the same module using `thiserror`
- [x] **TDD REQUIREMENT:** Make all validation tests pass (GREEN phase complete when all tests pass)

### Task 3: Refactor for Quality (REFACTOR Phase - AC: All)
- [x] Ensure `PathValidationError` implements `thiserror::Error` for proper context
- [x] Optimize string checking to avoid unnecessary allocations (use `AsRef<Path>` and components iteration)
- [x] Ensure all public functions have proper doc comments
- [x] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 4: Comprehensive Documentation (REFACTOR Phase - AC: All)
- [x] Add module-level documentation `//!` explaining security guarantees and "Strict vs Flexible" modes
- [x] Add example usage in doc comments for all public functions
- [x] Ensure doctests run and pass
- [x] **TDD REQUIREMENT:** `cargo test --doc` passes successfully

### Task 5: Quality Assurance and Commit (MANDATORY FINAL TASK)
- [x] **TDD VALIDATION:** Confirm all tests pass
- [x] Run `mise run fmt`
- [x] Run `mise run lint`
- [x] Run `mise run verify`
- [x] **CRITICAL:** Fix ALL linter warnings
- [x] Commit with conventional commit message: `feat(adapters): implement secure path validation utilities`

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

## Dev Agent Record

### Implementation Plan
Story 4.2 implements secure path validation utilities following strict TDD principles:
- RED phase: Created 22 comprehensive failing tests covering all security scenarios
- GREEN phase: Implemented minimal Validator with Strict/Flexible modes
- REFACTOR phase: Optimized for zero-allocation Cow<Path>, fixed all clippy warnings

### Completion Notes
✅ **All Tasks Complete** (2026-01-22)

**Implementation Summary:**
- Created `crates/adapters/src/spi/fs/validator.rs` with full security validation
- Implemented `Validator` with two modes:
  - **Flexible**: Allows external symlinks (dotfiles), enforces input traversal checks
  - **Strict**: Enforces root boundary, rejects escaping symlinks
- Validation checks: path traversal (`..`), absolute paths, hidden files (`.git`, `.env`, etc.)
- Async symlink resolution using `tokio::fs::canonicalize` (no blocking I/O)
- Zero-allocation path validation returning `Cow<'_, Path>`
- Comprehensive error types via `thiserror` with rich context

**Test Coverage:**
- 52 unit tests organized in submodules (all passing)
  - constructor: Validator creation tests
  - path_traversal: .. and parent directory attacks
  - absolute_paths: Unix/Windows absolute path handling
  - restricted_files: Hidden file (.git, .env, .ssh) detection
  - symlink_strict: Strict mode symlink validation
  - symlink_flexible: Flexible mode external symlink handling
  - valid_paths: Normal path acceptance and Cow optimization
  - platform_specific: Cross-platform separator handling
- 6 doctests (all passing)
- Total: 58 tests providing comprehensive security coverage

**Quality Gates:**
- ✅ All tests pass (58 total for path validation: 52 unit + 6 doc)
- ✅ `mise run fmt` - formatting complete
- ✅ `mise run lint` - no warnings (clippy with `-D warnings`)
- ✅ `mise run verify` - full quality gates passed
- ✅ Pre-commit hooks - all passed

**Technical Decisions:**
- Used `let chain` pattern for idiomatic Rust (requires edition 2024)
- Applied `#[non_exhaustive]` to PathValidationError for future extensibility
- Added `#[inline]` hints for small factory methods
- Allowed `clippy::pattern_type_mismatch` with reason for match ergonomics

## File List

### New Files
- `crates/adapters/src/spi/fs/validator.rs` - Path validation utilities with comprehensive unit tests (677 lines total: 351 implementation + 326 tests)

### Modified Files
- `crates/adapters/src/spi/fs/mod.rs` - Added validator module export
- `crates/adapters/Cargo.toml` - Added tempfile dev dependency

## Change Log
- **2026-01-22**: Implemented secure path validation utilities (Story 4.2)
  - Created Validator with Strict/Flexible modes for path security
  - Added comprehensive test suite (52 unit tests + 6 doctests = 58 total)
  - Tests organized by domain: constructor, path_traversal, absolute_paths, restricted_files, symlink_strict, symlink_flexible, valid_paths, platform_specific
  - Implemented async-safe symlink resolution using tokio::fs
  - Zero-allocation validation using Cow<'_, Path>
  - All quality gates passed (fmt, lint, verify, pre-commit hooks)
  - Commits: 000d043d (initial impl), e8c535f3 (test reorganization)
