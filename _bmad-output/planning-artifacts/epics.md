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

### Story 5.2: Implement Hierarchical Configuration Loading

As a user configuring lithos,
I want hierarchical configuration that respects precedence rules,
So that I can override settings at different levels (global, user, project, vault).

**Acceptance Criteria:**

**Given** Epic 4 provides unified file loading for TOML, JSON, YAML
**When** I implement hierarchical config using Figment per ADR 004
**Then** configuration loads with proper precedence: CLI > Environment > Config files > Defaults

**Given** hierarchical loading is implemented
**When** I test precedence
**Then** vault-level config overrides project-level, project overrides user-level, etc.

**Given** configuration files are loaded using Epic 4 infrastructure
**When** I validate TOML parsing
**Then** complex nested structures are properly deserialized through Epic 4's format detection

### Story 5.3: Add Configuration Validation and Error Handling

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

### Story 5.4: Implement Configuration Versioning and Migration

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

### Story 5.5: Create Sample Configuration Files

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
**When** they start lithos using Epic 4's file loading
**Then** configurations load successfully and demonstrate expected behavior

### Story 5.6: Review Epic 5 Test Suite

As a developer maintaining the configuration system,
I want an efficient test suite for Epic 5 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 5 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for configuration components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across configuration components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 5 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 5.7: Document Configuration System for Users

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
- Sample schema files created from docs/schemas/ JSON examples using Epic 4 loading foundation
- Schema validation (syntactic in adapter, semantic in domain)
- Schema-template integration contracts defined
- User documentation for schema creation

### Story 6.1: Create Schema Domain Interface and Port

As a developer implementing schema management,
I want a clean domain interface for schema operations,
So that schemas can be loaded and validated through a well-defined contract following hexagonal architecture.

**Acceptance Criteria:**

**Given** I need to define schema contracts
**When** I create the Schema domain interface
**Then** it includes SchemaPort trait with load and validate methods

**Given** SchemaPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated unit testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with clear separation between domain and infrastructure

### Story 6.2: Create Schema Property System

As a developer defining schema properties,
I want a complete property system with PropertyBank, Property, and PropertySpec variants,
So that schemas can define reusable property definitions with rich validation constraints.

**Acceptance Criteria:**

**Given** I need property definitions for schemas
**When** I create the property system based on docs/schemas/property_bank.json
**Then** PropertyBank provides singleton registry with Lookup method and $ref support

**Given** PropertyBank is implemented
**When** I define Property entities
**Then** each Property has ID (deterministic hash), Name, Required, Array, and Spec

**Given** PropertySpec variants are needed
**When** I implement type-specific specs
**Then** StringSpec, NumberSpec, BoolSpec, DateSpec, FileSpec provide appropriate validation

**Given** the property system is complete
**When** I validate against docs/schemas/ examples
**Then** all property types from the JSON schemas are supported

### Story 6.3: Implement Schema Loading with $ref Resolution

As a developer loading schema files,
I want schema loading with proper $ref resolution,
So that schemas can reference shared properties from the PropertyBank.

**Acceptance Criteria:**

**Given** Epic 4 provides file loading infrastructure
**When** I implement schema loading using Epic 4 for JSON parsing
**Then** schemas are loaded from JSON files in docs/schemas/

**Given** schemas contain $ref pointers
**When** I resolve references using PropertyBank from Story 6.2
**Then** $ref pointers are replaced with actual Property definitions

**Given** schema loading is implemented
**When** I load complex schemas like docs/schemas/pkm.json
**Then** all $ref resolutions work correctly and schemas are fully expanded

### Story 6.4: Implement Schema Inheritance Resolution

As a developer working with schema hierarchies,
I want inheritance resolution for schema chains,
So that child schemas can extend and modify parent schemas.

**Acceptance Criteria:**

**Given** schemas have Extends relationships
**When** I implement inheritance resolution
**Then** parent schemas are loaded and child properties are merged

**Given** schemas have Excludes lists
**When** I process inheritance
**Then** excluded parent properties are removed from the resolved schema

**Given** complex inheritance chains exist
**When** I resolve docs/schemas/ inheritance examples
**Then** multi-level inheritance works (e.g., task_child extends task extends base)

### Story 6.5: Add Schema Validation and Error Handling

As a developer validating schemas,
I want comprehensive schema validation with clear error messages,
So that invalid schemas are caught early with actionable feedback.

**Acceptance Criteria:**

**Given** schemas are loaded and resolved
**When** I validate schema structure
**Then** syntactic validation catches malformed JSON and missing required fields

**Given** schemas are validated
**When** I check semantic rules
**Then** inheritance chains are valid and property references exist

**Given** validation fails
**When** I provide error messages
**Then** errors include schema file path, line numbers, and suggested fixes

### Story 6.6: Create Sample Schema Files

As a user creating schemas,
I want comprehensive sample schemas demonstrating all features,
So that I can understand schema capabilities and use them as templates.

**Acceptance Criteria:**

**Given** docs/schemas/ contains JSON schema examples
**When** I create sample schemas for lithos
**Then** samples demonstrate all property types (string, number, bool, date, file)

**Given** samples are created
**When** I test inheritance
**Then** samples show Extends/Excludes patterns from docs/schemas/ examples

**Given** samples are created
**When** I validate them
**Then** all samples pass validation and demonstrate schema capabilities

### Story 6.7: Define Schema-Template Integration Contracts

As a developer integrating schemas with templates,
I want clear contracts for how schemas provide inputs to templates,
So that templates can safely access schema-defined properties.

**Acceptance Criteria:**

**Given** schemas define properties
**When** I define integration contracts
**Then** templates can access property values by schema name and property name

**Given** integration contracts exist
**When** templates reference schema properties
**Then** type-safe access is provided with validation

**Given** contracts are defined
**When** I validate against Epic 11 template requirements
**Then** all template input needs are satisfied by schema contracts

### Story 6.8: Review Epic 6 Test Suite

