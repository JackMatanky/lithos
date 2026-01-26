# Epic 7: Schema System & Validation **[MVP CORE]**

Users can define metadata schemas with field types, inheritance, and validation that provide input parameters for templates and enforce vault consistency.

**FRs covered:** FR8, FR9, FR10, FR11, FR12, FR13, FR14

**Implementation Notes:**

- **User Schema Format**: Simple object-based properties (`"properties": { "name": {...} }`)
- **Internal Domain Format**: Array-based properties after Decoder transformation
- **Schema Validation File**: `vault-schema.schema.json` validates user schema files
- **Example Vault**: `example_vault/` provides starter kit + test fixtures
- **Decoder Strategy**: Normalizes user format (object) → domain format (array)
- **Validator**: JSON Schema + semantic validation against vault-schema.schema.json
- **SchemaResolver**: Handles inheritance (extends/excludes) + circular dependency detection
- **Singleton Pattern**: `Arc<OnceLock<PropertyBank>>` (immutable) + `Arc<RwLock<HashMap>>` (runtime overrides)
- **Caching Strategy**: Decoupled `SchemaCache` trait with Redb implementation (Epic 5)
- **Adapter Structure**: `crates/adapters/src/spi/schema/` contains query.rs, command.rs, loader.rs, decoder.rs, validator.rs, cache.rs, registry.rs
- **Note:** Frontmatter validation moved to Epic 10.6 (application layer)
- **Note:** Schema-template integration moved to Epic 12.4 (template system)

---

## Story 7.1: Create Default Schema Files & Fix vault-schema.schema.json

As a developer setting up Epic 7,
I want corrected vault-schema.schema.json and comprehensive example vault,
So that validation matches user format and provides starter kit + test fixtures.

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

### **Example Vault Structure:**

**Given** need for user starter kit
**When** I create example_vault/ at root
**Then** structure matches:
```
example_vault/
├── README.md                           (full documentation)
├── .gitignore                          (ignore .lithos/cache/)
├── .lithos/
│   ├── schemas/
│   │   ├── vault-schema.schema.json
│   │   ├── property_bank.json
│   │   ├── task.json
│   │   ├── contact.json
│   │   └── ... (all 41 schemas)
│   └── templates/                      (placeholder for Epic 12)
└── notes/
    ├── task-example.md
    ├── contact-example.md
    └── project-example.md
```

**Given** schemas exist in docs/schemas/
**When** I move them to example_vault
**Then** move all 41 `*.json` files from `docs/schemas/` → `example_vault/.lithos/schemas/`
**And** move `docs/schemas/property_bank.json` → `example_vault/.lithos/schemas/`
**And** move `docs/schemas/examples/` → `example_vault/.lithos/schemas/examples/` (optional: can merge into main schemas/)
**And** UPDATE any path references in documentation

**Given** example notes are needed
**When** I create example notes
**Then** `example_vault/notes/task-example.md` has frontmatter matching task.json schema
**And** `example_vault/notes/contact-example.md` has frontmatter matching contact.json schema
**And** `example_vault/notes/project-example.md` demonstrates inheritance from task schema
**And** all notes include realistic content demonstrating schema constraints

**Given** .gitignore is needed
**When** I create example_vault/.gitignore
**Then** it ignores `.lithos/cache/` (future: generated cache files)
**And** it preserves `.lithos/schemas/` and `.lithos/templates/`

### **Validation:**

**Given** all schemas are created
**When** I run JSON Schema validation
**Then** validate all 41 schemas against vault-schema.schema.json
**And** fix any non-compliant schemas (add required `type` fields, fix constraint placement)
**And** all schemas pass validation without errors

**Given** test fixture accessibility
**When** I set up test infrastructure
**Then** create `tests/fixtures.rs` with `pub const EXAMPLE_VAULT_PATH: &str = "example_vault";`
**And** verify all schema files are readable at test time (use `std::fs::read_to_string`)
**And** verify directory structure is preserved in version control

### **Documentation:**

**Given** example_vault/ is created
**When** I write README.md
**Then** document all 41 schemas with table: name, extends, key features
**And** explain inheritance chains (e.g., task → task_project → task_meeting)
**And** show PropertyBank $ref usage with examples
**And** include "Usage as Starter Kit" section with copy commands
**And** include "Usage as Test Fixtures" section explaining integration test usage
**And** list all property types with examples (string enum, number range, date format, file directory)

**Given** project docs need updates
**When** I update top-level documentation
**Then** update `README.md` to mention example_vault as starter kit
**And** update `docs/architecture.md` to explain user format vs domain format distinction
**And** create migration guide for users referencing old `docs/schemas/` location

---

## Story 7.2: Implement SchemaLoader with PathValidator

As a developer loading schema files,
I want robust file loading with security validation,
So that schema files are safely read from disk before processing.

**Acceptance Criteria:**

### **SchemaLoader Structure:**

