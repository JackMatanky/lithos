# Epic 1: Foundational CLI with Static Template Engine

**Epic Goal:** Establish a production-ready Go CLI application with clean hexagonal architecture that can render static markdown templates using Go's text/template engine. This epic delivers the foundational infrastructure (project setup, logging, error handling, configuration) while providing immediate user value through the `lithos new` command that generates markdown notes from templates with basic template functions (now, path control). All domain models, ports, and adapters follow architecture v0.6.8 principles with proper dependency injection, enabling future epics to build upon a solid, testable foundation.

**Architecture References:**
- `docs/architecture/components.md` v0.6.8
- `docs/architecture/data-models.md` v0.6.8
- `docs/architecture/tech-stack.md`
- `docs/architecture/coding-standards.md`

---

## Story 1.1: Verify Development Tooling and Project Structure

As a developer,
I want to verify that all development tooling configuration files and project structure exist,
so that the project has consistent linting, formatting, security scanning, and proper hexagonal architecture from the start.

### Acceptance Criteria

**Development Tooling:**

- 1.1.1: Verify `.golangci.toml` exists with correct configuration:
  - Uses golangci-lint v2 format
  - Enables all linters from architecture tech stack
  - Line length set to 100 characters
  - Timeout set to 5m

- 1.1.2: Verify `.gitignore` exists with entries for:
  - Go build artifacts (*.exe, *.dll, *.so, *.dylib, *.test)
  - IDE files (.vscode/, .idea/)
  - Lithos-specific (.lithos/, testdata/vault-copies/)
  - OS files (.DS_Store, Thumbs.db)

- 1.1.3: Verify `.gitattributes` exists with:
  - Line ending normalization (* text=auto eol=lf)
  - Binary file markers for images, archives
  - Git LFS configuration for large files

- 1.1.4: Verify `.gitleaks.toml` exists with:
  - useDefault = true
  - AWS, GitHub, Slack secret detection rules
  - Allowlist for test files and documentation

- 1.1.5: Verify `.pre-commit-config.yaml` exists with hooks:
  - pre-commit-hooks (check-json, check-yaml, detect-private-key, etc.)
  - gitleaks (v8.28.0)
  - go-fmt, go-imports, go-vet-mod
  - golangci-lint (v2.4.0)

- 1.1.6: Run `pre-commit install` to activate hooks

- 1.1.7: Run `pre-commit run --all-files` and verify all hooks pass

**Project Structure:**

- 1.1.8: Verify `go.mod` exists with:
  - Module: `github.com/JackMatanky/lithos`
  - Go version: 1.23+ (required for generics)

- 1.1.9: Verify hexagonal architecture directories exist:
  - `internal/domain/` (core business logic)
  - `internal/app/` (core business services)
  - `internal/ports/api/` (primary/driving port interfaces)
  - `internal/ports/spi/` (secondary/driven port interfaces)
  - `internal/adapters/api/` (primary/driving adapters)
  - `internal/adapters/spi/` (secondary/driven adapters)
  - `internal/shared/` (cross-cutting concerns)

- 1.1.10: Verify test directories exist:
  - `tests/integration/`
  - `tests/e2e/`
  - `tests/utils/` (contains mocks.go)

- 1.1.11: Verify testdata structure exists:
  - `testdata/golden/` (expected outputs)
  - `testdata/schema/valid/` (valid schemas)
  - `testdata/schema/invalid/` (invalid schemas for testing)
  - `testdata/schema/properties/` (property bank files)
  - `testdata/notes/` (sample notes)
  - `testdata/templates/` (sample templates)

- 1.1.12: Verify CLI entrypoint exists at `cmd/lithos/main.go`

- 1.1.13: Verify `README.md` exists with project overview

- 1.1.14: Run `go mod tidy` to ensure dependencies are clean

- 1.1.15: Document verification results in commit message

- 1.1.16: Committed with message: `chore: verify development tooling and project structure`

**Prerequisites:** None (first story)

**Time Estimate:** 2 hours

**Architecture References:**
- Tech stack: `docs/architecture/tech-stack.md`
- Source tree: `docs/architecture/source-tree.md`
- Coding standards: `docs/architecture/coding-standards.md`

---

## Story 1.2: Implement Note Domain Models (NoteID, Frontmatter, Note)

As a developer,
I want to implement NoteID, Frontmatter, and Note domain models in note.go,
so that the domain layer has clean note entities without infrastructure dependencies.

### Acceptance Criteria

**NoteID Implementation:**

- 1.2.1: Create `internal/domain/note.go` with NoteID:
  - Type: `type NoteID string`
  - Constructor: `NewNoteID(value string) NoteID`
  - Method: `String() string` for logging/debugging

**Frontmatter Implementation:**

- 1.2.2: Add Frontmatter to `internal/domain/note.go`:
  - Field: `FileClass string` (computed from Fields)
  - Field: `Fields map[string]interface{}` (complete YAML frontmatter)
  - Constructor: `NewFrontmatter(fields map[string]interface{}) Frontmatter`
  - Helper: `extractFileClass(fields map[string]interface{}) string`
  - Method: `SchemaName() string` (returns FileClass)

**Note Implementation:**

- 1.2.3: Add Note to `internal/domain/note.go`:
  - Field: `ID NoteID`
  - Field: `Frontmatter Frontmatter` (composition, NOT embedding)
  - Constructor: `NewNote(id NoteID, frontmatter Frontmatter) Note`
  - Method: `SchemaName() string` (delegates to Frontmatter.SchemaName())

**Testing:**

- 1.2.4: Create unit tests in `internal/domain/note_test.go`:
  - Test NoteID: NewNoteID creates valid instance, String() returns value, can be used as map key
  - Test Frontmatter: NewFrontmatter constructs correctly, FileClass extracted from Fields["fileClass"], FileClass empty when not present, SchemaName() returns FileClass
  - Test Note: NewNote constructs with ID and Frontmatter, SchemaName() delegates correctly, no embedded File or filesystem paths, Note can be serialized to JSON

- 1.2.5: All tests pass: `go test ./internal/domain`

- 1.2.6: All linting passes: `golangci-lint run --fix internal/domain`

- 1.2.7: Committed with message: `feat(domain): implement Note domain models (NoteID, Frontmatter, Note)`

**Prerequisites:** Story 1.1

**Time Estimate:** 3 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#noteid` (v0.5.2)
- Data models: `docs/architecture/data-models.md#frontmatter` (v0.1.0)
- Data models: `docs/architecture/data-models.md#note` (v0.5.2)

---

## Story 1.3: Implement Template Domain Models (TemplateID, Template)

As a developer,
I want to implement TemplateID and Template domain models in template.go,
so that templates are represented as pure domain data without infrastructure concerns.

### Acceptance Criteria

**TemplateID Implementation:**

