# Epic 6: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.
**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14
**Implementation Notes:**

- Schema adapters (Command/Query with embedded Loader/Writer) created in this epic
- **SchemaDecoder Strategy**: Modularized decoding logic in `decoder.rs` to normalize TOML/JSON/YAML into `RawSchema`
- **Syntactic Validation**: dedicated `validator.rs` for format-specific schema validation
- **Port Updates**: Add `load`/`refresh` methods to SchemaCommand, ensure `validate_note` is NOT in ports (App service only)
- **Architecture**: Domain models exist (Epic 3), adapters integrate Epic 4 utilities + Domain Resolver
- **Adapter Structure**: `crates/adapters/src/schema/` contains query.rs, command.rs, loader.rs, writer.rs, registry.rs, decoder.rs, cache.rs
- **Singleton Pattern**: Hybrid `Arc<OnceLock<PropertyBank>>` (immutable base) + `Arc<RwLock<T>>` (runtime overrides)
- **Caching Strategy**: Decoupled `SchemaCache` trait with Redb implementation

## Story 6.1: Update Schema Ports and Implement Adapters

As a developer completing the schema bounded context,
I want updated CQRS ports and robust adapters with embedded utilities,
So that schema loading, resolution, and caching are handled correctly behind clean interfaces.

**Acceptance Criteria:**

**Given** existing ports in `crates/domain/src/ports/schema.rs`
**When** I update the SchemaCommand trait
**Then** it includes `load_all()` and `refresh(name)` methods (mutating state)
**And** `validate_note` is removed (belongs in App layer)

**Given** SchemaQuery trait
**When** I review methods
**Then** `find_by_id`, `find_by_name`, and `list` remain read-only operations

**Given** updated ports
**When** I implement SchemaLoader utility in `crates/adapters/src/schema/loader.rs`
**Then** it coordinates file reading, decoding, and domain resolution

**Given** adapters are needed
**When** I implement SchemaCommand and SchemaQuery adapters
**Then** they embed Loader/Writer/Cache utilities and implement the updated traits

**Given** adapters are implemented
**When** I export them in `crates/adapters/src/schema/mod.rs`
**Then** internal structs are re-exported with Schema prefix (SchemaQuery, SchemaCommand, SchemaLoader, SchemaDecoder)

## Story 6.2: Implement Schema Loading and Resolution Strategy

As a developer loading schema files,
I want robust loading with format normalization and $ref resolution,
So that complex schema hierarchies are correctly resolved into usable Schema aggregates.

**Acceptance Criteria:**

**Given** I need to handle multiple formats (TOML, JSON, YAML)
**When** I implement SchemaDecoder utility in `crates/adapters/src/schema/decoder.rs`
**Then** it implements a strategy to normalize content into `RawSchema` using Epic 4 parsers

**Given** schemas must be syntactically valid
**When** I implement `crates/adapters/src/schema/validator.rs`
**Then** it validates the structure of parsed content before domain resolution
**And** provides helpful error messages with line/column context pointing to solutions

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

**Given** PropertyBank domain entity exists (Epic 3)
**When** I implement Registry in `crates/adapters/src/schema/registry.rs`
**Then** it wraps PropertyBank with singleton management

**Given** configuration needs both read performance and mutability
**When** I design the singleton implementation
**Then** implement hybrid approach: `Arc<OnceLock<PropertyBank>>` for immutable loaded bank + `Arc<RwLock<T>>` for runtime overrides

**Given** CLI operations need consistent property access
**When** I implement singleton instance method
**Then** `Registry::global()` returns the same instance across all calls

**Given** performance is critical
**When** I benchmark access
**Then** singleton reads complete in <10ns (zero-lock path for base properties)

**Given** Registry requires initialization
**When** I integrate with SchemaLoader
**Then** `load_all()` populates the registry from the `property_bank.json` file

## Story 6.4: Implement Decoupled Schema Caching

As a developer optimizing performance,
I want a decoupled caching architecture with Redb persistence,
So that schema resolution is fast, persistent, and testable.

**Acceptance Criteria:**

**Given** caching requirements
**When** I implement caching architecture
**Then** `SchemaCache` trait defines storage interface (get, put, invalidate) in `crates/adapters/src/schema/cache_trait.rs`

**Given** persistence is needed
**When** I implement `RedbSchemaCache` in `crates/adapters/src/schema/cache_redb.rs`
**Then** it implements the trait using Redb serialization

**Given** source files change
**When** I implement integrity checking
**Then** Blake3 hashes of source content are compared against cached entries

**Given** hash mismatch occurs
**When** I request a schema
**Then** the Loader triggers re-resolution and updates the cache

**Given** SchemaLoader uses cache
**When** I design the integration
**Then** it accepts `Box<dyn SchemaCache>`, allowing Redb to be mocked in tests

## Story 6.5: Implement Frontmatter Compliance Service

As a developer ensuring vault consistency,
I want a Frontmatter Compliance Service in the Application Layer,
So that notes can be validated against their corresponding schemas.

**Acceptance Criteria:**

**Given** Note aggregate and Schema aggregate
**When** I implement `crates/app/src/services/compliance.rs`
**Then** it provides `validate_note(note, schema)` method

**Given** a note with frontmatter
**When** I validate against a schema
**Then** the service checks if frontmatter fields match Schema Property constraints

**Given** validation runs
**When** discrepancies are found
**Then** the service returns a list of warnings (does not block note usage)

**Given** integration with SchemaQuery
**When** I validate a note
**Then** the service looks up the correct schema using the note's `fileClass`

## Story 6.6: Define Schema-Template Integration Contracts

As a developer integrating schemas with templates,
I want clear contracts for how schemas provide inputs to templates,
So that templates can safely access schema-defined properties.

**Acceptance Criteria:**

**Given** schemas define properties
**When** I define integration contracts
**Then** templates can access property values by schema name and property name

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
**Then** `crates/adapters/src/schema/README.md` provides quick start examples

**Given** documentation exists
**When** I review content
**Then** it explains the Decoder strategy, Registry singleton, and Cache decoupling clearly
