# Story 3.7: Create Epic 3 Documentation

Status: done

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer working with the domain models,
I want comprehensive documentation of the domain entities, their relationships, and evolution guidelines,
So that developers understand the domain language and can work effectively with the models.

## Acceptance Criteria

1. **Given** all Epic 3 domain models are implemented
   **When** I create documentation
   **Then** it includes developer-focused content:
   - Domain entity relationships and bounded contexts
   - Semantic validation rules for each entity
   - Domain entity relationship contracts (how bounded contexts interact)
   - Evolution guidelines for domain models (when to add vs modify entities)
   - Architecture diagrams showing entity relationships and contracts

2. **Given** documentation is created
   **When** I validate completeness
   **Then** it covers all entities and their contracts: Note aggregate, Schema domain, Config, Template

3. **Given** documentation exists
   **When** I check relationship contracts
   **Then** it defines how bounded contexts interact (e.g., Template references Schema, Note uses Config)

4. **Given** documentation exists
   **When** a developer reads it
   **Then** they understand domain evolution rules and inter-entity contracts without needing user-facing knowledge

5. **Given** Epic 3 domain models are implemented
   **When** I review the source code
   **Then** all files in the domain (`crates/domain/src/`) are reviewed thoroughly to ensure full and proper use of doc comments with doc tests and that every relevant component has a well written doc comment.

## Tasks / Subtasks

### Task 1: Audit and Improve Inline Domain Documentation (Doc Comments & Doc Tests)
- [x] Thoroughly review all files in `crates/domain/src/` for full and proper use of doc comments (`///`).
- [x] Ensure every relevant component (structs, enums, traits, public methods) has a well-written, accurate, and precise doc comment.
- [x] Add or improve doc tests to provide runnable examples and verify invariants.
- [x] Ensure doc comments serve as the primary source of truth for developer-level understanding of individual components.

### Task 2: Implement Documentation Utility Enhancements (Anti-Patterns & Traceability)
- [x] **Traceability Audit**: Verify that all domain rules defined in the PRD and Architecture artifacts are accurately represented in the documentation.
- [x] **Anti-Patterns Section**: Add a "Common Pitfalls & Anti-Patterns" section to `docs/domain-models.md` for each bounded context (e.g., avoiding I/O in domain, incorrect phantom type usage).
- [x] **Cookbook Examples**: Add "How-to" snippets for common domain extensions (e.g., "Adding a new Config level", "Creating a custom PropertySpec").
- [x] **ADR Mapping**: Cross-reference documented domain behaviors with the relevant ADRs (Architecture Decision Records) to provide historical context.

### Task 3: Analyze Epic 3 Domain Models for Documentation Scope
- [x] Read all files in `crates/domain/src/note/` directory to understand Note bounded context: Note aggregate, Frontmatter, Link, Embed, Tag, Heading, Task, Section entities and their validation rules, business logic
- [x] Read all files in `crates/domain/src/schema/` directory to understand Schema bounded context: Schema aggregate, Property, PropertyBank, PropertySpec trait and implementations (StringSpec, NumberSpec, etc.), inheritance resolution, trait-based generic design
- [x] Read all files in `crates/domain/src/config/` directory to understand Config bounded context: Config entity with phantom types, ConfigValue, ConfigPath, ValidationRule, hierarchical merging, encryption boundary
- [x] Read all files in `crates/domain/src/template/` directory to understand Template bounded context: Template aggregate, VariableDefinition, TemplateComposition, MiniJinja compatibility, domain purity (no syntax validation)
- [x] Analyze inter-entity relationships: Note ↔ Config (defaults), Note ↔ Schema (validation), Template ↔ Schema (variable constraints), Template ↔ Config (execution settings)
- [x] Document evolution patterns: adding fields/subentities, modifying validation rules, trait evolution, phantom type changes, backward compatibility requirements
- [x] Create inventory document `_bmad-output/documentation-inventory/epic3-domain-entities.md` listing all entities with file locations, purposes, key methods, validation requirements