**Given** I need to load schema files
**When** I create SchemaLoader in `crates/adapters/src/spi/schema/loader.rs`
**Then** it holds `root_path: PathBuf` (from Config: schemasDir)
**And** it holds `path_validator: Box<dyn PathValidator>` (from Epic 4)
**And** it holds `format_dispatcher: FormatDispatcher` (from Epic 4)
**And** it provides `pub fn new(root_path: PathBuf, validator: Box<dyn PathValidator>) -> Self`

**Given** SchemaLoader initialization
**When** Command adapter creates it
**Then** root_path is `Config.vault_path.join(Config.schemas_dir)` (e.g., `vault/.lithos/schemas/`)
**And** PathValidator mode is configurable (Strict or Flexible)
**And** FormatDispatcher is initialized for JSON/TOML/YAML parsing

### **File Loading Logic:**

**Given** SchemaLoader.load_file(file_name)
**When** I implement file loading
**Then** construct full_path = `root_path.join(file_name)`
**And** call `path_validator.validate(full_path)` → return Err if fails (security check)
**And** call `std::fs::read_to_string(full_path)` → return Err(SchemaError::IoError) if fails
**And** call `format_dispatcher.parse::<serde_json::Value>(path, content)` → return Err(SchemaError::ParseError) if fails
**And** return `Ok((path, parsed_value))` tuple

**Given** SchemaLoader.load_all()
**When** I implement directory scanning
**Then** call `std::fs::read_dir(root_path)` to list all files
**And** filter to `*.json`, `*.toml`, `*.yaml`, `*.yml` extensions
**And** EXCLUDE `vault-schema.schema.json` (validation schema, not user schema)
**And** load each file using `load_file()`
**And** collect results as `Vec<(PathBuf, serde_json::Value)>`
**And** continue processing if individual files fail (resilience)
**And** collect errors as Vec<SchemaError> for reporting

**Given** PropertyBank loading
**When** I implement `load_property_bank()`
**Then** file name is `Config.property_bank_file` (default: "property_bank.json")
**And** use same validation/parsing logic as load_file()
**And** return `Result<serde_json::Value, SchemaError>`

### **Error Handling:**

**Given** SchemaLoader operations
**When** I define error types
**Then** `SchemaError::IoError { path, message }` for filesystem failures
**And** `SchemaError::ParseError { path, message, line, column }` for syntax errors
**And** `SchemaError::ValidationError { path, message }` for path security violations
**And** all errors include file path context for user troubleshooting

**Given** error messages
**When** I format errors using miette
**Then** include file path, line/column (if available), and suggested fixes
**And** example: "Failed to parse task.json at line 15: missing comma after property 'title'"

### **Testing:**

**Given** SchemaLoader is implemented
**When** I write unit tests
**Then** test successful load of valid JSON/TOML/YAML schemas
**And** test PathValidator rejection of paths outside root
**And** test FormatDispatcher handling of syntax errors (malformed JSON)
**And** test load_all() resilience (continues after individual file failures)
**And** test PropertyBank loading from property_bank.json
**And** use `example_vault/.lithos/schemas/` as test fixtures

---

## Story 7.3: Implement Decoder (Format Normalization)

As a developer transforming schema files,
I want format normalization from user object format to domain array format,
So that domain types receive consistent input regardless of source format.

**Acceptance Criteria:**

### **Decoder Responsibility:**

**Given** I need format normalization
**When** I define Decoder purpose
**Then** it transforms properties: object → array
**And** it normalizes $ref syntax across formats (JSON/TOML/YAML)
**And** it generates UUIDs for inline properties (if not provided)
**And** it produces RawSchema (domain input type)

**Given** Decoder does NOT handle
**When** I clarify scope
**Then** it does NOT resolve $refs (PropertyBank.decode() does this)
**And** it does NOT validate schemas (Validator does this)
**And** it does NOT merge inheritance (Resolver does this)

### **Decoder Structure:**

**Given** I create Decoder in `crates/adapters/src/spi/schema/decoder.rs`
**When** I design the API
**Then** it provides `pub fn decode(parsed: serde_json::Value, source_path: &Path) -> Result<RawSchema, SchemaError>`
**And** it is stateless (pure function, no internal state)
**And** it returns detailed errors with source path + line/column context

### **Properties Object → Array Transformation:**

**Given** parsed schema has properties as object
**When** I transform to RawSchema
**Then** example input (user format):
```json
{
  "name": "task",
  "properties": {
    "title": { "type": "string", "required": true, "pattern": "^.+$" },
    "status": { "$ref": "#/properties/task_status" }
  }
}
```
**And** output (domain format):
```rust
RawSchema {
    id: Uuid::now_v7(),
    name: SchemaName::new("task"),
    extends: None,
    excludes: HashSet::new(),
    properties: vec![
        RawProperty::Inline(RawPropertyInline {
            id: Uuid::now_v7(),
            name: "title",
            required: true,
            array: false,
            spec: PropertySpec::String(StringSpec { pattern: Some("^.+$"), enum: None }),
        }),
        RawProperty::Ref(RawPropertyRef {
            ref_path: "task_status",  // normalized: stripped "#/properties/" prefix
        }),
    ],
}
```

