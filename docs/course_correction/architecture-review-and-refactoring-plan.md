# Architecture Review and Refactoring Plan

## Document Purpose

This document captures the comprehensive architectural review conducted on 2025-10-24, identifying weaknesses in the original hexagonal architecture implementation and documenting the refactoring plan to address these issues. This serves as the authoritative reference for ongoing architecture updates.

---

## Executive Summary

**Original Issue:** The architecture documentation contained layer violations, mixed concerns, and terminology misuse that would lead to implementation problems.

**Key Problems Identified:**
1. Layer violations: File and Template models coupled domain to filesystem
2. CQRS terminology misused - had separated I/O operations, not true CQRS
3. Inconsistent ISP application across ports
4. CommandOrchestrator redundant facade with no added value
5. VaultIndexer God Service violating SRP
6. Result[T] pattern unnecessary in Go (idiomatic `(T, error)` preferred)
7. Validation concerns mixed with domain models

**Resolution Approach:** Incremental refactoring of architecture documentation following template structure, maintaining context and rationale while fixing violations.

---

## Critical Architectural Violations Identified

### 1. Layer Violation: File Model (CRITICAL)

**Location:** `docs/architecture/data-models.md:13-77`

**Original Classification:** "Domain Core (with SPI Adapter concerns)" - contradictory!

**Problem:**
```go
// WRONG - Domain coupled to filesystem
type File struct {
    Path     string      // Filesystem path - infrastructure!
    Basename string      // Computed from path
    Folder   string      // Computed from path
    ModTime  time.Time   // From os.Stat() - infrastructure!
}
```

**Why This Violates Hexagonal Architecture:**
- Domain references notes by filesystem paths (infrastructure detail)
- File paths are implementation details - could be database IDs, URLs, content hashes
- `ModTime` from `os.Stat()` directly couples domain to filesystem implementation
- Domain cannot change storage mechanism without model changes

**User Clarification:**
> "I agree with your reasoning regarding layer violation in the File and Template models"

**Correct Approach:**
```go
// Domain Core - Abstract identifier
type NoteID string

// SPI Adapter - Infrastructure metadata
type FileMetadata struct {
    Path     string
    Basename string
    Folder   string
    ModTime  time.Time
}

// Adapter translates NoteID ↔ Path
func (a *FileSystemAdapter) Store(note Note) error {
    meta := a.idToMetadata[note.ID]
    path := meta.Path  // Adapter handles translation
    // ...
}
```

**Resolution:**
- Create `NoteID` as abstract domain identifier
- Rename `File` to `FileMetadata` and move to SPI Adapter layer
- Adapters translate between `NoteID` and infrastructure identifiers

---

### 2. Layer Violation: Template Model (CRITICAL)

**Location:** `docs/architecture/data-models.md:476-508`

**Problem:**
```go
// WRONG - Domain coupled to filesystem
type Template struct {
    FilePath string  // Filesystem path - infrastructure!
    Name     string
    Content  string
}
```

**Original Flawed Justification:**
> "FilePath as domain identifier: FilePath is a domain concept - domain needs to identify and reference templates. Using file paths as identifiers is a valid domain choice, not adapter leakage."

**Why This Reasoning is WRONG:**
- Just because domain needs identifiers doesn't mean filesystem paths should BE those identifiers
- This couples domain to filesystem storage forever
- Cannot move templates to database/API without changing domain model
- Violates Dependency Inversion Principle

**User Agreement:**
> "I agree with your reasoning regarding layer violation in the File and Template models"

**Correct Approach:**
```go
// Domain Core - Abstract identifier
type TemplateID string

// Domain Core - Pure business data
type Template struct {
    ID      TemplateID
    Name    string
    Content string
}

// SPI Adapter - Infrastructure metadata
type TemplateMetadata struct {
    Path    string
    ModTime time.Time
}
```

---

### 3. CQRS Misapplication (SIGNIFICANT)

**Location:** `docs/architecture/high-level-architecture.md:42-43`

**Original Claim:**
> "CQRS Applied to Storage Layer Only... Commands write data, queries read data... use single unified models (Note, Schema, Template) - not separate read/write models"

**Problem:** This is NOT CQRS! This is just separated read/write methods (CacheCommandPort vs CacheQueryPort).

