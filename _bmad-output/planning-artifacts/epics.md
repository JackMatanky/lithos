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

**Template System (Group 6 - Epic 5 Dependency):**
- **Template Struct Name Conflict**: Go had naming collision with text/template package
- **Rust Approach**: Carefully namespace template domain models; leverage MiniJinja without conflicts

**Critical Success Factors:**
- **Hexagonal Architecture**: Strictly enforce boundaries from day one; domain layer pure logic with zero I/O
- **Test Architecture**: Mirror hexagonal structure (pure domain tests, adapter integration tests, E2E CLI tests)
- **Async Discipline**: Use tokio::test for integration tests; surface race conditions early
- **Performance Benchmarking**: Use criterion for NFR-critical paths; 10k-note vault benchmarks mandatory

### FR Coverage Map

- FR1 → Epic 10 (Modular templates with reusable sections)
- FR2 → Epic 10 (Interactive template execution with prompts/suggesters)
- FR3 → Epic 11 (Complex template composition with error prevention)
- FR4 → Epic 11 (Date formatting and manipulation functions)
- FR5 → Post-MVP Phase 1.5 (Dynamic commands and whitespace control)
- FR6 → Post-MVP Phase 1.5 (Custom user functions)
- FR7 → Post-MVP Phase 2a (Advanced template operations with hooks)
- FR8 → Epic 5 (Define metadata schemas with field types)
- FR9 → Epic 5 (Schema-driven templates with input parameters)
- FR10 → Epic 5 (Validate notes against schemas)
- FR11 → Epic 5 (Schema enums populate suggester options)
- FR12 → Epic 5 (File filtering via schema directory constraints)
- FR13 → Epic 5 (Date formatting via schema format strings)
- FR14 → Epic 5 (Schema inheritance and extension)
- FR15 → Epic 10 (Free-text input through prompts)
- FR16 → Epic 10 (Single-choice suggesters)
- FR17 → Epic 11 (Multi-selection suggesters)
- FR18 → Post-MVP Phase 1.5 (Contextual help and guidance)
- FR19 → Post-MVP Phase 1.5 (Progressive complexity modes)
- FR20 → Epic 8 (Index and search notes across vaults)
- FR21 → Epic 9 (Lookups by filename, path, schema keys)
- FR22 → Epic 9 (Resolve wiki-links and aliases)
- FR23 → Epic 9 (Query metadata from other notes)
- FR24 → Epic 8 (Maintain vault consistency)
- FR25 → Epic 8 (Handle large vaults without degradation)
- FR26 → Epic 4 (Configure template packs via TOML)
- FR27 → Epic 4 (Manage schema definitions via config)
- FR28 → Epic 4 (Set application preferences via config)
- FR29 → Post-MVP Phase 2c (Custom validation/linting rules)
- FR30 → Epic 12 (Cross-platform execution consistency)
- FR31 → Epic 12 (Access through terminal interfaces)
- FR32 → Post-MVP Phase 3a (External editor/IDE integration)
- FR33 → Post-MVP Phase 3a (Automated scripts/CI-CD support)
- FR34 → Post-MVP Phase 3b (Share template packs via Git)
- FR35 → Post-MVP Phase 3b (Discover community template packs)
- FR36 → Post-MVP Phase 3b (Validate third-party templates)
- FR37 → Post-MVP Phase 3b (Community contribution workflows)
- FR38 → Post-MVP Phase 4 (Access control for vault data)
- FR39 → Post-MVP Phase 4 (Encrypt sensitive configuration) - Basic version in Epic 4
- FR40 → Post-MVP Phase 4 (Audit template execution) - Basic version in Epic 12
- FR41 → Epic 12 (Execute commands with subcommands)
- FR42 → Epic 12 (Comprehensive help and documentation)
- FR43 → Epic 12 (View status and configuration)
- FR44 → Epic 12 (Manage vault operations from CLI)
- FR45 → Epic 12 (Run templates with output formats)
- FR46 → Epic 12 (Configure CLI behavior)
- FR47 → Epic 12 (Single-word command shortcuts)
- FR48 → Epic 12 (Clear, actionable error messages)
- FR49 → Epic 12 (Recover from failed executions with rollback)
- FR50 → Epic 12 (Diagnose and troubleshoot issues)

## Epic List

### Epic 1: Development Environment & Tooling
Developers have a fully configured development environment with quality gates, testing infrastructure, and task orchestration that enforces architectural standards.
**FRs covered:** Architecture requirements (tooling, quality gates)
**Implementation Notes:**
- Cargo workspace, mise, pre-commit hooks, clippy, rustfmt
- ADR review process and validation of existing ADRs (001-006)
- Foundation for all subsequent epics

### Epic 2: Test Architecture, Patterns & Utilities
Developers have comprehensive testing patterns for async code, event-driven systems, and CQRS, plus centralized test utilities for artifacts, temporary directories, and mise task orchestration that ensure 80%+ coverage and catch integration issues early.
**FRs covered:** NFR16 (test coverage), Architecture requirements (testing strategy)
**Implementation Notes:**
- Test patterns: async (tokio::test), event-driven, CQRS command/query separation
- Centralized test utilities: artifact output locations, tmp directory management, test helper functions
- mise.toml test tasks: test, test:unit, test:integration, test:coverage, test:watch, test:benchmark
- ADR creation guidelines for epics making architectural decisions

