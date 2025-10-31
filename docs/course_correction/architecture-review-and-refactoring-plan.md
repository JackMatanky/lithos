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

### 7. CommandOrchestrator: From Redundant Facade to Use Case Orchestrator (MODERATE)

**Location:** `docs/architecture/components.md:110-122`

**Initial Problem (October 24, 2025):** CommandOrchestrator is described as "Facade consumed by API adapters" but it's just a pass-through layer.

**Evidence:**
```go
// Just delegation, no added value
func (c *CommandOrchestrator) New(ctx, templateID) {
    return c.templateEngine.Execute(templateID, ctx)
}
```

**Initial User Response:**
> "considering the use of main.go for dependency injection, I am not sure I can properly justify CommandOrchestrator"

**Initial Decision:** Remove CommandOrchestrator

**Initial Alternative Approach:**
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

---

**Architecture Review (October 26, 2025):** User challenged the removal based on hexagonal architecture principles.

**User Feedback:**
> "it seems necessary that we rethink the CLI command system to ensure the CLICommandAdapter is not directly dependent on the service layer and that the CommandOrchestrator is a proper orchestrator and not just a facade"

**Revised Decision:** **Reinstate CommandOrchestrator** as proper use case orchestrator with proper hexagonal architecture

**Why CommandOrchestrator IS Necessary:**
- **Hexagonal Architecture:** Domain should not depend on API adapters, but needs to control application flow
- **Inversion of Control:** Domain starts the application and calls CLIPort.Start(), adapter calls back to CommandPort
- **Use Case Orchestration:** Coordinates multiple domain services for complete workflows (not just pass-through)
- **Real Added Value:** NoteID generation, file path resolution, workflow coordination, Note object creation

**Proper Architecture (v0.6.4):**
```go
// CommandOrchestrator (Domain Layer)
type CommandOrchestrator struct {
    cliPort         CLIPort  // API Port
    templateEngine  *TemplateEngine
    vaultIndexer    *VaultIndexer
    queryService    *QueryService
    frontmatterSvc  *FrontmatterService
    // ...
}

// Implements CommandPort callback interface
func (o *CommandOrchestrator) NewNote(ctx context.Context, templateID TemplateID) (Note, error) {
    // 1. Load and render template
    // 2. Extract frontmatter
    // 3. Validate frontmatter
    // 4. Generate NoteID (filename field → title slug → UUID)
    // 5. Resolve file path from template functions
    // 6. Create Note object
    // 7. Save to vault
    // 8. Return Note
}

// Starts application
func (o *CommandOrchestrator) Run(ctx context.Context) error {
    return o.cliPort.Start(ctx, o) // Pass itself as CommandPort
}

// CLIPort (API Port)
type CLIPort interface {
    Start(ctx context.Context, handler CommandPort) error
}

// CommandPort (Callback Interface)
type CommandPort interface {
    NewNote(ctx context.Context, templateID TemplateID) (Note, error)
    IndexVault(ctx context.Context) (IndexStats, error)
    FindTemplates(ctx context.Context, query string) ([]Template, error)
}
```

**When Facades ARE Useful:**
- Complex coordination logic across multiple services ✓ (CommandOrchestrator does this)
- Transaction management
- Common pre/post processing ✓ (NoteID generation, validation)
- Cross-cutting concerns (audit, metrics)

**Updated Understanding:** CommandOrchestrator is NOT a facade - it's a proper use case orchestrator with real coordination logic.

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

**User Decision (October 24, 2025):**
> "keep anemic for the MVP, but only mention FrontmatterService, which should have Extract and Validate methods, instead of two separate services"

**Rationale for Anemic Models (Note, Frontmatter):**
1. **Clean hexagonal:** Validation needs external dependencies (schema registry, query service). If Frontmatter has Validate(), it couples model to ports.
2. **Testing:** Pure data models trivial to construct in tests
3. **YAGNI:** For MVP, services are simple enough that rich models don't add value

**Post-MVP:** If Frontmatter passed through many service methods, rich models might make sense.

---

**User Decision Update (October 26, 2025):**
> "yes, I approve updating schema models to be rich while Note/Frontmatter stay anemic"

