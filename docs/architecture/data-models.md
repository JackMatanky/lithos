# Data Models

This section defines the data models used throughout the system, organized by architectural layer per hexagonal architecture principles. Models are classified as:

- **Domain Core:** Pure business logic with no infrastructure dependencies - contains only essential business data
- **Write Models (CQRS Command):** Domain models optimized for data integrity, validation, and persistence
- **Read Models (CQRS Query):** Domain models denormalized and optimized for fast queries with pre-built indices
- **API Adapter (Application Programming Interface):** Models used by driving adapters (CLI, TUI, LSP) that invoke domain operations
- **SPI Adapter (Service Provider Interface):** Models used by driven adapters (storage, filesystem, config) that provide services to domain

Domain models live in the core and contain only essential business data. Infrastructure concerns (file paths, timestamps, serialization) belong in adapter layer. The CQRS pattern separates write concerns (validation, integrity) from read concerns (query performance).

## FileMetadata

**Purpose:** Filesystem-specific metadata used exclusively by filesystem storage adapters. Maps NoteID to file paths and tracks file system state.

**Architecture Layer:** SPI Adapter (Infrastructure)

**Rationale:** FileMetadata is infrastructure model used by FileSystemReadAdapter and FileSystemWriteAdapter to translate between domain identifiers (NoteID) and filesystem paths. Domain never depends on filesystem paths - adapters handle this translation. Enables filesystem implementation details to change without affecting domain.

**Key Attributes:**

- `Path` (string) - Absolute path to file. Serves as primary key and unique identifier across the system. Immutable once set. Used for cache keys, file identification, and adapter operations. Format: OS-specific absolute path (e.g., `/vault/notes/contact.md`).
- `Basename` (string, computed) - Filename without path and extension. Computed from Path using `filepath.Base()` and `strings.TrimSuffix()`. Used by template `lookup()` function (returns `map[basename]Path`) and wikilink resolution `[[basename]]`. Computed once during construction, cached in struct.
- `Folder` (string, computed) - Parent directory path. Computed from Path using `filepath.Dir()`. Used by template functions for file organization queries (e.g., "all notes in projects/"). Computed once during construction, cached in struct.
- `Ext` (string, computed) - File extension including dot. Computed from Path using `filepath.Ext()`. Used for file type filtering (e.g., ".md", ".pdf", ".png"). Empty string if no extension. Computed once during construction, cached in struct.
- `ModTime` (time.Time) - File modification timestamp from `os.Stat()`. Used for staleness detection by comparing cached ModTime against current filesystem ModTime. Enables incremental indexing optimizations (scan only files modified since last index). Format: RFC3339 for JSON serialization.
- `Size` (int64) - File size in bytes from `os.Stat()`. Used for filtering large files or determining if content should be loaded. Post-MVP: may skip content loading for files above threshold.
- `MimeType` (string, computed) - MIME type detected from file extension or content. Computed using `mime.TypeByExtension(Ext)` or `http.DetectContentType(content)`. Used for file type classification and handling. Examples: "text/markdown", "application/pdf", "image/png".

**Relationships:**

- Used internally by FileSystemReadAdapter and FileSystemWriteAdapter to map NoteID ↔ Path
- Never exposed to domain services
- Created during vault scanning by FileSystemReadAdapter
- Cached in adapters for performance

**Design Decisions:**

- **Adapter-only model:** Domain never sees or depends on filesystem paths - keeps infrastructure concerns isolated
- **Computed fields cached:** Basename/Folder computed once during construction to avoid repeated string operations
- **Staleness detection:** ModTime enables incremental indexing - skip unchanged files
- **Clean separation:** Keeps filesystem concerns out of domain layer
- **Shared by CQRS adapters:** Both FileSystemReadAdapter and FileSystemWriteAdapter use this metadata model

**Helper Functions:**

```go
// NewFileMetadata creates FileMetadata from path and fs.FileInfo
// Called by adapter during vault scanning
func NewFileMetadata(path string, info fs.FileInfo) FileMetadata {
    ext := filepath.Ext(path)
    return FileMetadata{
        Path:     path,
        Basename: computeBasename(path),
        Folder:   computeFolder(path),
        Ext:      ext,
        ModTime:  info.ModTime(),
        Size:     info.Size(),
        MimeType: computeMimeType(ext),
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

// computeMimeType detects MIME type from file extension
// Returns MIME type string (e.g., "text/markdown", "application/pdf")
func computeMimeType(ext string) string {
    mimeType := mime.TypeByExtension(ext)
    if mimeType == "" {
        return "application/octet-stream" // Default for unknown types
    }
    return mimeType
}
```

---

## VaultFile

**Purpose:** Data transfer object used by VaultReaderPort to return scanned vault files with metadata and content. Embeds FileMetadata and adds raw file content. Used for vault indexing workflow.

**Architecture Layer:** SPI Adapter (Data Transfer Object)

**Rationale:** VaultFile is a simple DTO that combines filesystem metadata (FileMetadata) with file content. It's the return type for VaultReaderPort.ScanAll/ScanModified, providing VaultIndexer with everything needed to construct Note domain models. Not a domain model - just infrastructure data transfer between port and service.

**Key Attributes:**

- `FileMetadata` (embedded) - All filesystem metadata (Path, Basename, Folder, Ext, ModTime, Size, MimeType)
- `Content` ([]byte) - Raw file content. For MVP: markdown text from .md files. Post-MVP: may be nil for large files (lazy load via VaultReaderPort.Read()).