### Epic 3: Core Domain Models & Value Objects
Developers have a clear, shared domain language with rich domain models that embody business rules and validation logic, informed by Obsidian patterns and Go implementation lessons learned.
**FRs covered:** Architecture requirements (DDD domain models)
**Implementation Notes:**
- Core stable models: Config, Schema, Note, Frontmatter, Template + value objects
- Models informed by Obsidian structures (TFile, CachedMetadata) and Go implementation
- Flexibility for Rust-specific refinements and supplementary models in later epics
- Mocks for domain interfaces created as needed (not upfront)

### Epic 4: Configuration Management System
Users can configure lithos through hierarchical TOML files with validation, supporting template packs and schema definitions.
**FRs covered:** FR26, FR27, FR28
**Implementation Notes:**
- Figment-based hierarchical config per ADR 004
- ConfigPort and mocks created in this epic
- Sample config files from docs/refs/obsidian/ converted as test fixtures
- User documentation for configuration

### Epic 5: Schema System & Validation
Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**
- SchemaPort and mocks created in this epic
- Sample schema files from docs/refs/obsidian/ converted as test fixtures
- Schema validation (syntactic in adapter, semantic in domain)
- User documentation for schema creation

### Epic 6: Event Bus & Orchestration Infrastructure
System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.
**FRs covered:** Architecture requirements (event-driven, CQRS foundation)
**Implementation Notes:**
- Hybrid Event Bus (MPSC/Broadcast/Watch per ADR 006)
- Event payload schema design and validation
- EventBusPort mocks for testing
- Prevents god-object orchestrators (Go lesson learned)
- May create ADR for event patterns if architectural decisions made

### Epic 7: Storage Layer & Persistence
System has zero-copy persistent storage with ACID transactions using Redb + rkyv that supports high-performance queries and maintains data consistency.
**FRs covered:** Architecture requirements (Redb + rkyv storage per ADR 001)
**Implementation Notes:**
- Redb + rkyv per ADR 001 (no SQLite - decision already made)
- Storage schema design review against Epic 8-9 query requirements
- Unit of Work pattern for transactional consistency
- Storage port mocks for testing
- May create ADR for storage schema patterns if needed

### Epic 8: Vault File System Integration & Indexing Engine
Users can index large vaults (1000+ files) in <2 seconds with incremental updates, reliable crash-free operation, and persistent storage.
**FRs covered:** FR20, FR24, FR25
**Implementation Notes:**
- VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort and mocks created
- pulldown-cmark for markdown parsing (adapter layer per ADR 003)
- Sample vault notes from docs/refs/obsidian/ as test fixtures
- Performance benchmarking stories for NFR2 validation (<2s for 1000+ files)
- Observability/metrics for indexing performance

### Epic 9: Query Service & Knowledge Graph
Users can perform fast lookups by filename, path, or schema keys, resolve wiki-links and aliases, and query metadata from other notes for template use.
**FRs covered:** FR21, FR22, FR23
**Implementation Notes:**
- QueryPort and mocks created in this epic
- CQRS read side (Epic 8 is write side)
- Performance benchmarking stories for NFR1 validation (<500ms queries)
- Observability/metrics for query performance

### Epic 10: Basic Interactive Template System
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

### Epic 11: Advanced Template Features
Users can compose complex templates with date functions, multi-suggesters, and error prevention for production-ready template workflows.
**FRs covered:** FR3, FR4, FR17
**Implementation Notes:**
- Extends Epic 10 template system (not replacement)
- Date formatting with chrono (Rust-native)
- Template composition patterns
- User documentation for advanced template features
- Performance validation for complex templates

### Epic 12: CLI Interface & Error Handling
Users can execute lithos commands with intuitive CLI, comprehensive help, single-word shortcuts, and actionable error diagnostics.
**FRs covered:** FR41, FR42, FR43, FR44, FR45, FR46, FR47, FR48, FR49, FR50, FR30, FR31
**Implementation Notes:**
- Clap for CLI, miette for diagnostics per ADR 005
- CommandPort, AuditPort created if needed
- Dependency injection wiring for all ports
- Cross-platform support (macOS primary, Linux)
- Observability/audit logging for FR40 (basic version)
- User CLI documentation

### Epic 13: Test Suite Review & Optimization
Development team has a validated, efficient test suite with no redundancy, full coverage of critical paths, and effective system validation.
**FRs covered:** NFR16 (comprehensive test coverage), NFR25 (zero crashes)
**Implementation Notes:**
- Final holistic review after all epic-level tests complete
- Identify overlapping/redundant tests across Epics 4-12
- Validate test suite efficiency and execution time
- Ensure 80%+ coverage without bloat
- Architectural boundary validation (hexagonal, CQRS, event-driven)
- Note: Each epic 4-12 has its own test validation story; this is final optimization

### Epic 14: User Documentation & Onboarding
Users have comprehensive documentation, starter templates, sample schemas, and migration guides that enable successful adoption.
**FRs covered:** NFR13 (clear help), NFR20 (migration paths), NFR28 (installation success)
**Implementation Notes:**
- Consolidates documentation from Epics 4-12
- Starter kit from converted docs/refs/obsidian/ samples (sanitized)
- Installation guide, quickstart, migration guides
- API documentation for power users
- Progressive complexity documentation (basic → advanced)
- Note: Documentation created at story-level in epics; this consolidates and polishes