**Rationale for Rich Models (Schema, Property, PropertySpec):**
1. **No external dependencies:** Schema validation only checks internal structure - no ports needed
2. **Polymorphism benefit:** PropertySpec variants implement interface with Validate() method
3. **Structural integrity:** Validation is inherent to schema definition itself
4. **Fail-fast at load time:** Models can self-validate when loaded

**Implementation (v0.6.0):**
```go
// Rich models with structural validation
func (s Schema) Validate(ctx context.Context) error
func (p Property) Validate(ctx context.Context) error
func (s StringSpec) Validate(ctx context.Context) error
func (n NumberSpec) Validate(ctx context.Context) error
```

**SchemaValidator Service Still Needed:**
- Orchestrates model validation (calls Schema.Validate() on each)
- Performs cross-schema validation (Extends references, PropertyBank $ref, duplicate names)
- Individual models can't validate cross-schema concerns

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
    Type() PropertySpecType
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

**Note (October 26, 2025):** This decision was revisited - PropertySpec now has Validate() methods as rich models (see Decision 1 update). The separation principle still applies for complex validators.

---

### Decision 11: Internal vs Injected Dependencies

**Context (October 26, 2025):** SchemaValidator and SchemaResolver are only used by SchemaEngine.

**User Question:**
> "the SchemaValidator and SchemaResolver are strictly embedded in SchemaEngine, so couldn't their instantiation be defined in SchemaEngine and be triggered when SchemaEngine is instantiated in the DI?"

**Decision:** SchemaEngine **internally instantiates** SchemaValidator and SchemaResolver rather than receiving them via constructor injection.

**Rule Established:**
- **Inject:** Dependencies that cross boundaries, need substitution for testing, or are shared services
- **Internally Instantiate:** Single-use dependencies that are implementation details

**Example (v0.6.1):**
```go
// SchemaEngine internally creates its validators
func NewSchemaEngine(loader SchemaLoader, registry SchemaRegistry, log Logger) *SchemaEngine {
    validator := NewSchemaValidator()
    resolver := NewSchemaResolver()
    return &SchemaEngine{
        loader:    loader,    // injected (port)
        registry:  registry,  // injected (port)
        validator: validator, // internal
        resolver:  resolver,  // internal
        log:       log,       // injected (shared)
    }
}
```

---

### Decision 12: Validation vs Linting Philosophy

**Context (October 26, 2025):** How strict should frontmatter validation be?

**User Clarification:**
> "if validating that a field is an array, the validator should not coerce a field into an array because that would be the job of a linter. a validator should not coerce every value because sometimes it needs to raise an error"

**Decision:** **Validator is strict** - raises errors for type mismatches. **Linter is permissive** (future feature) - auto-fixes issues.

**Validation Philosophy (v0.6.2):**
- **NO semantic coercion:** `tags: work` when Property.Array = true is **ERROR**, not auto-fix to `[work]`
- **NO type coercion across semantic boundaries:** `count: "42"` when NumberSpec is **ERROR**, not auto-fix to `42`
- **YES in-memory normalization:** YAML int → float64 for NumberSpec (YAML parser artifact)
- **Files never modified:** All normalization is in-memory for validation logic only

**Rationale:**
- Validation errors reveal schema mismatches that should be fixed in schema definition
- Auto-coercion hides problems and prevents catching user mistakes
- Linting is separate future feature for intentional auto-fixing

---

### Decision 13: Template File Path Control

**Context (October 26, 2025):** Templates need to control where generated notes are saved in the vault.

**User Realization:**
> "I realize now that unless the Go stdlib can be used to find a file path and designate where a file is created, then we need to add functions to the custom function map... similar to the move and path functions in templater"

**Decision:** Add file path control functions to TemplateEngine's custom function map (inspired by Obsidian Templater's file module).

**Functions Added (v0.6.3):**
- `path()` - Returns the target file path for the note being created
- `folder(path)` - Returns parent directory of path
- `basename(path)` - Returns filename without extension
- `extension(path)` - Returns file extension
- `join(parts...)` - Joins path segments using OS-appropriate separator
- `vaultPath()` - Returns absolute vault root path from Config

**Example Template:**
```go
{{- $targetPath := join (vaultPath) "contacts" (printf "%s.md" (prompt "filename" "Filename" "")) -}}
```

---

### Decision 14: NewNote Primary Behavior and Return Type

**Context (October 26, 2025):** What should `lithos new` command return?