### Task 3: Create Domain Entity Documentation Structure
- [x] Create file `docs/domain-models.md` with title `# Epic 3 Domain Models`, 2-3 paragraph overview of all 4 bounded contexts, and table of contents
- [x] In `docs/domain-models.md`, establish documentation hierarchy with sections for each bounded context using the template: Overview, Structure, Rust-Specific Patterns Used, Validation Rules, Business Logic, Relationships, Evolution Guidelines
- [x] Add Documentation Standards section in `docs/domain-models.md` with naming conventions, documentation template description, and code example requirements
- [x] Create file `docs/domain-entity-template.md` containing the complete documentation template with fill-in-the-blank sections for all required information and example filled-out template
- [x] Add Splitting Criteria section in `docs/domain-models.md` with decision matrix: if total lines > 2000, bounded context section > 500 lines, readability affected, or specific context needs frequent updates, split to separate files
- [x] Monitor line count during documentation writing and decide on splitting if criteria are met

### Task 4: Document Note Bounded Context
- [x] Add `## Note Bounded Context` section to `docs/domain-models.md` with 2-3 paragraph overview explaining Note as main domain entity representing Obsidian notes
- [x] Write Structure subsection listing complete Note aggregate structure: Note entity fields (id, path, frontmatter, links, embeds, tags, headings, tasks, sections), all subentity structures with their fields and validation requirements
- [x] Write Rust-Specific Patterns Used subsection: UUID v7 identity, memory strategy (Box<str>, Arc<str>), immutability, type safety with enums, error handling with thiserror
- [x] Write Validation Rules subsection: path validation (non-empty, relative, .md extension), tag validation (regex, no empty segments), heading level validation (1-6), frontmatter date validation (ISO 8601), link/embed target validation
- [x] Write Business Logic subsection: semantic validation during construction, internal consistency checks, relationship invariants, vault context requirements
- [x] Write Relationships subsection: Note ↔ Config contract (defaults), Note ↔ Schema contract (validation), internal composition relationships
- [x] Write Evolution Guidelines subsection: adding new fields/subentities with defaults, modifying existing fields with migration paths, removing fields with deprecation periods
- [x] Add ASCII architecture diagram showing Note aggregate and all subentities with relationships
- [x] Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md` to ensure all entities and validation rules are documented

### Task 5: Document Schema Bounded Context
- [x] Add `## Schema Bounded Context` section to `docs/domain-models.md` with 2-3 paragraph overview explaining Schema for defining metadata structure and validation rules
- [x] Write Structure subsection listing complete Schema structure: Schema aggregate (name, extends, excludes, properties), Property entity (id, name, required, array, spec), PropertyBank singleton, PropertySpec trait and implementations (StringSpec, NumberSpec, etc.), inheritance resolution, trait-based generic design
- [x] Write Rust-Specific Patterns Used subsection: trait-based polymorphism with associated types, const generics for compile-time validation, trait objects for runtime flexibility, deterministic ID generation, zero-cost abstraction
- [x] Write Validation Rules subsection: schema name validation, property name validation, PropertySpec validation (string length/bounds/patterns, number ranges/step, bool/date/file constraints), inheritance cycle detection, excludes validation
- [x] Write Business Logic subsection: inheritance resolution algorithm, Property Bank lookup, override semantics, excludes processing, deterministic resolution, cycle detection with DFS
- [x] Write Relationships subsection: Schema ↔ PropertyBank (reusable properties), Schema ↔ Schema (inheritance), PropertyBank ↔ PropertySpec (validation implementations), Template ↔ Schema (variable constraints)
- [x] Write Evolution Guidelines subsection: adding PropertySpec variants, modifying constraints, inheritance changes, trait evolution, const generic changes
- [x] Add ASCII architecture diagram showing Schema, PropertyBank, and PropertySpec hierarchy with relationships
- [x] Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md` to ensure all entities and validation rules are documented

### Task 6: Document Config Bounded Context
- [x] **Write Config Bounded Context section in `docs/domain-models.md`**:
- [x] **Review against inventory**: Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md`

### Task 7: Document Template Bounded Context
- [x] Document Template entity with modular composition and variable definitions
- [x] Detail VariableDefinition enum variants with type constraints and defaults
- [x] Document TemplateComposition for inheritance and modular assembly
- [x] Include MiniJinja compatibility requirements and domain layer boundaries
- [x] Document Template relationships with Schema bounded context (variable validation)
- [x] If splitting occurs, create `docs/domain/template.md` for in-depth details

### Task 8: Create Bounded Context Interaction Contracts
- [x] Create Bounded Context Contracts section in `docs/domain-models.md`
- [x] Document Note ↔ Config Contract with sequence diagram
- [x] Document Note ↔ Schema Contract with sequence diagram
- [x] Document Template ↔ Schema Contract with sequence diagram
- [x] Document Template ↔ Config Contract with sequence diagram
- [x] Add contract evolution rules section

