# Epic 13: Advanced Template Features **[PHASE 1.5]**

## Overview

Users can compose complex templates with date functions, multi-suggesters, and error prevention for production-ready template workflows.

**FRs covered:** FR3 (template composition), FR4 (date functions), FR17 (multi-select), FR48 (error diagnostics)

## Implementation Notes

- **Foundation**: Extends Epic 12 template system (not replacement - builds on MiniJinja + TemplateExecutor)
- **Integration Points**:
  - Epic 12: Uses TemplateExecutor, PromptSession, BindingService as foundation
  - Epic 7: Schema validation for template variables and partials
  - Epic 11: Query results used in multi-select suggesters
  - Epic 14: Miette diagnostics for template errors (circular dependencies, missing partials)
- **Date Handling**: `chrono` for date math, `chrono-english` for natural language parsing
- **Template Composition**: MiniJinja `include` statements with dependency graph validation
- **Multi-Select UI**: Dialoguer multi-select with fuzzy search via `skim` crate
- **Performance Targets**:
  - Template dry-run validation: <100ms for templates with 10 partials
  - Date parsing: <10ms for natural language inputs
  - Multi-select rendering: <50ms for lists of 1000 items
  - Complex template execution: <500ms meeting NFR1
- **Error Prevention**: Dry-run validation before first user prompt (FR48)
- **Location**:
  - `crates/domain/src/template/` - TemplateGraph, TemplateDate value objects
  - `crates/app/src/services/template/` - DryRunService, DateResolutionService
  - `crates/adapters/src/api/ui/` - MultiSelectUI implementation
