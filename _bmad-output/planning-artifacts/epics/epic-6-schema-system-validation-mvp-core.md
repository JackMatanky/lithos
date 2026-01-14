# Epic 6: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**
- SchemaPort and mocks created in this epic
- Sample schema files created from docs/schemas/ JSON examples using Epic 4 loading foundation
- Schema validation (syntactic in adapter, semantic in domain)
- Frontmatter-schema compliance validation (application layer, warnings-only)
- Schema-template integration contracts defined
- User documentation for schema creation
- Clear terminology: schema properties vs frontmatter fields to avoid Go implementation confusion

## Story 6.1: Implement Schema CQRS Ports from Epic 3

As a developer completing the schema bounded context,
I want to implement the SchemaCommand and SchemaQuery ports defined in Epic 3,
So that schema operations follow CQRS separation with proper command/query handling.

**Acceptance Criteria:**

**Given** Epic 3 defined SchemaCommand and SchemaQuery trait interfaces
**When** I implement the concrete ports
**Then** SchemaCommand handles schema creation, updates, and $ref resolution

**Given** SchemaCommand is implemented
**When** I validate command operations
**Then** it supports loading JSON schemas, resolving inheritance, and storing validated schemas

**Given** SchemaQuery is implemented
**When** I validate query operations
**Then** it supports retrieving schemas by name and validating notes against schemas

**Given** both ports are implemented
**When** I test integration
**Then** commands and queries work together for complete schema management

## Story 6.2: Create Schema Domain Interface and Port

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

## Story 6.3: Create Schema Property System

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

## Story 6.4: Implement Schema Loading with $ref Resolution

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

## Story 6.5: Implement Schema Inheritance Resolution

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

## Story 6.6: Add Schema Validation and Error Handling

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

## Story 6.7: Create Sample Schema Files

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

## Story 6.8: Define Schema-Template Integration Contracts

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

## Story 6.9: Implement Frontmatter-Schema Compliance Validation

As a developer ensuring vault consistency,
I want frontmatter-schema compliance validation as an application service,
So that notes can be validated against their corresponding schemas for caching and querying improvements.

**Acceptance Criteria:**

**Given** notes have frontmatter with file class keys
**When** I implement compliance validation service
**Then** validation occurs between frontmatter fields and schema properties (not schema validation)

**Given** compliance validation service is triggered by events
**When** vault indexing runs
**Then** notes are validated for schema compliance with warnings logged

**Given** compliance validation service is triggered by events
**When** template checking runs ("doctor" command)
**Then** templates are validated for proper frontmatter usage

**Given** compliance validation runs
**When** frontmatter doesn't match schema
**Then** warnings are generated (non-blocking) for caching/querying consistency

**Given** Config defines file class keys
**When** compliance validation determines schema
**Then** only file class key from config is used (no other config influence on validation rules)

## Story 6.10: Review Epic 6 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 6 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** docs/testing/developer-guide.md provides testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, and utilities

**Given** all Epic 6 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 6 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

## Story 6.11: Document Schema System for Users

As a user creating schemas,
I want comprehensive documentation for schema creation and usage,
So that I can effectively define and use schemas in lithos.

**Acceptance Criteria:**

**Given** schema system is implemented
**When** I create user documentation
**Then** it includes all schema features: properties, inheritance, validation, frontmatter compliance, examples

**Given** documentation exists
**When** I check completeness
**Then** it covers all property types, inheritance patterns, and frontmatter compliance from docs/schemas/

**Given** users read the documentation
**When** they create schemas
**Then** they can define valid schemas and understand frontmatter compliance without developer assistance
