# Epic 6: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**

- Schema adapters (Command/Query with embedded Loader/Writer) created in this epic
- **SchemaDecoder Strategy**: Modularized decoding logic in `decoder.rs` that normalizes the output of **FormatDispatcher** into a standard `RawSchema` aggregate, resolving format-specific syntax variations in property references ($refs).
- **Syntactic Validation**: dedicated `validator.rs` for format-specific schema validation
- **Port Updates**: Add `load`/`refresh` methods to SchemaCommand, ensure `validate_note` is NOT in ports (App service only)
- **Architecture**: Domain models exist (Epic 3), adapters integrate Epic 4 PathValidator (security) and FormatDispatcher (data processing) + Domain Resolver.
- **Adapter Structure**: `crates/adapters/src/spi/schema/` contains query.rs, command.rs, loader.rs, writer.rs, registry.rs, decoder.rs, cache.rs
- **Singleton Pattern**: Hybrid `Arc<OnceLock<PropertyBank>>` (immutable base) + `Arc<RwLock<T>>` (runtime overrides)
- **Caching Strategy**: Decoupled `SchemaCache` trait with Redb implementation

## Story 6.1: Update Schema Ports and Implement Adapters

As a developer completing the schema bounded context,
I want updated CQRS ports and robust adapters with embedded utilities,
So that schema loading, resolution, and caching are handled correctly behind clean interfaces.

**Acceptance Criteria:**

**Given** existing ports in `crates/domain/src/ports/schema.rs`
**When** I update `SchemaCommand` trait
**Then** `load_all()` signature is `async fn load_all(&self) -> Result<(), SchemaError>`
**And** `refresh(name)` signature is `async fn refresh(&self, name: &str) -> Result<(), SchemaError>`

**Given** `SchemaQuery` trait
**When** I review methods
**Then** `get(name)` returns `Result<Option<Schema>, SchemaError>` to handle missing schemas gracefully
**And** `list()` returns `Result<Vec<SchemaName>, SchemaError>`
**And** SchemaQuery methods remain read-only and side-effect free

**Given** `SchemaLoader` implementation
**When** I design the architecture
**Then** adapters coordinate between loaders/writers without refactoring I/O layer (extensibility behavior)

**Given** `SchemaLoader` implementation
**When** I implement the structure
**Then** it holds a reference to `Box<dyn SchemaCache>` for decoupled storage logic
**And** it initializes with a root path provided by `Config`

**Given** `SchemaLoader` implementation
**When** I implement file loading
**Then** it uses `PathValidator` (Strict or Flexible mode based on config) to ensure path security before attempting I/O.

**Given** `SchemaLoader` has validated and read a file
**When** it needs to deserialize the content
**Then** it delegates the parsing to `FormatDispatcher` before passing the resulting data to the `SchemaDecoder` for normalization.

**Given** `SchemaCommand` adapter
**When** I implement error handling
**Then** `SchemaError` includes specific variants for IO, parsing, resolution, and cache failures with context

**Given** adapters are needed
**When** I implement SchemaCommand and SchemaQuery adapters
**Then** they embed Loader/Writer/Cache utilities and implement the updated traits

**Given** adapters are implemented
**When** I export them in `crates/adapters/src/spi/schema/mod.rs`
**Then** internal structs are re-exported with Schema prefix (SchemaQuery, SchemaCommand, SchemaLoader, SchemaDecoder)

## Story 6.2: Implement Schema Loading and Resolution Strategy

As a developer loading schema files,
I want robust loading with format normalization and $ref resolution,
So that complex schema hierarchies are correctly resolved into usable Schema aggregates.

**Acceptance Criteria:**

**Given** `SchemaCommand` adapter implements the domain port
**When** it executes a load operation
**Then** it orchestrates the flow: `SchemaLoader` (I/O) → `FormatDispatcher` (Parsing) → `SchemaDecoder` (Normalization)

