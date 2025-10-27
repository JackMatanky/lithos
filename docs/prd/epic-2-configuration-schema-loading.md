# Epic 2: Configuration & Schema Loading

**Epic Goal:** Implement the schema runtime exactly as documented in architecture v0.6.8 so Lithos can load configuration-driven schemas, validate them, resolve inheritance, and expose the results to downstream services.

**Dependencies:** Epic 1 (Foundational CLI & Template Engine)

**Architecture References:**
- `docs/architecture/components.md` v0.6.8 — SchemaEngine, SchemaValidator, SchemaResolver, SchemaPort, SchemaRegistryPort, ConfigPort
- `docs/architecture/data-models.md` v0.6.8 — Schema, Property, PropertySpec variants, PropertyBank
- `docs/architecture/error-handling-strategy.md` v0.5.9 — SchemaError, ValidationError
- `docs/architecture/coding-standards.md` v0.6.7 — `(T, error)` signatures, logging, testing

---

## Story 2.1: Implement Schema Domain Model

As a developer, I need the Schema domain struct to match the architecture definition so it can be validated and resolved without leaking infrastructure concerns.

### Acceptance Criteria
1. In `internal/domain/schema.go`, define `Schema` with exactly the four attributes in `data-models.md#schema`: `Name string`, `Extends string`, `Excludes []string`, `Properties []Property`.
2. Implement `Validate(ctx context.Context) error` exactly as described in `data-models.md#schema` (non-empty name, `Excludes` only when `Extends` is set, each excluded property present, and delegation to `Property.Validate`). Return `SchemaError` on violations.
3. Add unit tests (`internal/domain/schema_test.go`) covering success, missing name, invalid excludes, and property validation delegation.
4. Run `golangci-lint run ./internal/domain` and `go test ./internal/domain`.

---

## Story 2.2: Implement Property & PropertySpec Domain Models

As a developer, I want Property and PropertySpec to reflect their documented structure so Schema.Validate can delegate correctly.

### Acceptance Criteria
1. In `internal/domain/property.go`, define `Property` with attributes `Name string`, `Required bool`, `Array bool`, `Spec PropertySpec`.
2. Implement `Validate(ctx context.Context) error` that checks name is non-empty, `Spec != nil`, and calls `Spec.Validate(ctx)`; return `SchemaError` or `ValidationError` when violations occur.
3. In `internal/domain/property_spec.go`, declare `PropertySpec` interface with methods `Type() PropertyType` and `Validate(ctx context.Context) error` as described in `data-models.md#propertyspec`.
4. Implement the documented variants (`StringSpec`, `NumberSpec`, `BoolSpec`, `DateSpec`, `FileSpec`) with the fields listed in the data-models table, ensuring each variant validates its own configuration (e.g., regex compilation, enum non-empty, min ≤ max).
5. Add unit tests for each PropertySpec variant validating both acceptable and error cases, plus tests for `Property.Validate`.
6. Run `golangci-lint run ./internal/domain` and `go test ./internal/domain`.

---

## Story 2.3: Implement PropertyBank Domain Model

As a developer, I need a PropertyBank representation that captures the singleton registry of reusable properties described in the architecture.

### Acceptance Criteria
1. Create `internal/domain/property_bank.go` with struct `PropertyBank` holding `Properties map[string]Property`.
2. Provide `NewPropertyBank(props map[string]Property) (PropertyBank, error)` that ensures the map is non-nil, keys are non-empty, and each stored property passes `Property.Validate` when constructed.
3. Unit tests confirm constructor validation and successful JSON unmarshalling using the examples from `data-models.md`.
4. Run `golangci-lint run ./internal/domain` and `go test ./internal/domain`.

---

## Story 2.4: Define SchemaPort Interface & Filesystem Loader Adapter

As a developer, I want the SchemaPort SPI boundary and its filesystem adapter so schemas/property bank are loaded per architecture.

### Acceptance Criteria
1. In `internal/ports/spi/schema.go`, declare `SchemaPort` with signature `Load(ctx context.Context) ([]domain.Schema, domain.PropertyBank, error)` and GoDoc referencing `components.md#schemaport`.
2. Implement `internal/adapters/spi/schema/loader.go` with a constructor accepting `domain.Config`, an `fs.FS` (or equivalent), and a logger.
3. `Load` must:
   - Read property bank JSON (`Config.PropertyBankFile` within `Config.SchemasDir`) into `PropertyBank`.
   - Read every `.json` schema file in `Config.SchemasDir` (non-recursive per architecture notes) and unmarshal into `Schema`.
   - Return slices of raw schemas (unresolved) along with the property bank.
4. Wrap IO/JSON issues with `ResourceError` or `SchemaError` including the file path context.
5. Unit tests cover successful load, missing property bank, malformed schema file, and duplicate schema filenames.
6. Run `golangci-lint run ./internal/adapters/spi/schema ./internal/ports/spi` and `go test ./internal/adapters/spi/schema ./internal/ports/spi`.

---

## Story 2.5: Define SchemaRegistryPort & In-Memory Adapter

As a developer, I want an in-memory registry adapter so resolved schemas and properties can be queried via the documented interface.