### Task 9: Create Evolution Guidelines and Architecture Diagrams
- [x] Create Epic 3 Architecture Diagrams section in `docs/domain-models.md`
- [x] Create Domain Model Evolution Guidelines section
- [x] Create dedicated evolution guidelines document if needed
- [x] Review all diagrams and guidelines for completeness

### Task 10: Validate Documentation Completeness and Quality
- [x] Cross-reference documentation against actual domain model implementations
- [x] Ensure all validation rules and business logic are documented
- [x] Validate that relationship contracts are clearly defined and actionable
- [x] Test documentation by having another developer use it to understand domain models
- [x] Incorporate feedback and iterate on documentation clarity

### Task 11: Project Lifecycle & Documentation Updates
- [x] **ROADMAP UPDATE:** Update `ROADMAP.md` to mark Epic 3 (Implement fundamental domain models) as complete in Milestone 1.
- [x] **CHANGELOG UPDATE:** Update `CHANGELOG.md` with Epic 3 highlights (Note aggregate, Schema system, Config hierarchy, Template composition).
- [x] Ensure all documentation accurately reflects the final state of the implementation.

### Task 12: Quality Assurance, Commit, and Remote Sync (MANDATORY FINAL TASK)
- [x] Run `mise run fmt` to format all code according to project standards
- [x] Run `mise run lint` to check for all code quality issues and anti-patterns
- [x] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [x] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [x] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [x] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [x] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [x] **MANDATORY:** Confirm all documentation meets quality standards and covers all requirements
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `docs: complete epic 3 documentation and finalize project artifacts`
- [ ] **REMOTE SYNC:** Push all changes to the remote branch `rust-conversion` using `git push`.
- [ ] **CI VERIFICATION:** Monitor and verify that all GitHub Action CI checks pass for the pushed changes.

## Dev Notes

### Epic 3 Domain Model Scope
- **Target Entities**: Note aggregate (with 7 subentities), Schema domain, Config hierarchy, Template composition system
- **Documentation Focus**: Developer understanding of domain language, relationships, and evolution rules
- **Quality Standard**: Enable developers to work effectively with domain models without user-facing knowledge
- **Architecture Context**: Hexagonal boundaries, bounded context interactions, domain purity requirements

### Previous Story Intelligence
**Story 3.4 Critical Architectural Lesson:**
- **MAJOR FIX**: MiniJinja syntax validation moved from domain to adapter layer
- **Domain Purity**: Domain entities store content as opaque strings, validation in adapters
- **Testing Implication**: Domain tests validate business rules only, no syntax concerns
- **Documentation Need**: Clear hexagonal boundaries and responsibility separation

**Story 3.3 Config Implementation Patterns:**
- **Hierarchical Structure**: Global → User → Project → Vault configuration layers
- **Validation Integration**: Semantic validation ensures configuration integrity
- **Error Handling**: thiserror::Error for typed configuration validation errors

**Story 3.2 Schema Complexity:**
- **PropertyBank Registry**: Singleton for reusable property definitions
- **Inheritance System**: Extends/Excludes for schema composition
- **Type Safety**: PropertySpec variants with specific validation rules

**Story 3.1 Note Aggregate Foundation:**
- **Rich Subentities**: Frontmatter, Links, Embeds, Tags, Headings, Tasks, Sections
- **Semantic Validation**: Internal consistency validation per entity
- **Vault Integration**: Path resolution and wiki-link handling

### Architecture Documentation Requirements
**Progressive Documentation Strategy:**
- **Power Users**: Get API docs + advanced guides with implementation details
- **New Users**: Get guided tutorials with concrete examples
- **Documentation as Code**: mdBook format with concrete outcomes focus
- **Migration Guides**: Critical for adoption and understanding evolution

**Hexagonal Architecture Documentation:**
- **Domain Layer**: Pure business logic, no external dependencies, entity validation
- **Adapter Layer**: External integrations, I/O operations, framework-specific code
- **Ports**: Clean interfaces between domain and adapters
- **Contracts**: Clear interaction patterns between bounded contexts

### Git Intelligence Summary
**Documentation Patterns Established:**
- Comprehensive README.md created (Story 1.8)
- ADR process established (Story 1.7)
- Project roadmap with milestones (Story 1.9)
- Consistent commit messages with conventional format

