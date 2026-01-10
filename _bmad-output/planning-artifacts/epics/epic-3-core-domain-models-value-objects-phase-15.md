# Epic 3: Core Domain Models & Value Objects **[PHASE 1.5]**

Developers have a clear, shared domain language with rich domain models that embody business rules and validation logic, informed by Obsidian patterns and Go implementation lessons learned.
**FRs covered:** Architecture requirements (DDD domain models)
**Implementation Notes:**
- Core stable models: Config, Schema, Note, Frontmatter, Template + value objects
- Models informed by Obsidian structures (TFile, CachedMetadata) and Go implementation
- Flexibility for Rust-specific refinements and supplementary models in later epics
- Mocks for domain interfaces created as needed (not upfront)

## Story 3.1: Create Note Bounded Context

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

## Story 3.2: Create Schema Bounded Context

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

## Story 3.3: Create Config Bounded Context

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

## Story 3.4: Create Template Bounded Context

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

## Story 3.5: Review Epic 3 Test Suite for Efficiency

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

## Story 3.6: Create Epic 3 Documentation

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
