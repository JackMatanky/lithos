---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-02-advanced-elicitation-complete']
inputDocuments:
  - label: PRD
    path: _bmad-output/planning-artifacts/prd.md
    category: planning
  - label: Architecture
    path: _bmad-output/planning-artifacts/architecture.md
    category: planning
  - label: UX Design Specification
    path: _bmad-output/planning-artifacts/ux-design-specification.md
    category: planning
  - label: Project Context
    path: _bmad-output/project-context.md
    category: technical
  - label: Go Implementation Lessons Learned (2025-11-05)
    path: _archive/go-implementation/_bmad-output/implementation-artifacts/course_correction/2025-11-05-comprehensive-architectural-review.md
    category: lessons_learned
---

# lithos - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for lithos, decomposing the requirements from the PRD, UX Design, Architecture, and Go implementation lessons learned into implementable stories.

## Requirements Inventory

### Functional Requirements

#### Template Management (FR1-FR7)
- FR1: Users can create modular templates composed of reusable sections with variables
- FR2: Users can execute templates interactively with prompts, suggesters, and multi-suggesters
- FR3: Users can compose complex templates from multiple sections with error prevention
- FR4: Users can apply date formatting and manipulation functions to template content
- FR5: Users can include dynamic commands and whitespace control in templates
- FR6: Users can define and use custom user functions within templates
- FR7: Users can execute advanced template operations with hooks and complex commands

#### Schema Management (FR8-FR14)
- FR8: Users can define metadata schemas with field types (string, number, date, file, boolean)
- FR9: Users can create schema-driven templates where field properties provide input parameters
- FR10: Users can validate notes against schemas with clear error feedback
- FR11: Users can use schema enums to populate suggester options in templates
- FR12: Users can filter file selections using schema-defined directory constraints
- FR13: Users can format dates using schema-defined format strings
- FR14: Users can inherit and extend schema definitions between related types

#### Interactive Input (FR15-FR19)
- FR15: Users can provide free-text input through template prompts
- FR16: Users can select from single-choice lists using suggesters
- FR17: Users can select multiple items from lists using multi-suggesters
- FR18: Users can receive contextual help and guidance during input
- FR19: Users can access progressive complexity modes for different expertise levels

#### Vault Operations (FR20-FR25)
- FR20: Users can index and search notes across entire vaults
- FR21: Users can perform lookups by filename, path, or schema-defined keys
- FR22: Users can resolve wiki-style links and aliases throughout vaults
- FR23: Users can query metadata fields from other notes for template use
- FR24: Users can maintain vault consistency across template operations
- FR25: Users can handle large vaults (1000+ files) without performance degradation

#### Configuration Management (FR26-FR29)
- FR26: Users can configure template packs using TOML files
- FR27: Users can manage schema definitions through configuration files
- FR28: Users can set application preferences via configuration
- FR29: Users can define custom validation rules and linting settings

#### Cross-Environment Compatibility (FR30-FR33)
- FR30: Users can execute templates consistently across operating systems
- FR31: Users can access templates through terminal interfaces
- FR32: Users can integrate with external editors and IDEs
- FR33: Users can run templates in automated scripts and CI/CD pipelines

#### Community Features (FR34-FR37)
- FR34: Users can share and distribute template packs via Git repositories
- FR35: Users can discover and adopt community-created template packs
- FR36: Users can validate third-party templates against schemas
- FR37: Users can contribute improvements to shared template ecosystems

#### Security & Privacy (FR38-FR40)
- FR38: Users can control access to sensitive vault data and templates
- FR39: Users can encrypt sensitive configuration and schema files
- FR40: Users can audit template execution and data access patterns

#### Command Line Interface (FR41-FR47)
- FR41: Users can execute lithos commands with subcommands for templates, schemas, and vaults
- FR42: Users can access comprehensive help and documentation from the CLI
- FR43: Users can view status and configuration of templates and schemas
- FR44: Users can manage vault operations (index, search, validate) from command line
- FR45: Users can run templates with various output formats and destinations
- FR46: Users can configure CLI behavior and preferences
- FR47: Users can execute most important commands with single words (e.g., `lithos new` opens fuzzy picker for template selection)

#### Error Handling & Recovery (FR48-FR50)
- FR48: Users can receive clear, actionable error messages when operations fail
- FR49: Users can recover from failed template executions with rollback capabilities
- FR50: Users can diagnose and troubleshoot configuration and schema issues

### NonFunctional Requirements

#### Performance (NFR1-NFR4)
- NFR1: Template execution completes in under 500ms for individual operations
- NFR2: Vault indexing completes in under 2 seconds for 1000+ files
- NFR3: File I/O operations maintain efficient read/write performance for large vault scalability
- NFR4: CLI commands provide instant feedback and help

#### Security (NFR5-NFR7)
- NFR5: Sensitive configuration and schema files are encrypted at rest
- NFR6: Users control access permissions for vault data and templates
- NFR7: Template execution and data access are logged for auditing

#### Scalability (NFR8-NFR10)
- NFR8: System handles vaults with thousands of files without performance degradation
- NFR9: Memory usage remains bounded under 500MB for typical operations
- NFR10: Multiple template executions run concurrently without interference

#### Integration (NFR11-NFR12)
- NFR11: MVP supports macOS, with Linux added if implementation complexity is minimal
- NFR12: CLI integrates reliably with terminal environments

#### Usability (NFR13-NFR15)
- NFR13: CLI provides clear help, auto-completion, and command discoverability
- NFR14: Error messages are actionable and help users troubleshoot issues
- NFR15: Progressive complexity modes accommodate different user expertise levels

#### Maintainability (NFR16-NFR18)
- NFR16: Code maintains comprehensive test coverage (80%+) and contributor documentation
- NFR17: Binary distribution provides self-contained executables without external dependencies
- NFR18: Safe rollback and version management support system updates

#### Compatibility (NFR19-NFR20)
- NFR19: System gracefully handles Obsidian vault structure changes
- NFR20: Migration paths support transition from existing template workflows

