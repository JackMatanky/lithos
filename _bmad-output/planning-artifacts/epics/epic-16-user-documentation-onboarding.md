# Epic 16: User Documentation & Onboarding

## Overview

Users have comprehensive documentation, starter templates, sample schemas, and migration guides that enable successful adoption.

**FRs covered:** NFR13 (clear help), NFR15 (progressive complexity), NFR20 (migration paths), NFR28 (installation success)

## Implementation Notes

- **Documentation Strategy**: Consolidates epic-level documentation from Epics 4-15 into cohesive user guides
- **Integration Points**:
  - All Epics 4-15: Aggregates documentation created at story level
  - Epic 6: Documents configuration hierarchy and settings
  - Epic 7: Documents schema system and validation
  - Epic 10: Documents vault indexing and file management
  - Epic 11: Documents query syntax and search operations
  - Epic 12/13: Documents template creation and advanced features
  - Epic 14: Documents CLI commands and error recovery
- **Starter Kit**: Based on converted docs/refs/obsidian/ samples (sanitized, tested)
- **Documentation Formats**:
  - Markdown for human-readable docs
  - mdBook for website generation
  - man pages for CLI reference (generated from Clap)
  - API docs via rustdoc for developers
- **Target Audiences**:
  - **Beginners**: Installation, quickstart, basic templates
  - **Intermediate**: Schema creation, advanced templates, vault organization
  - **Advanced**: Plugin development, API reference, architecture
  - **Migrators**: Obsidian/Templater conversion guides
- **Documentation Locations**:
  - `docs/` - user-facing documentation (mdBook source)
  - `docs/api/` - API reference (rustdoc output)
  - `docs/examples/` - working code examples
  - `docs/starter-kit/` - templates and schemas for new users
  - `docs/migration/` - migration guides from other tools
- **Validation Strategy**:
  - All code examples are executable tests (doc tests)
  - All CLI commands tested in integration tests
  - All configuration examples validated against schema
  - Documentation built and deployed in CI/CD
- **Progressive Complexity**: Clear learning paths with prerequisites stated
- **Cross-References**: Extensive linking between related topics
- **Search**: Full-text search via mdBook built-in search
- **Performance**: Documentation site loads in <1s, search <100ms
- **Accessibility**: WCAG 2.1 AA compliance for documentation website
- **Localization**: English primary, structure supports future i18n

### Story 16.1: [Docs] Installation and Setup Guide

As a new user, I want clear installation instructions and setup guidance, so that I can get lithos running quickly on my system.
**Acceptance Criteria:**


## Story 16.1: Create Installation and Setup Guide

As a new user, I want clear installation instructions and setup guidance,
So that I can get lithos running quickly on my system.

**Acceptance Criteria:**

**Given** lithos targets macOS primary, Linux secondary
**When** I create installation guide in `docs/installation.md`
**Then** it includes platform-specific instructions:
- macOS: Homebrew formula, binary download, building from source
- Linux: Package managers (apt, dnf, pacman), binary download, cargo install
- Prerequisites: Rust toolchain (for source builds), system dependencies

**Given** NFR28 requires 95% successful installations
**When** I write installation steps
**Then** each step is numbered, testable, and includes expected output
**And** common errors are documented with solutions
**And** troubleshooting section covers: permissions, PATH, dependencies

**Given** first-run experience is critical
**When** user runs lithos for first time
**Then** documentation guides through:
1. Verify installation: `lithos --version`
2. Create first vault: `lithos vault init ~/notes`
3. Set vault path: `lithos config set vault.path ~/notes`
4. Verify setup: `lithos vault index`

**Given** Epic 6 configuration is complex
**When** I document initial configuration
**Then** explain hierarchy: global.toml (system-wide) vs vault.toml (vault-specific)
**And** provide minimal config example that works out-of-box
**And** reference Epic 6 docs for advanced configuration

**Given** users may encounter platform-specific issues
**When** I create troubleshooting guide
**Then** document common issues:
- macOS: Gatekeeper warnings for unsigned binaries
- Linux: Missing system libraries (rkyv, redb dependencies)
- Permissions: .lithos directory creation failures
**And** each issue includes: symptom, cause, solution

