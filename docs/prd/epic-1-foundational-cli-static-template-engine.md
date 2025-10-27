# Epic 1: Foundational CLI with Static Template Engine

**Epic Goal:** Establish a production-ready Go CLI application with clean hexagonal architecture that can render static markdown templates using Go's text/template engine. This epic delivers the foundational infrastructure (project setup, logging, error handling, configuration) while providing immediate user value through the `lithos new` command that generates markdown notes from templates with basic template functions (now, path control). All domain models, ports, and adapters follow architecture v0.6.8 principles with proper dependency injection, enabling future epics to build upon a solid, testable foundation.

**Architecture References:**
- `docs/architecture/components.md` v0.6.8
- `docs/architecture/data-models.md` v0.6.8
- `docs/architecture/tech-stack.md`
- `docs/architecture/coding-standards.md`

---

## Story 1.1: Verify Development Tooling Configuration

As a developer,
I want to verify that all development tooling configuration files exist and are properly configured,
so that the project has consistent linting, formatting, and security scanning from the start.

### Acceptance Criteria

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

- 1.1.8: Document verification results in commit message

**Prerequisites:** None (first story)

**Time Estimate:** 1-2 hours

**Architecture References:**
- Tech stack: `docs/architecture/tech-stack.md`
- Coding standards: `docs/architecture/coding-standards.md`

---

## Story 1.2: Verify Go Module and Project Structure

As a developer,
I want to verify the Go module is initialized and project structure follows hexagonal architecture,
so that the foundation is correct before adding code.

### Acceptance Criteria

- 1.2.1: Verify `go.mod` exists with:
  - Module: `github.com/JackMatanky/lithos`
  - Go version: 1.23+ (required for generics)

- 1.2.2: Verify hexagonal architecture directories exist:
  - `internal/domain/` (core business logic)
  - `internal/ports/api/` (primary/driving port interfaces)
  - `internal/ports/spi/` (secondary/driven port interfaces)
  - `internal/adapters/api/` (primary/driving adapters)
  - `internal/adapters/spi/` (secondary/driven adapters)
  - `internal/shared/` (cross-cutting concerns)

- 1.2.3: Verify test directories exist:
  - `tests/integration/`
  - `tests/e2e/`
  - `tests/utils/` (contains mocks.go)

- 1.2.4: Verify testdata structure exists:
  - `testdata/golden/` (expected outputs)
  - `testdata/schema/valid/` (valid schemas)
  - `testdata/schema/invalid/` (invalid schemas for testing)
  - `testdata/schema/properties/` (property bank files)
  - `testdata/notes/` (sample notes)
  - `testdata/templates/` (sample templates)

- 1.2.5: Verify CLI entrypoint exists at `cmd/lithos/main.go`

- 1.2.6: Verify `README.md` exists with project overview

- 1.2.7: Run `go mod tidy` to ensure dependencies are clean

- 1.2.8: Committed with message: `chore: verify project structure and Go module`

**Prerequisites:** Story 1.1

**Time Estimate:** 1 hour

**Architecture References:**
- Source tree: `docs/architecture/source-tree.md`

---

## Story 1.3: Implement NoteID Domain Model

As a developer,
I want to implement the NoteID domain model as an opaque identifier,
so that notes are identified without infrastructure coupling.

### Acceptance Criteria

- 1.3.1: Create `internal/domain/note_id.go`:
  - Type: `type NoteID string`
  - Constructor: `NewNoteID(value string) NoteID`
  - Method: `String() string` for logging/debugging

- 1.3.2: Create unit tests in `internal/domain/note_id_test.go`:
  - Test: NewNoteID creates valid instance
  - Test: String() returns underlying value
  - Test: NoteID can be used as map key

- 1.3.3: All tests pass: `go test ./internal/domain`

- 1.3.4: All linting passes: `golangci-lint run internal/domain`

- 1.3.5: Committed with message: `feat(domain): implement NoteID model`

**Prerequisites:** Story 1.2

**Time Estimate:** 1 hour

**Architecture References:**
- Data models: `docs/architecture/data-models.md#noteid` (v0.5.2)

---

## Story 1.4: Implement Frontmatter Domain Model

As a developer,
I want to implement the Frontmatter domain model,
so that note metadata can be represented as pure domain data.

### Acceptance Criteria

- 1.4.1: Create `internal/domain/frontmatter.go`:
  - Field: `FileClass string` (computed from Fields)
  - Field: `Fields map[string]interface{}` (complete YAML frontmatter)
  - Constructor: `NewFrontmatter(fields map[string]interface{}) Frontmatter`
  - Helper: `extractFileClass(fields map[string]interface{}) string`
  - Method: `SchemaName() string` (returns FileClass)

- 1.4.2: Create unit tests in `internal/domain/frontmatter_test.go`:
  - Test: NewFrontmatter constructs correctly
  - Test: FileClass extracted from Fields["fileClass"]
  - Test: FileClass empty when not present
  - Test: SchemaName() returns FileClass

- 1.4.3: All tests pass: `go test ./internal/domain`

- 1.4.4: All linting passes: `golangci-lint run internal/domain`

- 1.4.5: Committed with message: `feat(domain): implement Frontmatter model`

**Prerequisites:** Story 1.2

**Time Estimate:** 1.5 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#frontmatter` (v0.1.0)

---

## Story 1.5: Implement Note Domain Model

As a developer,
I want to implement the Note domain model using NoteID and Frontmatter composition,
so that notes follow clean architecture without embedded infrastructure.

### Acceptance Criteria