#### Observability (NFR21-NFR23)
- NFR21: Comprehensive logging enables debugging of template execution and vault operations
- NFR22: Performance metrics track system behavior for optimization
- NFR23: Diagnostic tools help users identify and resolve issues

#### Reliability (NFR24-NFR26)
- NFR24: System achieves 99.9% uptime for CLI operations
- NFR25: Zero crashes during normal vault operations
- NFR26: Failed operations provide clear recovery paths and state preservation

#### Deployment (NFR27-NFR29)
- NFR27: Binary updates complete successfully in under 30 seconds with automatic rollback on failure
- NFR28: Installation process succeeds for 95% of users without manual intervention
- NFR29: Version compatibility maintained across patch releases

### Additional Requirements

**Architectural Foundation:**
- **Workspace-Based Hexagonal Architecture**: Cargo Workspaces with strict hexagonal boundaries (domain → app → adapters → cli)
- **CQRS Pattern**: Separate Command (Write) and Query (Read) models with hybrid event bus
- **Unit of Work (Transactional Context)**: Atomic commands with TransactionContext and deferred domain event dispatch
- **Dependency Injection**: Constructor injection with `Arc<dyn Trait>` for Port implementations

**Technology Stack:**
- **Core Runtime**: Rust 1.92+ with Tokio 1.49 async runtime ('full' features enabled)
- **Storage Engine**: Redb 3.1 + rkyv 0.8 (Zero-copy deserialization, ACID KV storage)
- **Identity**: UUID v7 (Time-ordered, sortable unique identifiers)
- **Template Engine**: MiniJinja 2.14 (VM-based template engine for user-defined Markdown templates)
- **Markdown Parser**: pulldown-cmark 0.13 (Event-streaming parser for high-speed link extraction)
- **Configuration**: Figment 0.10 (Hierarchical configuration: Global → User → Project → Vault)
- **Error Handling**: miette 7.6 (High-fidelity terminal diagnostics) + thiserror 2.0 (Structured errors) + anyhow 1.0 (Context chaining)
- **Tooling**: mise 2026.1.0 (Primary task orchestration and tool version management)
- **Quality Gates**: pre-commit hooks (clippy, rustfmt, complexity checks mandatory before commits)

**Code Quality Standards:**
- **Cognitive Complexity**: Hard limit of 15 (warn) / 25 (deny) enforced via clippy
- **Test Coverage**: Target 80%+ coverage via tarpaulin, with nextest for concurrent test execution
- **No-Panic Policy**: Never use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code
- **Async Safety**: Never block async threads >10ms; use `spawn_blocking` for std::fs and heavy CPU tasks
- **Lock Discipline**: Never hold `std::sync::MutexGuard` across `.await` points

**Critical Rust Implementation Rules:**
- **Hexagonal Boundary Enforcement**: `crates/domain` must have NO external dependencies and NO I/O; all Ports defined as `#[async_trait]` traits
- **Validation Layers**: Syntactic validation (structure/format) in Adapter layer; Semantic validation (business rules) in Domain layer
- **Path Protocol**: Use `PathBuf` (owned) or `&Path` (borrowed) for all file paths; NEVER use `String` for paths
- **Persistence Strategy**: rkyv usage isolated to `adapters/spi/storage` layer; domain entities remain ergonomically usable
- **Error Standards**: Layered errors (thiserror in domain, anyhow + .context() in app/adapters, miette in CLI)

**Go Implementation Lessons Learned (Critical Architectural Insights):**

**Validation Architecture (Group 1 - FOUNDATIONAL):**
- **Anemic Domain Model Anti-Pattern**: Go implementation had entities as "data bags" with all logic in services, violating DDD rich model principles
- **Rust Approach**: Domain entities MUST own logic pertaining to their own data; use builder patterns and validation methods on entities
- **IO in Domain Layer Violation**: Go FrontmatterService.Extract() performed file parsing (goldmark) in domain layer
- **Rust Approach**: Extract frontmatter in adapter layer; domain receives pre-parsed data

**Storage Architecture (Group 2 - FOUNDATIONAL):**
- **CQRS Confusion**: Go implementation separated ports (CacheWriterPort/CacheReaderPort) but not models
- **Rust Approach**: Consider separate read/write models (NoteProjection vs Note) if query optimization requires it
- **Storage Write Coordination**: Go implementation had no coordination pattern for dual-write to BoltDB + SQLite
- **Rust Approach**: Implement Unit of Work pattern from day one; use TransactionContext for atomic operations
- **DTO Architecture**: Go DTOs (FileMetadata, VaultFile) were too generic; didn't leverage stdlib patterns (fs.FileInfo)
- **Rust Approach**: Create storage-specific DTOs; leverage Rust's Path/PathBuf and filesystem abstractions

**Event-Driven Architecture (Group 3 - CRITICAL):**
- **God-Object Orchestrators**: Go implementation used god-object services mixing concerns
- **Rust Approach**: Implement event-driven architecture from day one; use Hybrid Event Bus (MPSC for indexing, broadcast for signals, watch for LSP state)
- **Domain Events**: NoteIndexed, VaultIndexingCompleted, TemplateExecuted, SchemaLoaded for loose coupling

**Configuration Management (Group 4):**
- **Singleton Anti-Pattern**: Go implementation had improper singleton for Config and PropertyBank
- **Rust Approach**: Use `OnceCell` for lazy initialization with thread-safe access; Arc<Config> for shared immutable configuration

**Schema Domain System (Group 5):**
- **Port Coupling**: Go had unnecessary complexity with separate SchemaLoaderPort and SchemaRegistryPort
- **Rust Approach**: Simplify to single SchemaPort; automatic registration on load

**Template System (Group 6 - Epic 6 Dependency):**
- **Template Struct Name Conflict**: Go had naming collision with text/template package
- **Rust Approach**: Carefully namespace template domain models; leverage MiniJinja without conflicts