**User Clarification:**
> "the intended primary behavior of lithos new is to save a rendered markdown file in the vault... a secondary behavior could be that after saving... the CLI could give the option of viewing the file in stdout"

**Decision:** Primary behavior is **save rendered note to vault**. Secondary optional behavior is display to stdout.

**NewNote Return Type (v0.6.4):** Returns `Note` object (not string) because:
- Primary concern is persisting to vault
- Note object contains all metadata (ID, frontmatter, content)
- CLI adapter can extract what it needs for display
- Enables future behaviors (open in editor, add to index, etc.)

**NoteID Generation Strategy:**
```go
// Priority 1: Use explicit filename field from frontmatter
if filename, ok := fm.Fields["filename"].(string); ok {
    return NoteID(filename), nil
}
// Priority 2: Slugify title field
if title, ok := fm.Fields["title"].(string); ok {
    return NoteID(slugify(title)), nil
}
// Priority 3: Generate UUID
return NoteID(generateUUID()), nil
```

---

### Decision 15: SRP Decomposition Pattern

**Context (October 26, 2025):** Public methods in adapters were doing too much.

**User Requirement:**
> "all methods would have to be decomposed into private SRP methods and embedded in the public methods"

**Decision:** All public methods with multiple steps **MUST** decompose into private SRP (Single Responsibility Principle) methods.

**Pattern (v0.6.4):**
- **Public methods:** Orchestrate workflow
- **Private methods:** Focused, single-responsibility tasks

**Example (CobraCLIAdapter):**
```go
// Public - orchestrates
func (a *CobraCLIAdapter) Start(ctx context.Context, handler CommandPort) error {
    rootCmd := a.buildRootCommand()
    rootCmd.AddCommand(
        a.buildNewCommand(handler),
        a.buildIndexCommand(handler),
        a.buildFindCommand(handler),
    )
    return rootCmd.ExecuteContext(ctx)
}

// Private - builds command
func (a *CobraCLIAdapter) buildNewCommand(handler CommandPort) *cobra.Command

// Private - handles workflow
func (a *CobraCLIAdapter) handleNewCommand(...) error {
    templateID, err := a.selectTemplate(...)
    note, err := handler.NewNote(...)
    return a.displayNoteCreated(...)
}

// Private - template selection
func (a *CobraCLIAdapter) selectTemplate(...) (TemplateID, error)

// Private - display result
func (a *CobraCLIAdapter) displayNoteCreated(...) error
```

**Benefit:** Each method has clear, testable responsibility.

---

### Decision 16: Filesystem Operations - YAGNI Principle

**Context (October 24-26, 2025):** Should we create FileSystemPort, FileReaderPort, FileWriterPort, FileWalkerPort interfaces?

**Decision:** **No filesystem ports for MVP.** Use Go stdlib (`os.ReadFile`, `filepath.Walk`) and `moby/sys/atomicwriter` directly in adapters.

