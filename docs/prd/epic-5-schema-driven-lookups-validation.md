# Epic 5: Schema-Driven Lookups & Validation

**Epic Goal:** Connect TemplateEngine, FrontmatterService, QueryService, and CommandOrchestrator as described in architecture v0.6.8 so schemas drive interactive lookups and note validation.

**Dependencies:** Epic 3 (Vault Indexing Engine)

**Architecture References:**
- `docs/architecture/components.md` v0.6.8 — TemplateEngine, FrontmatterService, QueryService, CommandOrchestrator
- `docs/architecture/data-models.md` v0.6.8 — Note, Frontmatter, PropertySpec
- `docs/architecture/error-handling-strategy.md` v0.5.9 — ValidationError, InteractiveError
- `docs/architecture/coding-standards.md` v0.6.7 — logging, testing

---

## Story 5.1: Add TemplateEngine Query Helpers

As a developer, I want TemplateEngine to expose `lookup`, `query`, and `fileClass` helpers exactly as documented so templates can use schema-driven lookups.

### Acceptance Criteria
1. Extend `internal/app/template/service.go` to register helpers `lookup(name string)`, `query(filter QueryFilter)`, and `fileClass(id NoteID)` as referenced in `components.md#templateengine`.
2. `lookup` should call `QueryService.ByPath`/`ByFileClass` depending on arguments and return data structures suitable for template consumption.
3. `query` should wrap `QueryService.ByFrontmatter` using a simple struct (`field`, `value`) matching architecture guidance (no custom DSL).
4. Unit tests verify each helper delegates to QueryService and handles empty results gracefully.
5. Run `golangci-lint run ./internal/app/template` and `go test ./internal/app/template`.

---

## Story 5.2: Use QueryService in Frontmatter FileSpec Validation

As a developer, I need FrontmatterService to leverage QueryService when validating `FileSpec` properties so file references follow the architecture workflow.

### Acceptance Criteria
1. Update `FrontmatterService.Validate` to call `QueryService.ByPath` (or similar) when processing FileSpec properties, failing with `ValidationError` when referenced files are missing.
2. Include tests covering valid references, missing files, and case sensitivity behaviour described in data-models.
3. Run `golangci-lint run ./internal/app/frontmatter` and `go test ./internal/app/frontmatter`.

---

## Story 5.3: Align CommandOrchestrator NewNote Workflow

As a developer, I want `CommandOrchestrator.NewNote` to follow the ten-step workflow enumerated in the architecture.

### Acceptance Criteria
1. Ensure `NewNote` performs the documented steps: load template → render → extract frontmatter → validate → generate NoteID → resolve path → construct note → persist via VaultWriterPort → persist via CacheWriterPort → return note.
2. Return typed errors for each failure stage (wrapping underlying errors) and log cache write warnings without failing the overall operation.
3. Add unit tests using fakes to assert call order and error propagation (template load failure, validation failure, vault persistence failure, cache warning).
4. Run `golangci-lint run ./internal/app/command` and `go test ./internal/app/command`.

---

## Story 5.4: Provide Schema-Driven Lookup Integration Test

As a QA-minded developer, I want an integration test demonstrating schema-driven lookups end to end.

### Acceptance Criteria
1. Add `tests/integration/schema_lookup_test.go` that boots TemplateEngine, QueryService (with populated indices), and renders a template using `lookup`/`query` helpers.
2. Fixture should include schemas, property bank, cache notes, and template leveraging the helper functions; assertions verify rendered output and validation behaviour.
3. Document how to run the suite in `docs/architecture/testing-strategy.md` (append to Schema Runtime section).
4. Run `go test ./tests/integration -run SchemaLookup`.
