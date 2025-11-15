# Data Models

This section defines the data models used throughout the system, organized by architectural layer per hexagonal architecture principles.

## Model Layer Classification

Models are classified by their architectural layer and purpose:

| Layer                           | Purpose                                                       | Examples                                                            | Location Pattern                        |
| ------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------- | --------------------------------------- |
| **Domain Core**                 | Pure business entities and value objects with domain behavior | Note, Frontmatter, Schema, Property, PropertyBank, Template, Config | `internal/domain/`                      |
| **Application Models**          | Service-layer coordination models (currently none for MVP)    | -                                                                   | `internal/app/*/models.go`              |
| **Transport DTOs (Adapter)**    | Data transfer between layers, infrastructure concerns         | VaultFile, BoltDBMetadata, SQLiteMetadata                           | `internal/adapters/spi/*/dto.go`        |
| **Projection Models (Adapter)** | Read-optimized views for queries (currently none for MVP)     | -                                                                   | `internal/adapters/spi/cache/*_view.go` |
| **Domain Events**               | Significant domain occurrences for pub/sub                    | NoteIndexed, VaultIndexingComplete, FrontmatterValidated            | `internal/domain/events.go`             |

**Key Principles:**

- **Domain Core:** Pure business logic with no infrastructure dependencies - contains only essential business data + behavior
- **Infrastructure DTOs:** Filesystem paths, timestamps, serialization details belong in adapter layer
- **CQRS Separation:** Write concerns (validation, integrity) separated from read concerns (query performance) in operations/ports, not models (single model for MVP)
- **Event-Driven:** Domain events enable loose coupling between services via publish/subscribe pattern

## FileMetadata

**Purpose:** Filesystem-specific metadata used exclusively by filesystem storage adapters. Maps NoteID to file paths and tracks file system state.

**Architecture Layer:** SPI Adapter (Infrastructure)

**Rationale:** FileMetadata is infrastructure model used by VaultReadAdapter and VaultWriteAdapter to translate between domain identifiers (NoteID) and filesystem paths. Domain never depends on filesystem paths - adapters handle this translation. Enables filesystem implementation details to change without affecting domain.

**Key Attributes:**

- `Path` (string) - Absolute path to file. Serves as primary key and unique identifier across the system. Immutable once set. Used for cache keys, file identification, and adapter operations. Format: OS-specific absolute path (e.g., `/vault/notes/contact.md`).
- `Basename` (string, computed) - Filename without path and extension. Computed from Path using `filepath.Base()` and `strings.TrimSuffix()`. Used by template `lookup()` function (returns `map[basename]Path`) and wikilink resolution `[[basename]]`. Computed once during construction, cached in struct.
- `Folder` (string, computed) - Parent directory path. Computed from Path using `filepath.Dir()`. Used by template functions for file organization queries (e.g., "all notes in projects/"). Computed once during construction, cached in struct.
- `Ext` (string, computed) - File extension including dot. Computed from Path using `filepath.Ext()`. Used for file type filtering (e.g., ".md", ".pdf", ".png"). Empty string if no extension. Computed once during construction, cached in struct.
- `ModTime` (time.Time) - File modification timestamp from `os.Stat()`. Used for staleness detection by comparing cached ModTime against current filesystem ModTime. Enables incremental indexing optimizations (scan only files modified since last index). Format: RFC3339 for JSON serialization.
- `Size` (int64) - File size in bytes from `os.Stat()`. Used for filtering large files or determining if content should be loaded. Post-MVP: may skip content loading for files above threshold.
- `MimeType` (string, computed) - MIME type detected from file extension or content. Computed using `mime.TypeByExtension(Ext)` or `http.DetectContentType(content)`. Used for file type classification and handling. Examples: "text/markdown", "application/pdf", "image/png".

**Relationships:**

- Used internally by VaultReadAdapter and VaultWriteAdapter to map NoteID ↔ Path
- Never exposed to domain services
- Created during vault scanning by VaultReadAdapter
- Cached in adapters for performance

**Design Decisions:**

- **Adapter-only model:** Domain never sees or depends on filesystem paths - keeps infrastructure concerns isolated
- **Computed fields cached:** Basename/Folder computed once during construction to avoid repeated string operations
- **Staleness detection:** ModTime enables incremental indexing - skip unchanged files
- **Clean separation:** Keeps filesystem concerns out of domain layer
- **Shared by CQRS adapters:** Both VaultReadAdapter and VaultWriteAdapter use this metadata model

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

## VaultFile (Transport DTO)

**Purpose:** Infrastructure data transfer object for vault file scanning. Returns vault-relative file metadata with content for use by VaultReaderAdapter internally.

**Architecture Layer:** SPI Adapter (Transport DTO)

**Location:** `internal/adapters/spi/dto/vault_file.go`

**Rationale:** VaultFile is a lean DTO that delegates to Go stdlib `fs.FileInfo` instead of duplicating fields. Uses vault-relative paths for portability. VaultFile is **internal to adapters** - VaultReaderAdapter constructs Note domain models from VaultFile and returns Notes to application layer.

**Key Attributes:**

- `Path` (string) - **Vault-relative path** with forward slashes (e.g., `"notes/meeting.md"`). Portable across platforms. Normalized using `filepath.ToSlash()`.
- `Info` (fs.FileInfo) - **Delegates to Go stdlib** for ModTime, Size, Mode, IsDir. No duplication.
- `Content` ([]byte) - Raw file content loaded on-demand. For MVP: markdown text from .md files.

**Computed Methods:**

```go
// Basename returns filename without extension
func (v VaultFile) Basename() string {
    base := filepath.Base(v.Path)
    return strings.TrimSuffix(base, filepath.Ext(base))
}

// Folder returns parent directory path
func (v VaultFile) Folder() string {
    return filepath.Dir(v.Path)
}

// Ext returns file extension with dot
func (v VaultFile) Ext() string {
    return filepath.Ext(v.Path)
}

// ModTime delegates to fs.FileInfo
func (v VaultFile) ModTime() time.Time {
    return v.Info.ModTime()
}

// Size delegates to fs.FileInfo
func (v VaultFile) Size() int64 {
    return v.Info.Size()
}

// AbsolutePath helper for I/O operations
func (v VaultFile) AbsolutePath(vaultRoot string) string {
    return filepath.Join(vaultRoot, filepath.FromSlash(v.Path))
}
```

**Path Normalization Helpers:**

```go
// NormalizePath converts absolute path to vault-relative with forward slashes
func NormalizePath(absPath, vaultRoot string) (string, error) {
    relPath, err := filepath.Rel(vaultRoot, absPath)
    if err != nil {
        return "", err
    }
    return filepath.ToSlash(relPath), nil
}

// NewVaultFile creates VaultFile from absolute path and fs.FileInfo
func NewVaultFile(absPath, vaultRoot string, info fs.FileInfo, content []byte) (VaultFile, error) {
    relPath, err := NormalizePath(absPath, vaultRoot)
    if err != nil {
        return VaultFile{}, err
    }

    return VaultFile{
        Path:    relPath,
        Info:    info,
        Content: content,
    }, nil
}
```