**Rationale (YAGNI - You Aren't Gonna Need It):**
- MVP has **single file source:** Local filesystem only
- No requirement for multiple storage backends (S3, HTTP, embedded)
- Creating ports/adapters for single implementation adds unnecessary abstraction
- Go stdlib provides all needed functionality
- Can add ports later if multiple implementations actually needed

**What Adapters Use Directly:**
- `os.ReadFile` - Read files
- `filepath.Walk` - Scan directories
- `atomicwriter.WriteFile` - Atomic writes (cache, notes)
- `os.Stat` - File metadata

**Future Migration Path:**
If future needs arise (S3, HTTP, embedded files), add ports then:
```go
// Future - only if needed
type FileReaderPort interface {
    Read(ctx context.Context, path string) ([]byte, error)
}

type FileWriterPort interface {
    Write(ctx context.Context, path string, data []byte) error
}
```

**Impact on Architecture:**
- VaultIndexer uses Go stdlib directly (no FileWalker/FileReader ports)
- CommandOrchestrator uses `atomicwriter.WriteFile` directly (no FileWriter port)
- Cache adapters use Go stdlib directly (documented in components.md)

**Change Log:** v0.5.11 (removed filesystem ports per YAGNI), components.md notes added

---

### Decision 17: Vault Operations with CQRS Pattern

**Context (October 26, 2025):** How should domain services (VaultIndexer, CommandOrchestrator) interact with the vault (markdown files)?

**Key Realization:** Domain services should NOT use Go stdlib directly (violates hexagonal architecture) and should NOT depend on generic FileSystemPort (wrong abstraction level).

**Decision:** Create **VaultReaderPort** and **VaultWriterPort** following CQRS pattern with business-level abstractions.

**Architecture Pattern:**

```go
// CQRS Read Side - Vault scanning for indexing
type VaultReaderPort interface {
    // Full scan for initial index build
    ScanAll(ctx context.Context) ([]VaultFile, error)

    // Incremental scan for large vaults (future scalability)
    ScanModified(ctx context.Context, since time.Time) ([]VaultFile, error)

    // Single file content read (not just .md - any vault file)
    Read(ctx context.Context, path string) ([]byte, error)
}

// CQRS Write Side - Vault persistence
type VaultWriterPort interface {
    Persist(ctx context.Context, note Note, path string) error
    Delete(ctx context.Context, path string) error
}

// VaultFile - SPI Adapter DTO (embeds FileMetadata + adds Content)
type VaultFile struct {
    FileMetadata        // Embedded: Path, Basename, Folder, Ext, ModTime, Size, MimeType
    Content      []byte // Raw file content (optional - nil for large files)
}

// FileMetadata - SPI Adapter model (extended with Ext, Size, MimeType)
type FileMetadata struct {
    Path     string      // Absolute path
    Basename string      // Filename without extension (computed)
    Folder   string      // Parent directory (computed)
    Ext      string      // File extension with dot (computed) - e.g., ".md"
    ModTime  time.Time   // Modification timestamp
    Size     int64       // File size in bytes
    MimeType string      // MIME type (computed from Ext)
}
```

**CacheWriterPort Update (consistency with VaultWriterPort):**

```go
// CQRS Write Side - Cache persistence
type CacheWriterPort interface {
    Persist(ctx context.Context, note Note) error  // Renamed from Write
    Delete(ctx context.Context, id NoteID) error
}
```

**Why CQRS at Vault Level:**

The vault IS the source of truth, and the cache is a projection:

```
Vault (Source of Truth - All Vault Files)
    ↓ VaultReaderPort.ScanAll/ScanModified
VaultIndexer (Projection Builder)
    ↓ CacheWriterPort.Persist
Cache Layer (Projections/Read Models)
    ├─> JSON Cache (.lithos/cache/) - Full data (MVP)
    └─> BoltDB Index (future) - Optimized queries
    ↓ CacheReaderPort.Read/List
QueryService (Query Handler)
    ↓
User Queries
```

**Rationale:**

1. **Business-level abstraction:** Ports express "scan vault", "persist note" (business intent), not "read file", "walk directory" (infrastructure detail)
2. **Hexagonal architecture compliance:** Domain depends on domain-defined ports, infrastructure implements them
3. **Future-proof for hybrid index (NFR4):**
   - `ScanModified()` enables incremental indexing for large vaults (100K+ notes)
   - Supports multiple cache backends (JSON + BoltDB)
   - Read/write separation enables optimization
4. **Testability:** Mock vault operations without filesystem I/O
5. **CQRS at two levels:**
   - Vault level: VaultReader/VaultWriter (source of truth)
   - Cache level: CacheReader/CacheWriter (projections)
6. **Method naming:**
   - `Read()` not `ReadNote()` - vault contains all files, not just markdown notes
   - `Persist()` not `Write()` - avoids redundancy with "Writer" in port name
   - Consistent naming across VaultWriter and CacheWriter

**Implementation (MVP):**

```go
// Separate adapters for CQRS read/write separation and ISP compliance

// VaultReaderAdapter - Implements VaultReaderPort
type VaultReaderAdapter struct {
    config Config
    log    Logger
}

func (a *VaultReaderAdapter) ScanAll(ctx context.Context) ([]RawNote, error) {
    // Uses filepath.Walk, os.ReadFile internally (infrastructure detail)
}

func (a *VaultReaderAdapter) ScanModified(ctx context.Context, since time.Time) ([]RawNote, error) {
    // Uses filepath.Walk with ModTime check, os.ReadFile internally
}

func (a *VaultReaderAdapter) Read(ctx context.Context, path string) (RawNote, error) {
    // Uses os.ReadFile, os.Stat internally
}

// VaultWriterAdapter - Implements VaultWriterPort
type VaultWriterAdapter struct {
    config Config
    log    Logger
}

func (a *VaultWriterAdapter) Persist(ctx context.Context, note Note, path string) error {
    // Uses atomicwriter.WriteFile internally (infrastructure detail)
}

func (a *VaultWriterAdapter) Delete(ctx context.Context, path string) error {
    // Uses os.Remove internally
}
```

**Rationale for Separate Adapters:**
- **CQRS pattern:** Read and write concerns are separate
- **ISP compliance:** Components only depend on what they need (VaultIndexer doesn't need write operations)
- **Modularity:** Can optimize read and write independently
- **Testability:** Mock read/write operations separately
- **Future flexibility:** Can swap read implementation (e.g., file watching) without affecting writes

**Domain Service Dependencies:**

```go
type VaultIndexer struct {
    vaultReader        VaultReaderPort        // CQRS read side
    cacheWriter        CacheWriterPort        // CQRS write side
    frontmatterService *FrontmatterService
    queryService       *QueryService
    // ...
}

type CommandOrchestrator struct {
    vaultWriter VaultWriterPort        // CQRS write side
    cacheWriter CacheWriterPort        // Dual write for consistency
    templateEngine *TemplateEngine
    // ...
}
```

**VaultIndexer Workflow (Building Notes from VaultFile):**

```go
func (v *VaultIndexer) Build(ctx context.Context) (IndexStats, error) {
    // 1. Scan vault - returns []VaultFile
    vaultFiles, err := v.vaultReader.ScanAll(ctx)
    if err != nil {
        return IndexStats{}, fmt.Errorf("vault scan failed: %w", err)
    }

    // 2. Process each vault file
    for _, vf := range vaultFiles {
        // Filter: MVP only processes markdown files
        if vf.Ext != ".md" {
            continue // Skip non-markdown files
        }

        // Extract frontmatter from content
        fm, err := v.frontmatterService.Extract(vf.Content)
        if err != nil {
            v.log.Warn().Str("path", vf.Path).Err(err).Msg("frontmatter extraction failed")
            continue
        }

        // Validate frontmatter against schema
        if err := v.frontmatterService.Validate(ctx, fm); err != nil {
            v.log.Warn().Str("path", vf.Path).Err(err).Msg("frontmatter validation failed")
            continue
        }

        // Derive NoteID from path (adapter concern - abstracts filesystem)
        noteID := v.deriveNoteIDFromPath(vf.Path)

        // Construct Note domain model
        note := Note{
            ID:          noteID,
            Frontmatter: fm,
            // Post-MVP: Content: vf.Content
        }

        // Persist to cache (projection)
        if err := v.cacheWriter.Persist(ctx, note); err != nil {
            return IndexStats{}, fmt.Errorf("cache write failed: %w", err)
        }
    }

    return IndexStats{Total: len(vaultFiles)}, nil
}

// deriveNoteIDFromPath translates filesystem path to domain identifier
// Adapter-level concern kept in VaultIndexer for path→ID translation
func (v *VaultIndexer) deriveNoteIDFromPath(path string) NoteID {
    // Use relative path from vault root as ID
    relPath, _ := filepath.Rel(v.config.VaultPath, path)
    return NoteID(relPath)
}
```

**Dual Write Pattern (CommandOrchestrator.NewNote):**

```go
func (o *CommandOrchestrator) NewNote(ctx context.Context, templateID TemplateID) (Note, error) {
    // 1. Render, validate, create note
    // 2. Persist to vault (source of truth)
    if err := o.vaultWriter.Persist(ctx, note, path); err != nil {
        return Note{}, err
    }
    // 3. Persist to cache (projection) - keeps index in sync
    if err := o.cacheWriter.Persist(ctx, note); err != nil {
        // Log warning but don't fail - can rebuild index later
        o.log.Warn().Err(err).Msg("failed to update cache")
    }
    return note, nil
}
```

**What This Is NOT:**

- ❌ Generic FileSystemPort with low-level operations
- ❌ Domain services using Go stdlib directly
- ❌ Infrastructure abstraction without business meaning

**What This IS:**

- ✅ Business-level ports expressing vault operations
- ✅ CQRS pattern for scalability (NFR4 compliance)
- ✅ Clean hexagonal architecture
- ✅ Future-proof for hybrid index with BoltDB
- ✅ Consistent naming (Persist, not Write)

**Change Log:** v0.6.8 (added VaultReaderPort and VaultWriterPort with CQRS pattern, renamed CacheWriterPort.Write to Persist)

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

### Phase 2: Data Models (COMPLETED ✅)

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
   - Used by: FileSystemReadAdapter, FileSystemWriteAdapter, and TemplateLoader adapters
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

6. **Schema (Updated to Rich Model - v0.6.0):**
   - Key Attributes: Name, Extends, Excludes, Properties
   - Removed ResolvedProperties (adapter concern)
   - Inheritance resolution in SchemaLoader adapter
   - **Rich model with Validate() method** for structural integrity
   - Change Log: v0.5.3, v0.6.0

7. **Property (Updated to Rich Model - v0.6.0):**
   - Simplified to: Name, Required, Array, Spec
   - Interface-based composition (PropertySpec)
   - **Rich model with Validate() method** for structural integrity
   - Change Log: v0.5.3, v0.6.0

8. **PropertySpec Variants (Updated to Rich Models - v0.6.0):**
   - PropertySpec is interface, variants implement it
   - Each variant has Validate() method for structural integrity
   - StringSpec, NumberSpec, FileSpec, DateSpec, BoolSpec
   - Generics for type safety
   - Change Log: v0.5.4, v0.6.0

9. **PropertyBank:**
   - Singleton pattern
   - $ref resolution specification
   - Loaded by SchemaLoader
   - Used for property reuse
   - Change Log: v0.5.5

10. **TemplateID:**
    - Abstract identifier like NoteID
    - Decouples from filesystem
    - Change Log: v0.5.6

11. **Template:**
    - Use TemplateID instead of FilePath
    - Minimal: ID, Name, Content
    - Pure data structure
    - Change Log: v0.5.6

12. **No TemplateMetadata Model:**
    - Decision: Reuse existing FileMetadata for template file metadata
    - TemplateLoader adapters use FileMetadata directly
    - Avoids unnecessary duplication
    - Change Log: v0.5.6

13. **Config:**
    - Reclassified as Domain Value Object
    - Immutable configuration data
    - Loaded by ConfigLoader adapter
    - Change Log: v0.5.7

14. **VaultFile (Added - v0.6.8):**
    - Purpose: Data transfer object for vault scanning
    - Architecture Layer: SPI Adapter (DTO)
    - Embeds FileMetadata + adds Content ([]byte)
    - Returned by VaultReaderPort.ScanAll/ScanModified/Read
    - Consumed by VaultIndexer to construct Note domain models
    - Not a domain model - infrastructure data transfer only
    - Change Log: v0.6.8

---

### Phase 3: Components (PARTIALLY COMPLETED 🔄)

**File:** `docs/architecture/components.md`

**Previously Completed (v0.5.11):**

1. **Domain Services:**
   - Removed VaultIndexer (decomposed per SRP)
   - Updated: TemplateEngine, FrontmatterService, SchemaEngine with generics, VaultIndexer, QueryService

2. **API Ports:**
   - Updated CLIPort

3. **SPI Ports:**
   - Split: CacheCommandPort/CacheQueryPort → CacheWriter/CacheReader
   - Added: SchemaPort, TemplatePort
   - Split: PromptPort/FinderPort
   - Added: ConfigPort, SchemaRegistryPort
   - Removed filesystem ports per YAGNI

4. **SPI Adapters:**
   - Split JSONCache adapters
   - Added: SchemaLoaderAdapter, TemplateLoaderAdapter
   - Added: PromptUIAdapter, FuzzyfindAdapter
   - Added: ViperAdapter, SchemaRegistryAdapter
   - Updated: CobraCLIAdapter

**Additional Completed Work (v0.6.1 - v0.6.6):**

5. **Rich Domain Models and Validation Services (v0.6.0 - v0.6.1):**
   - Added SchemaValidator service (orchestrates model validation, performs cross-schema checks)
   - Added SchemaResolver service (resolves inheritance and $ref using dependency graph)
   - Clarified SchemaEngine internally instantiates SchemaValidator and SchemaResolver
   - Updated Schema, Property, PropertySpec to be rich models with Validate() methods

6. **Frontmatter Validation Architecture (v0.6.2):**
   - Expanded FrontmatterService with detailed 7-step validation workflow
   - Documented type coercion rules and strict validation philosophy
   - Clarified in-memory normalization only (files never modified)
   - No semantic coercion (scalar→array is linting, not validation)

7. **Template File Path Functions (v0.6.3):**
   - Added file path control functions to TemplateEngine
   - Functions: path(), folder(), basename(), extension(), join(), vaultPath()
   - Enables templates to control their own save locations

8. **CLI Architecture Redesign (v0.6.4):**
   - Reinstated CommandOrchestrator as proper use case orchestrator
   - Redesigned CLIPort with Start(ctx, handler) method
   - Added CommandPort callback interface (NewNote, IndexVault, FindTemplates)
   - Implemented proper hexagonal architecture with inversion of control
   - Expanded CobraCLIAdapter with SRP decomposition pattern
   - Documented NoteID generation strategy (filename field → title slug → UUID)

9. **Dependency Injection Documentation (v0.6.5):**
   - Added DI Pattern section to components.md
   - Documented initialization order (5 layers)
   - Clarified internal vs injected dependencies
   - Provided example main.go structure

10. **Validation Architecture Overview (v0.6.6):**
    - Added comprehensive validation architecture section
    - Separated schema validation (structural, startup) from frontmatter validation (business rules, runtime)
    - Documented validation philosophy and complexity differences

11. **Vault Operations with CQRS Pattern (v0.6.8):**
    - Added VaultReaderPort (ScanAll, ScanModified, Read) for CQRS read-side vault access
    - Added VaultWriterPort (Persist, Delete) for CQRS write-side vault persistence
    - Added VaultReaderAdapter and VaultWriterAdapter implementing respective ports
    - Renamed CacheWriterPort.Write() to Persist() for naming consistency
    - Updated VaultIndexer dependencies (now depends on VaultReaderPort, not generic filesystem)
    - Updated CommandOrchestrator dependencies (now depends on VaultWriterPort and CacheWriterPort)
    - Documented dual write pattern (vault + cache) for eventual consistency
    - Updated component diagrams replacing FileSystemPort with vault ports
    - Updated DI initialization order and example main.go
    - Uses VaultFile DTO (embeds FileMetadata + Content) as return type

**Remaining Planned Changes:**

- Add: FrontmatterExtractor, NoteBuilder, IndexOrchestrator, ProjectionBuilder (VaultIndexer decomposition details)
- Additional shared package updates

---

### Phase 4: Error Handling Strategy and Coding Standards (COMPLETED ✅)

**Files:** `docs/architecture/error-handling-strategy.md` (v0.5.9), `docs/architecture/coding-standards.md` (v0.6.7)

**Completed Changes:**

1. **Error Handling Strategy (v0.5.9):**
   - Removed Result[T] pattern references
   - Renamed ValidationError to FrontmatterError
   - Split StorageError into CacheReadError/CacheWriteError/FileSystemError
   - Documented domain-specific error types with standard `error` interface
   - Error wrapping with `fmt.Errorf("context: %w", err)`

2. **Coding Standards (v0.6.7):**
   - Removed all Result[T] pattern requirements
   - Updated to mandate idiomatic Go `(T, error)` signatures throughout
   - Updated error handling section with proper wrapping and unwrapping
   - Changed naming conventions table: removed Result helpers, added Error Types
   - Required domain-specific error types to implement `error` interface with `Unwrap()` method
   - Required adapters to convert infrastructure errors to domain error types

---

### Phase 5: New Documentation (COMPLETED via components.md ✅)

**Decision:** Instead of creating separate files, added comprehensive sections to `docs/architecture/components.md`

**Dependency Injection Documentation (v0.6.5):**
- Added "Dependency Injection Pattern" section at end of components.md
- Documented initialization order: Infrastructure → SPI Adapters → Domain Services → CommandOrchestrator → API Adapters
- Clarified internal vs injected dependencies pattern
- Provided example main.go structure showing constructor injection
- Explained when to inject (cross boundaries, need substitution) vs internally instantiate (single-use, internal logic)

**Validation Architecture Documentation (v0.6.6):**
- Added "Validation Architecture Overview" section at end of components.md
- Separated schema validation (structural integrity, simple, startup, rich models) from frontmatter validation (business rules, complex, runtime, anemic models)
- Documented YAML → JSON Schema impedance mismatch
- Included type coercion rules and strict validation philosophy
- Cross-referenced FrontmatterService and SchemaValidator sections

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