**Given** properties object iteration
**When** I decode properties
**Then** iterate over object keys (property names)
**And** for each key-value pair, determine if value is PropertyRef or InlineProperty
**And** PropertyRef detection: check for `$ref` field (object with single key)
**And** InlineProperty detection: check for `type` field

### **PropertyRef Normalization:**

**Given** $ref syntax varies by format
**When** I normalize references
**Then** JSON format: `{ "$ref": "#/properties/task_status" }`
**And** TOML format: `ref = "task_status"` (no $ prefix allowed in TOML keys)
**And** YAML format: `$ref: "#/properties/task_status"` OR `ref: "task_status"`

**Given** $ref parsing
**When** I extract property name
**Then** strip "#/properties/" prefix if present → "task_status"
**And** if no prefix, use value as-is → "task_status"
**And** store normalized name in RawPropertyRef.ref_path

### **InlineProperty Construction:**

**Given** inline property object
**When** I decode to RawPropertyInline
**Then** generate `id: Uuid::now_v7()` (time-ordered unique identifier)
**And** extract `name` from object key (e.g., "title")
**And** extract `required` (bool, default: false)
**And** extract `array` (bool, default: false)
**And** extract `type` (string: "string" | "number" | "bool" | "date" | "file")
**And** build PropertySpec based on type + type-specific constraints

**Given** PropertySpec construction
**When** I parse type-specific constraints
**Then** for `type: "string"`: extract `enum`, `pattern` → StringSpec
**And** for `type: "number"`: extract `min`, `max`, `step` → NumberSpec
**And** for `type: "bool"`: no constraints → BoolSpec::default()
**And** for `type: "date"`: extract `format` → DateSpec
**And** for `type: "file"`: extract `file_class`, `directory` → FileSpec

### **Schema-Level Fields:**

**Given** schema-level fields
**When** I decode RawSchema
**Then** generate `id: Uuid::now_v7()` for schema identity
**And** extract `name` → parse as SchemaName (validates alphanumeric + dash/underscore)
**And** extract `extends` (Option<String>) → parse as Option<SchemaName>
**And** extract `excludes` (Vec<String>) → parse as HashSet<PropertyName>
**And** return DomainError if SchemaName/PropertyName validation fails

### **Error Handling:**

**Given** decoding can fail
**When** I handle errors
**Then** return `SchemaError::DecoderError { path, message, field }` for:
- Missing required fields (`name`, `properties`)
- Invalid schema name format (not alphanumeric)
- Invalid property type (not in enum)
- Type mismatch (e.g., `enum` on number property)
- Invalid $ref format

**Given** error messages
**When** I format errors
**Then** include source path, field name, and expected format
**And** example: "Invalid property type 'integer' in task.json field 'priority'. Expected: string, number, bool, date, file"

### **Testing:**

**Given** Decoder is implemented
**When** I write unit tests
**Then** test properties object → array transformation
**And** test $ref normalization (with/without "#/properties/" prefix)
**And** test UUID generation for inline properties
**And** test PropertySpec construction for all types
**And** test schema-level field extraction (name, extends, excludes)
**And** test error handling for invalid formats
**And** parameterized tests: same schema in JSON/TOML/YAML → identical RawSchema output

---

## Story 7.4: Implement Validator (JSON Schema + Semantic Validation)

As a developer ensuring schema correctness,
I want syntactic and semantic validation of user schemas,
So that invalid schemas are rejected early with clear error messages.

**Acceptance Criteria:**

### **Validator Responsibility:**

**Given** I need schema validation
**When** I define Validator purpose
**Then** it validates SYNTACTIC compliance (against vault-schema.schema.json)
**And** it validates SEMANTIC rules (business logic: circular inheritance, duplicate names)
**And** it runs BEFORE Decoder transformation
**And** it provides detailed error messages with line/column context

### **Validator Structure:**

**Given** I create Validator in `crates/adapters/src/spi/schema/validator.rs`
**When** I design the API
**Then** it provides `pub fn validate(parsed: &serde_json::Value, source_path: &Path) -> Result<(), SchemaError>`
**And** it holds `json_schema: jsonschema::JSONSchema` (compiled vault-schema.schema.json)
**And** it initializes once: `static VALIDATOR: OnceLock<jsonschema::JSONSchema> = OnceLock::new();`

**Given** Validator initialization
**When** Command adapter uses it
**Then** load `vault-schema.schema.json` from `example_vault/.lithos/schemas/` OR embedded in binary
**And** compile to jsonschema::JSONSchema
**And** compilation errors are fatal (should never happen if vault-schema.schema.json is valid)

### **Syntactic Validation (JSON Schema):**

**Given** parsed schema as serde_json::Value
**When** I validate against vault-schema.schema.json
**Then** call `json_schema.validate(parsed)` → returns iterator of validation errors
**And** if errors exist, return `SchemaError::ValidationError { path, violations: Vec<String> }`
**And** each violation includes JSON path to invalid field (e.g., "/properties/title/type")

