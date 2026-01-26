# Epic 12: Basic Interactive Template System **[MVP CORE]**

Users can create and execute modular templates with schema-driven interactive prompts that generate validated notes with essential template functions.
**FRs covered:** FR1, FR2, FR9, FR15, FR16
**Implementation Notes:**

- TemplatePort, UIPort and mocks created in this epic
- MiniJinja integration per ADR 0003
- Sample templates from docs/refs/obsidian/ converted as test fixtures
- Schema-driven inputs (enums → suggesters)
- User documentation for basic template creation
- Performance benchmarking for NFR1 validation (<500ms execution)
- May create ADR for interactive UI patterns

### Story 12.1: [Domain] Unified Prompt, Suggestion, and Source Models

As a developer, I want domain entities that represent template variables and suggestion sources, so that the elicitation logic supports both simple lists and complex key-value maps.
**Acceptance Criteria:**

- **Given** the `domain` crate
- **When** I define the elicitation models
- **Then** `Suggestion` supports `display_text` (what the user sees) and `value` (what the template receives).
- **And** `ElicitationSource` supports `Static`, `DynamicQuery`, and `SchemaDerived` variants.
- **And** the `UIPort` trait is designed to return the complex `value` type.
  **References:** FR16, FR17

### Story 12.2: [Adapters/API] Basic Suggesters with List and Mapping Support

As a template author, I want basic suggesters that can accept both simple arrays/lists and key-value mappings, so that I can create interactive templates without requiring schema definitions.
**Acceptance Criteria:**

- **Given** a template with a `suggest()` call
- **When** I pass an array like `["Option 1", "Option 2", "Option 3"]`
- **Then** the suggester displays these options and returns the selected string value.
- **And** when I pass a mapping like `{display: "Option 1", value: "opt1"}`
- **Then** the suggester displays the "display" text but returns the "value" for template use.
- **And** the basic suggester works independently of schema definitions.
  **References:** FR16, FR17

### Story 12.3: [App] Schema-Driven Query Automation & Binding

As a template author, I want the schema to automatically simplify my queries, so that I don't have to manually write folder-listing logic in every template.
**Acceptance Criteria:**

- **Given** a schema property with a `FileSpec` and a `directory` constraint
- **When** a template variable is bound to this property
- **Then** the system automatically generates a "List Files" query for that directory to populate the suggester.
- **And** the `BindingService` raises a `miette` warning for variables missing from the schema while allowing them to proceed as simple string prompts.
- **And** schema metadata (enums, descriptions) automatically enriches the prompt definitions.
  **References:** FR9, FR12, FR14

### Story 12.4: [App] Schema-Driven Template Features & Dynamic Context Resolution

As a user, I want my suggesters populated by automated schema queries and templates to leverage schema constraints,
So that I can pick from up-to-date vault data with schema-driven filtering and validation.

**Acceptance Criteria:**

### **Schema-to-Suggester Mapping:**

**Given** a template variable is bound to a schema property
**When** property has PropertySpec::File with constraints
**Then** FileSpec.directory constraint auto-generates folder query
**And** FileSpec.file_class constraint filters to notes matching schema name
**And** example: `file_class: "task_project"` → only shows notes with `fileClass: "task_project"`
**And** example: `directory: "(41_personal|42_education)/"` → only shows notes in matching directories

**Given** a schema property has PropertySpec::String with enum
**When** template variable is bound to this property
**Then** enum values auto-populate suggester options
**And** example: `enum: ["to_do", "in_progress", "done"]` → suggester shows these 3 options
**And** user can only select from enum list (validated)

**Given** schema properties define validation constraints
**When** template prompts for user input
**Then** constraints are enforced during input collection
**And** number properties enforce min/max bounds during input
**And** date properties enforce format during input
**And** pattern constraints provide validation feedback

### **Dynamic Context Resolution:**

**Given** a `PromptSession`
**When** the resolution service runs
**Then** it identifies all required vault queries (schema-derived + template-explicit)
**And** it executes queries in parallel before prompting user
**And** it caches query results for session duration (performance)

**Given** schema-derived queries
**When** resolution service processes property bindings
**Then** FileSpec properties generate "List Files" queries automatically
**And** queries include directory filters and file_class filters
**And** results are pre-fetched before first prompt

**Given** query results with display vs value distinction
**When** resolution service formats results
**Then** it handles map-like results (e.g., Note Title vs. File Path)
**And** suggester displays human-readable text (title)
**And** template receives machine-usable value (file path)

### **Default Value Propagation:**

**Given** schema property has default value (future enhancement - not in current PropertySpec)
**When** template variable is bound to this property
**Then** default value is available in template context (placeholder for future)
**And** missing frontmatter fields use schema defaults (placeholder for future)

### **Type-Safe Validation:**

