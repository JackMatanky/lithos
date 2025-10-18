# Coding Standards

These standards are **MANDATORY** for Lithos contributors and AI agents. They are intentionally minimal and target project-specific gotchas; general Go best practices are assumed. Violations require explicit, commented exceptions.

## Scope & Terms

- Interpret **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, **MAY** per RFC 2119.
- **Application core** covers `internal/domain` and `internal/app`.
- **Ports** reside under `internal/ports` (API/SPI).
- **Adapters** reside under `internal/adapters`.
- **Exception annotation** = inline `//nolint:<rule> reason:<why>` documented at the violation point.

## Standards Enforcement

- **On Save:** Editors **MUST** run `golangci-lint run` (which invokes formatting/linters per `.golangci.toml`).
- **On Commit:** `golangci-lint run` and `gitleaks detect` **MUST** pass (enforced via pre-commit).
- **On Pull Request:** CI **MUST** pass `go test ./...`, `golangci-lint run`, `gitleaks detect`.
- **Manual:** `just verify` **MAY** be used to execute the full check suite locally.

## Core Standards

- Go **1.25+ MUST** be used (CI enforces via toolchain).
- The application core **MUST** use the Result pattern supplied by `internal/shared/errors` (Rust-like `Result[T]`). Plain `(T, error)` signatures **MUST NOT** cross port boundaries.
- Shared logging (`internal/shared/logger`) **MUST** be the only logging facility; no `fmt.Print*` or `log.*`.
- Functions performing I/O or long-running work **MUST** accept `context.Context` as the first parameter and abort on cancellation.
- `VaultIndexingService` and cache adapters **MUST** continue to use atomic temp-file → rename patterns.

## Naming Conventions

| Element         | Convention                | Example                  |
| --------------- | ------------------------- | ------------------------ |
| Ports           | PascalCase + `Port`       | `TemplateRepositoryPort` |
| Adapters        | PascalCase + `Adapter`    | `TemplateFSAdapter`      |
| Domain Services | PascalCase descriptive    | `TemplateEngineService`  |
| Result Helpers  | `errors.Ok`, `errors.Err` | `errors.Err[Note](err)`  |
| Test Doubles    | `Fake`/`Stub` prefix      | `FakeSchemaLoader`       |

Names **MUST NOT** repeat package context (e.g., avoid `template.TemplateEngine`). Keep receiver names 1‑2 letters.

## Critical Rules

- Functions over **60 lines** or with >2 nested control structures **SHOULD** be refactored.
- Shared maps/slices **MUST NOT** be mutated without synchronization.
- New goroutines **MUST** be clearly documented and tied to context cancellation.
- Ports **MUST** remain lean (≤3 methods); grow only with proven need.
- Adapters **MUST NOT** import other adapters; communication flows through ports.
- `panic` **MUST NOT** be used outside package `main` initialization.

## Error Handling

- The application core and ports **MUST** expose `Result[T]` signatures.
- Adapters interfacing with third-party libraries **MAY** use `(T, error)` internally but **MUST** convert to `Result[T]` before returning.
- Errors **MUST** be wrapped with contextual messages (template ID, file path, schema name).
- Use `errors.Is`/`errors.As` for comparisons; never rely on `==` for non-sentinel errors.

## Testing

- Unit tests **MUST** live beside the code under test (`*_test.go`) and use table-driven cases for branches.
- Integration tests **MAY** reside under `tests/` when they require full vault fixtures; they **MUST** be callable via `just verify`.
- Tests **MUST** cover success, validation failure, and cancellation paths for command orchestration.
- Golden files belong under `testdata/` mirroring vault layout.

## Documentation

- Every package **MUST** have a package comment documenting responsibility.
- Exported identifiers **MUST** have GoDoc summarizing purpose, error conditions, and context requirements.
- Concurrency and side effects **MUST** be documented where applicable.
- Deprecated APIs **MUST** use the `Deprecated:` prefix with an alternative.

---
