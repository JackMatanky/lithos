## Epic 6: Schema System **[MVP CORE]**

Users can define metadata schemas with field types, create schema-driven templates, validate notes against schemas, and use schema features like enums and inheritance.
**FRs covered:** FR8-FR14
**Implementation Notes:**
- Schema definition with field types
- Schema-driven templates
- Note validation against schemas
- Schema enums for suggesters
- File filtering with schema constraints
- Date formatting via schemas
- Schema inheritance

### Story 6.1: Implement Schema Definition DSL

As a developer defining metadata schemas,
I want a DSL for defining schemas with field types,
So that schemas can be created programmatically with proper type safety.

**Acceptance Criteria:**

**Given** the schema DSL is implemented
**When** I define a schema
**Then** I can specify field types: string, number, date, file, boolean

**Given** schema fields are defined
**When** I validate the schema
**Then** type constraints are enforced

**Given** invalid schema definitions exist
**When** I parse schemas
**Then** clear error messages indicate the issues

### Story 6.2: Create Schema-Driven Template Engine

As a developer creating templates,
I want templates driven by schema properties,
So that template inputs are automatically generated from schema definitions.

**Acceptance Criteria:**

**Given** a schema is defined
**When** I create a template
**Then** template inputs are automatically derived from schema fields

**Given** schema-driven templates are used
**When** I execute templates
**Then** input prompts match schema field types and constraints

**Given** schema changes
**When** I update templates
**Then** template inputs automatically reflect schema changes

### Story 6.3: Implement Note Validation Against Schemas

As a developer validating notes,
I want notes validated against schemas,
So that metadata consistency is enforced with clear error feedback.

**Acceptance Criteria:**

**Given** notes have frontmatter
**When** I validate against a schema
**Then** all required fields are present and correctly typed

**Given** validation fails
**When** I check errors
**Then** clear, actionable error messages are provided

**Given** multiple schemas exist
**When** I validate notes
**Then** the correct schema is selected based on note metadata

### Story 6.4: Add Schema Enum Support for Suggesters

As a developer using enums in schemas,
I want enum values used in suggesters,
So that template inputs provide controlled choice lists.

**Acceptance Criteria:**

**Given** schema fields have enums
**When** I create suggesters
**Then** enum values populate the suggestion list

**Given** enum suggesters are used
**When** I select values
**Then** only valid enum values are accepted

**Given** enum values change
**When** I update schemas
**Then** suggesters automatically reflect the changes

### Story 6.5: Implement File Filtering with Schema Constraints

As a developer selecting files,
I want schema-defined directory constraints,
So that file selections are limited to appropriate locations.

**Acceptance Criteria:**

**Given** schema fields have file type with directory constraints
**When** I create file selectors
**Then** only files in specified directories are shown

**Given** directory constraints are set
**When** I browse files
**Then** constrained file lists are provided

**Given** constraints change
**When** I update schemas
**Then** file filtering automatically adapts

### Story 6.6: Add Date Formatting via Schema Format Strings

As a developer formatting dates,
I want schema-defined format strings,
So that dates are consistently formatted across templates.

**Acceptance Criteria:**

**Given** schema date fields have format strings
**When** I format dates
**Then** the specified format is used

**Given** format strings are invalid
**When** I validate schemas
**Then** format string syntax is checked

**Given** dates are formatted
**When** I display them
**Then** consistent formatting is applied

### Story 6.7: Implement Schema Inheritance and Extension

As a developer reusing schemas,
I want inheritance and extension capabilities,
So that common field definitions can be shared and specialized.

**Acceptance Criteria:**

**Given** base schemas exist
**When** I create derived schemas
**Then** inheritance works with Extends and Excludes

**Given** inherited schemas are used
**When** I validate notes
**Then** combined field sets are applied

**Given** inheritance chains are complex
**When** I resolve schemas
**Then** proper field resolution occurs without conflicts
