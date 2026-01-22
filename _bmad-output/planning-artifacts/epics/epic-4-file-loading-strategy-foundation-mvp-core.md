# Epic 4: File Loading Strategy Foundation **[MVP CORE]**

System has unified file parsing strategies for different configuration formats that enable consistent deserialization and validation across the application.
**FRs covered:** Architecture requirements (file loading infrastructure)
**Implementation Notes:**

- **Strategy Pattern**: Unified parsing strategy for TOML, JSON, YAML files
- **Dispatcher**: Automatic file format detection by extension
- **Rich Errors**: Comprehensive parse errors with file paths, line numbers, and column context
- **Infrastructure Utility**: Implemented as an SPI helper in `crates/adapters/src/spi/parsers.rs`
- **Enabler**: Provides the foundation for Configuration (Epic 5), Schema (Epic 6), and Template (Epic 11) loading

## Story 4.1: Create Parser Strategy Interface

As a developer implementing file parsing across the application,
I want a unified strategy interface for parsing different file formats,
So that TOML, JSON, and YAML files can be parsed consistently with proper error handling.

**Acceptance Criteria:**

**Given** I need to parse different configuration file formats
**When** I create a parser strategy interface
**Then** it defines clear contracts for format detection and type-safe deserialization using `serde::de::DeserializeOwned`

**Given** the strategy interface exists
**When** I implement errors
**Then** a dedicated `ParseError` enum provides rich context (path, line, column) for all formats using `thiserror`

**Given** the architecture requires clean SPI
**When** I implement the module
**Then** it resides in `adapters/src/spi/parsers.rs` with clean, non-redundant naming and appropriate aliases in `spi/mod.rs`

## Story 4.2: Implement Format Strategies and Dispatcher

As a developer parsing configuration files,
I want reliable format strategies and an auto-detecting dispatcher,
So that files are correctly interpreted regardless of their format.

**Acceptance Criteria:**

**Given** I have files in different formats
**When** I implement the strategies
**Then** `Toml`, `Json`, and `Yaml` structs implement the strategy interface using `toml`, `serde_json`, and `serde_yaml` crates

**Given** multiple strategies exist
**When** I use the `Dispatcher`
**Then** it correctly identifies file types by extension (.toml, .json, .yaml, .yml) and dispatches to the correct strategy

**Given** parsing is executed
**When** errors occur
**Then** the dispatcher propagates rich error information including specific line numbers and syntax details via `ParseError`

## Story 4.3: Add Basic File Loading Validation

As a developer loading configuration files,
I want basic validation of loaded data,
So that obviously malformed files are caught early with helpful error messages.

**Acceptance Criteria:**

**Given** files are parsed into domain types
**When** I validate basic structure
**Then** checks for required top-level structure and basic type consistency are performed through strict Serde deserialization

**Given** validation fails
**When** I provide error messages
**Then** they include file path, line numbers, and suggested fixes via the `ParseError` context

## Story 4.4: Integrate Parsers with Domain Adapters

As a developer implementing domain ports,
I want to use the parser strategies in my adapters,
So that configuration, schemas, and templates can be loaded from the filesystem.

**Acceptance Criteria:**

**Given** the parser infrastructure exists
**When** I implement `ConfigQuery` or `SchemaQuery` adapters
**Then** they use the `Dispatcher` to handle file parsing after reading content from `tokio::fs`

**Given** integration is complete
**When** I run the system
**Then** configuration and schema files are loaded successfully with full type safety

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
**Then** all expensive operations are justified, and enum dispatch is used over trait objects for better performance

## Story 4.6: Review Epic 4 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 4 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** all Epic 4 components are implemented
**When** I conduct adversarial review
**Then** I ensure 100% coverage of format detection and error context propagation for all supported formats

## Story 4.7: Documentation Audit

As a developer maintaining the long-term health of the codebase,
I want a comprehensive audit of all Epic 4 documentation and doc comments,
So that the codebase remains self-documenting, precise, and free of unnecessary clutter.

**Acceptance Criteria:**

**Given** Epic 4 implementation is complete
**When** I conduct the audit
**Then** every public API has accurate, high-fidelity doc comments explaining the "Why" and showing usage examples