**Given** `SchemaDecoder` receives parsed data from `FormatDispatcher`
**When** it normalizes the output into a `RawSchema`
**Then** it resolves format-specific syntax variations for property references (`$refs`)
**And** it enforces a unified internal representation for property definitions regardless of the source format's idiosyncratic nesting.

**Given** a schema uses inheritance or property references (`$refs`)
**When** the `SchemaDecoder` processes the references
**Then** it handles syntax variations (e.g., string-based vs. object-based refs) and normalizes them into standard domain reference types before domain aggregate construction.

**Given** `SchemaCommand` requires valid source data
**When** I implement syntactic validation in `crates/adapters/src/spi/schema/validator.rs`
**Then** it performs a structural compliance check to ensure the document contains all mandatory schema keys (e.g., version, metadata) before domain objects are initialized.

**Given** the normalized RawSchema
**When** the adapter performs a schema compliance check
**Then** it ensures that all property definitions follow the expected structural types (e.g., ensuring a Number spec doesn't contain String constraints).

**Given** a syntactic or compliance failure
**When** reporting errors
**Then** it provides the file path, line number, and column number using `miette::SourceSpan` derived from the original source passed through the orchestration chain.

**Given** `SchemaLoader` logic
**When** I implement resolution loop
**Then** it detects and reports circular dependencies (A extends B extends A) as a critical error
**And** it continues processing other independent schemas even if one fails (resilience behavior)

**Given** validation errors occur
**When** I format error output
**Then** messages are helpful, pointing the user to the exact line/column and suggesting possible solutions (UX behavior)

**Given** RawSchemas are loaded
**When** I use the domain `SchemaGraph` and `SchemaResolver` (from Epic 3)
**Then** inheritance chains (extends/excludes) are resolved in topological order

**Given** resolution is complete
**When** I store results
**Then** fully resolved `Schema` aggregates are passed to the Registry/Cache

## Story 6.3: Implement PropertyBank Singleton Registry

As a developer implementing the schema bounded context,
I want a PropertyBankRegistry that provides singleton instance management,
So that all operations access the same reusable property definitions consistently.

**Acceptance Criteria:**

**Given** `Registry` implementation
**When** I design the internal structure
**Then** `base: Arc<OnceLock<PropertyBank>>` holds the immutable bank loaded from disk
**And** `overrides: Arc<RwLock<HashMap<String, Property>>>` holds runtime-defined properties

**Given** access patterns
**When** I implement `lookup(key)`
**Then** it checks `overrides` first (read lock), then falls back to `base` (no lock)
**And** this strategy ensures zero-lock contention for standard properties

**Given** initialization flow
**When** `SchemaLoader::load_all()` completes
**Then** it calls `Registry::init(bank)` which succeeds only once
**And** subsequent calls return an error or are ignored

**Given** CLI operations need consistent property access
**When** I implement singleton instance method
**Then** `Registry::global()` returns the same instance across all calls

**Given** hot reloading will be needed
**When** I design the singleton implementation
**Then** Registry supports atomic updates using AtomicPtr swap pattern (concurrency behavior)

**Given** performance is critical
**When** I benchmark access
**Then** singleton reads complete in <10ns (zero-lock path for base properties)

## Story 6.4: Implement Decoupled Schema Caching

As a developer optimizing performance,
I want a decoupled caching architecture with Redb persistence,
So that schema resolution is fast, persistent, and testable.

**Acceptance Criteria:**

**Given** caching requirements
**When** I implement caching architecture
**Then** `SchemaCache` trait defines storage interface (get, put, invalidate) in `crates/adapters/src/spi/schema/cache_trait.rs`

**Given** `RedbSchemaCache` adapter
**When** I implement storage
**Then** it uses a dedicated Redb table `schemas` with `String` (name) keys
**And** values are rkyv-serialized `Schema` aggregates for zero-copy deserialization

**Given** cache invalidation logic
**When** source file hash changes
**Then** `get()` returns `CacheMiss` to trigger reload
**And** `put()` overwrites the existing entry with new hash and schema data

**Given** unit testing requirement
**When** I implement `MockSchemaCache`
**Then** it stores schemas in a `HashMap` for fast, filesystem-free testing

**Given** SchemaLoader uses cache
**When** I design the integration
**Then** it accepts `Box<dyn SchemaCache>`, allowing Redb to be mocked in tests

## Story 6.5: Implement Frontmatter Compliance Service

As a developer ensuring vault consistency,
I want a Frontmatter Compliance Service in the Application Layer,
So that notes can be validated against their corresponding schemas.

**Acceptance Criteria:**

**Given** `validate_note` method
**When** checking a required string property
**Then** it verifies the field exists in frontmatter and is a non-empty string
**And** returns `ComplianceWarning::MissingField` if absent

**Given** `validate_note` method
**When** checking a number property with range (min 1, max 10)
**Then** it verifies the value is within bounds
**And** returns `ComplianceWarning::ValueOutOfRange` if 11 is provided

**Given** validation runs
**When** discrepancies are found
**Then** the service returns a list of warnings (does not block note usage)

**Given** integration with `SchemaQuery`
**When** note has `fileClass: "project"`
**Then** service calls `query.get("project")`
**And** if schema is missing, returns `ComplianceWarning::SchemaNotFound` (non-blocking)

## Story 6.6: Define Schema-Template Integration Contracts

As a developer integrating schemas with templates,
I want clear contracts for how schemas provide inputs to templates,
So that templates can safely access schema-defined properties.

**Acceptance Criteria:**

**Given** `SchemaTemplateContract` trait
**When** I define the interface
**Then** it exposes `get_variable_constraints(schema_name)` returning a map of variable rules
**And** allows templates to validate user input against schema rules

**Given** template rendering context
**When** I implement variable lookup
**Then** schema default values are available as fallback for missing frontmatter fields

**Given** integration contracts exist
**When** templates reference schema properties
**Then** type-safe access is provided with validation

**Given** contracts are defined
**When** I validate against Epic 11 template requirements
**Then** all template input needs are satisfied by schema contracts

## Story 6.7: Review Epic 6 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 6 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** docs/testing/developer-guide.md
**When** I review tests
**Then** I ensure compliance with async patterns and fixture usage

**Given** all public components (Decoder, Loader, Registry, Cache)
**When** I verify coverage
**Then** unit tests exist for every public method

**Given** format strategies (TOML/JSON/YAML)
**When** I test Decoder
**Then** parameterized tests verify identical `RawSchema` output for equivalent inputs in different formats

**Given** inheritance logic
**When** I test resolution
**Then** integration tests verify multi-level inheritance matches expectations

## Story 6.8: Create Default Schema Files

As a user creating schemas,
I want default schema files demonstrating all features,
So that I can understand schema capabilities and use them as templates.

**Acceptance Criteria:**

**Given** I need default schemas
**When** I create JSON/TOML/YAML examples in `docs/defaults/schemas/`
**Then** examples cover all PropertySpec types (string, number, bool, date, file)

**Given** inheritance is a key feature
**When** I create default hierarchy
**Then** examples demonstrate `base-note` -> `note` -> `project-note` inheritance chain

**Given** PropertyBank is required
**When** I create `property_bank.json`
**Then** it contains reusable common properties (title, tags, created_date)

**Given** defaults are created
**When** I test with SchemaLoader
**Then** all default files resolve correctly without errors

## Story 6.9: Document Schema Adapters

As a developer working with the schema system,
I want comprehensive documentation for the adapter layer,
So that I understand how loading, resolution, and caching interact.

**Acceptance Criteria:**

**Given** Epic 6 implementation is complete
**When** I create developer documentation
**Then** `docs/adapters/schema-adapters.md` is created following the domain-models pattern

**Given** module structure
**When** I create README
**Then** `crates/adapters/src/spi/schema/README.md` provides quick start examples

**Given** documentation exists
**When** I review content
**Then** it explains the Decoder strategy, Registry singleton, and Cache decoupling clearly
