# Epic 4: Schema-Driven Lookups & Validation

This epic connects TemplateEngine, FrontmatterService, QueryService, and CommandOrchestrator as specified in architecture v0.6.8. The stories ensure templates can leverage schema-aware lookups, note generation validates frontmatter end-to-end, and regression tests protect the workflow. Execution proceeds from TemplateEngine helpers to validation updates, orchestration alignment, and integration testing. This epic builds on the core indexing infrastructure from Epic 3 and should be completed before Epic 5's interactive features.

---

## Story 4.1 TemplateEngine Lookup Helpers

As a template author,
I want TemplateEngine helpers for schema-aware lookups,
so that templates can query indexed notes directly through documented functions.

**Prerequisites:** Epic 3 complete.

### Acceptance Criteria
1. `internal/app/template/service.go` registers `lookup`, `query`, and `fileClass` helpers exactly as in `docs/architecture/components.md#templateengine`, delegating to QueryService and Config.
2. Helpers satisfy FR3 and FR9 by supporting wikilink formatting, schema-aware lookups, and appropriate error propagation using `InteractiveError` or standard errors per context.
3. Implementation ensures helper outputs are immutable copies and do not expose underlying index structures.
4. Unit tests verify success, empty results, error propagation, and helper behaviour with both slice and map inputs.

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