- 1.5.1: Create `internal/domain/note.go`:
  - Field: `ID NoteID`
  - Field: `Frontmatter Frontmatter` (composition, NOT embedding)
  - Constructor: `NewNote(id NoteID, frontmatter Frontmatter) Note`
  - Method: `SchemaName() string` (delegates to Frontmatter.SchemaName())

- 1.5.2: Create unit tests in `internal/domain/note_test.go`:
  - Test: NewNote constructs with ID and Frontmatter
  - Test: SchemaName() delegates correctly
  - Test: No embedded File or filesystem paths
  - Test: Note can be serialized to JSON

- 1.5.3: All tests pass: `go test ./internal/domain`

- 1.5.4: All linting passes: `golangci-lint run internal/domain`

- 1.5.5: Committed with message: `feat(domain): implement Note model with NoteID composition`

**Prerequisites:** Story 1.3 (NoteID), Story 1.4 (Frontmatter)

**Time Estimate:** 1.5 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#note` (v0.5.2)

---

## Story 1.6: Implement TemplateID Domain Model

As a developer,
I want to implement the TemplateID domain model,
so that templates are identified by name for Go text/template composition.

### Acceptance Criteria

- 1.6.1: Create `internal/domain/template_id.go`:
  - Type: `type TemplateID string`
  - Constructor: `NewTemplateID(value string) TemplateID`
  - Method: `String() string` for logging/debugging

- 1.6.2: Create unit tests in `internal/domain/template_id_test.go`:
  - Test: NewTemplateID creates valid instance
  - Test: String() returns underlying value
  - Test: TemplateID can be used as map key

- 1.6.3: All tests pass: `go test ./internal/domain`

- 1.6.4: All linting passes: `golangci-lint run internal/domain`

- 1.6.5: Committed with message: `feat(domain): implement TemplateID model`

**Prerequisites:** Story 1.2

**Time Estimate:** 1 hour

**Architecture References:**
- Data models: `docs/architecture/data-models.md#templateid` (v0.5.6)

---

## Story 1.7: Implement Template Domain Model

As a developer,
I want to implement the Template domain model with ID and Content only,
so that templates are pure domain data without infrastructure concerns.

### Acceptance Criteria

- 1.7.1: Create `internal/domain/template.go`:
  - Field: `ID TemplateID`
  - Field: `Content string` (raw template text)
  - Constructor: `NewTemplate(id TemplateID, content string) Template`
  - NO FilePath field (infrastructure concern)
  - NO Parsed field (caching concern)

- 1.7.2: Create unit tests in `internal/domain/template_test.go`:
  - Test: NewTemplate constructs with ID and Content
  - Test: No filesystem dependencies
  - Test: Template can store Go template syntax in Content

- 1.7.3: All tests pass: `go test ./internal/domain`

- 1.7.4: All linting passes: `golangci-lint run internal/domain`

- 1.7.5: Committed with message: `feat(domain): implement Template model`

**Prerequisites:** Story 1.6 (TemplateID)

**Time Estimate:** 1 hour

**Architecture References:**
- Data models: `docs/architecture/data-models.md#template` (v0.5.6)

---

## Story 1.8: Implement Config Domain Model

As a developer,
I want to implement the Config domain model as a value object,
so that configuration is represented in the domain layer.

### Acceptance Criteria

- 1.8.1: Create `internal/domain/config.go`:
  - Field: `VaultPath string` (default: ".")
  - Field: `TemplatesDir string` (default: "templates/")
  - Field: `SchemasDir string` (default: "schemas/")
  - Field: `PropertyBankFile string` (default: "property_bank.json")
  - Field: `CacheDir string` (default: ".lithos/cache/")
  - Field: `LogLevel string` (default: "info")
  - Constructor: `NewConfig(...)` with all fields
  - Constructor: `DefaultConfig()` using defaults

- 1.8.2: Create unit tests in `internal/domain/config_test.go`:
  - Test: NewConfig constructs with all fields
  - Test: DefaultConfig uses correct defaults
  - Test: Config is immutable (copy on modification pattern if needed)

- 1.8.3: All tests pass: `go test ./internal/domain`

- 1.8.4: All linting passes: `golangci-lint run internal/domain`

- 1.8.5: Committed with message: `feat(domain): implement Config model`

**Prerequisites:** Story 1.2

**Time Estimate:** 1.5 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#config` (v0.5.7)

---

## Story 1.9: Implement Base Error Types

As a developer,
I want to implement base error types using idiomatic Go error handling,
so that errors follow Go conventions without Result[T] pattern.

### Acceptance Criteria

- 1.9.1: Create `internal/shared/errors/types.go`:
  - `BaseError` struct with `message string` and `cause error`
  - Implements `Error() string` and `Unwrap() error`
  - Constructor: `NewBaseError(message string, cause error) BaseError`

- 1.9.2: Implement `ValidationError` in same file:
  - Embeds `BaseError`
  - Fields: `property string`, `reason string`, `value interface{}`
  - Constructor: `NewValidationError(...)`
  - Methods: `Property()`, `Reason()`, `Value()`

- 1.9.3: Implement `ResourceError` in same file:
  - Embeds `BaseError`
  - Fields: `resource string`, `operation string`, `target string`
  - Constructor: `NewResourceError(...)`
  - Methods: `Resource()`, `Operation()`, `Target()`

- 1.9.4: Create unit tests in `internal/shared/errors/types_test.go`:
  - Test: Error construction and methods
  - Test: errors.Unwrap() works correctly
  - Test: errors.Is() and errors.As() work correctly

