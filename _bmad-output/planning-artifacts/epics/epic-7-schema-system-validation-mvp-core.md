# Epic 7: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.

**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14

## Implementation Notes

- **User Schema Format**: Simple object-based properties (`"properties": { "name": {...} }`)
- **Internal Domain Format**: Array-based properties after Decoder transformation
- **Schema Validation File**: `vault-schema.schema.json` validates user schema files
- **Example Vault**: `example_vault/` provides starter kit + test fixtures (based on `docs/refs/obsidian`)
- **Decoder Strategy**: Normalizes user format (object) → domain format (array)
- **Validator**: JSON Schema + semantic validation against vault-schema.schema.json
- **SchemaResolver**: Handles inheritance (extends/excludes) + circular dependency detection
- **Singleton Pattern**: `Arc<OnceLock<PropertyBank>>` (immutable) + `Arc<RwLock<HashMap>>` (runtime overrides)
- **Caching Strategy**: CQRS-split cache traits (CacheQuery/CacheCommand) with Redb implementation (Epic 5)
- **Adapter Structure**: `crates/adapters/src/spi/schema/` contains query.rs, command.rs, loader.rs, decoder.rs, validator.rs, cache.rs, registry.rs
- **Note:** Frontmatter validation moved to Epic 10.6 (application layer)
- **Note:** Schema-template integration moved to Epic 12.4 (template system)

---

## Story 7.1: Fix vault-schema.schema.json for User Format

As a developer setting up Epic 7,
I want corrected vault-schema.schema.json that validates user schema format,
So that validation matches user format expectations.

**Acceptance Criteria:**

### **Schema Validation File:**

**Given** users write schemas in simple object format
**When** I create vault-schema.schema.json
**Then** it validates user schema files with properties as object (not array)
**And** universal attributes (`type`, `required`, `array`) are top-level fields
**And** type-specific constraints are optional fields validated per type
**And** JSON Schema `allOf` + `if/then` prevents invalid constraint combinations

**Given** vault-schema.schema.json structure
**When** I define InlineProperty
**Then** it has single definition (not separate per type)
**And** `type` field is enum: ["string", "number", "bool", "date", "file"]
**And** `required` and `array` are boolean fields (default: false)
**And** string constraints: `enum` (array of strings), `pattern` (regex)
**And** number constraints: `min`, `max`, `step` (all numbers)
**And** date constraints: `format` (string)
**And** file constraints: `file_class`, `directory` (both strings)

**Given** type-specific validation
**When** I implement JSON Schema validation rules
**Then** `type: "string"` allows only `enum` and `pattern` (not min/max/step/format/file_class/directory)
**And** `type: "number"` allows only `min`, `max`, `step` (not enum/pattern/format/file_class/directory)
**And** `type: "bool"` allows NO type-specific constraints
**And** `type: "date"` allows only `format` (not enum/pattern/min/max/step/file_class/directory)
**And** `type: "file"` allows only `file_class`, `directory` (not enum/pattern/min/max/step/format)

**Given** PropertyRef support
**When** I define reference structure
**Then** `$ref` field matches pattern `^#/properties/[a-zA-Z0-9_-]+$`
**And** PropertyRef objects contain ONLY `$ref` field (additionalProperties: false)

**Given** schema-level structure
**When** I define Schema object
**Then** required fields: `name` (string matching `^[a-zA-Z0-9_-]+$`), `properties` (object)
**And** optional fields: `extends` (string), `excludes` (array of strings)
**And** `properties` is object with patternProperties matching `^[a-zA-Z0-9_-]+$`
**And** each property value is oneOf: [PropertyRef, InlineProperty]

**Given** validation examples
**When** I add examples to vault-schema.schema.json
**Then** examples demonstrate all property types (string, number, bool, date, file)
**And** examples show $ref usage referencing PropertyBank
**And** examples show inheritance (extends/excludes)
**And** examples show type-specific constraints (enum, pattern, min/max, format, file_class)