As a developer maintaining the schema system,
I want an efficient test suite for Epic 6 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 6 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for schema components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across schema components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 6 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 6.9: Document Schema System for Users

As a user creating schemas,
I want comprehensive documentation for schema creation and usage,
So that I can effectively define and use schemas in lithos.

**Acceptance Criteria:**

**Given** schema system is implemented
**When** I create user documentation
**Then** it includes all schema features: properties, inheritance, validation, examples

**Given** documentation exists
**When** I check completeness
**Then** it covers all property types and inheritance patterns from docs/schemas/

**Given** users read the documentation
**When** they create schemas
**Then** they can define valid schemas without developer assistance

### Epic 7: Event Bus & Orchestration Infrastructure
System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.
**FRs covered:** Architecture requirements (event-driven, CQRS foundation)
**Implementation Notes:**
- Hybrid Event Bus (MPSC/Broadcast/Watch per ADR 006)
- Event payload schema design and validation
- Event persistence for debugging and recovery
- EventBusPort mocks for testing
- Integration contracts for other epics
- Prevents god-object orchestrators (Go lesson learned)
- May create ADR for event patterns if architectural decisions made

### Story 7.1: Create Event Bus Domain Interface and Port

As a developer implementing event-driven architecture,
I want a clean domain interface for event operations,
So that events can be published and subscribed to through well-defined contracts.

**Acceptance Criteria:**

**Given** I need event bus contracts
**When** I create the EventBusPort trait
**Then** it includes publish and subscribe methods for domain events

**Given** EventBusPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated event testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with async event handling

### Story 7.2: Define Complete Domain Event Types

As a developer coordinating events across the system,
I want complete domain event definitions,
So that all events from Epics 3-6 are properly defined and coordinated.

**Acceptance Criteria:**

**Given** events are defined in Epics 3, 4, 5, 6
**When** I consolidate all domain events
**Then** complete event type definitions exist with consistent naming and payloads

**Given** event types are defined
**When** I validate consistency
**Then** all events follow EventBusPort contracts and have proper serialization

**Given** events are consolidated
**When** I check for completeness
**Then** all system events are defined (ConfigurationLoaded, SchemaLoaded, NoteIndexed, etc.)

### Story 7.3: Implement MPSC Data Plane

As a developer needing reliable event delivery,
I want MPSC data plane for indexing operations,
So that events are delivered reliably without loss in the indexing pipeline.

**Acceptance Criteria:**

**Given** I need reliable event delivery
**When** I implement MPSC data plane per ADR 006
**Then** actor-based mailbox pattern handles indexing events

**Given** MPSC data plane is implemented
**When** I test event delivery
**Then** events are processed in order without loss

**Given** high event volume occurs
**When** I monitor performance
**Then** bounded channels prevent memory issues during indexing

### Story 7.4: Implement Broadcast Control Plane

As a developer needing global signaling,
I want broadcast control plane for system signals,
So that shutdown and global notifications work across all components.

**Acceptance Criteria:**

**Given** I need global signaling
**When** I implement broadcast control plane per ADR 006
**Then** shutdown and system-wide notifications are supported

**Given** broadcast control plane is implemented
**When** I send global signals
**Then** all subscribers receive notifications reliably

**Given** system shutdown occurs
**When** I broadcast shutdown signal
**Then** graceful shutdown happens across all components

### Story 7.5: Implement Watch State Plane

As a developer needing state synchronization,
I want watch state plane for LSP integration,
So that real-time state changes are communicated to IDE integrations.

**Acceptance Criteria:**

**Given** I need state synchronization
**When** I implement watch state plane per ADR 006
**Then** LSP clients receive real-time vault state updates

**Given** watch state plane is implemented
**When** I monitor state changes
**Then** subscribers get immediate notifications of state updates

**Given** LSP integration is active
**When** I change vault state
**Then** watch notifications enable sub-50ms IDE responsiveness

### Story 7.6: Implement Event Publishing and Subscription

As a developer using the event system,
I want complete publish/subscribe functionality,
So that components can publish events and subscribe to relevant notifications.

**Acceptance Criteria:**

**Given** event planes are implemented
**When** I add publishing functionality
**Then** components can publish events to appropriate planes

**Given** publishing is implemented
**When** I add subscription functionality
**Then** components can subscribe to event types they need

**Given** publish/subscribe is complete
**When** I test end-to-end
**Then** events flow from publishers to subscribers correctly

### Story 7.7: Add Event Payload Validation and Error Handling

As a developer ensuring event integrity,
I want event payload validation and error handling,
So that malformed events are caught and handled gracefully.

**Acceptance Criteria:**

**Given** events are published
**When** I validate payloads
**Then** event structure and required fields are checked

**Given** validation fails
**When** I handle errors
**Then** clear error messages are logged without crashing the system

**Given** malformed events occur
**When** I process them
**Then** system continues operating with degraded functionality for bad events

### Story 7.8: Implement Event Persistence for Debugging

As a developer debugging event flows,
I want event persistence capabilities,
So that event history can be inspected for troubleshooting and system analysis.

**Acceptance Criteria:**

**Given** I need event debugging
**When** I implement event persistence
**Then** recent events are stored for inspection

**Given** event persistence is implemented
**When** I debug issues
**Then** event sequences can be replayed and analyzed

**Given** persistence is active
**When** I check performance impact
**Then** persistence adds minimal overhead to normal operations

### Story 7.9: Define Event Bus Integration Contracts

As a developer integrating with the event system,
I want clear integration contracts,
So that other epics know how to publish and subscribe to events.

**Acceptance Criteria:**

**Given** event system is implemented
**When** I define integration contracts
**Then** clear patterns exist for event publishing in each epic

**Given** integration contracts exist
**When** other epics integrate
**Then** consistent subscription patterns are followed

**Given** contracts are defined
**When** I validate system integration
**Then** all epics properly integrate with the event bus

### Story 7.10: Create Event Bus Mocks for Testing

As a developer testing event-driven code,
I want comprehensive event bus mocks,
So that event interactions can be tested in isolation.

