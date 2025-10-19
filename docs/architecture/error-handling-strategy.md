# Error Handling Strategy

## General Approach

- **Explicit Results:** Domain services return `Result[T]` wrappers backed by the shared `internal/shared/errors` package (custom implementation using Go generics) to express success or error while preserving Go’s `error` interface.
- **Error Types:** Use domain-specific errors (`ValidationError`, `SchemaError`, `TemplateError`, `ConfigError`, `StorageError`) enriched with context (component, template path, schema name).
- **Propagation Rules:** No panics except for programmer assertions; errors bubble upward, wrapped with additional context using `errors.Join` where appropriate. Context cancellation must be checked at key boundaries (`select { case <-ctx.Done(): ... }`).
- **User Feedback:** CommandOrchestrator maps domain errors to CLI exit codes (`0` success, `1` validation/template issues, `2` system faults) and renders actionable remediation hints.

## Logging Standards

- **Library:** `github.com/rs/zerolog` via the shared `logger` package (JSON mode by default; pretty-print when TTY detected).
- **Required Fields:** `correlation_id` (UUIDv7 per command), `component` (e.g., `template.engine`), `command` (`new`, `find`, `index`), plus optional `template_id` or `file_path`.
- **Levels:**
  - `debug` for development diagnostics and verbose flag usage.
  - `info` for normal command summaries.
  - `warn` for validation failures or partial successes.
  - `error` for unexpected faults or data corruption attempts.
- **Sensitive Data:** Prompt responses and rendered content never appear in logs—only metadata and error hints.

## Error Handling Patterns

- **External API Errors:** None in MVP. When remote adapters arrive, they must wrap upstream errors into `TemplateError`/`StorageError` using the Result helpers and follow exponential backoff + retry guardrails defined in adapter docs.
- **Business Logic Errors:** Validation failures include schema name, offending field, and remediation message; template parsing errors include line/column if available. CommandOrchestrator surfaces these as structured output plus exit code `1`.
- **Data Consistency:** VaultIndexer and JSONFileCacheAdapter always write to temp files and rename atomically; partial writes trigger rollback with warning logs. `lithos index` remains idempotent, so reruns after failures are safe.
- **Testing Contracts:** Unit tests assert both `Result[T]` states and logged context to ensure AI-generated code respects the error model.

---