**Given** JSON Schema validation failures
**When** I format errors
**Then** example violations:
- "Missing required field: /properties/title/type"
- "Invalid value for /properties/priority/type: Expected string, number, bool, date, file. Got: integer"
- "Invalid constraint: /properties/priority/enum not allowed for type 'number'"
- "Invalid $ref format: /properties/author/$ref must match pattern ^#/properties/[a-zA-Z0-9_-]+$"

### **Semantic Validation (Business Rules):**

**Given** schema passes JSON Schema validation
**When** I validate business rules
**Then** check for circular inheritance (see Circular Dependency Detection below)
**And** check for duplicate property names within single schema
**And** check for invalid excludes (excluding non-existent parent properties)
**And** return `SchemaError::SemanticError { path, rule, message }`

**Given** duplicate property name detection
**When** I validate properties object
**Then** iterate over property names
**And** check for duplicates using HashSet<String>
**And** error: "Duplicate property name 'title' in task.json"

**Given** excludes validation
**When** schema has `extends` and `excludes`
**Then** verify excludes is non-empty ONLY if extends exists
**And** error: "Schema task.json has 'excludes' but no 'extends' field"

### **Circular Dependency Detection:**

**Given** schemas may have inheritance chains
**When** I detect circular dependencies
**Then** build dependency graph: Map<SchemaName, Option<SchemaName>> (child → parent via `extends`)
**And** use depth-first search (DFS) to detect cycles
**And** maintain visited set and recursion stack during DFS
**And** if cycle detected, return `SchemaError::CircularInheritance { chain: Vec<SchemaName> }`

**Given** circular inheritance example
**When** schemas are: A extends B, B extends C, C extends A
**Then** detected cycle: ["A", "B", "C", "A"]
**And** error message: "Circular inheritance detected: A → B → C → A"

**Given** multi-level inheritance (non-circular)
**When** schemas are: project_meeting extends task_project, task_project extends task, task extends base_note
**Then** validation passes (linear chain, no cycle)

### **Error Reporting with miette:**

**Given** validation failures
**When** I format errors
**Then** use miette::SourceSpan to highlight exact location in source file
**And** include file path, line/column, and error context (3 lines before/after)
**And** suggest fixes where possible:
- "Did you mean 'number' instead of 'integer'?"
- "Remove 'excludes' field or add 'extends' field"
- "Break circular inheritance: remove 'extends' from schema C"

### **Testing:**

**Given** Validator is implemented
**When** I write unit tests
**Then** test JSON Schema validation with valid schemas (all pass)
**And** test JSON Schema validation with invalid schemas (missing type, wrong constraint)
**And** test circular inheritance detection (3-schema cycle, 4-schema cycle)
**And** test duplicate property name detection
**And** test excludes without extends detection
**And** test error message formatting with miette
**And** use `example_vault/.lithos/schemas/` as valid test cases

---

## Story 7.5: Implement SchemaResolver (Circular Check + Inheritance)

As a developer resolving schema inheritance,
I want schemas resolved in topological order with parent property merging,
So that complex inheritance chains produce correct final schemas.

**Acceptance Criteria:**

### **SchemaResolver Integration:**

**Given** SchemaResolver already exists in `crates/domain/src/schema/resolver.rs` (Epic 3)
**When** I integrate with Epic 7
**Then** adapters use `Resolver::resolve(raw: RawSchema, parent: Option<&Schema>, bank: &PropertyBank) -> Result<Schema, DomainError>`
**And** Resolver is pure domain logic (no I/O, no file loading)
**And** Command adapter orchestrates resolution (loads files, calls Resolver)

### **Resolution Orchestration:**

**Given** Command adapter has all RawSchemas
**When** I orchestrate resolution
**Then** build dependency graph: Map<SchemaName, Option<SchemaName>> (child → parent via `extends`)
**And** perform topological sort to determine resolution order
**And** resolve schemas in order: parents before children
**And** store resolved Schema in cache after each resolution

**Given** topological sort
**When** I determine resolution order
**Then** example: [base_note, task, task_project, task_meeting]
**And** base_note has no parent → resolve first
**And** task extends base_note → resolve second (after base_note)
**And** task_project extends task → resolve third (after task)
**And** task_meeting extends task_project → resolve fourth (after task_project)

**Given** resolution order is computed
**When** I resolve each schema
**Then** lookup parent Schema from cache (if `extends` exists)
**And** call `Resolver::resolve(raw, parent, property_bank)`
**And** store resulting Schema in cache
**And** continue to next schema

### **Inheritance Merging Logic (Domain Resolver):**

**Given** Resolver.resolve() implementation (Epic 3)
**When** I merge parent properties
**Then** start with empty HashMap<String, Property> for resolved properties
**And** if parent exists, iterate over parent.properties()
**And** for each parent property, add to HashMap UNLESS in excludes set
**And** then iterate over child's own properties (RawProperty)
**And** resolve inline properties → Property (convert RawPropertyInline to Property)
**And** resolve $refs → Property (lookup in PropertyBank via decode())
**And** child properties OVERRIDE parent properties (if duplicate names)

