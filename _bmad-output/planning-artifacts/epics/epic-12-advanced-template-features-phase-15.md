# Epic 12: Advanced Template Features **[PHASE 1.5]**
Users can compose complex templates with date functions, multi-suggesters, and error prevention for production-ready template workflows.
**FRs covered:** FR3, FR4, FR17
**Implementation Notes:**
- Extends Epic 10 template system (not replacement)
- Date formatting with chrono (Rust-native)
- Template composition patterns
- User documentation for advanced template features
- Performance validation for complex templates

### Story 12.1: [Domain] Template Dependency & Recursion Models
As a developer, I want to represent template relationships in the domain, so that I can detect circular dependencies and missing files before execution.
**Acceptance Criteria:**
- **Given** the `domain` crate
- **When** I define the `TemplateGraph` model
- **Then** it can represent parent-child relationships between templates via `include` statements.
- **And** the `CycleDetector` service can identify infinite recursion paths in a graph of template paths.
- **And** rich domain errors are defined for `CircularDependency` and `MissingPartial`.
**References:** FR3

### Story 12.2: [Domain] TemplateDate Value Object
As a template author, I want a robust date domain model, so that I can perform reliable date math and formatting in my templates.
**Acceptance Criteria:**
- **Given** a date input
- **When** I create a `TemplateDate` value object
- **Then** it supports operations like `add_days(n)`, `subtract_days(n)`, and `format(String)`.
- **And** it correctly handles leap years and timezone offsets.
- **And** it is serializable for use in the template rendering context.
**References:** FR4

### Story 12.3: [App] Template Composition "Dry Run" Orchestrator
As a user, I want the system to verify my template structure before asking for input, so that I don't waste time on a session that will fail due to a missing file.
**Acceptance Criteria:**
- **Given** a template execution request
- **When** the `DryRunService` runs
- **Then** it recursively parses the AST of the root template and all included partials.
- **And** if any partial is missing or a cycle is detected, it returns a `miette` diagnostic immediately.
- **And** this check must pass before the first prompt is displayed to the user.
**References:** FR3, FR48

### Story 12.4: [App] Context-Aware Format Sensing Service
As a template author, I want the system to automatically format array variables based on their position in the file, so that my frontmatter remains valid YAML while my content remains readable markdown.
**Acceptance Criteria:**
- **Given** a rendering session
- **When** a variable is rendered between the `---` YAML delimiters
- **Then** the system automatically applies the `yaml_array_style` (Block vs. Flow) from the configuration.
- **And** it applies YAML-safe escaping to string values.
- **And** variables rendered outside the delimiters default to standard Markdown formatting.
**References:** FR17, FR26

### Story 12.5: [Adapters/SPI] Chrono-based Natural Language Date Adapter
As a user, I want to provide relative dates like "tomorrow" or "next Friday" in my prompts, so that I can create notes for future events easily.
**Acceptance Criteria:**
- **Given** a natural language string from a prompt
- **When** the `DateResolutionAdapter` is called
- **Then** it resolves the string into a concrete timestamp using `chrono-english`.
- **And** it respects the `timezone` and `first_day_of_week` settings from the vault configuration.
- **And** it provides a fallback to the current date if the input is ambiguous.
**References:** FR4

### Story 12.6: [Adapters/API] Multi-Select Terminal UI
As a user, I want to select multiple items from a list using a fuzzy-searchable terminal picker, so that I can quickly populate array fields like tags or contacts.
**Acceptance Criteria:**
- **Given** a list of suggestions
- **When** the `UIPort::prompt_multi_select` is called
- **Then** it renders a picker allowing the user to toggle multiple items (e.g., using spacebar).
- **And** it supports fuzzy-searching the display labels.
- **And** it returns a collection of the internal `values` for the selected items.
**References:** FR17

### Story 12.7: [Test] Epic 12 Test Suite Review & Optimization
As a developer, I want a comprehensive and efficient test suite for the advanced template features, so that I can maintain the code with confidence.
**Acceptance Criteria:**
- **Given** the implementation of Epic 12
- **When** I run the test suite
- **Then** it achieves 90%+ coverage for the `TemplateGraph`, `TemplateDate`, and composition services.
- **And** property-based tests verify that circular dependency detection works for complex template hierarchies.
- **And** the suite validates that date operations handle edge cases (leap years, timezone boundaries).
**References:** NFR16

### Story 12.8: [Docs] Epic 12 User & Developer Documentation
As a user, I want clear instructions on how to use advanced template features like composition and date functions, so that I can create sophisticated template workflows.
**Acceptance Criteria:**
- **Given** a completed Epic 12
- **When** I review the documentation
- **Then** it includes examples of template composition with includes and cycles.
- **And** it provides natural language date parsing examples.
- **And** it explains multi-select suggester usage for array fields.
- **And** it documents the dry-run validation process.
**References:** NFR13
