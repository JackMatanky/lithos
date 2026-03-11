---
title: "Project Context Analysis"
description: "Analysis of project requirements, constraints, and architectural context for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Architecture Analysis"
---

# Project Context Analysis

## Requirements Overview

**Functional Requirements:**
The project encompasses 50 functional requirements across several key areas:

**Template Management (FR1-FR7):**
- Modular template creation, execution, and composition
- Date functions, dynamic commands, and custom user functions
- Advanced template operations with hooks and complex commands

**Schema Management (FR8-FR14):**
- Schema definition, validation, and inheritance patterns
- Interactive schema-driven templates with enums and file filtering
- Date formatting and complex validation rules

**Interactive Input (FR15-FR19):**
- Rich prompts, suggesters, and multi-selection interfaces
- Contextual help and progressive complexity modes

**Vault Operations (FR20-FR25):**
- Comprehensive indexing, searching, and link resolution
- Metadata queries and large vault performance optimization
- Vault consistency maintenance and cross-references

**Additional Core Features:**
- Configuration management, cross-environment compatibility
- Community features, security/privacy controls
- Rich command-line interface with subcommands and status reporting
- Comprehensive error handling and recovery mechanisms

These requirements drive a modular architecture with isolated business contexts (note, schema, template), sophisticated template engines, comprehensive schema validation systems, rich CLI interaction patterns, and robust vault indexing capabilities. The design emphasizes schema-driven development where schemas provide input parameters and validation rules, enabling modular template composition without manual coding. Files serve as the authoritative source of truth, with the database acting as a rebuildable projection/cache optimized for queries.

**Non-Functional Requirements:**

**Performance:**
- Template operations: <500ms response time
- Vault indexing: <2 seconds for 1000+ files
- Memory usage: <500MB for typical operations

**Security & Privacy:**
- Encrypted configuration files for sensitive data
- Comprehensive audit logging for template execution
- Access controls and data protection measures

**Scalability & Reliability:**
- Handle large vaults (1000+ files) without performance degradation
- 99.9% uptime with zero crashes in normal operation
- Graceful error recovery and state preservation

**Integration & Compatibility:**
- macOS and Linux cross-platform support (primary targets)
- CLI-first design with terminal environment optimization
- Graceful handling of Obsidian vault structure changes
- Migration path support for existing workflows

**Usability & Maintainability:**
- Clear help systems and progressive complexity disclosure
- Comprehensive testing coverage and self-contained binaries
- Fast deployment with rollback capabilities
- Comprehensive logging and diagnostic capabilities

**Scale & Complexity:**

- Primary domain: CLI developer tool with comprehensive template/schema management and future LSP ecosystem expansion
- Complexity level: medium-high (50 functional requirements, solo developer, ecosystem expansion)
- Estimated architectural components: 12-16 (isolated business contexts with unified Storage traits, storage implementations, CLI framework, template engine, schema system, vault indexer, interactive components)
- MVP scope recommendation: Reduce initial scope to 20-25 core functional requirements focusing on template execution, schema validation, and basic vault operations to maintain solo developer velocity and achieve 6-month MVP timeline
- Success metrics: Template creation time reduction (target: 75% faster than manual), crash rate (target: 0%), schema compliance automation (target: 95%)
- Agile guardrails: Weekly sprints with demo deliverables, daily standups, monthly retrospectives despite solo development to maintain discipline

## Technical Constraints & Dependencies

**Core Language:** Rust 1.92+ for memory safety and performance, enabling zero-cost abstractions and compile-time guarantees that prevent GC pauses during complex template composition.

**Platform Support:** macOS and Linux as primary targets, with future Windows support. Single binary distribution with no external runtime dependencies.

**Vault Integration:** Must work with Obsidian vault structure (markdown files, frontmatter, wikilinks) while being app-agnostic and supporting complex schema inheritance and validation.

**Template Engine:** Need to provide powerful templating functionality (prompts, suggesters, date functions, dynamic commands, custom functions, hooks, complex operations) adapted to Rust's capabilities and CLI-first workflow.