**Given** property resolution
**When** I handle RawProperty variants
**Then** RawProperty::Inline → construct Property from id, name, required, array, spec
**And** RawProperty::Ref → call `property_bank.decode(ref_path)` → clone Property
**And** if $ref not found, return `DomainError::PropertyNotFound(ref_path)`

**Given** final resolved properties
**When** I construct Schema
**Then** convert HashMap values to Vec<Property>
**And** sort by property name for determinism (alphabetical order)
**And** call `Schema::new(raw.id, raw.name, final_props)`
**And** Schema.properties IS the resolved list (no separate resolved_properties field)

### **Excludes Handling:**

**Given** child schema has excludes
**When** I merge parent properties
**Then** example: parent=task has ["title", "status"], child=task_minimal has excludes=["status"]
**And** merged properties: ["title"] (status excluded)
**And** child can add own properties: ["title", "minimal_description"]

### **Multi-Level Inheritance:**

**Given** schemas: base_note → task → task_project
**When** I resolve task_project
**Then** resolve base_note first → Schema with [id, created_date]
**And** resolve task second → Schema with [id, created_date, title, status, due_date]
**And** resolve task_project third → Schema with [id, created_date, title, status, due_date, project_name]
**And** each level merges parent properties correctly

### **Error Handling:**

**Given** resolution can fail
**When** I handle errors
**Then** return `DomainError::PropertyNotFound(ref_path)` if $ref invalid
**And** return `DomainError::SchemaNotFound(parent_name)` if extends references missing schema
**And** return `DomainError::CircularInheritance(chain)` if cycle detected during topological sort
**And** all errors propagate to Command adapter → SchemaError

### **Testing:**

**Given** SchemaResolver integration
**When** I write integration tests
**Then** test single-level inheritance (task extends base_note)
**And** test multi-level inheritance (task_meeting extends task_project extends task extends base_note)
**And** test excludes handling (child excludes parent property)
**And** test property override (child redefines parent property)
**And** test $ref resolution via PropertyBank
**And** test error: missing parent schema
**And** test error: $ref to non-existent property
**And** use `example_vault/.lithos/schemas/` as test data

---

## Story 7.6: Implement SchemaCache (Epic 5 Integration)

As a developer optimizing schema loading,
I want persistent caching of resolved schemas,
So that schema resolution is fast and survives restarts.

**Acceptance Criteria:**

### **SchemaCache Trait:**

**Given** I need decoupled caching
**When** I create SchemaCache trait in `crates/adapters/src/spi/schema/cache.rs`
**Then** it defines:
```rust
pub trait SchemaCache: Send + Sync {
    fn get(&self, name: &str) -> Result<Option<Schema>, SchemaError>;
    fn put(&self, name: &str, schema: Schema) -> Result<(), SchemaError>;
    fn invalidate(&self, name: &str) -> Result<(), SchemaError>;
    fn clear(&self) -> Result<(), SchemaError>;
}
```

**Given** SchemaCache interface
**When** I design operations
**Then** `get()` returns None if schema not cached (cache miss)
**And** `put()` stores resolved Schema with metadata (timestamp, source_hash)
**And** `invalidate()` removes single schema from cache (for reload)
**And** `clear()` removes all schemas (for full refresh)

### **RedbSchemaCache Implementation:**

**Given** I implement Redb-backed cache
**When** I create RedbSchemaCache
**Then** it uses Redb table `"schemas"` (from Epic 5 RedbCache)
**And** keys are String (schema name: "task", "contact", etc.)
**And** values are rkyv-serialized Schema aggregates (zero-copy deserialization)

**Given** RedbSchemaCache.get()
**When** I retrieve cached schema
**Then** open Redb read transaction
**And** lookup key in "schemas" table
**And** if found, deserialize using rkyv → Schema
**And** if not found, return Ok(None) (cache miss)
**And** if deserialization fails, return Err(SchemaError::CacheError)

**Given** RedbSchemaCache.put()
**When** I store schema
**Then** serialize Schema using rkyv (archived format)
**And** open Redb write transaction
**And** insert key-value into "schemas" table
**And** commit transaction
**And** if write fails, return Err(SchemaError::CacheError)

**Given** cache invalidation strategy
**When** source file changes
**Then** compute SHA256 hash of source file content
**And** store hash alongside schema in cache metadata
**And** on get(), compare cached hash vs current file hash
**And** if hashes differ, return Ok(None) to trigger reload

### **MockSchemaCache Implementation:**

**Given** I need testing without Redb
**When** I create MockSchemaCache
**Then** it stores schemas in `HashMap<String, Schema>`
**And** implements same SchemaCache trait
**And** used in unit tests for fast, filesystem-free testing

### **Cache Integration with Command Adapter:**

**Given** Command adapter orchestrates loading
**When** I integrate cache
**Then** Command holds `Box<dyn SchemaCache>`
**And** on load_all(), check cache for each schema first
**And** if cache hit (with valid hash), skip file loading + resolution
**And** if cache miss, load file → decode → validate → resolve → cache.put()
**And** invalidate cache entry if source file hash changed

