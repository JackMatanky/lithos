# Testing Strategy

## Unit Tests

- **Scope:** Core services under `internal/app` and value objects in `internal/domain` (template engine, schema registry/validator, query service, indexing service).
- **Approach:** Table-driven tests using Go’s `testing` package with in-memory fakes for ports (e.g., FakeCacheAdapter, FakeInteractive). Assert both `Result[T]` states and domain side effects.
- **Requirements:** Cover happy path, validation failure, context cancellation, and error propagation scenarios. Verify atomic write behavior via temp directories.
- **Location:** Co-located `*_test.go` files near implementation. Use `testdata/` for fixtures with helper copy utilities to avoid mutating source fixtures.

## Integration Tests

- **Scope:** Cobra CLI command flows (`lithos new`, `lithos find`, `lithos index`) interacting with a representative vault.
- **Approach:** Execute commands through the Cobra runner (or compiled binary for smoke runs) against vault fixtures cloned into temp directories. Compare generated files to golden outputs and inspect logs for context fields.
- **Requirements:** Ensure CLI exit codes map correctly to success/warning/error, indexing produces warnings for invalid files, and template generation respects interactive prompts.
- **Location:** `tests/integration/` with scripts runnable via `just verify`. Mark tests requiring external binaries as `//go:build integration`.

## End-to-End / Smoke Tests

- **Scope:** Released binary executing a full workflow (index → new → find) using a sample vault.
- **Approach:** Scripted run (e.g., `just smoke`) invoked in release CI after `goreleaser --snapshot`. Capture logs and ensure cache artifacts appear under `.lithos/`.
- **Frequency:** Release pipeline + optional nightly job; failures block artifact publishing.

## Test Data Management

- **Fixtures:** Immutable vault fixtures under `testdata/vault/` containing templates, schemas, and notes. Tests copy fixtures into temp directories before mutation.
- **Golden Files:** Store rendered notes and cache JSON snapshots in `testdata/golden/`; verify via byte comparison.
- **Cleanup:** Use `t.Cleanup` to remove temp directories and reset environment variables after each test.

## Continuous Testing

- **CI Pipeline:**
  - `go test ./... -race` (unit + integration with build tags).
  - `golangci-lint run`, `gitleaks detect`, optional `gosec`.
  - `go test -bench=.` (template rendering benchmark) recorded but non-blocking.
- **Coverage Targets:** ≥85% for `internal/app`, ≥70% overall; reports uploaded to PR discussions.
- **Developer Workflow:** `just verify` runs full suite locally; partial runs (`just test-unit`, `just test-integration`) available for targeted checks.

---