**Acceptance Criteria:**

**Given** I need to test event interactions
**When** I create event bus mocks
**Then** mock implementations allow testing different event scenarios

**Given** mocks are available
**When** I write event-driven tests
**Then** tests can verify event publishing and subscription logic

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate real event bus behavior for comprehensive testing

### Story 7.11: Review Epic 7 Test Suite

As a developer maintaining the event system,
I want an efficient test suite for Epic 7 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 7 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for event system components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across event components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 7 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 7.12: Document Event Bus Integration for Developers

As a developer integrating with the event system,
I want comprehensive developer documentation for event bus usage,
So that other epics can properly publish and subscribe to events.

**Acceptance Criteria:**

**Given** event system is implemented
**When** I create developer documentation
**Then** it includes event publishing/subscription patterns and integration contracts

**Given** documentation exists
**When** developers read it
**Then** they understand how to integrate with the event bus in their epics

**Given** integration docs are complete
**When** other epics implement event integration
**Then** they follow consistent patterns without architectural review

### Epic 8: Storage Layer & Persistence
System has zero-copy persistent storage with ACID transactions using Redb + rkyv that supports high-performance queries and maintains data consistency.
**FRs covered:** Architecture requirements (Redb + rkyv storage per ADR 001)
**Implementation Notes:**
- Redb + rkyv per ADR 001 (no SQLite - decision already made)
- Storage schema design review against Epic 9-10 query requirements
- Unit of Work pattern for transactional consistency
- Storage performance benchmarking (NFR2, NFR9 validation)
- Storage backup and corruption recovery (clean slate protocol)
- Storage schema migration and evolution
- Storage port mocks for testing
- May create ADR for storage schema patterns if needed

### Story 8.1: Create Storage Domain Interface and Ports

As a developer implementing data persistence,
I want clean domain interfaces for storage operations,
So that data can be stored and retrieved through well-defined contracts following hexagonal architecture.

**Acceptance Criteria:**

**Given** I need storage contracts
**When** I create storage domain ports
**Then** CacheWriterPort, CacheReaderPort, and VaultWriterPort traits are defined

**Given** storage ports are defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated storage testing

**Given** the domain interfaces exist
**When** I validate the design
**Then** they follow hexagonal principles with clear separation between domain and infrastructure

### Story 8.2: Implement Redb + rkyv Storage Foundation

As a developer needing high-performance persistence,
I want Redb + rkyv implementation with memory bounds,
So that data is stored efficiently with zero-copy deserialization and controlled memory usage.

**Acceptance Criteria:**

**Given** I need persistent storage
**When** I implement Redb + rkyv per ADR 001
**Then** ACID transactions and MVCC concurrency are supported

**Given** rkyv serialization is implemented
**When** I validate zero-copy deserialization
**Then** data is accessed without memory copying for performance

**Given** storage operations run
**When** I monitor memory usage
**Then** operations stay within NFR9 bounds (500MB limit)

### Story 8.3: Add Unit of Work Pattern for Transactions

As a developer ensuring data consistency,
I want Unit of Work pattern for atomic operations,
So that multiple storage operations are committed together or rolled back as a unit.

**Acceptance Criteria:**

**Given** I need transactional consistency
**When** I implement Unit of Work pattern
**Then** TransactionContext manages atomic operations with proper isolation

**Given** Unit of Work is implemented
**When** I handle concurrent operations
**Then** CQRS write/read operations don't deadlock each other

**Given** transactions are used
**When** errors occur mid-transaction
**Then** automatic rollback preserves data consistency

### Story 8.4: Implement Storage Schema Design with Query Requirements

As a developer optimizing data access,
I want storage schema designed for query performance,
So that Epic 9-10 queries can be executed efficiently against the storage layout.

**Acceptance Criteria:**

**Given** Epic 9-10 query requirements are known
**When** I design storage schema
**Then** data layout optimizes for common query patterns (by path, by schema, etc.)

**Given** storage schema is designed
**When** I validate against query needs
**Then** Note lookups, schema filtering, and metadata queries are optimized

**Given** schema design is complete
**When** I benchmark query performance
**Then** operations meet NFR1 requirements (<500ms for queries)

### Story 8.5: Add Storage Validation and Error Handling

As a developer ensuring storage reliability,
I want comprehensive validation and error recovery,
So that storage corruption is detected and recovered gracefully.

**Acceptance Criteria:**

**Given** storage operations occur
**When** I validate data integrity
**Then** corruption is detected before it causes system issues

**Given** corruption is detected
**When** I implement recovery
**Then** clean slate protocol recreates storage from source data

**Given** storage errors occur
**When** I handle them
**Then** clear error messages guide recovery without data loss

### Story 8.6: Implement Storage Backup and Corruption Recovery

As a developer protecting against data loss,
I want backup and recovery mechanisms,
So that storage corruption can be recovered without losing vault data.

**Acceptance Criteria:**

**Given** I need data protection
**When** I implement backup strategy
**Then** periodic backups preserve recent storage state

**Given** corruption occurs
**When** I trigger recovery
**Then** clean slate protocol rebuilds storage from vault files

**Given** backup/recovery is implemented
**When** I test disaster scenarios
**Then** data can be recovered with minimal downtime

### Story 8.7: Implement Storage Schema Migration and Evolution

As a developer updating storage requirements,
I want schema evolution capabilities,
So that storage format can change safely across versions without data loss.

**Acceptance Criteria:**

**Given** storage schema needs changes
**When** I implement migration
**Then** forward/backward compatibility is maintained

**Given** migrations are implemented
**When** I upgrade storage
**Then** existing data is transformed to new schema automatically

**Given** schema evolution is complete
**When** I validate compatibility
**Then** rollbacks are possible if migration fails

### Story 8.8: Implement Storage Performance Benchmarking

As a developer validating performance requirements,
I want comprehensive storage benchmarking,
So that NFR2 (2s vault indexing) and NFR9 (500MB memory) are validated at the storage layer.