**Given** template variable is bound to schema property
**When** user provides input
**Then** input is validated against PropertySpec constraints
**And** string pattern validation runs before accepting input
**And** number range validation runs before accepting input
**And** enum validation ensures value is in allowed list
**And** validation errors display helpful messages with retry option

### **Schema-Template Contract:**

**Given** templates need to query schema constraints
**When** template rendering occurs
**Then** SchemaQuery port provides schema metadata access
**And** template can query property constraints for variables
**And** template can check if property is required, array, etc.
**And** BindingService (Story 12.3) handles schema lookup and variable binding

### **Integration with BindingService (Story 12.3):**

**Given** BindingService binds variables to schema properties
**When** template declares variable with schema reference
**Then** BindingService looks up property from SchemaQuery
**And** extracts constraints (enum, directory, file_class, etc.)
**And** generates appropriate prompt configuration (suggester vs text input)
**And** raises miette warning if variable not found in schema (allows proceeding as string prompt)

### **Fallback Behavior:**

**Given** schema-driven features are unavailable (schema not found, schema load error)
**When** template requires user input
**Then** system falls back to basic prompt() function (free text input)
**And** user is informed of degraded operation mode ("Schema not found, using manual input")
**And** template completion remains achievable without schema

  **References:** FR2, FR9, FR11, FR12, FR14, FR23

### Story 12.5: [App] Interactive Loop Orchestrator (Atomic Workflow)

As a user, I want the system to ensure my vault remains clean if I cancel a template execution, so that I don't have to manually delete partial or empty files.
**Acceptance Criteria:**

- **Given** an active elicitation session
- **When** the user sends an `Abort` signal (Ctrl-C) or an error occurs
- **Then** the service terminates immediately and ensures no files are written to the vault.
- **And** all intermediate prompt data is purged from memory.
- **And** the orchestrator ensures the "Clean Slate" policy is respected across all execution steps.
  **References:** FR24, FR49

### Story 12.6: [Adapters/SPI] MiniJinja Variable Inspector & Extensions

As the system, I need to discover template requirements and provide a way for authors to trigger custom suggesters, so that the elicitation process is both automated and flexible.
**Acceptance Criteria:**

- **Given** a markdown template
- **When** the `TemplateInspector` runs
- **Then** it uses MiniJinja AST traversal to find all undeclared variables without performing a full render.
- **And** it registers a `suggest(options)` global function in the MiniJinja environment that can trigger the `UIPort` with custom data.
  **References:** FR1, FR6

### Story 12.7: [Adapters/API] Fuzzy Picker with Key-Value Support

As a user, I want a beautiful and responsive fuzzy-search interface in my terminal, so that I can find and select options quickly.
**Acceptance Criteria:**

- **Given** a list of `Suggestions` from the App layer
- **When** the `UIPort` implementation (via `inquire` or `dialoguer`) is called
- **Then** it renders a fuzzy-searchable list in the terminal.
- **And** searching only filters by the `display_text`, but the component returns the internal `value` upon selection.
- **And** it correctly captures terminal interrupt signals and returns an `Abort` signal to the App layer.
  **References:** FR15, FR16

### Story 12.8: [Test] Obsidian Templater Template Conversion & Fixtures

As a developer, I want to use real-world Obsidian templates as test fixtures, so that I can verify Lithos provides a viable migration path for power users.
**Acceptance Criteria:**

- **Given** the templates in `docs/refs/obsidian/00_system/`
- **When** I convert `42_00_action_item.md` to Lithos format
- **Then** the Lithos implementation must achieve the same metadata generation and file placement as the original.
- **And** the automated schema-derived queries must match the output of the original manual Javascript queries.
  **References:** NFR20

### Story 12.9: [Adapters/SPI] Chrono Date/Time Function Integration

As a template author, I want access to date/time functions using the existing chrono crate, so that I can format dates and perform date arithmetic without adding new dependencies.
**Acceptance Criteria:**

- **Given** the chrono crate is already in the tech stack
- **When** I integrate date functions into MiniJinja
- **Then** templates can use `date_now()`, `date_format()`, and `date_add()` functions.
- **And** functions follow chrono API patterns (not moment.js).
- **And** all functions are documented with examples in the standard library reference.
  **References:** FR4, ADR 0003

### Story 12.10: [Adapters/SPI] Convert Case String Function Integration

As a template author, I want string case conversion functions using the convert_case crate, so that I can generate proper identifiers and titles without custom implementations.
**Acceptance Criteria:**

- **Given** the convert_case crate is available
- **When** I integrate case functions into MiniJinja
- **Then** templates can use `str_title_case()`, `str_snake_case()`, `str_kebab_case()`, etc.
- **And** functions handle Unicode properly and follow convert_case API patterns.
- **And** functions are documented with examples in the standard library reference.
  **References:** FR1

### Story 12.11: [Adapters/SPI] Slug Generation Function Integration

