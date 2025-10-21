# Data Models

This section defines the data models used throughout the system, organized by architectural layer per hexagonal architecture principles. Models are classified as:

- **Domain Core:** Pure business logic, no infrastructure dependencies
- **API Adapter (Application Programming Interface):** Models used by driving adapters (CLI, TUI, LSP) that invoke domain operations
- **SPI Adapter (Service Provider Interface):** Models used by driven adapters (storage, filesystem, config) that provide services to domain
- **Split:** Models that span domain and adapter layers with clear separation of concerns

Domain models live in the core and define what the business cares about. Adapter models live at the edges and handle infrastructure concerns like filesystem I/O, caching, serialization, and user interaction.

## File

**Purpose:** Represents physical file identity and filesystem metadata. Separates file system concerns from content metadata, following Templater's proven design pattern of separating `tp.file.*` (file operations) from `tp.frontmatter.*` (content metadata).

**Architecture Layer:** Domain Core (with SPI Adapter concerns)

**Rationale:** File represents the physical note file's identity and metadata. While filesystem operations are adapter concerns, the File model itself is a domain concept—the domain needs to identify and reference notes via file paths. This separation from Frontmatter (content metadata) mirrors Templater's module architecture and prevents conflating "where the note is" from "what the note contains." Enables clean template function mapping (`{{ .File.Path }}` → `tp.file.path()`).

**Key Attributes:**

- `Path` (string) - Absolute path to note file. Serves as primary key and unique identifier across the system. Immutable once set. Used for body loading, cache keys, wikilink resolution, and note identification. Format: OS-specific absolute path (e.g., `/vault/notes/contact.md`).
- `Basename` (string, computed) - Filename without path and extension. Computed from Path using `filepath.Base()` and `strings.TrimSuffix()`. Used by template `lookup()` function (returns `map[basename]Path`) and wikilink resolution `[[basename]]`. Computed once during construction, cached in struct.
- `Folder` (string, computed) - Parent directory path. Computed from Path using `filepath.Dir()`. Used by template functions for file organization queries (e.g., "all notes in projects/"). Computed once during construction, cached in struct.
- `ModTime` (time.Time) - File modification timestamp from `os.Stat()`. Used for staleness detection by comparing cached ModTime against current filesystem ModTime. Enables incremental indexing optimizations (scan only files modified since last index). Format: RFC3339 for JSON serialization.

**Relationships:**

- File composed into Note (File + Frontmatter = complete note representation)
- File used by Query Service for path-based filtering and lookups
- File created during vault indexing from `os.Stat()` metadata
- File persisted to `.lithos/index/vault.json` as part of Note serialization

**Design Decisions:**

- **No stdlib abstraction:** Direct struct properties instead of `fs.FileInfo` interface. Simpler for MVP—easier testing (struct literals), fewer abstractions, direct property access. We call `os.Stat()` anyway for ModTime, so no savings from interface.

- **Computed fields cached:** Basename and Folder computed once during construction using stdlib `filepath` functions. Prevents repeated string manipulation during queries/templates. Not recomputed on deserialization—values stored in JSON.

- **Path as primary key:** Absolute file path uniquely identifies notes. Alternative considered: generate UUID, but adds complexity without MVP benefit. File paths are stable within vault lifetime.

- **ModTime enables optimizations:** Storing modification time enables future incremental indexing (Epic 3 enhancement). Adapter can skip re-parsing unchanged files. Zero cost for MVP, value for post-MVP.

- **Separation from content:** File model contains zero content metadata. All content concerns (frontmatter, fileClass, tags) belong in Frontmatter model. Clean separation mirrors Templater's `tp.file.*` vs `tp.frontmatter.*` module design.

**Helper Functions:**

```go
// NewFile creates File from path and filesystem metadata
// Called by adapter during vault indexing
func NewFile(path string, modTime time.Time) File {
    return File{
        Path:     path,
        Basename: computeBasename(path),
        Folder:   computeFolder(path),
        ModTime:  modTime,
    }
}

// computeBasename extracts basename from file path
// Removes path and extension (e.g., "/vault/note.md" → "note")
func computeBasename(path string) string {
    base := filepath.Base(path)
    return strings.TrimSuffix(base, filepath.Ext(base))
}

// computeFolder extracts parent directory from file path
// Returns directory path (e.g., "/vault/note.md" → "/vault")
func computeFolder(path string) string {
    return filepath.Dir(path)
}
```

**Additional Information:**