**True CQRS Separates:**
- **Command Model** - optimized for writes (validation, business rules)
- **Query Model** - optimized for reads (denormalized, indexed)
- **Synchronization** - keeps models consistent

**User Agreement on True CQRS:**
> "proper CQRS should be implemented to properly isolate and modularize separate concerns"

**Initial CQRS Design (Later Refined):**
```go
// Write Model
type NoteCommand struct {
    Identity    NoteIdentity
    Metadata    NoteMetadata
    Frontmatter Frontmatter
    Content     string
    SchemaName  string
    ValidationRules []ValidationRule
}

// Read Model
type NoteProjection struct {
    ID        NoteIdentity
    FileClass string
    Fields    map[string]any
    Basename  string  // Denormalized
    Folder    string  // Denormalized
    Tags      []string // Extracted
}
```

**Critical User Insight - Basename/Folder Are Infrastructure!**

User challenged:
> "we said Basename and Folder are infrastructure concerns, so how can NoteProjection contain them while remaining in the domain core?"

**This exposed a fundamental flaw:** If Basename/Folder are infrastructure (computed from filesystem paths), they can't be in domain models!

**User's Key Realization:**
> "your suggestion makes Note and NoteProjection have the same attributes, in which case NoteProjection would be redundant"

**Final Decision - Single Note Model for MVP:**
```go
// Domain Core - Single model
type Note struct {
    ID          NoteID
    Frontmatter Frontmatter
}

// CQRS in operations/ports, not models
type CacheWriter interface {
    Store(ctx context.Context, note Note) error
}

type CacheReader interface {
    Get(ctx context.Context, id NoteID) (Note, error)
    Filter(ctx context.Context, query Query) ([]Note, error)
}

// Query indices built by QueryService (infrastructure)
type QueryService struct {
    byBasename  map[string]NoteID
    byFolder    map[string][]NoteID
    byFileClass map[string][]NoteID
}
```

**User's Future Vision:**
> "I think this is the right direction for the MVP and in the future the different Schema models could be used to assist in structuring the read models"

**Rationale:**
- For MVP: Single Note model, CQRS only in operations/ports
- Post-MVP: Schema-specific projections for business-level denormalizations (e.g., Project schema with computed task counts, completion percentages)
- Query indices (Basename, Folder) are service-level infrastructure, not model-level

---

### 4. ISP Violations - Inconsistent Port Granularity (SIGNIFICANT)

**Location:** `docs/architecture/components.md:146-242`

**Problem:** Inconsistent application of Interface Segregation Principle across ports.

**Evidence:**

**Good ISP (Separated):**
```go
type PromptPort interface {
    Prompt(cfg PromptConfig) (string, error)
    // ... 5 methods for prompts
}

type FuzzyFinderPort interface {
    Find(items []TemplateMetadata) (TemplateMetadata, error)
    // ... 2 methods for fuzzy finding
}
```

**Bad ISP (Bundled):**
```go
// WRONG - Bundles read/write/walk
type FileSystemPort interface {
    ReadFile(path string) ([]byte, error)      // Read concern
    WriteFileAtomic(path string, data []byte) error  // Write concern
    Walk(root string, fn WalkFunc) error       // Walk concern
}

// WRONG - Bundles query and command
type ConfigPort interface {
    Config() Config                           // Query
    Reload(ctx context.Context) (Config, error)  // Command
}
```

**User Agreement:**
> "I agree the FileSystemPort and ConfigPort violate ISP and must be modified"

**Correct Approach:**
```go
// Separate read/write/walk concerns
type FileReader interface {
    Read(ctx context.Context, path string) ([]byte, error)
}

type FileWriter interface {
    Write(ctx context.Context, path string, data []byte) error
}

type FileWalker interface {
    Walk(ctx context.Context, root string, fn WalkFunc) error
}

// Services depend only on what they need
type TemplateEngine struct {
    templates TemplateLoader  // Only needs read, not write
    storage   StorageReader   // Only needs read, not write
    prompter  Prompter
}
```

**Impact:** Components that only need read access still must depend on write methods, violating "depend only on what you need."

---

### 5. Config Misclassification (SIGNIFICANT)

**Location:** `docs/architecture/data-models.md:512-546`