**Given** old lithos.schema.json exists
**When** I replace it
**Then** DELETE `docs/schemas/lithos.schema.json`
**And** CREATE `example_vault/.lithos/schemas/vault-schema.schema.json`
**And** remove internal domain concepts (Property.id, PropertySpec, resolved_properties)

---

## Story 7.2: Create example_vault with Default Schemas

As a developer setting up Epic 7,
I want example vault with schemas and sample notes,
So that I have starter kit + test fixtures based on real vault data.

**Acceptance Criteria:**

**Given** need for user starter kit and test fixtures
**When** I create example_vault/ structure
**Then** it follows the structure documented in `docs/obsidian-vault-guide.md`
**And** schemas are located in `example_vault/.lithos/schemas/`
**And** sample notes demonstrate schema usage in `example_vault/notes/`
**And** includes `.gitignore` ignoring `.lithos/cache/` but preserving schemas and templates

**Given** schemas exist in docs/refs/obsidian or docs/schemas/
**When** I populate example_vault
**Then** migrate existing schemas to `example_vault/.lithos/schemas/`
**And** include `vault-schema.schema.json` for validation
**And** include `property_bank.json` with reusable properties
**And** all 41+ schemas are available for testing

**Given** example notes are needed
**When** I create sample notes
**Then** provide examples demonstrating: basic schema usage, inheritance (extends), PropertyBank references ($ref)
**And** notes have valid frontmatter matching their schemas
**And** notes demonstrate realistic vault patterns

**Given** test fixture accessibility
**When** I set up test infrastructure
**Then** tests can reference `example_vault/` as fixtures via constant
**And** all schema files are readable at test time
**And** directory structure is preserved in version control

**Given** documentation is needed
**When** I create example_vault/README.md
**Then** document available schemas with inheritance chains
**And** explain usage as starter kit (copy to new vault)
**And** explain usage as test fixtures for Epic 7+ testing

---

## Story 7.3: Implement SchemaLoader with PathValidator

As a developer loading schema files,
I want robust file loading with security validation,
So that schema files are safely read from disk before processing.

**Acceptance Criteria:**

**Given** I need secure schema file loading
**When** I create SchemaLoader in `crates/adapters/src/spi/schema/loader.rs`
**Then** it integrates Epic 4 PathValidator for security checks
**And** it integrates Epic 4 FormatDispatcher for JSON/TOML/YAML parsing
**And** it accepts root_path from Config (e.g., `vault/.lithos/schemas/`)

**Given** SchemaLoader loads individual files
**When** I implement load_file(file_name)
**Then** it validates path security via PathValidator before reading
**And** it parses content via FormatDispatcher (auto-detects JSON/TOML/YAML)
**And** it returns parsed `serde_json::Value` with source path
**And** it returns SchemaError for: I/O failures, parse errors, path validation failures

**Given** SchemaLoader loads all schemas
**When** I implement load_all()
**Then** it scans directory for `*.json`, `*.toml`, `*.yaml`, `*.yml` files
**And** it EXCLUDES `vault-schema.schema.json` (validation schema, not user schema)
**And** it continues processing if individual files fail (resilient loading)
**And** it collects errors for reporting while returning successful loads

**Given** PropertyBank needs loading
**When** I implement load_property_bank()
**Then** file name comes from Config.property_bank_file (default: "property_bank.json")
**And** it uses same validation/parsing logic as load_file()

**Given** errors must be actionable
**When** I define error types
**Then** SchemaError variants include: IoError, ParseError, ValidationError
**And** errors include file path context for troubleshooting
**And** errors use miette formatting with file/line/column when available

**Given** testing is needed
**When** I write unit tests
**Then** test loading valid schemas (JSON/TOML/YAML)
**And** test PathValidator rejection (paths outside root)
**And** test resilience (load_all continues after individual failures)
**And** use `example_vault/.lithos/schemas/` as test fixtures

---

## Story 7.4: Implement Decoder (Format Normalization)

