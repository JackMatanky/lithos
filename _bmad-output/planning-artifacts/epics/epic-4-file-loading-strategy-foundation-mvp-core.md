# Epic 4: File Loading Strategy Foundation **[MVP CORE]**

System has unified file loading strategies for different configuration formats that enable consistent parsing and validation across the application.
**FRs covered:** Architecture requirements (file loading infrastructure)
**Implementation Notes:**
- Unified loading strategy for TOML, JSON, YAML files
- File format detection and parsing
- Basic validation infrastructure
- Enables both configuration (Epic 5) and schema (Epic 6) loading

## Story 4.1: Create Unified File Loading Interface

As a developer implementing file loading across the application,
I want a unified interface for loading different file formats,
So that TOML, JSON, and YAML files can be loaded consistently with proper error handling.

**Acceptance Criteria:**

**Given** I need to load different configuration file formats
**When** I create a unified loading interface
**Then** it supports TOML, JSON, and YAML with automatic format detection

**Given** the unified interface exists
**When** I load files
**Then** format detection works by file extension or content analysis

**Given** file loading fails
**When** I check error handling
**Then** clear error messages indicate format issues and file locations

## Story 4.2: Implement Format Detection and Parsing

As a developer parsing configuration files,
I want reliable format detection and parsing,
So that files are correctly interpreted regardless of their format.

**Acceptance Criteria:**

**Given** I have files in different formats
**When** I implement parsing
**Then** TOML files are parsed with toml crate, JSON with serde_json, YAML with serde_yaml

**Given** parsing is implemented
**When** I test format detection
**Then** it correctly identifies file types by extension (.toml, .json, .yaml, .yml)

**Given** files have parsing errors
**When** I handle them
**Then** errors include specific line numbers and syntax error details

## Story 4.3: Add Basic File Loading Validation

As a developer loading configuration files,
I want basic validation of loaded data,
So that obviously malformed files are caught early with helpful error messages.

**Acceptance Criteria:**

**Given** files are loaded
**When** I validate basic structure
**Then** checks for required top-level structure and basic type consistency

**Given** validation fails
**When** I provide error messages
**Then** they include file path, line numbers, and suggested fixes

**Given** basic validation passes
**When** I proceed with application-specific validation
**Then** the data is ready for domain-specific processing

## Story 4.4: Create Loading Strategy Mocks for Testing

As a developer testing file loading functionality,
I want mocks for the loading strategy,
So that I can test file loading in isolation without actual file system operations.

**Acceptance Criteria:**

**Given** I need to test loading strategies
**When** I create mocks
**Then** mock implementations allow testing different file formats and error conditions

**Given** mocks are available
**When** I write unit tests
**Then** tests can verify loading logic without file system dependencies

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate real file loading behavior for comprehensive testing

## Story 4.5: Adversarial Refactor of Epic 4 Foundation

As an adversarial senior developer,
I want to brutally review and refactor the Epic 4 loading foundation,
So that it follows the leanest, most performant idiomatic Rust practices, balances OOP/FP principles, and eliminates technical debt before testing and documentation.

**Acceptance Criteria:**

**Given** the Epic 4 implementation is complete
**When** I conduct an adversarial refactor
**Then** the code follows strict SRP (Single Responsibility Principle) and has zero redundant logic

**Given** Rust-specific performance targets
**When** I optimize the code
**Then** all expensive operations (clones, allocations) are justified or eliminated in favor of zero-copy patterns where possible

**Given** the hybrid OOP/FP architecture
**When** I review the implementation
**Then** I ensure appropriate use of traits (OOP) vs functional patterns (iterators, closures, immutable state)

## Story 4.6: Review Epic 4 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 4 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** docs/testing/developer-guide.md provides testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, and utilities

**Given** all Epic 4 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage

## Story 4.7: Comprehensive Documentation Audit

As a developer maintaining the long-term health of the codebase,
I want a comprehensive audit of all Epic 4 documentation and doc comments,
So that the codebase remains self-documenting, precise, and free of unnecessary clutter.

**Acceptance Criteria:**

**Given** Epic 4 implementation and testing are complete
**When** I conduct the documentation audit
**Then** every public API has accurate, high-fidelity doc comments (///)

**Given** the "Why" mandate in project-context.md
**When** I review doc comments
**Then** they focus on invariants and architectural context rather than just repeating the function name

**Given** doc-tests are used as "Living Documentation"
**When** I audit the examples
**Then** every doc-test is accurate, functional, and demonstrates idiomatic usage without boilerplate noise