**Given** installation success rate must be measurable
**When** I implement validation
**Then** create smoke test script: `scripts/verify-install.sh`
**And** script checks: binary exists, runs without error, creates vault
**And** installation guide links to smoke test for self-verification

**References:** NFR28, NFR13


## Story 16.2: Create Quick Start Tutorial

As a new user, I want a hands-on tutorial to create my first note with lithos,
So that I can experience the core functionality immediately.

**Acceptance Criteria:**

**Given** users need immediate value to stay engaged
**When** I create quickstart guide in `docs/quickstart.md`
**Then** tutorial completes in <15 minutes for first-time users
**And** covers: vault setup, first template, first note, basic search
**And** uses starter kit templates (Story 16.3) for consistency

**Given** concepts must be introduced gradually
**When** I structure the tutorial
**Then** learning sequence:
1. What is a vault? (2 min)
2. Create your first vault (`lithos vault init ~/my-notes`)
3. What is a template? (2 min)
4. Execute starter template (`lithos template new daily-note`)
5. View created note (open in editor)
6. What is indexing? (1 min)
7. Search your vault (`lithos vault search "today"`)
8. Next steps (link to full documentation)

**Given** Epic 12 provides template execution
**When** tutorial demonstrates template use
**Then** use simple daily-note template from starter kit
**And** template prompts for: date, mood, goals (3 variables max)
**And** show template output in markdown format

**Given** Epic 11 provides search functionality
**When** tutorial demonstrates search
**Then** explain basic search: text match, schema filter, metadata filter
**And** show search results with clickable paths

**Given** users learn by doing
**When** I write tutorial content
**Then** every step has: command to run, expected output, explanation
**And** provide copy-paste ready commands
**And** include screenshots for key UI interactions

**References:** NFR13, NFR15

## Story 16.3: Create Starter Template and Schema Library

As a new user, I want ready-to-use templates and schemas for common use cases,
So that I can start productive work immediately.

**Acceptance Criteria:**

**Given** docs/refs/obsidian/ contains example templates
**When** I create starter kit in `docs/starter-kit/`
**Then** sanitize and convert Obsidian templates to lithos format
**And** include templates: daily-note, project, contact, meeting-note, knowledge-note
**And** include schemas: person, project, meeting, article, book

**Given** Epic 7 provides schema system
**When** I create starter schemas
**Then** each schema includes:
- Property definitions (name, type, constraints)
- Validation rules
- Usage examples
- Integration with templates

**Given** Epic 12 templates use schema-driven prompts
**When** I create starter templates
**Then** templates reference schemas via fileClass
**And** prompts auto-populate from schema constraints
**And** variables use suggesters from Epic 11 queries where applicable

**Given** templates must be immediately usable
**When** user installs starter kit
**Then** provide installation script: `lithos install-starter-kit`
**And** script copies templates to vault templates directory
**And** script copies schemas to vault schemas directory
**And** validates all files after installation

**Given** templates need documentation
**When** I document each template
**Then** include README.md in starter-kit/ with:
- Template descriptions and use cases
- Required schemas
- Customization examples
- Common patterns

**Given** users need migration path
**When** I provide Obsidian template conversions
**Then** document conversion patterns:
- Templater tp.date.now() → lithos date functions
- Templater tp.system.suggester() → lithos suggesters
- Templater tp.file.include() → MiniJinja includes
**And** provide side-by-side comparison examples

**References:** NFR20, NFR13

## Story 16.4: Create Migration Guide from Obsidian

As an existing Obsidian user, I want guidance on migrating my workflow to lithos,
So that I can transition smoothly with minimal disruption.

**Acceptance Criteria:**

**Given** many users come from Obsidian/Templater
**When** I create migration guide in `docs/migration/obsidian.md`
**Then** map Obsidian concepts to lithos equivalents:
- Vault → Vault (compatible, lithos adds .lithos/ directory)
- Templates → MiniJinja templates (conversion required)
- Templater plugin → Built-in template system
- Dataview → Epic 11 query system
- Frontmatter → Frontmatter (compatible, YAML)
- Tags → Tags (compatible)
- Wiki-links → Wiki-links (compatible, Epic 11 resolves)

