# Epic 2: Configuration & Schema Loading

## Status

Complete

This epic establishes the schema runtime exactly as documented in architecture v0.6.8. Completing it ensures Lithos loads configuration-driven schemas, validates them, resolves inheritance, and exposes canonical definitions to every downstream service. The stories progress from core domain models through SPI adapters to full runtime bootstrapping and regression coverage.

---

## Story 2.1 Schema Domain Model

As a developer,
I want the Schema domain struct to match the architecture specification,
so that validation and resolution operate on canonical schema data.

**Prerequisites:** None.

### Acceptance Criteria
1. `internal/domain/schema.go` exposes `Schema` with exactly the four attributes mandated in `docs/architecture/data-models.md#schema`, including matching JSON/YAML tags.
2. `Schema.Validate(ctx)` enforces the architecture rules (non-empty name, excludes only when extends is set, excludes target existing properties, delegates to `Property.Validate`) and returns `SchemaError` instances conforming to `docs/architecture/error-handling-strategy.md`.
3. Constructor or helper functions preserve immutability expectations noted in `docs/architecture/components.md#schemaengine` (no shared mutation across slices/maps).
4. Unit tests in `internal/domain/schema_test.go` cover success, missing name, invalid excludes, duplicate property detection, and ensure validation errors include schema identifiers per FR5 in `docs/prd/requirements.md`.
5. Linting and tests (`golangci-lint run ./internal/domain`, `go test ./internal/domain`) are executed and documented in story notes.

---

## Story 2.2 Property & PropertySpec Models

As a developer,
I want Property and PropertySpec models that mirror the architecture definitions,
so that schema validation can rely on consistent constraint behaviour.

**Prerequisites:** Story 2.1.

### Acceptance Criteria
1. `internal/domain/property.go` defines `Property` fields exactly as listed in `docs/architecture/data-models.md#property`, including JSON tags and required/array semantics.
2. `Property.Validate(ctx)` enforces architecture rules and returns error types consistent with `docs/architecture/error-handling-strategy.md` while preserving unknown metadata (FR6 in `docs/prd/requirements.md`).
3. `internal/domain/property_spec.go` declares `PropertySpec` (`Type()`, `Validate(ctx)`) and implements each variant (String, Number, Bool, Date, File) with the documented attributes (regex, step, enum, directory, fileClass) from `data-models.md#propertyspec`.
4. Each PropertySpec includes GoDoc referencing the relevant architecture component and ensures immutability/value-object semantics per `docs/architecture/components.md#propertyspec`.
5. Unit tests cover constructor success, configuration validation failures, type-specific validation rules, and ensure errors include property identifiers satisfying FR7 in `docs/prd/requirements.md`.
6. `golangci-lint run ./internal/domain` and `go test ./internal/domain` pass and are recorded in story notes.

---

## Story 2.3 PropertyBank Domain Model

As a developer,
I want a PropertyBank representation that captures the singleton registry of reusable properties,
so that schemas can reference shared definitions via `$ref`.

**Prerequisites:** Stories 2.1, 2.2.

### Acceptance Criteria
1. `internal/domain/property_bank.go` defines `PropertyBank` with the shape documented in `docs/architecture/data-models.md#propertybank`, ensuring the singleton semantics described in `docs/architecture/components.md#propertybank`.
2. `NewPropertyBank` validates non-empty keys, invokes `Property.Validate`, and returns informative `SchemaError` messages when configuration issues occur.
3. Provide helper `Lookup(id string) (Property, bool)` that preserves read-only access without exposing internal maps, aligning with FR5 requirements.
4. Unit tests cover constructor validation, lookup behaviour, and unmarshalling from example fixtures in `testdata/schemas/property_bank.json` (or equivalent) ensuring compatibility with FR8 (`docs/prd/requirements.md`).
5. `golangci-lint run ./internal/domain` and `go test ./internal/domain` succeed and are documented.

---

## Story 2.4 SchemaPort Interface & Loader Adapter

As a developer,
I want the SchemaPort SPI and filesystem loader,
so that schemas and the property bank load through the documented boundary.

**Prerequisites:** Stories 2.1–2.3.

### Acceptance Criteria
1. `internal/ports/spi/schema.go` declares `SchemaPort` per `docs/architecture/components.md#schemaport`, including GoDoc that references the architecture section and FR5/FR9 linkage.
2. `internal/adapters/spi/schema/loader.go` resolves file paths using `domain.Config` semantics, loads the property bank before schemas, and returns raw `[]Schema` plus `PropertyBank` while preserving unknown fields (FR6).
3. Errors wrap in `SchemaError` or `ResourceError` with file path context consistent with `docs/architecture/error-handling-strategy.md` and include remediation hints.
4. Unit tests cover successful load, missing property bank, malformed schema JSON, duplicate schema names, and verify that property bank is loaded exactly once per process.
5. `golangci-lint run ./internal/adapters/spi/schema ./internal/ports/spi` and `go test ./internal/adapters/spi/schema ./internal/ports/spi` pass.

---

## Story 2.5 SchemaRegistryPort & Adapter

As a developer,
I want an in-memory schema registry adapter,
so that resolved schemas and properties are accessible to downstream services.

**Prerequisites:** Story 2.4.