**Given** cache performance
**When** I benchmark operations
**Then** cached schema retrieval <1ms (Redb read)
**And** cache miss triggers full pipeline: load (I/O) + parse + decode + validate + resolve + cache
**And** subsequent loads use cache (avoid re-resolution)

### **Cache Persistence:**

**Given** Redb is persistent storage
**When** application restarts
**Then** cache survives restart (Redb file persists)
**And** cached schemas available immediately
**And** no re-resolution needed unless source files changed

### **Error Handling:**

**Given** cache operations can fail
**When** I handle errors
**Then** return `SchemaError::CacheError { message }` for:
- Redb transaction failures
- Serialization/deserialization errors
- Disk I/O errors

**Given** cache errors are non-fatal
**When** cache operation fails
**Then** log warning: "Schema cache unavailable, falling back to full load"
**And** continue with file loading + resolution (graceful degradation)

### **Testing:**

**Given** SchemaCache is implemented
**When** I write unit tests
**Then** test RedbSchemaCache get/put/invalidate/clear
**And** test MockSchemaCache operations
**And** test cache hit (schema loaded from cache)
**And** test cache miss (schema loaded from file)
**And** test hash invalidation (file changed → cache miss)
**And** test persistence (restart → cache available)
**And** integration test: load_all() uses cache correctly

---

## Story 7.7: Implement PropertyBank Singleton Registry

As a developer managing reusable properties,
I want PropertyBank singleton with fast access and runtime overrides,
So that all schema operations access the same property definitions consistently.

**Acceptance Criteria:**

### **PropertyBankRegistry Structure:**

**Given** I need singleton pattern
**When** I create Registry in `crates/adapters/src/spi/schema/registry.rs`
**Then** it holds:
```rust
pub struct PropertyBankRegistry {
    base: Arc<OnceLock<PropertyBank>>,           // Immutable base from disk
    overrides: Arc<RwLock<HashMap<String, Property>>>,  // Runtime overrides
}
```

**Given** Registry design
**When** I explain the pattern
**Then** `base` is initialized once from property_bank.json (immutable, no lock contention)
**And** `overrides` allows runtime-defined properties (LSP, user plugins)
**And** lookup priority: overrides first (RwLock read), then base (no lock)

### **Registry Initialization:**

**Given** Command adapter loads PropertyBank
**When** I initialize Registry
**Then** load property_bank.json via SchemaLoader
**And** decode to PropertyBank (properties object → HashMap)
**And** call `PropertyBankRegistry::init(bank)` (sets OnceLock)
**And** subsequent init() calls return error or are ignored (idempotent)

**Given** global singleton instance
**When** I implement global access
**Then** provide `PropertyBankRegistry::global() -> &'static PropertyBankRegistry`
**And** use `static REGISTRY: OnceLock<PropertyBankRegistry> = OnceLock::new();`
**And** initialize on first access (lazy init)
**And** panic if not initialized before first access (caller error)

### **Property Lookup:**

**Given** Registry.lookup(key: &str)
**When** I implement lookup
**Then** acquire `overrides` read lock
**And** check if key exists in overrides HashMap
**And** if found, return cloned Property (release lock immediately)
**And** if not found, fall back to base PropertyBank.get_by_name(key)
**And** if not in base, return None

**Given** lookup performance
**When** I benchmark
**Then** override lookup: <50ns (HashMap read + RwLock read)
**And** base lookup: <10ns (OnceLock read, no lock contention)
**And** 99% of lookups hit base (zero lock path)

### **Runtime Overrides:**

**Given** Registry.register_override(name, property)
**When** I add runtime property
**Then** acquire `overrides` write lock
**And** insert/update property in HashMap
**And** release lock
**And** subsequent lookups prioritize override

**Given** use case for overrides
**When** LSP defines schema property dynamically
**Then** call `Registry::global().register_override("lsp_temp_property", property)`
**And** schema resolution can reference this property via $ref
**And** override persists until application restart (not saved to disk)

### **Hot Reload Support (Future):**

**Given** hot reload will be needed (future Epic)
**When** I design for extensibility
**Then** support `AtomicPtr<PropertyBank>` swap pattern for base
**And** allows atomic replacement of entire PropertyBank
**And** lock-free reads during hot reload
**And** implementation deferred to future epic (placeholder for now)

### **Error Handling:**

**Given** Registry operations can fail
**When** I handle errors
**Then** return `SchemaError::RegistryError { message }` for:
- Init called multiple times (already initialized)
- Global accessed before init (not initialized)
- Property name conflicts (override existing base property - log warning)

### **Testing:**

**Given** PropertyBankRegistry is implemented
**When** I write unit tests
**Then** test initialization (init once succeeds, subsequent init fails)
**And** test global singleton access
**And** test lookup priority (overrides before base)
**And** test runtime override registration
**And** test performance: base lookup <10ns, override lookup <50ns
**And** test concurrent access (multiple threads lookup simultaneously)
**And** integration test: load property_bank.json → init Registry → lookup properties

---

## Story 7.8: Implement Schema Command Adapters