- 1.9.5: **Verify NO Result[T] type exists**

- 1.9.6: All tests pass: `go test ./internal/shared/errors`

- 1.9.7: All linting passes: `golangci-lint run internal/shared/errors`

- 1.9.8: Committed with message: `feat(shared): implement base error types with idiomatic Go`

**Prerequisites:** Story 1.2

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Error Package
- Coding standards: `docs/architecture/coding-standards.md` - Error Handling (v0.6.7)

---

## Story 1.10: Implement Domain-Specific Error Types

As a developer,
I want to implement domain-specific error types for templates, schemas, and frontmatter,
so that errors provide context for each domain area.

### Acceptance Criteria

- 1.10.1: Create `internal/shared/errors/template.go`:
  - `TemplateError` type using BaseError
  - Constructor: `NewTemplateError(message string, templateID string, cause error)`
  - Method: `TemplateID() string`

- 1.10.2: Create `internal/shared/errors/schema.go`:
  - `SchemaError` type using BaseError
  - Constructor: `NewSchemaError(message string, schemaName string, cause error)`
  - Method: `SchemaName() string`

- 1.10.3: Create `internal/shared/errors/frontmatter.go`:
  - `FrontmatterError` type using BaseError
  - Constructor: `NewFrontmatterError(message string, field string, cause error)`
  - Method: `Field() string`

- 1.10.4: Create unit tests for each error type:
  - Test: Construction and accessor methods
  - Test: Error message formatting
  - Test: Unwrapping works correctly

- 1.10.5: All tests pass: `go test ./internal/shared/errors`

- 1.10.6: All linting passes: `golangci-lint run internal/shared/errors`

- 1.10.7: Committed with message: `feat(shared): implement domain-specific error types`

**Prerequisites:** Story 1.9 (base errors)

**Time Estimate:** 1.5 hours

**Architecture References:**
- Change log: v0.5.9 - Error types split

---

## Story 1.11: Implement Error Helper Functions

As a developer,
I want to implement error wrapping helper functions,
so that errors can be wrapped with context throughout the codebase.

### Acceptance Criteria

- 1.11.1: Create `internal/shared/errors/helpers.go`:
  - `Wrap(err error, message string) error` - wraps with fmt.Errorf
  - `WrapWithContext(err error, format string, args ...interface{}) error`
  - Uses Go stdlib `errors.Join()` for multiple errors

- 1.11.2: Create unit tests in `internal/shared/errors/helpers_test.go`:
  - Test: Wrap() preserves original error
  - Test: WrapWithContext() formats message correctly
  - Test: errors.Unwrap() works on wrapped errors

- 1.11.3: All tests pass: `go test ./internal/shared/errors`

- 1.11.4: All linting passes: `golangci-lint run internal/shared/errors`

- 1.11.5: Committed with message: `feat(shared): implement error helper functions`

**Prerequisites:** Story 1.9 (base errors)

**Time Estimate:** 1 hour

**Architecture References:**
- Coding standards: `docs/architecture/coding-standards.md` - Error wrapping with fmt.Errorf

---

## Story 1.12: Implement Logger Package

As a developer,
I want to implement structured logging with zerolog,
so that the application has consistent logging across all components.

### Acceptance Criteria

- 1.12.1: Add `github.com/rs/zerolog` v1.34.0 to `go.mod`

- 1.12.2: Create `internal/shared/logger/logger.go`:
  - `New(output io.Writer, level string) zerolog.Logger`
  - TTY detection via `golang.org/x/term`
  - Level parsing: "debug", "info", "warn", "error"

- 1.12.3: Implement context methods:
  - `WithComponent(logger zerolog.Logger, component string) zerolog.Logger`
  - `WithOperation(logger zerolog.Logger, operation string) zerolog.Logger`
  - `WithCorrelationID(logger zerolog.Logger, id string) zerolog.Logger`

- 1.12.4: Implement test helper:
  - `NewTest() zerolog.Logger` - returns logger with ioutil.Discard

- 1.12.5: Create unit tests in `internal/shared/logger/logger_test.go`:
  - Test: Log level configuration
  - Test: TTY detection switches output format
  - Test: Context methods add fields

- 1.12.6: All tests pass: `go test ./internal/shared/logger`

- 1.12.7: All linting passes: `golangci-lint run internal/shared/logger`

- 1.12.8: Committed with message: `feat(shared): implement logger package with zerolog`

**Prerequisites:** Story 1.2

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Logger
- Tech stack: zerolog v1.34.0

---

## Story 1.13: Implement Registry Package

As a developer,
I want to implement generic thread-safe registry with CQRS interfaces,
so that schemas and templates can be stored in-memory.

### Acceptance Criteria

- 1.13.1: Create `internal/shared/registry/registry.go`:
  - Generic type: `Registry[T any]`
  - Thread-safe with `sync.RWMutex`
  - Storage: `map[string]T`

- 1.13.2: Define CQRS interfaces:
  - `Reader[T any]`: Get(), Exists(), ListKeys()
  - `Writer[T any]`: Register(), Clear()
  - `Registry[T any]` embeds both

- 1.13.3: Implement registry struct and methods:
  - Constructor: `New[T any]() Registry[T]`
  - Get() uses RLock()
  - Register() uses Lock()
  - Returns ErrNotFound for missing keys

- 1.13.4: Create unit tests in `internal/shared/registry/registry_test.go`:
  - Test: Generic instantiation (string, int, struct)
  - Test: Concurrent reads don't block
  - Test: Writes block correctly
  - Test: ErrNotFound for missing keys

