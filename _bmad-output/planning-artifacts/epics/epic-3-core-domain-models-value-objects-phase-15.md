# Epic 3: Core Domain Models & Value Objects **[PHASE 1.5]**

Developers have a clear, shared domain language with rich domain models that embody business rules and validation logic, with domain events and CQRS ports for future adapter implementation.
**FRs covered:** Architecture requirements (DDD domain models)
**Implementation Notes:**
- Core stable models: Config (Vault/Global merging), Schema (with PropertyBank), Note (aggregate), Template
- Models informed by Obsidian structures and Go implementation lessons learned
- Hexagonal architecture: Domain contains business logic, adapters handle I/O
- CQRS ports defined in domain, implemented by later epics (5,6,9)
- Single-file-per-context approach with 1000+ line splitting guideline
- Domain events for state changes and cross-context coordination
- Rich Domain Models: Entities and Value Objects enforce their own invariants through encapsulation and private fields.

## Story 3.1: Create Config Bounded Context

As a developer managing application configuration,
I want a Config domain model with Global → Vault hierarchy, optional schema versions, and flexible trusted vaults,
So that configuration supports quick use (no config required) while enabling advanced features for power users.

**Acceptance Criteria:**

**Given** I have researched configuration merging patterns
**When** I review the Config bounded context
**Then** Config supports merging VaultConfig and GlobalConfig with business rules (Vault > Global precedence)

**Given** Config entity is defined
**When** I check validation integration
**Then** semantic validation ensures configuration integrity and type safety

**Given** configuration patterns are established
**When** I validate the design
**Then** Config supports encrypted sensitive fields and validation rules

**Given** the Config bounded context is defined
**When** I check domain events
**Then** ConfigUpdated event is emitted for configuration changes

**Given** hierarchical merging is needed
**When** I implement merging in domain
**Then** vault-level config overrides global-level (Global → Vault two-tier system, optional global config)

**Given** schema versioning is needed
**When** I implement vault config
**Then** schema_version is optional and defaults to current Lithos binary version for quick use

**Given** vault naming is needed
**When** I implement vault metadata
**Then** vault name defaults to directory basename (e.g., `/vaults/work` → "work")

**Given** trusted vaults are needed
**When** I implement global config
**Then** trusted_vaults supports list format OR map format (not both) with validation

**Given** global templates are needed
**When** I implement global config
**Then** global config supports schemas_dir and templates_dir for global template library

**Given** CQRS separation is needed
**When** I define ports
**Then** ConfigCommand and ConfigQuery trait interfaces are provided for future implementation

## Story 3.2: Create Note Bounded Context

As a developer working with note data,
I want a comprehensive Note aggregate with all subentities and domain events,
So that the domain accurately represents the rich structure of notes in Obsidian vaults.

**Acceptance Criteria:**

**Given** I have researched Obsidian note structures and wiki-link patterns
**When** I review the Note bounded context
**Then** the Note aggregate includes these subentities:
- Note (main entity with identity and metadata)
- Frontmatter (YAML metadata with fields and Config integration)
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

**Given** the Note bounded context is defined
**When** I check domain events
**Then** NoteCreated and NoteFrontmatterValidated events are emitted for note lifecycle

**Given** CQRS separation is needed
**When** I define ports
**Then** NoteCommand and NoteQuery trait interfaces are provided for future implementation

## Story 3.3: Create Schema Bounded Context

As a developer defining metadata schemas,
I want a complete schema domain with PropertyBank, Property, and PropertySpec variants with domain events,
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

**Given** the Schema bounded context is defined
**When** I check domain events
**Then** SchemaCreated and PropertyBankUpdated events are emitted for schema lifecycle

**Given** CQRS separation is needed
**When** I define ports
**Then** SchemaCommand and SchemaQuery trait interfaces are provided for future implementation

## Story 3.4: Create Template Bounded Context

As a developer working with template definitions,
I want a Template domain model with validation and domain events,
So that template structure and business rules are properly validated at the domain level.

**Acceptance Criteria:**

**Given** I have researched template engine patterns
**When** I review the Template bounded context
**Then** Template entity includes structure, syntax, and business rule validation

**Given** Template entity is defined
**When** I check semantic validation
**Then** template syntax, structure, and composition validation occurs internally

**Given** template patterns are established
**When** I validate the design
**Then** Template supports modular composition and variable definitions

**Given** the Template bounded context is defined
**When** I check domain events
**Then** TemplateCreated event is emitted for template lifecycle

**Given** CQRS separation is needed
**When** I define ports
**Then** TemplateCommand and TemplateQuery trait interfaces are provided for future implementation

## Story 3.5: Consolidate Domain Core and Internal Utilities

As a developer maintaining the domain layer,
I want to consolidate shared logic and internal utilities into a central `lib.rs` and core modules within the domain crate,
So that the codebase remains DRY, maintainable, and architectural boundaries are respected.

**Acceptance Criteria:**

**Given** redundant utility functions and shared logic exist across Note, Schema, Config, and Template bounded contexts
**When** I refactor the domain crate
**Then** shared logic is moved to `crates/domain/src/lib.rs` or internal core modules

**Given** internal domain utilities are exposed publicly
**When** I review visibility
**Then** internal utilities use `pub(crate)` to prevent leaking into the application layer

**Given** the domain crate is the "inviolate core"
**When** I consolidate utilities
**Then** ZERO external dependencies (except justified ones like `serde` or `thiserror`) are introduced

**Given** common patterns like UUID v7 handling or shared error mapping are used
**When** I implement core utilities
**Then** they are implemented once in the core and reused across all bounded contexts

## Story 3.6: Review Epic 3 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 3 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** docs/testing/developer-guide.md provides testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, and utilities

**Given** ADR 0013 controls domain serialization strategy
**When** I review domain entities
**Then** optional serde derives are allowed but rkyv derives are prohibited

**Given** all Epic 3 domain models are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests actually validate business requirements vs implementation details

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 3 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**Given** domain models evolve
**When** I update tests
**Then** test maintenance cost is <20% of development time

**Given** CQRS ports are defined
**When** I test port interfaces
**Then** trait interface tests validate correct signatures and contracts

## Story 3.7: Create Epic 3 Documentation

As a developer working with the domain models,
I want comprehensive documentation of the domain entities, their relationships, domain events, and evolution guidelines,
So that developers understand the domain language and can work effectively with the models.

**Acceptance Criteria:**

**Given** all Epic 3 domain models are implemented
**When** I create documentation
**Then** it includes developer-focused content:
- Domain entity relationships and bounded contexts
- Semantic validation rules for each entity
- Domain events and their purposes
- CQRS port interfaces and contracts
- Domain entity relationship contracts (how bounded contexts interact)
- Evolution guidelines for domain models (when to add vs modify entities)
- Architecture diagrams showing entity relationships, events, and ports

**Given** documentation is created
**When** I validate completeness
**Then** it covers all entities and their contracts: Config (Vault/Global merging), Note aggregate, Schema domain, Template

**Given** documentation exists
**When** I check relationship contracts
**Then** it defines how bounded contexts interact (e.g., Template references Schema, Note uses Config fileClass)

**Given** documentation exists
**When** a developer reads it
**Then** they understand domain evolution rules, event-driven architecture, and inter-entity contracts without needing user-facing knowledge

**Given** Epic 3 domain models are implemented
**When** I review the source code
**Then** all files in the domain (`crates/domain/src/`) are reviewed thoroughly to ensure full and proper use of doc comments with doc tests and that every relevant component has a well written doc comment.