**Original Classification:** "SPI Adapter (Configuration)"

**Problem:** Config struct is labeled as adapter concern, but then used directly by domain services. This is contradictory.

**Original Flawed Statement:**
> "Config is pure infrastructure wiring"

**But Then:**
- Multiple domain services receive Config via constructor injection
- Domain services use Config paths for business decisions
- Config contains VaultPath, TemplatesDir, SchemasDir - business requirements!

**User Agreement:**
> "I agree with your claim that Config is misclassified and your suggested correction"

**Correct Classification:**
- **Config struct:** Domain Value Object (immutable business configuration data)
- **ConfigPort/ConfigViperAdapter:** SPI Adapter (loads and provides Config)

**Rationale:**
- Configuration contains business-relevant paths and settings
- The fact that Viper loads it doesn't make Config itself infrastructure
- Multiple domain components need these settings for business decisions
- Domain value objects can be loaded by adapters

---

### 6. VaultIndexer God Service (SIGNIFICANT)

**Location:** `docs/architecture/components.md:84-96`

**Problem:** VaultIndexer does too much, violating Single Responsibility Principle.

**Current Dependencies (6!):**
- FileSystemPort
- CacheCommandPort
- FrontmatterValidator
- SchemaValidator
- QueryService
- Logger

**What It Does:**
1. Scanning filesystem (coordination)
2. Parsing frontmatter (extraction)
3. Validating data (business rules)
4. Persisting cache (storage)
5. Updating query indices (CQRS write-side)

**User Agreement:**
> "I could not agree more with your assessment and suggested correction of the VaultIndexer! one minor adjustment to the suggested correction is that the frontmatter is validated against the schema so NoteValidator should be FrontmatterValidator"

**Correct Decomposition:**
```go
// Service 1: FrontmatterExtractor
type FrontmatterExtractor struct {
    yamlParser YAMLParser
}
func (e *FrontmatterExtractor) Extract(ctx context.Context, content []byte) (Frontmatter, error)

// Service 2: NoteBuilder
type NoteBuilder struct {
    extractor *FrontmatterExtractor
}
func (b *NoteBuilder) Build(ctx context.Context, id NoteID, metadata NoteMetadata, content []byte) (Note, error)

// Service 3: FrontmatterValidator (renamed from NoteValidator)
type FrontmatterValidator struct {
    schemaRegistry SchemaRegistryPort
    validatorReg   *ValidatorRegistry
}
func (v *FrontmatterValidator) Validate(ctx context.Context, schemaName string, fm Frontmatter) error

// Service 4: IndexOrchestrator (coordinates workflow)
type IndexOrchestrator struct {
    fsRead      FileReader
    fsWalk      FileWalker
    noteBuilder *NoteBuilder
    validator   *FrontmatterValidator
    cacheWriter CacheWriter
    logger      Logger
}
func (o *IndexOrchestrator) Index(ctx context.Context, paths []string) (IndexStats, error)

// Service 5: ProjectionBuilder (sync read model)
type ProjectionBuilder struct {
    // Builds query indices from notes
}
```

**Benefits:**
- Each service has single responsibility
- FrontmatterExtractor reusable
- Easy to test in isolation
- Clear separation of concerns

---

### 7. CommandOrchestrator Redundant Facade (MODERATE)

**Location:** `docs/architecture/components.md:110-122`

**Problem:** CommandOrchestrator is described as "Facade consumed by API adapters" but it's just a pass-through layer.

**Evidence:**
```go
// Just delegation, no added value
func (c *CommandOrchestrator) New(ctx, templateID) {
    return c.templateEngine.Execute(templateID, ctx)
}
```

**User Response:**
> "considering the use of main.go for dependency injection, I am not sure I can properly justify CommandOrchestrator"

**Decision:** Remove CommandOrchestrator

**Alternative Approach:**
```go
// main.go does DI
func main() {
    // Build domain services
    templateEngine := domain.NewTemplateEngine(...)
    indexOrchestrator := domain.NewIndexOrchestrator(...)

    // Inject into CLI adapter directly
    cliAdapter := adapters.NewCobraCLI(templateEngine, indexOrchestrator)
    cliAdapter.Execute(os.Args)
}
```