- 1.13.5: All tests pass: `go test ./internal/shared/registry`

- 1.13.6: All linting passes: `golangci-lint run internal/shared/registry`

- 1.13.7: Committed with message: `feat(shared): implement thread-safe registry with CQRS`

**Prerequisites:** Story 1.2

**Time Estimate:** 2.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#shared-internal-packages` - Registry

---

## Story 1.14: Implement ConfigPort Interface

As a developer,
I want to define the ConfigPort SPI interface,
so that configuration loading follows hexagonal architecture.

### Acceptance Criteria

- 1.14.1: Create `internal/ports/spi/config.go`:
  - Interface: `ConfigPort`
  - Method: `Load(ctx context.Context) (Config, error)`

- 1.14.2: Add documentation comments:
  - Describe port responsibility
  - Document Load() method contract
  - Reference architecture components.md

- 1.14.3: All linting passes: `golangci-lint run internal/ports/spi`

- 1.14.4: Committed with message: `feat(ports): define ConfigPort SPI interface`

**Prerequisites:** Story 1.8 (Config model)

**Time Estimate:** 0.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#spi-port-interfaces` - ConfigPort (v0.5.11)

---

## Story 1.15: Implement ViperAdapter

As a developer,
I want to implement config loading with Viper,
so that configuration can be loaded from file, environment, and flags.

### Acceptance Criteria

- 1.15.1: Add `github.com/spf13/viper` to `go.mod`

- 1.15.2: Create `internal/adapters/spi/config/viper_adapter.go`:
  - Implements ConfigPort interface
  - Constructor: `NewViperAdapter(log zerolog.Logger) *ViperAdapter`

- 1.15.3: Implement Load() with precedence:
  1. Environment variables (LITHOS_*)
  2. Config file (`lithos.json`)
  3. Defaults from Config.DefaultConfig()

- 1.15.4: Implement upward search for lithos.json:
  - Start from CWD
  - Search upward through parents
  - Use directory containing lithos.json as VaultPath

- 1.15.5: Environment variable mapping:
  - LITHOS_VAULT_PATH → VaultPath
  - LITHOS_LOG_LEVEL → LogLevel
  - (etc. for all Config fields)

- 1.15.6: Validate VaultPath exists and is directory

- 1.15.7: Create unit tests in `internal/adapters/spi/config/viper_adapter_test.go`:
  - Test: Defaults when no config
  - Test: Config file overrides defaults
  - Test: Env vars override config file
  - Test: Upward search finds config

- 1.15.8: All tests pass: `go test ./internal/adapters/spi/config`

- 1.15.9: All linting passes: `golangci-lint run internal/adapters/spi/config`

- 1.15.10: Committed with message: `feat(adapters): implement Viper config adapter`

**Prerequisites:** Story 1.8 (Config), Story 1.12 (Logger), Story 1.14 (ConfigPort)

**Time Estimate:** 3 hours

**Architecture References:**
- Components: `docs/architecture/components.md#spi-adapters` - ViperAdapter

---

## Story 1.16: Implement FileMetadata SPI Model

As a developer,
I want to implement FileMetadata as an SPI adapter model,
so that filesystem metadata is isolated from the domain.

### Acceptance Criteria

- 1.16.1: Create `internal/adapters/spi/file_metadata.go`:
  - Fields: Path, Basename, Folder, Ext, ModTime, Size, MimeType
  - Constructor: `NewFileMetadata(path string, info fs.FileInfo) FileMetadata`
  - Helper: `computeBasename(path string) string`
  - Helper: `computeFolder(path string) string`
  - Helper: `computeMimeType(ext string) string`

- 1.16.2: Create unit tests in `internal/adapters/spi/file_metadata_test.go`:
  - Test: NewFileMetadata constructs correctly
  - Test: Computed fields (Basename, Folder, Ext)
  - Test: MIME type detection

- 1.16.3: All tests pass: `go test ./internal/adapters/spi`

- 1.16.4: All linting passes: `golangci-lint run internal/adapters/spi`

- 1.16.5: Committed with message: `feat(adapters): implement FileMetadata SPI model`

**Prerequisites:** Story 1.2

**Time Estimate:** 1.5 hours

**Architecture References:**
- Data models: `docs/architecture/data-models.md#filemetadata` (v0.5.1)

---

## Story 1.17: Implement TemplatePort Interface

As a developer,
I want to define the TemplatePort SPI interface,
so that template loading follows hexagonal architecture.

### Acceptance Criteria

- 1.17.1: Create `internal/ports/spi/template.go`:
  - Interface: `TemplatePort`
  - Method: `List(ctx context.Context) ([]TemplateID, error)`
  - Method: `Load(ctx context.Context, id TemplateID) (Template, error)`

- 1.17.2: Add documentation comments:
  - Describe port responsibility
  - Document method contracts
  - Reference architecture components.md

- 1.17.3: All linting passes: `golangci-lint run internal/ports/spi`

- 1.17.4: Committed with message: `feat(ports): define TemplatePort SPI interface`

**Prerequisites:** Story 1.6 (TemplateID), Story 1.7 (Template)

**Time Estimate:** 0.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#spi-port-interfaces` - TemplatePort (v0.5.11)

---

## Story 1.18: Implement TemplateLoaderAdapter

As a developer,
I want to implement template loading from filesystem,
so that templates can be loaded via TemplatePort interface.

### Acceptance Criteria

- 1.18.1: Create `internal/adapters/spi/template/template_loader.go`:
  - Implements TemplatePort interface
  - Constructor: `NewTemplateLoaderAdapter(config Config, log zerolog.Logger)`
  - Internal cache: `map[TemplateID]FileMetadata`

