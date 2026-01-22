# Epic 4: File Loading Strategy Foundation **[MVP CORE]**

System has unified file parsing strategies and security utilities that enable consistent deserialization and validation across the application.
**FRs covered:** Architecture requirements (file loading infrastructure)
**Implementation Notes:**

- **Parser Strategy**: Unified parsing utility for TOML, JSON, YAML files (Completed in 4.1)
- **Security Utilities**: Stateless path validation to prevent traversal attacks (Story 4.2)
- **Location**: All file-system related SPI helpers (parsers, validators) reside in `crates/adapters/src/spi/fs/`
- **Pattern**: Adapters (Config, Schema) will use `tokio::fs` directly, validated by 4.2 utils, parsed by 4.1 dispatcher.

## Story 4.1: Create Parser Strategy Interface

As a developer implementing file parsing across the application,
I want a unified strategy interface for parsing different file formats,
So that TOML, JSON, and YAML files can be parsed consistently with proper error handling.

**Acceptance Criteria:**

**Given** I need to parse different configuration file formats
**When** I create a parser strategy interface
**Then** it defines clear contracts for format detection and type-safe deserialization

**Given** the strategy interface exists
**When** I implement errors
**Then** a dedicated `ParseError` enum provides rich context (path, line, column)

**Given** the architecture requires clean SPI
**When** I implement the module
**Then** it resides in `adapters/src/spi/fs/parsers.rs` (Refactor: move from `spi/parsers.rs` if necessary)

## Story 4.2: Implement Path Validation Utilities

As a developer implementing adapters,
I want shared path validation utilities,
So that I can prevent path traversal and enforce security rules without duplicating logic in every adapter.

**Acceptance Criteria:**

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
**When** I use the safe resolution utility
**Then** it ensures the resolved target path resides within the intended root directory, returning a `SymlinkEscapeError` if it escapes

**Given** valid inputs
**When** validation succeeds
**Then** it returns a sanitized `PathBuf` ready for safe I/O operations

## Story 4.3: Epic 4 Adversarial Refactor

As an adversarial senior developer,
I want to brutally review and refactor the Epic 4 loading foundation,
So that it follows the leanest, most performant idiomatic Rust practices, balances OOP/FP principles, and eliminates technical debt before testing and documentation.

**Acceptance Criteria:**

**Given** the Epic 4 implementation is complete
**When** I conduct an adversarial refactor
**Then** the code follows strict SRP (Single Responsibility Principle) and has zero redundant logic

**Given** memory usage concerns
**When** I optimize the code
**Then** all `clone()` operations are justified or removed in favor of zero-copy patterns where possible

**Given** data structure choices
**When** I review the implementation
**Then** enum memory layouts are verified (using `std::mem::size_of`) to ensure compact representation

## Story 4.4: Review Epic 4 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 4 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** public utility functions in `parsers.rs` and `validator.rs`
**When** I review the test suite
**Then** each function has a corresponding unit test ensuring logical correctness

**Given** distinct error conditions (Traversal, Restricted, SymlinkEscape, ParseError)
**When** I check test coverage
**Then** specific test cases exist for _each_ error variant to ensure robust error propagation

**Given** public API components
**When** I run `cargo test --doc`
**Then** all documentation tests pass successfully

## Story 4.5: Documentation Audit

As a developer maintaining the long-term health of the codebase,
I want a comprehensive audit of all Epic 4 documentation and doc comments,
So that the codebase remains self-documenting, precise, and free of unnecessary clutter.

**Acceptance Criteria:**

**Given** public modules in `crates/adapters/src/spi/fs/`
**When** I conduct the audit
**Then** each module has `//!` level documentation explaining its domain purpose and usage

**Given** public functions and structs
**When** I conduct the audit
**Then** they have `///` comments including a `# Examples` section with runnable code snippets

**Given** error enums
**When** I conduct the audit
**Then** each variant is documented with the specific condition that triggers it