**When Facades ARE Useful:**
- Complex coordination logic across multiple services
- Transaction management
- Common pre/post processing
- Cross-cutting concerns (audit, metrics)

**Current Case:** None of these apply. CLI can call domain services directly through ports.

---

### 8. Error Handling: Result[T] vs (T, error) (MODERATE)

**Location:** `docs/architecture/error-handling-strategy.md`

**Problem:** Architecture uses custom Result[T] pattern mixed with idiomatic Go `(T, error)`.

**Evidence of Inconsistency:**
```go
// SchemaValidator uses Result[T]
ValidateSchema(ctx, schema) Result[ValidationResult]

// But other services use (T, error)
Execute(templateID string, ctx) (RenderResult, error)
Lookup(ctx, criteria) (Note, error)
```

**User Decision:**
> "if (T, error) is idiomatic Go, then it should be used"

**Rationale:**
- Result[T] useful in languages without multiple returns
- In Go, `(T, error)` is idiomatic and interoperates with stdlib
- Familiar to Go ecosystem
- No need to fight the language

**Resolution:** Use `(T, error)` throughout, remove Result[T] pattern

**Error Wrapping Pattern:**
```go
// Use fmt.Errorf with %w for wrapping
if err := someFunc(); err != nil {
    return fmt.Errorf("context: %w", err)
}

// Check error types with errors.Is() and errors.As()
var valErr ValidationError
if errors.As(err, &valErr) {
    // Handle validation error
}
```

---

### 9. Validation Architecture Complexity

**User's Critical Insight:**

> "I just realized that the schema system defines JSON schema and its validation system could be much simpler in that it needs to check whether the json is broken and if the objects are correctly configured. however, I also just realized the frontmatter validation might be much harder because it has to translate the conciseness of the frontmatter yaml back to the json schema"

**Two Different Validation Concerns:**

**1. Schema Validation (Simple - Structural Integrity)**
- Purpose: Validate JSON schema definition itself is well-formed
- Scope: JSON syntax + schema structure rules
- Complexity: Low - just check the schema definition is valid
- When: Schema load time (once)

**2. Frontmatter Validation (Complex - Business Rules + Type Coercion)**
- Purpose: Validate YAML frontmatter data against schema rules
- Scope: Type coercion + constraint checking + cross-field validation
- Complexity: High - translate YAML → apply JSON Schema rules
- When: Every note indexing/validation

**The Complexity: YAML → JSON Schema Impedance Mismatch**

```yaml
# YAML allows many representations
count: 42        # int
count: "42"      # string
count: 42.0      # float
# All might need to map to NumberSpec

tags: work       # Scalar
tags: [work]     # Array with one element
tags:            # Array with multiple
  - work
  - personal

active: true / yes / on / 1  # All boolean true
```

**User's Service Consolidation Decision:**

> "keep anemic for the MVP, but only mention FrontmatterService, which should have Extract and Validate methods, instead of two separate services"

**Final Design:**
```go
// Single service for frontmatter concerns
type FrontmatterService struct {
    schemaRegistry SchemaRegistry
}

// Extract YAML to Frontmatter
func (s *FrontmatterService) Extract(ctx context.Context, content []byte) (Frontmatter, error)

// Validate with type coercion
func (s *FrontmatterService) Validate(ctx context.Context, fm Frontmatter) error {
    // 1. Get schema from fm.FileClass
    // 2. For each property, coerce YAML type to expected type
    // 3. Validate coerced value against PropertySpec
}
```

---

## User Clarifications and Decisions

### Decision 1: Anemic vs Rich Domain Models

**Question:** Should Frontmatter define Validate and Extract methods, or keep behavior in services?

**User Decision:**
> "keep anemic for the MVP, but only mention FrontmatterService, which should have Extract and Validate methods, instead of two separate services"

**Rationale:**
1. **Clean hexagonal:** Validation needs external dependencies (schema registry, query service). If Frontmatter has Validate(), it couples model to ports.
2. **Testing:** Pure data models trivial to construct in tests
3. **YAGNI:** For MVP, services are simple enough that rich models don't add value

**Post-MVP:** If Frontmatter passed through many service methods, rich models might make sense.

---

### Decision 2: CQRS Model Separation