- 1.18.2: Implement List() method:
  - Scan Config.TemplatesDir using filepath.Walk
  - Find all .md files
  - Derive TemplateID from basename
  - Build FileMetadata for each
  - Return sorted []TemplateID

- 1.18.3: Implement Load() method:
  - Look up TemplateID in cache
  - Read file with os.ReadFile
  - Return Template(ID, Content)
  - Return ResourceError if not found

- 1.18.4: Create unit tests in `internal/adapters/spi/template/template_loader_test.go`:
  - Test: List() finds templates in testdata
  - Test: Load() reads content correctly
  - Test: Error for missing template

- 1.18.5: Create integration test in `tests/integration/template_loader_test.go`:
  - Use existing testdata/templates/
  - Test: List() finds static-template.md, basic-note.md
  - Test: Load("static-template") returns correct content

- 1.18.6: All tests pass: `go test ./internal/adapters/spi/template ./tests/integration`

- 1.18.7: All linting passes: `golangci-lint run internal/adapters/spi/template`

- 1.18.8: Committed with message: `feat(adapters): implement TemplateLoader adapter`

**Prerequisites:** Story 1.7 (Template), Story 1.8 (Config), Story 1.12 (Logger), Story 1.16 (FileMetadata), Story 1.17 (TemplatePort)

**Time Estimate:** 3 hours

**Architecture References:**
- Components: `docs/architecture/components.md#spi-adapters` - TemplateLoaderAdapter

---

## Story 1.19: Implement TemplateEngine - Load Method

As a developer,
I want to implement the TemplateEngine Load method,
so that templates can be loaded via dependency injection.

### Acceptance Criteria

- 1.19.1: Create `internal/domain/template_engine.go`:
  - Constructor: `NewTemplateEngine(templatePort TemplatePort, config Config, log zerolog.Logger)`
  - Dependencies: TemplatePort, Config, Logger

- 1.19.2: Implement `Load(ctx context.Context, templateID TemplateID) (Template, error)`:
  - Delegates to TemplatePort.Load()
  - Logs loading operation
  - Returns Template or error

- 1.19.3: Create unit tests in `internal/domain/template_engine_test.go`:
  - Test: Load() delegates to port correctly
  - Test: Error propagation from port
  - Use mock TemplatePort from tests/utils/mocks.go

- 1.19.4: Create mock in `tests/utils/mocks.go`:
  - `MockTemplatePort` struct
  - Implements TemplatePort interface
  - Methods: SetTemplates(), SetLoadError()

- 1.19.5: All tests pass: `go test ./internal/domain`

- 1.19.6: All linting passes: `golangci-lint run internal/domain`

- 1.19.7: Committed with message: `feat(domain): implement TemplateEngine Load method`

**Prerequisites:** Story 1.7 (Template), Story 1.8 (Config), Story 1.12 (Logger), Story 1.17 (TemplatePort)

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - TemplateEngine

---

## Story 1.20: Implement TemplateEngine - Basic Functions

As a developer,
I want to implement basic template functions (now, toLower, toUpper),
so that templates can perform simple transformations.

### Acceptance Criteria

- 1.20.1: Add to `internal/domain/template_engine.go`:
  - Private method: `buildFuncMap() template.FuncMap`
  - Function: `now(format string) string` - current time with Go layout
  - Function: `toLower(s string) string`
  - Function: `toUpper(s string) string`

- 1.20.2: Create unit tests in `internal/domain/template_engine_test.go`:
  - Test: now() returns formatted timestamp
  - Test: toLower() converts correctly
  - Test: toUpper() converts correctly
  - Test: Function map is registered

- 1.20.3: All tests pass: `go test ./internal/domain`

- 1.20.4: All linting passes: `golangci-lint run internal/domain`

- 1.20.5: Committed with message: `feat(domain): add basic template functions`

**Prerequisites:** Story 1.19 (TemplateEngine Load)

**Time Estimate:** 1.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - TemplateEngine custom functions

---

## Story 1.21: Implement TemplateEngine - File Path Functions

As a developer,
I want to implement file path control template functions,
so that templates can manipulate paths for note organization.

### Acceptance Criteria

- 1.21.1: Add to `internal/domain/template_engine.go` function map:
  - Function: `path() string` - returns target path (empty for now)
  - Function: `folder(p string) string` - parent directory using filepath.Dir()
  - Function: `basename(p string) string` - filename without extension
  - Function: `extension(p string) string` - file extension using filepath.Ext()
  - Function: `join(parts ...string) string` - filepath.Join()
  - Function: `vaultPath() string` - returns Config.VaultPath

- 1.21.2: Create unit tests in `internal/domain/template_engine_test.go`:
  - Test: folder() returns parent directory
  - Test: basename() strips path and extension
  - Test: extension() returns extension with dot
  - Test: join() uses OS-appropriate separator
  - Test: vaultPath() returns config value

- 1.21.3: All tests pass: `go test ./internal/domain`

- 1.21.4: All linting passes: `golangci-lint run internal/domain`

- 1.21.5: Committed with message: `feat(domain): add file path control template functions`

**Prerequisites:** Story 1.20 (basic functions)

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - File path control functions (v0.6.3)

---

## Story 1.22: Implement TemplateEngine - Render Method

As a developer,
I want to implement the TemplateEngine Render method,
so that templates can be executed with the function map.

### Acceptance Criteria

