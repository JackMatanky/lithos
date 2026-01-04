# Epic 4: Schema-Driven Lookups & Validation

This epic connects TemplateEngine, FrontmatterService, QueryService, and CommandOrchestrator as specified in architecture v0.6.8. The stories ensure templates can leverage schema-aware lookups, note generation validates frontmatter end-to-end, and regression tests protect the workflow. Execution proceeds from core functionality to event-driven enhancements, validation, testing, and documentation. This epic builds on the core indexing infrastructure from Epic 3 and incorporates event-driven patterns for observability.

---

## Story 4.1 Template Interface Pattern

As the domain architect,
I want TemplateEngine consumers to depend on a domain-level `Template` interface,
so that schema-driven workflows stay hexagonal, mockable, and portable across adapters.

**Prerequisites:** Epic 3 TemplateEngine foundation complete.

### Acceptance Criteria
1. `internal/domain/template.go` defines `TemplateID` (string alias) and `Template` interface with `ID()`/`Execute()` methods plus a concrete domain implementation.
2. `internal/adapters/spi/template/go_template.go` implements the interface by wrapping `*template.Template`, including constructor + shared `FuncMap` wiring.
3. All domain/application services (TemplateEngine, CLI commander, orchestrators) accept/return the interface only—no direct `*template.Template` imports outside adapters.
4. TemplateLoader SPI ports return the interface, integration tests prove mocks can replace real templates, and QA docs capture the new seam.
5. Documentation (`docs/architecture/data-models.md`, this PRD) and performance benchmarks reference the Template interface migration with updated risk/coverage notes.

---

## Story 4.2 Template Function Lookup

As a developer,
I want TemplateEngine to expose lookup functions (lookup, query, fileClass),
so that templates can access indexed notes and perform schema-driven queries.

**Prerequisites:** Story 4.1.

### Acceptance Criteria
1. TemplateEngine exposes `lookup(basename)` function returning note data by basename
2. TemplateEngine exposes `query(filter)` function for complex queries with filter objects
3. TemplateEngine exposes `fileClass(noteID)` function returning schema classification
4. All functions delegate to QueryService with proper error handling
5. Functions include GoDoc referencing FR9 schema-driven requirements

---

## Story 4.3 Command Orchestrator NewNote Workflow

As a developer,
I want CommandOrchestrator.NewNote to follow the ten-step workflow in the architecture,
so that note creation is schema-driven and keeps vault and cache in sync.

**Prerequisites:** Stories 3.1–3.7, Story 4.1.

### Acceptance Criteria
1. `CommandOrchestrator.NewNote` executes the documented sequence (template load, render, frontmatter extract/validate, NoteID generation, path resolution, vault persist, cache persist) exactly as in `docs/architecture/components.md#commandorchestrator`.
2. Method returns typed errors consistent with `error-handling-strategy.md`, logs summary info, and satisfies FR2 (non-interactive execution) and FR6/FR7 validation guarantees.
3. Unit tests with fakes verify success path, template load failure, validation failure, vault persistence failure, cache warning handling, and ensure partial failures leave vault/cache consistent.

---

## Story 4.4 Frontmatter End-to-End Validation

As a developer,
I want CommandOrchestrator.NewNote to perform complete frontmatter validation against schemas,
so that invalid notes are rejected before vault persistence and schema-driven constraints are enforced.

**Prerequisites:** Story 4.3.

### Acceptance Criteria
1. CommandOrchestrator.NewNote includes comprehensive frontmatter validation using FrontmatterService.Validate()
2. Validation occurs after template rendering but before NoteID generation and persistence
3. Validation checks all schema constraints including FileSpec references, type validation, and required fields
4. Validation errors prevent note creation and return actionable error messages with field names, expected types, and remediation hints

---

## Story 4.5 Template Function FileClass

As a developer,
I want TemplateEngine to expose a fileClass template function,
so that templates can access a note's schema classification for conditional logic.

**Prerequisites:** Stories 4.1-4.2.

### Acceptance Criteria
1. TemplateEngine exposes `fileClass` function: `{{fileClass noteID}}` returns schema name for given note
2. Function accepts NoteID parameter (string basename or full path)
3. Function delegates to QueryService to retrieve note's fileClass from frontmatter
4. Function returns schema name (e.g., "contact", "project") or empty string if note not found
5. Function handles invalid NoteID parameters without crashing and includes GoDoc referencing FR9