The File model represents Lithos's commitment to domain-driven design inspired by Templater's proven architecture. By separating file identity from content metadata, we create clean boundaries that map directly to template functions (`{{ .File.Path }}`, `{{ .File.Folder }}`). The model is intentionally minimal—only filesystem identity and metadata, zero content concerns. This separation enables independent evolution: file operations don't touch frontmatter, and frontmatter validation doesn't care about file paths. The computed fields (Basename, Folder) are cached to avoid repeated string operations during template rendering. ModTime positions us for future incremental indexing without model changes.

---

## Frontmatter

**Purpose:** Represents note content metadata extracted from YAML frontmatter. Separates content concerns from file identity, following Templater's `tp.frontmatter.*` module pattern.

**Architecture Layer:** Domain Core

**Rationale:** Frontmatter is pure domain logic—it defines what the note contains, not where it lives. This model encapsulates all content metadata concerns: schema references (fileClass), user-defined fields, and structured metadata. Validation logic operates exclusively on Frontmatter, never on File. Mirrors Templater's clean separation between file operations and frontmatter access. Properties (schema definitions) validate Fields (actual data).

**Key Attributes:**

- `FileClass` (string, optional) - Schema reference extracted from `fields["fileClass"]`. Used for validation lookup (links to Schema.Name). Empty string if note has no fileClass. Denormalized from Fields map for query performance (avoids repeated map lookups during filtering).
- `Fields` (map[string]interface{}) - Complete parsed YAML frontmatter as flexible map. Preserves all user-defined fields including unknown fields (FR6). Keys are case-sensitive. Supports nested maps for structured metadata. This is the authoritative source for all note content metadata.

**Relationships:**

- Frontmatter validated against Schema by Validator (Schema.Properties validate Frontmatter.Fields)
- Frontmatter composed into Note (File + Frontmatter = complete note)
- Frontmatter created during vault indexing from YAML parsing
- Frontmatter persisted to `.lithos/index/vault.json` as part of Note serialization

**Design Decisions:**

- **FileClass denormalized:** Extracted from Fields map and stored separately for query performance. Enables fast filtering by fileClass without parsing Fields. Trade-off: slight data duplication for significant query speedup.

- **Fields preserved as-is:** Complete frontmatter map stored without filtering. Supports FR6 requirement (preserve unknown fields). Validation happens separately via Validator + Schema (not at model level). Unknown fields pass through untouched.

- **Flexible map over struct:** Using `map[string]interface{}` instead of typed struct enables schema-free notes and user-defined fields. Aligns with Obsidian's flexible frontmatter philosophy. Type checking happens at validation layer, not model layer.

- **Fields vs Properties terminology:** "Fields" = actual data values in frontmatter. "Properties" = schema definitions/rules. This distinction eliminates ambiguity and aligns with JSON Schema terminology.

**Helper Functions:**

```go
// NewFrontmatter creates Frontmatter from parsed YAML
// Called by adapter after YAML parsing
func NewFrontmatter(fields map[string]interface{}) Frontmatter {
    return Frontmatter{
        FileClass: extractFileClass(fields),
        Fields:    fields,
    }
}

// extractFileClass safely extracts fileClass from frontmatter fields
// Returns empty string if not present or wrong type
func extractFileClass(fields map[string]interface{}) string {
    if fc, ok := fields["fileClass"].(string); ok {
        return fc
    }
    return ""
}

// SchemaName returns FileClass for schema validation lookup
// Usage: validator.Validate(frontmatter.SchemaName(), frontmatter)
func (f Frontmatter) SchemaName() string {
    return f.FileClass
}
```

**Additional Information:**

The Frontmatter model embodies separation of concerns. It knows nothing about files—no paths, no timestamps, no folders. It represents what the note says, not where it lives. This enables clean template function design (`{{ .Frontmatter.Fields.title }}`), straightforward validation (Validator operates only on Frontmatter), and independent evolution (file operations never touch frontmatter). The flexible map structure honors Obsidian's philosophy of user freedom while the FileClass denormalization provides query performance.

---

## Note

**Purpose:** Composite model combining File (identity/metadata) and Frontmatter (content/metadata). Represents a complete markdown note for operations requiring both file and content information.

**Architecture Layer:** Domain Core

**Rationale:** Note is a composition of File and Frontmatter, providing a unified view when operations need both aspects. Most queries work with just File OR Frontmatter, but template rendering, validation, and indexing need both. This composition pattern follows domain-driven design principles—Note is the aggregate root, File and Frontmatter are value objects.

**Key Attributes:**

- `File` (File) - File identity and filesystem metadata (embedded struct)
- `Frontmatter` (Frontmatter) - Content metadata from YAML frontmatter (embedded struct)

**Relationships:**