As a template author, I want URL-friendly slug generation using the slug crate, so that I can create valid identifiers for file names and URLs.
**Acceptance Criteria:**

- **Given** a slug crate is available (str_slug or similar)
- **When** I integrate slug functions into MiniJinja
- **Then** templates can use `str_slug()` to generate URL-safe identifiers.
- **And** the function handles Unicode, removes special characters, and replaces spaces with hyphens.
- **And** the function is documented with examples in the standard library reference.
  **References:** FR1

### Story 12.12: [Adapters/SPI] Base64 Encoding Function Integration

As a template author, I want base64 encoding/decoding functions using the base64 crate, so that I can encode binary data or create compact representations.
**Acceptance Criteria:**

- **Given** the base64 crate is available
- **When** I integrate base64 functions into MiniJinja
- **Then** templates can use `base64_encode()` and `base64_decode()` functions.
- **And** functions support standard base64 encoding/decoding.
- **And** functions are documented with examples in the standard library reference.
  **References:** Additional utility functions

### Story 12.13: [Adapters/SPI] Random Value Generation Integration

As a template author, I want random value functions using the rand crate, so that I can generate random numbers, strings, or selections for testing and variety.
**Acceptance Criteria:**

- **Given** the rand crate is available
- **When** I integrate random functions into MiniJinja
- **Then** templates can use `rand_int()`, `rand_float()`, and `rand_choice()` functions.
- **And** functions use cryptographically secure random generation where appropriate.
- **And** functions are documented with examples in the standard library reference.
  **References:** Additional utility functions

### Story 12.14: [Adapters/SPI] UUID Generation Function Integration

As a template author, I want UUID generation using the existing uuid crate, so that I can create unique identifiers for files and records.
**Acceptance Criteria:**

- **Given** the uuid crate is already in the tech stack
- **When** I integrate UUID functions into MiniJinja
- **Then** templates can use `uuid_v7()` to generate time-ordered unique identifiers.
- **And** the function follows the UUID v7 specification for sortability.
- **And** the function is documented with examples in the standard library reference.
  **References:** Additional utility functions

### Story 12.15: Template System Resource Limits and Timeouts

As a system administrator, I want template execution to be bounded by resource limits and timeouts, so that runaway templates cannot exhaust system resources or hang indefinitely.
**Acceptance Criteria:**
**Given** template execution starts
**When** resource limits are exceeded
**Then** execution is terminated gracefully with clear error messages
**And** partial outputs are cleaned up automatically
**And** system resources are protected from exhaustion

**Given** template operations run
**When** timeouts are exceeded
**Then** operations are cancelled with rollback to clean state
**And** users receive actionable timeout messages
**And** long-running operations provide progress indicators

### Story 12.16: Template System Fallback Strategies

As a user experiencing template failures, I want automatic fallback mechanisms, so that template operations degrade gracefully rather than failing completely.
**Acceptance Criteria:**
**Given** advanced template features fail
**When** fallback mechanisms activate
**Then** operations continue with basic functionality
**And** users are informed of degraded operation mode
**And** full functionality is restored when issues are resolved

**Given** schema-driven features are unavailable
**When** templates require user input
**Then** they fall back to basic prompt() functions
**And** manual data entry remains possible
**And** template completion is still achievable

### Story 12.17: MiniJinja Template Performance Regression Testing

As a performance engineer, I want automated regression tests for MiniJinja template operations, so that the architectural choice of MiniJinja remains optimal and template execution stays under 500ms NFR1.
**Acceptance Criteria:**
**Given** MiniJinja template implementation
**When** performance regression tests run
**Then** template rendering benchmarks are compared against 450μs baseline
**And** complex template execution is validated under 500ms limit
**And** template compilation performance regressions trigger alerts
**And** template benchmarks run in CI/CD for every template-related change

### Story 12.18: Review Epic 12 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 12 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 12 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 12 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** the implementation of Epic 12
**When** I run the test suite
**Then** it achieves 90%+ coverage for the `PromptSession` state machine and `BindingService`
**And** property-based tests verify that `Abort` signals never result in filesystem side-effects
**And** the suite validates architectural boundaries (e.g. Domain has zero I/O)

**Given** all Epic 12 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 12 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

**References:** NFR16

### Story 12.19: Epic 12 User and Developer Documentation

As a user, I want clear instructions on how to create and use interactive templates with schema support, so that I can leverage the full power of the system.

**Acceptance Criteria:**

**Given** a completed Epic 12
**When** I review the documentation
**Then** it includes a guide on how schemas automate folder-picking queries
**And** it provides examples for using the `suggest()` helper for ad-hoc terminal prompts
**And** it explains the "Clean Slate" policy and how to recover from errors
**And** it lists all available standard library functions with usage examples
**And** it documents resource limits, timeouts, and fallback behaviors

**References:** NFR13