**Critical Success Factors:**
- **Hexagonal Architecture**: Strictly enforce boundaries from day one; domain layer pure logic with zero I/O
- **Test Architecture**: Mirror hexagonal structure (pure domain tests, adapter integration tests, E2E CLI tests)
- **Async Discipline**: Use tokio::test for integration tests; surface race conditions early
- **Performance Benchmarking**: Use criterion for NFR-critical paths; 10k-note vault benchmarks mandatory

### FR Coverage Map

- FR1 → Epic 11 (Modular templates with reusable sections)
- FR2 → Epic 11 (Interactive template execution with prompts/suggesters)
- FR3 → Epic 12 (Complex template composition with error prevention)
- FR4 → Epic 12 (Date formatting and manipulation functions)
- FR5 → Post-MVP Phase 1.5 (Dynamic commands and whitespace control)
- FR6 → Post-MVP Phase 1.5 (Custom user functions)
- FR7 → Post-MVP Phase 2a (Advanced template operations with hooks)
- FR8 → Epic 6 (Define metadata schemas with field types)
- FR9 → Epic 6 (Schema-driven templates with input parameters)
- FR10 → Epic 6 (Validate notes against schemas)
- FR11 → Epic 6 (Schema enums populate suggester options)
- FR12 → Epic 6 (File filtering via schema directory constraints)
- FR13 → Epic 6 (Date formatting via schema format strings)
- FR14 → Epic 6 (Schema inheritance and extension)
- FR15 → Epic 11 (Free-text input through prompts)
- FR16 → Epic 11 (Single-choice suggesters)
- FR17 → Epic 12 (Multi-selection suggesters)
- FR18 → Post-MVP Phase 1.5 (Contextual help and guidance)
- FR19 → Post-MVP Phase 1.5 (Progressive complexity modes)
- FR20 → Epic 9 (Index and search notes across vaults)
- FR21 → Epic 10 (Lookups by filename, path, schema keys)
- FR22 → Epic 10 (Resolve wiki-links and aliases)
- FR23 → Epic 10 (Query metadata from other notes)
- FR24 → Epic 9 (Maintain vault consistency)
- FR25 → Epic 9 (Handle large vaults without degradation)
- FR26 → Epic 5 (Configure template packs via TOML)
- FR27 → Epic 5 (Manage schema definitions via config)
- FR28 → Epic 5 (Set application preferences via config)
- FR29 → Post-MVP Phase 2c (Custom validation/linting rules)
- FR30 → Epic 13 (Cross-platform execution consistency)
- FR31 → Epic 13 (Access through terminal interfaces)
- FR32 → Post-MVP Phase 3a (External editor/IDE integration)
- FR33 → Post-MVP Phase 3a (Automated scripts/CI-CD support)
- FR34 → Post-MVP Phase 3b (Share template packs via Git)
- FR35 → Post-MVP Phase 3b (Discover community template packs)
- FR36 → Post-MVP Phase 3b (Validate third-party templates)
- FR37 → Post-MVP Phase 3b (Community contribution workflows)
- FR38 → Post-MVP Phase 4 (Access control for vault data)
- FR39 → Post-MVP Phase 4 (Encrypt sensitive configuration) - Basic version in Epic 5
- FR40 → Post-MVP Phase 4 (Audit template execution) - Basic version in Epic 13
- FR41 → Epic 13 (Execute commands with subcommands)
- FR42 → Epic 13 (Comprehensive help and documentation)
- FR43 → Epic 13 (View status and configuration)
- FR44 → Epic 13 (Manage vault operations from CLI)
- FR45 → Epic 13 (Run templates with output formats)
- FR46 → Epic 13 (Configure CLI behavior)
- FR47 → Epic 13 (Single-word command shortcuts)
- FR48 → Epic 13 (Clear, actionable error messages)
- FR49 → Epic 13 (Recover from failed executions with rollback)
- FR50 → Epic 13 (Diagnose and troubleshoot issues)

## Epic 1: Development Environment & Tooling

Developers have a fully configured development environment with quality gates, testing infrastructure, and task orchestration that enforces architectural standards.
**FRs covered:** Architecture requirements (tooling, quality gates)
**Implementation Notes:**
- Cargo workspace structure (4 crates: domain, app, adapters, cli)
- mise.toml with task orchestration (test, bench, coverage, watch, etc.)
- pre-commit-config.yaml with stringent quality gates
- clippy.toml with cognitive complexity < 15 (warn) / 25 (deny) and all anti-pattern denies
- rustfmt.toml with import sorting and formatting standards
- deny.toml for dependency security auditing
- ADR review process and validation of existing ADRs (001-006)
- README.md with project overview and setup instructions
- Foundation for all subsequent epics

### Story 1.1: Initialize Cargo Workspace Structure

As a developer setting up the project foundation,
I want to create the Cargo workspace with 4 hexagonal architecture crates,
So that the project has clear separation between domain, application, infrastructure, and CLI layers.

**Acceptance Criteria:**

**Given** a new Rust project directory
**When** I run the workspace initialization commands
**Then** a Cargo workspace is created with the following structure:
```
lithos/
├── Cargo.toml (workspace configuration)
├── crates/
│   ├── domain/ (pure business logic, no I/O)
│   ├── app/ (application services and orchestration)
│   ├── adapters/ (infrastructure implementations)
│   └── cli/ (binary entry point)
└── Cargo.lock
```

**Given** the Cargo workspace structure exists
**When** I check the crate dependencies
**Then** the dependencies follow hexagonal boundaries:
- domain crate has no external dependencies
- app crate depends only on domain
- adapters crate depends on domain + external crates
- cli crate depends on app + adapters

**Given** the workspace is initialized
**When** I run `cargo check`
**Then** all crates compile without errors

### Story 1.2: Configure mise.toml for Task Orchestration

As a developer working on the project,
I want comprehensive mise tasks for development workflows,
So that I can efficiently run tests, benchmarks, formatting, and other development tasks.

