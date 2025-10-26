# Error Handling Strategy

## General Approach

- **Idiomatic Go Errors:** Domain services and ports use standard Go `(T, error)` return signatures. No custom Result[T] wrapper types.
- **Error Types:** Use domain-specific error types (`FrontmatterError`, `SchemaError`, `TemplateError`, `ConfigError`, `CacheReadError`, `CacheWriteError`, `FileSystemError`) that implement the standard `error` interface, enriched with context (component, template path, schema name).
- **Error Wrapping:** Errors bubble upward with additional context using `fmt.Errorf("context: %w", err)` for proper error chain unwrapping via `errors.Is()` and `errors.As()`.
- **Propagation Rules:** No panics except for programmer assertions; errors propagate through return values. Context cancellation must be checked at key boundaries (`select { case <-ctx.Done(): return ctx.Err() }`).
- **User Feedback:** Application service (main.go) maps domain errors to CLI exit codes (`0` success, `1` validation/template issues, `2` system faults) and renders actionable remediation hints.

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

- **External API Errors:** None in MVP. When remote adapters arrive, they must wrap upstream errors into domain-specific error types using `fmt.Errorf("context: %w", err)` and follow exponential backoff + retry guardrails defined in adapter docs.

- **Business Logic Errors:** Frontmatter validation failures include schema name, offending field, and remediation message; template parsing errors include line/column if available. CLI adapter surfaces these as structured output plus exit code `1`.

- **Data Consistency:** VaultIndexer and JSONFileCacheAdapter always write to temp files and rename atomically; partial writes trigger rollback with warning logs. `lithos index` remains idempotent, so reruns after failures are safe.

- **Testing Contracts:** Unit tests assert both success and error return values plus logged context to ensure code respects the error model. Use `errors.Is()` and `errors.As()` to verify error types.

## Domain Error Types

Domain-specific error types provide rich context for debugging and user feedback. All implement the standard `error` interface and support error unwrapping via `Unwrap() error` method.

### FrontmatterError

Frontmatter validation failures from FrontmatterService.Validate().

**Fields:**
- `Schema` (string) - Schema name used for validation
- `Field` (string) - Field name that failed validation
- `Rule` (string) - Validation rule violated ("required", "pattern", "min", "max", "enum", "type", etc.)
- `Value` (any) - Actual field value that failed
- `Message` (string) - Human-readable error message with remediation hint

**Example:**
```go
&FrontmatterError{
    Schema:  "contact",
    Field:   "email",
    Rule:    "pattern",
    Value:   "invalid-email",
    Message: "field 'email' must match pattern '^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$'",
}
```

### SchemaError

Schema loading or inheritance resolution failures from SchemaLoader adapter.

**Fields:**
- `Schema` (string) - Schema name that caused the error
- `Type` (string) - Error type: "not_found", "circular_dependency", "invalid_json", "invalid_ref", "parse_error"
- `Message` (string) - Detailed error message
- `Err` (error) - Wrapped underlying error (optional)

**Example:**
```go
&SchemaError{
    Schema:  "contact",
    Type:    "circular_dependency",
    Message: "circular inheritance detected: contact -> person -> contact",
}
```

### TemplateError

Template parsing or execution failures from TemplateEngine service.

**Fields:**
- `TemplateID` (TemplateID) - Template that caused the error
- `Line` (int) - Line number where error occurred (0 if unknown)
- `Column` (int) - Column number where error occurred (0 if unknown)
- `Type` (string) - Error type: "parse_error", "execution_error", "function_error"
- `Message` (string) - Detailed error message
- `Err` (error) - Wrapped underlying error (optional)

**Example:**
```go
&TemplateError{
    TemplateID: TemplateID{value: "contact"},
    Line:       15,
    Column:     8,
    Type:       "function_error",
    Message:    "prompt function requires 'name' parameter",
}
```

### ConfigError

Configuration loading or validation failures from ConfigLoader adapter.

**Fields:**
- `Key` (string) - Config key that caused the error
- `Type` (string) - Error type: "missing", "invalid_type", "invalid_value", "path_not_found"
- `Message` (string) - Human-readable error message with remediation hint
- `Err` (error) - Wrapped underlying error (optional)

**Example:**
```go
&ConfigError{
    Key:     "VaultPath",
    Type:    "path_not_found",
    Message: "vault path '/invalid/path' does not exist or is not readable",
}
```

### CacheReadError

Cache read or query failures from CacheReader port implementations (CQRS read side).

**Fields:**
- `Operation` (string) - Operation that failed: "read", "query", "scan", "index_load"
- `NoteID` (NoteID) - Note ID that caused the error (zero value if not applicable)
- `Message` (string) - Error description
- `Err` (error) - Wrapped underlying error

**Example:**
```go
&CacheReadError{
    Operation: "read",
    NoteID:    NoteID{value: "contact-john"},
    Message:   "failed to read cached note",
    Err:       fmt.Errorf("cache file corrupted"),
}
```

### CacheWriteError

Cache write or indexing failures from CacheWriter port implementations (CQRS write side).

**Fields:**
- `Operation` (string) - Operation that failed: "write", "delete", "index_update"
- `NoteID` (NoteID) - Note ID that caused the error (zero value if not applicable)
- `Message` (string) - Error description
- `Err` (error) - Wrapped underlying error

**Example:**
```go
&CacheWriteError{
    Operation: "write",
    NoteID:    NoteID{value: "contact-john"},
    Message:   "failed to write note to cache",
    Err:       fmt.Errorf("disk full"),
}
```

### FileSystemError

File system operation failures from FileReader/FileWriter/FileWalker port implementations.

**Fields:**
- `Operation` (string) - Operation that failed: "read", "write", "scan", "delete", "stat"
- `Path` (string) - File path that caused the error
- `Message` (string) - Error description
- `Err` (error) - Wrapped underlying error

**Example:**
```go
&FileSystemError{
    Operation: "read",
    Path:      "/vault/templates/contact.md",
    Message:   "failed to read template file",
    Err:       os.ErrPermission,
}
```

## Error Wrapping Guidelines

- **Adapters wrap infrastructure errors:** Convert `os.ErrNotExist` → `FileSystemError`, `json.SyntaxError` → `SchemaError`, etc.
- **Services add context:** Use `fmt.Errorf("validating frontmatter: %w", err)` to add operation context while preserving error chain.
- **Preserve error chains:** Always use `%w` verb for wrapping to enable `errors.Is()` and `errors.As()` checks.
- **CLI adapter unwraps for user display:** Extract domain error types and format user-friendly messages with remediation hints.

---
