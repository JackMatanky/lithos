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
- The application core and ports **MUST** use idiomatic Go `(T, error)` signatures. Domain-specific error types **MUST** implement the standard `error` interface and support error unwrapping via `Unwrap() error`.
- Shared logging (`internal/shared/logger`) **MUST** be the only logging facility; no `fmt.Print*` or `log.*`.
- Functions performing I/O or long-running work **MUST** accept `context.Context` as the first parameter and abort on cancellation.
- `VaultIndexer` and cache adapters **MUST** continue to use atomic temp-file → rename patterns.

## Naming Conventions

| Element         | Convention             | Example                           |
| --------------- | ---------------------- | --------------------------------- |
| Ports           | PascalCase + `Port`    | `TemplateRepositoryPort`          |
| Adapters        | PascalCase + `Adapter` | `TemplateFSAdapter`               |
| Domain Services | PascalCase descriptive | `TemplateEngine`                  |
| Error Types     | PascalCase + `Error`   | `FrontmatterError`, `SchemaError` |
| Test Doubles    | `Fake`/`Stub` prefix   | `FakeSchemaLoader`                |

Names **MUST NOT** repeat package context (e.g., avoid `template.TemplateEngine`). Keep receiver names 1‑2 letters.

## Critical Rules

- Functions over **60 lines** or with >2 nested control structures **SHOULD** be refactored.
- Shared maps/slices **MUST NOT** be mutated without synchronization.
- New goroutines **MUST** be clearly documented and tied to context cancellation.
- Ports **MUST** remain lean (≤3 methods); grow only with proven need.
- Adapters **MUST NOT** import other adapters; communication flows through ports.
- `panic` **MUST NOT** be used outside package `main` initialization.

## Error Handling

- The application core and ports **MUST** use idiomatic Go `(T, error)` return signatures throughout.
- Domain-specific error types **MUST** implement the standard `error` interface and support unwrapping via `Unwrap() error` method.
- Errors **MUST** be wrapped with contextual messages using `fmt.Errorf("context: %w", err)` to preserve error chains for `errors.Is()` and `errors.As()` checks.
- Adapters **MUST** convert infrastructure errors to domain-specific error types (e.g., `os.ErrNotExist` → `FileSystemError`).
- Use `errors.Is`/`errors.As` for comparisons; never rely on `==` for non-sentinel errors.

## Testing

- Unit tests **MUST** live beside the code under test (`*_test.go`) and use table-driven cases for branches.
- Integration tests **MUST** reside under `tests/integration` when they require full vault fixtures; they **MUST** be callable via `just verify`.
- Tests **MUST** cover success, validation failure, and cancellation paths for command orchestration.
- Golden files belong under `testdata/` mirroring vault layout.

## Documentation

- Every package **MUST** have a package comment documenting responsibility.
- Exported identifiers **MUST** have GoDoc summarizing purpose, error conditions, and context requirements.
- Concurrency and side effects **MUST** be documented where applicable.
- Deprecated APIs **MUST** use the `Deprecated:` prefix with an alternative.

## Validation Layer Separation

Lithos implements validation at two distinct architectural layers with different responsibilities:

### Validation Principles

| Layer             | Concern   | Responsibility                                                             | Complexity                  |
| ----------------- | --------- | -------------------------------------------------------------------------- | --------------------------- |
| **Adapter Layer** | Syntactic | Structure, format, types (YAML parsing, regex compilation, file existence) | Low - infrastructure checks |
| **Domain Layer**  | Semantic  | Business rules, schema compliance, cross-field validation                  | High - domain logic         |

### Naming Conventions

Validation method names **MUST** clearly indicate their layer:

| Layer                   | Method Pattern        | Example                                                      |
| ----------------------- | --------------------- | ------------------------------------------------------------ |
| **Adapter (Syntactic)** | `ValidateSyntax()`    | `propertySpec.ValidateSyntax()` - regex compiles, min <= max |
| **Adapter (Syntactic)** | `IsValidStructure()`  | `parser.IsValidStructure()` - YAML delimiters present        |
| **Domain (Semantic)**   | `Validate()`          | `schema.Validate()` - business logic compliance              |
| **Domain (Semantic)**   | `IsSchemaCompliant()` | `frontmatter.IsSchemaCompliant()` - schema rules satisfied   |