**Acceptance Criteria:**

**Given** mise is installed in the project
**When** I run `mise run --list` or check mise.toml
**Then** the following tasks are available:
- `mise run test` - Run all tests
- `mise run test:unit` - Domain layer unit tests only
- `mise run test:integration` - Cross-crate integration tests
- `mise run test:coverage` - Generate coverage report with tarpaulin
- `mise run test:watch` - Watch mode for TDD development
- `mise run bench` - Run performance benchmarks
- `mise run fmt` - Format all code
- `mise run lint` - Run clippy linting
- `mise run verify` - Full quality gate (fmt + lint + test)

**Given** I have researched Rust project best practices for task orchestration
**When** I review the mise.toml configuration
**Then** tasks follow these best practices:
- Tool versions are pinned (Rust 1.92+, clippy, rustfmt versions)
- Tasks use proper shell escaping for cross-platform compatibility
- Tasks include helpful descriptions and usage examples
- Tasks integrate with pre-commit hooks where appropriate

**Given** mise tasks are configured
**When** I run `mise run verify`
**Then** the full quality pipeline executes successfully

### Story 1.3: Set Up Stringent Pre-Commit Hooks

As a developer committing code,
I want automatic quality checks before every commit,
So that code quality standards are enforced and poor code is caught early.

**Acceptance Criteria:**

**Given** pre-commit framework is configured
**When** I check .pre-commit-config.yaml
**Then** the hooks include these stringent quality gates:
- `clippy` with all configured lints
- `rustfmt` with import sorting verification
- `cargo test` for unit tests
- `cargo deny check` for dependency security

**Given** I have researched pre-commit best practices for Rust projects
**When** I review the configuration
**Then** the hooks follow these best practices:
- Hooks run in parallel where possible for speed
- Hooks fail fast on critical issues
- Hooks include clear error messages for failures
- Hooks respect .gitignore patterns

**Given** pre-commit hooks are installed
**When** I attempt to commit code that violates quality standards
**Then** the commit is blocked with clear error messages

**Given** pre-commit hooks are installed
**When** I commit properly formatted, tested code
**Then** the commit succeeds without delays

### Story 1.4: Configure clippy.toml with Cognitive Complexity Limits

As a developer writing code,
I want clippy to enforce cognitive complexity limits as a quality safeguard,
So that functions remain maintainable and complex logic is broken down appropriately.

**Acceptance Criteria:**

**Given** I have researched clippy best practices for cognitive complexity
**When** I review clippy.toml configuration
**Then** cognitive complexity limits are set to:
- `cognitive-complexity-threshold = 15` (warn level)
- `too-many-lines-threshold = 100` (function length limit)
- Deny level complexity threshold configured

**Given** I have researched anti-pattern prevention in Rust
**When** I check the clippy configuration
**Then** these anti-patterns are denied:
- `clippy::unwrap_used` - No unwrap in production code
- `clippy::expect_used` - No expect in production code
- `clippy::todo` - No TODO comments in production code
- `clippy::unimplemented` - No unimplemented!() in production code
- `clippy::dbg_macro` - No debug prints in production code

**Given** clippy.toml is configured with stringent rules
**When** I run `cargo clippy`
**Then** code exceeding complexity limits generates warnings/errors

**Given** code exceeds cognitive complexity limits
**When** I run clippy
**Then** specific line numbers and suggestions are provided for refactoring

### Story 1.5: Configure rustfmt.toml with Import Sorting

As a developer formatting code,
I want consistent import sorting and formatting standards,
So that code style is uniform and readable across the codebase.

**Acceptance Criteria:**

**Given** I have researched rustfmt best practices for large Rust projects
**When** I review rustfmt.toml configuration
**Then** import sorting is configured with:
- `imports_granularity = "Crate"` - Group imports by crate
- `group_imports = "StdExternalCrate"` - Standard library, external crates, then internal
- Consistent indentation and line width settings

**Given** rustfmt.toml is configured
**When** I run `cargo fmt`
**Then** all imports are sorted consistently across the codebase

**Given** code with unsorted imports
**When** I run `cargo fmt --check`
**Then** the command fails with specific file locations needing formatting

**Given** I have researched formatting standards for Rust ecosystems
**When** I check the configuration
**Then** settings align with Rust community standards for:
- Maximum line width (typically 100-120 characters)
- Brace style consistency
- Comment formatting
- Macro formatting

### Story 1.6: Set Up deny.toml for Dependency Security Auditing

As a developer managing dependencies,
I want automatic security and license auditing of dependencies,
So that vulnerabilities and incompatible licenses are caught before they become issues.

**Acceptance Criteria:**

**Given** I have researched cargo-deny best practices for Rust projects
**When** I review deny.toml configuration
**Then** the following checks are enabled:
- `advisories` - Security vulnerability scanning
- `licenses` - License compatibility checking
- `bans` - Forbidden dependency detection
- `sources` - Source verification

**Given** deny.toml is configured
**When** I run `cargo deny check`
**Then** all dependency checks pass without security issues

**Given** a dependency with security vulnerabilities exists
**When** I run `cargo deny check advisories`
**Then** specific CVEs and affected dependencies are reported

**Given** I have researched license compatibility for open source projects
**When** I check the license configuration
**Then** acceptable licenses include common permissive licenses:
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause
- GPL licenses excluded for compatibility
- Copyleft licenses flagged for review

### Story 1.7: Establish ADR Review Process and Validate Existing ADRs

As a developer making architectural decisions,
I want a clear process for documenting and reviewing ADRs,
So that architectural decisions are well-reasoned, documented, and validated.

**Acceptance Criteria:**

**Given** the ADR directory exists with documents 001-006
**When** I review the ADR review process
**Then** a clear process is documented for:
- When to create an ADR (architectural decisions affecting multiple epics)
- ADR template and required sections
- Review and approval process
- How ADRs relate to implementation