As a developer transforming schema files,
I want format normalization from user object format to domain array format,
So that domain types receive consistent input regardless of source format.

**Acceptance Criteria:**

**Given** I need format normalization
**When** I create Decoder in `crates/adapters/src/spi/schema/decoder.rs`
**Then** it transforms properties: user object format → domain array format
**And** it normalizes $ref syntax across JSON/TOML/YAML
**And** it generates UUIDs (v7) for inline properties
**And** it produces RawSchema (domain input type)

**Given** Decoder scope boundaries
**When** I clarify responsibilities
**Then** Decoder does NOT resolve $refs (PropertyBank.decode() handles this)
**And** Decoder does NOT validate schemas (Validator handles this)
**And** Decoder does NOT merge inheritance (SchemaResolver handles this)
**And** Decoder is stateless (pure function)

**Given** properties transformation is critical
**When** I decode user format to domain format
**Then** user properties object with key-value pairs becomes domain properties array
**And** each property is either: Inline (with type/constraints) or Ref (with $ref path)
**And** inline properties include: id (UUID v7), name, required, array, type, constraints
**And** refs are normalized: strip "#/properties/" prefix if present

**Given** $ref syntax varies by format
**When** I normalize references
**Then** JSON: `{ "$ref": "#/properties/name" }` → normalized to "name"
**And** TOML: `ref = "name"` (no $ prefix) → normalized to "name"
**And** YAML: `$ref: "#/properties/name"` OR `ref: "name"` → normalized to "name"

**Given** PropertySpec construction
**When** I build type-specific constraints
**Then** string type: extract enum, pattern
**And** number type: extract min, max, step
**And** bool type: no constraints
**And** date type: extract format
**And** file type: extract file_class, directory

**Given** schema-level fields
**When** I decode schema metadata
**Then** generate UUID v7 for schema identity
**And** extract and validate name (alphanumeric + dash/underscore)
**And** extract optional extends (schema name)
**And** extract optional excludes (property names)

**Given** decoding can fail
**When** I handle errors
**Then** return SchemaError::DecoderError for: missing required fields, invalid names, invalid types, type mismatches, invalid $ref format
**And** errors include source path, field name, and expected format

**Given** format consistency is required
**When** I write tests
**Then** parameterized tests verify: same schema in JSON/TOML/YAML → identical RawSchema output
**And** test $ref normalization (with/without "#/properties/" prefix)
**And** test UUID generation for inline properties
**And** test PropertySpec construction for all 5 types

---

## Story 7.5: Implement Validator (JSON Schema + Semantic Validation)

As a developer ensuring schema correctness,
I want syntactic and semantic validation of user schemas,
So that invalid schemas are rejected early with clear error messages.

**Acceptance Criteria:**

**Given** I need comprehensive schema validation
**When** I create Validator in `crates/adapters/src/spi/schema/validator.rs`
**Then** it validates SYNTACTIC compliance (against vault-schema.schema.json using jsonschema crate)
**And** it validates SEMANTIC rules (circular inheritance, duplicate names, invalid excludes)
**And** it runs BEFORE Decoder transformation (validates raw parsed data)

**Given** syntactic validation
**When** I validate against vault-schema.schema.json
**Then** compile vault-schema.schema.json to JSONSchema (once, static initialization)
**And** validate parsed schema returns violation errors with JSON paths
**And** violations indicate: missing fields, invalid types, constraint mismatches, invalid $ref format

**Given** semantic validation
**When** I validate business rules
**Then** detect circular inheritance using graph traversal (build dependency graph, check for cycles)
**And** detect duplicate property names within single schema
**And** detect invalid excludes (excludes without extends, or excluding non-existent properties)
**And** return SchemaError::SemanticError with rule and message

**Given** circular inheritance must be detected
**When** schemas form inheritance cycles
**Then** validator detects cycles via dependency graph traversal
**And** error includes full cycle chain (e.g., "A → B → C → A")
**And** non-circular multi-level chains pass validation