**Design Decisions:**

- **fs.FileInfo delegation:** Eliminates 5 duplicated fields (ModTime, Size, Mode, IsDir, Name). Uses well-tested stdlib interface.
- **Vault-relative paths:** Enables cache portability across machines. Matches Obsidian pattern. Forward slashes for cross-platform consistency.
- **Computed methods:** No cached fields. Methods compute on-demand from Path. Minimal memory footprint.
- **Internal to adapters:** VaultFile never exposed to application layer. VaultReaderAdapter uses it internally, constructs Note domain models, returns Notes.
- **Transport DTO (adapter layer):** Not a domain model. Pure infrastructure data transfer within adapter layer.

**Adapter Workflow:**

```go
// VaultReaderAdapter (internal/adapters/spi/vault/reader.go)
func (a *VaultReaderAdapter) ScanAll(ctx context.Context) ([]domain.Note, error) {
    // 1. Scan filesystem → []VaultFile (internal DTO)
    vaultFiles := a.scanFilesystem()

    // 2. Parse each VaultFile → Note
    notes := make([]domain.Note, 0, len(vaultFiles))
    for _, vf := range vaultFiles {
        // Parse markdown → NoteMetadata
        metadata, err := a.markdownParser.ParseMetadata(ctx, vf.Content)

        // Construct Note from metadata + path
        note := domain.NewNote(vf.Path, metadata)
        notes = append(notes, note)
    }

    // 3. Return domain models (not DTOs)
    return notes, nil
}
```

**Relationships:**

- Used internally by VaultReaderAdapter during filesystem scanning
- Converted to Note domain models before returning to application layer
- Path used as Note identifier (no separate NoteID abstraction)
- Content parsed by MarkdownParserAdapter to extract frontmatter, links, headings, tasks

---

## FrontmatterDTO (Transport DTO)

**Purpose:** Adapter-layer data transfer object for frontmatter parsing. Provides syntactic validation (YAML structure) before converting to domain.Frontmatter for semantic validation (schema compliance).

**Architecture Layer:** SPI Adapter (Transport DTO)

**Location:** `internal/adapters/spi/vault/frontmatter.go`

**Rationale:** Two-layer validation pattern separates infrastructure concerns (YAML parsing) from domain concerns (schema compliance). MarkdownParserAdapter handles syntactic validation using FrontmatterDTO, then converts to domain.Frontmatter for business validation by FrontmatterService.

**Key Attributes:**

- `Fields` map[string]any - Parsed YAML frontmatter from markdown file

**Adapter-Layer Validation:**

```go
// ValidateSyntax checks YAML structure without schema knowledge
// Returns error if:
// - YAML parsing failed
// - Frontmatter is not a map (must be key-value pairs)
// - Fields contain unsupported types (functions, channels, etc.)
func (dto FrontmatterDTO) ValidateSyntax() error {
    if dto.Fields == nil {
        return fmt.Errorf("frontmatter fields are nil")
    }

    // Validate no unsupported types in fields
    for key, val := range dto.Fields {
        if !isSupportedType(val) {
            return fmt.Errorf("field %s has unsupported type", key)
        }
    }

    return nil
}

// isSupportedType checks if value type is allowed in frontmatter
// Allowed: string, int, int64, float64, bool, []any, []string, map[string]any
func isSupportedType(val any) bool {
    switch val.(type) {
    case string, int, int64, float64, bool:
        return true
    case []any, []string, []interface{}:
        return true
    case map[string]any, map[string]interface{}:
        return true
    default:
        return false
    }
}
```

**Conversion to Domain:**

```go
// ToDomain converts FrontmatterDTO to domain.Frontmatter
// Called by MarkdownParserAdapter after syntactic validation
func (dto FrontmatterDTO) ToDomain() domain.Frontmatter {
    return domain.NewFrontmatter(dto.Fields)
}
```

**Usage in MarkdownParserAdapter:**

```go
// MarkdownParserAdapter (internal/adapters/spi/vault/markdown_parser.go)
func (a *MarkdownParserAdapter) ParseMetadata(ctx context.Context, content []byte) (NoteMetadata, error) {
    // 1. Parse YAML frontmatter → FrontmatterDTO
    dto, err := a.parseYAML(content)
    if err != nil {
        return NoteMetadata{}, fmt.Errorf("YAML parse error: %w", err)
    }

    // 2. Syntactic validation (adapter layer)
    if err := dto.ValidateSyntax(); err != nil {
        return NoteMetadata{}, fmt.Errorf("frontmatter syntax invalid: %w", err)
    }

    // 3. Convert to domain model
    frontmatter := dto.ToDomain()

    // 4. Parse other metadata (links, headings, tasks)
    links := a.parseLinks(content)
    headings := a.parseHeadings(content)
    tasks := a.parseTasks(content)

    return NoteMetadata{
        Frontmatter: frontmatter,
        Links:       links,
        Headings:    headings,
        Tasks:       tasks,
    }, nil
}
```

**Design Decisions:**

- **Syntactic vs Semantic separation:** FrontmatterDTO handles YAML parsing concerns (adapter layer). domain.Frontmatter validated by FrontmatterService for schema compliance (application layer).
- **Simple DTO:** No methods beyond ValidateSyntax() and ToDomain(). No business logic.
- **Type safety:** ValidateSyntax() ensures only supported YAML types in frontmatter (prevents issues with complex types).
- **One-way conversion:** FrontmatterDTO → domain.Frontmatter (no reverse). Domain models don't depend on adapter DTOs.
- **Error context:** Validation errors indicate YAML structural issues, not schema violations.

**Relationships:**

- Created by MarkdownParserAdapter.parseYAML() from markdown content
- Validated via ValidateSyntax() before domain conversion
- Converted to domain.Frontmatter via ToDomain()
- Never exposed outside MarkdownParserAdapter (internal DTO)

---

## Note (Domain Entity)

**Purpose:** Core business entity representing a markdown note. Aggregate root combining identity (path), frontmatter, and structural metadata (links, headings, tasks).

**Architecture Layer:** Domain Core

**Location:** `internal/domain/note.go`

**Key Attributes:**