**User's Realization:**
> "your suggestion makes Note and NoteProjection have the same attributes, in which case NoteProjection would be redundant"

**Final Decision:**
- Single `Note` model for MVP (ID + Frontmatter)
- CQRS separation in operations/ports (CacheWriter vs CacheReader)
- Query indices built by QueryService (infrastructure), not in model
- Post-MVP: Schema-specific projections for business denormalizations

**User's Vision:**
> "I think this is the right direction for the MVP and in the future the different Schema models could be used to assist in structuring the read models"

---

### Decision 3: Frontmatter Field Classifications

**User Clarification:**
> "FileClass is a frontmatter field and has to be extracted, so it should be signified as computed"

**Corrected:**
```go
type Frontmatter struct {
    FileClass string         // Computed from Fields["fileClass"]
    Fields    map[string]any // Authoritative source
}
```

**User Clarification:**
> "Frontmatter is comprised of FileClass and Fields, so it seems to make the most sense that it should be used in Note and NoteProjection"

**Resolution:** Note composes Frontmatter, not duplicating its fields.

---

### Decision 4: Storage Port Naming

**User Correction:**
> "the Storage ports are CacheReader and CacheWriter"

**Corrected:**
```go
type CacheWriter interface {
    Store(ctx context.Context, note Note) error
    Delete(ctx context.Context, id NoteID) error
}

type CacheReader interface {
    Get(ctx context.Context, id NoteID) (Note, error)
    Filter(ctx context.Context, query Query) ([]Note, error)
}
```

---

### Decision 5: Adapter Naming - CQRS Split

**User Correction:**
> "we split the file system adapter into FileSystemWriteAdapter and FileSystemReadAdapter to match CQRS, but you only reference FileSystemAdapter here"

**Corrected:** All references use `FileSystemReadAdapter` and `FileSystemWriteAdapter`

---

### Decision 6: Note Relationships

**User Clarification:**
> "Note is not validated, only Frontmatter is validated"

**Corrected:** Relationships show FrontmatterService validates Frontmatter, not Note.

**User Clarification:**
> "I do not understand why Note would be used by the template engine when the template engine creates notes. the only use I can think of is that the template engine uses the query service which queries against note data"

**Corrected:**
- Template engine **creates** Notes (via rendering)
- Template engine **queries** existing Notes (via QueryService for lookup/query functions)
- Relationship: "Queried by QueryService (used by template engine's lookup/query functions)"

---

### Decision 7: Design Decision Terminology

**User Feedback:**
> "is 'anemic model' the design decision or that it is a pure data structure? I think the pure data structure design decision is more descriptive."

**Corrected:** Use "Pure data structure" instead of "Anemic model" in design decisions.

---

### Decision 8: Validation Redundancy

**User Feedback:**
> "do not mention that Frontmatter is validated because it is already written under the Frontmatter data model"

**Applied:** Don't repeat validation information in Note relationships since it's documented in Frontmatter section.

---

### Decision 9: Registry Package Concern

**User Partial Agreement:**
> "I partially agree with your claim that the shared registry package creates circular dependency. the documentation was not written accurately and in practice the shared registry packages are only supposed to be used by the adapters and injected into the service layer through interfaces. although, I am open to alternative implementations"

**Clarification:** Registry used by adapters, injected to services via interfaces. Not a shared dependency between layers.

---

### Decision 10: PropertySpec Validation Separation

**User Insight:**
> "wouldn't it be preferable to separate specific property specs from their validation in order that a general SchemaValidator interface using generics could be implemented and used for all parts of the schema system?"

**Agreement:** Separate spec definition (data) from validation logic (behavior).

**Design:**
```go
// PropertySpec is pure data
type PropertySpec[T any] interface {
    Type() PropertyType
    Constraints() any
}

// Separate validator with generic logic
type PropertyValidator[T any] interface {
    Validate(ctx context.Context, value T, spec PropertySpec[T]) error
}

// Unified registry
type ValidatorRegistry struct {
    stringValidator PropertyValidator[string]
    numberValidator PropertyValidator[float64]
    // ...
}
```

---

## Refactoring Plan and Progress

### Phase 1: High-Level Architecture (COMPLETED ✅)

**File:** `docs/architecture/high-level-architecture.md`