- 1.22.1: Add to `internal/domain/template_engine.go`:
  - Method: `Render(ctx context.Context, templateID TemplateID) (string, error)`
  - Loads template via Load()
  - Creates text/template with template.ID as name
  - Registers function map via buildFuncMap()
  - Parses template.Content
  - Executes with empty data (static for now)
  - Returns rendered string

- 1.22.2: Error handling:
  - Template not found → ResourceError from Load()
  - Parse error → TemplateError with details
  - Execute error → TemplateError with context

- 1.22.3: Create unit tests in `internal/domain/template_engine_test.go`:
  - Test: Render() executes static template
  - Test: Render() uses now() function
  - Test: Render() uses path functions
  - Test: Parse error returns TemplateError
  - Use mock TemplatePort

- 1.22.4: Create integration test in `tests/integration/template_engine_test.go`:
  - Use testdata/templates/static-template.md
  - Compare output to testdata/golden/static-template-expected.md
  - Verify all functions work

- 1.22.5: All tests pass: `go test ./internal/domain ./tests/integration`

- 1.22.6: All linting passes: `golangci-lint run internal/domain`

- 1.22.7: Committed with message: `feat(domain): implement TemplateEngine Render method`

**Prerequisites:** Story 1.21 (path functions)

**Time Estimate:** 3 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - TemplateEngine Render

---

## Story 1.23: Implement CLICommandPort Interface

As a developer,
I want to define the CLICommandPort API interface,
so that CLI framework integration follows hexagonal architecture.

### Acceptance Criteria

- 1.23.1: Create `internal/ports/api/cli_command.go`:
  - Interface: `CLICommandPort`
  - Method: `Start(ctx context.Context, handler CommandHandler) error`

- 1.23.2: Create `internal/ports/api/command_handler.go`:
  - Interface: `CommandHandler`
  - Method: `NewNote(ctx context.Context, templateID TemplateID) (Note, error)`
  - (Additional methods added in later epics)

- 1.23.3: Add documentation comments:
  - Describe hexagonal callback pattern
  - Document method contracts
  - Reference architecture components.md

- 1.23.4: All linting passes: `golangci-lint run internal/ports/api`

- 1.23.5: Committed with message: `feat(ports): define CLICommandPort and CommandHandler interfaces`

**Prerequisites:** Story 1.5 (Note), Story 1.6 (TemplateID)

**Time Estimate:** 1 hour

**Architecture References:**
- Components: `docs/architecture/components.md#api-port-interfaces` - CLICommandPort (v0.6.4)
- Components: CommandHandler callback interface

---

## Story 1.24: Implement CobraCLIAdapter - Setup

As a developer,
I want to set up the CobraCLIAdapter with Cobra framework,
so that the CLI can parse commands and delegate to domain.

### Acceptance Criteria

- 1.24.1: Add dependencies to go.mod:
  - github.com/spf13/cobra
  - github.com/spf13/pflag

- 1.24.2: Create `internal/adapters/api/cli/cobra_cli.go`:
  - Implements CLICommandPort interface
  - Constructor: `NewCobraCLIAdapter(log zerolog.Logger)`
  - Internal field: `handler CommandHandler`

- 1.24.3: Implement Start() method skeleton:
  - Stores handler parameter
  - Calls buildRootCommand()
  - Executes root command with context
  - Returns error

- 1.24.4: Implement buildRootCommand() private method:
  - Use: "lithos"
  - Short: "Template-driven markdown note generator"
  - SilenceUsage: true
  - SilenceErrors: true

- 1.24.5: Create unit tests in `internal/adapters/api/cli/cobra_cli_test.go`:
  - Test: Start() stores handler correctly
  - Test: buildRootCommand() creates command
  - Use mock CommandHandler from tests/utils/mocks.go

- 1.24.6: Create mock in `tests/utils/mocks.go`:
  - `MockCommandHandler` struct
  - Implements CommandHandler interface
  - Method: SetNewNoteResult()

- 1.24.7: All tests pass: `go test ./internal/adapters/api/cli`

- 1.24.8: All linting passes: `golangci-lint run internal/adapters/api/cli`

- 1.24.9: Committed with message: `feat(adapters): implement CobraCLI adapter setup`

**Prerequisites:** Story 1.12 (Logger), Story 1.23 (CLICommandPort)

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#api-adapters` - CobraCLIAdapter (v0.6.4)

---

## Story 1.25: Implement CobraCLIAdapter - Version Command

As a developer,
I want to implement the version command,
so that users can check the CLI version.

### Acceptance Criteria

- 1.25.1: Add to `internal/adapters/api/cli/cobra_cli.go`:
  - Private method: `buildVersionCommand() *cobra.Command`
  - Command: `lithos version`
  - Short: "Print version information"
  - RunE: prints "lithos v0.1.0"

- 1.25.2: Update buildRootCommand():
  - Add version subcommand via AddCommand()

- 1.25.3: Create unit tests in `internal/adapters/api/cli/cobra_cli_test.go`:
  - Test: version command prints correctly

- 1.25.4: All tests pass: `go test ./internal/adapters/api/cli`

- 1.25.5: All linting passes: `golangci-lint run internal/adapters/api/cli`

- 1.25.6: Committed with message: `feat(adapters): add version command to CLI`

**Prerequisites:** Story 1.24 (CobraCLI setup)

**Time Estimate:** 1 hour

**Architecture References:**
- Components: `docs/architecture/components.md#api-adapters` - CobraCLIAdapter

---

## Story 1.26: Implement CobraCLIAdapter - New Command Structure

As a developer,
I want to implement the new command structure,
so that the command can be registered and parse arguments.

### Acceptance Criteria