**Acceptance Criteria:**

**Given** I need performance validation
**When** I implement benchmarking
**Then** tests run with 1000+ notes to validate NFR2 timing

**Given** benchmarking is implemented
**When** I measure memory usage
**Then** operations stay within NFR9 bounds during peak load

**Given** performance benchmarks run
**When** I analyze results
**Then** storage layer meets all performance requirements before Epic 9-10 integration

### Story 8.9: Create Storage Mocks for Testing

As a developer testing storage-dependent code,
I want comprehensive storage mocks,
So that storage interactions can be tested in isolation without database setup.

**Acceptance Criteria:**

**Given** I need to test storage interactions
**When** I create storage mocks
**Then** mock implementations simulate all storage port behaviors

**Given** mocks are available
**When** I write storage-dependent tests
**Then** tests verify correct storage operations without real database

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate realistic storage behavior for comprehensive testing

### Story 8.10: Review Epic 8 Test Suite

As a developer maintaining the storage system,
I want an efficient test suite for Epic 8 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 8 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for storage components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across storage components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 8 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 8.11: Document Storage System for Developers

As a developer working with data persistence,
I want comprehensive developer documentation for storage operations,
So that storage can be properly used and maintained across the application.

**Acceptance Criteria:**

**Given** storage system is implemented
**When** I create developer documentation
**Then** it includes storage operations, migration procedures, and performance characteristics

**Given** documentation exists
**When** developers read it
**Then** they understand storage operations and maintenance procedures

**Given** storage docs are complete
**When** other epics need storage integration
**Then** they can implement proper storage usage without architectural review

### Epic 9: Vault File System Integration & Indexing Engine
Users can index large vaults (1000+ files) in <2 seconds with incremental updates, reliable crash-free operation, and persistent storage.
**FRs covered:** FR20, FR24, FR25
**Implementation Notes:**
- VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort and mocks created
- pulldown-cmark for markdown parsing (adapter layer per ADR 003)
- Sample vault notes from docs/refs/obsidian/ as test fixtures
- Performance benchmarking stories for NFR2 validation (<2s for 1000+ files)
- Observability/metrics for indexing performance
- Integration with Epic 7 (event publishing) and Epic 8 (storage persistence)

### Story 9.1: Create Vault Domain Interfaces and Ports

As a developer implementing vault operations,
I want clean domain interfaces for vault access,
So that vault operations follow hexagonal architecture principles.

**Acceptance Criteria:**

**Given** I need vault operation contracts
**When** I create vault domain ports
**Then** VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort are defined

**Given** vault ports are defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated vault testing

**Given** the domain interfaces exist
**When** I validate the design
**Then** they follow hexagonal principles with clear separation between domain and infrastructure

### Story 9.2: Implement Vault File System Scanner

As a developer scanning vault directories,
I want efficient filesystem scanning with concurrent access handling,
So that vault files can be discovered and processed safely.

**Acceptance Criteria:**

**Given** I need to scan vault directories
**When** I implement filesystem scanner
**Then** recursive directory traversal discovers all markdown files

**Given** file scanning is implemented
**When** I handle concurrent access
**Then** proper file locking or change detection prevents conflicts

**Given** vault scanning runs
**When** I monitor performance
**Then** large vaults (1000+ files) are scanned efficiently

### Story 9.3: Implement Markdown Parser for Frontmatter Extraction

As a developer parsing vault files,
I want reliable frontmatter extraction from markdown,
So that note metadata can be indexed and queried.

**Acceptance Criteria:**

**Given** I need to parse markdown files
**When** I implement frontmatter extraction using pulldown-cmark
**Then** YAML frontmatter is correctly parsed from markdown files

**Given** markdown parsing is implemented
**When** I handle malformed files
**Then** parsing errors are handled gracefully without crashing

**Given** frontmatter extraction works
**When** I validate completeness
**Then** all standard frontmatter fields are properly extracted

### Story 9.4: Create Vault Indexing Engine with Incremental Updates

As a developer building the indexing system,
I want an indexing engine that supports incremental updates,
So that only changed files are reprocessed for efficiency.

**Acceptance Criteria:**

**Given** I need vault indexing
**When** I create the indexing engine
**Then** it processes vault files and builds searchable index

**Given** indexing engine is implemented
**When** I handle incremental updates
**Then** only modified files are re-indexed based on change detection

**Given** incremental indexing works
**When** I validate efficiency
**Then** large vaults show significant performance improvement over full rebuilds

### Story 9.5: Add Indexing Performance Optimization and Monitoring

As a developer optimizing indexing performance,
I want performance monitoring and optimization for NFR2 compliance,
So that vault indexing completes in <2 seconds for 1000+ files.

**Acceptance Criteria:**

**Given** I need performance optimization
**When** I implement monitoring
**Then** indexing operations are timed and metrics collected

**Given** performance monitoring is active
**When** I optimize bottlenecks
**Then** concurrent processing and memory management improve performance

**Given** optimizations are implemented
**When** I benchmark with 1000+ files
**Then** indexing completes in <2 seconds meeting NFR2

**Given** memory usage is monitored
**When** I validate bounds
**Then** indexing stays within NFR9 500MB memory limit

### Story 9.6: Implement Indexing Error Recovery and Crash Prevention

As a developer ensuring indexing reliability,
I want error recovery and crash prevention mechanisms,
So that indexing failures don't corrupt the system or lose data.

**Acceptance Criteria:**

**Given** I need error recovery
**When** I implement failure handling
**Then** individual file parsing errors don't stop the entire indexing process

**Given** crash prevention is implemented
**When** I handle system interruptions
**Then** indexing can resume from interruption point

**Given** error recovery works
**When** I validate robustness
**Then** indexing achieves zero crashes during normal vault operations (NFR25)

### Story 9.7: Integrate Indexing with Storage Persistence

As a developer coordinating indexing with storage,
I want indexing results persisted to storage,
So that indexed data is available for queries and survives restarts.

**Acceptance Criteria:**