**Given** ADRs 001-006 exist
**When** I validate them against the established template
**Then** all ADRs follow the proper format:
- Status (Accepted/Rejected/Pending)
- Context and problem description
- Considered alternatives
- Decision with rationale
- Consequences and trade-offs

**Given** the ADR review process is established
**When** a new architectural decision is needed
**Then** the process guides creation of properly formatted ADRs

**Given** I have researched ADR best practices
**When** I review the process
**Then** it follows industry standards:
- MADR (Markdown Architectural Decision Records) format
- Clear decision drivers and constraints
- Stakeholder involvement in decisions
- Regular review and update process

### Story 1.8: Create Comprehensive README.md

As a developer or user discovering the project,
I want a clear overview of the project with setup instructions,
So that I can quickly understand what lithos is and how to get started.

**Acceptance Criteria:**

**Given** README.md is created
**When** I review the content structure
**Then** it includes these essential sections:
- Project description and what makes it special
- Key features and capabilities
- Quick start installation instructions
- Basic usage examples
- Architecture overview (hexagonal, CQRS, etc.)
- Development setup for contributors
- Links to detailed documentation

**Given** I have researched README best practices for open source Rust projects
**When** I check the README format
**Then** it follows best practices:
- Clear badges for CI status, coverage, version
- Table of contents for long documents
- Code examples that are tested and runnable
- Links to CONTRIBUTING.md and CODE_OF_CONDUCT.md
- Performance benchmarks and compatibility matrix

**Given** README.md exists
**When** a new user visits the repository
**Then** they can understand the project purpose within 2 minutes

**Given** README.md exists
**When** a developer wants to contribute
**Then** setup instructions are clear and comprehensive

## Epic 2: Test Architecture, Patterns & Utilities
Developers have comprehensive testing patterns for async code, event-driven systems, and CQRS, plus centralized test utilities for artifacts, temporary directories, and mise task orchestration that ensure 80%+ coverage and catch integration issues early.
**FRs covered:** NFR16 (test coverage), Architecture requirements (testing strategy)
**Implementation Notes:**
- Test patterns: async (tokio::test), event-driven, CQRS command/query separation
- Centralized test utilities: artifact output locations, tmp directory management, test helper functions
- mise.toml test tasks: test, test:unit, test:integration, test:coverage, test:watch, test:benchmark
- ADR creation guidelines for epics making architectural decisions

### Story 2.1: Establish Async Testing Patterns and Infrastructure

As a developer testing async code,
I want standardized patterns for testing tokio-based async operations,
So that async tests are reliable, race-condition free, and properly isolated.

**Acceptance Criteria:**

**Given** I have researched async testing best practices in Rust
**When** I review the async testing infrastructure
**Then** standardized patterns are established for:
- `#[tokio::test]` macro usage with proper runtime setup
- `spawn_blocking` for CPU-intensive operations in tests
- `CancellationToken` for graceful test shutdown
- Race condition detection and prevention

**Given** async testing patterns are established
**When** I write an async unit test
**Then** the test follows the established patterns:
- Proper `#[tokio::test]` attribute usage
- No blocking operations without `spawn_blocking`
- Proper error handling for async operations
- Test isolation without shared state

**Given** async tests are running
**When** I check for race conditions
**Then** tests use proper synchronization primitives and avoid flaky behavior

**Given** I have researched tokio testing ecosystem
**When** I check test dependencies
**Then** optimal crates are selected:
- `tokio::test` for basic async testing
- `tokio::time::timeout` for preventing hanging tests
- Proper test runtime configuration

### Story 2.2: Create Event-Driven Testing Patterns

As a developer testing event-driven systems,
I want patterns for testing domain events and event bus interactions,
So that event-driven code is thoroughly tested with proper isolation and verification.

**Acceptance Criteria:**

**Given** I have researched event-driven testing patterns
**When** I review the event testing infrastructure
**Then** patterns are established for:
- Event publishing and subscription testing
- Event payload verification
- Event ordering and timing verification
- Mock event bus implementations for unit tests

**Given** event-driven testing patterns are established
**When** I test an event publisher
**Then** the test verifies:
- Correct events are published
- Event payloads contain expected data
- Events are published at the correct time
- Error handling for failed event publishing

**Given** event-driven testing patterns are established
**When** I test an event subscriber
**Then** the test verifies:
- Subscriber receives expected events
- Event handling logic executes correctly
- Subscriber handles malformed events gracefully
- Subscription lifecycle management

**Given** I have researched event testing in domain-driven design
**When** I check the patterns
**Then** they follow DDD testing best practices:
- Event sourcing verification patterns
- Event storming validation
- Domain event contract testing
- Integration testing for event flows

### Story 2.3: Establish CQRS Testing Patterns

As a developer testing CQRS command and query separation,
I want patterns for testing write operations and read models separately,
So that command side and query side code are tested in isolation with proper verification.

**Acceptance Criteria:**

**Given** I have researched CQRS testing patterns
**When** I review the CQRS testing infrastructure
**Then** patterns are established for:
- Command handler testing (write side)
- Query handler testing (read side)
- Command/query separation verification
- Eventual consistency testing between write and read models

**Given** CQRS testing patterns are established
**When** I test a command handler
**Then** the test verifies:
- Command validation logic
- State changes are applied correctly
- Domain events are published
- Error cases are handled appropriately

**Given** CQRS testing patterns are established
**When** I test a query handler
**Then** the test verifies:
- Query execution returns correct data
- Query performance meets requirements
- Query isolation from write operations
- Caching behavior if applicable

**Given** I have researched CQRS testing in Rust ecosystems
**When** I check the implementation
**Then** it addresses common CQRS testing challenges:
- Testing eventual consistency
- Mocking read model updates
- Verifying command/query separation
- Testing cross-aggregate consistency

### Story 2.4: Create Centralized Test Utilities and Infrastructure

As a developer writing tests across the codebase,
I want centralized test utilities for common testing needs,
So that tests are consistent, maintainable, and don't duplicate utility code.