**Given** error reporting must be actionable
**When** validation fails
**Then** use miette formatting with file path, line/column, and error context
**And** suggest fixes where possible (e.g., "Did you mean 'number' instead of 'integer'?")

**Given** testing is needed
**When** I write unit tests
**Then** test valid schemas pass (use example_vault schemas)
**And** test invalid schemas fail with specific errors
**And** test circular inheritance detection (3+ schema cycles)
**And** test duplicate property detection
**And** test excludes validation

---

## Story 7.6: Implement SchemaResolver Integration

As a developer resolving schema inheritance,
I want schemas resolved in topological order with parent property merging,
So that complex inheritance chains produce correct final schemas.

**Acceptance Criteria:**

**Given** SchemaResolver exists in domain (Epic 3)
**When** I integrate with Epic 7 adapters
**Then** Command adapter orchestrates: load files → decode → validate → resolve via domain Resolver
**And** Resolver is pure domain logic (no I/O, receives RawSchema + parent Schema + PropertyBank)

**Given** resolution requires ordering
**When** I orchestrate resolution in Command adapter
**Then** build dependency graph from RawSchemas (child → parent via extends)
**And** perform topological sort to determine resolution order (parents before children)
**And** resolve schemas in order, caching each resolved Schema

**Given** inheritance merging follows specific rules
**When** domain Resolver merges properties
**Then** start with parent properties (if extends exists)
**And** exclude properties listed in excludes set
**And** add child's own properties
**And** child properties OVERRIDE parent properties (if duplicate names)
**And** resolve $refs by looking up in PropertyBank

**Given** multi-level inheritance is supported
**When** schemas chain (e.g., base_note → task → task_project → task_meeting)
**Then** each level correctly inherits and can override/exclude parent properties
**And** final resolved Schema contains complete property list

**Given** resolution can fail
**When** I handle errors
**Then** return DomainError::PropertyNotFound if $ref invalid
**And** return DomainError::SchemaNotFound if extends references missing schema
**And** return DomainError::CircularInheritance if cycle detected
**And** propagate domain errors to SchemaError in adapter

**Given** testing is needed
**When** I write integration tests
**Then** test single-level inheritance
**And** test multi-level inheritance (4+ levels)
**And** test excludes handling
**And** test property override
**And** test $ref resolution via PropertyBank
**And** use example_vault schemas as test data

---

## Story 7.7: Implement SchemaCache (Epic 5 Integration)

As a developer optimizing schema loading,
I want persistent caching of resolved schemas,
So that schema resolution is fast and survives restarts.

**Acceptance Criteria:**

**Given** I need decoupled caching abstraction
**When** I create SchemaCache trait in `crates/adapters/src/spi/schema/cache.rs`
**Then** it defines operations: get, put, invalidate, clear
**And** it is Send + Sync for thread-safe usage

**Given** RedbSchemaCache provides persistence
**When** I implement Redb-backed cache using Epic 5 `RedbBuilder`
**Then** it uses `RedbBuilder::new().path(db_path).table_name("schemas").build()` to create reader/writer pair
**And** keys are schema names (String), values are Schema aggregates
**And** values use rkyv serialization via Epic 5 `Entry<Schema>` wrapper (zero-copy deserialization per ADR 0002)
**And** metadata stored via `RedbWriter::put_with_metadata()`: timestamp, source_hash (SHA256 of file content)

**Given** cache invalidation is needed
**When** source files change
**Then** compare cached hash vs current file hash
**And** return cache miss if hashes differ (triggers reload)

**Given** MockSchemaCache enables testing
**When** I create in-memory mock
**Then** it uses HashMap<String, Schema> for fast, filesystem-free testing
**And** implements same SchemaCache trait

**Given** Command adapter integrates cache
**When** I orchestrate loading
**Then** check cache first (with hash validation)
**And** if cache hit with valid hash, skip file loading + resolution
**And** if cache miss, load → decode → validate → resolve → cache.put()

