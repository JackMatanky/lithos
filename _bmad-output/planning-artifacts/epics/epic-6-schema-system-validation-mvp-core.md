# Epic 6: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**
- SchemaPort and mocks created in this epic
- Sample schema files created from docs/schemas/ JSON examples using Epic 4 loading foundation
- Schema validation (syntactic in adapter, semantic in domain)
- Schema-template integration contracts defined
- User documentation for schema creation

## Story 6.1: Create Schema Domain Interface and Port

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

## Story 6.2: Create Schema Property System

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

## Story 6.3: Implement Schema Loading with $ref Resolution

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

## Story 6.4: Implement Schema Inheritance Resolution

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

## Story 6.5: Add Schema Validation and Error Handling

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

## Story 6.6: Create Sample Schema Files

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

## Story 6.7: Define Schema-Template Integration Contracts

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

## Story 6.8: Review Epic 6 Test Suite

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

## Story 6.9: Document Schema System for Users

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
