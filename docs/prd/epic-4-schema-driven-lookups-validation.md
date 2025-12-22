# Epic 4: Schema-Driven Lookups & Validation

This epic connects TemplateEngine, FrontmatterService, QueryService, and CommandOrchestrator as specified in architecture v0.6.8. The stories ensure templates can leverage schema-aware lookups, note generation validates frontmatter end-to-end, and regression tests protect the workflow. Execution proceeds from TemplateEngine helpers to validation updates, orchestration alignment, and integration testing. This epic builds on the core indexing infrastructure from Epic 3 and should be completed before Epic 5's interactive features.

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

## Story 4.2 Frontmatter FileSpec Validation with QueryService

As a developer,
I want FrontmatterService to validate FileSpec properties using QueryService,
so that file references are checked against the indexed vault.

**Prerequisites:** Stories 3.6–3.7, Story 4.1.

### Acceptance Criteria
1. `FrontmatterService.Validate` consults QueryService for FileSpec properties exactly as in `docs/architecture/components.md#frontmatterservice`, including query hints from FR8.
2. Validation errors return `ValidationError` instances referencing offending fields and remediation steps per `error-handling-strategy.md`.
3. Unit tests cover valid references, missing files, case sensitivity, and ensure references to wikilinks resolve correctly.

---

## Story 4.3 CommandOrchestrator NewNote Workflow

As a developer,
I want CommandOrchestrator.NewNote to follow the ten-step workflow in the architecture,
so that note creation is schema-driven and keeps vault and cache in sync.

**Prerequisites:** Stories 3.1–3.7, Story 4.2.

### Acceptance Criteria
1. `CommandOrchestrator.NewNote` executes the documented sequence (template load, render, frontmatter extract/validate, NoteID generation, path resolution, vault persist, cache persist) exactly as in `docs/architecture/components.md#commandorchestrator`.
2. Method returns typed errors consistent with `error-handling-strategy.md`, logs summary info, and satisfies FR2 (non-interactive execution) and FR6/FR7 validation guarantees.
3. Unit tests with fakes verify success path, template load failure, validation failure, vault persistence failure, cache warning handling, and ensure partial failures leave vault/cache consistent.

---

## Story 4.4 Schema-Driven Lookup Integration Test

As a QA-focused developer,
I want an integration test that exercises schema-driven template lookups end to end,
so that future changes cannot break the combined workflow.

**Prerequisites:** Stories 4.1–4.3.

### Acceptance Criteria
1. `tests/integration/schema_lookup_test.go` spins up TemplateEngine, QueryService, and FrontmatterService with real fixtures (schemas/property bank/cache notes) exercising lookup helpers, FileSpec validation, and CommandOrchestrator note creation.
2. The test suite verifies FR3/FR8 behaviours (interactive helpers, schema-driven lookups) and ensures rendered output matches golden files.
3. `docs/architecture/testing-strategy.md` documents the `go test ./tests/integration -run SchemaLookup` command and fixture layout for reproducibility.