**Schema System:** Complex inheritance, validation, and interactive input system that must be performant, user-friendly in terminal environment, and support advanced features like file filtering and date formatting.

**CLI Complexity:** Rich interactive experiences with fuzzy finding, suggesters, multi-selection, progressive help, and single-word commands requiring sophisticated terminal interaction patterns. The CLI-first approach is viable with intelligent interfaces where schemas drive UX - enums become select lists, dates get formatters, with progressive complexity for different user expertise levels.

**Storage & Persistence:** **Redb + rkyv** for embedded persistence within `lithos-core`, enabling zero-copy deserialization essential for performance.

**Async Runtime:** Embrace Rust's async capabilities for CLI responsiveness, but use a **Sync-First** core domain to maximize compiler optimizations and simplicity, bridging to async only at the edges.

## Cross-Cutting Concerns Identified

**Performance:** Sub-millisecond response times for interactive operations via zero-copy data paths. Design with <500ms targets using direct database access.

**Error Handling:** Clear, actionable error messages using `miette` in the CLI, with typed `thiserror` definitions in the core.

**Type Safety:** Type-driven design - enforce invariants through the type system, private fields by default, validation at construction, newtype wrappers for domain constraints. Make illegal states unrepresentable.

**Architecture:** **Single-Crate Core** (`lithos-core`) to minimize zero-copy friction, with a thin CLI driver (`lithos-cli`). This replaces the traditional multi-crate workspace to prioritize performance and simplicity while maintaining logical modularity.

- **Logical Boundaries:** Enforced via visibility modifiers and module boundaries
- **Business Contexts:** note, schema, template are isolated (no cross-imports)
- **Cross-Cutting Context:** config (user-configurable business rules) available to all contexts
- **Pure Infrastructure:** db, fs (generic utilities) available to all contexts
- **Storage Pattern:** Each context defines unified `Storage` trait combining reads and writes
- **Zero-Copy:** Closure-based `with_archived()` methods provide zero-copy reads
- **Testing:** Storage traits enable test substitution (RedbStorage, InMemoryStorage, FakeStorage)

**Interactive UX:** Fuzzy finding, schema-driven prompts, suggesters, multi-selection, progressive help - demands advanced CLI interaction patterns for 50+ features, with contextual help and guidance during input operations.

**Cross-Platform Compatibility:** Consistent behavior across macOS/Linux, portable vault paths, terminal compatibility for comprehensive CLI interface.

**Security & Auditing:** Control access to vault data, audit template execution, encrypt sensitive configuration with logging requirements.

**Template/Schema Complexity:** Modular composition, inheritance chains, validation rules, interactive inputs - affects core business logic throughout the system.

**Configuration Management:** TOML-based configuration with extensive settings for templates, schemas, validation rules, and CLI behavior.

**Functional Orchestration:** Following ADR 002, use direct function composition for orchestration:
- Domain operations return `Result<T, E>` for error propagation
- Loaders orchestrate multi-phase pipelines (File → Raw → Domain → Storage)
- State machines model linear phase transitions (Schema/Config loading)
- Normal control flow for branching logic (cache checks, conditional loading)
- No event emission required - simple, idiomatic Rust patterns

**Test Architecture:** Pure domain tests (business logic), storage implementation tests (Redb/InMemory/Fake adapters), integration tests (cross-context workflows), end-to-end CLI tests. Storage traits enable separate test strategies (fast in-memory tests, integration tests with real DB). Performance testing with criterion benchmarks. Security testing for config encryption. TDD approach targeting 80%+ coverage.

**Documentation Strategy:** Match progressive complexity UX - power users get API docs + advanced guides, new users get quickstart tutorials + guided CLI help. Migration guides critical for adoption. Documentation as code with mdBook focusing on concrete outcomes.

**Open Source Considerations:** Personal project with community contribution potential. Design for contributor onboarding with clear examples, comprehensive documentation, and modular architecture supporting plugin ecosystem development.