**Changes Made:**
1. Updated Key Architectural Decisions:
   - Ports speak in abstractions (storage, user interaction), not specifics
   - Added "Domain Uses Abstract Identifiers" decision
   - Corrected CQRS to true model separation (write/read models, synchronization)

2. Updated Architectural Patterns section:
   - CQRS now describes true separation (models + operations)
   - Clarified write model optimization (validation, integrity)
   - Clarified read model optimization (denormalized, indices)

3. Updated Design Principles:
   - Lean Ports (2-5 methods)
   - ISP Compliance (separate read/write even when same adapter)
   - Lean Domain Models (only essential data, no behavior)
   - CQRS with Separate Models
   - Dependency Injection via main.go
   - Idiomatic Go Error Handling

**Change Log Entry:** v0.5.0 - Updated High Level Architecture with clean hexagonal design principles

---

### Phase 2: Data Models (IN PROGRESS 🔄)

**File:** `docs/architecture/data-models.md`

**Completed:**

1. **Updated Introduction:**
   - Added Write Models (CQRS Command) classification
   - Added Read Models (CQRS Query) classification
   - Kept API Adapter classification for context
   - Clarified CQRS pattern purpose

2. **FileMetadata (Renamed from File):**
   - Purpose: Filesystem-specific metadata for storage adapters
   - Architecture Layer: SPI Adapter (Infrastructure)
   - Used by: FileSystemReadAdapter and FileSystemWriteAdapter
   - Never exposed to domain
   - Computed fields cached (Basename, Folder)
   - Change Log: v0.5.1

3. **NoteID (New):**
   - Purpose: Abstract domain identifier
   - Architecture Layer: Domain Core
   - Opaque to domain (string value)
   - Adapters translate NoteID ↔ infrastructure identifiers
   - Change Log: v0.5.2

4. **Frontmatter:**
   - Purpose: YAML content metadata
   - FileClass marked as computed
   - References FrontmatterService.Extract() and .Validate()
   - Anemic model (pure data structure)
   - Change Log: v0.5.2

5. **Note:**
   - Minimal composition: ID + Frontmatter
   - No infrastructure metadata
   - Pure data structure
   - Single model for MVP (CQRS in ports, not models)
   - Relationships: CacheWriter, CacheReader, QueryService
   - Kept Post-MVP section on body content indexing
   - Change Log: v0.5.2

6. **Schema:**
   - Key Attributes: Name, Extends, Excludes, Properties
   - Removed ResolvedProperties (adapter concern)
   - Inheritance resolution in SchemaLoader adapter
   - Pure data structure
   - Change Log: v0.5.3

7. **Property:**
   - Simplified to: Name, Required, Array, Spec
   - Pure data structure
   - Interface-based composition (PropertySpec)
   - Change Log: v0.5.3

**Remaining (TODO):**

8. **PropertySpec Variants:**
   - Separate spec definition from validation
   - Use generics for type safety
   - StringSpec, NumberSpec, FileSpec, DateSpec, BoolSpec

9. **PropertyBank:**
   - Singleton pattern
   - $ref resolution specification
   - Loaded by SchemaLoader
   - Used for property reuse

10. **TemplateID (New):**
    - Abstract identifier like NoteID
    - Decouples from filesystem

11. **Template:**
    - Use TemplateID instead of FilePath
    - Minimal: ID, Name, Content
    - Pure data structure

12. **TemplateMetadata (New):**
    - SPI Adapter model
    - Path, ModTime
    - Used by TemplateLoader adapters

13. **Config:**
    - Reclassify as Domain Value Object
    - Immutable configuration data
    - Loaded by ConfigLoader adapter

---

### Phase 3: Components (PENDING)

**File:** `docs/architecture/components.md`

**Planned Changes:**

1. **Update Domain Services:**
   - Remove VaultIndexer
   - Add: FrontmatterExtractor, NoteBuilder, FrontmatterValidator, IndexOrchestrator, ProjectionBuilder
   - Remove: CommandOrchestrator
   - Update: TemplateEngine, SchemaValidator, QueryService

2. **Update API Ports:**
   - Remove CLICommandPort (or update to remove CommandOrchestrator reference)
   - CLI calls domain services directly