**Given** template syntax differs significantly
**When** I document template conversion
**Then** provide conversion table:

| Obsidian/Templater | Lithos |
|--------------------|--------|
| `<% tp.date.now() %>` | `{{ date.now() }}` |
| `<% tp.date.now("YYYY-MM-DD") %>` | `{{ date.now().format("%Y-%m-%d") }}` |
| `<% tp.system.suggester(items) %>` | `{{ prompt("Choose", items) }}` |
| `<% tp.file.include("[[partial]]") %>` | `{% include "partial.md" %}` |

**Given** Epic 13 provides advanced template features
**When** I document template migration
**Then** explain Epic 13 features that replace Templater:
- Date functions: chrono-based, natural language
- Multi-select: Dialoguer with fuzzy search
- Template composition: MiniJinja includes with cycle detection

**Given** migration must handle edge cases
**When** I document limitations
**Then** clearly state incompatibilities:
- JavaScript execution (Templater tp.user scripts) → not supported
- Dynamic file creation → use CLI workflows instead
- Obsidian-specific APIs → use lithos equivalents

**Given** users need practical migration steps
**When** I provide migration workflow
**Then** step-by-step process:
1. Backup Obsidian vault
2. Create lithos vault pointing to same directory
3. Run `lithos vault index` to index existing notes
4. Convert templates one-by-one (use conversion table)
5. Test converted templates
6. Gradually adopt lithos schemas for validation

**References:** NFR20, NFR13

## Story 16.5: Create User Manual and Feature Reference

As a power user, I want comprehensive documentation of all features and configuration options,
So that I can master advanced functionality.

**Acceptance Criteria:**

**Given** all Epics 6-15 have story-level documentation
**When** I consolidate into user manual in `docs/manual/`
**Then** organize by topic:
- `config.md` - Epic 6 configuration system
- `schemas.md` - Epic 7 schema validation
- `templates.md` - Epic 12 basic templates
- `advanced-templates.md` - Epic 13 composition, dates, multi-select
- `vault.md` - Epic 10 indexing, file management
- `search.md` - Epic 11 query syntax, filters
- `cli.md` - Epic 14 all commands, flags, shortcuts
- `troubleshooting.md` - Epic 14 error recovery, diagnostics

**Given** Epic 6 configuration has many options
**When** I document configuration
**Then** include complete reference:
- All global.toml settings with defaults
- All vault.toml settings with defaults
- Environment variables (LITHOS_*)
- Precedence rules
- Validation constraints

**Given** Epic 7 schema system is complex
**When** I document schemas
**Then** include:
- Schema file format (YAML structure)
- Property types (string, number, boolean, date, file)
- Constraints (min/max, pattern, enum, required)
- Inheritance (extends, excludes)
- Validation error interpretation

**Given** Epic 11 query syntax has many operators
**When** I document search
**Then** provide query syntax reference:
- Text search: simple text, quoted phrases, wildcards
- Schema filter: `--schema <name>`
- Metadata filter: `--tag <tag>` `--status <status>`
- Composition: AND/OR logic
- Results formatting: table, JSON, paths-only

**Given** Epic 14 CLI has many commands
**When** I document CLI
**Then** include for each command:
- Synopsis (one-line description)
- Usage syntax with all flags
- Examples (basic and advanced)
- Exit codes
- Related commands

**Given** troubleshooting is critical
**When** I create troubleshooting guide
**Then** organize by symptom:
- "Command not found" → installation/PATH issues
- "Vault not indexed" → Epic 10 indexing errors
- "Template validation failed" → Epic 7 schema errors
- "Circular dependency detected" → Epic 13 composition errors
**And** each issue has: symptom, cause, solution, prevention

**References:** NFR13, NFR15

## Story 16.6: Create API Documentation for Developers

As a developer extending lithos, I want API documentation for the plugin system and extension points,
So that I can build custom integrations.

**Acceptance Criteria:**