- Note validated by Validator (uses both File.Path for error messages and Frontmatter for validation)
- Note returned by Query Service when both file and content data needed
- Note created during vault indexing (File from `os.Stat()`, Frontmatter from YAML parsing)
- Note persisted to `.lithos/index/vault.json` (complete serialization)
- Note used by Template Service for rendering (templates access both File and Frontmatter)

**Design Decisions:**

- **Composition over inheritance:** Note embeds File and Frontmatter as struct fields. Enables direct property access in templates: `{{ .File.Path }}`, `{{ .Frontmatter.Fields.title }}`. Simpler than accessor methods.

- **Aggregate root pattern:** Note is the aggregate root in DDD terms. File and Frontmatter are value objects with no identity outside Note context. Operations that modify note data go through Note constructor, ensuring consistency.

- **CQRS in operations:** The CQRS separation is in **operations** (CacheCommandPort for writes, CacheQueryPort for reads), not models. Single Note model used by both command and query sides. Provides CQRS benefits without model proliferation complexity.

- **Template-friendly structure:** Embedded structs (not pointers) enable clean template syntax without nil checks. Templates access `{{ .File.Basename }}` directly. Reduces template boilerplate.

**Helper Functions:**

```go
// NewNote creates Note from file and frontmatter components
// Called by adapter during vault indexing after parsing
func NewNote(file File, frontmatter Frontmatter) Note {
    return Note{
        File:        file,
        Frontmatter: frontmatter,
    }
}

// SchemaName returns FileClass from embedded Frontmatter
// Convenience method for schema validation
func (n Note) SchemaName() string {
    return n.Frontmatter.FileClass
}
```

**Additional Information:**

The Note model represents Lithos's pragmatic approach to domain modeling—split where it matters (File vs Frontmatter), compose where it helps (Note aggregate). This three-model structure mirrors Templater's proven module architecture while enabling Go idioms (struct embedding, value types). The composition provides flexibility: queries can return `[]File` when only paths needed, validation operates on `Frontmatter`, but template rendering gets full `Note`. CQRS benefits come from operational separation, not model separation, avoiding translation overhead.

**Post-MVP Considerations - Body Content Indexing:**

When to add body parsing (future indicators):