3. **Update SPI Ports:**
   - Split FileSystemPort → FileReader, FileWriter, FileWalker
   - Rename: CacheCommandPort → CacheWriter
   - Rename: CacheQueryPort → CacheReader
   - Add: SchemaLoader (with inheritance resolution)
   - Remove: SchemaRegistryPort (just use interface)

4. **Update SPI Adapters:**
   - Split: LocalFileSystemAdapter → FileSystemReadAdapter, FileSystemWriteAdapter
   - Update: JSONFileCacheAdapter implements both CacheWriter and CacheReader
   - Add: SchemaLoaderAdapter (handles inheritance, $ref resolution)

5. **Update Shared Packages:**
   - Update Error Package: Remove Result[T], use (T, error)
   - Add: Lean error types (FieldError, MultiError)
   - Update: Registry usage (adapter-only, injected via interfaces)

---

### Phase 4: Error Handling Strategy (PENDING)

**File:** `docs/architecture/error-handling-strategy.md`

**Planned Changes:**

1. **General Approach:**
   - Remove Result[T] pattern
   - Use idiomatic `(T, error)`
   - Error wrapping with `fmt.Errorf("context: %w", err)`
   - Domain-specific error types implement `error` interface

2. **Error Types:**
   - Lean shared types: FieldError, MultiError
   - Domain-specific wrappers: SchemaError, FrontmatterValidationError, TemplateError
   - No "ValidationError" domain (validation is cross-cutting concern)

3. **Error Propagation:**
   - Use `errors.Is()` and `errors.As()` for type checking
   - Wrap errors with context at each layer
   - No panics except programmer assertions

---

### Phase 5: New Documentation (PENDING)

**New File:** `docs/architecture/dependency-injection.md`

**Content:**
1. main.go DI pattern
2. Layer initialization order: Infrastructure → Domain → Application → API
3. Constructor injection examples
4. No DI framework needed

**New File:** `docs/architecture/validation-architecture.md`

**Content:**
1. Schema Validation (simple structural checks)
2. Frontmatter Validation (complex with type coercion)
3. YAML → JSON Schema impedance mismatch
4. Type coercion rules and examples
5. FrontmatterService design

---

## Template Compliance Requirements

Per `.bmad-core/templates/architecture-tmpl.yaml`, each data model must have:

**Required Sections:**
```yaml
- id: model
  title: "{{model_name}}"
  template: |
    **Purpose:** {{model_purpose}}

    **Key Attributes:**
    - {{attribute_1}}: {{type_1}} - {{description_1}}
    - {{attribute_2}}: {{type_2}} - {{description_2}}

    **Relationships:**
    - {{relationship_1}}
    - {{relationship_2}}
```

**Additional Allowed (from current docs):**
- **Architecture Layer:** Classification
- **Design Decisions:** Rationale for choices
- **Helper Functions:** (optional code examples)
- **Additional Information:** (optional extended discussion)

**Must Update change-log.md After Every File Change**

---

## Key Principles Established

### Hexagonal Architecture Principles

1. **Dependency Direction:** Always inward toward domain
2. **Port Definition:** Domain defines what it needs, not what adapters provide
3. **Abstract Identifiers:** Domain uses opaque IDs (NoteID, TemplateID), not infrastructure refs
4. **Pure Domain:** No filesystem, database, or framework dependencies in domain

### Model Design Principles

1. **Lean Models:** Only essential data, no computed fields in domain
2. **Pure Data Structures:** No behavior in models (anemic for MVP)
3. **Interface Composition:** Use interfaces for polymorphism (PropertySpec)
4. **CQRS in Operations:** Single model for MVP, separate ports (CacheWriter/CacheReader)

### Service Design Principles

1. **Single Responsibility:** Each service has one clear job
2. **ISP Compliance:** Separate interfaces for different concerns
3. **Constructor Injection:** Dependencies via constructors, wired in main.go
4. **No Facades:** Don't add orchestration layers without added value

### Error Handling Principles

1. **Idiomatic Go:** Use `(T, error)`, not Result[T]
2. **Domain Errors:** Type-safe errors implementing `error` interface
3. **Error Wrapping:** Context via `fmt.Errorf("context: %w", err)`
4. **Type Checking:** Use `errors.Is()` and `errors.As()`

---

## Validation Architecture Design