**Given** lithos uses hexagonal architecture
**When** I document architecture in `docs/api/architecture.md`
**Then** explain layers:
- Domain: Core business logic (ports, aggregates, value objects)
- Application: Use case orchestration (services)
- Adapters: External integrations (API, SPI)
- Infrastructure: Cross-cutting concerns (logging, events)

**Given** Epic 8 provides event bus for extensibility
**When** I document event system
**Then** include:
- EventBus architecture (DataPlane, ControlPlane, StatePlane)
- Event types (NoteIndexed, NoteDeleted, ConfigUpdated, etc.)
- Subscribing to events
- Publishing custom events
- Performance considerations (<50ms latency)

**Given** developers need code examples
**When** I provide extension examples
**Then** include working examples:
- Custom template function (register with MiniJinja)
- Custom suggester (implement UIPort)
- Custom schema validator (implement validation trait)
- Custom storage backend (implement CacheReader/Writer)

**Given** rustdoc generates API reference
**When** I configure rustdoc
**Then** ensure all public APIs have doc comments
**And** examples in doc comments are executable (doc tests)
**And** rustdoc output published to docs/api/rust/

**Given** ports define extension points
**When** I document ports
**Then** list all domain ports:
- ConfigCommand/ConfigQuery (Epic 6)
- SchemaCommand/SchemaQuery (Epic 7)
- TemplatePort, UIPort (Epic 12)
- QueryPort (Epic 11)
- CacheReader/CacheWriter (Epic 5)
**And** explain when to implement each port

**References:** NFR13

## Story 16.7: Implement Progressive Complexity Documentation Structure

As a user at any skill level, I want documentation organized by complexity,
So that I can learn at my own pace without being overwhelmed.

**Acceptance Criteria:**