- **Heading extraction:** Parse markdown headings for navigation/search
- **Tag extraction:** Parse inline tags (#tag) beyond frontmatter tags
- **Link graph:** Parse wikilinks [[note]] for relationship mapping
- **Block references:** Parse block IDs for Obsidian block-reference support

Current MVP only indexes frontmatter. Post-MVP Phase 3 (Enhanced Querying) may require body parsing and richer Note.Body model.

---

## Schema

**Purpose:** Defines metadata class structure with property constraints and inheritance. Governs validation rules for notes of a given `fileClass`. Schemas are loaded from JSON files and resolved at application startup.

**Architecture Layer:** Domain Core

**Rationale:** Schema represents core business rules about what constitutes valid metadata. Validation logic, inheritance semantics, and property constraints are pure domain concerns with no infrastructure dependencies. Schemas are loaded by adapters but the Schema model itself is domain logic.

**Key Attributes:**

- `Name` (string) - Schema identifier matching `fileClass` frontmatter value (e.g., "contact", "project", "daily-note"). Must be unique within vault. Used as key in schema registry map.
- `Extends` (string, optional) - Parent schema name for inheritance chains. References another schema by Name (not pointer to avoid cycles). Can form multi-level chains (e.g., "fleeting-note" extends "base-note" extends "note"). Empty string means no parent.
- `Excludes` ([]string, optional) - Parent property names to remove from inherited schema. Enables subtractive inheritance when child needs to narrow parent's property set. Applied after parent resolution, before child property merging. Property names must match exactly (case-sensitive).
- `Properties` ([]Property) - Property definitions declared in this schema file. Represents the delta/override for inherited schemas, or complete property set for root schemas. Properties with same name as parent override parent definition.
- `ResolvedProperties` ([]Property, computed) - Flattened property list after applying inheritance resolution, exclusions, and merging. Computed by Builder pattern during schema loading. This is the authoritative property list used by Validator. Never persisted to disk (always computed).

**Relationships:**

- Schema extends another Schema (optional, supports C→B→A chains). Resolution is recursive.
- Schema contains multiple Property definitions (one-to-many composition)
- Schema.Properties may reference PropertyBank entries via `$ref` (resolved during schema loading before inheritance)
- Note.Frontmatter validated against Schema (lookup via frontmatter `fileClass` → Schema.Name)
- Schema resolved by Builder pattern at load time (eager resolution, not lazy)

**Design Decisions:**

- **Properties vs Fields terminology:** Schema has "Properties" (validation rules). Note.Frontmatter has "Fields" (actual data). This semantic distinction eliminates ambiguity. Validator checks if Fields satisfy Property constraints.

- **Separate Properties vs ResolvedProperties:** Source `Properties` preserves original JSON content (immutable). `ResolvedProperties` is computed output of Builder pattern. Separation prevents mutating source data and enables audit trail of inheritance.

- **Excludes for subtractive inheritance:** Added to support real-world scenario where child schema needs fewer properties than parent (e.g., simplified version of a complex schema). Without this, inheritance is only additive.

- **String-based Extends reference:** Uses schema name string, not Go pointer, to avoid circular dependency issues in struct definitions. Schema registry (map[string]\*Schema) resolves references after all schemas loaded.

- **Eager resolution at startup:** Inheritance chains resolved during application initialization (fail-fast on circular dependencies per Epic 2, Story 2.6). Validator never sees unresolved schemas. Performance: O(n\*d) where n=schemas, d=depth, acceptable for MVP (<100 schemas expected).

- **Resolution order:** (1) Load all schema files, (2) Build dependency graph, (3) Detect cycles, (4) Resolve in topological order (leaves first), (5) For each schema: get parent's ResolvedProperties → apply Excludes → merge/override with child Properties → store in ResolvedProperties.

- **Property override semantics:** If child Property.Name matches parent Property.Name, child completely replaces parent (not merging property attributes). This is explicit override, not attribute-level merge.

**Additional Information:**

Schema inheritance provides powerful reusability for similar note types. For example, a base "note" schema could define common properties (title, tags, created), while specialized schemas like "meeting-note" or "person" extend the base and add domain-specific properties. The eager resolution strategy ensures validation is fast (no runtime resolution overhead) at the cost of slightly longer startup time. For MVP with <100 schemas, this tradeoff is acceptable. The Builder pattern isolates complexity—domain validators simply receive fully-resolved schemas and don't need to understand inheritance mechanics.

> **Adapter boundary reminder:** Schema definitions are serialized as JSON on disk, but decoding and discriminator handling occur in the SchemaLoader adapter (see Epic 2, Story 2.4). The domain models described here stay infrastructure-free and are instantiated via constructors that enforce the rules above.

---

## Property

**Purpose:** Defines a single metadata property with type constraints, validation rules, and optional dynamic value sourcing. Building block of schema definitions. Properties describe what data can be stored in frontmatter and how it should be validated.

**Architecture Layer:** Domain Core

**Rationale:** Property constraints are business rules that define valid metadata structure. Validation logic (required, array, type checking) is pure domain behavior. No dependencies on filesystem, network, or UI concerns. Properties (schema definitions) validate Fields (actual frontmatter data).

**Key Attributes:**

- `Name` (string) - Property identifier matching frontmatter key. Case-sensitive. Must be valid YAML key (no special chars except dash/underscore). Used as map key when validating Note.Frontmatter.Fields.
- `Required` (boolean) - Whether property must be present in note's frontmatter. If true, validation fails if key missing. If false, property is optional. Does not apply to array properties (empty array is valid for required array properties).
- `Array` (boolean) - Whether property accepts multiple values (YAML list) vs single value. If true, frontmatter value must be array (even if single element). If false, value must be scalar. Validator checks type before invoking PropertySpec validation.
- `Spec` (PropertySpec interface) - Type-specific configuration containing validation rules and constraints. Exactly one spec type per property based on semantic type (string, number, date, file, boolean). Contains the "how to validate" logic.

**Relationships:**

- Property belongs to Schema (composition, owned by Schema.Properties and Schema.ResolvedProperties)
- Property contains exactly one PropertySpec implementation (interface polymorphism)
- Property may be sourced from PropertyBank via `$ref` (resolved during schema loading)

**Design Decisions:**

- **Properties vs Fields terminology:** "Property" = schema definition with validation rules. "Field" = actual data value in frontmatter. Clear semantic distinction prevents confusion. Validator checks if Frontmatter.Fields satisfy Schema.Properties constraints.

- **Composition with PropertySpec interface:** Eliminates the "God Object" problem of having many nullable attributes (enum, pattern, format, query, min, max). Each property type has dedicated spec struct with only relevant attributes. Type system enforces valid combinations at compile time.

- **Interface-based polymorphism:** PropertySpec interface allows each type to implement custom `Validate(value interface{}) error` logic. Common behavior abstracted to interface, type-specific behavior encapsulated in concrete implementations.

- **Standard JSON unmarshaling (MVP):** Schemas use JSON format for simpler stdlib unmarshaling (~20 LOC). Go's `encoding/json` handles discriminator-based unmarshaling via struct tags. Inspect `type` field, then unmarshal into appropriate PropertySpec. No custom unmarshaling needed for MVP.

- **Required vs Array semantics:** `Required` means "key must exist." For array properties, empty array `[]` satisfies required constraint (property exists, just has no values). To require non-empty array, add validation to PropertySpec.

- **Name validation:** While Name is any string in model, Schema loader should validate that names are valid YAML keys and don't conflict with Lithos reserved fields (e.g., "fileClass" is user-defined, but loader could warn about shadowing).

- **PropertyBank reference support:** Property can be created via direct definition OR by reference to PropertyBank entry using `$ref` attribute. Schema loader resolves references before schema validation. See PropertyBank model for reference resolution pattern.

**Additional Information:**

The Property model uses composition over inheritance through the PropertySpec interface, following the Interface Segregation Principle. This design prevents the anti-pattern of a massive Property struct with dozens of nullable attributes where only a few apply to each property type. Custom JSON unmarshaling happens transparently—users just write intuitive schema JSON, and the loader constructs the correct PropertySpec implementation. The `Required` and `Array` attributes are orthogonal—you can have required scalars, optional scalars, required arrays, or optional arrays. This covers all common validation scenarios without complex nested rules.

---

## PropertySpec (Type-Specific Configurations)

**Purpose:** Type-specific validation rules and constraints for Property definitions. Enables clean separation of concerns for different property types. Each PropertySpec implementation knows how to validate values of its type.

**Architecture Layer:** Domain Core

**Rationale:** Validation rules are business logic. Each spec type (StringPropertySpec, NumberPropertySpec, etc.) encapsulates domain-specific validation behavior using stdlib functions. No infrastructure dependencies—validation is pure logic.

**Variants:**

**StringPropertySpec:**

- `Enum` ([]string, optional) - Allowed values as fixed list. If non-empty, value must be in list (exact match, case-sensitive). Empty list means no enum constraint (any string valid).
- `Pattern` (string, optional) - Regex pattern for custom string validation. If non-empty, value must match pattern. Uses Go `regexp` package. Empty string means no pattern constraint.
- **Validation logic:** Check enum first (if present), then pattern (if present). Both constraints can coexist (value must be in enum AND match pattern).
- **Example:** `enum: ["red", "green", "blue"]` or `pattern: "^[A-Z][a-z]+$"` (capitalized word)

**NumberPropertySpec:**

- `Min` (float64, optional) - Minimum allowed value (inclusive). Nullable pointer distinguishes "not set" from "0". If set, value must be >= Min.
- `Max` (float64, optional) - Maximum allowed value (inclusive). If set, value must be <= Max.
- `Step` (float64, optional) - Increment/decrement amount. If 1.0, implies integer values (validator checks for fractional part). If 0.1, implies one decimal precision. If nil, any precision allowed.
- **Validation logic:** All numbers treated as float64 (YAML unmarshals numbers as float64). If Step=1.0, check that value has no fractional part (i.e., is integer). Check Min/Max bounds.
- **Design rationale:** Unified number type (no separate int/float) aligns with YAML/JSON lack of type distinction. Step provides semantic hint for integer vs decimal.
- **Example:** `min: 0, max: 100, step: 1` (integer 0-100) or `min: 0.0, max: 1.0, step: 0.01` (percentage with 2 decimals)

**DatePropertySpec:**

- `Format` (string) - Go time layout string (e.g., "2006-01-02", "2006-01-02T15:04:05Z07:00"). Uses Go stdlib `time.Parse(format, value)`. If empty, defaults to RFC3339.
- **Validation logic:** Parse string value using `time.Parse(Format, value)`. If parse succeeds, value is valid date. If parse fails, return error with format hint.
- **Example:** `format: "2006-01-02"` (ISO date) or `format: "Jan 2, 2006"` (human-readable)

**FilePropertySpec:**

- `FileClass` (string, optional) - Restricts valid file references to notes with specific fileClass value or regex pattern. Supports negation via `^` prefix. Examples: `"project"` (exact match), `"^archive"` (NOT archive), `"(project|task)"` (regex: project OR task). Empty string means no fileClass restriction.
- `Directory` (string, optional) - Restricts valid file references to notes within specific vault directory path. Path is relative to vault root. Supports negation via `^` prefix. Examples: `"projects/"` (notes in projects/), `"^archive/"` (NOT in archive/), `"work/.*"` (regex: anything under work/). Empty string means no directory restriction.
- **Validation logic:** Check that value is valid file path (either absolute path or wikilink format `[[basename]]`). If FileClass or Directory constraints set, validate that referenced file exists in vault index and matches ALL specified constraints (conjunction/AND). Negation inverts the match.
- **Filter conjunction:** When both FileClass and Directory set, both conditions must be satisfied (AND logic). Example: `{"fileClass": "project", "directory": "work/"}` matches project notes in work/ directory only.

**BoolPropertySpec:**

- No additional configuration. Values must be boolean (true/false).
- **Validation logic:** Type check only - value must be Go bool type.

**Relationships:**

- Exactly one PropertySpec variant per Property (composition via interface)
- FilePropertySpec uses FileClass/Directory attributes for dynamic lookups (no separate filter model)
- Each PropertySpec implements `Validate(value interface{}) error` method

**Design Decisions:**

- **Unified NumberPropertySpec:** Handles both integer and float via `Step` attribute. Simplifies type system and aligns with YAML's lack of int/float distinction. Validator detects integer vs decimal based on Step value.

- **Interface-based validation:** Each spec implements its own `Validate()` method. Enables polymorphic validation without type switches in validator. Validator just calls `property.Spec.Validate(value)` regardless of concrete type.

- **Step-based integer detection:** If Step=1.0, validator checks `value == math.Floor(value)`. This is semantic check (not type check), aligning with YAML treating `42` and `42.0` identically.

- **Stdlib-based validation:** All validation uses Go stdlib: `strings.Contains`, `regexp.MatchString`, `time.Parse`, `reflect.DeepEqual`. This ensures consistent error messages and behavior. No third-party validation libraries needed.

- **Nil pointer semantics:** For optional attributes (Enum, Pattern, Min, Max, Step, FileClass, Directory), nil pointer means "no constraint." Empty value (empty slice, zero number) has different meaning (e.g., empty Enum list = no values allowed, nil Enum = any value allowed).

- **FilePropertySpec flattened attributes (MVP):** FileClass and Directory are direct attributes on FilePropertySpec for MVP simplicity. Post-MVP could introduce nested `Filter` struct with additional filter types (Tags, ModTime, etc.) while maintaining backward compatibility via JSON unmarshaling.

**Additional Information:**

PropertySpec variants provide type-safe validation without runtime type switching. The interface-based approach enables polymorphism—validators work with the PropertySpec interface, unaware of concrete implementations. Each variant uses only Go stdlib for validation, ensuring portability and consistent behavior across platforms. The NumberPropertySpec's unified approach (no separate int/float types) aligns with YAML/JSON semantics where `42` and `42.0` are equivalent. The Step attribute provides semantic meaning (integer vs decimal) without artificial type boundaries. FilePropertySpec's Filter integration enables dynamic validation—property values can be constrained to notes matching arbitrary query conditions, supporting use cases like "only valid project references."

---

## PropertyBank

**Purpose:** Provides a library of reusable, pre-configured Property definitions that schemas can reference by name. Reduces duplication across schema definitions, ensures consistency for common properties (e.g., `standard_title`, `standard_tags`), and enables centralized property definition management.

**Architecture Layer:** Domain Core

**Rationale:** PropertyBank is pure domain concern—it's a registry of business rules (property constraints) that can be reused. No infrastructure dependencies. Loaded by SchemaLoaderPort adapter from JSON files, but the model itself represents domain knowledge about common property patterns.

**Key Attributes:**

- `Properties` (map[string]Property) - Named property definitions keyed by unique identifier (e.g., "standard_title", "iso_date", "email_address")
- `Location` (string) - Path to property bank directory containing JSON definition files (default: `schemas/properties/`)

**Relationships:**

- PropertyBank loaded before Schema definitions during startup (SchemaLoaderPort orchestrates)
- Schema.Properties can reference PropertyBank entries via special `ref` attribute (resolved during schema loading)
- Property definitions in PropertyBank are templates—schemas can override specific attributes when referencing

**Reference Resolution Pattern:**

Schemas reference property bank entries using JSON reference syntax:

```json
{
  "name": "contact",
  "properties": [
    { "$ref": "#/properties/standard_title" },
    { "$ref": "#/properties/standard_tags" },
    {
      "name": "email",
      "required": true,
      "type": "string",
      "pattern": "^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"
    }
  ]
}
```

Property bank definitions stored in `schemas/properties/common.json`:

```json
{
  "properties": {
    "standard_title": {
      "name": "title",
      "required": true,
      "type": "string",
      "pattern": "^.{1,200}$"
    },
    "standard_tags": {
      "name": "tags",
      "required": false,
      "array": true,
      "type": "string"
    }
  }
}
```

**Design Decisions:**

- **Properties vs Fields terminology:** PropertyBank contains "Properties" (reusable validation rule definitions), not "Fields" (actual data). Consistent with Schema.Properties terminology. Clear semantic distinction.

- **JSON format:** Simpler unmarshaling than YAML (~20 LOC vs ~50 LOC custom code). Frontmatter remains YAML (Obsidian convention), but schema definitions prioritize Go stdlib integration.

- **Reference-based composition:** Schemas reference properties by ID (`$ref` JSON pointer syntax), not copy. Schema loader resolves references at load time by looking up PropertyBank registry.

- **Inline override support (Post-MVP):** Future enhancement could allow schemas to override specific property attributes when referencing:

  ```json
  {
    "$ref": "#/properties/standard_title",
    "required": false // Override: make title optional for this schema
  }
  ```

  MVP uses simple substitution (no attribute-level merging).

- **Multiple property bank files:** Supports organizing property banks by domain (e.g., `properties/common.json`, `properties/contacts.json`, `properties/projects.json`). All loaded into single registry keyed by property ID.

- **Load order:** PropertyBank loaded before schemas during SchemaLoaderPort.LoadSchemas() call. Ensures all references can be resolved.

- **No circular references:** Property banks cannot reference other property banks (flat structure). Post-MVP could add property inheritance if needed.

**Implementation Notes:**

Schema Engine Adapter adds ~50 LOC for property bank loading and reference resolution:

1. Scan `schemas/properties/*.json` files
2. Parse each file into PropertyBank structure
3. Merge all property definitions into single registry map
4. During schema parsing, detect `$ref` attributes
5. Look up referenced property in registry
6. Substitute reference with property definition
7. Continue with normal schema validation

**Additional Information:**

PropertyBank solves the "common property definition" problem elegantly. Without it, every schema must redefine standard properties like `title`, `tags`, `created`, `modified`—leading to inconsistencies (different patterns, required settings) and maintainability burden. With PropertyBank, define once, reference everywhere. The JSON format choice aligns with Go's excellent stdlib JSON support while keeping frontmatter in YAML (user-facing, Obsidian standard). The `$ref` syntax follows JSON Schema conventions, making it familiar to users with schema experience. Post-MVP could enhance with property inheritance, attribute-level overrides, or validation rules, but simple reference substitution covers 80% of reuse needs.

---

## Template

**Purpose:** Represents an executable template for note generation. Contains template metadata (filepath, display name) and content for execution. This is the **single Template model** used across all architectural layers. Following YAGNI principle for MVP, we use one model for both template discovery and execution.

**Architecture Layer:** Domain Core

**Rationale:** Template execution logic (parsing, rendering, function execution) is pure business logic. The model represents what the domain cares about: template identity (FilePath, Name), executable content (Content), and cached parse tree (Parsed). FilePath and Name are domain identifiers - the fact that adapters use FilePath to read files doesn't make them adapter concerns. Domain defines its need for file-based template identification. Content and Parsed are pure domain - no infrastructure dependencies in execution logic.

**Key Attributes:**

- `FilePath` (string) - Absolute path to template file. Serves as unique identifier for template references and composition. Immutable once set. Used by domain for template resolution (e.g., `{{template "section-name"}}`).
- `Name` (string) - Human-readable display name derived from filename (basename without .md extension). Used for template selection UI and references. Computed from FilePath during template loading.
- `Content` (string) - Raw template text. Contains Go `text/template` syntax with Lithos-specific function calls (`prompt`, `suggester`, `lookup`, `query`, `now`, etc.). Loaded from disk by adapters and provided to domain.
- `Parsed` (\*template.Template, optional) - Go template AST after parsing. Computed on first execution, cached for performance. Nil until first use. Pure domain concern - parsing and execution logic.

**Relationships:**

- Template may reference other Templates via `{{template "section-name"}}` directive (implicit, resolved by Go's text/template engine)
- Template executed during note generation with injected function map and data context
- Template functions access ports (StoragePort, InteractivePort, FileSystemPort) via closure during rendering
- Template discovered via filesystem scanning for `lithos find` command (FilePath and Name used for display)

**Design Decisions:**

- **Single model for MVP (YAGNI):** One Template definition used for both discovery and execution eliminates translation overhead. Simplifies codebase, reduces bugs, easier to understand. Can split into separate models post-MVP if lazy loading becomes critical.
- **FilePath as domain identifier:** FilePath is a domain concept - domain needs to identify and reference templates. Using file paths as identifiers is a valid domain choice, not adapter leakage. Alternative considered: abstract IDs with separate mapping, but adds complexity without MVP benefit.
- **Name computed from FilePath:** Display name derived from filename using same logic as Note.Basename. Keeps model simple, avoids data duplication. Computed once during template loading, cached in model.
- **Content loaded by adapters:** Adapters read file content and populate Template model. Domain receives ready-to-execute templates. This separation keeps file I/O in adapter layer while template model remains in domain.
- **Parsed AST cached:** After first execution, parsed template cached in model to avoid reparsing. Performance optimization - parsing happens once per template per application lifecycle.
- **Minimal composition model:** Template composition (FR1) handled by Go stdlib `text/template` via `{{define}}`/`{{template}}` directives. No custom section tracking needed - leverage mature, well-tested functionality.

**Additional Information:**

The unified Template model follows the same YAGNI principle as the unified Note model. FilePath and Name are identifiers, not infrastructure concerns - they're how the domain conceptualizes template identity. For MVP with small template directories (<100 templates), having a single model for both discovery and execution is pragmatic. The domain doesn't care that FilePath happens to be a filesystem path; it's simply the identifier. Post-MVP, if template management becomes more complex (e.g., templates from databases, APIs, or requiring lazy loading), we can refactor. Until then, one simple domain model serves all needs without premature abstraction.

---

## Config

**Purpose:** Application configuration loaded from `lithos.yaml` and environment variables. Defines vault structure and operational settings used by adapters.

**Architecture Layer:** SPI Adapter (Configuration)

**Rationale:** Configuration is pure infrastructure wiring - filesystem paths, log levels, directory locations. Domain never needs to know about these concerns. Viper adapter reads config and uses it to construct other adapters with appropriate settings.

**Key Attributes:**

- `VaultPath` (string) - Root directory of vault. Default: current working directory. All relative paths in config are resolved relative to this. Must exist and be readable. Viper searches upward from current directory to find `lithos.yaml`, then uses that directory as VaultPath.
- `TemplatesDir` (string) - Path to templates directory. Default: `{VaultPath}/templates`. Can be absolute or relative to VaultPath. Must exist for `lithos new` and `lithos find` commands. Scanner lists all `.md` files in this directory.
- `SchemasDir` (string) - Path to schemas directory. Default: `{VaultPath}/schemas`. Can be absolute or relative to VaultPath. Must exist if schemas are used. Loader parses all `.yaml` files in this directory at startup.
- `CacheDir` (string) - Path to index cache. Default: `{VaultPath}/.lithos/cache`. Can be absolute or relative to VaultPath. Created automatically if missing. Must be writable. FileCache adapter stores one `.json` file per indexed note.
- `LogLevel` (string) - Logging verbosity for zerolog. One of: "debug", "info", "warn", "error". Default: "info". Case-insensitive. Invalid values fall back to "info" with warning. Controls stdout/stderr output verbosity.

**Relationships:**

- Used by all adapters for initialization and runtime configuration
- Loaded at startup via Viper configuration adapter (reads `lithos.yaml`, environment variables, flags in that precedence order)
- Passed to components via constructor injection (dependency injection pattern)

**Design Decisions:**

- **Flat structure:** No nested config objects for MVP simplicity. All settings are top-level keys in YAML. This keeps config file simple and reduces unmarshaling complexity.
- **Sensible defaults:** Empty config file is valid - all paths default to sensible vault-relative locations. Enables quickstart: user can run `lithos index` with zero configuration if vault uses standard directory structure.
- **String paths:** Paths stored as strings, not file handles or custom Path types. Adapters resolve paths on demand using `filepath.Join` and `filepath.Abs`. This keeps config serializable and adapter-agnostic.
- **Validation at load time:** Config loading should validate that VaultPath exists, is directory, and is readable. Other paths validated lazily when accessed (TemplatesDir validated on first `lithos find`, not at config load).
- **Environment variable override:** Viper supports env vars like `LITHOS_VAULT_PATH`. Precedence: CLI flags > env vars > config file > defaults. This enables CI/CD override without modifying config files.
- **No secrets in config:** Config is committed to git (per PRD, vaults are git repositories). No API keys, tokens, or passwords. Future: if external API integrations added, use separate credential files or system keychain.

**Additional Information:**

Config is a pure adapter model - it exists solely to wire infrastructure concerns. The flat structure keeps configuration simple and readable for users. Sensible defaults mean a user can create an empty `lithos.yaml` and the application works immediately if using standard directory conventions. The precedence order (CLI flags > env vars > config file > defaults) provides flexibility for different environments - developers can override locally via flags, CI/CD can inject via environment variables, and teams can share baseline config in version control. String-based paths keep Config serializable and platform-agnostic - no special types needed.

---