### Acceptance Criteria
1. `internal/ports/spi/schema.go` declares methods exactly as documented in `docs/architecture/components.md#schemaregistryport` and GoDoc references FR5/FR7.
2. `internal/adapters/spi/schema/registry.go` stores defensive copies, ensures thread safety via `sync.RWMutex`, and returns `SchemaError` with `ErrNotFound` classification when lookups fail.
3. Adapter exposes `RegisterAll` idempotently, clearing stale entries before registration to avoid drift with FR9.
4. Unit tests cover concurrent read access, duplicate registration, missing schema/property lookups, and confirm that returned schemas/properties cannot mutate internal state.
5. `golangci-lint run ./internal/adapters/spi/schema ./internal/ports/spi` and `go test ./internal/adapters/spi/schema ./internal/ports/spi` succeed.

---

## Story 2.6 SchemaValidator Service

As a developer,
I want SchemaValidator to orchestrate model and cross-schema validation,
so that invalid schemas are rejected before runtime proceeds.

**Prerequisites:** Stories 2.1–2.5.

### Acceptance Criteria
1. `internal/app/schema/validator.go` implements `ValidateAll(ctx, schemas)` exactly as in `docs/architecture/components.md#schemavalidator`, aggregating model validation, extends checks, `$ref` checks, and duplicate name detection.
2. The validator returns aggregated `SchemaError`/`ValidationError` instances with contextual messages that reference offending schema/property names (FR5, FR7).
3. Logging and metrics hooks (if present) follow guidance in `docs/architecture/components.md#schemaengine` (debug-level details only when enabled).
4. Unit tests cover valid input, missing parent schema, missing `$ref`, duplicate names, and ensure errors use `errors.Join` semantics.
5. `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema` succeed.

---

## Story 2.7 SchemaResolver Service

As a developer,
I want SchemaResolver to resolve inheritance and `$ref` substitution,
so that downstream consumers receive flattened schemas.

**Prerequisites:** Stories 2.1–2.6.

### Acceptance Criteria
1. `internal/app/schema/resolver.go` implements the algorithm from `docs/architecture/components.md#schemaresolver`, including topological sorting, cycle detection with informative error messages, property merge semantics, and `$ref` substitution using PropertyBank.
2. Resolver preserves original schemas (no mutation) and returns resolved copies with `ResolvedProperties` populated to satisfy FR5 and FR9.
3. Unit tests cover multi-level inheritance, excludes removing parent properties, cycle detection, missing `$ref`, and verify resolved schemas respect override semantics.
4. `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema` succeed.

---

## Story 2.8 SchemaEngine Orchestrator

As a developer,
I want SchemaEngine to orchestrate load, validation, resolution, and registration,
so that one call prepares schemas for use.

**Prerequisites:** Stories 2.4–2.7.

### Acceptance Criteria
1. `internal/app/schema/engine.go` constructs `Engine` with injected `SchemaPort`, `SchemaRegistryPort`, and logger, instantiating Validator and Resolver internally per `docs/architecture/components.md#schemaengine`.
2. `Load(ctx)` executes the documented order (SchemaPort.Load → Validator.ValidateAll → Resolver.Resolve → Registry.RegisterAll) and logs stage durations to satisfy NFR3 indexing observability.
3. Accessor methods (`GetSchema`, `HasSchema`, `GetProperty`, `HasProperty`) delegate to the registry and surface errors consistent with `error-handling-strategy.md`.
4. Unit tests with fakes verify call order, error propagation at each stage, logging behaviour, and that resolved schemas are registered.
5. `golangci-lint run ./internal/app/schema` and `go test ./internal/app/schema` succeed.

---

## Story 2.9 CLI Schema Bootstrapping

As a developer,
I want the CLI startup to initialize the schema runtime,
so that every command operates with validated schemas.

**Prerequisites:** Stories 2.4–2.8.

### Acceptance Criteria
1. CLI composition root loads configuration via ConfigPort, constructs SchemaLoaderAdapter and SchemaRegistryAdapter, instantiates SchemaEngine, and calls `Load` before initializing services that depend on schemas (FrontmatterService, VaultIndexer) as required by `components.md#commandorchestrator`.
2. Initialization failures emit actionable messages (stderr) and exit non-zero, referencing remediation guidance in `docs/architecture/error-handling-strategy.md`.
3. Integration/smoke tests exercise startup with sample schemas and the failure path when the property bank is missing, satisfying FR5 and NFR1 (macOS focus).

---

## Story 2.10 Schema Runtime Contract Tests

As a QA-focused developer,
I want regression tests for the full schema runtime,
so that future changes cannot silently break schema loading.

**Prerequisites:** Stories 2.4–2.9.

### Acceptance Criteria
1. `tests/integration/schema_runtime_test.go` boots SchemaEngine with real adapters and fixtures covering valid schemas, missing parents, missing `$ref`, circular inheritance, and invalid property bank entries, matching scenarios in `docs/architecture/components.md#schemaengine`.
2. Golden files capture expected error output; the test asserts error text includes remediation hints as defined in `error-handling-strategy.md` and satisfies FR5/FR7 regression requirements.
3. `docs/architecture/testing-strategy.md` documents how to run `go test ./tests/integration -run SchemaRuntime` and references fixture layout.