**Acceptance Criteria:**

**Given** I have researched test utility patterns in large Rust projects
**When** I review the centralized test utilities
**Then** utilities are provided for:
- Temporary directory creation and cleanup
- Test artifact output management
- Test data fixtures and factories
- Common assertion helpers

**Given** centralized test utilities exist
**When** I write a test needing temporary files
**Then** I can use standardized temporary directory utilities:
- Automatic cleanup after test completion
- Cross-platform path handling
- Unique directory names to avoid conflicts
- Proper error handling for directory operations

**Given** centralized test utilities exist
**When** I write a test needing test data
**Then** I can use standardized fixture utilities:
- Domain object factories with valid defaults
- Sample data generation for various scenarios
- Serialization helpers for complex objects
- Reusable test data across multiple tests

**Given** I have researched test isolation best practices
**When** I check the utilities
**Then** they ensure proper test isolation:
- No shared state between tests
- Proper cleanup of resources
- Database/transaction isolation for integration tests
- Process isolation for system tests

### Story 2.5: Configure Mise Test Task Orchestration

As a developer running tests during development,
I want comprehensive mise tasks for different testing scenarios,
So that I can efficiently run tests, check coverage, and maintain code quality during development.

**Acceptance Criteria:**

**Given** I have researched mise task orchestration for Rust projects
**When** I review the mise.toml test tasks
**Then** comprehensive test tasks are configured:
- `mise run test` - Run all tests with optimal parallelization
- `mise run test:unit` - Domain layer unit tests only
- `mise run test:integration` - Cross-crate integration tests
- `mise run test:coverage` - Generate coverage report (tarpaulin)
- `mise run test:watch` - Watch mode for TDD workflow

**Given** mise test tasks are configured
**When** I run `mise run test`
**Then** tests execute with:
- Optimal parallelization for speed
- Proper output formatting
- Clear success/failure indication
- Timing information for slow tests

**Given** mise test tasks are configured
**When** I run `mise run test:coverage`
**Then** coverage report is generated:
- HTML report for browser viewing
- Coverage percentage calculation
- File-by-file coverage breakdown
- Integration with CI/CD pipelines

**Given** I have researched continuous testing workflows
**When** I check the mise configuration
**Then** tasks support modern development workflows:
- Watch mode for automatic test re-running
- Fast feedback for TDD cycles
- Integration with IDEs and editors
- Remote development environment support

### Epic 3: Core Domain Models & Value Objects
Developers have a clear, shared domain language with rich domain models that embody business rules and validation logic, informed by Obsidian patterns and Go implementation lessons learned.
**FRs covered:** Architecture requirements (DDD domain models)
**Implementation Notes:**
- Core stable models: Config, Schema, Note, Frontmatter, Template + value objects
- Models informed by Obsidian structures (TFile, CachedMetadata) and Go implementation
- Flexibility for Rust-specific refinements and supplementary models in later epics
- Mocks for domain interfaces created as needed (not upfront)

### Story 3.1: Create Note Bounded Context

As a developer working with note data,
I want a comprehensive Note aggregate with all subentities,
So that the domain accurately represents the rich structure of notes in Obsidian vaults.

**Acceptance Criteria:**

**Given** I have researched Obsidian note structures and wiki-link patterns
**When** I review the Note bounded context
**Then** the Note aggregate includes these subentities:
- Note (main entity with identity and metadata)
- Frontmatter (YAML metadata with fields)
- Links (wiki-links, aliases, and references)
- Embeds (embedded content references)
- Tags (hierarchical tag system)
- Headings (document structure)
- Tasks (task management with status)
- Sections (content organization)

**Given** the Note aggregate is defined
**When** I validate entity relationships
**Then** Frontmatter is a subentity of Note (Note contains Frontmatter)

**Given** semantic validation is integrated
**When** I create a Note instance
**Then** internal consistency validation occurs (semantic validation per entity)

**Given** I have researched Obsidian vault patterns
**When** I check the Note entity design
**Then** it supports vault-relative paths and wiki-link resolution

### Story 3.2: Create Schema Bounded Context

As a developer defining metadata schemas,
I want a complete schema domain with PropertyBank, Property, and PropertySpec variants,
So that schemas can define reusable property definitions with rich validation constraints.

**Acceptance Criteria:**

**Given** I have researched schema domain patterns for metadata validation systems
**When** I review the Schema bounded context
**Then** it includes these domain models:
- Schema entity (Name, Extends, Excludes, Properties[], ResolvedProperties[])
- PropertyBank entity (singleton registry of reusable Property definitions)
- Property entity (ID, Name, Required, Array, Spec)
- PropertySpec trait with variants: StringSpec, NumberSpec, BoolSpec, DateSpec, FileSpec

**Given** the Schema entity is defined
**When** I check inheritance capabilities
**Then** Schema supports Extends (parent schema) and Excludes (properties to remove)

**Given** PropertyBank is defined
**When** I validate its design
**Then** it provides singleton registry with Lookup method and reference support

**Given** Property entity is defined
**When** I check identity generation
**Then** ID is deterministically generated from hash of Name + Spec content

**Given** PropertySpec variants are defined
**When** I review type-specific constraints
**Then** each variant supports appropriate validation:
- StringSpec: enum values and regex patterns
- NumberSpec: min/max/step constraints
- BoolSpec: marker type (no constraints)
- DateSpec: format strings
- FileSpec: fileClass and directory restrictions

**Given** semantic validation is integrated
**When** I create Schema instances
**Then** internal consistency validation occurs for all entities

### Story 3.3: Create Config Bounded Context

As a developer managing application configuration,
I want a Config domain model with validation,
So that configuration changes are validated and the domain enforces configuration integrity.

**Acceptance Criteria:**

**Given** I have researched hierarchical configuration patterns
**When** I review the Config bounded context
**Then** Config entity supports hierarchical structure (Global → User → Project → Vault)