As a developer coordinating schema operations,
I want Command and Query adapters orchestrating all schema utilities,
So that schema loading, caching, and querying work together seamlessly.

**Acceptance Criteria:**

### **SchemaCommand Adapter:**

**Given** I create SchemaCommand in `crates/adapters/src/spi/schema/command.rs`
**When** I design the structure
**Then** it holds:
```rust
pub struct SchemaCommand {
    loader: SchemaLoader,
    decoder: Decoder,
    validator: Validator,
    cache: Box<dyn SchemaCache>,
    registry: Arc<PropertyBankRegistry>,
}
```

**Given** SchemaCommand initialization
**When** Command adapter is created
**Then** initialize SchemaLoader with root path from Config
**And** initialize cache (RedbSchemaCache or MockSchemaCache)
**And** reference global PropertyBankRegistry
**And** Decoder and Validator are stateless (no initialization needed)

### **SchemaCommand.load_all() Implementation:**

**Given** SchemaCommand.load_all()
**When** I orchestrate full load
**Then** **Step 1:** Load PropertyBank
- `loader.load_property_bank()` → serde_json::Value
- `validator.validate(property_bank_value)` → validate structure
- `decoder.decode_property_bank(property_bank_value)` → PropertyBank
- `PropertyBankRegistry::init(bank)` → initialize singleton

**And** **Step 2:** Load all schema files
- `loader.load_all()` → Vec<(PathBuf, serde_json::Value)>
- For each (path, parsed_value):
  - `validator.validate(parsed_value, path)` → check syntax + semantics
  - `decoder.decode(parsed_value, path)` → RawSchema
  - Collect all RawSchemas

**And** **Step 3:** Detect circular inheritance
- Build dependency graph from RawSchemas (extends relationships)
- Perform DFS to detect cycles
- If cycle found, return `SchemaError::CircularInheritance { chain }`

**And** **Step 4:** Topological sort
- Compute resolution order (parents before children)
- Example order: [base_note, task, task_project, task_meeting]

**And** **Step 5:** Resolve schemas in order
- For each RawSchema in topological order:
  - Check cache: `cache.get(name)` with hash validation
  - If cache hit, use cached Schema
  - If cache miss:
    - Lookup parent Schema from cache (if extends exists)
    - Call `Resolver::resolve(raw, parent, registry.base)`
    - Store resolved Schema: `cache.put(name, schema)`

**And** **Step 6:** Return success
- Return `Ok(())` if all schemas resolved
- Log summary: "Loaded X schemas (Y from cache, Z resolved)"

**Given** load_all() error handling
**When** individual schema fails
**Then** continue processing other schemas (resilience)
**And** collect errors for reporting
**And** return aggregated error summary at end

### **SchemaCommand.refresh(name) Implementation:**

**Given** SchemaCommand.refresh(name)
**When** I reload single schema
**Then** **Step 1:** Invalidate cache: `cache.invalidate(name)`
**And** **Step 2:** Load file: `loader.load_file(name)`
**And** **Step 3:** Validate: `validator.validate(parsed)`
**And** **Step 4:** Decode: `decoder.decode(parsed)`
**And** **Step 5:** Resolve: `Resolver::resolve(raw, parent, registry)`
**And** **Step 6:** Update cache: `cache.put(name, schema)`
**And** return `Ok(())` if successful

### **SchemaQuery Adapter:**

**Given** I create SchemaQuery in `crates/adapters/src/spi/schema/query.rs`
**When** I design the structure
**Then** it holds:
```rust
pub struct SchemaQuery {
    cache: Box<dyn SchemaCache>,
}
```

**Given** SchemaQuery is read-only
**When** I implement methods
**Then** `get(name)` returns `Result<Option<Schema>, SchemaError>`
**And** `list()` returns `Result<Vec<SchemaName>, SchemaError>`
**And** NO side effects (no file loading, no resolution)