**Given** indexing produces results
**When** I integrate with Epic 8 storage
**Then** indexed data is persisted using storage ports

**Given** storage integration works
**When** I handle large indexes
**Then** storage operations maintain performance within bounds

**Given** persistence is implemented
**When** I restart the system
**Then** indexed data is available without re-indexing

### Story 9.8: Implement Indexing Event Publishing

As a developer coordinating indexing with the event system,
I want indexing to publish events for system coordination,
So that other components are notified of indexing progress and completion.

**Acceptance Criteria:**

**Given** indexing operations occur
**When** I integrate with Epic 7 event bus
**Then** indexing publishes NoteIndexed, VaultIndexingStarted, VaultIndexingCompleted events

**Given** event publishing works
**When** I monitor indexing progress
**Then** subscribers receive real-time updates on indexing status

**Given** events are published
**When** I validate integration
**Then** other epics can subscribe to indexing events without tight coupling

### Story 9.9: Implement Indexing State Persistence

As a developer enabling resumable indexing,
I want indexing state persisted for interruption recovery,
So that long-running indexing operations can resume after interruptions.

**Acceptance Criteria:**

**Given** I need resumable indexing
**When** I implement state persistence
**Then** current indexing progress is saved periodically

**Given** state persistence works
**When** I interrupt indexing
**Then** indexing can resume from saved state without restarting

**Given** resumption works
**When** I validate reliability
**Then** large vault indexing survives system interruptions gracefully

### Story 9.10: Create Sample Vault Test Data

As a developer testing indexing functionality,
I want representative sample vault data,
So that indexing can be tested with realistic data volumes and patterns.

**Acceptance Criteria:**

**Given** I need test data
**When** I create sample vaults from docs/refs/obsidian/
**Then** samples include various file types, frontmatter patterns, and link structures

**Given** sample data exists
**When** I test indexing
**Then** samples validate all indexing scenarios and edge cases

**Given** samples are comprehensive
**When** I benchmark performance
**Then** test results are representative of real vault indexing performance

### Story 9.11: Create Vault Operation Mocks for Testing

As a developer testing vault-dependent code,
I want comprehensive mocks for vault operations,
So that vault interactions can be tested in isolation without filesystem access.

**Acceptance Criteria:**

**Given** I need to test vault operations
**When** I create mocks for vault ports
**Then** mock implementations simulate all vault port behaviors

**Given** mocks are available
**When** I write vault-dependent tests
**Then** tests verify correct vault operations without real filesystem

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate realistic vault behavior for comprehensive testing

### Story 9.12: Review Epic 9 Test Suite

As a developer maintaining the vault indexing system,
I want an efficient test suite for Epic 9 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 9 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for vault indexing components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across indexing components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 9 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 9.13: Document Vault Indexing System for Developers

As a developer working with vault operations,
I want comprehensive developer documentation for indexing,
So that vault indexing can be properly understood and maintained.

**Acceptance Criteria:**

**Given** indexing system is implemented
**When** I create developer documentation
**Then** it includes indexing algorithms, performance characteristics, and maintenance procedures

**Given** documentation exists
**When** developers read it
**Then** they understand indexing operations and troubleshooting procedures

**Given** indexing docs are complete
**When** other components integrate
**Then** they can work with indexed data effectively

### Epic 10: Query Service & Knowledge Graph
Users can perform fast lookups by filename, path, or schema keys, resolve wiki-links and aliases, and query metadata from other notes for template use.
**FRs covered:** FR21, FR22, FR23
**Implementation Notes:**
- QueryPort and mocks created in this epic
- CQRS read side (Epic 9 is write side)
- Performance benchmarking stories for NFR1 validation (<500ms queries)
- Observability/metrics for query performance
- File class queries for schema-based filtering
- Integration with Epic 8 storage and Epic 7 events

### Story 10.1: Create Query Domain Interface and Port

As a developer implementing query operations,
I want clean domain interfaces for query access,
So that queries follow hexagonal architecture principles.

**Acceptance Criteria:**

**Given** I need query operation contracts
**When** I create QueryPort trait
**Then** it includes methods for lookups, filtering, and resolution

**Given** QueryPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated query testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with clear separation between domain and infrastructure

### Story 10.2: Integrate Query Service with Storage Layer

As a developer coordinating queries with persistence,
I want query service integrated with storage layer,
So that queries retrieve data from the persisted index efficiently.

**Acceptance Criteria:**

**Given** I need storage integration
**When** I connect with Epic 8 storage
**Then** queries use storage ports for data retrieval

**Given** storage integration works
**When** I handle large datasets
**Then** queries maintain performance through storage optimization

**Given** integration is complete
**When** I validate data consistency
**Then** queries return data matching the indexed state

### Story 10.3: Implement Basic Query Operations

As a user needing to find notes,
I want basic lookup operations by filename and path,
So that I can quickly locate specific notes in the vault.

**Acceptance Criteria:**

**Given** I need basic queries
**When** I implement lookup operations
**Then** queries by filename and path return correct results

**Given** basic queries work
**When** I test with indexed data
**Then** results are retrieved from Epic 9 indexed data

**Given** lookups are implemented
**When** I validate performance
**Then** basic queries complete within acceptable time limits

### Story 10.4: Implement Schema-Based Query Filtering

As a user organizing notes by schema,
I want to filter queries by schema keys and metadata,
So that I can find notes with specific properties or schemas (when schemas are used).

**Acceptance Criteria:**

**Given** schemas are available
**When** I implement schema-based filtering
**Then** queries can filter by schema-defined metadata fields

**Given** schema filtering works
**When** I test with different schemas
**Then** results are correctly filtered by schema properties

**Given** filtering is implemented
**When** I validate edge cases
**Then** queries handle missing metadata gracefully and work without schemas

**Given** users don't use schemas
**When** they run queries
**Then** filtering works through direct frontmatter field queries

### Story 10.5: Implement File Class Query Operations