- 1.3.1: Create `internal/domain/template.go` with TemplateID:
  - Type: `type TemplateID string`
  - Constructor: `NewTemplateID(value string) TemplateID`
  - Method: `String() string` for logging/debugging

**Template Implementation:**

- 1.3.2: Add Template to `internal/domain/template.go`:
  - Field: `ID TemplateID`
  - Field: `Content string` (raw template text)
  - Constructor: `NewTemplate(id TemplateID, content string) Template`
  - NO FilePath field (infrastructure concern)
  - NO Parsed field (caching concern)

**Testing:**

- 1.3.3: Create unit tests in `internal/domain/template_test.go`:
  - Test TemplateID: NewTemplateID creates valid instance, String() returns value, can be used as map key
  - Test Template: NewTemplate constructs with ID and Content, no filesystem dependencies, Template can store Go template syntax in Content

- 1.3.4: All tests pass: `go test ./internal/domain`

- 1.3.5: All linting passes: `golangci-lint run --fix internal/domain`

- 1.3.6: Committed with message: `feat(domain): implement Template domain models (TemplateID, Template)`

**Prerequisites:** Story 1.1

**Time Estimate:** 2 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#templateid` (v0.5.6)
- Data models: `docs/architecture/data-models.md#template` (v0.5.6)

---

## Story 1.4: Implement Config Domain Model

As a developer,
I want to implement the Config domain model as a value object,
so that configuration is represented in the domain layer.

### Acceptance Criteria

- 1.4.1: Create `internal/domain/config.go`:
  - Field: `VaultPath string` (default: ".")
  - Field: `TemplatesDir string` (default: "templates/")
  - Field: `SchemasDir string` (default: "schemas/")
  - Field: `PropertyBankFile string` (default: "property_bank.json")
  - Field: `CacheDir string` (default: ".lithos/cache/")
  - Field: `LogLevel string` (default: "info")
  - Constructor: `NewConfig(...)` with all fields
  - Constructor: `DefaultConfig()` using defaults

- 1.4.2: Create unit tests in `internal/domain/config_test.go`:
  - Test: NewConfig constructs with all fields
  - Test: DefaultConfig uses correct defaults
  - Test: Config is immutable (copy on modification pattern if needed)

- 1.4.3: All tests pass: `go test ./internal/domain`

- 1.4.4: All linting passes: `golangci-lint run --fix internal/domain`

- 1.4.5: Committed with message: `feat(domain): implement Config model`

**Prerequisites:** Story 1.1

**Time Estimate:** 2 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#config` (v0.5.7)

---

## Story 1.5: Implement Shared Error Package

As a developer,
I want to implement all error types using idiomatic Go error handling,
so that the application has consistent error handling without Result[T] pattern.

### Acceptance Criteria

**Base Error Types:**

- 1.5.1: Create `internal/shared/errors/types.go`:
  - `BaseError` struct with `message string` and `cause error` fields
  - Implements `Error() string` and `Unwrap() error` methods
  - Constructor: `NewBaseError(message string, cause error) BaseError`

- 1.5.2: Add `ValidationError` to `internal/shared/errors/types.go`:
  - Embeds `BaseError`
  - Fields: `property string`, `reason string`, `value interface{}`
  - Constructor: `NewValidationError(property, reason string, value interface{}, cause error) *ValidationError`
  - Methods: `Property() string`, `Reason() string`, `Value() interface{}`

- 1.5.3: Add `ResourceError` to `internal/shared/errors/types.go`:
  - Embeds `BaseError`
  - Fields: `resource string`, `operation string`, `target string`
  - Constructor: `NewResourceError(resource, operation, target string, cause error) *ResourceError`
  - Methods: `Resource() string`, `Operation() string`, `Target() string`

- 1.5.4: Create unit tests in `internal/shared/errors/types_test.go`:
  - Test: Error construction and methods
  - Test: errors.Unwrap() works correctly
  - Test: errors.Is() and errors.As() work correctly

**Domain-Specific Errors:**

- 1.5.5: Create `internal/shared/errors/template.go`:
  - `TemplateError` type using BaseError
  - Constructor: `NewTemplateError(message string, templateID string, cause error)`
  - Method: `TemplateID() string`
  - Unit tests for construction and unwrapping

- 1.5.6: Create `internal/shared/errors/schema.go`:
  - `SchemaError` type using BaseError
  - Constructor: `NewSchemaError(message string, schemaName string, cause error)`
  - Method: `SchemaName() string`
  - Unit tests for construction and unwrapping

- 1.5.7: Create `internal/shared/errors/frontmatter.go`:
  - `FrontmatterError` type using BaseError
  - Constructor: `NewFrontmatterError(message string, field string, cause error)`
  - Method: `Field() string`
  - Unit tests for construction and unwrapping

**Helper Functions:**

- 1.5.8: Create `internal/shared/errors/helpers.go`:
  - `Wrap(err error, message string) error` - wraps with fmt.Errorf
  - `WrapWithContext(err error, format string, args ...interface{}) error`
  - Uses Go stdlib `errors.Join()` for multiple errors

- 1.5.9: Create unit tests in `internal/shared/errors/helpers_test.go`:
  - Test: Wrap() preserves original error
  - Test: WrapWithContext() formats message correctly
  - Test: errors.Unwrap() works on wrapped errors

- 1.5.10: **Verify NO Result[T] type exists anywhere in codebase**

- 1.5.11: All tests pass: `go test ./internal/shared/errors`

- 1.5.12: All linting passes: `golangci-lint run --fix internal/shared/errors`

- 1.5.13: Committed with message: `feat(shared): implement error package with idiomatic Go`

**Prerequisites:** Story 1.1

**Time Estimate:** 4 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Error Package
- Coding standards: `docs/architecture/coding-standards.md` - Error Handling (v0.6.7, removed Result[T])
- Change log: v0.5.9 - Error types split (FrontmatterError, CacheReadError, etc.)

---

## Story 1.6: Implement Logger Package

As a developer,
I want to implement structured logging with zerolog,
so that the application has consistent, structured logging across all components with configurable output formats.

### Acceptance Criteria

- 1.6.1: Add `github.com/rs/zerolog` v1.34.0 to `go.mod`

- 1.6.2: Create `internal/shared/logger/logger.go`:
  - Global `Log zerolog.Logger` instance
  - `New(output io.Writer, level string) zerolog.Logger` - creates configured logger
  - TTY detection via `golang.org/x/term` - pretty-print for terminals, JSON for pipes
  - Level parsing: "debug", "info", "warn", "error" (case-insensitive)

- 1.6.3: Context methods:
  - `WithComponent(logger zerolog.Logger, component string) zerolog.Logger`
  - `WithOperation(logger zerolog.Logger, operation string) zerolog.Logger`
  - `WithCorrelationID(logger zerolog.Logger, id string) zerolog.Logger`