**Given** SchemaQuery.get(name)
**When** I retrieve schema
**Then** call `cache.get(name)` → Option<Schema>
**And** if Some(schema), return Ok(Some(schema))
**And** if None, return Ok(None) (schema not loaded or doesn't exist)
**And** does NOT trigger file load (Command is responsible for loading)

**Given** SchemaQuery.list()
**When** I list all schemas
**Then** iterate over all cache entries
**And** collect schema names
**And** return `Ok(Vec<SchemaName>)`

### **Port Trait Implementations:**

**Given** domain ports exist in `crates/domain/src/ports/schema.rs`
**When** I implement traits
**Then** SchemaCommand implements `SchemaCommandPort`
**And** SchemaQuery implements `SchemaQueryPort`
**And** methods match port signatures (async if needed)

### **Module Exports:**

**Given** adapters are implemented
**When** I export them in `crates/adapters/src/spi/schema/mod.rs`
**Then** re-export with Schema prefix:
```rust
pub use command::SchemaCommand;
pub use query::SchemaQuery;
pub use loader::SchemaLoader;
pub use decoder::Decoder;
pub use validator::Validator;
pub use cache::{SchemaCache, RedbSchemaCache, MockSchemaCache};
pub use registry::PropertyBankRegistry;
```

### **Error Handling:**

**Given** adapters coordinate multiple components
**When** I handle errors
**Then** propagate component-specific errors (IoError, ParseError, ValidationError, etc.)
**And** wrap in `SchemaError` with context (which operation failed)
**And** example: "Failed to load schema 'task.json': Validator error: Missing required field 'type' at property 'priority'"

### **Testing:**

**Given** Command and Query adapters
**When** I write integration tests
**Then** test full load_all() pipeline: load → validate → decode → resolve → cache
**And** test cache hit path (second load_all() uses cache)
**And** test refresh() updates single schema
**And** test Query.get() after Command.load_all()
**And** test Query.list() returns all loaded schemas
**And** test error: circular inheritance detected
**And** test error: invalid schema file (validation failure)
**And** test error: missing PropertyBank file
**And** use `example_vault/.lithos/schemas/` as test data

---

## Story 7.9: Review Epic 7 Test Suite & Documentation

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 7 test suite and documentation to its foundation,
So that tests are comprehensive, maintainable, documentation is clear, and the system catches real-world issues before production deployment.

**Acceptance Criteria:**

### **Test Suite Review:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 7 components are implemented (Loader, Decoder, Validator, Resolver integration, Cache, Registry, Command, Query)
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

**Given** format strategies (JSON/TOML/YAML)
**When** I test Decoder
**Then** parameterized tests verify identical `RawSchema` output for equivalent inputs in different formats

**Given** inheritance logic
**When** I test resolution
**Then** integration tests verify multi-level inheritance matches expectations
**And** test circular inheritance detection
**And** test excludes handling
**And** test property override

**Given** cache behavior
**When** I test SchemaCache
**Then** test cache hit (no re-resolution)
**And** test cache miss (triggers full pipeline)
**And** test hash invalidation (file changed)
**And** test persistence (restart → cache available)

**Given** PropertyBank singleton
**When** I test Registry
**Then** test initialization idempotency
**And** test lookup priority (overrides → base)
**And** test concurrent access (multiple threads)
**And** benchmark: <10ns for base lookup, <50ns for override lookup

### **Documentation Creation:**

**Given** Epic 7 implementation is complete
**When** I create developer documentation
**Then** create `docs/adapters/schema-system.md` following architecture doc pattern
**And** document orchestration flow: Loader → Decoder → Validator → Resolver → Cache
**And** explain user format vs domain format distinction (object properties → array properties)
**And** document PropertyBank singleton pattern and usage
**And** document SchemaCache architecture and Redb integration

**Given** module-level documentation
**When** I create README
**Then** create `crates/adapters/src/spi/schema/README.md` with quick start examples
**And** show example: load schema, query schema, register override property
**And** explain each component's responsibility (Loader, Decoder, Validator, etc.)

**Given** example_vault documentation
**When** I review example_vault/README.md
**Then** ensure it documents all 41 schemas clearly
**And** provide usage examples for starter kit
**And** explain how test fixtures work

**Given** vault-schema.schema.json documentation
**When** I add user guidance
**Then** add comments explaining each field (name, properties, extends, excludes)
**And** add examples for each property type (string, number, bool, date, file)
**And** document constraint usage (enum, pattern, min/max, format, file_class/directory)

**Given** migration guide needed
**When** users reference old docs/schemas/
**Then** create `docs/migrations/schemas-to-example-vault.md`
**And** explain relocation: docs/schemas/ → example_vault/.lithos/schemas/
**And** update any references in existing documentation

### **Doc Test Requirements:**

**Given** public APIs exist
**When** I write doc tests
**Then** SchemaLoader has doc test showing file load
**And** Decoder has doc test showing user format → RawSchema transformation
**And** Validator has doc test showing validation success/failure
**And** PropertyBankRegistry has doc test showing lookup
**And** SchemaCommand has doc test showing load_all() usage
**And** SchemaQuery has doc test showing get() usage

### **Edge Cases and Error Scenarios:**

**Given** I take adversarial position
**When** I identify missing test coverage
**Then** test empty properties object (valid: no properties defined)
**And** test excludes non-existent property (semantic error)
**And** test $ref to missing PropertyBank entry (PropertyNotFound error)
**And** test malformed JSON (syntax error with line/column)
**And** test invalid schema name (non-alphanumeric)
**And** test extends non-existent parent (SchemaNotFound error)
**And** test very deep inheritance (20 levels, performance test)
**And** test large schema (1000 properties, performance test)

### **Performance Benchmarks:**

**Given** Epic 7 operations are performance-critical
**When** I create benchmarks
**Then** benchmark SchemaLoader.load_all() with 41 schemas
**And** benchmark Decoder transformation (100 schemas)
**And** benchmark Validator.validate() (100 schemas)
**And** benchmark SchemaResolver resolution (multi-level inheritance)
**And** benchmark SchemaCache get/put operations
**And** benchmark PropertyBankRegistry lookup (base vs override)
**And** target: load_all() <100ms for 41 schemas (cold cache)
**And** target: load_all() <10ms for 41 schemas (hot cache)

---