- 1.26.1: Add to `internal/adapters/api/cli/cobra_cli.go`:
  - Private method: `buildNewCommand() *cobra.Command`
  - Command: `lithos new [template-id]`
  - Short: "Create a new note from template"
  - Args: cobra.MaximumNArgs(1)
  - Flags: `--view, -v` (boolean) - display content after creation
  - RunE: calls handleNewCommand() (implemented in next story)

- 1.26.2: Update buildRootCommand():
  - Add new subcommand via AddCommand()

- 1.26.3: Create unit tests in `internal/adapters/api/cli/cobra_cli_test.go`:
  - Test: new command parses template-id argument
  - Test: new command parses --view flag

- 1.26.4: All tests pass: `go test ./internal/adapters/api/cli`

- 1.26.5: All linting passes: `golangci-lint run internal/adapters/api/cli`

- 1.26.6: Committed with message: `feat(adapters): add new command structure to CLI`

**Prerequisites:** Story 1.25 (version command)

**Time Estimate:** 1.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#api-adapters` - CobraCLIAdapter SRP decomposition

---

## Story 1.27: Implement CommandOrchestrator

As a developer,
I want to implement CommandOrchestrator that coordinates the NewNote workflow,
so that use cases are orchestrated properly.

### Acceptance Criteria

- 1.27.1: Create `internal/domain/command_orchestrator.go`:
  - Constructor: `NewCommandOrchestrator(cliPort CLICommandPort, templateEngine *TemplateEngine, config Config, log zerolog.Logger)`
  - Implements CommandHandler interface
  - Dependencies: CLICommandPort, TemplateEngine, Config, Logger

- 1.27.2: Implement `Run(ctx context.Context) error`:
  - Calls cliPort.Start(ctx, self)
  - Returns error from CLI

- 1.27.3: Implement `NewNote(ctx context.Context, templateID TemplateID) (Note, error)`:
  - Step 1: Render template via templateEngine.Render()
  - Step 2: Generate NoteID from templateID (basename strategy)
  - Step 3: Create empty Frontmatter (no parsing yet)
  - Step 4: Construct Note(noteID, frontmatter)
  - Step 5: Write file to {Config.VaultPath}/{noteID}.md using os.WriteFile
  - Step 6: Return Note

- 1.27.4: Error handling:
  - Template error → return TemplateError
  - File write error → wrap with WrapWithContext()

- 1.27.5: Create unit tests in `internal/domain/command_orchestrator_test.go`:
  - Test: Run() calls CLICommandPort.Start()
  - Test: NewNote() orchestrates correctly
  - Test: NewNote() generates NoteID from templateID
  - Test: NewNote() writes file to vault
  - Use mocks from tests/utils/mocks.go

- 1.27.6: All tests pass: `go test ./internal/domain`

- 1.27.7: All linting passes: `golangci-lint run internal/domain`

- 1.27.8: Committed with message: `feat(domain): implement CommandOrchestrator`

**Prerequisites:** Story 1.5 (Note), Story 1.8 (Config), Story 1.12 (Logger), Story 1.22 (TemplateEngine Render), Story 1.23 (CLICommandPort)

**Time Estimate:** 3 hours

**Architecture References:**
- Components: `docs/architecture/components.md#domain-services` - CommandOrchestrator (v0.6.4)
- Components: NewNote workflow section

---

## Story 1.28: Implement CobraCLIAdapter - New Command Handler

As a developer,
I want to implement the new command handler logic,
so that the command delegates to CommandOrchestrator and displays results.

### Acceptance Criteria

- 1.28.1: Add to `internal/adapters/api/cli/cobra_cli.go`:
  - Private method: `handleNewCommand(cmd *cobra.Command, args []string) error`
  - Extract template-id from args[0] (error if missing)
  - Create TemplateID from args[0]
  - Call handler.NewNote(cmd.Context(), templateID)
  - Display result via displayNoteCreated()
  - Return formatted error via formatError()

- 1.28.2: Implement `displayNoteCreated(cmd *cobra.Command, note Note)`:
  - Print: `✓ Created: {VaultPath}/{note.ID}.md`
  - If --view flag: print separator + content + separator

- 1.28.3: Implement `formatError(err error) error`:
  - Check error type with errors.As()
  - Format user-friendly messages for each type
  - Return formatted error

- 1.28.4: Update buildNewCommand() RunE to call handleNewCommand()

- 1.28.5: Create unit tests in `internal/adapters/api/cli/cobra_cli_test.go`:
  - Test: handleNewCommand() extracts templateID
  - Test: handleNewCommand() calls handler.NewNote()
  - Test: displayNoteCreated() formats output
  - Test: formatError() formats error types

- 1.28.6: All tests pass: `go test ./internal/adapters/api/cli`

- 1.28.7: All linting passes: `golangci-lint run internal/adapters/api/cli`

- 1.28.8: Committed with message: `feat(adapters): implement new command handler`

**Prerequisites:** Story 1.26 (new command structure), Story 1.27 (CommandOrchestrator)

**Time Estimate:** 2.5 hours

**Architecture References:**
- Components: `docs/architecture/components.md#api-adapters` - CobraCLIAdapter handlers

---

## Story 1.29: Wire Dependency Injection in main.go

As a developer,
I want to wire all components in main.go using constructor injection,
so that the application starts with proper initialization order.

### Acceptance Criteria