**Code Quality Standards Applied:**
- Clippy cognitive complexity limits (<25)
- No unwrap/expect/panic in production code
- Comprehensive error handling with thiserror
- Proper trait implementations (From/TryFrom, not ad-hoc methods)

## Technical Requirements

### Documentation Structure and Standards

**Documentation Structure (Single File Primary):**
- **Primary Approach**: Single comprehensive `docs/domain-models.md` file
- **Splitting Criteria**: Only split if file exceeds 2000 lines OR becomes hard to maintain/read
- **Split Structure**: If split occurs, overview in `docs/domain/overview.md` with essential information
- **Specific Files**: In-depth technical details in `docs/domain/` directory with clear cross-references
- **Leverage Existing**: Use existing `architecture.md` for hexagonal boundaries (no separate file needed)

**Single File Organization:**
```markdown
# Epic 3 Domain Models

## Overview
[Essential information about all 4 bounded contexts]

## Note Bounded Context
[Complete Note documentation with all entities and relationships]

## Schema Bounded Context
[Complete Schema documentation with inheritance and validation]

## Config Bounded Context
[Complete Config documentation with hierarchy and contracts]

## Template Bounded Context
[Complete Template documentation with composition and MiniJinja integration]

## Bounded Context Contracts
[All inter-context interaction contracts and evolution guidelines]

## Architecture Diagrams
[Visual representations of relationships and boundaries]
```

**Splitting Strategy (If Needed):**
```
docs/
├── domain-models.md               # Single comprehensive file (primary approach)
└── domain/
    ├── overview.md                # Essential information about all bounded contexts
    ├── note.md                    # Note bounded context in-depth technical details
    ├── schema.md                  # Schema bounded context in-depth technical details
    ├── config.md                  # Config bounded context in-depth technical details
    ├── template.md                # Template bounded context in-depth technical details
    └── evolution-guidelines.md    # Domain model evolution patterns

# Existing file (minimal update):
_bmad-output/planning-artifacts/architecture.md  # [MINIMAL UPDATE] Hexagonal boundaries documentation
```

**Splitting Strategy (If Needed):**
```
docs/
├── domain-models.md               # Single comprehensive file (primary approach)
├── domain/
│   ├── overview.md                # Essential information about all bounded contexts
│   ├── note.md                    # Note bounded context in-depth technical details
│   ├── schema.md                  # Schema bounded context in-depth technical details
│   ├── config.md                  # Config bounded context in-depth technical details
│   └── template.md                # Template bounded context in-depth technical details
└── domain/
    └── evolution-guidelines.md    # Domain model evolution patterns
```

**Entity Documentation Template:**
```markdown
# [Entity Name] Entity

## Overview
[Purpose and business context]

## Structure
[Fields, types, relationships]

## Rust-Specific Patterns Used
- **Const Generics**: [If used for compile-time validation]
- **Phantom Types**: [If used for context safety]
- **Associated Types**: [If used in ports/interfaces]
- **Memory Optimization**: [Box<str>, Arc<str> usage]
- **Virtual Clock**: [Use of time_test! for deterministic time logic]
- **Domain Purity**: [Enforcement via Domain Purity Guardian]
- **Factory Macros**: [Use of test_builder! for domain fixtures]

## Validation Rules
[Semantic validation requirements]

## Business Logic
[Key business rules and invariants]

## Relationships
[Connections to other entities/contexts]

## Evolution Guidelines
[When/how to modify this entity]
```

**Advanced Rust Patterns Documentation:**
- **Const Generics**: Used in PropertySpec for compile-time constraint validation (e.g., StringSpec<200>)
- **Phantom Types**: Employed in Config hierarchies to prevent context mixing at compile-time
- **Associated Types**: Used in repository ports for type-safe query results and error handling
- **Memory Optimization**: Arc<str> for shared strings, Box<str> for heap-efficient storage

### Domain Entity Coverage Requirements

**Note Bounded Context:**
- **Note Aggregate**: Main entity with identity, metadata, and content
- **Subentities**: Frontmatter (YAML), Links (wiki-links), Embeds, Tags, Headings, Tasks, Sections
- **Validation**: Semantic consistency, vault-relative paths, wiki-link resolution
- **Contracts**: Uses Config for metadata, Schema for validation, Template for rendering