**Given** cache performance is critical
**When** I measure operations
**Then** cached schema retrieval <1ms (Redb read + rkyv deserialize)
**And** cache persists across restarts (Redb file on disk)

**Given** cache errors are non-fatal
**When** cache operations fail
**Then** log warning and fall back to full load pipeline (graceful degradation)

**Given** testing is needed
**When** I write unit tests
**Then** test cache hit (skip full pipeline)
**And** test cache miss (trigger full pipeline)
**And** test hash invalidation (file changed)
**And** test persistence (restart → cache available)

---

## Story 7.8: Implement PropertyBank Singleton Registry

As a developer managing reusable properties,
I want PropertyBank singleton with fast access and runtime overrides,
So that all schema operations access the same property definitions consistently.

**Acceptance Criteria:**

**Given** I need singleton pattern for PropertyBank
**When** I create PropertyBankRegistry in `crates/adapters/src/spi/schema/registry.rs`
**Then** it uses `Arc<OnceLock<PropertyBank>>` for immutable base (loaded from property_bank.json)
**And** it uses `Arc<RwLock<HashMap<String, Property>>>` for runtime overrides
**And** lookup priority: check overrides (RwLock read) → fall back to base (no lock)

**Given** Registry must be initialized
**When** Command adapter loads PropertyBank
**Then** call `PropertyBankRegistry::init(bank)` to populate OnceLock
**And** subsequent init() calls return error (already initialized)

**Given** global singleton is needed
**When** I implement global access
**Then** provide `PropertyBankRegistry::global() -> &'static PropertyBankRegistry`
**And** use lazy initialization on first access

**Given** property lookup must be fast
**When** I implement lookup(key)
**Then** check overrides HashMap (RwLock read)
**And** if not found, check base PropertyBank (no lock)
**And** achieve performance targets: base <10ns, override <50ns (99% hit base)

**Given** runtime overrides support LSP
**When** I implement register_override(name, property)
**Then** acquire overrides write lock
**And** insert/update property in HashMap
**And** subsequent lookups prioritize override
**And** overrides persist until restart (not saved to disk)

**Given** testing is needed
**When** I write unit tests
**Then** test initialization (once succeeds, duplicate fails)
**And** test global singleton access
**And** test lookup priority (overrides before base)
**And** test runtime override registration
**And** test concurrent access (multiple threads)
**And** benchmark performance targets

---

## Story 7.9: Implement Schema Command and Query Adapters

As a developer coordinating schema operations,
I want Command and Query adapters orchestrating all schema utilities,
So that schema loading, caching, and querying work together seamlessly.

**Acceptance Criteria:**

**Given** I need Command adapter for write operations
**When** I create SchemaCommand in `crates/adapters/src/spi/schema/command.rs`
**Then** it composes: SchemaLoader, Decoder, Validator, SchemaCache, PropertyBankRegistry
**And** it implements SchemaCommandPort trait from domain

**Given** SchemaCommand.load_all() orchestrates full pipeline
**When** I implement loading
**Then** **Phase 1:** Load PropertyBank (load → validate → decode → init Registry)
**And** **Phase 2:** Load all schemas (load → validate → decode → collect RawSchemas)
**And** **Phase 3:** Detect circular inheritance (build graph, check cycles)
**And** **Phase 4:** Topological sort (determine resolution order: parents before children)
**And** **Phase 5:** Resolve schemas (check cache → if miss: resolve via domain Resolver → cache.put())
**And** **Phase 6:** Return success with summary (X schemas loaded, Y from cache, Z resolved)

**Given** error handling is resilient
**When** individual schemas fail
**Then** continue processing other schemas (collect errors)
**And** return aggregated error summary at end

**Given** SchemaCommand.refresh(name) reloads single schema
**When** I implement refresh
**Then** invalidate cache → load file → validate → decode → resolve → update cache

**Given** I need Query adapter for read operations
**When** I create SchemaQuery in `crates/adapters/src/spi/schema/query.rs`
**Then** it holds SchemaCache reference only (no loader, no decoder)
**And** it implements SchemaQueryPort trait from domain