**Relationships:**

- Returned by VaultReaderPort.ScanAll() and VaultReaderPort.ScanModified()
- Consumed by VaultIndexer to construct Note domain models
- Embeds FileMetadata for filesystem metadata
- Content extracted by FrontmatterService to create Frontmatter

**Design Decisions:**

- **Embeds FileMetadata:** Reuses existing metadata structure (DRY principle). VaultFile = FileMetadata + Content.
- **DTO, not domain model:** Simple data transfer between vault scanning (adapter) and indexing (domain service). No behavior.
- **Content optional (post-MVP):** For large files (PDFs, videos), Content may be nil. VaultIndexer checks MimeType and decides whether to load content.
- **Used only in indexing workflow:** CommandOrchestrator.NewNote uses VaultWriterPort directly, doesn't need VaultFile.

**Helper Functions:**

```go
// NewVaultFile creates VaultFile from FileMetadata and content
// Called by VaultReaderAdapter during vault scanning
func NewVaultFile(metadata FileMetadata, content []byte) VaultFile {
    return VaultFile{
        FileMetadata: metadata,
        Content:      content,
    }
}
```

**Usage Example (VaultIndexer):**

```go
func (v *VaultIndexer) Build(ctx context.Context) (IndexStats, error) {
    // 1. Scan vault
    vaultFiles, err := v.vaultReader.ScanAll(ctx)

    // 2. For each file, construct Note
    for _, vf := range vaultFiles {
        // Filter: only process markdown files for MVP
        if vf.Ext != ".md" {
            continue
        }

        // Extract frontmatter from content
        fm, err := v.frontmatterService.Extract(vf.Content)

        // Validate frontmatter
        v.frontmatterService.Validate(ctx, fm)

        // Derive NoteID from path (adapter translates Path → NoteID)
        noteID := deriveNoteIDFromPath(vf.Path)

        // Construct Note domain model
        note := Note{
            ID:          noteID,
            Frontmatter: fm,
            // Post-MVP: Content: vf.Content
        }

        // Persist to cache
        v.cacheWriter.Persist(ctx, note)
    }
}
```

---

## NoteID

**Purpose:** Abstract domain identifier for notes. Decouples domain logic from infrastructure storage mechanism.

**Architecture Layer:** Domain Core

**Key Attributes:**

- `value` (string) - Opaque identifier. Domain doesn't know if this represents file path, UUID, database key, or URL.

**Relationships:**

- Used by all domain services to reference notes
- Translated by storage adapters (FileSystemReadAdapter/FileSystemWriteAdapter map NoteID ↔ file paths)
- Used as map keys in QueryService indices

**Design Decisions:**

- **Opaque to domain:** Domain never inspects or constructs IDs - adapters handle translation
- **Simple string type:** Minimal overhead, easy to serialize and compare
- **Future-proof:** Can change storage mechanism without changing domain logic

---

## Frontmatter

**Purpose:** Represents note content metadata extracted from YAML frontmatter. Pure data structure with no behavior.

**Architecture Layer:** Domain Core

**Key Attributes:**

- `FileClass` (string, computed) - Schema reference extracted from `fields["fileClass"]`. Used for validation lookup. Empty string if not present.
- `Fields` (map[string]any) - Complete parsed YAML frontmatter as flexible map. Preserves all user-defined fields. Keys are case-sensitive. Supports nested maps.

**Relationships:**

- Extracted from markdown by FrontmatterService.Extract()
- Validated against Schema by FrontmatterService.Validate()
- Composed into Note
- Used by domain services for business logic

**Design Decisions:**

- **Anemic model:** Pure data structure with no behavior. All operations (extraction, validation) implemented in FrontmatterService.
- **FileClass computed:** Extracted from Fields["fileClass"] for convenience
- **Fields as authoritative source:** All frontmatter data stored in Fields map

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

---

## Note

**Purpose:** Core business entity representing a markdown note. Aggregate root combining identity and content metadata.

**Architecture Layer:** Domain Core

**Key Attributes:**

- `ID` (NoteID) - Abstract identifier
- `Frontmatter` (Frontmatter) - Content metadata from YAML

**Relationships:**