As a user categorizing notes by type,
I want to query notes by fileClass for schema-based organization,
So that I can find all "contact" notes or "project" notes efficiently.

**Acceptance Criteria:**

**Given** I need fileClass queries
**When** I implement fileClass filtering
**Then** queries can find all notes with specific fileClass values

**Given** fileClass queries work
**When** I test with schema inheritance
**Then** queries respect schema hierarchies and inheritance

**Given** fileClass operations are implemented
**When** I validate performance
**Then** fileClass queries are optimized for large result sets

### Story 10.6: Add Wiki-Link and Alias Resolution

As a user working with interconnected notes,
I want wiki-links and aliases resolved to actual note paths,
So that links work correctly across the knowledge graph.

**Acceptance Criteria:**

**Given** I need link resolution
**When** I implement wiki-link resolution
**Then** [[link]] syntax resolves to actual file paths

**Given** alias resolution is implemented
**When** I handle alias lookups
**Then** alias references resolve to correct targets

**Given** link resolution works
**When** I validate completeness
**Then** all wiki-link and alias patterns are properly resolved

### Story 10.7: Implement Query Cache Invalidation via Events

As a developer maintaining query performance,
I want cache invalidation through event system,
So that query results stay current when index updates occur.

**Acceptance Criteria:**

**Given** I need cache invalidation
**When** I integrate with Epic 7 events
**Then** query caches invalidate when NoteIndexed events are received

**Given** event integration works
**When** I test cache consistency
**Then** queries return updated results after index changes

**Given** invalidation is implemented
**When** I monitor performance
**Then** cache hit rates remain high while data stays current

### Story 10.8: Add Query Performance Optimization and Caching

As a developer optimizing query speed,
I want performance optimization with intelligent caching,
So that queries complete in <500ms meeting NFR1 requirements.

**Acceptance Criteria:**

**Given** I need performance optimization
**When** I implement caching with LRU strategy
**Then** frequently accessed query results are cached

**Given** caching is implemented
**When** I set TTL policies
**Then** cache entries expire appropriately to stay current

**Given** optimization works
**When** I benchmark queries
**Then** average query time is <500ms meeting NFR1

**Given** performance is validated
**When** I monitor metrics
**Then** cache hit rates and query latencies are tracked

### Story 10.9: Implement Query Result Formatting

As a user consuming query results,
I want results formatted appropriately for different use cases,
So that query output can be used directly or displayed clearly.

**Acceptance Criteria:**

**Given** I need result formatting
**When** I implement formatters
**Then** results can be formatted as lists, tables, or structured data

**Given** formatting works
**When** I test different output needs
**Then** formats are appropriate for CLI display and programmatic use

**Given** formatting is implemented
**When** I validate completeness
**Then** all query types have appropriate default formatting

### Story 10.10: Implement Advanced Query Composition

As a power user needing complex searches,
I want to compose multiple query conditions,
So that I can perform sophisticated searches across multiple criteria.

**Acceptance Criteria:**

**Given** I need complex queries
**When** I implement query composition
**Then** multiple conditions can be combined with AND/OR logic

**Given** composition works
**When** I test nested conditions
**Then** complex queries execute correctly and efficiently

**Given** advanced queries are implemented
**When** I validate performance
**Then** complex queries still meet NFR1 timing requirements

### Story 10.11: Create Query Operation Mocks for Testing

As a developer testing query-dependent code,
I want comprehensive mocks for query operations,
So that query interactions can be tested in isolation.

**Acceptance Criteria:**

**Given** I need to test query interactions
**When** I create mocks for QueryPort
**Then** mock implementations simulate all query behaviors

**Given** mocks are available
**When** I write query-dependent tests
**Then** tests verify correct query usage without real data

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate realistic query behavior for comprehensive testing

### Story 10.12: Review Epic 10 Test Suite

As a developer maintaining the query service,
I want an efficient test suite for Epic 10 components,
So that tests provide good coverage without redundancy or excessive execution time.

**Acceptance Criteria:**

**Given** all Epic 10 components are implemented with tests
**When** I review the test suite
**Then** it achieves 90%+ coverage for query service components

**Given** the test suite is implemented
**When** I check for redundancy
**Then** no duplicate test cases exist across query components

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 10 suite

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code

### Story 10.13: Document Query Service for Developers

As a developer working with query operations,
I want comprehensive developer documentation for the query service,
So that query functionality can be properly understood and used.

**Acceptance Criteria:**

**Given** query service is implemented
**When** I create developer documentation
**Then** it includes query APIs, performance characteristics, and caching behavior

**Given** documentation exists
**When** developers read it
**Then** they understand query operations and integration patterns

**Given** query docs are complete
**When** other components integrate
**Then** they can use query service effectively and efficiently

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

#### Story 11.1: [Domain] Unified Prompt, Suggestion, and Source Models
As a developer, I want domain entities that represent template variables and suggestion sources, so that the elicitation logic supports both simple lists and complex key-value maps.
**Acceptance Criteria:**
- **Given** the `domain` crate
- **When** I define the elicitation models
- **Then** `Suggestion` supports `display_text` (what the user sees) and `value` (what the template receives).
- **And** `ElicitationSource` supports `Static`, `DynamicQuery`, and `SchemaDerived` variants.
- **And** the `UIPort` trait is designed to return the complex `value` type.
**References:** FR16, FR17

#### Story 11.2: [Adapters/API] Basic Suggesters with List and Mapping Support
As a template author, I want basic suggesters that can accept both simple arrays/lists and key-value mappings, so that I can create interactive templates without requiring schema definitions.
**Acceptance Criteria:**
- **Given** a template with a `suggest()` call
- **When** I pass an array like `["Option 1", "Option 2", "Option 3"]`
- **Then** the suggester displays these options and returns the selected string value.
- **And** when I pass a mapping like `{display: "Option 1", value: "opt1"}`
- **Then** the suggester displays the "display" text but returns the "value" for template use.
- **And** the basic suggester works independently of schema definitions.
**References:** FR16, FR17