- 1.6.4: Sensitive data filtering:
  - Filter password, token, apiKey fields from logs
  - Redact with "[REDACTED]" string

- 1.6.5: Test helper:
  - `NewTest() zerolog.Logger` - returns logger with ioutil.Discard for testing

- 1.6.6: Create unit tests in `internal/shared/logger/logger_test.go`:
  - Test: Log level configuration works
  - Test: TTY detection switches output format
  - Test: Context methods add fields correctly
  - Test: Sensitive data filtering works

- 1.6.7: All tests pass: `go test ./internal/shared/logger`

- 1.6.8: All linting passes: `golangci-lint run --fix internal/shared/logger`

- 1.6.9: Committed with message: `feat(shared): implement logger package with zerolog`

**Prerequisites:** Story 1.1

**Time Estimate:** 2.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Logger
- Tech stack: `docs/architecture/tech-stack.md` - zerolog v1.34.0

---

## Story 1.7: Implement Registry Package

As a developer,
I want to implement generic thread-safe registry with CQRS interfaces,
so that schemas and templates can be stored in-memory with concurrent read access.

### Acceptance Criteria

- 1.7.1: Create `internal/shared/registry/registry.go`:
  - Uses Go 1.23+ generics: `Registry[T any]`
  - Thread-safe with `sync.RWMutex`
  - Internal storage: `map[string]T`

- 1.7.2: Define CQRS interfaces in same file:
  - `Reader[T any]` interface:
    - `Get(key string) (T, error)`
    - `Exists(key string) bool`
    - `ListKeys() []string`
  - `Writer[T any]` interface:
    - `Register(key string, value T) error`
    - `Clear()`
  - `Registry[T any]` interface embeds both Reader and Writer

- 1.7.3: Implement `registry[T any]` struct (unexported):
  - Fields: `mu sync.RWMutex`, `items map[string]T`
  - Constructor: `New[T any]() Registry[T]`

- 1.7.4: Implement all interface methods:
  - `Get()` uses `RLock()` for concurrent reads
  - `Register()` uses `Lock()` for exclusive writes
  - `Exists()` uses `RLock()` for concurrent reads
  - Returns `ErrNotFound` from errors package when key doesn't exist

- 1.7.5: Create unit tests in `internal/shared/registry/registry_test.go`:
  - Test: Generic type instantiation works (string, int, custom struct)
  - Test: Concurrent reads don't block each other
  - Test: Writes block reads correctly
  - Test: ErrNotFound returned for missing keys
  - Test: ListKeys() returns all registered keys

- 1.7.6: Create benchmark tests:
  - Benchmark: Concurrent read performance
  - Benchmark: Write contention

- 1.7.7: All tests pass: `go test ./internal/shared/registry`

- 1.7.8: All linting passes: `golangci-lint run --fix internal/shared/registry`

- 1.7.9: Committed with message: `feat(shared): implement thread-safe registry with CQRS`

**Prerequisites:** Story 1.1

**Time Estimate:** 3 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Registry Package

---

## Story 1.8: Implement Config Loading Infrastructure

As a developer,
I want to implement ConfigPort interface and ViperAdapter,
so that application configuration can be loaded from file, environment, and flags following hexagonal architecture.

### Acceptance Criteria

**ConfigPort Interface:**

- 1.8.1: Create `internal/ports/spi/config.go`:
  - Interface: `ConfigPort`
  - Method: `Load(ctx context.Context) (Config, error)`
  - Add documentation comments:
    - Describe port responsibility
    - Document Load() method contract
    - Reference architecture components.md

**ViperAdapter Implementation:**

- 1.8.2: Add `github.com/spf13/viper` to `go.mod`

- 1.8.3: Create `internal/adapters/spi/config/viper.go`:
  - Implements ConfigPort interface
  - Constructor: `NewViperAdapter(log zerolog.Logger) *ViperAdapter`

- 1.8.4: Implement Load() method with precedence (highest to lowest):
  1. CLI flags (not implemented yet, reserved for future stories)
  2. Environment variables (LITHOS_VAULT_PATH, LITHOS_LOG_LEVEL, etc.)
  3. Config file (`lithos.json` searched upward from CWD)
  4. Defaults from Config.DefaultConfig()

- 1.8.5: Config file search pattern:
  - Start from current working directory
  - Search upward through parent directories
  - Stop at first `lithos.json` found
  - Use that directory as VaultPath
  - If no config found, use CWD as VaultPath

- 1.8.6: Environment variable mapping:
  - `LITHOS_VAULT_PATH` → Config.VaultPath
  - `LITHOS_TEMPLATES_DIR` → Config.TemplatesDir
  - `LITHOS_SCHEMAS_DIR` → Config.SchemasDir
  - `LITHOS_PROPERTY_BANK_FILE` → Config.PropertyBankFile
  - `LITHOS_CACHE_DIR` → Config.CacheDir
  - `LITHOS_LOG_LEVEL` → Config.LogLevel

- 1.8.7: Validate VaultPath exists and is directory (return error if not)

**Testing:**

- 1.8.8: Create unit tests in `internal/adapters/spi/config/viper_test.go`:
  - Test: Defaults work when no config file
  - Test: Config file values override defaults
  - Test: Environment variables override config file
  - Test: VaultPath validation rejects non-existent paths
  - Test: Upward search finds config in parent directories

- 1.8.9: Create integration test with `testdata/vault/lithos.json`:
  - Create sample config file in testdata
  - Verify adapter loads values correctly

- 1.8.10: All tests pass: `go test ./internal/ports/spi ./internal/adapters/spi/config`

- 1.8.11: All linting passes: `golangci-lint run --fix internal/ports/spi internal/adapters/spi/config`

- 1.8.12: Committed with message: `feat: implement config loading with ConfigPort and ViperAdapter`

**Prerequisites:** Story 1.4 (Config model), Story 1.6 (Logger)

**Time Estimate:** 4 hours

**Architecture References:**
- Components: `docs/architecture/components.md#spi-port-interfaces` - ConfigPort (v0.5.11)
- Components: `docs/architecture/components.md#spi-adapters` - ViperAdapter

---

## Story 1.9: Implement Template Loading Infrastructure

As a developer,
I want to implement FileMetadata model, TemplatePort interface, and TemplateLoaderAdapter,
so that templates can be loaded from filesystem following hexagonal architecture.

### Acceptance Criteria

**FileMetadata Model:**

- 1.9.1: Create `internal/adapters/spi/file_metadata.go`:
  - SPI adapter model (NOT domain)
  - Fields:
    - `Path string` - absolute file path
    - `Basename string` - filename without path/extension (computed)
    - `Folder string` - parent directory (computed)
    - `Ext string` - file extension including dot (computed)
    - `ModTime time.Time` - modification timestamp
    - `Size int64` - file size in bytes
    - `MimeType string` - MIME type (computed)
  - Constructor: `NewFileMetadata(path string, info fs.FileInfo) FileMetadata`
  - Helper functions:
    - `computeBasename(path string) string` - removes path and extension
    - `computeFolder(path string) string` - returns directory path
    - `computeMimeType(ext string) string` - detects MIME type from extension