### Acceptance Criteria
1. In `internal/ports/spi/schema_registry.go`, declare `SchemaRegistryPort` with methods `RegisterAll(ctx context.Context, schemas []domain.Schema, bank domain.PropertyBank) error`, `GetSchema(ctx context.Context, name string) (domain.Schema, error)`, `GetProperty(ctx context.Context, name string) (domain.Property, error)`, `HasSchema(ctx context.Context, name string) bool`, and `HasProperty(ctx context.Context, name string) bool`.
2. Implement adapter `internal/adapters/spi/schema/registry.go` storing copies of schemas/properties in maps guarded by `sync.RWMutex`.
3. Unknown lookups must return `SchemaError` tagged with the missing identifier.
4. Unit tests verify registration idempotency, concurrency safety (use `t.Parallel()` and run with `-race`), and not-found error messaging.
5. Run `golangci-lint run ./internal/adapters/spi/schema ./internal/ports/spi` and `go test ./internal/adapters/spi/schema ./internal/ports/spi`.

---

## Story 2.6: Implement SchemaValidator Service

As a developer, I want SchemaValidator to orchestrate model-level and cross-schema validation exactly as described in the architecture.

### Acceptance Criteria
1. Create `internal/app/schema/validator.go` defining `type Validator struct {}` with method `ValidateAll(ctx context.Context, schemas []domain.Schema, bank domain.PropertyBank) error`.
2. Implementation must:
   - Call `schema.Validate(ctx)` for each schema and accumulate errors.
   - Ensure `Extends` references refer to loaded schemas.
   - Ensure `$ref` references point to entries in PropertyBank.
   - Detect duplicate schema names.
   - Aggregate failures using `errors.Join`.
3. Unit tests cover happy path, missing parent schema, missing `$ref`, duplicate schema names, and aggregated error output.
4. Run `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema`.

---

## Story 2.7: Implement SchemaResolver Service

As a developer, I need SchemaResolver to perform inheritance resolution and `$ref` substitution according to the algorithm in the architecture doc.

### Acceptance Criteria
1. Add `internal/app/schema/resolver.go` with `func Resolve(ctx context.Context, schemas []domain.Schema, bank domain.PropertyBank) ([]domain.Schema, error)` (free function or struct) matching `components.md#schemaresolver`.
2. Implementation must:
   - Detect circular inheritance chains and return `SchemaError` containing the cycle path.
   - Produce a topologically sorted order before merging.
   - For each schema: start with parent properties (if any), remove `Excludes`, merge child `Properties`, and substitute `$ref` entries using the PropertyBank map (simple replacement, no overrides).
   - Return new slices with resolved properties (original schemas remain unchanged).
3. Unit tests exercise multi-level inheritance, excludes removing parent properties, cycle detection, and missing `$ref`.
4. Run `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema`.

---

## Story 2.8: Implement SchemaEngine Orchestrator

As a developer, I want SchemaEngine to orchestrate loading, validation, resolution, and registration in the sequence mandated by the architecture.

### Acceptance Criteria
1. Implement `internal/app/schema/engine.go` with `type Engine struct` holding a `SchemaPort`, `SchemaRegistryPort`, and logger. Internally instantiate `Validator` and `Resolver` (per architecture guidance—do not inject them).
2. Provide `func NewEngine(port schema.SchemaPort, registry schema.SchemaRegistryPort, log zerolog.Logger) *Engine`.
3. `Load(ctx context.Context)` must:
   - Call `port.Load(ctx)` to obtain raw schemas and property bank.
   - Invoke `validator.ValidateAll`.
   - Call `resolver.Resolve` to produce resolved schemas.
   - Register results through `registry.RegisterAll`.
   - Log each stage with duration at info level.
4. Expose helper methods `GetSchema/HasSchema/GetProperty/HasProperty` that delegate to the registry.
5. Unit tests use fakes to assert call order, error propagation, and logging.
6. Run `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema`.

---

## Story 2.9: Bootstrap Schema Runtime in CLI Startup

As a developer, I need the CLI composition root to initialize the schema runtime before services (like FrontmatterService and VaultIndexer) use it.

### Acceptance Criteria
1. Update the CLI startup (e.g., `cmd/lithos/main.go`) to:
   - Load configuration via ConfigPort.
   - Construct SchemaLoaderAdapter (using the OS filesystem and configuration paths).
   - Construct SchemaRegistryAdapter.
   - Instantiate Engine via `NewEngine` and call `Load` before wiring downstream services.
2. Ensure initialization failures print actionable messages to stderr and exit with non-zero status.
3. Add or update integration test(s) verifying successful startup with sample schemas and failure when property bank is missing.
4. Document the wiring briefly (code comment) referencing `components.md#schemaengine`.

---

## Story 2.10: Establish Schema Runtime Contract Tests

As a QA-focused developer, I want regression tests that exercise the full schema runtime so future changes can be caught early.

### Acceptance Criteria
1. Add `tests/integration/schema_runtime_test.go` that boots Engine with real adapters against fixtures in `testdata/schemas/` (valid set, missing parent schema, missing `$ref`, circular inheritance).
2. Store expected error outputs in `testdata/golden/schema/*.txt` and assert them in tests.
3. Update `docs/architecture/testing-strategy.md` to mention the new integration suite and the command `go test ./tests/integration -run SchemaRuntime`.
4. Ensure CI/DoD checklist references running the schema runtime integration suite.