#### Story 11.3: [App] Schema-Driven Query Automation & Binding
As a template author, I want the schema to automatically simplify my queries, so that I don't have to manually write folder-listing logic in every template.
**Acceptance Criteria:**
- **Given** a schema property with a `FileSpec` and a `directory` constraint
- **When** a template variable is bound to this property
- **Then** the system automatically generates a "List Files" query for that directory to populate the suggester.
- **And** the `BindingService` raises a `miette` warning for variables missing from the schema while allowing them to proceed as simple string prompts.
- **And** schema metadata (enums, descriptions) automatically enriches the prompt definitions.
**References:** FR9, FR12, FR14

#### Story 11.4: [App] Dynamic Context Resolution Service
As a user, I want my suggesters to be populated by both automated schema queries and explicit template queries, so that I can pick from up-to-date vault data.
**Acceptance Criteria:**
- **Given** a `PromptSession`
- **When** the resolution service runs
- **Then** it executes all required vault queries (both schema-derived and template-explicit) before prompting the user.
- **And** it handles map-like results where keys and values differ (e.g. Note Title vs. File Path).
- **And** it caches query results for the duration of the session to ensure performance.
**References:** FR2, FR23

#### Story 11.5: [App] Interactive Loop Orchestrator (Atomic Workflow)
As a user, I want the system to ensure my vault remains clean if I cancel a template execution, so that I don't have to manually delete partial or empty files.
**Acceptance Criteria:**
- **Given** an active elicitation session
- **When** the user sends an `Abort` signal (Ctrl-C) or an error occurs
- **Then** the service terminates immediately and ensures no files are written to the vault.
- **And** all intermediate prompt data is purged from memory.
- **And** the orchestrator ensures the "Clean Slate" policy is respected across all execution steps.
**References:** FR24, FR49

#### Story 11.6: [Adapters/SPI] MiniJinja Variable Inspector & Extensions
As the system, I need to discover template requirements and provide a way for authors to trigger custom suggesters, so that the elicitation process is both automated and flexible.
**Acceptance Criteria:**
- **Given** a markdown template
- **When** the `TemplateInspector` runs
- **Then** it uses MiniJinja AST traversal to find all undeclared variables without performing a full render.
- **And** it registers a `suggest(options)` global function in the MiniJinja environment that can trigger the `UIPort` with custom data.
**References:** FR1, FR6

#### Story 11.7: [Adapters/API] Fuzzy Picker with Key-Value Support
As a user, I want a beautiful and responsive fuzzy-search interface in my terminal, so that I can find and select options quickly.
**Acceptance Criteria:**
- **Given** a list of `Suggestions` from the App layer
- **When** the `UIPort` implementation (via `inquire` or `dialoguer`) is called
- **Then** it renders a fuzzy-searchable list in the terminal.
- **And** searching only filters by the `display_text`, but the component returns the internal `value` upon selection.
- **And** it correctly captures terminal interrupt signals and returns an `Abort` signal to the App layer.
**References:** FR15, FR16

#### Story 11.8: [Test] Obsidian Templater Template Conversion & Fixtures
As a developer, I want to use real-world Obsidian templates as test fixtures, so that I can verify Lithos provides a viable migration path for power users.
**Acceptance Criteria:**
- **Given** the templates in `docs/refs/obsidian/00_system/`
- **When** I convert `42_00_action_item.md` to Lithos format
- **Then** the Lithos implementation must achieve the same metadata generation and file placement as the original.
- **And** the automated schema-derived queries must match the output of the original manual Javascript queries.
**References:** NFR20

#### Story 11.9: [Adapters/SPI] Chrono Date/Time Function Integration
As a template author, I want access to date/time functions using the existing chrono crate, so that I can format dates and perform date arithmetic without adding new dependencies.
**Acceptance Criteria:**
- **Given** the chrono crate is already in the tech stack
- **When** I integrate date functions into MiniJinja
- **Then** templates can use `date_now()`, `date_format()`, and `date_add()` functions.
- **And** functions follow chrono API patterns (not moment.js).
- **And** all functions are documented with examples in the standard library reference.
**References:** FR4, ADR 002

#### Story 11.10: [Adapters/SPI] Convert Case String Function Integration
As a template author, I want string case conversion functions using the convert_case crate, so that I can generate proper identifiers and titles without custom implementations.
**Acceptance Criteria:**
- **Given** the convert_case crate is available
- **When** I integrate case functions into MiniJinja
- **Then** templates can use `str_title_case()`, `str_snake_case()`, `str_kebab_case()`, etc.
- **And** functions handle Unicode properly and follow convert_case API patterns.
- **And** functions are documented with examples in the standard library reference.
**References:** FR1

#### Story 11.11: [Adapters/SPI] Slug Generation Function Integration
As a template author, I want URL-friendly slug generation using the slug crate, so that I can create valid identifiers for file names and URLs.
**Acceptance Criteria:**
- **Given** a slug crate is available (str_slug or similar)
- **When** I integrate slug functions into MiniJinja
- **Then** templates can use `str_slug()` to generate URL-safe identifiers.
- **And** the function handles Unicode, removes special characters, and replaces spaces with hyphens.
- **And** the function is documented with examples in the standard library reference.
**References:** FR1

#### Story 11.12: [Adapters/SPI] Base64 Encoding Function Integration
As a template author, I want base64 encoding/decoding functions using the base64 crate, so that I can encode binary data or create compact representations.
**Acceptance Criteria:**
- **Given** the base64 crate is available
- **When** I integrate base64 functions into MiniJinja
- **Then** templates can use `base64_encode()` and `base64_decode()` functions.
- **And** functions support standard base64 encoding/decoding.
- **And** functions are documented with examples in the standard library reference.
**References:** Additional utility functions

#### Story 11.13: [Adapters/SPI] Random Value Generation Integration
As a template author, I want random value functions using the rand crate, so that I can generate random numbers, strings, or selections for testing and variety.
**Acceptance Criteria:**
- **Given** the rand crate is available
- **When** I integrate random functions into MiniJinja
- **Then** templates can use `rand_int()`, `rand_float()`, and `rand_choice()` functions.
- **And** functions use cryptographically secure random generation where appropriate.
- **And** functions are documented with examples in the standard library reference.
**References:** Additional utility functions