### Schema Validation (Simple)

**What:** Validate JSON schema definition structure
**When:** Schema load time (once)
**Complexity:** Low

```go
type SchemaValidator struct{}

func (v *SchemaValidator) Validate(schema Schema) error {
    // Check Name not empty
    // Check Properties valid
    // Check Extends/Excludes consistent
    // Check PropertySpec structures valid
}
```

### Frontmatter Validation (Complex)

**What:** Validate YAML frontmatter against schema with type coercion
**When:** Every note indexing/validation
**Complexity:** High

```go
type FrontmatterService struct {
    schemaRegistry SchemaRegistry
}

func (s *FrontmatterService) Validate(ctx context.Context, fm Frontmatter) error {
    schema := s.schemaRegistry.Get(fm.FileClass)

    for _, prop := range schema.Properties {
        value := fm.Fields[prop.Name]

        // Type coercion (YAML → expected type)
        coerced := s.coerceValue(value, prop.Spec.Type())

        // Validate coerced value
        prop.Spec.Validate(ctx, coerced)
    }
}
```

**Type Coercion Examples:**
```go
// String coercion
"42" or 42 or true → "42", "42", "true"

// Number coercion
"42" or 42 or 42.0 → 42.0

// Boolean coercion
"true", "yes", "on", 1, true → true
"false", "no", "off", 0, false → false

// Array coercion
value or [value] → [value]
```

---

## Next Steps

1. **Complete data-models.md:**
   - PropertySpec variants (with validation separation)
   - PropertyBank (singleton + $ref spec)
   - TemplateID, Template, TemplateMetadata
   - Config reclassification

2. **Update components.md:**
   - Decompose VaultIndexer
   - Remove CommandOrchestrator
   - Split FileSystemPort
   - Update all port/adapter references

3. **Update error-handling-strategy.md:**
   - Remove Result[T]
   - Add lean error types
   - Add error wrapping examples

4. **Create dependency-injection.md:**
   - Document main.go DI pattern
   - Show layer initialization order
   - Provide examples

5. **Create validation-architecture.md:**
   - Document schema vs frontmatter validation
   - YAML→JSON coercion rules
   - FrontmatterService design

---

## Critical Success Factors

1. **Maintain Template Compliance:** Each model follows template structure
2. **Update Change Log:** After every file modification
3. **No Layer Violations:** Domain never depends on infrastructure
4. **Consistent Terminology:** CacheWriter/CacheReader, not generic Storage
5. **Clear Relationships:** Document how models interact
6. **Adapter References:** Use FileSystemReadAdapter/FileSystemWriteAdapter consistently
7. **Pure Data Structures:** No behavior in models for MVP
8. **ISP Compliance:** Separate interfaces for different concerns

---

## Glossary

**Key Terms:**
- **Domain Core:** Pure business logic, no infrastructure dependencies
- **SPI Adapter:** Service Provider Interface - driven adapters implementing ports
- **API Adapter:** Application Programming Interface - driving adapters using domain
- **CQRS:** Command Query Responsibility Segregation
- **ISP:** Interface Segregation Principle
- **DIP:** Dependency Inversion Principle
- **Anemic Model:** Data structure with no behavior (methods in services)
- **Rich Model:** Data structure with behavior methods
- **Aggregate Root:** Main entity that owns value objects (e.g., Note owns Frontmatter)
- **Value Object:** Immutable object with no identity (e.g., Frontmatter)

**Port Naming:**
- CacheWriter (not CacheCommandPort)
- CacheReader (not CacheQueryPort)
- FileReader, FileWriter, FileWalker (not FileSystemPort)

**Adapter Naming:**
- FileSystemReadAdapter
- FileSystemWriteAdapter
- SchemaLoaderAdapter
- ConfigLoaderAdapter

**Service Naming:**
- FrontmatterService (not FrontmatterExtractor + FrontmatterValidator)
- IndexOrchestrator (not VaultIndexer)
- QueryService
- TemplateEngine

---

## End of Review Document

This document serves as the comprehensive reference for the architecture refactoring. All future updates should align with the principles, decisions, and patterns documented here.

**Document Version:** 1.0
**Last Updated:** 2025-10-24
**Status:** Active Reference for Ongoing Refactoring