**Given** Config entity is defined
**When** I check validation integration
**Then** semantic validation ensures configuration integrity and type safety

**Given** configuration patterns are established
**When** I validate the design
**Then** Config supports encrypted sensitive fields and validation rules

### Story 3.4: Create Template Bounded Context

As a developer working with template definitions,
I want a Template domain model with validation,
So that template structure and syntax are properly validated at the domain level.

**Acceptance Criteria:**

**Given** I have researched template engine patterns
**When** I review the Template bounded context
**Then** Template entity includes structure and syntax validation

**Given** Template entity is defined
**When** I check semantic validation
**Then** template syntax and structure validation occurs internally

**Given** template patterns are established
**When** I validate the design
**Then** Template supports modular composition and variable definitions

### Story 3.5: Review Epic 3 Test Suite for Efficiency

As a developer maintaining the codebase,
I want an efficient test suite for Epic 3 domain models,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 3 domain models are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for domain entities and validation logic

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across domain models

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 3 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

**Given** domain models evolve
**When** I update tests
**Then** test maintenance cost is <20% of development time

### Story 3.6: Create Epic 3 Documentation

As a developer working with the domain models,
I want comprehensive documentation of the domain entities, their relationships, and evolution guidelines,
So that developers understand the domain language and can work effectively with the models.

**Acceptance Criteria:**

**Given** all Epic 3 domain models are implemented
**When** I create documentation
**Then** it includes developer-focused content:
- Domain entity relationships and bounded contexts
- Semantic validation rules for each entity
- Domain entity relationship contracts (how bounded contexts interact)
- Evolution guidelines for domain models (when to add vs modify entities)
- Architecture diagrams showing entity relationships and contracts

**Given** documentation is created
**When** I validate completeness
**Then** it covers all entities and their contracts: Note aggregate, Schema domain, Config, Template

**Given** documentation exists
**When** I check relationship contracts
**Then** it defines how bounded contexts interact (e.g., Template references Schema, Note uses Config)

**Given** documentation exists
**When** a developer reads it
**Then** they understand domain evolution rules and inter-entity contracts without needing user-facing knowledge

### Epic 4: File Loading Strategy Foundation
System has unified file loading strategies for different configuration formats that enable consistent parsing and validation across the application.
**FRs covered:** Architecture requirements (file loading infrastructure)
**Implementation Notes:**
- Unified loading strategy for TOML, JSON, YAML files
- File format detection and parsing
- Basic validation infrastructure
- Enables both configuration (Epic 5) and schema (Epic 6) loading

### Story 4.1: Create Unified File Loading Interface

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

### Story 4.2: Implement Format Detection and Parsing

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

### Story 4.3: Add Basic File Loading Validation

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

### Story 4.4: Create Loading Strategy Mocks for Testing

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

### Story 4.5: Review Epic 4 Test Suite

As a developer maintaining the file loading foundation,
I want an efficient test suite for Epic 4 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 4 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for loading strategy components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across loading components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 4 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Epic 5: Configuration Management System
Users can configure lithos through hierarchical TOML files with validation, supporting template packs and schema definitions.
**FRs covered:** FR26, FR27, FR28
**Implementation Notes:**
- Figment-based hierarchical config per ADR 004 using Epic 4 loading foundation
- ConfigPort and mocks created in this epic
- Sample config files based on JSON schema (lithos-specific)
- User documentation for configuration

### Story 5.1: Create Config Domain Interface and Port

As a developer implementing configuration management,
I want a clean domain interface for configuration loading,
So that configuration can be loaded through a well-defined contract following hexagonal architecture.

**Acceptance Criteria:**

**Given** I need to define configuration contracts
**When** I create the Config domain interface
**Then** it includes ConfigPort trait with async load method

**Given** ConfigPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated unit testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with clear separation between domain and infrastructure

### Story 4.2: Implement Hierarchical Configuration Loading

As a user configuring lithos,
I want hierarchical configuration that respects precedence rules,
So that I can override settings at different levels (global, user, project, vault).

**Acceptance Criteria:**

**Given** I need hierarchical config loading
**When** I implement with Figment per ADR 004
**Then** configuration loads with proper precedence: CLI > Environment > Config files > Defaults

**Given** hierarchical loading is implemented
**When** I test precedence
**Then** vault-level config overrides project-level, project overrides user-level, etc.

**Given** configuration files are loaded
**When** I validate TOML parsing
**Then** complex nested structures are properly deserialized

### Story 4.3: Add Configuration Validation and Error Handling

As a user providing configuration,
I want clear validation and helpful error messages,
So that I can identify and fix configuration issues quickly.

**Acceptance Criteria:**

**Given** configuration is loaded
**When** I validate config structure
**Then** semantic validation occurs for required fields and value ranges

**Given** validation fails
**When** I check error messages
**Then** errors are actionable with specific field locations and suggested fixes

**Given** configuration validation is implemented
**When** I test error handling
**Then** partial invalid configs provide clear guidance on what needs to be fixed

### Story 4.4: Implement Configuration Versioning and Migration

As a developer maintaining lithos,
I want configuration versioning and migration support,
So that configuration files can evolve safely across versions without breaking user setups.

**Acceptance Criteria:**

**Given** configuration evolves over time
**When** I implement versioning
**Then** config files include version field for compatibility checking

**Given** version mismatches are detected
**When** I run migration
**Then** automatic migration transforms old config to new format

**Given** breaking changes occur
**When** users upgrade
**Then** clear error messages guide them through manual migration steps

### Story 4.5: Create Sample Configuration Files

As a user getting started with lithos,
I want sample configuration files based on a complete JSON schema,
So that I can understand configuration options and get started quickly with validated configs.

**Acceptance Criteria:**

**Given** I need sample configurations
**When** I create a complete JSON schema for lithos configuration
**Then** the schema defines all possible configuration options with types, defaults, and validation rules

**Given** the JSON schema exists
**When** I create sample config files
**Then** samples are provided in TOML, JSON, and YAML formats showing common configuration patterns