- `Path` (string) - **Vault-relative path as identifier** (e.g., `"notes/meeting.md"`). Unique, immutable primary key. Replaces abstract NoteID.
- `Frontmatter` (Frontmatter) - Content metadata from YAML frontmatter
- `Links` ([]Link) - All links found in markdown (wikilinks + standard links)
- `Headings` ([]Heading) - Markdown heading structure with hierarchy
- `Tags` ([]string) - Hashtags extracted from content (#tag, #nested/tag)
- `Tasks` ([]TaskItem) - Task list items with completion status

**Entity Methods (Enrichment):**

```go
// Validation
func (n Note) Validate(ctx context.Context) error {
    // Semantic validation - schema compliance, business rules
    return n.Frontmatter.Validate(ctx)
}

// Factory constructor with validation
func NewNote(path string, frontmatter Frontmatter, links []Link, headings []Heading, tags []string, tasks []TaskItem) (Note, error) {
    note := Note{
        Path:        path,
        Frontmatter: frontmatter,
        Links:       links,
        Headings:    headings,
        Tags:        tags,
        Tasks:       tasks,
    }

    if err := note.Validate(context.Background()); err != nil {
        return Note{}, err
    }

    return note, nil
}

// Delegation methods for common frontmatter access
func (n Note) FileClass() string {
    return n.Frontmatter.FileClass
}

func (n Note) Aliases() []string {
    return n.Frontmatter.GetStringSlice("aliases")
}

func (n Note) Title() string {
    title, _ := n.Frontmatter.GetString("title")
    return title
}

func (n Note) HasTag(tag string) bool {
    for _, t := range n.Tags {
        if t == tag {
            return true
        }
    }
    return false
}

// Extraction methods for storage adapters
func (n Note) ModTime() time.Time {
    // Storage adapters track ModTime separately (not in domain)
    // See FileDatesDTO for staleness detection
    return time.Time{}
}
```

**Relationships:**

- Constructed by VaultReaderAdapter from VaultFile + MarkdownParserAdapter
- Stored via CacheWriterPort (BoltDB + SQLite dual-write)
- Retrieved via CacheReaderPort
- Queried by QueryService (template engine's lookup/query functions)
- Path used as unique identifier across all storage (no NoteID abstraction needed)

**Design Decisions:**

- **Path as identifier:** Vault-relative path IS the identity. No abstract NoteID. If file moves, treat as delete+create for MVP.
- **NoteMetadata attributes embedded:** Frontmatter, Links, Headings, Tags, Tasks directly on Note (not separate NoteMetadata wrapper).
- **Rich domain entity:** Has validation, factory, delegation methods. NOT anemic.
- **Storage extraction via methods:** Note provides FileClass(), Aliases(), Title() for storage adapters to extract what they need.
- **Single model for MVP:** Used by both write and read operations. CQRS separation in operations/ports, not models.
- **Aggregate root:** Note is DDD aggregate root. Frontmatter, Link, Heading, TaskItem are value objects.

**Additional Information:**

Note construction happens in **VaultReaderAdapter** (adapter layer), not VaultIndexer (application layer). Adapter parses markdown via MarkdownParserAdapter, constructs fully-formed Note domain models, returns them to VaultIndexer for validation and persistence.

---

### NoteID (REMOVED - Replaced by Path)

**Status:** REMOVED in architectural course correction (2025-11-05)

**Reason for Removal:** Over-abstraction without benefit. NoteID was opaque identifier requiring translation layer, but vault structure inherently uses paths as natural identifiers.

**Replaced By:** Note.Path (vault-relative path string)

**Migration:**

- **Before:** NoteID abstraction with adapter translation (NoteID ↔ file path mapping)
- **After:** Path as direct identifier (e.g., `"notes/meeting.md"`)

**Benefits of Path as Identifier:**

- **Simplicity:** No translation layer needed - path is natural vault identifier
- **Portability:** Vault-relative paths work across machines when vault synced
- **Human-readable:** `"notes/daily/2024-01-15.md"` more meaningful than opaque UUID
- **Storage alignment:** BoltDB, SQLite, filesystem all use paths naturally
- **Query simplicity:** Path-based lookups match user mental model

**Impact:**

- Removed NoteID type and translation logic from adapters
- Note identified directly by Path field (string)
- Storage adapters use Path as primary key
- No mapping tables needed (simplified architecture)

---

### NoteMetadata (EMBEDDED IN NOTE)

**Status:** Attributes embedded directly in Note entity (architectural course correction 2025-11-05)

**Previous Design:** Separate NoteMetadata container returned by parser

**Current Design:** Metadata attributes embedded as Note entity fields

**Note Structure with Embedded Metadata:**

```go
type Note struct {
    // Identity
    Path string  // Vault-relative path as identifier

    // Parsed metadata (from MarkdownParserAdapter)
    Frontmatter Frontmatter
    Links       []Link
    Headings    []Heading
    Tags        []string
    Tasks       []TaskItem

    // Computed metadata (populated after Note construction)
    Backlinks   []Link  // Computed by analyzing Links across vault
}
```

**Two-Phase Note Population:**

**Phase 1 - Construction (Parsing):**

```go
// MarkdownParserAdapter parses markdown and constructs Note
func (a *MarkdownParserAdapter) ParseNote(path string, content []byte) (domain.Note, error) {
    // Parse all metadata from goldmark AST (single pass)
    frontmatter := a.parseFrontmatter(content)
    links := a.parseLinks(content)
    headings := a.parseHeadings(content)
    tags := a.parseTags(content)
    tasks := a.parseTasks(content)

    // Construct Note with parsed attributes (Backlinks empty)
    note := domain.NewNote(path, frontmatter, links, headings, tags, tasks)
    return note, nil
}
```

**Phase 2 - Enrichment (Computation):**

```go
// BacklinkService computes backlinks after all notes parsed
func (s *BacklinkService) ComputeBacklinks(notes []domain.Note) {
    for i := range notes {
        // Analyze Links from all other notes
        backlinks := s.findReferencesToNote(notes[i].Path, notes)

        // Populate Backlinks field (mutation after construction)
        notes[i].Backlinks = backlinks
    }
}
```

**Design Decisions:**

- **Embedded attributes:** Note directly contains metadata fields (no wrapper structure)
- **Entity enrichment:** Note has delegation methods using these attributes (FileClass(), Title(), Aliases())
- **Two-phase population:** Parsed metadata at construction, computed metadata (Backlinks) added later
- **Mutable for Backlinks (MVP):** Note allows Backlinks mutation after construction for MVP simplicity
- **Single-pass parsing:** MarkdownParserAdapter extracts all parsed metadata in one goldmark AST walk
- **Backlinks computed:** Requires vault-wide Link analysis, populated by BacklinkService post-indexing

**Benefits:**

- **Simplified access:** `note.Links` instead of `note.Metadata.Links`
- **Rich entity:** Note provides domain methods using embedded attributes
- **No intermediate wrapper:** Direct attribute access
- **Clear separation:** Parsed attributes vs computed attributes

**Relationships:**

- Note constructed by MarkdownParserAdapter with parsed attributes
- Backlinks populated by BacklinkService after vault-wide analysis
- All attributes stored together in Note entity
- Services access note attributes directly

**Additional Information:**

Entity enrichment pattern eliminates intermediate NoteMetadata wrapper. Note entity directly owns its attributes and provides delegation methods. For MVP, Note permits Backlinks mutation after construction (pragmatic choice). Post-MVP may refactor to immutable Note with BacklinkEnrichment pattern if needed.

---

#### Frontmatter

**Purpose:** Rich domain entity representing note metadata with type-safe accessors for field inspection and delegation methods for common frontmatter values.

**Architecture Layer:** Domain Core (`internal/domain/frontmatter.go`)

**Key Attributes:**

- `Fields` map[string]any - Complete parsed YAML frontmatter preserving all user-defined fields

**Entity Enrichment:**

Frontmatter transitions from anemic data structure to rich domain entity with:

1. **Generic Field Access:** Get() returns raw values, caller handles type assertions
2. **Field Inspection:** Has() checks existence, type checkers (IsString, IsArray, etc.) validate types
3. **Delegation Methods:** FileClass(), Title(), Aliases() provide convenient access for Note
4. **Config Dependency:** FileClass() uses global Config.FileClassKey singleton

**Relationships:**

- Parsed from FrontmatterDTO by MarkdownParserAdapter (syntactic validation in adapter)
- Embedded in Note entity (composed aggregate)
- Validated by FrontmatterService.Validate() in application layer (semantic validation)
- Used by Note delegation methods (Note.FileClass(), Note.Aliases(), Note.Title())

**Design Decisions:**

- **No FileClass caching:** Computed on-demand via FileClass() method
- **Config dependency acceptable:** Frontmatter depends on Config singleton (loaded before all other components)
- **Two-layer validation:**
  - Adapter layer: FrontmatterDTO.ValidateSyntax() - YAML parsing, structure
  - Application layer: FrontmatterService.Validate() - Schema compliance (not in domain entity)
- **Flexible map preserved:** map[string]any supports schema-free notes and unknown fields (FR6)
- **Caller-side type safety:** Get() returns any; caller uses type checkers then type assertions
- **Fields vs Properties:** "Fields" = data values, "Properties" = schema definitions

**Field Access Methods:**

```go
// Get retrieves raw field value - caller handles type assertions
func (f Frontmatter) Get(key string) (any, bool) {
    val, ok := f.Fields[key]
    return val, ok
}

// Has checks field existence
func (f Frontmatter) Has(key string) bool {
    _, ok := f.Fields[key]
    return ok
}
```

**Type Checker Methods:**

```go
// IsString checks if field is string type
func (f Frontmatter) IsString(key string) bool {
    val, ok := f.Fields[key]
    if !ok {
        return false
    }
    _, ok = val.(string)
    return ok
}

// IsArray checks if field is array/slice type
func (f Frontmatter) IsArray(key string) bool {
    val, ok := f.Fields[key]
    if !ok {
        return false
    }
    switch val.(type) {
    case []any, []string, []interface{}:
        return true
    default:
        return false
    }
}

// IsInt checks if field is integer type (handles YAML number parsing)
func (f Frontmatter) IsInt(key string) bool {
    val, ok := f.Fields[key]
    if !ok {
        return false
    }
    switch val.(type) {
    case int, int64, float64:
        return true
    default:
        return false
    }
}

// IsBool checks if field is boolean type
func (f Frontmatter) IsBool(key string) bool {
    val, ok := f.Fields[key]
    if !ok {
        return false
    }
    _, ok = val.(bool)
    return ok
}

// IsMap checks if field is map type
func (f Frontmatter) IsMap(key string) bool {
    val, ok := f.Fields[key]
    if !ok {
        return false
    }
    _, ok = val.(map[string]any)
    return ok
}
```

**Delegation Methods for Note:**

```go
// FileClass retrieves schema key using Config.FileClassKey
// Accesses global Config singleton
func (f Frontmatter) FileClass() string {
    key := config.Get().FileClassKey
    if val, ok := f.Fields[key].(string); ok {
        return val
    }
    return ""
}

// Title retrieves note title from "title" field
func (f Frontmatter) Title() string {
    if val, ok := f.Fields["title"].(string); ok {
        return val
    }
    return ""
}

// Aliases retrieves note aliases from "aliases" field
// Handles []string, []any with string elements, single string -> []string
func (f Frontmatter) Aliases() []string {
    val, ok := f.Fields["aliases"]
    if !ok {
        return []string{}
    }

    // Handle []string
    if strSlice, ok := val.([]string); ok {
        return strSlice
    }

    // Handle []any or []interface{}
    if anySlice, ok := val.([]any); ok {
        result := make([]string, 0, len(anySlice))
        for _, item := range anySlice {
            if str, ok := item.(string); ok {
                result = append(result, str)
            }
        }
        return result
    }

    // Single string -> slice
    if str, ok := val.(string); ok {
        return []string{str}
    }

    return []string{}
}
```

**Factory Constructor:**

```go
// NewFrontmatter creates Frontmatter from parsed fields
// Called by MarkdownParserAdapter after converting FrontmatterDTO
func NewFrontmatter(fields map[string]any) Frontmatter {
    return Frontmatter{Fields: fields}
}
```

**Usage Example:**

```go
// Type-safe field access workflow
if fm.IsString("author") {
    val, _ := fm.Get("author")
    author := val.(string)  // Safe: type already checked
    // Use author
}

// Delegation methods
fileClass := note.Frontmatter.FileClass()  // Uses Config.FileClassKey
title := note.Frontmatter.Title()
aliases := note.Frontmatter.Aliases()
```

---

#### Link

**Purpose:** Represents a link found in markdown content. Captures both wikilinks (`[[target]]`) and standard markdown links (`[text](url)`). Used for backlink computation, graph analysis, and link validation.

**Architecture Layer:** Domain Core (Value Object)

**Key Attributes:**

- `Text` (string) - Display text for the link. For wikilinks without alias: same as Destination. For aliased wikilinks `[[target|alias]]`: alias text. For markdown links `[text](url)`: text portion.
- `Destination` (string) - Link target. For wikilinks: note basename or path. For markdown links: URL or relative path. For external links: full URL.
- `IsWikilink` (bool) - True if wikilink format `[[...]]`, false if markdown format `[...](...)`. Enables different resolution strategies.

**Relationships:**

- Component of NoteMetadata ([]Link field)
- Used by future LinkService for wikilink resolution and link validation
- Enables backlink computation (Link.Destination → find notes with matching path)
- Foundation for graph queries and knowledge graph visualization

**Design Decisions:**

- **Value object:** Immutable link representation. Two Link instances with identical attributes are equivalent.
- **Unified link model:** Single structure for both wikilinks and markdown links. IsWikilink flag enables different handling.
- **No link resolution in model:** Link stores raw destination as found in markdown. Resolution (basename → full path) happens in services.
- **Text vs Destination:** Separate fields enable aliased wikilinks `[[note|display text]]` and markdown links `[text](url)` with different text/destination.

**Helper Functions:**

```go
// NewWikilink creates Link from wikilink syntax
// Example: [[meeting notes]] or [[notes/meeting|Meeting]]
func NewWikilink(destination, text string) Link {
    if text == "" {
        text = destination // No alias, text same as destination
    }
    return Link{
        Text:        text,
        Destination: destination,
        IsWikilink:  true,
    }
}

// NewMarkdownLink creates Link from markdown link syntax
// Example: [Obsidian](https://obsidian.md) or [README](./README.md)
func NewMarkdownLink(text, destination string) Link {
    return Link{
        Text:        text,
        Destination: destination,
        IsWikilink:  false,
    }
}
```

**Additional Information:**

Link model enables Obsidian-style wikilink features: `[[note]]` links to note by basename, `[[folder/note|alias]]` links with display alias, backlinks computed by finding all notes linking to target. Markdown links `[text](url)` also supported for external references and relative paths. Future features: link validation (check destination exists), orphaned note detection (no incoming links), broken link detection (destination not found), graph visualization (nodes=notes, edges=links).

---

#### Heading

**Purpose:** Represents a markdown heading with level and text. Enables navigation, table of contents generation, and heading-based queries.

**Architecture Layer:** Domain Core (Value Object)

**Key Attributes:**

- `Level` (int) - Heading level (1-6). Corresponds to markdown `#` count: `# Title` = 1, `## Section` = 2, etc. Enables hierarchy detection.
- `Text` (string) - Heading text without `#` markers. Used for display, search, and navigation. Trimmed of leading/trailing whitespace.

**Relationships:**

- Component of NoteMetadata ([]Heading field)
- Used by future HeadingNavigationService for outline generation
- Enables heading-based queries via MetadataQueryPort.QueryByHeading()
- Foundation for document outline and table of contents features

**Design Decisions:**

- **Value object:** Immutable heading representation. Two Heading instances with identical attributes are equivalent.
- **Simple structure:** Just level and text - no position, no nesting. Hierarchy computed from level sequence if needed.
- **Level as int:** 1-6 per markdown spec. Invalid levels (0, 7+) rejected during parsing.
- **No anchor IDs:** Heading doesn't store anchor ID (`# Title {#custom-id}`). Future enhancement if anchor links needed.

**Helper Functions:**

```go
// NewHeading creates Heading from level and text
// Called by MarkdownParserAdapter during AST walking
func NewHeading(level int, text string) (Heading, error) {
    if level < 1 || level > 6 {
        return Heading{}, fmt.Errorf("invalid heading level: %d (must be 1-6)", level)
    }
    return Heading{
        Level: level,
        Text:  strings.TrimSpace(text),
    }, nil
}

// IsTopLevel returns true if heading is level 1 (# Title)
func (h Heading) IsTopLevel() bool {
    return h.Level == 1
}
```

**Additional Information:**

Heading model enables document structure analysis and navigation. Future features: table of contents generation (render heading hierarchy), heading-based navigation (jump to section), outline view (collapsible heading tree), heading search (find notes with specific sections). Goldmark AST provides heading information during parsing - MarkdownParserAdapter extracts into Heading structs. Heading sequence represents document structure: increasing levels = going deeper, decreasing levels = coming back up.

---

#### TaskItem

**Purpose:** Represents a task/checkbox item found in markdown content. Records task text and completion status for task management queries.

**Architecture Layer:** Domain Core (Value Object)

**Key Attributes:**

- `Text` string - Task description extracted from TextBlock node (without checkbox markers)
- `IsChecked` bool - Task completion status from TaskCheckBox.IsChecked (true = `[x]`, false = `[ ]`)
- `Line` int - Line number in source markdown for reference

**Relationships:**

- Component of Note entity ([]TaskItem field)
- Extracted by MarkdownParserAdapter using goldmark TaskList extension
- Used by future TaskService for task queries

**Design Decisions:**

- **Value object (MVP):** Simple immutable data structure - no manipulation methods
- **goldmark TaskList extension:** Uses goldmark's `ast.TaskCheckBox` node with `IsChecked` boolean
- **Text from parent node:** Task text comes from parent `TextBlock` node, not TaskCheckBox itself
- **Line number tracking:** Enables future task updates (find task by line)
- **Binary status (MVP):** IsChecked bool handles `[x]` (true) and `[ ]` (false) only
- **Post-MVP evolution:** Will support custom task statuses via Config (e.g., `[-]` in-progress, `[>]` deferred). Design will evolve from IsChecked bool to Status field with config-driven status definitions.

**Parsing Strategy:**

```go
// MarkdownParserAdapter walks goldmark AST to extract tasks
// 1. Find ast.TaskCheckBox nodes
// 2. Get IsChecked boolean from TaskCheckBox
// 3. Walk parent TextBlock to extract task text
// 4. Get line number from node position
// 5. Return []domain.TaskItem (no goldmark dependency in domain)
```

**Factory Constructor (MVP):**

```go
// NewTaskItem creates TaskItem from parsed values
// Called by MarkdownParserAdapter during AST walking
func NewTaskItem(text string, isChecked bool, line int) TaskItem {
    return TaskItem{
        Text:      strings.TrimSpace(text),
        IsChecked: isChecked,
        Line:      line,
    }
}
```

**Additional Information:**

TaskItem model enables basic task tracking for MVP. goldmark TaskList extension provides `ast.TaskCheckBox` nodes; adapter extracts to domain model without goldmark dependency. Post-MVP will expand to support custom task statuses (in-progress, deferred, cancelled) via Config.TaskStatuses settings, evolving IsChecked bool to Status string field.

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
- Inheritance resolved by SchemaExtender adapter
- Structural validation via Schema.Validate() called by SchemaValidator

**Design Decisions:**

- **Rich domain model:** Contains structural validation behavior via Validate() method. No external dependencies - pure domain logic checking structure.
- **Inheritance in source form:** Schema stores original Extends/Excludes/Properties from JSON. SchemaResolver service resolves inheritance and provides flattened properties to FrontmatterService.
- **Resolution details:** SchemaResolver now uses name-keyed maps to merge overrides and hydrates PropertyBank references within the same pass (no secondary substitution step).
- **Properties vs Fields terminology:** Schema has "Properties" (validation rules). Frontmatter has "Fields" (actual data).
- **Excludes dependent on Extends:** Excludes only meaningful when Extends is not empty.
- **String-based Extends reference:** Uses schema name string, not Go pointer, to avoid circular dependency issues in struct definitions. Schema registry (map[string]\*Schema) resolves references after all schemas loaded.
- **Eager resolution at startup:** Inheritance chains resolved during application initialization (fail-fast on circular dependencies per Epic 2, Story 2.6). Validator never sees unresolved schemas. Performance: O(n\*d) where n=schemas, d=depth, acceptable for MVP (<100 schemas expected).
- **Resolution order:** (1) Load all schema files, (2) Build dependency graph, (3) Detect cycles, (4) Resolve in topological order (leaves first), (5) For each schema: get parent's ResolvedProperties → apply Excludes → merge/override with child Properties → store in ResolvedProperties.
- **Property override semantics:** If child Property.Name matches parent Property.Name, child completely replaces parent (not merging property attributes). This is explicit override, not attribute-level merge.
- **Immutability:** Schema instances are immutable after construction. Properties and Excludes slices are defensively copied during creation to prevent external modification.
- **JSON/YAML Serialization:** Schemas serialize as JSON or YAML objects with name, extends (optional), excludes (optional), and properties array. ResolvedProperties is omitted from serialization (computed field).

**JSON/YAML Format Example:**

```json
{
  "name": "contact",
  "extends": "base-note",
  "excludes": ["internal_id"],
  "properties": [
    { "$ref": "#/properties/standard_title" },
    { "$ref": "#/properties/standard_created" },
    {
      "name": "email",
      "required": true,
      "array": false,
      "spec": {
        "pattern": "^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"
      }
    }
  ]
}
```

**Additional Information:**

Schema inheritance provides powerful reusability for similar note types. For example, a base "note" schema could define common properties (title, tags, created), while specialized schemas like "meeting_note" or "person" extend the base and add domain-specific properties. The eager resolution strategy ensures validation is fast (no runtime resolution overhead) at the cost of slightly longer startup time. For MVP with <100 schemas, this tradeoff is acceptable. The Builder pattern isolates complexity—domain validators simply receive fully-resolved schemas and don't need to understand inheritance mechanics.

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
- **Immutability:** PropertyBank instances are immutable after construction. Properties map is defensively copied during creation to prevent external modification.
- **JSON/YAML Serialization:** PropertyBank serializes as JSON object with single "properties" field containing the property map. No YAML support (JSON-only for MVP).

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

### Property

**Purpose:** Defines a single metadata field with validation constraints. Building block of Schema definitions. Rich domain model with structural validation behavior.

**Architecture Layer:** Domain Core (Rich Domain Model)

**Key Attributes:**

- `ID` (string) - Unique identifier for this property entity, generated using hash of (Name + Spec content) for deterministic identity.
- `Name` (string) - Property identifier matching frontmatter key. Case-sensitive.
- `Required` (bool) - Whether property must be present. Empty array satisfies required for array properties.
- `Array` (bool) - Whether property accepts multiple values (YAML list) vs single scalar value.
- `Spec` (PropertySpec) - Type-specific validation constraints (interface for polymorphism).

**Key Methods:**

- `Validate(ctx context.Context) error` - Validates property structure (Name not empty, Spec valid). Delegates PropertySpec validation to Spec.Validate().
- `InPropertyBank(bank PropertyBank) bool` - Checks if this property exists in the given PropertyBank by ID comparison.

**Relationships:**

- Belongs to Schema (composition)
- Contains one PropertySpec implementation (no more $ref in domain layer)
- Used by FrontmatterService to validate Frontmatter.Fields
- Structural validation via Property.Validate() called by Schema.Validate()

**Design Decisions:**

- **DDD Entity:** Property is now a domain entity with identity (ID field) rather than a value object. ID enables reliable membership checking in PropertyBank.
- **Hash-based Identity:** ID generated from hash of (Name + Spec content) ensures deterministic, reproducible identity for the same property definition.
- **No more $ref in domain:** PropertyBank references resolved at infrastructure layer (adapter), domain works with resolved Property entities only.
- **Simplified Structure:** Removed Ref field and IProperty interface - all properties now have inline Spec, resolved by infrastructure layer.
- **Entity vs Value Object:** Properties are entities because they need identity for PropertyBank membership checking, unlike PropertySpec which remains a value object.
- **Immutability:** Property instances are immutable entities. Created via constructor validation, never modified after creation.
- **JSON/YAML Serialization:** Properties serialize as JSON objects with id, name, required, array, and spec fields.

**JSON/YAML Format Examples:**

```json
// Property entity with inline spec
{
  "id": "a1b2c3d4...",
  "name": "email",
  "required": true,
  "array": false,
  "spec": {
    "type": "string",
    "pattern": "^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"
  }
}

// Property entity with different spec type
{
  "id": "e5f6g7h8...",
  "name": "tags",
  "required": false,
  "array": true,
  "spec": {
    "type": "string"
  }
}
```

---

### PropertySpec (Type-Specific Configurations)

**Purpose:** Interface for type-specific validation constraint definitions. Defines what constraints apply to a property (min/max, patterns, enums) as immutable value objects with structural validation behavior. Each PropertySpec variant validates its own constraint structure.

**Architecture Layer:** Domain Core (Value Objects with Behavior)

**Rationale:** PropertySpec variants are DDD value objects—immutable constraint definitions identified by their attributes, not by identity. They define constraint data (e.g., "min: 0, max: 100") AND validate constraint structure (e.g., regex pattern is valid). This leverages polymorphism—each PropertySpec type knows how to validate its own constraints.

**Key Methods (Interface):**

- `Type() PropertySpecType` - Returns property type identifier (string, number, date, file, boolean)
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
- **Immutability:** All PropertySpec variants are immutable after construction. No setters or modification methods.
- **JSON/YAML Serialization:** Each PropertySpec variant serializes as JSON/YAML object with type-specific fields. Interface is resolved via discriminator pattern during unmarshaling.

---

#### StringSpec

**Purpose:** Defines string validation constraints (allowed values, patterns) as immutable value object with structural validation.

**Key Attributes:**

- `Enum` ([]string, optional) - Allowed values as fixed list. If non-empty, value must be in list (exact match, case-sensitive). Empty list means no values allowed, nil means any string valid.
- `Pattern` (string, optional) - Regex pattern for custom validation. If non-empty, value must match pattern. Uses Go `regexp` package. Empty string or nil means no pattern constraint.

**Key Methods:**

- `Type() PropertySpecType` - Returns `PropertyTypeString`
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

#### NumberSpec

**Purpose:** Defines numeric validation constraints (min/max bounds, step increments) as immutable value object with structural validation.

**Key Attributes:**

- `Min` (\*float64, optional) - Minimum allowed value (inclusive). Nullable pointer distinguishes "not set" from "0". If set, value must be >= Min.
- `Max` (\*float64, optional) - Maximum allowed value (inclusive). If set, value must be <= Max.
- `Step` (\*float64, optional) - Increment/decrement amount. If 1.0, implies integer values. If 0.1, implies one decimal precision. If nil, any precision allowed.

**Key Methods:**

- `Type() PropertySpecType` - Returns `PropertyTypeNumber`
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

#### DateSpec

**Purpose:** Defines date/time format constraints as immutable value object with structural validation.

**Key Attributes:**

- `Format` (string) - Go time layout string (e.g., "2006-01-02", "2006-01-02T15:04:05Z07:00"). Uses Go stdlib `time.Parse(format, value)`. If empty, defaults to RFC3339.

**Key Methods:**

- `Type() PropertySpecType` - Returns `PropertyTypeDate`
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

#### FileSpec

**Purpose:** Defines file reference validation constraints (fileClass filters, directory filters) as immutable value object with structural validation.

**Key Attributes:**

- `FileClass` (string, optional) - Restricts valid file references to notes with specific fileClass value or regex pattern. Supports negation via `^` prefix. Examples: `"project"` (exact match), `"^archive"` (NOT archive), `"(project|task)"` (regex: project OR task). Empty string or nil means no fileClass restriction.
- `Directory` (string, optional) - Restricts valid file references to notes within specific vault directory path. Path is relative to vault root. Supports negation via `^` prefix. Examples: `"projects/"` (notes in projects/), `"^archive/"` (NOT in archive/), `"work/.*"` (regex: anything under work/). Empty string or nil means no directory restriction.

**Key Methods:**

- `Type() PropertySpecType` - Returns `PropertyTypeFile`
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

#### BoolSpec

**Purpose:** Defines boolean validation (no additional constraints). Marker value object with no structural validation needed.

**Key Attributes:**

- None. Presence of BoolSpec indicates property accepts boolean values only.

**Key Methods:**

- `Type() PropertySpecType` - Returns `PropertyTypeBool`
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

## Template

**Purpose:** Domain interface for executable note generation templates. Wraps Go's text/template to provide domain-aligned template identity and execution behavior.

**Architecture Layer:** Domain Core (Interface)

**Location:** `internal/domain/template.go`

**Rationale:** Template transitions from anemic struct to interface wrapping \*template.Template. This enables rich domain behavior (ID, Execute) while delegating parsing and rendering to stdlib. Interface provides domain abstraction and testability.

**Interface Definition:**

```go
// Template represents an executable note generation template
type Template interface {
    // ID returns template identifier for composition and lookup
    ID() string

    // Execute renders template with provided data context
    Execute(data any) (string, error)
}
```

**Concrete Implementation (GoTemplate):**

```go
// GoTemplate wraps *template.Template for domain interface compliance
// Location: internal/domain/template.go
type GoTemplate struct {
    id   string
    tmpl *template.Template  // Wraps stdlib template
}

// NewGoTemplate creates Template from parsed *template.Template
// Called by TemplateEngine application service after parsing
func NewGoTemplate(id string, tmpl *template.Template) Template {
    return &GoTemplate{
        id:   id,
        tmpl: tmpl,
    }
}

func (t *GoTemplate) ID() string {
    return t.id
}

func (t *GoTemplate) Execute(data any) (string, error) {
    var buf strings.Builder
    if err := t.tmpl.Execute(&buf, data); err != nil {
        return "", err
    }
    return buf.String(), nil
}
```

**Relationships:**

- Template interface implemented by GoTemplate domain model
- Wraps \*template.Template from Go stdlib (delegation pattern)
- Created by TemplateEngine application service after parsing template files
- Templates may reference other templates via `{{template "name"}}` composition (resolved by stdlib)
- Executed with data context (note metadata, config values, query results)
- Template ID used for lookup and composition within text/template namespace

**Design Decisions:**

- **Interface over struct:** Template is interface, not anemic data bag. Encapsulates identity and execution behavior.
- **Composition wrapping:** GoTemplate wraps \*template.Template, delegates parsing/rendering to stdlib.
- **Domain abstraction:** Interface provides domain-aligned API while hiding text/template implementation details.
- **Testability:** Interface enables mocking Template for unit tests without filesystem dependencies.
- **ID as domain concept:** Template ID is intrinsic domain requirement for `{{template "name"}}` composition, not infrastructure leakage.
- **Immutable after creation:** Template wraps parsed \*template.Template (read-only), no mutation after construction.
- **Thread-safe execution:** \*template.Template is safe for concurrent Execute calls.
- **No custom caching:** Leverage text/template's built-in template association and lookup.

**Template Composition Example:**

```go
// Domain usage: Templates reference each other via ID
// templates/header.tmpl defines "header" template
// templates/daily-note.tmpl uses {{template "header" .}}

// TemplateEngine (application service) loads both into namespace
// Client executes by ID
template, _ := engine.GetTemplate("daily-note")
output, _ := template.Execute(data)
```

**Benefits:**

- **Rich domain model:** Template has behavior (Execute), not just data
- **Stdlib leverage:** Delegates to mature, tested text/template implementation
- **Type safety:** Interface provides compile-time guarantees
- **Domain focus:** Hides parsing/caching concerns from domain consumers
- **Mockable:** Interface enables test doubles without filesystem I/O

**Additional Information:**

Template interface provides domain abstraction over text/template stdlib. GoTemplate is thin wrapper that delegates execution to \*template.Template while providing domain-aligned interface. Template parsing, namespace management, and function registration handled by TemplateEngine application service (see components.md). Template composition (`{{template "name"}}`) resolved by text/template stdlib namespace.

---

### TemplateID

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

## Config

**Purpose:** Application configuration loaded from `lithos.json` and environment variables. Defines vault structure and operational settings. Immutable value object representing application configuration state.

**Architecture Layer:** Domain Core (Value Object)

**Rationale:** Config is a DDD value object—immutable configuration data identified by its attributes. While loaded by ConfigLoader adapter, the Config model itself represents domain knowledge about vault structure (where templates, schemas, property bank live). Domain services receive Config via dependency injection to locate resources.

**Key Attributes:**

- `VaultPath` (string) - Root directory of vault. Default: current working directory. All relative paths in config are resolved relative to this. Must exist and be readable. ConfigLoader searches upward from current directory to find `lithos.json`, then uses that directory as VaultPath.
- `TemplatesDir` (string) - Path to templates directory. Default: `{VaultPath}/templates/`. Can be absolute or relative to VaultPath. Must exist for `lithos new` and `lithos find` commands. TemplateLoader scans all `.md` files in this directory.
- `SchemasDir` (string) - Path to schemas directory. Default: `{VaultPath}/schemas/`. Can be absolute or relative to VaultPath. Must exist if schemas are used. SchemaLoader parses all schema JSON files in this directory at startup.
- `PropertyBankFile` (string) - Filename of property bank file within SchemasDir. Default: `property_bank.json`. Full path is `{SchemasDir}/{PropertyBankFile}`. Optional—if missing, schemas cannot use `$ref` references.
- `FileClassKey` (string) - Frontmatter field name for schema selection. Default: `"fileClass"`. Supports custom keys like `"type"`, `"category"`, `"kind"` for different vault conventions. Used by Frontmatter.FileClass() to extract schema identifier. Enables schema flexibility without code changes.
- `CacheDir` (string) - Path to index cache directory. Default: `{VaultPath}/.lithos/cache/`. Can be absolute or relative to VaultPath. Created automatically if missing. Must be writable. Epic 3 hybrid storage uses BoltDB (`.lithos/cache/lithos.db`) for hot cache and SQLite (`.lithos/cache/lithos_metadata.db`) for deep storage.
- `LogLevel` (string) - Logging verbosity for zerolog. One of: "debug", "info", "warn", "error". Default: "info". Case-insensitive. Invalid values fall back to "info" with warning. Controls stdout/stderr output verbosity.

**Relationships:**

- Used by all adapters for initialization and runtime configuration
- Loaded at startup via ConfigLoader adapter (reads `lithos.json`, environment variables, flags in that precedence order)
- Passed to components via constructor injection (dependency injection pattern)
- PropertyBankFile used by SchemaLoader to locate property bank within SchemasDir

**Design Decisions:**

- **Value object (DDD):** Immutable configuration data identified by its attributes. Two Config instances with identical values are equivalent. Loaded once at startup, never modified.
- **JSON format for MVP:** Config file is `lithos.json` for MVP. Post-MVP: expand to support TOML and YAML formats for user preference.
- **Flat structure (MVP):** Flat Config struct with all settings as top-level fields for simplicity. Alternative composed structure with logical groupings (VaultConfig, SchemaConfig, TemplateConfig, LoggingConfig) available post-MVP if config grows complex.
- **FileClassKey for schema flexibility:** Configurable field name for schema selection (default `"fileClass"`). Supports different vault conventions (`"type"`, `"category"`, `"kind"`) without code changes. Accessed via global singleton Config by Frontmatter.FileClass().
- **Sensible defaults:** Empty config file is valid - all paths default to sensible vault-relative locations. FileClassKey defaults to `"fileClass"` (Obsidian convention). Enables quickstart: user can run `lithos index` with zero configuration if vault uses standard directory structure.
- **String paths:** Paths stored as strings, not file handles or custom Path types. Adapters resolve paths on demand using `filepath.Join` and `filepath.Abs`. This keeps config serializable and adapter-agnostic.
- **PropertyBankFile is filename only:** Not a full path. Always located in SchemasDir. SchemaLoader constructs full path: `filepath.Join(config.SchemasDir, config.PropertyBankFile)`.
- **Validation at load time:** ConfigLoader validates that VaultPath exists, is directory, and is readable. FileClassKey validated as non-empty string. Other paths validated lazily when accessed (TemplatesDir validated on first `lithos find`, not at config load).
- **Environment variable override:** ConfigLoader supports env vars like `LITHOS_VAULT_PATH`, `LITHOS_FILE_CLASS_KEY`. Precedence: CLI flags > env vars > config file > defaults. This enables CI/CD override without modifying config files.
- **No secrets in config:** Config is committed to git (per PRD, vaults are git repositories). No API keys, tokens, or passwords. Future: if external API integrations added, use separate credential files or system keychain.

**Additional Information:**

Config is a domain value object representing application configuration state. While loaded by infrastructure adapter (ConfigLoader), the model itself represents domain knowledge about vault structure and resource locations. The flat structure keeps configuration simple and readable for users. Sensible defaults mean a user can create an empty `lithos.json` and the application works immediately if using standard directory conventions. The precedence order (CLI flags > env vars > config file > defaults) provides flexibility for different environments - developers can override locally via flags, CI/CD can inject via environment variables, and teams can share baseline config in version control. String-based paths keep Config serializable and platform-agnostic - no special types needed. For MVP, JSON format provides simplicity with excellent Go stdlib support. Post-MVP expansion to TOML/YAML gives users format choice.

---

## Domain Events

**Purpose:** Event models for event-driven architecture. Represents significant domain occurrences that other components react to via publish/subscribe pattern. Implemented in Epic 3 to eliminate god-objects and enable clean CQRS separation.

**Architecture Layer:** Domain Core (Active in Epic 3)

**Status:** ACTIVE - Epic 3 implements event-driven architecture (Story 3.29). Event bus with domain events replaces direct service dependencies to eliminate god-object pattern (CLICommander, VaultIndexer).

### DomainEvent Interface

**Purpose:** Base interface for all domain events. Provides common event metadata.

**Key Methods:**

- `EventType() string` - Returns event type identifier (e.g., "NoteIndexed", "FrontmatterValidated")
- `OccurredAt() time.Time` - Returns event timestamp
- `AggregateID() string` - Returns ID of aggregate that triggered event (e.g., NoteID, SchemaName)

### Event Types

#### Indexing Events

**NoteIndexed** - Published when single note successfully indexed

- **Fields:** NoteID, Path, FileClass, OccurredAt
- **Use Cases:** Update search index, refresh graph, trigger backlink computation

**VaultIndexingComplete** - Published when full vault indexing finishes

- **Fields:** NotesIndexed (int), Duration, OccurredAt
- **Use Cases:** QueryService rebuilds indices, UI shows indexing complete, cache warmup

#### Validation Events

**FrontmatterValidated** - Published when frontmatter validation completes

- **Fields:** NoteID, SchemaName, IsValid (bool), Errors ([]ValidationError), OccurredAt
- **Use Cases:** Collect validation statistics, UI shows validation errors, quality metrics

#### Configuration Events

**SchemaLoaded** - Published when single schema successfully loaded

- **Fields:** SchemaName, PropertyCount (int), OccurredAt
- **Use Cases:** Audit log, reload dependent schemas, validation cache invalidation

**SchemasReloaded** - Published when all schemas reloaded (hot reload)

- **Fields:** SchemaCount (int), OccurredAt
- **Use Cases:** Clear validation caches, notify UI, audit configuration changes

### Event-Driven Architecture Benefits (Epic 3 Implementation)

**Implementation:** Story 3.29 implements EventBus infrastructure with in-memory goroutine-based async dispatch.

**Benefits Realized:**

- **God-Object Elimination:** CLICommander and VaultIndexer no longer accumulate dependencies - services communicate via events
- **CQRS Separation:** QueryService subscribes to VaultIndexingComplete event (pure read-side), VaultIndexer publishes events (command-side)
- **Decoupling:** Services don't directly depend on each other - add new subscribers without modifying publishers
- **Extensibility:** New features subscribe to existing events (e.g., MetricsService subscribes to FrontmatterValidated)
- **Testability:** Mock EventBus for unit tests, test event flows independently

**Trade-offs Accepted:**

- **Infrastructure Complexity:** EventBus implementation, subscription management (acceptable for god-object elimination)
- **Debugging Complexity:** Async execution harder to trace (mitigated by comprehensive event logging with trace IDs)
- **Eventual Consistency:** Subscribers process with delay (mitigated by synchronous dispatch for critical events)

**Publisher/Subscriber Architecture:**

- **Publishers:** VaultIndexer (NoteIndexed, VaultIndexingComplete), FrontmatterService (FrontmatterValidated), SchemaEngine (SchemaLoaded, SchemasReloaded)
- **Subscribers:** VaultIndexer (NoteIndexed → update indices), QueryService (VaultIndexingComplete → rebuild query structures), MetricsService (FrontmatterValidated → stats)

**Implementation Details:** See high-level-architecture.md "Orchestration Pattern Decision" section for complete event-driven architecture specification.

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
  ├─> FileClassKey: string (default: "fileClass")
  ├─> CacheDir: string (default: ".lithos/cache/")
  └─> LogLevel: string

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
  ├─> Path: string (vault-relative path as identifier)
  └─> Frontmatter
        ├─> FileClass: string (computed from Fields[Config.FileClassKey])
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