---

## Story 4.6 Event-Driven Schema Lookups

As a developer,
I want schema-driven lookups to use the repository's event-driven architecture,
so that lookups are decoupled, observable, and support reactive behaviors.

**Prerequisites:** Stories 4.4, 4.5.

### Acceptance Criteria
1. TemplateEngine integrates with EventBus for lookup event publishing
2. FrontmatterService publishes ValidationPerformedEvent on validation completion
3. QueryService subscribes to SchemaUpdatedEvent for cache invalidation
4. CommandOrchestrator publishes NoteCreatedEvent after successful note creation
5. Lookup events include performance metrics and context for tracing
6. Validation events include remediation hints and field-level error details
7. Event publishing doesn't impact lookup performance (<5% overhead)

---

## Story 4.7 Schema-Driven Lookup Integration Test

As a QA-focused developer,
I want an integration test that exercises schema-driven template lookups end to end,
so that future changes cannot break the combined workflow.

**Prerequisites:** Stories 4.1–4.6.

### Acceptance Criteria
1. `tests/integration/schema_lookup_test.go` spins up TemplateEngine, QueryService, and FrontmatterService with real fixtures (schemas/property bank/cache notes) exercising lookup helpers, FileSpec validation, and CommandOrchestrator note creation.
2. The test suite verifies FR3/FR8 behaviours (interactive helpers, schema-driven lookups) and ensures rendered output matches golden files.
3. `docs/architecture/testing-strategy.md` documents the `go test ./tests/integration -run SchemaLookup` command and fixture layout for reproducibility.

---

## Story 4.8 Epic 4 Code Refactoring and Modularization

As a developer,
I want to refactor all files, structs, and functions updated during Epic 4 to ensure they follow SRP, DRY, modularity, and maintainability principles,
so that god-objects are eliminated, code is clean, and future maintenance is simplified.

**Prerequisites:** Stories 4.1–4.7.

### Acceptance Criteria

1. **Single Responsibility Principle (SRP):** Every struct, function, and package has one clear responsibility
2. **DRY (Don't Repeat Yourself):** Eliminate code duplication across Epic 4 components
3. **Modularity:** Components are loosely coupled and highly cohesive
4. **God-Object Elimination:** Break down any structs with >5 responsibilities into focused components
5. **Template Engine Functions:** Refactor lookup, query, fileClass functions for clarity and performance
6. **Schema Resolution:** Ensure schema resolver follows SRP and handles edge cases cleanly
7. **Query Service:** Modularize query operations and eliminate any monolithic methods
8. **Validation Logic:** Separate validation concerns from business logic
9. **Integration Points:** Clean up interfaces between template engine, query service, and schema system
10. **Test Coverage:** Maintain >80% coverage after refactoring
11. **Linting:** Zero golangci-lint violations
12. **Performance:** No regression in integration test execution time
13. **Backward Compatibility:** All existing functionality preserved

---

## Story 4.9 Dependency Injection and E2E Test for Schema-Driven Lookups

As a developer,
I want to implement dependency injection for the schema-driven lookup components and add comprehensive e2e tests,
so that the schema-driven lookup functionality is properly wired and thoroughly tested end-to-end.

**Prerequisites:** Stories 4.1–4.8.

### Acceptance Criteria

1. All schema-driven lookup components are properly registered in the DI container
2. The main.go file includes the schema-driven lookup dependencies in the application wiring
3. E2E tests cover the complete schema-driven lookup workflow from template function calls through schema resolution to data retrieval
4. The e2e tests validate actual schema-driven behavior with realistic test data
5. Test scenarios include various lookup patterns and schema configurations

---

## Story 4.10 Documentation Update for Schema-Driven Lookups Release

As a maintainer,
I want to update all project documentation to reflect the completed schema-driven lookups implementation,
so that users and developers have accurate, comprehensive documentation for the new schema-driven lookup capabilities.

**Prerequisites:** Stories 4.1–4.9.

### Acceptance Criteria
1. README.md updated with schema-driven lookup features and template functions
2. Architecture documentation updated with schema-driven lookup engine details
3. API documentation created for new template functions and schema-driven capabilities
4. Code comments updated for all schema-driven lookup code
5. Change log updated with Epic 4 completion details
6. All technical details match the implemented components
7. Template function examples work with the actual implementation