**Rationale:** Clear naming prevents confusion about validation layer. Developers immediately know if validation checks infrastructure concerns (syntactic) or business rules (semantic).

### Implementation Examples

#### Syntactic Validation (Adapter Layer)

```go
// internal/adapters/spi/schema/validator.go
func (v *SchemaValidator) ValidateSyntax(schema Schema) error {
    // Check JSON structure
    if schema.Name == "" {
        return errors.New("schema name required")
    }

    // Check regex patterns compile
    for _, prop := range schema.Properties {
        if err := prop.Spec.ValidateSyntax(); err != nil {
            return err
        }
    }

    return nil
}

// PropertySpec variants implement ValidateSyntax
func (s StringSpec) ValidateSyntax() error {
    if s.Pattern != "" {
        if _, err := regexp.Compile(s.Pattern); err != nil {
            return fmt.Errorf("invalid pattern regex: %w", err)
        }
    }
    return nil
}
```

#### Semantic Validation (Domain Layer)

```go
// internal/domain/frontmatter_service.go
func (s *FrontmatterService) IsSchemaCompliant(ctx context.Context, fm Frontmatter) error {
    // Look up schema (business logic)
    schema, err := s.schemaRegistry.GetSchema(ctx, fm.FileClass)
    if err != nil {
        return err
    }

    // Validate required fields (business rule)
    for _, prop := range schema.Properties {
        if prop.Required && !hasField(fm.Fields, prop.Name) {
            return RequiredFieldError{Property: prop.Name}
        }
    }

    // Validate against PropertySpec constraints (business rules)
    for name, value := range fm.Fields {
        prop := schema.GetProperty(name)
        if err := s.validateConstraints(value, prop.Spec); err != nil {
            return err
        }
    }

    return nil
}
```

### Decision Tree: Which Validation Layer?

```
Does validation require business context (schemas, domain rules)?
├─ YES → Domain Layer (Semantic)
│   ├─ Method name: Validate() or IsSchemaCompliant()
│   ├─ Location: Domain services (FrontmatterService, SchemaEngine)
│   └─ Examples: Schema compliance, required fields, type constraints
│
└─ NO → Adapter Layer (Syntactic)
    ├─ Method name: ValidateSyntax() or IsValidStructure()
    ├─ Location: Adapters (SchemaValidator, MarkdownParserAdapter)
    └─ Examples: YAML parsing, regex compilation, file existence
```

### Critical Rules

- Syntactic validation **MUST** occur in adapter layer before domain layer receives data
- Semantic validation **MUST** occur in domain layer after syntactic validation passes
- Adapters **MUST NOT** perform semantic validation (no schema lookups, no business rule checks)
- Domain services **MUST NOT** perform syntactic validation (no regex compilation, no YAML parsing)
- Method names **MUST** follow naming conventions to clearly indicate validation layer
- Cross-layer validation calls **MUST** flow adapter → domain (never domain → adapter)

### Validation Workflow Example

```go
// Adapter Layer: Syntactic validation
func (a *SchemaLoaderAdapter) Load(ctx context.Context) ([]Schema, error) {
    // 1. Parse JSON (syntactic)
    schemas, err := a.parseSchemaFiles()
    if err != nil {
        return nil, err
    }

    // 2. Validate syntax (adapter layer)
    validator := NewSchemaValidator()
    if err := validator.ValidateSyntax(schemas); err != nil {
        return nil, err
    }

    // 3. Pass to domain layer for semantic validation
    return schemas, nil
}

// Domain Layer: Semantic validation
func (e *SchemaEngine) Load(ctx context.Context) error {
    // 1. Load from adapter (syntactic validation already done)
    schemas, bank, err := e.schemaLoader.Load(ctx)
    if err != nil {
        return err
    }

    // 2. Validate business rules (semantic)
    for _, schema := range schemas {
        if err := schema.Validate(ctx); err != nil {
            return err
        }
    }

    // 3. Register for use by domain services
    e.registry.RegisterSchemas(schemas)
    return nil
}
```

**See Also:** `docs/architecture/components.md` contains detailed component documentation with validation layer responsibilities for each service and adapter.

---