- 1.9.2: Create unit tests in `internal/adapters/spi/file_metadata_test.go`:
  - Test: NewFileMetadata construction and all helper functions
  - Test: Computed fields (Basename, Folder, Ext) are correct
  - Test: MIME type detection works for common file types

**TemplatePort Interface:**

- 1.9.3: Create `internal/ports/spi/template.go`:
  - Interface: `TemplatePort`
  - Method: `List(ctx context.Context) ([]TemplateID, error)` - list available template IDs
  - Method: `Load(ctx context.Context, id TemplateID) (Template, error)` - load template by ID
  - Add documentation comments:
    - Describe port responsibility
    - Document method contracts
    - Reference architecture components.md

**TemplateLoaderAdapter:**

- 1.9.4: Create `internal/adapters/spi/template/loader.go`:
  - Implements TemplatePort interface
  - Constructor: `NewTemplateLoaderAdapter(config Config, log zerolog.Logger) *TemplateLoaderAdapter`
  - Internal metadata cache: `map[TemplateID]FileMetadata` for ID → Path mapping

- 1.9.5: Implement List() method:
  - Scan Config.TemplatesDir using `filepath.Walk`
  - Find all `.md` files
  - Derive TemplateID from basename (filename without `.md` extension)
  - Build FileMetadata for each template
  - Populate internal cache
  - Return sorted []TemplateID

- 1.9.6: Implement Load() method:
  - Look up TemplateID in metadata cache
  - If not found, return ResourceError with "template not found"
  - Read file content using `os.ReadFile`
  - Return Template with ID and Content
  - Validate UTF-8 encoding (return ValidationError if invalid)

- 1.9.7: Error handling:
  - Missing template → ResourceError with "template not found"
  - Read failure → ResourceError with wrapped os.Error
  - Invalid UTF-8 → ValidationError

**Testing:**

- 1.9.8: Create unit tests in `internal/adapters/spi/template/loader_test.go`:
  - Test: FileMetadata construction and helper functions
  - Test: List() finds all templates in testdata
  - Test: Load() reads template content correctly
  - Test: Errors returned for missing templates

- 1.9.9: Create integration test in `tests/integration/template_loader_test.go`:
  - Use existing `testdata/templates/` directory
  - Test: List() finds static-template.md and basic-note.md
  - Test: Load("static-template") returns Template with correct content
  - Test: Load("basic-note") returns Template with correct content
  - Test: Load("nonexistent") returns ResourceError

- 1.9.10: All tests pass: `go test ./internal/adapters/spi/... ./tests/integration/...`

- 1.9.11: All linting passes: `golangci-lint run --fix internal/adapters/spi internal/ports/spi`

- 1.9.12: Committed with message: `feat: implement template loading infrastructure`

**Prerequisites:** Story 1.3 (Template, TemplateID models), Story 1.4 (Config), Story 1.5 (errors), Story 1.6 (Logger)

**Time Estimate:** 4.5 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#filemetadata` (v0.5.1)
- Data models: `docs/architecture/data-models.md#templateid` (v0.5.6)
- Data models: `docs/architecture/data-models.md#template` (v0.5.6)
- Components: `docs/architecture/components.md#spi-port-interfaces` - TemplatePort (v0.5.11)
- Components: `docs/architecture/components.md#spi-adapters` - TemplateLoaderAdapter

---

## Story 1.10: Implement TemplateEngine Service

As a developer,
I want to implement complete TemplateEngine domain service with Load, Render, and all template functions,
so that templates can be loaded from port, parsed with Go text/template, and rendered with custom functions.

### Acceptance Criteria

**TemplateEngine Setup and Load:**

- 1.10.1: Create `internal/app/template_engine.go`:
  - Constructor: `NewTemplateEngine(templatePort TemplatePort, config Config, log zerolog.Logger) *TemplateEngine`
  - Dependencies: TemplatePort (injected), Config (injected), Logger (injected)

- 1.10.2: Implement `Load(ctx context.Context, templateID TemplateID) (Template, error)`:
  - Delegates to TemplatePort.Load()
  - Logs loading operation with template ID
  - Returns Template or error

**Function Map Setup:**

- 1.10.3: Implement private method `buildFuncMap() template.FuncMap`:
  - Returns template.FuncMap with all custom functions
  - Registers all functions described below

**Basic Template Functions:**

- 1.10.4: Implement basic functions in buildFuncMap():
  - `now(format string) string` - current timestamp with Go time layout (e.g., "2006-01-02")
  - `toLower(s string) string` - lowercase conversion using strings.ToLower
  - `toUpper(s string) string` - uppercase conversion using strings.ToUpper

**File Path Control Functions (v0.6.3):**

- 1.10.5: Implement file path control functions in buildFuncMap():
  - `path() string` - returns target file path (empty for MVP, used in Epic 3 for vault operations)
  - `folder(p string) string` - returns parent directory using filepath.Dir()
  - `basename(p string) string` - returns filename without extension (strips path and extension)
  - `extension(p string) string` - returns file extension using filepath.Ext() (includes dot)
  - `join(parts ...string) string` - joins path segments using filepath.Join() (OS-appropriate separator)
  - `vaultPath() string` - returns Config.VaultPath

**Render Method:**

- 1.10.6: Implement `Render(ctx context.Context, templateID TemplateID) (string, error)`:
  - Step 1: Load template via Load(ctx, templateID)
  - Step 2: Create `text/template` instance with template.ID as name
  - Step 3: Register function map via buildFuncMap()
  - Step 4: Parse template.Content using template.Parse()
  - Step 5: Execute template with empty data context (static rendering for Epic 1)
  - Step 6: Return rendered string
  - Error handling:
    - Template not found → ResourceError from Load()
    - Parse error → TemplateError with line/column info and template name
    - Execute error → TemplateError with template name and error details

**Testing:**

- 1.10.7: Add mock to `tests/utils/mocks.go`:
  - `MockTemplatePort` struct implementing TemplatePort interface
  - Internal storage: `map[TemplateID]Template`
  - Method: `SetTemplates(templates map[TemplateID]Template)` - configure mock responses
  - Method: `SetLoadError(err error)` - configure error response
  - Implements: List() returns keys from internal storage
  - Implements: Load() returns template from storage or configured error