#### Story 11.14: [Adapters/SPI] UUID Generation Function Integration
As a template author, I want UUID generation using the existing uuid crate, so that I can create unique identifiers for files and records.
**Acceptance Criteria:**
- **Given** the uuid crate is already in the tech stack
- **When** I integrate UUID functions into MiniJinja
- **Then** templates can use `uuid_v7()` to generate time-ordered unique identifiers.
- **And** the function follows the UUID v7 specification for sortability.
- **And** the function is documented with examples in the standard library reference.
**References:** Additional utility functions

#### Story 11.15: [Test] Epic 11 Test Suite Review & Optimization
As a developer, I want a comprehensive and efficient test suite for the interactive template system, so that I can maintain the code with confidence.
**Acceptance Criteria:**
- **Given** the implementation of Epic 11
- **When** I run the test suite
- **Then** it achieves 90%+ coverage for the `PromptSession` state machine and `BindingService`.
- **And** property-based tests verify that `Abort` signals never result in filesystem side-effects.
- **And** the suite validates architectural boundaries (e.g. Domain has zero I/O).
**References:** NFR16

#### Story 11.16: [Docs] Epic 11 User & Developer Documentation
As a user, I want clear instructions on how to create and use interactive templates with schema support, so that I can leverage the full power of the system.
**Acceptance Criteria:**
- **Given** a completed Epic 11
- **When** I review the documentation
- **Then** it includes a guide on how schemas automate folder-picking queries.
- **And** it provides examples for using the `suggest()` helper for ad-hoc terminal prompts.
- **And** it explains the "Clean Slate" policy and how to recover from errors.
- **And** it lists all available standard library functions with usage examples.
**References:** NFR13
**Acceptance Criteria:**
- **Given** a completed Epic 11
- **When** I review the documentation
- **Then** it includes a guide on how schemas automate folder-picking queries.
- **And** it provides examples for using the `suggest()` helper for ad-hoc terminal prompts.
- **And** it explains the "Clean Slate" policy and how to recover from errors.
- **And** it lists all available standard library functions with usage examples.
**References:** NFR13

### Epic 12: Advanced Template Features
Users can compose complex templates with date functions, multi-suggesters, and error prevention for production-ready template workflows.
**FRs covered:** FR3, FR4, FR17
**Implementation Notes:**
- Extends Epic 10 template system (not replacement)
- Date formatting with chrono (Rust-native)
- Template composition patterns
- User documentation for advanced template features
- Performance validation for complex templates

#### Story 12.1: [Domain] Template Dependency & Recursion Models
As a developer, I want to represent template relationships in the domain, so that I can detect circular dependencies and missing files before execution.
**Acceptance Criteria:**
- **Given** the `domain` crate
- **When** I define the `TemplateGraph` model
- **Then** it can represent parent-child relationships between templates via `include` statements.
- **And** the `CycleDetector` service can identify infinite recursion paths in a graph of template paths.
- **And** rich domain errors are defined for `CircularDependency` and `MissingPartial`.
**References:** FR3

#### Story 12.2: [Domain] TemplateDate Value Object
As a template author, I want a robust date domain model, so that I can perform reliable date math and formatting in my templates.
**Acceptance Criteria:**
- **Given** a date input
- **When** I create a `TemplateDate` value object
- **Then** it supports operations like `add_days(n)`, `subtract_days(n)`, and `format(String)`.
- **And** it correctly handles leap years and timezone offsets.
- **And** it is serializable for use in the template rendering context.
**References:** FR4

#### Story 12.3: [App] Template Composition "Dry Run" Orchestrator
As a user, I want the system to verify my template structure before asking for input, so that I don't waste time on a session that will fail due to a missing file.
**Acceptance Criteria:**
- **Given** a template execution request
- **When** the `DryRunService` runs
- **Then** it recursively parses the AST of the root template and all included partials.
- **And** if any partial is missing or a cycle is detected, it returns a `miette` diagnostic immediately.
- **And** this check must pass before the first prompt is displayed to the user.
**References:** FR3, FR48

#### Story 12.4: [App] Context-Aware Format Sensing Service
As a template author, I want the system to automatically format array variables based on their position in the file, so that my frontmatter remains valid YAML while my content remains readable markdown.
**Acceptance Criteria:**
- **Given** a rendering session
- **When** a variable is rendered between the `---` YAML delimiters
- **Then** the system automatically applies the `yaml_array_style` (Block vs. Flow) from the configuration.
- **And** it applies YAML-safe escaping to string values.
- **And** variables rendered outside the delimiters default to standard Markdown formatting.
**References:** FR17, FR26

#### Story 12.5: [Adapters/SPI] Chrono-based Natural Language Date Adapter
As a user, I want to provide relative dates like "tomorrow" or "next Friday" in my prompts, so that I can create notes for future events easily.
**Acceptance Criteria:**
- **Given** a natural language string from a prompt
- **When** the `DateResolutionAdapter` is called
- **Then** it resolves the string into a concrete timestamp using `chrono-english`.
- **And** it respects the `timezone` and `first_day_of_week` settings from the vault configuration.
- **And** it provides a fallback to the current date if the input is ambiguous.
**References:** FR4

#### Story 12.6: [Adapters/API] Multi-Select Terminal UI
As a user, I want to select multiple items from a list using a fuzzy-searchable terminal picker, so that I can quickly populate array fields like tags or contacts.
**Acceptance Criteria:**
- **Given** a list of suggestions
- **When** the `UIPort::prompt_multi_select` is called
- **Then** it renders a picker allowing the user to toggle multiple items (e.g., using spacebar).
- **And** it supports fuzzy-searching the display labels.
- **And** it returns a collection of the internal `values` for the selected items.
**References:** FR17

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