**Given** SchemaQuery is read-only
**When** I implement methods
**Then** get(name) returns cached Schema (no file loading)
**And** list() returns all cached schema names
**And** NO side effects (no writes, no resolution)

**Given** adapters implement domain ports
**When** I verify trait implementations
**Then** SchemaCommand implements SchemaCommandPort
**And** SchemaQuery implements SchemaQueryPort
**And** method signatures match port definitions (async if needed)

**Given** module exports follow conventions
**When** I export adapters
**Then** re-export with Schema prefix: SchemaCommand, SchemaQuery, SchemaLoader, Decoder, Validator, SchemaCache, PropertyBankRegistry

**Given** testing is needed
**When** I write integration tests
**Then** test full load_all() pipeline (end-to-end)
**And** test cache hit path (second load_all() uses cache)
**And** test refresh() updates single schema
**And** test Query.get() returns cached schemas
**And** test error scenarios (circular inheritance, validation failures, missing PropertyBank)
**And** use example_vault as test data

---

## Story 7.10: Review Epic 7 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 7 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 7 components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** all public APIs have runnable doc tests demonstrating usage

**Given** all Epic 7 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage
**And** I assess if tests validate business requirements vs implementation details
**And** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 7 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures (example_vault), avoid flaky behavior, and maintain clear intent
**And** test code follows same quality standards as production code with proper documentation

**Given** format-agnostic behavior is critical
**When** I test Decoder
**Then** parameterized tests verify identical RawSchema output for JSON/TOML/YAML inputs

**Given** inheritance logic is complex
**When** I test resolution
**Then** integration tests verify multi-level inheritance, circular detection, excludes handling, property override

**Given** cache behavior affects performance
**When** I test SchemaCache
**Then** test cache hit/miss paths, hash invalidation, persistence across restarts

**Given** PropertyBank singleton is performance-critical
**When** I test Registry
**Then** test initialization, lookup priority, concurrent access, performance benchmarks (<10ns base, <50ns override)

---

## Story 7.11: Document Schema System

As a developer working with the schema system,
I want comprehensive documentation for the adapter layer and user guidance,
So that I understand how loading, resolution, and caching interact, and users can create schemas effectively.

**Acceptance Criteria:**

**Given** Epic 7 implementation is complete
**When** I create developer documentation
**Then** create `docs/adapters/schema-system.md` following architecture doc pattern
**And** document orchestration flow: Loader → Decoder → Validator → Resolver → Cache
**And** explain user format vs domain format distinction (object properties → array properties)
**And** document PropertyBank singleton pattern and usage
**And** document SchemaCache architecture and Redb integration

**Given** module-level documentation is needed
**When** I create README
**Then** create `crates/adapters/src/spi/schema/README.md` with quick start examples
**And** explain each component's responsibility (Loader, Decoder, Validator, Resolver, Cache, Registry)

**Given** example_vault documentation is critical
**When** I review example_vault/README.md
**Then** ensure it documents all schemas with inheritance chains
**And** provide usage examples as starter kit
**And** explain usage as test fixtures

**Given** vault-schema.schema.json needs user guidance
**When** I add documentation
**Then** add comments explaining schema structure (name, properties, extends, excludes)
**And** provide examples for each property type and constraint

**Given** migration guide is needed
**When** users reference old docs/schemas/
**Then** create `docs/migrations/schemas-to-example-vault.md`
**And** explain relocation and update documentation references

**Given** public APIs need doc tests
**When** I write documentation
**Then** all public components have runnable doc tests in `# Examples` sections
**And** doc tests demonstrate realistic usage patterns

**Given** performance benchmarks validate design
**When** I create benchmarks
**Then** benchmark load_all() with 41 schemas (target: <100ms cold, <10ms hot cache)
**And** benchmark Decoder, Validator, Resolver, Cache operations
**And** benchmark PropertyBankRegistry lookup performance

---