- 1.10.8: Create unit tests in `internal/app/template_engine_test.go`:
  - Test: Load() delegates to TemplatePort correctly
  - Test: Load() logs operation
  - Test: Load() propagates errors from port
  - Test: now() function returns formatted timestamp
  - Test: toLower() converts to lowercase
  - Test: toUpper() converts to uppercase
  - Test: folder() returns parent directory
  - Test: basename() strips path and extension
  - Test: extension() returns extension with dot
  - Test: join() uses OS-appropriate path separator
  - Test: vaultPath() returns config value
  - Test: Render() executes static template
  - Test: Render() uses now() function correctly
  - Test: Render() uses path control functions correctly
  - Test: Parse error returns TemplateError with details
  - Test: Execute error returns TemplateError with context
  - All tests use MockTemplatePort from tests/utils/mocks.go

- 1.10.9: Create integration test in `tests/integration/template_engine_test.go`:
  - Use existing testdata/templates/static-template.md
  - Compare rendered output to testdata/golden/static-template-expected.md
  - Verify all template functions work end-to-end:
    - now() with various format strings
    - toLower() and toUpper()
    - File path control functions (folder, basename, extension, join, vaultPath)
  - Test: Template composition works ({{template "name"}})

- 1.10.10: All tests pass: `go test ./internal/app ./tests/integration`

- 1.10.11: All linting passes: `golangci-lint run --fix internal/app`

- 1.10.12: Committed with message: `feat(app): implement TemplateEngine service with all functions`

**Prerequisites:** Story 1.3 (Template, TemplateID), Story 1.4 (Config), Story 1.5 (errors), Story 1.6 (Logger), Story 1.9 (TemplatePort)

**Time Estimate:** 5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - TemplateEngine (v0.1.0, updated v0.6.2)
- Components: Custom template functions section
- Change log: v0.6.3 - File path template functions

---

## Story 1.11: Implement CLI Port Interfaces

As a developer,
I want to define CLICommandPort and CommandHandler API interfaces,
so that CLI framework integration follows hexagonal callback pattern.

### Acceptance Criteria

- 1.11.1: Create `internal/ports/api/cli_command.go`:
  - Interface: `CLICommandPort`
  - Method: `Start(ctx context.Context, handler CommandHandler) error`
  - Add documentation comments:
    - Describe hexagonal callback pattern (CLI adapter receives handler, calls handler methods)
    - Document Start() method contract
    - Reference architecture components.md

- 1.11.2: Create `internal/ports/api/command_handler.go`:
  - Interface: `CommandHandler`
  - Method: `NewNote(ctx context.Context, templateID TemplateID) (Note, error)`
  - Add documentation comments:
    - Note: Additional methods will be added in later epics
    - Describe use case handler contract
    - Document NewNote() method as orchestrating template rendering and note creation
    - Reference architecture components.md

- 1.11.3: All linting passes: `golangci-lint run --fix internal/ports/api`

- 1.11.4: Committed with message: `feat(ports): define CLI port interfaces`

**Prerequisites:** Story 1.2 (Note, NoteID), Story 1.3 (TemplateID)

**Time Estimate:** 1 hour

**Architecture References:**
- Components: `docs/architecture/components.md#api-port-interfaces` - CLICommandPort (v0.6.4)
- Components: CommandHandler callback interface
- Change log: v0.6.4 - CommandOrchestrator and CLICommandPort pattern

---

## Story 1.12: Implement CobraCLI Adapter

As a developer,
I want to implement complete CobraCLI adapter with root command, version command, and new command,
so that users can interact with the application via CLI following hexagonal architecture with SRP decomposition.

### Acceptance Criteria

**Dependencies and Setup:**

- 1.12.1: Add dependencies to `go.mod`:
  - `github.com/spf13/cobra`
  - `github.com/spf13/pflag`

- 1.12.2: Create `internal/adapters/api/cli/cobra.go`:
  - Implements CLICommandPort interface
  - Constructor: `NewCobraCLIAdapter(log zerolog.Logger) *CobraCLIAdapter`
  - Internal field: `handler CommandHandler` (stored in Start())
  - Internal field: `log zerolog.Logger`

**Start Method:**

- 1.12.3: Implement `Start(ctx context.Context, handler CommandHandler) error`:
  - Stores handler parameter in internal field
  - Calls buildRootCommand() private method
  - Executes root command with context: rootCmd.ExecuteContext(ctx)
  - Returns error from command execution

**Root Command:**