- Stored via CacheWriter port
- Retrieved via CacheReader port
- Queried by QueryService (used by template engine's lookup/query functions)
- Created during vault indexing (ID from adapter, Frontmatter from FrontmatterService)

**Design Decisions:**

- **Minimal composition:** Only ID + Frontmatter. No infrastructure metadata (paths, timestamps).
- **Aggregate root:** Note is DDD aggregate root. Frontmatter is value object.
- **Pure data structure:** No behavior or methods. Operations implemented in domain services.
- **Single model for MVP:** Used by both write operations (CacheWriter) and read operations (CacheReader). CQRS separation in operations/ports, not models.
- **Future extensibility:** Post-MVP can introduce schema-specific projections if needed for denormalized query data.

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

**Purpose:** Defines metadata structure with property constraints and inheritance. Governs validation rules for notes of a given `fileClass`. Rich domain model with structural validation behavior.

**Architecture Layer:** Domain Core (Rich Domain Model)

**Key Attributes:**

- `Name` (string) - Schema identifier matching `fileClass` frontmatter value (e.g., "contact", "project", "daily-note")
- `Extends` (string, optional) - Parent schema name for inheritance chains. Can form multi-level chains (e.g., "fleeting-note" extends "base-note" extends "note"). Empty string means no parent.
- `Excludes` ([]string, optional) - Parent property names to exclude from inheritance. Only applicable when Extends is not empty. Enables subtractive inheritance.
- `Properties` ([]Property) - Property definitions for this schema. For inherited schemas, represents delta/override. For root schemas, complete property set.

**Key Methods:**

- `Validate(ctx context.Context) error` - Validates schema structure (Name not empty, Properties valid, Excludes only set when Extends present). Delegates property validation to each Property.Validate(). Returns SchemaError on structural issues.

**Relationships:**

- Schema may extend another Schema (optional inheritance chains)
- Schema contains multiple Property definitions
- Frontmatter validated against resolved Schema by FrontmatterService
- Loaded from JSON files by SchemaLoader adapter
- Inheritance resolved by SchemaResolver service
- Structural validation via Schema.Validate() called by SchemaValidator

**Design Decisions:**

- **Rich domain model:** Contains structural validation behavior via Validate() method. No external dependencies - pure domain logic checking structure.
- **Inheritance in source form:** Schema stores original Extends/Excludes/Properties from JSON. SchemaResolver service resolves inheritance and provides flattened properties to FrontmatterService.
- **Properties vs Fields terminology:** Schema has "Properties" (validation rules). Frontmatter has "Fields" (actual data).
- **Excludes dependent on Extends:** Excludes only meaningful when Extends is not empty.

- **String-based Extends reference:** Uses schema name string, not Go pointer, to avoid circular dependency issues in struct definitions. Schema registry (map[string]\*Schema) resolves references after all schemas loaded.

- **Eager resolution at startup:** Inheritance chains resolved during application initialization (fail-fast on circular dependencies per Epic 2, Story 2.6). Validator never sees unresolved schemas. Performance: O(n\*d) where n=schemas, d=depth, acceptable for MVP (<100 schemas expected).

- **Resolution order:** (1) Load all schema files, (2) Build dependency graph, (3) Detect cycles, (4) Resolve in topological order (leaves first), (5) For each schema: get parent's ResolvedProperties → apply Excludes → merge/override with child Properties → store in ResolvedProperties.

- **Property override semantics:** If child Property.Name matches parent Property.Name, child completely replaces parent (not merging property attributes). This is explicit override, not attribute-level merge.

**Additional Information:**

Schema inheritance provides powerful reusability for similar note types. For example, a base "note" schema could define common properties (title, tags, created), while specialized schemas like "meeting-note" or "person" extend the base and add domain-specific properties. The eager resolution strategy ensures validation is fast (no runtime resolution overhead) at the cost of slightly longer startup time. For MVP with <100 schemas, this tradeoff is acceptable. The Builder pattern isolates complexity—domain validators simply receive fully-resolved schemas and don't need to understand inheritance mechanics.

> **Adapter boundary reminder:** Schema definitions are serialized as JSON on disk, but decoding and discriminator handling occur in the SchemaLoader adapter (see Epic 2, Story 2.4). The domain models described here stay infrastructure-free and are instantiated via constructors that enforce the rules above.

---

## PropertyBank

**Purpose:** Singleton registry of reusable, pre-configured Property definitions that schemas can reference via `$ref`. Reduces duplication across schema definitions, ensures consistency for common properties (e.g., `standard_title`, `standard_tags`), and enables centralized property definition management.

**Architecture Layer:** Domain Core (Singleton)

**Rationale:** PropertyBank is pure domain concern—it's a singleton registry of business rules (property constraints) that can be reused. No infrastructure dependencies. Loaded once at startup by SchemaLoader adapter from single JSON file, but the model itself represents domain knowledge about common property patterns.

**Key Attributes:**

- `Properties` (map[string]Property) - Named property definitions keyed by unique identifier (e.g., "standard_title", "iso_date", "email_address"). Loaded from single property bank JSON file at startup.

**Relationships:**

- PropertyBank loaded before Schema definitions during startup (SchemaLoader orchestrates)
- Schema.Properties can reference PropertyBank entries via `$ref` syntax (resolved during schema loading by SchemaLoader)
- Property definitions in PropertyBank are templates—simple substitution for MVP (no attribute-level overrides)

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

Property bank definitions stored in single file `schemas/property_bank.json` (configurable via Config.PropertyBankPath):

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

- **Singleton pattern:** Only one PropertyBank instance exists per application lifecycle. Loaded once at startup from single JSON file (default: `schemas/property_bank.json`, configurable via Config.PropertyBankPath).

- **Properties vs Fields terminology:** PropertyBank contains "Properties" (reusable validation rule definitions), not "Fields" (actual data). Consistent with Schema.Properties terminology.

- **JSON format:** Simpler unmarshaling than YAML. Frontmatter remains YAML (Obsidian convention), but schema definitions prioritize Go stdlib integration.

- **$ref resolution format:** Schemas reference properties using JSON pointer syntax: `{"$ref": "#/properties/{property-name}"}`. SchemaLoader resolves references at load time by looking up PropertyBank.Properties map.

- **Simple substitution (MVP):** Referenced property completely replaces `$ref` object. No attribute-level merging or overrides. Post-MVP could support inline overrides:

  ```json
  {
    "$ref": "#/properties/standard_title",
    "required": false // Override: make title optional for this schema
  }
  ```

- **Load order:** PropertyBank loaded before schemas during SchemaLoader.LoadSchemas() call. Ensures all `$ref` references can be resolved. Missing references cause schema loading to fail at startup (fail-fast).

- **Flat structure:** Properties cannot reference other properties (no nested `$ref` in PropertyBank itself). Post-MVP could add property composition if needed.

**Implementation Notes:**

SchemaLoader adapter implements property bank loading and `$ref` resolution (~30 LOC):

1. Construct property bank path from Config: `filepath.Join(config.SchemasDir, config.PropertyBankFile)` (default: `schemas/property_bank.json`)
2. Load single property bank JSON file from constructed path
3. Parse into PropertyBank structure with Properties map
4. During schema parsing, detect `$ref` attributes in property definitions
5. Look up referenced property in PropertyBank.Properties map by key
6. Substitute `$ref` object with referenced property definition
7. Continue with normal schema validation
8. Fail at startup if `$ref` references non-existent property (fail-fast)

**Additional Information:**

PropertyBank solves the "common property definition" problem elegantly. Without it, every schema must redefine standard properties like `title`, `tags`, `created`, `modified`—leading to inconsistencies (different patterns, required settings) and maintainability burden. With PropertyBank, define once, reference everywhere. The JSON format choice aligns with Go's excellent stdlib JSON support while keeping frontmatter in YAML (user-facing, Obsidian standard). The `$ref` syntax follows JSON Schema conventions, making it familiar to users with schema experience. Post-MVP could enhance with property inheritance, attribute-level overrides, or validation rules, but simple reference substitution covers 80% of reuse needs.

---

## Property

**Purpose:** Defines a single metadata field with validation constraints. Building block of Schema definitions. Rich domain model with structural validation behavior.

**Architecture Layer:** Domain Core (Rich Domain Model)

**Key Attributes:**

- `Name` (string) - Property identifier matching frontmatter key. Case-sensitive.
- `Required` (bool) - Whether property must be present. Empty array satisfies required for array properties.
- `Array` (bool) - Whether property accepts multiple values (YAML list) vs single scalar value.
- `Spec` (PropertySpec) - Type-specific validation constraints (interface for polymorphism).

**Key Methods:**

- `Validate(ctx context.Context) error` - Validates property structure (Name not empty, Spec not nil). Delegates PropertySpec validation to Spec.Validate(). Returns error on structural issues.

**Relationships:**

- Belongs to Schema (composition)
- Contains one PropertySpec implementation
- May be sourced from PropertyBank via `$ref` (resolved by SchemaResolver)
- Used by FrontmatterService to validate Frontmatter.Fields
- Structural validation via Property.Validate() called by Schema.Validate()

**Design Decisions:**

- **Rich domain model:** Contains structural validation behavior via Validate() method. Delegates to PropertySpec.Validate() for polymorphic validation.
- **Interface-based composition:** PropertySpec interface enables type-specific validation without nullable attributes.
- **Properties vs Fields terminology:** "Property" = schema definition. "Field" = actual frontmatter data.
- **Required vs Array orthogonal:** Can have required scalars, optional scalars, required arrays, or optional arrays.

---

## PropertySpec (Type-Specific Configurations)

**Purpose:** Interface for type-specific validation constraint definitions. Defines what constraints apply to a property (min/max, patterns, enums) as immutable value objects with structural validation behavior. Each PropertySpec variant validates its own constraint structure.

**Architecture Layer:** Domain Core (Value Objects with Behavior)

**Rationale:** PropertySpec variants are DDD value objects—immutable constraint definitions identified by their attributes, not by identity. They define constraint data (e.g., "min: 0, max: 100") AND validate constraint structure (e.g., regex pattern is valid). This leverages polymorphism—each PropertySpec type knows how to validate its own constraints.

**Key Methods (Interface):**

- `Type() PropertyType` - Returns property type identifier (string, number, date, file, boolean)
- `Validate(ctx context.Context) error` - Validates constraint structure (e.g., pattern is valid regex, min <= max, enum not empty). Pure structural validation with no external dependencies.

**Relationships:**

- Exactly one PropertySpec variant per Property (composition via interface)
- Used by FrontmatterService to validate Frontmatter.Fields against constraints
- FileSpec uses FileClass/Directory attributes for dynamic lookups against vault index
- Structural validation via PropertySpec.Validate() called by Property.Validate()

**Design Decisions:**

- **Value objects with behavior:** PropertySpec variants are immutable value objects that validate their own structural integrity. Two StringSpecs with identical Enum/Pattern are equivalent.

- **Polymorphic validation:** Each PropertySpec variant implements Validate() for type-specific structural checks. Avoids type switches in validator service.

- **Interface-based polymorphism:** PropertySpec interface enables type-safe composition. Property contains one PropertySpec variant without nullable attributes or type switches.

- **Nil pointer semantics:** For optional attributes, nil pointer means "no constraint." Empty value has different meaning (e.g., empty Enum list = no values allowed, nil Enum = any value allowed).

---

### StringSpec

**Purpose:** Defines string validation constraints (allowed values, patterns) as immutable value object with structural validation.

**Key Attributes:**

- `Enum` ([]string, optional) - Allowed values as fixed list. If non-empty, value must be in list (exact match, case-sensitive). Empty list means no values allowed, nil means any string valid.
- `Pattern` (string, optional) - Regex pattern for custom validation. If non-empty, value must match pattern. Uses Go `regexp` package. Empty string or nil means no pattern constraint.

**Key Methods:**

- `Type() PropertyType` - Returns `PropertyTypeString`
- `Validate(ctx context.Context) error` - Validates Pattern is valid regex if specified. Returns error if pattern compilation fails.

**Validation Implementation Example:**

```go
func (s StringSpec) Validate(ctx context.Context) error {
    if s.Pattern != "" {
        if _, err := regexp.Compile(s.Pattern); err != nil {
            return fmt.Errorf("invalid pattern regex: %w", err)
        }
    }
    // Enum doesn't need validation - any string list is valid
    return nil
}
```

**Design Decisions:**

- **Enum and Pattern can coexist:** Both constraints can be specified. FrontmatterService checks enum first (if present), then pattern (if present). Value must satisfy both (AND logic).
- **Case-sensitive enum:** Exact string matching. User must include all case variations in enum if case-insensitive behavior desired.
- **Pattern validation at load time:** Validate() ensures regex compiles at schema load time, not at frontmatter validation time.

**Example:**

```json
{
  "enum": ["red", "green", "blue"]
}
```

or

```json
{
  "pattern": "^[A-Z][a-z]+$"
}
```

---

### NumberSpec

**Purpose:** Defines numeric validation constraints (min/max bounds, step increments) as immutable value object with structural validation.

**Key Attributes:**

- `Min` (\*float64, optional) - Minimum allowed value (inclusive). Nullable pointer distinguishes "not set" from "0". If set, value must be >= Min.
- `Max` (\*float64, optional) - Maximum allowed value (inclusive). If set, value must be <= Max.
- `Step` (\*float64, optional) - Increment/decrement amount. If 1.0, implies integer values. If 0.1, implies one decimal precision. If nil, any precision allowed.

**Key Methods:**

- `Type() PropertyType` - Returns `PropertyTypeNumber`
- `Validate(ctx context.Context) error` - Validates Min <= Max if both specified, Step > 0 if specified. Returns error on invalid constraints.

**Validation Implementation Example:**

```go
func (n NumberSpec) Validate(ctx context.Context) error {
    if n.Min != nil && n.Max != nil && *n.Min > *n.Max {
        return fmt.Errorf("min (%f) cannot be greater than max (%f)", *n.Min, *n.Max)
    }
    if n.Step != nil && *n.Step <= 0 {
        return fmt.Errorf("step must be positive, got %f", *n.Step)
    }
    return nil
}
```

**Design Decisions:**

- **Unified number type:** Handles both integer and float via `Step` attribute. Simplifies type system and aligns with YAML's lack of int/float distinction.
- **Step-based integer semantics:** If Step=1.0, FrontmatterService checks `value == math.Floor(value)`. This is semantic check (not type check), aligning with YAML treating `42` and `42.0` identically.
- **All numbers as float64:** YAML unmarshals numbers as float64. FrontmatterService validates as float64, uses Step to determine if fractional part allowed.
- **Constraint validation at load time:** Validate() ensures min/max/step are coherent at schema load time.

**Example:**

```json
{
  "min": 0,
  "max": 100,
  "step": 1
}
```

(integer 0-100)

---

### DateSpec

**Purpose:** Defines date/time format constraints as immutable value object with structural validation.

**Key Attributes:**

- `Format` (string) - Go time layout string (e.g., "2006-01-02", "2006-01-02T15:04:05Z07:00"). Uses Go stdlib `time.Parse(format, value)`. If empty, defaults to RFC3339.

**Key Methods:**

- `Type() PropertyType` - Returns `PropertyTypeDate`
- `Validate(ctx context.Context) error` - Validates Format is valid Go time layout by attempting to parse reference time. Returns error if format invalid.

**Validation Implementation Example:**

```go
func (d DateSpec) Validate(ctx context.Context) error {
    if d.Format == "" {
        return nil // Empty format defaults to RFC3339, always valid
    }
    // Test format by parsing reference time
    referenceTime := "Mon Jan 2 15:04:05 MST 2006"
    if _, err := time.Parse(d.Format, referenceTime); err != nil {
        return fmt.Errorf("invalid time format: %w", err)
    }
    return nil
}
```

**Design Decisions:**

- **Go time layout format:** Uses Go's reference time format (Jan 2 15:04:05 2006 MST). Enables flexible date/time parsing with stdlib.
- **Default RFC3339:** If Format empty or nil, FrontmatterService uses RFC3339 (ISO 8601 compatible).
- **Format validation at load time:** Validate() ensures format string is valid at schema load time.

**Example:**

```json
{
  "format": "2006-01-02"
}
```

(ISO date: YYYY-MM-DD)

---

### FileSpec

**Purpose:** Defines file reference validation constraints (fileClass filters, directory filters) as immutable value object with structural validation.

**Key Attributes:**

- `FileClass` (string, optional) - Restricts valid file references to notes with specific fileClass value or regex pattern. Supports negation via `^` prefix. Examples: `"project"` (exact match), `"^archive"` (NOT archive), `"(project|task)"` (regex: project OR task). Empty string or nil means no fileClass restriction.
- `Directory` (string, optional) - Restricts valid file references to notes within specific vault directory path. Path is relative to vault root. Supports negation via `^` prefix. Examples: `"projects/"` (notes in projects/), `"^archive/"` (NOT in archive/), `"work/.*"` (regex: anything under work/). Empty string or nil means no directory restriction.

**Key Methods:**

- `Type() PropertyType` - Returns `PropertyTypeFile`
- `Validate(ctx context.Context) error` - Validates FileClass and Directory patterns are valid regex if they contain regex syntax. Returns error if patterns invalid.

**Validation Implementation Example:**

```go
func (f FileSpec) Validate(ctx context.Context) error {
    // Validate FileClass regex if present
    if f.FileClass != "" {
        pattern := strings.TrimPrefix(f.FileClass, "^") // Remove negation prefix
        if _, err := regexp.Compile(pattern); err != nil {
            return fmt.Errorf("invalid fileClass pattern: %w", err)
        }
    }
    // Validate Directory regex if present
    if f.Directory != "" {
        pattern := strings.TrimPrefix(f.Directory, "^") // Remove negation prefix
        if _, err := regexp.Compile(pattern); err != nil {
            return fmt.Errorf("invalid directory pattern: %w", err)
        }
    }
    return nil
}
```

**Design Decisions:**

- **Filter conjunction (AND logic):** When both FileClass and Directory set, both conditions must be satisfied. Example: `{"fileClass": "project", "directory": "work/"}` matches project notes in work/ directory only.
- **Negation support:** `^` prefix inverts the match. Enables exclusion patterns (e.g., "any note except archives").
- **Regex patterns:** FileClass and Directory support regex for flexible matching. FrontmatterService uses Go `regexp` package.
- **Pattern validation at load time:** Validate() ensures regex patterns compile at schema load time.
- **Flattened attributes (MVP):** FileClass and Directory are direct attributes for MVP simplicity. Post-MVP could introduce nested Filter struct with additional filter types (Tags, ModTime, etc.).
- **Vault index dependency:** FrontmatterService validates that referenced file exists in vault index (loaded via CacheReader) and matches constraints. Requires indexed vault.

**Example:**

```json
{
  "fileClass": "project",
  "directory": "work/"
}
```

---

### BoolSpec

**Purpose:** Defines boolean validation (no additional constraints). Marker value object with no structural validation needed.

**Key Attributes:**

- None. Presence of BoolSpec indicates property accepts boolean values only.

**Key Methods:**

- `Type() PropertyType` - Returns `PropertyTypeBool`
- `Validate(ctx context.Context) error` - Always returns nil. No constraints to validate.

**Validation Implementation Example:**

```go
func (b BoolSpec) Validate(ctx context.Context) error {
    return nil // No constraints to validate for boolean type
}
```

**Design Decisions:**

- **Type check only:** FrontmatterService validates that value is Go bool type (true/false). No additional constraints possible.
- **Marker value object:** Empty struct. Presence in Property.Spec indicates boolean type.
- **No-op validation:** Validate() always succeeds since there are no constraints to check.

---


## TemplateID

**Purpose:** Template name used for identification and composition. Represents the intrinsic domain concept of "template name" required by Go's `text/template` composition system.

**Architecture Layer:** Domain Core

**Key Attributes:**

- `value` (string) - Template name. Typically basename of template file without extension (e.g., "contact-header", "daily-note"). Used in template composition syntax: `{{template "contact-header"}}`.

**Relationships:**

- Used by TemplateEngine for template composition via Go `text/template` package
- Used in template references: `{{template "name"}}` and `{{block "name"}}`
- TemplateLoader adapter derives TemplateID from filename basename (scans Config.TemplatesDir, default: `templates/`)
- TemplateLoader uses FileMetadata (SPI adapter) to map TemplateID ↔ file paths
- Used as map keys in template registries

**Design Decisions:**

- **Name as domain concept:** Unlike NoteID (truly opaque), TemplateID represents template name—an intrinsic domain requirement for Go's `text/template` composition system. Not a layer violation.
- **Basename convention:** By convention, TemplateID matches file basename (without path/extension). Adapter derives this during loading from `templates/contact-header.md` → `"contact-header"`.
- **Storage agnostic within constraint:** Templates could come from database, API, or filesystem, but all need a name for `{{template}}` references. Basename is pragmatic choice.
- **Simple identifier type:** Just a string wrapper, not a DDD value object. Primitive identifier with no complex structure.

---

## Template

**Purpose:** Represents an executable template for note generation. Pure data structure containing template identity and content. Single model used across all layers following YAGNI principle for MVP.

**Architecture Layer:** Domain Core

**Rationale:** Template is a pure data structure—just identity (TemplateID) and content (raw template text). All execution logic (parsing, rendering, function execution) lives in TemplateEngine service. Pure domain entity with no infrastructure dependencies.

**Key Attributes:**

- `ID` (TemplateID) - Template name for identification and composition. Used in `{{template "name"}}` references.
- `Content` (string) - Raw template text. Contains Go `text/template` syntax with Lithos-specific function calls (`prompt`, `suggester`, `lookup`, `query`, `now`, etc.).

**Relationships:**

- Template may reference other Templates via `{{template "name"}}` directive (resolved by Go's `text/template` engine)
- Loaded by TemplateLoader adapter from files in Config.TemplatesDir (default: `templates/`)
- TemplateLoader uses FileMetadata to track filesystem paths (TemplateID ↔ Path mapping)
- Executed by TemplateEngine service with injected function map and data context
- Template functions access ports (CacheReader, InteractivePort, FileReader) via closure during rendering

**Design Decisions:**

- **Pure data structure:** Just ID + Content. No behavior, no cached parse trees (Parsed field removed). All execution logic in TemplateEngine service.

- **Single model for MVP (YAGNI):** One Template definition used for both discovery and execution eliminates translation overhead. Can split into separate models post-MVP if lazy loading becomes critical.

- **TemplateID as name (domain concept):** ID represents template name—intrinsic requirement for Go's `text/template` composition. Not a layer violation because naming is core to template composition logic.

- **Content loaded by adapters:** TemplateLoader adapter reads file content from Config.TemplatesDir and populates Template model. Domain receives ready-to-use templates.

- **FileMetadata for path mapping:** TemplateLoader uses general-purpose FileMetadata (SPI adapter) to map TemplateID ↔ filesystem paths. No need for TemplateMetadata—FileMetadata is reusable infrastructure model.

- **Composition via Go stdlib:** Template composition handled by Go's `text/template` via `{{define}}`/`{{template}}` directives. No custom section tracking needed—leverage mature, well-tested functionality.

**Additional Information:**

The lean Template model (ID + Content only) follows clean architecture principles—pure data structure with behavior in services. TemplateID represents an intrinsic domain concept (template name for composition), not infrastructure leakage. The distinction from Note/NoteID is important: NoteID is truly opaque (domain doesn't care about note naming), while TemplateID must be meaningful because Go's `text/template` requires names for composition (`{{template "contact-header"}}`). For MVP with small template directories (<100 templates), single model for both discovery and execution is pragmatic. FileMetadata handles filesystem concerns in adapter layer, keeping Template pure domain.

---

## Config

**Purpose:** Application configuration loaded from `lithos.json` and environment variables. Defines vault structure and operational settings. Immutable value object representing application configuration state.

**Architecture Layer:** Domain Core (Value Object)

**Rationale:** Config is a DDD value object—immutable configuration data identified by its attributes. While loaded by ConfigLoader adapter, the Config model itself represents domain knowledge about vault structure (where templates, schemas, property bank live). Domain services receive Config via dependency injection to locate resources.

**Key Attributes:**

- `VaultPath` (string) - Root directory of vault. Default: current working directory. All relative paths in config are resolved relative to this. Must exist and be readable. ConfigLoader searches upward from current directory to find `lithos.json`, then uses that directory as VaultPath.
- `TemplatesDir` (string) - Path to templates directory. Default: `{VaultPath}/templates/`. Can be absolute or relative to VaultPath. Must exist for `lithos new` and `lithos find` commands. TemplateLoader scans all `.md` files in this directory.
- `SchemasDir` (string) - Path to schemas directory. Default: `{VaultPath}/schemas/`. Can be absolute or relative to VaultPath. Must exist if schemas are used. SchemaLoader parses all schema JSON files in this directory at startup.
- `PropertyBankFile` (string) - Filename of property bank file within SchemasDir. Default: `property_bank.json`. Full path is `{SchemasDir}/{PropertyBankFile}`. Optional—if missing, schemas cannot use `$ref` references.
- `CacheDir` (string) - Path to index cache directory. Default: `{VaultPath}/.lithos/cache/`. Can be absolute or relative to VaultPath. Created automatically if missing. Must be writable. JSONFileCacheAdapter stores one `.json` file per indexed note.
- `LogLevel` (string) - Logging verbosity for zerolog. One of: "debug", "info", "warn", "error". Default: "info". Case-insensitive. Invalid values fall back to "info" with warning. Controls stdout/stderr output verbosity.

**Relationships:**

- Used by all adapters for initialization and runtime configuration
- Loaded at startup via ConfigLoader adapter (reads `lithos.json`, environment variables, flags in that precedence order)
- Passed to components via constructor injection (dependency injection pattern)
- PropertyBankFile used by SchemaLoader to locate property bank within SchemasDir

**Design Decisions:**

- **Value object (DDD):** Immutable configuration data identified by its attributes. Two Config instances with identical values are equivalent. Loaded once at startup, never modified.

- **JSON format for MVP:** Config file is `lithos.json` for MVP. Post-MVP: expand to support TOML and YAML formats for user preference.

- **Flat structure:** No nested config objects for MVP simplicity. All settings are top-level keys in JSON. This keeps config file simple and reduces unmarshaling complexity.

- **Sensible defaults:** Empty config file is valid - all paths default to sensible vault-relative locations. Enables quickstart: user can run `lithos index` with zero configuration if vault uses standard directory structure.

- **String paths:** Paths stored as strings, not file handles or custom Path types. Adapters resolve paths on demand using `filepath.Join` and `filepath.Abs`. This keeps config serializable and adapter-agnostic.

- **PropertyBankFile is filename only:** Not a full path. Always located in SchemasDir. SchemaLoader constructs full path: `filepath.Join(config.SchemasDir, config.PropertyBankFile)`.

- **Validation at load time:** ConfigLoader validates that VaultPath exists, is directory, and is readable. Other paths validated lazily when accessed (TemplatesDir validated on first `lithos find`, not at config load).

- **Environment variable override:** ConfigLoader supports env vars like `LITHOS_VAULT_PATH`. Precedence: CLI flags > env vars > config file > defaults. This enables CI/CD override without modifying config files.

- **No secrets in config:** Config is committed to git (per PRD, vaults are git repositories). No API keys, tokens, or passwords. Future: if external API integrations added, use separate credential files or system keychain.

**Additional Information:**

Config is a domain value object representing application configuration state. While loaded by infrastructure adapter (ConfigLoader), the model itself represents domain knowledge about vault structure and resource locations. The flat structure keeps configuration simple and readable for users. Sensible defaults mean a user can create an empty `lithos.json` and the application works immediately if using standard directory conventions. The precedence order (CLI flags > env vars > config file > defaults) provides flexibility for different environments - developers can override locally via flags, CI/CD can inject via environment variables, and teams can share baseline config in version control. String-based paths keep Config serializable and platform-agnostic - no special types needed. For MVP, JSON format provides simplicity with excellent Go stdlib support. Post-MVP expansion to TOML/YAML gives users format choice.

---

## Data Model Relationships Diagram

**Legend:**

- 🔵 Domain Core (Entities/Aggregates)
- 🔷 Domain Core (Value Objects)
- 🟢 SPI Adapter models
- ├─> Composition/contains
- └─> Reference/uses

```
═══════════════════════════════════════════════════════════════
[Domain Core Layer - Value Objects]
═══════════════════════════════════════════════════════════════

Config 🔷 (immutable configuration)
  ├─> VaultPath: string
  ├─> TemplatesDir: string (default: "templates/")
  ├─> SchemasDir: string (default: "schemas/")
  ├─> PropertyBankFile: string (default: "property_bank.json")
  ├─> CacheDir: string (default: ".lithos/cache/")
  └─> LogLevel: string

NoteID 🔵 (simple identifier)
  └─> value: string (opaque)

TemplateID 🔵 (simple identifier)
  └─> value: string (template name/basename)

PropertySpec 🔷 (interface for polymorphic value objects)
  ├─> StringSpec
  │     ├─> Enum: []string
  │     └─> Pattern: string
  ├─> NumberSpec
  │     ├─> Min: *float64
  │     ├─> Max: *float64
  │     └─> Step: *float64
  ├─> DateSpec
  │     └─> Format: string
  ├─> FileSpec
  │     ├─> FileClass: string
  │     └─> Directory: string
  └─> BoolSpec
        └─> (no attributes)

═══════════════════════════════════════════════════════════════
[Domain Core Layer - Entities & Aggregates]
═══════════════════════════════════════════════════════════════

Note 🔵 (Aggregate Root)
  ├─> ID: NoteID
  └─> Frontmatter
        ├─> FileClass: string (computed from Fields["fileClass"])
        └─> Fields: map[string]any

Template 🔵 (Entity)
  ├─> ID: TemplateID
  └─> Content: string

Schema 🔵 (Entity)
  ├─> Name: string
  ├─> Extends: string (optional, references another Schema)
  ├─> Excludes: []string
  └─> Properties: []Property
        └─> each Property:
              ├─> Name: string
              ├─> Required: bool
              ├─> Array: bool
              └─> Spec: PropertySpec (one variant)

PropertyBank 🔵 (Singleton)
  └─> Properties: map[string]Property (referenced via $ref)

═══════════════════════════════════════════════════════════════
[SPI Adapter Layer]
═══════════════════════════════════════════════════════════════

FileMetadata 🟢 (infrastructure - maps domain IDs to filesystem)
  ├─> Path: string (absolute filesystem path)
  ├─> Basename: string (computed)
  ├─> Folder: string (computed)
  └─> ModTime: time.Time

═══════════════════════════════════════════════════════════════
Cross-Model Relationships:
═══════════════════════════════════════════════════════════════

Schema → Schema
  └─> Inheritance via Extends (resolved by SchemaLoader adapter)

Property → PropertyBank
  └─> References via $ref (resolved by SchemaLoader adapter)

Property → PropertySpec
  └─> Contains one PropertySpec variant (polymorphism)

FileSpec → Note
  └─> FileClass/Directory filter references vault index of Notes

Frontmatter → Schema
  └─> Validated by FrontmatterService using Schema lookup via FileClass

NoteID ↔ FileMetadata (adapter layer)
  └─> FileSystemReadAdapter/FileSystemWriteAdapter map NoteID to Path

TemplateID ↔ FileMetadata (adapter layer)
  └─> TemplateLoader maps TemplateID to Path (reuses FileMetadata)

Config → PropertyBank
  └─> PropertyBankFile + SchemasDir = full path to property bank file

Config → TemplateLoader
  └─> TemplatesDir = directory to scan for template files

═══════════════════════════════════════════════════════════════
Key Architecture Principles:
═══════════════════════════════════════════════════════════════

✓ Abstract identifiers (NoteID, TemplateID) decouple domain from storage
✓ FileMetadata is SPI adapter - domain never sees filesystem paths
✓ PropertySpec variants are value objects - immutable constraints
✓ Config is value object - immutable, loaded once at startup
✓ Single Note model for MVP (CQRS in operations/ports, not models)
✓ PropertyBank is singleton - one instance per application lifecycle
✓ TemplateID = template name (intrinsic to Go text/template, not layer violation)
✓ All domain models are pure data - behavior in services (FrontmatterService, TemplateEngine)
✓ FileMetadata reused for both notes and templates (DRY principle)
```