- 1.29.1: Implement main() in `cmd/lithos/main.go`:
  - Create context
  - Initialize infrastructure layer (logger, config)
  - Initialize SPI adapters (TemplateLoader)
  - Initialize domain services (TemplateEngine)
  - Initialize API adapters (CobraCLI)
  - Initialize CommandOrchestrator
  - Call orchestrator.Run()
  - Handle errors with log.Fatal()

- 1.29.2: Follow initialization order from architecture v0.6.5:
  - Layer 1: Logger, Config
  - Layer 2: SPI Adapters
  - Layer 3: Domain Services
  - Layer 4: API Adapters
  - Layer 5: CommandOrchestrator

- 1.29.3: Error handling:
  - Config load failure → log.Fatal() with message
  - Application failure → log.Fatal() with message

- 1.29.4: Manual testing:
  - Run `go build -o bin/lithos cmd/lithos/main.go`
  - Run `./bin/lithos version` → prints version
  - Run `./bin/lithos new static-template` → creates note
  - Verify file exists with rendered content

- 1.29.5: All linting passes: `golangci-lint run cmd/lithos`

- 1.29.6: Committed with message: `feat: wire dependency injection in main.go`

**Prerequisites:** Story 1.27 (CommandOrchestrator), Story 1.28 (CLI handler)

**Time Estimate:** 2 hours

**Architecture References:**
- Components: `docs/architecture/components.md#dependency-injection-pattern` (v0.6.5)
- Components: Initialization order

---

## Story 1.30: Create End-to-End Test

As a developer,
I want to create an end-to-end test for the complete workflow,
so that Epic 1 functionality is verified.

### Acceptance Criteria

- 1.30.1: Create `tests/e2e/lithos_new_test.go`:
  - Test: Full application flow from CLI to file creation
  - Setup: Create temporary vault directory
  - Copy testdata template to temp vault
  - Execute: Run lithos new command
  - Verify: File created with correct content
  - Cleanup: Remove temporary directory

- 1.30.2: Test scenarios:
  - Test: `lithos new static-template` creates note
  - Test: `lithos new basic-note` uses basic functions
  - Test: Error when template not found

- 1.30.3: All tests pass: `go test ./tests/e2e`

- 1.30.4: All linting passes: `golangci-lint run tests/e2e`

- 1.30.5: Committed with message: `test: add end-to-end test for lithos new command`

**Prerequisites:** Story 1.29 (main.go wiring)

**Time Estimate:** 2.5 hours

**Architecture References:**
- Testing strategy: `docs/architecture/testing-strategy.md`

---

## Story 1.31: Update Documentation

As a developer,
I want to update project documentation for Epic 1 completion,
so that users can install and use lithos.

### Acceptance Criteria

- 1.31.1: Update `README.md`:
  - Add installation instructions
  - Add quick start guide (create vault, add template, run lithos new)
  - Add configuration reference (all Config fields)
  - Add template function reference (now, path functions)
  - Link to architecture docs

- 1.31.2: Create `CHANGELOG.md`:
  - Version 0.1.0 - Epic 1 Complete
  - List features: CLI, template rendering, basic functions
  - Link to architecture docs

- 1.31.3: Verify all tests pass:
  - Run `go test ./...` → all pass
  - Run `golangci-lint run` → no warnings
  - Run `pre-commit run --all-files` → all pass

- 1.31.4: Manual testing checklist:
  - [ ] lithos version works
  - [ ] lithos new creates notes
  - [ ] --view flag displays content
  - [ ] Template functions work
  - [ ] Config loads from file
  - [ ] Errors are user-friendly

- 1.31.5: Committed with message: `docs: update README and CHANGELOG for Epic 1`

**Prerequisites:** Story 1.30 (e2e test)

**Time Estimate:** 2 hours

**Architecture References:**
- All architecture documents

---

## Epic 1 Completion Summary

### Total Stories: 31
### Estimated Time: 54-62 hours (2.7-3.1 weeks for AI agent)

### Deliverables

✅ **Infrastructure:**
- Development tooling verified (linting, formatting, security)
- Project structure verified (hexagonal architecture)
- testdata structure used correctly

✅ **Domain Models (v0.6.8):**
- NoteID (1.3)
- Frontmatter (1.4)
- Note with NoteID composition (1.5)
- TemplateID (1.6)
- Template (1.7)
- Config (1.8)

✅ **Shared Packages:**
- Error types - idiomatic Go, no Result[T] (1.9-1.11)
- Logger with zerolog (1.12)
- Registry with CQRS (1.13)

✅ **Ports & Adapters:**
- ConfigPort + ViperAdapter (1.14-1.15)
- TemplatePort + TemplateLoaderAdapter (1.17-1.18)
- CLICommandPort + CobraCLIAdapter (1.23-1.26, 1.28)
- FileMetadata SPI model (1.16)

✅ **Domain Services:**
- TemplateEngine with functions (1.19-1.22)
- CommandOrchestrator (1.27)

✅ **Template Functions:**
- Basic: now, toLower, toUpper (1.20)
- Path control: path, folder, basename, extension, join, vaultPath (1.21)

✅ **Testing:**
- Unit tests for all components
- Integration tests in tests/integration/
- End-to-end tests in tests/e2e/
- Mocks in tests/utils/mocks.go

✅ **Documentation:**
- README with quick start (1.31)
- CHANGELOG for v0.1.0 (1.31)

### Architecture Coverage (Epic 1)

**23 of 76 components implemented (30%)**

Components deferred to later epics:
- Epic 2: Schema loading, validation, resolution
- Epic 3: Vault indexing, frontmatter extraction/validation, CQRS cache
- Epic 4: Interactive prompts, fuzzy finder
- Epic 5: Query service, additional use cases