**Given** users have varying skill levels
**When** I organize documentation in `docs/`
**Then** create clear learning paths:
- **Beginner/** - installation, quickstart, first template
- **Intermediate/** - schemas, advanced templates, vault organization
- **Advanced/** - plugin development, performance tuning, architecture

**Given** each level builds on previous
**When** I write content
**Then** prerequisites clearly stated at document start
**And** concepts introduced incrementally
**And** avoid redundant explanations via cross-references

**Given** navigation must be intuitive
**When** I structure mdBook
**Then** SUMMARY.md organizes by complexity:
```
# Summary
- [Installation](installation.md)
- [Quick Start](quickstart.md)
- [Beginner Guide](beginner/README.md)
  - [Your First Template](beginner/first-template.md)
  - [Basic Vault Management](beginner/vault-basics.md)
- [Intermediate Guide](intermediate/README.md)
  - [Creating Schemas](intermediate/schemas.md)
  - [Advanced Templates](intermediate/templates.md)
- [Advanced Guide](advanced/README.md)
  - [Plugin Development](advanced/plugins.md)
  - [Architecture Deep Dive](advanced/architecture.md)
```

**Given** cross-references guide users
**When** I link related topics
**Then** use clear link text: "See [Schema Validation](../intermediate/schemas.md) for details"
**And** provide "Next Steps" section at end of each document
**And** link to related CLI commands, configuration options

**Given** search helps users find information
**When** I configure mdBook search
**Then** enable full-text search across all documentation
**And** search results prioritize beginner content
**And** search includes code examples

**References:** NFR15, NFR13

## Story 16.8: Implement Documentation Validation and Testing

As a documentation maintainer, I want validation that all documentation is accurate and complete,
So that users receive reliable information.

**Acceptance Criteria:**

**Given** documentation contains code examples
**When** I implement validation
**Then** all Rust code examples are doc tests (run with `cargo test --doc`)
**And** all CLI commands tested in integration tests
**And** all templates in examples/ are valid MiniJinja syntax

**Given** documentation references configuration options
**When** I validate config docs
**Then** cross-check against Epic 6 Config schema
**And** ensure all documented options exist in code
**And** ensure all code options are documented

**Given** documentation includes CLI commands
**When** I validate CLI docs
**Then** run all documented commands in CI
**And** verify output matches documented examples
**And** check all flags and options are correct

**Given** documentation may become stale
**When** I implement continuous validation
**Then** CI job runs on every doc change:
1. Build mdBook site
2. Run doc tests
3. Validate config examples
4. Test CLI commands
5. Check broken links
**And** CI fails if any validation fails

**Given** starter kit must work
**When** I test starter kit
**Then** integration test:
1. Install starter kit
2. Execute each template
3. Validate created notes against schemas
4. Verify all templates render without errors

**Given** documentation coverage must be complete
**When** I audit documentation
**Then** checklist:
- All Epics 6-15 features documented
- All CLI commands documented
- All configuration options documented
- All error codes documented
- All NFRs addressed in relevant docs

**References:** NFR13, NFR16

- **Given** the completed system
- **When** I create the installation guide
- **Then** it includes step-by-step instructions for macOS and Linux.
- **And** it covers prerequisites, binary installation, and initial configuration.
- **And** it achieves 95% successful installations based on user feedback.
  **References:** NFR28

### Story 16.2: [Docs] Quick Start Tutorial

As a new user, I want a hands-on tutorial to create my first note with lithos, so that I can experience the core functionality immediately.
**Acceptance Criteria:**

- **Given** the completed system
- **When** I create the quick start guide
- **Then** it walks through creating a simple note template and executing it.
- **And** it introduces basic concepts (vaults, schemas, templates) through examples.
- **And** it takes <15 minutes to complete for first-time users.
  **References:** NFR13

### Story 16.3: [Docs] Starter Template and Schema Library

As a new user, I want ready-to-use templates and schemas for common use cases, so that I can start productive work immediately.
**Acceptance Criteria:**

- **Given** the converted Obsidian templates
- **When** I create the starter kit
- **Then** it includes sanitized templates for daily notes, projects, contacts, and knowledge notes.
- **And** it provides sample schemas for common metadata patterns.
- **And** all starter content is documented with usage examples.
  **References:** NFR20

### Story 16.4: [Docs] Migration Guide from Obsidian

As an existing Obsidian user, I want guidance on migrating my workflow to lithos, so that I can transition smoothly with minimal disruption.
**Acceptance Criteria:**

- **Given** the template conversion examples
- **When** I create the migration guide
- **Then** it maps Obsidian concepts to lithos equivalents (Templater → lithos templates).
- **And** it provides conversion examples for common template patterns.
- **And** it addresses compatibility considerations and limitations.
  **References:** NFR20

### Story 16.5: [Docs] User Manual and Feature Reference

As a power user, I want comprehensive documentation of all features and configuration options, so that I can master advanced functionality.
**Acceptance Criteria:**

- **Given** all epic-level documentation
- **When** I consolidate the user manual
- **Then** it includes detailed sections for templates, schemas, vaults, and CLI.
- **And** it documents all configuration options and environment variables.
- **And** it provides troubleshooting guides for common issues.
  **References:** NFR13

### Story 16.6: [Docs] API Documentation for Developers

As a developer extending lithos, I want API documentation for the plugin system and extension points, so that I can build custom integrations.
**Acceptance Criteria:**

- **Given** the system architecture
- **When** I create the API documentation
- **Then** it documents the hexagonal architecture ports and adapters.
- **And** it provides examples for creating custom template functions and suggesters.
- **And** it includes the Rust API reference for power users.
  **References:** NFR13

### Story 16.7: [Docs] Progressive Complexity Documentation Structure

As a user at any skill level, I want documentation organized by complexity, so that I can learn at my own pace without being overwhelmed.
**Acceptance Criteria:**

- **Given** all documentation content
- **When** I organize it by complexity levels
- **Then** it provides clear learning paths: Beginner → Intermediate → Advanced.
- **And** each level builds on the previous without redundant explanations.
- **And** cross-references guide users to more detailed information when needed.
  **References:** NFR15

### Story 16.8: [Test] Epic 16 Documentation Validation

As a documentation maintainer, I want validation that all documentation is accurate and complete, so that users receive reliable information.
**Acceptance Criteria:**

- **Given** the completed documentation
- **When** I validate it against the implementation
- **Then** all code examples are tested and functional.
- **And** all CLI commands in documentation work as described.
- **And** all configuration options are accurately documented.
  **References:** NFR13