- 1.12.4: Implement private method `buildRootCommand() *cobra.Command`:
  - Use: "lithos"
  - Short: "Template-driven markdown note generator for Obsidian vaults"
  - SilenceUsage: true (don't print usage on command errors)
  - SilenceErrors: true (handle errors in domain)
  - Adds version subcommand via AddCommand(buildVersionCommand())
  - Adds new subcommand via AddCommand(buildNewCommand())

**Version Command:**

- 1.12.5: Implement private method `buildVersionCommand() *cobra.Command`:
  - Command: `lithos version`
  - Short: "Print version information"
  - RunE: prints "lithos v0.1.0" to stdout
  - Returns nil error on success

**New Command:**

- 1.12.6: Implement private method `buildNewCommand() *cobra.Command`:
  - Command: `lithos new [template-id]`
  - Short: "Create a new note from template"
  - Args: `cobra.MaximumNArgs(1)` (template-id is optional for now, will error in handler if missing)
  - Flags: `--view, -v` (boolean, default false) - display content after creation
  - RunE: calls handleNewCommand(cmd, args)

**New Command Handler:**

- 1.12.7: Implement private method `handleNewCommand(cmd *cobra.Command, args []string) error`:
  - Extract template-id from args[0] (return error if missing with message "template-id required")
  - Create TemplateID from args[0]: `templateID := domain.NewTemplateID(args[0])`
  - Call handler.NewNote(cmd.Context(), templateID)
  - On success: call displayNoteCreated(cmd, note)
  - On failure: return formatError(err)

- 1.12.8: Implement private method `displayNoteCreated(cmd *cobra.Command, note Note)`:
  - Print to stdout: `✓ Created: {Config.VaultPath}/{note.ID}.md`
  - Check --view flag via cmd.Flags().GetBool("view")
  - If --view is true:
    - Print separator line (80 characters of "=")
    - Print note content (read from file)
    - Print separator line

- 1.12.9: Implement private method `formatError(err error) error`:
  - Check error type using errors.As()
  - Format user-friendly messages:
    - ResourceError (template not found) → "Template '{name}' not found in {TemplatesDir}"
    - TemplateError (parse/render) → "Template error in '{name}': {message}"
    - Generic error → "Error: {message}"
  - Return formatted error for Cobra to display

**Testing:**

- 1.12.10: Add mock to `tests/utils/mocks.go`:
  - `MockCommandHandler` struct implementing CommandHandler interface
  - Internal field: `newNoteResult` (Note, error)
  - Method: `SetNewNoteResult(note Note, err error)` - configure mock response
  - Implements: NewNote() returns configured result

- 1.12.11: Create unit tests in `internal/adapters/api/cli/cobra_test.go`:
  - Test: Start() stores handler correctly
  - Test: buildRootCommand() creates command with correct structure
  - Test: version command prints "lithos v0.1.0"
  - Test: new command parses template-id argument correctly
  - Test: new command parses --view flag correctly
  - Test: handleNewCommand() extracts templateID from args
  - Test: handleNewCommand() returns error when args empty
  - Test: handleNewCommand() calls handler.NewNote() with correct arguments
  - Test: displayNoteCreated() formats output correctly without --view flag
  - Test: displayNoteCreated() displays content with --view flag
  - Test: formatError() formats ResourceError correctly
  - Test: formatError() formats TemplateError correctly
  - Test: formatError() formats generic error correctly
  - All tests use MockCommandHandler from tests/utils/mocks.go

- 1.12.12: All tests pass: `go test ./internal/adapters/api/cli`

- 1.12.13: All linting passes: `golangci-lint run --fix internal/adapters/api/cli`

- 1.12.14: Committed with message: `feat(adapters): implement CobraCLI adapter with version and new commands`

**Prerequisites:** Story 1.6 (Logger), Story 1.11 (CLICommandPort, CommandHandler interfaces)

**Time Estimate:** 5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#api-adapters` - CobraCLIAdapter (v0.5.11, updated v0.6.4)
- Components: SRP decomposition pattern section

---

## Story 1.13: Implement CommandOrchestrator

As a developer,
I want to implement CommandOrchestrator that orchestrates the NewNote use case workflow,
so that CLI commands are coordinated through domain services following hexagonal architecture.

### Acceptance Criteria

- 1.13.1: Create `internal/app/command_orchestrator.go`:
  - Constructor: `NewCommandOrchestrator(cliPort CLICommandPort, templateEngine *TemplateEngine, config Config, log zerolog.Logger) *CommandOrchestrator`
  - Implements CommandHandler interface (from ports/api/command_handler.go)
  - Dependencies: CLICommandPort (injected), TemplateEngine (injected), Config (injected), Logger (injected)

- 1.13.2: Implement `Run(ctx context.Context) error`:
  - Calls `cliPort.Start(ctx, self)` passing orchestrator as handler (hexagonal callback pattern)
  - Returns error from CLI execution

- 1.13.3: Implement `NewNote(ctx context.Context, templateID TemplateID) (Note, error)` workflow:
  - Step 1: Render template via `templateEngine.Render(ctx, templateID)`
    - Log operation: "Rendering template {templateID}"
  - Step 2: Generate NoteID from templateID (temporary strategy for Epic 1):
    - Use template basename as note filename: `noteID := domain.NewNoteID(templateID.String())`
  - Step 3: Create empty Frontmatter (no YAML parsing in Epic 1):
    - `frontmatter := domain.NewFrontmatter(map[string]interface{}{})`
  - Step 4: Construct Note:
    - `note := domain.NewNote(noteID, frontmatter)`
  - Step 5: Write rendered content to file:
    - Target path: `{Config.VaultPath}/{noteID}.md`
    - Use `os.WriteFile(path, []byte(renderedContent), 0644)`
    - Log operation: "Writing note to {path}"
  - Step 6: Return Note

- 1.13.4: Error handling:
  - Template not found → return ResourceError from TemplateEngine (propagates from TemplatePort)
  - Render error → return TemplateError from TemplateEngine
  - File write error → wrap with WrapWithContext("failed to write note to {path}", err)

- 1.13.5: Create unit tests in `internal/app/command_orchestrator_test.go`:
  - Test: Run() calls CLICommandPort.Start() with self as handler
  - Test: Run() propagates errors from CLI
  - Test: NewNote() orchestrates template rendering correctly
  - Test: NewNote() generates NoteID from templateID (basename strategy)
  - Test: NewNote() creates empty Frontmatter
  - Test: NewNote() constructs Note with NoteID and Frontmatter
  - Test: NewNote() writes file to vault at correct path
  - Test: NewNote() returns ResourceError when template not found
  - Test: NewNote() returns TemplateError on render failure
  - Test: NewNote() wraps file write errors with context
  - All tests use mocks from tests/utils/mocks.go (MockCLICommandPort, MockTemplatePort)

- 1.13.6: Add mock to `tests/utils/mocks.go`:
  - `MockCLICommandPort` struct implementing CLICommandPort interface
  - Internal field: `startCalled bool`, `startHandler CommandHandler`
  - Method: `Start(ctx, handler)` stores handler and sets startCalled flag
  - Method: `WasStartCalled() bool` returns startCalled
  - Method: `GetHandler() CommandHandler` returns stored handler

- 1.13.7: All tests pass: `go test ./internal/app`

- 1.13.8: All linting passes: `golangci-lint run --fix internal/app`

- 1.13.9: Committed with message: `feat(app): implement CommandOrchestrator with NewNote use case`

**Prerequisites:** Story 1.2 (Note, NoteID), Story 1.4 (Config), Story 1.5 (errors), Story 1.6 (Logger), Story 1.10 (TemplateEngine), Story 1.11 (CLICommandPort, CommandHandler)

**Time Estimate:** 4 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - CommandOrchestrator (v0.6.4)
- Components: NewNote workflow section
- Change log: v0.6.4 - CommandOrchestrator as proper use case orchestrator

---

## Story 1.14: Wire Dependency Injection and Create End-to-End Test

As a developer,
I want to wire all components in main.go using constructor-based dependency injection and create end-to-end test,
so that the complete application works from CLI invocation to file creation with proper initialization order.

### Acceptance Criteria

**Dependency Injection in main.go:**

- 1.14.1: Implement main() function in `cmd/lithos/main.go`:
  - Create context: `ctx := context.Background()`
  - Follow initialization order per architecture v0.6.5 (documented below)

- 1.14.2: Infrastructure layer initialization (Layer 1):
  - Create logger: `log := logger.New(os.Stdout, "info")` (default level)
  - Create config adapter: `configAdapter := config.NewViperAdapter(log)`
  - Load config: `cfg, err := configAdapter.Load(ctx)` - **fatal error if fails**
  - Update logger level from config: `log = logger.New(os.Stdout, cfg.LogLevel)`

- 1.14.3: SPI adapter initialization (Layer 2):
  - Create TemplateLoaderAdapter: `templateLoader := template.NewTemplateLoaderAdapter(cfg, log)`

- 1.14.4: Domain service initialization (Layer 3):
  - Create TemplateEngine: `templateEngine := app.NewTemplateEngine(templateLoader, cfg, log)`

- 1.14.5: API adapter initialization (Layer 4):
  - Create CobraCLIAdapter: `cliAdapter := cli.NewCobraCLIAdapter(log)`

- 1.14.6: CommandOrchestrator initialization (Layer 5):
  - Create CommandOrchestrator: `orchestrator := app.NewCommandOrchestrator(cliAdapter, templateEngine, cfg, log)`

- 1.14.7: Start application:
  - Call `err := orchestrator.Run(ctx)`
  - If error, log fatal and exit with code 1: `log.Fatal().Err(err).Msg("application failed")`

- 1.14.8: Error handling:
  - Config load failure → `log.Fatal().Err(err).Msg("failed to load configuration")`
  - Application failure → `log.Fatal().Err(err).Msg("application failed")`

**End-to-End Test:**

- 1.14.9: Create `tests/e2e/lithos_new_test.go`:
  - Test scenario: Full application flow from CLI to file creation
  - Setup:
    - Create temporary vault directory using `os.MkdirTemp`
    - Create templates/ subdirectory
    - Copy template from testdata/templates/static-template.md to temp vault
    - Set environment variable LITHOS_VAULT_PATH to temp directory
  - Execute:
    - Build lithos binary: `go build -o {tempDir}/lithos cmd/lithos/main.go`
    - Run command: `{tempDir}/lithos new static-template`
  - Verify:
    - Check file exists: `{tempDir}/static-template.md`
    - Read file content
    - Compare to expected output (verify template functions executed)
  - Cleanup:
    - Remove temporary directory

- 1.14.10: Additional test scenarios in same file:
  - Test: `lithos new basic-note` creates note with basic functions (now, toLower, toUpper)
  - Test: Error when template not found (returns exit code 1)
  - Test: `lithos version` prints version string

**Manual Testing:**

- 1.14.11: Manual test checklist (documented in story, not automated):
  - Build: `go build -o bin/lithos cmd/lithos/main.go`
  - Run: `./bin/lithos version` → prints "lithos v0.1.0"
  - Run: `./bin/lithos new static-template` (with testdata as vault) → creates note
  - Verify: File exists at expected location with rendered content
  - Run: `./bin/lithos new static-template --view` → displays content after creation
  - Run: `./bin/lithos new nonexistent` → shows error "Template 'nonexistent' not found"
  - Verify: Template functions work (now() shows current date, path functions work)
  - Verify: Config loads from lithos.json if present
  - Verify: Environment variables override config (test with LITHOS_LOG_LEVEL=debug)
  - Verify: Error messages are user-friendly (not stack traces)

- 1.14.12: All tests pass: `go test ./...` (all packages)

- 1.14.13: All linting passes: `golangci-lint run` (all code)

- 1.14.14: Committed with message: `feat: wire dependency injection in main.go and add e2e test`

**Prerequisites:** Story 1.13 (CommandOrchestrator)

**Time Estimate:** 4 hours

**Architecture References:**
- Components: `docs/architecture/components.md#dependency-injection-pattern` (v0.6.5)
- Components: Initialization order section
- Components: Example main.go structure
- Testing: `docs/architecture/testing-strategy.md`

---

## Story 1.15: Update Documentation for Epic 1 Release

As a developer,
I want to update project documentation with installation instructions, quick start guide, and feature list,
so that users can install and use lithos effectively.

### Acceptance Criteria

**README Update:**

- 1.15.1: Update `README.md` with comprehensive content:
  - **Installation section:**
    - Go install: `go install github.com/JackMatanky/lithos@latest`
    - Requirements: Go 1.23+ required for generics
  - **Quick Start section:**
    - Step 1: Create vault directory or use existing Obsidian vault
    - Step 2: Create `lithos.json` in vault root (optional, uses defaults if missing)
    - Step 3: Create `templates/` directory in vault
    - Step 4: Create template file (example provided: contact.md with frontmatter and functions)
    - Step 5: Run `lithos new contact` to generate note
    - Step 6: Check generated file in vault
  - **Configuration Reference section:**
    - Document all Config fields with descriptions and defaults:
      - VaultPath (default: ".") - Root directory of vault
      - TemplatesDir (default: "templates/") - Template files location
      - SchemasDir (default: "schemas/") - Schema files location (Epic 2)
      - PropertyBankFile (default: "property_bank.json") - Property bank filename (Epic 2)
      - CacheDir (default: ".lithos/cache/") - Index cache location (Epic 3)
      - LogLevel (default: "info") - Logging verbosity (debug, info, warn, error)
    - Document environment variable overrides (LITHOS_*)
    - Document config file search behavior (upward from CWD)
  - **Template Function Reference section:**
    - Basic functions:
      - `now(format)` - Current timestamp with Go time layout
      - `toLower(s)` - Lowercase conversion
      - `toUpper(s)` - Uppercase conversion
    - File path control functions:
      - `path()` - Target file path (empty in Epic 1, used in Epic 3)
      - `folder(p)` - Parent directory of path
      - `basename(p)` - Filename without extension
      - `extension(p)` - File extension including dot
      - `join(parts...)` - Join path segments with OS separator
      - `vaultPath()` - Vault root directory path
    - Include examples for each function
  - **CLI Commands section:**
    - `lithos version` - Print version information
    - `lithos new <template-id>` - Create note from template
    - `lithos new <template-id> --view` - Create note and display content
  - **Architecture section:**
    - Brief description of hexagonal architecture
    - Link to detailed architecture docs: `docs/architecture/`
  - **Contributing section:**
    - Link to development setup instructions
    - Note about pre-commit hooks
    - Link to architecture documentation for contributors

**CHANGELOG Creation:**

- 1.15.2: Create `CHANGELOG.md` with Epic 1 release:
  - Version 0.1.0 - Epic 1 Complete (date: TBD)
  - **Features Added:**
    - CLI with `version` and `new` commands
    - Static template rendering using Go text/template
    - Basic template functions: now, toLower, toUpper
    - File path control functions: path, folder, basename, extension, join, vaultPath
    - Configuration loading from file, environment variables, and defaults
    - Hexagonal architecture with clean domain layer (no infrastructure dependencies)
    - Comprehensive error handling with user-friendly messages
    - Structured logging with zerolog
    - Template loading from filesystem
    - Note creation workflow (template → rendered content → file)
  - **Architecture:**
    - Clean hexagonal architecture following v0.6.8 specifications
    - Domain models: NoteID, Frontmatter, Note, TemplateID, Template, Config
    - Domain services: TemplateEngine, CommandOrchestrator
    - Ports: ConfigPort, TemplatePort, CLICommandPort, CommandHandler
    - Adapters: ViperAdapter, TemplateLoaderAdapter, CobraCLIAdapter
    - Shared packages: Logger (zerolog), Error handling (idiomatic Go), Registry (generic CQRS)
  - **Testing:**
    - Unit tests for all components
    - Integration tests for template loading and rendering
    - End-to-end test for complete CLI workflow
    - Test coverage: [to be measured]
  - **Documentation:**
    - Complete architecture documentation in `docs/architecture/`
    - README with installation and quick start
    - Template function reference
    - Configuration reference
  - **Link to Epic 1 documentation:** `docs/prd/epic-1-foundational-cli-static-template-engine.md`

**Final Verification:**

- 1.15.3: Verify all tests pass:
  - Run `go test ./...` → all pass
  - Verify test coverage is reasonable (aim for >70% for domain/app layers)

- 1.15.4: Verify all linting passes:
  - Run `golangci-lint run` → no warnings or errors
  - Verify code follows architecture v0.6.8 specifications

- 1.15.5: Verify pre-commit hooks:
  - Run `pre-commit run --all-files` → all hooks pass
  - Verify no secrets detected by gitleaks
  - Verify Go formatting (gofmt, goimports) applied
  - Verify Go static analysis (go vet) passes
  - Verify golangci-lint passes

- 1.15.6: Committed with message: `docs: update README and create CHANGELOG for Epic 1 release`

**Prerequisites:** Story 1.14 (main.go wiring and e2e test)

**Time Estimate:** 3 hours

**Architecture References:**
- All architecture documents in `docs/architecture/`

---

## Epic 1 Completion Summary

### Total Stories: 15
### Estimated Time: 53 hours (~2.5 weeks for AI agent with 2-4 hour story sizes)

### Story Breakdown by Time

| Story | Title | Time | Cumulative |
|-------|-------|------|------------|
| 1.1 | Verify Tooling and Structure | 2h | 2h |
| 1.2 | Note Models (NoteID, Frontmatter, Note) | 3h | 5h |
| 1.3 | Template Models (TemplateID, Template) | 2h | 7h |
| 1.4 | Config Model | 2h | 9h |
| 1.5 | Shared Error Package | 4h | 13h |
| 1.6 | Logger Package | 2.5h | 15.5h |
| 1.7 | Registry Package | 3h | 18.5h |
| 1.8 | Config Loading (Port + Adapter) | 4h | 22.5h |
| 1.9 | Template Loading (Model + Port + Adapter) | 4.5h | 27h |
| 1.10 | TemplateEngine Service | 5h | 32h |
| 1.11 | CLI Port Interfaces | 1h | 33h |
| 1.12 | CobraCLI Adapter | 5h | 38h |
| 1.13 | CommandOrchestrator | 4h | 42h |
| 1.14 | DI + E2E Test | 4h | 46h |
| 1.15 | Documentation | 3h | 49h |

**Adjusted Total: 49 hours** (not 53h, recalculated from breakdown)

### Deliverables

✅ **Infrastructure:**
- Development tooling verified (Story 1.1)
- Project structure verified with hexagonal architecture (Story 1.1)
- testdata structure used correctly (Stories 1.9, 1.10, 1.14)
- Mocks in tests/utils/mocks.go (Stories 1.10, 1.12, 1.13)

✅ **Domain Models (v0.6.8):**
- NoteID, Frontmatter, Note in internal/domain/note.go (Story 1.2)
- TemplateID, Template in internal/domain/template.go (Story 1.3)
- Config in internal/domain/config.go (Story 1.4)

✅ **Shared Packages:**
- Error types - idiomatic Go, no Result[T] (Story 1.5)
- Logger with zerolog (Story 1.6)
- Registry with CQRS (Story 1.7)

✅ **Ports & Adapters:**
- ConfigPort + ViperAdapter (Story 1.8)
- TemplatePort + TemplateLoaderAdapter (Story 1.9)
- CLICommandPort + CommandHandler + CobraCLIAdapter (Stories 1.11-1.12)
- FileMetadata SPI model (Story 1.9)

✅ **Domain Services:**
- TemplateEngine with all functions in internal/app/ (Story 1.10)
- CommandOrchestrator in internal/app/ (Story 1.13)

✅ **Template Functions:**
- Basic: now, toLower, toUpper (Story 1.10)
- Path control: path, folder, basename, extension, join, vaultPath (Story 1.10)

✅ **Testing:**
- Unit tests for all components (all stories)
- Integration tests in tests/integration/ (Stories 1.9, 1.10)
- End-to-end tests in tests/e2e/ (Story 1.14)
- Mocks in tests/utils/mocks.go (Stories 1.10, 1.12, 1.13)

✅ **CLI Commands:**
- `lithos version` (Story 1.12)
- `lithos new <template-id>` with --view flag (Story 1.12)

✅ **Documentation:**
- README with installation and quick start (Story 1.15)
- CHANGELOG for v0.1.0 (Story 1.15)
- Configuration reference (Story 1.15)
- Template function reference (Story 1.15)

### Architecture Coverage (Epic 1)

**23 of 76 components implemented (30%)**

**Implemented Components:**

| Category | Components | Stories |
|----------|------------|---------|
| Domain Models | NoteID, Frontmatter, Note, TemplateID, Template, Config | 1.2, 1.3, 1.4 |
| SPI Models | FileMetadata | 1.9 |
| Domain Services | TemplateEngine, CommandOrchestrator | 1.10, 1.13 |
| SPI Ports | ConfigPort, TemplatePort | 1.8, 1.9 |
| SPI Adapters | ViperAdapter, TemplateLoaderAdapter | 1.8, 1.9 |
| API Ports | CLICommandPort, CommandHandler | 1.11 |
| API Adapters | CobraCLIAdapter | 1.12 |
| Shared Packages | Logger, Errors, Registry | 1.5, 1.6, 1.7 |
| Template Functions | 12 functions (basic + path control) | 1.10 |
| DI & Main | main.go initialization | 1.14 |

**Components Deferred to Later Epics:**

- **Epic 2:** Schema models (Schema, Property, PropertySpec variants, PropertyBank), SchemaEngine, SchemaValidator, SchemaResolver, SchemaPort, SchemaLoaderAdapter, SchemaRegistryAdapter
- **Epic 3:** VaultIndexer, QueryService, FrontmatterService, CQRS cache ports (CacheReader/Writer), CQRS vault ports (VaultReader/Writer), cache adapters, vault adapters, VaultFile DTO
- **Epic 4:** PromptPort, FinderPort, PromptUIAdapter, FuzzyfindAdapter, interactive template functions (prompt, suggester)
- **Epic 5:** Additional CommandOrchestrator use cases, query template functions (lookup, query, fileClass), full CLI integration

### Success Criteria

✅ All 15 stories completed
✅ All tests passing (`go test ./...`)
✅ All linting passing (`golangci-lint run`)
✅ `lithos new` command creates notes successfully
✅ No architectural violations (verified against v0.6.8)
✅ Clean hexagonal architecture with proper dependency injection
✅ Ready for Epic 2 (schema loading and validation)

---

## Next Epic

**Epic 2: Configuration & Schema Loading** - Implement schema loading from JSON, schema inheritance and resolution, property bank, and schema validation at startup. This will enable Epic 3 to validate frontmatter against schemas during vault indexing.