**Given** sample files exist
**When** I validate against the schema
**Then** all samples pass validation and demonstrate all major configuration features

**Given** users have sample configs
**When** they start lithos
**Then** configurations load successfully and demonstrate expected behavior

### Story 4.6: Review Epic 4 Test Suite

As a developer maintaining the configuration system,
I want an efficient test suite for Epic 4 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 4 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for configuration components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across config components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 4 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 4.7: Document Configuration System for Users

As a user configuring lithos,
I want comprehensive documentation for configuration options,
So that I can understand and customize lithos behavior effectively.

**Acceptance Criteria:**

**Given** configuration system is implemented
**When** I create user documentation
**Then** it includes all configuration options with examples and defaults

**Given** documentation exists
**When** I check completeness
**Then** it covers hierarchical loading, validation rules, and troubleshooting

**Given** users read the documentation
**When** they configure lithos
**Then** they can successfully customize behavior without developer assistance

### Epic 6: Schema System & Validation
Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**
- SchemaPort and mocks created in this epic
- Sample schema files created as lithos-specific examples using Epic 4 loading foundation
- Schema validation (syntactic in adapter, semantic in domain)
- User documentation for schema creation

### Epic 7: Event Bus & Orchestration Infrastructure
System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.
**FRs covered:** Architecture requirements (event-driven, CQRS foundation)
**Implementation Notes:**
- Hybrid Event Bus (MPSC/Broadcast/Watch per ADR 006)
- Event payload schema design and validation
- EventBusPort mocks for testing
- Prevents god-object orchestrators (Go lesson learned)
- May create ADR for event patterns if architectural decisions made

### Epic 8: Storage Layer & Persistence
System has zero-copy persistent storage with ACID transactions using Redb + rkyv that supports high-performance queries and maintains data consistency.
**FRs covered:** Architecture requirements (Redb + rkyv storage per ADR 001)
**Implementation Notes:**
- Redb + rkyv per ADR 001 (no SQLite - decision already made)
- Storage schema design review against Epic 9-10 query requirements
- Unit of Work pattern for transactional consistency
- Storage port mocks for testing
- May create ADR for storage schema patterns if needed

### Epic 9: Vault File System Integration & Indexing Engine
Users can index large vaults (1000+ files) in <2 seconds with incremental updates, reliable crash-free operation, and persistent storage.
**FRs covered:** FR20, FR24, FR25
**Implementation Notes:**
- VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort and mocks created
- pulldown-cmark for markdown parsing (adapter layer per ADR 003)
- Sample vault notes from docs/refs/obsidian/ as test fixtures
- Performance benchmarking stories for NFR2 validation (<2s for 1000+ files)
- Observability/metrics for indexing performance

### Epic 10: Query Service & Knowledge Graph
Users can perform fast lookups by filename, path, or schema keys, resolve wiki-links and aliases, and query metadata from other notes for template use.
**FRs covered:** FR21, FR22, FR23
**Implementation Notes:**
- QueryPort and mocks created in this epic
- CQRS read side (Epic 9 is write side)
- Performance benchmarking stories for NFR1 validation (<500ms queries)
- Observability/metrics for query performance

### Epic 11: Basic Interactive Template System
Users can create and execute modular templates with schema-driven interactive prompts that generate validated notes with essential template functions.
**FRs covered:** FR1, FR2, FR9, FR15, FR16
**Implementation Notes:**
- TemplatePort, UIPort and mocks created in this epic
- MiniJinja integration per ADR 002
- Sample templates from docs/refs/obsidian/ converted as test fixtures
- Schema-driven inputs (enums → suggesters)
- User documentation for basic template creation
- Performance benchmarking for NFR1 validation (<500ms execution)
- May create ADR for interactive UI patterns

### Epic 12: Advanced Template Features
Users can compose complex templates with date functions, multi-suggesters, and error prevention for production-ready template workflows.
**FRs covered:** FR3, FR4, FR17
**Implementation Notes:**
- Extends Epic 10 template system (not replacement)
- Date formatting with chrono (Rust-native)
- Template composition patterns
- User documentation for advanced template features
- Performance validation for complex templates

### Epic 13: CLI Interface & Error Handling
Users can execute lithos commands with intuitive CLI, comprehensive help, single-word shortcuts, and actionable error diagnostics.
**FRs covered:** FR41, FR42, FR43, FR44, FR45, FR46, FR47, FR48, FR49, FR50, FR30, FR31
**Implementation Notes:**
- Clap for CLI, miette for diagnostics per ADR 005
- CommandPort, AuditPort created if needed
- Dependency injection wiring for all ports
- Cross-platform support (macOS primary, Linux)
- Observability/audit logging for FR40 (basic version)
- User CLI documentation

### Epic 14: Test Suite Review & Optimization
Development team has a validated, efficient test suite with no redundancy, full coverage of critical paths, and effective system validation.
**FRs covered:** NFR16 (comprehensive test coverage), NFR25 (zero crashes)
**Implementation Notes:**
- Final holistic review after all epic-level tests complete
- Identify overlapping/redundant tests across Epics 4-12
- Validate test suite efficiency and execution time
- Ensure 80%+ coverage without bloat
- Architectural boundary validation (hexagonal, CQRS, event-driven)
- Note: Each epic 4-12 has its own test validation story; this is final optimization

### Epic 15: User Documentation & Onboarding
Users have comprehensive documentation, starter templates, sample schemas, and migration guides that enable successful adoption.
**FRs covered:** NFR13 (clear help), NFR20 (migration paths), NFR28 (installation success)
**Implementation Notes:**
- Consolidates documentation from Epics 4-12
- Starter kit from converted docs/refs/obsidian/ samples (sanitized)
- Installation guide, quickstart, migration guides
- API documentation for power users
- Progressive complexity documentation (basic → advanced)
- Note: Documentation created at story-level in epics; this consolidates and polishes