**Schema Bounded Context:**
- **Schema Entity**: Name, Extends, Excludes, Properties arrays
- **PropertyBank**: Singleton registry for reusable Property definitions
- **Property Entity**: ID, Name, Required, Array, Spec (deterministic hashing)
- **PropertySpec Variants**: String (enum/pattern), Number (range), Bool (simple), Date (format), File (types)
- **Contracts**: Provides validation rules for Template variables and Note frontmatter

**Config Bounded Context:**
- **Hierarchical Structure**: Global → User → Project → Vault precedence
- **Validation**: Type safety, integrity checks, merge conflict resolution
- **Contracts**: Provides configuration for Note metadata, Template rendering, Schema defaults

**Template Bounded Context:**
- **Template Entity**: UUID identity, content storage, variable definitions
- **VariableDefinition Enum**: String/Number/Bool/Date/File with constraints
- **TemplateComposition**: Extends/includes with dependency resolution
- **MiniJinja Compatibility**: Domain stores opaque content, adapter handles syntax
- **Contracts**: References Schema for variable validation, uses Config for rendering parameters

### Bounded Context Interaction Contracts

**Note ↔ Config Contract:**
- Note uses Config for frontmatter defaults and metadata validation
- Config provides hierarchical fallback for missing note properties
- Contract: Config changes don't break existing Note validation

**Note ↔ Schema Contract:**
- Note frontmatter validated against linked Schema
- Schema provides property definitions and validation rules
- Contract: Schema evolution maintains backward compatibility

**Template ↔ Schema Contract:**
- Template variables validated against Schema property specs
- Schema provides type information and constraints for template variables
- Contract: Schema changes reflected in template validation

**Template ↔ Config Contract:**
- Config provides template execution parameters and defaults
- Template uses Config for rendering context and environment settings
- Contract: Config changes don't break template rendering

### Evolution Guidelines

**Entity Modification Rules:**
- **Add New Fields/Properties**: Always backward compatible, optional with defaults
- **Modify Existing Fields**: Requires migration path and backward compatibility
- **Remove Fields/Properties**: Only after deprecation period, with clear migration guide
- **Change Validation Rules**: Must maintain existing valid data validity

**Bounded Context Evolution:**
- **Contract Changes**: Require coordinated updates across interacting contexts
- **Breaking Changes**: Document migration path and timeline
- **New Contexts**: Define clear interaction contracts with existing contexts
- **Versioning**: Use semantic versioning for domain model changes

**Hexagonal Architecture Evolution:**
- **Domain Changes**: Can be breaking if contracts change
- **Adapter Changes**: Should be non-breaking for domain consumers
- **Port Changes**: Require coordinated updates across domain and adapters
- **New Ports**: Define clear responsibilities and interaction patterns

## Senior Developer Review (AI)

**Date:** 2026-01-21
**Reviewer:** OpenCode

**Summary:**
- Corrected Config bounded context terminology and added TrustedVaults coverage.
- Added bounded-context contract diagrams alongside existing architecture diagrams.
- Added missing doc-test examples for TemplateMetadata and TaskStatus.

**Status:** Changes requested addressed; documentation aligns with ACs.

## Change Log
- 2026-01-21: Documented TrustedVaults/SettingValue corrections, added contract diagrams, and added doc-test examples for TemplateMetadata and TaskStatus.

## Dev Agent Record

### Agent Model Used

OpenCode

### Debug Log References

- `mise run fmt`
- `mise run lint`
- `mise run verify`
- `pre-commit run --all-files`

### Completion Notes List
- ✅ Reviewed Epic 3 domain model sources across Note, Schema, Config, Template, and shared validation utilities.
- ✅ Authored `docs/domain-models.md` with bounded context documentation, contracts, diagrams, evolution guidelines, anti-patterns, and ADR mapping.
- ✅ Created `docs/domain-entity-template.md` for consistent entity documentation.
- ✅ Built inventory at `_bmad-output/documentation-inventory/epic3-domain-entities.md` mapping entities, files, purposes, methods, validation rules.
- ✅ Updated `ROADMAP.md` and `CHANGELOG.md` to reflect Epic 3 documentation completion.

### File List
- _bmad-output/documentation-inventory/epic3-domain-entities.md
- docs/domain-entity-template.md
- docs/domain-models.md
- ROADMAP.md
- CHANGELOG.md
- _bmad-output/implementation-artifacts/sprint-status.yaml
- crates/domain/src/note/task.rs
- crates/domain/src/template/aggregate.rs