- **Architecture Patterns**:
  - **Dependency Graph**: TemplateGraph uses adjacency list for cycle detection (Tarjan's algorithm)
  - **Value Object**: TemplateDate wraps chrono::DateTime with domain operations
  - **Service Layer**: DryRunService orchestrates AST parsing + dependency validation
  - **Adapter Pattern**: DateResolutionAdapter bridges chrono-english to domain TemplateDate
- **May Create**: ADR for template composition validation patterns if complex edge cases emerge

## Story 13.1: Implement Template Dependency Graph and Cycle Detection

As a developer, I want to represent template relationships in the domain,
So that I can detect circular dependencies and missing files before execution.

**Acceptance Criteria:**

**Given** MiniJinja templates use `{% include "partial.md" %}` for composition
**When** I define TemplateGraph in `crates/domain/src/template/graph.rs`
**Then** it represents dependency relationships as directed graph: `HashMap<PathBuf, Vec<PathBuf>>`
**And** nodes are template paths, edges are include dependencies
**And** graph supports operations: `add_edge()`, `get_dependencies()`, `find_cycles()`

**Given** circular dependencies cause infinite loops during rendering
**When** I implement cycle detection
**Then** use Tarjan's strongly connected components algorithm (O(V+E) complexity)
**And** cycle detection runs during dry-run validation (Story 13.3)
**And** cycles are reported as: `CircularDependency { cycle: Vec<PathBuf> }` domain error

**Given** cycle detection must be efficient
**When** I benchmark with complex template hierarchies
**Then** detecting cycles in graph with 100 templates + 200 edges completes in <10ms
**And** algorithm handles deeply nested includes (depth >10) without stack overflow
**And** performance is validated via criterion benchmarks

**Given** missing partials cause runtime failures
**When** I implement dependency validation
**Then** TemplateGraph validates all edges point to existing files
**And** missing partials reported as: `MissingPartial { parent: PathBuf, missing: PathBuf }` domain error
**And** validation checks file existence before template execution starts

**Given** dependency graph must be built from template AST
**When** I implement graph construction
**Then** parse MiniJinja template AST to extract `include` statements
**And** recursively traverse included templates to build complete graph
**And** graph construction handles relative paths (resolved relative to template file location)

**Given** Epic 14 needs actionable diagnostics
**When** I define domain errors
**Then** `CircularDependency` includes full cycle path: `A → B → C → A`
**And** `MissingPartial` includes parent template and missing file path
**And** errors implement `miette::Diagnostic` for Epic 14 CLI formatting

**Given** template graph supports dependency analysis
**When** I implement graph queries
**Then** `get_all_dependencies(root: PathBuf)` returns transitive closure of all included templates
**And** `get_dependents(partial: PathBuf)` returns all templates that include this partial
**And** queries used for cache invalidation (if partial changes, invalidate all dependents)

## Story 13.3: Implement Template Dry-Run Validation Service

As a user, I want the system to verify my template structure before asking for input,
So that I don't waste time on a session that will fail due to a missing file.

**Acceptance Criteria:**

**Given** Epic 12 TemplateExecutor executes templates interactively
**When** I implement DryRunService in `crates/app/src/services/template/dry_run.rs`
**Then** it runs BEFORE Epic 12 PromptSession starts (pre-flight validation)
**And** validation occurs before first user prompt to prevent wasted effort
**And** service depends on: TemplateGraph (Story 13.1), Epic 12 TemplateLoader

**Given** template composition uses MiniJinja includes
**When** I implement validation workflow
**Then** validation steps:
1. Parse root template AST using MiniJinja parser
2. Extract all `{% include "partial.md" %}` statements from AST
3. Recursively parse each partial to find nested includes
4. Build TemplateGraph (Story 13.1) from all discovered dependencies
5. Run cycle detection on graph
6. Validate all referenced partials exist on disk

**Given** circular dependencies cause infinite loops
**When** cycle detection fails
**Then** return `CircularDependency` error with full cycle path
**And** Epic 14 CLI formats as: `Circular dependency: root.md → a.md → b.md → a.md`

**Given** dry-run must be fast to avoid delaying user
**When** I benchmark validation
**Then** validating template with 10 partials completes in <100ms
**And** validation with 50 partials completes in <500ms

**References:** FR3, FR48, NFR1

## Story 13.4: Implement Context-Aware YAML Format Sensing

As a template author, I want the system to automatically format array variables,
So that my frontmatter remains valid YAML while my content remains readable markdown.

**Acceptance Criteria:**

**Given** templates have frontmatter (YAML) and content (Markdown) sections
**When** I implement FormatSensingService in `crates/app/src/services/template/format_sensing.rs`
**Then** it detects template sections:
- Frontmatter: between `---` delimiters
- Content: after closing `---`

**Given** Epic 6 configuration specifies YAML array style
**When** rendering arrays in frontmatter
**Then** respect `yaml_array_style` config: `block` (multi-line) or `flow` (inline)
**And** apply YAML-safe escaping to string values (escape quotes, special chars)

**Given** arrays in content should be readable
**When** rendering arrays outside frontmatter
**Then** use Markdown list format by default
**And** no YAML escaping applied

**References:** FR17, FR26

## Story 13.5: Implement Natural Language Date Resolution

As a user, I want to provide relative dates like "tomorrow" or "next Friday",
So that I can create notes for future events easily.

**Acceptance Criteria:**

**Given** users input natural language date strings
**When** I implement DateResolutionAdapter in `crates/adapters/src/spi/template/date_adapter.rs`
**Then** it uses `chrono-english` crate for parsing
**And** resolves strings: "today", "tomorrow", "next Friday", "in 2 weeks", "Jan 27"

**Given** Epic 6 configuration specifies timezone and locale
**When** parsing dates
**Then** respect `timezone` setting for date calculations
**And** respect `first_day_of_week` for relative week calculations

**Given** ambiguous inputs need fallback
**When** parsing fails
**Then** fallback to current date with warning
**And** log diagnostic: "Could not parse '{input}', using today's date"

**References:** FR4

## Story 13.6: Implement Multi-Select Terminal UI

As a user, I want to select multiple items from a list,
So that I can quickly populate array fields like tags or contacts.

**Acceptance Criteria:**

**Given** Epic 12 UIPort trait defines prompt operations
**When** I implement `prompt_multi_select()` in Dialoguer adapter
**Then** render terminal UI with:
- Checkboxes for each item (spacebar to toggle)
- Fuzzy search filter (type to narrow list)
- Arrow keys for navigation
- Enter to confirm selection

**Given** Epic 11 query results populate suggesters
**When** multi-select displays query results
**Then** show display text (note title) but return value (file path)
**And** support selection of 100+ items without performance degradation

**Given** fuzzy search improves usability
**When** user types filter text
**Then** use `skim` crate for fuzzy matching algorithm
**And** matching is case-insensitive
**And** highlights matched characters in display

**References:** FR17


**References:** FR3, FR48

## Story 13.2: Implement TemplateDate Value Object with Date Math

As a template author, I want a robust date domain model,
So that I can perform reliable date math and formatting in my templates.

**Acceptance Criteria:**

**Given** templates need date manipulation (FR4: "yesterday", "next Friday", date math)
**When** I define TemplateDate in `crates/domain/src/template/date.rs`
**Then** it wraps `chrono::DateTime<chrono::Local>` with domain operations
**And** provides builder pattern: `TemplateDate::now()`, `TemplateDate::parse("2025-01-27")`
**And** implements `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize` for template use

**Given** date math operations are core functionality
**When** I implement arithmetic methods
**Then** provide operations:
- `add_days(i64)` - add/subtract days (negative values subtract)
- `add_weeks(i64)` - add/subtract weeks
- `add_months(i64)` - add/subtract months (handles variable month lengths)
- `add_years(i64)` - add/subtract years (handles leap years)
**And** all operations return `Self` for method chaining

**Given** date formatting must support multiple output formats
**When** I implement `format(pattern: &str)` method
**Then** use chrono's strftime patterns: `%Y-%m-%d`, `%B %d, %Y`, etc.
**And** provide convenience methods:
- `to_iso8601()` → `"2025-01-27T14:30:00Z"`
- `to_human()` → `"January 27, 2025"`
- `to_obsidian()` → `"[[2025-01-27]]"` (wiki-link format)

**Given** leap years and month boundaries are edge cases
**When** I test date arithmetic
**Then** `2024-02-29 + 1 year` → `2025-02-28` (leap year handling)
**And** `2025-01-31 + 1 month` → `2025-02-28` (month overflow handling)
**And** `2025-12-31 + 1 day` → `2026-01-01` (year rollover)

**Given** timezones affect date calculations
**When** I implement timezone support
**Then** TemplateDate respects Epic 6 vault configuration timezone setting
**And** default timezone is system local timezone if not configured
**And** timezone conversions use chrono: `to_timezone(tz: Tz)` method

**Given** templates render dates in context
**When** I integrate with MiniJinja
**Then** TemplateDate implements MiniJinja's `Object` trait for template access
**And** template can call: `{{ created_date.add_days(7).format("%Y-%m-%d") }}`
**And** date operations are chainable in template expressions

**Given** date parsing must be robust
**When** I implement `parse(input: &str)` constructor
**Then** support formats: ISO8601, RFC3339, common date formats
**And** return domain error `InvalidDateFormat { input, expected_formats }` on parse failure
**And** parsing uses chrono's `DateTime::parse_from_str()` with multiple format attempts

**Given** serialization is needed for template context
**When** I derive serde traits
**Then** TemplateDate serializes as ISO8601 string: `"2025-01-27T14:30:00Z"`
**And** deserialization handles multiple input formats gracefully
**And** serialized dates are timezone-aware (include offset information)

**References:** FR4, FR26

### Story 13.3: [App] Template Composition "Dry Run" Orchestrator

As a user, I want the system to verify my template structure before asking for input, so that I don't waste time on a session that will fail due to a missing file.
**Acceptance Criteria:**

- **Given** a template execution request
- **When** the `DryRunService` runs
- **Then** it recursively parses the AST of the root template and all included partials.
- **And** if any partial is missing or a cycle is detected, it returns a `miette` diagnostic immediately.
- **And** this check must pass before the first prompt is displayed to the user.
  **References:** FR3, FR48

### Story 13.4: [App] Context-Aware Format Sensing Service

As a template author, I want the system to automatically format array variables based on their position in the file, so that my frontmatter remains valid YAML while my content remains readable markdown.
**Acceptance Criteria:**

- **Given** a rendering session
- **When** a variable is rendered between the `---` YAML delimiters
- **Then** the system automatically applies the `yaml_array_style` (Block vs. Flow) from the configuration.
- **And** it applies YAML-safe escaping to string values.
- **And** variables rendered outside the delimiters default to standard Markdown formatting.
  **References:** FR17, FR26

### Story 13.5: [Adapters/SPI] Chrono-based Natural Language Date Adapter

As a user, I want to provide relative dates like "tomorrow" or "next Friday" in my prompts, so that I can create notes for future events easily.
**Acceptance Criteria:**

- **Given** a natural language string from a prompt
- **When** the `DateResolutionAdapter` is called
- **Then** it resolves the string into a concrete timestamp using `chrono-english`.
- **And** it respects the `timezone` and `first_day_of_week` settings from the vault configuration.
- **And** it provides a fallback to the current date if the input is ambiguous.
  **References:** FR4

### Story 13.6: [Adapters/API] Multi-Select Terminal UI

As a user, I want to select multiple items from a list using a fuzzy-searchable terminal picker, so that I can quickly populate array fields like tags or contacts.
**Acceptance Criteria:**

- **Given** a list of suggestions
- **When** the `UIPort::prompt_multi_select` is called
- **Then** it renders a picker allowing the user to toggle multiple items (e.g., using spacebar).
- **And** it supports fuzzy-searching the display labels.
- **And** it returns a collection of the internal `values` for the selected items.
  **References:** FR17

### Story 13.7: Review Epic 13 Test Suite

As a developer, I want a comprehensive and efficient test suite for the advanced template features, so that I can maintain the code with confidence.
**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 13 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 13 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 13 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 13 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**Given** the implementation of Epic 13
**When** I run the test suite
**Then** it achieves 90%+ coverage for the `TemplateGraph`, `TemplateDate`, and composition services.
**And** property-based tests verify that circular dependency detection works for complex template hierarchies.
**And** the suite validates that date operations handle edge cases (leap years, timezone boundaries).

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

**References:** NFR16

### Story 13.8: [Docs] Epic 13 User & Developer Documentation

As a user, I want clear instructions on how to use advanced template features like composition and date functions, so that I can create sophisticated template workflows.
**Acceptance Criteria:**

- **Given** a completed Epic 13
- **When** I review the documentation
- **Then** it includes examples of template composition with includes and cycles.
- **And** it provides natural language date parsing examples.
- **And** it explains multi-select suggester usage for array fields.
- **And** it documents the dry-run validation process.
  **References:** NFR13
