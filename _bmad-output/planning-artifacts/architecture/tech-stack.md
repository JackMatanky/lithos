# Tech Stack

This is the DEFINITIVE technology selection section. All technology choices documented here serve as the single source of truth for the project. Any deviations from these selections must be documented with architectural decision records.

## Cloud Infrastructure

**N/A for MVP** - Lithos operates entirely on local file systems with no cloud dependencies. CI/CD for releases uses GitHub Actions (documented in Infrastructure & Deployment section).

## Technology Stack Table

**Versioning Strategy:** Pin to specific minor versions (e.g., `v1.8.x`) to receive patch updates automatically while maintaining compatibility. Review and update dependencies quarterly. Go version specifies minimum; recommend using latest stable patch release within the minor version.

| Category                       | Technology                        | Version                | Purpose                                        | Rationale                                                                                                                                                                                                                                                 |
| ------------------------------ | --------------------------------- | ---------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Core Language & Runtime**    |
| Language                       | Go                                | 1.23+ (min 1.23.0)     | Primary development language                   | Modern features (iterators, improved errors.Join), excellent CLI ecosystem. Minimum 1.23 provides testing improvements needed for Epic 1.1/4.1.                                                                                                           |
| **CLI & Configuration**        |
| CLI Framework                  | github.com/spf13/cobra            | v1.8.1                 | Command structure & parsing                    | Industry standard for Go CLIs (used by kubectl, gh, hugo). Robust subcommand support, excellent flag handling. PRD explicitly specifies. Enables future TUI adapter without core changes.                                                                 |
| Configuration                  | github.com/spf13/viper            | v1.19.0                | Config file management                         | Seamless Cobra integration, supports YAML/ENV/flags hierarchy. PRD explicitly specifies. Handles `lithos.yaml` directory traversal search.                                                                                                                |
| **Data Processing**            |
| YAML Parsing                   | github.com/goccy/go-yaml          | v1.11.3                | Schema YAML processing                         | Superior performance critical for vault indexing (PRD: <300ms render). Chosen over `go-yaml/yaml` for 2-3x speed improvement. Used for schema validation and processing. Frontmatter parsing handled by goldmark-frontmatter extension.                   |
| Frontmatter Extraction         | go.abhg.dev/goldmark/frontmatter  | v0.2.0                 | Type-safe frontmatter extraction with goldmark | Specialized goldmark extension providing production-ready YAML/TOML frontmatter parsing with type-safe API. Chosen for robust edge case handling, performance optimization, and seamless goldmark integration.                                            |
| Markdown Processing            | github.com/yuin/goldmark          | v1.7.1                 | Fast markdown parser/renderer                  | Extensible markdown parser with AST access for frontmatter extraction, heading parsing, and template rendering. Chosen for performance and CommonMark compliance. Enables future markdown features without custom development.                            |
| **Schema & Property Bank**     |
| Schema Definition Format       | JSON                              | encoding/json (stdlib) | Schema and property bank definitions           | JSON chosen over YAML for MVP simplicity: stdlib unmarshaling, easier Go struct mapping, sufficient expressiveness for schema definitions. Frontmatter remains YAML (Obsidian standard).                                                                  |
| Template Engine                | text/template (stdlib)            | Go 1.23+               | Template rendering & function registry         | Go stdlib template engine. PRD explicitly bases design on this. Zero external dependencies. Provides variable substitution and control flow.                                                                                                              |
| **Storage & Indexing**         |
| Hot Cache (Metadata Index)     | go.etcd.io/bbolt                  | v1.4.3                 | Embedded key/value store for hot metadata      | Pure Go, high-performance key/value store (fork of BoltDB) for O(1) lookups of critical metadata (paths, aliases, file class). Chosen for speed and simplicity. Actively maintained by etcd team. Stored in `.lithos/cache/hot.db`.                       |
| Cold Cache (Query Index)       | modernc.org/sqlite                | v1.31.0                | Embedded SQL database for complex queries      | CGo-free, pure Go SQLite implementation. Enables complex, schema-aware queries on frontmatter fields without external dependencies or a C toolchain. Schema-driven views provide typed, indexed access to frontmatter. Stored in `.lithos/cache/cold.db`. |
| **User Interaction**           |
| Interactive Prompts            | github.com/manifoldco/promptui    | v0.9.0                 | Text input & selection UI                      | Clean API, good terminal UX for Epic 5 interactive features. **Note: Last release Oct 2021 - mature/stable but unmaintained. InteractivePort abstraction enables future migration to `charmbracelet/huh` when adding TUI (Post-MVP Phase 4).**            |
| Fuzzy Finder                   | github.com/ktr0731/go-fuzzyfinder | v0.8.0                 | Interactive template selection                 | fzf-like fullscreen experience, minimal implementation overhead for `lithos find` command. PRD explicitly specifies. Active maintenance.                                                                                                                  |
| Terminal Utilities             | golang.org/x/term                 | v0.22.0                | TTY detection, terminal sizing                 | Required for interactive UI libraries. Enables detecting if output is TTY for color control, terminal width for formatting, and raw mode for prompts. Transitive dependency but documented explicitly.                                                    |
| **Observability**              |
| Logging                        | github.com/rs/zerolog             | v1.33.0                | Structured logging                             | High-performance zero-allocation JSON logger with CLI-friendly pretty-print mode. PRD explicitly specifies. Supports both machine-readable (JSON) and human-readable (colorized) output.                                                                  |
| **Development & Release**      |
| Testing                        | testing (stdlib)                  | Go 1.23+               | Unit & integration tests                       | Table-driven tests, subtests, fuzzing support, test coverage reporting. No external dependencies. Epic 1.1 test vault + Epic 5.1 interactive test harness.                                                                                                |
| Release Tooling                | github.com/goreleaser/goreleaser  | v2.1.0                 | Cross-platform builds & distribution           | De-facto standard for Go releases. Automates building for multiple OS/arch, generates release notes, and publishes to GitHub releases. PRD specifies. Deferred to post-MVP per Epic 1 scope reduction.                                                    |
| **Standard Library Utilities** |
| File System                    | os, path/filepath (stdlib)        | Go 1.23+               | Vault scanning, file I/O                       | Explicit choice to use stdlib over abstraction libraries (e.g., `afero`). Aligns with "Standard Library First" principle. Provides cross-platform path handling.                                                                                          |
| Atomic File Writes             | github.com/moby/sys/atomicwriter  | v0.1.0                 | Atomic cache writes                            | Drop-in replacement for `os.WriteFile` with atomic guarantees (temp file + rename). Prevents partial writes on crash/disk-full. Battle-tested in Docker/Moby production. Used for cache persistence.                                                      |

## Library Maintenance Notes

**Actively Maintained:**

- ✅ Cobra, Viper, go-yaml, zerolog, goreleaser, go-fuzzyfinder, bbolt, sqlite - All are actively maintained.

**Stable but Unmaintained:**

- ⚠️ **promptui** (last release Oct 2021) - Chosen for MVP due to simplicity and proven stability. Hexagonal architecture's InteractivePort abstraction protects against lock-in. **Migration path:** When implementing TUI (Post-MVP Phase 4), migrate to `charmbracelet/huh` v2+ which integrates with BubbleTea framework and provides active maintenance.

**Post-MVP Technology Roadmap:**

- **charmbracelet/huh** - Replace promptui when adding TUI support (Phase 4).
- **charmbracelet/bubbletea** - TUI framework for Phase 4 implementation.

## Technology Details

This section provides a deeper dive into how each key technology is used within the Lithos codebase, including architectural context, rationale, and actionable guidance for developers.

### Guiding Architectural Principles

Two core principles from the architectural review govern how technologies are used. Developers must understand these to contribute effectively.

1.  **Hexagonal Validation Layers**: The location of data validation is strictly defined to keep the domain pure.
    - **Syntactic Validation** (checking format, structure, and types) belongs in the **Adapter Layer**. For example, the Vault adapter is responsible for parsing frontmatter with `goldmark` and validating that it is well-formed YAML.
    - **Semantic Validation** (checking business rules) belongs in the **Domain Layer**. For example, a domain service is responsible for validating that a note's frontmatter complies with the rules defined by its corresponding Schema.
2.  **Rich Domain Models**: The project follows a Rich Domain Model approach, moving away from anemic models (structs that are just data bags).
    - **Anti-Pattern to Avoid**: Previously, entities like `Frontmatter` and `Template` were anemic, with all logic in services (`FrontmatterService`, `TemplateEngine`).
    - **Correct Pattern**: An entity must encapsulate its own business logic. For example, the `Template` entity should have a `Render()` method, and the `Frontmatter` entity should have methods to validate its fields against a schema.

### Standard Library

#### `io/fs`

**Purpose**: Provides core interfaces for file system access, enabling the creation of file system-agnostic code and improving testability.

**Core Interfaces**:

- **`fs.FS`**: An interface representing a file system with an `Open` method. Your functions can accept this interface to work with any file system implementation, including in-memory ones for testing.
- **`fs.FileInfo`**: A standard interface for file metadata (`Name`, `Size`, `Mode`, `ModTime`, `IsDir`).
- **`fs.DirEntry`**: An efficient interface for directory traversal, returned by `fs.ReadDir` and `fs.WalkDir`. Its `Type()` method is faster than `FileInfo()` as it doesn't require a `stat` syscall.

**Key Patterns**:

1. **Data Structures with `fs.FileInfo`**: Instead of creating custom structs that duplicate file metadata, embed `fs.FileInfo` to leverage the standard library.

   ```go
   type FileObject struct {
       Path    string
       Info    fs.FileInfo // Embed the standard interface
       Content []byte
   }
   ```

2. **Extending Metadata**: Use the `Sys()` method on `fs.FileInfo` to provide access to application-specific file metadata.
   ```go
   // A custom FileInfo that includes extra data.
   type CustomFileInfo struct {
       fs.FileInfo // Embedded
       ExtraData string
   }
   // The Sys() method returns the custom implementation.
   func (c *CustomFileInfo) Sys() any { return c }
   ```
3. **Efficient Traversal**: Use `fs.WalkDir` with `fs.DirEntry` for better performance than the older `filepath.Walk`.
4. **Testing**: Use `testing/fstest.MapFS` to create an in-memory filesystem for fast, hermetic tests without touching the disk.

   ```go
   mockFS := fstest.MapFS{
       "notes/note1.md": {Data: []byte("file content")},
   }
   // Pass mockFS to any function that accepts fs.FS
   ```

**Best Practices**:

- Use `fs.WalkDir` and `fs.DirEntry` for directory traversal to avoid unnecessary syscalls.
- Design functions to accept `fs.FS` to make them more flexible and easier to test.
- Use `testing/fstest.MapFS` for unit testing file-handling logic.

#### `path/filepath`

**Purpose**: Provides utilities for cross-platform file path manipulation, essential for writing portable file-handling code.

**Core Functions**:

- **Path Joining**: `filepath.Join()` combines path elements using the correct OS-specific separator. **Always use this over string concatenation.**
- **Path Cleaning**: `filepath.Clean()` lexically simplifies paths (e.g., removing `.` and `..`).
- **Path Extraction**: `filepath.Dir()`, `filepath.Base()`, and `filepath.Ext()` extract the directory, filename, and extension from a path.
- **Traversal**: `filepath.WalkDir()` (Go 1.16+) efficiently walks a file tree. It is preferred over the older `filepath.Walk()`.
- **Cross-Platform Conversion**: `filepath.ToSlash()` and `filepath.FromSlash()` convert between native OS separators and forward slashes, useful for serialization or network transfer.

**Key Differences: `path` vs `filepath`**:

| Package        | Separator   | Use Case                            |
| :------------- | :---------- | :---------------------------------- |
| **`path`**     | Always `/`  | URL paths, portable data structures |
| **`filepath`** | OS-specific | Filesystem operations               |

**Best Practices**:

- **Always use `filepath.Join()`** to construct paths for cross-platform compatibility.
- Use `filepath.WalkDir()` for better performance and lower memory usage compared to `filepath.Walk`.
- When storing or transmitting paths, prefer a portable format (forward slashes using `filepath.ToSlash()`) and convert back to the native format (`filepath.FromSlash()`) only when interacting with the OS.
- Validate user-supplied paths to prevent directory traversal attacks (e.g., by ensuring a cleaned path does not start with `..`).

#### `text/template`

**Purpose**: A powerful data-driven templating engine for generating text output.

**Core Patterns**:

- **Parsing**: Parse templates once on startup and cache them for reuse, as parsing is an expensive operation. `template.ParseFiles()` and `template.ParseGlob()` are common for loading templates from disk.
- **Execution**: Use `Execute()` to render a template with data. Use `ExecuteTemplate()` to render a specific named template from a parsed set.
- **Functions**: Add custom functions to templates using `template.FuncMap`. **Crucially, functions must be registered with the `.Funcs()` method _before_ parsing to avoid a panic.**
- **Composition**: Use `{{template "name" .}}` to include one template within another (e.g., for partials like headers or footers).
- **Inheritance**: Use `{{block "name" .}}...{{end}}` in a base template to define sections that child templates can override.

**Best Practices**:

- Cache parsed templates for performance, especially in server applications. `*template.Template` is safe for concurrent execution.
- Always register custom functions with `.Funcs()` before parsing.
- Validate template syntax at parse time by checking the returned error.
- Be aware that data-related errors (e.g., calling a method on a nil pointer) only occur at runtime during `Execute()`.

### Third-Party Dependencies

#### `github.com/spf13/cobra`

- **Role**: Core CLI Framework.
- **Architectural Pattern**: Cobra provides the primary adapter for user interaction, defined in `internal/adapters/primary/cli`. Its structure allows for a clean separation between CLI definitions and the application's core logic, which is invoked from the commands' `RunE` functions.
- **Developer Guidance**:
  - **Usage Pattern**: To add a new command, create a new file in the `cli` directory, define the `cobra.Command` struct, and add it to the root command in `root.go`'s `init()` function.
  - **Validation**: Use `PersistentPreRunE` for root-level validation (like ensuring the vault path exists) and `PreRunE` for command-specific validation. This keeps the `RunE` function clean and focused on business logic.

#### `github.com/spf13/viper`

- **Role**: Configuration Management.
- **Architectural Pattern**: Viper is used in the `internal/adapters/spi/config` adapter. A key architectural decision is the use of a `sync.Once` pattern to manage the `domain.Config` object as a thread-safe singleton.
- **Developer Guidance**:
  - **Usage Pattern**: Application code should not interact with Viper directly. Instead, it should receive the strongly-typed `domain.Config` struct via dependency injection.
  -   - **Testing Strategy**: When testing a component that needs configuration, create an instance of the `domain.Config` struct directly. The singleton pattern includes test helpers for swapping the global instance in integration tests.

#### `github.com/yuin/goldmark`

**Purpose**: A highly extensible, CommonMark-compliant Markdown parser. Used for processing Markdown, accessing its Abstract Syntax Tree (AST) for metadata extraction, and rendering to HTML.

**Core APIs & Patterns**:

- **Simple Conversion**: `md.Convert(source, writer)` provides a one-shot conversion from Markdown to HTML.
- **AST Access**: `md.Parser().Parse(reader)` returns the root `ast.Node`, allowing for tree traversal with `ast.Walk()`. This is key for extracting metadata like headings and links.
- **Frontmatter Parsing**: Use the `go.abhg.dev/goldmark/frontmatter` extension to parse YAML or TOML frontmatter from a document. The data is made available via `frontmatter.Get(parser.Context)`.

**Example: Extracting Metadata**

```go
import (
    "github.com/yuin/goldmark"
    "github.com/yuin/goldmark/ast"
    "github.com/yuin/goldmark/text"
    "go.abhg.dev/goldmark/frontmatter"
    "github.com/yuin/goldmark/parser"
)

func extractMetadata(source []byte) (map[string]any, []string) {
    md := goldmark.New(goldmark.WithExtensions(&frontmatter.Extender{}))
    ctx := parser.NewContext()
    doc := md.Parser().Parse(text.NewReader(source), parser.WithContext(ctx))

    // Extract frontmatter
    var fm map[string]any
    if fmData := frontmatter.Get(ctx); fmData != nil {
        fmData.Decode(&fm)
    }

    // Extract all links
    var links []string
    ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
        if entering {
            if link, ok := n.(*ast.Link); ok {
                links = append(links, string(link.Destination))
            }
        }
        return ast.WalkContinue, nil
    })

    return fm, links
}
```

**Best Practices**:

- Use a single `ast.Walk` pass to extract all required metadata (e.g., links, headings, task lists) for efficiency.
- Use the `go.abhg.dev/goldmark/frontmatter` extension for robust and easy frontmatter parsing.
- To add custom Markdown syntax, you can implement and register your own `parser.BlockParser` or `parser.InlineParser`.

#### `go.etcd.io/bbolt`

**Purpose**: A pure Go, embedded key/value store (a fork of BoltDB). Ideal for high-performance, persistent caching or simple database needs without external dependencies.

**Core Transaction Patterns**:

- **Read-Only**: `db.View(func(tx *bolt.Tx) error { ... })`. Multiple `View` transactions can run concurrently without locks.
- **Read-Write**: `db.Update(func(tx *bolt.Tx) error { ... })`. Only one `Update` transaction can run at a time, as it holds an exclusive lock. The transaction is automatically committed if `nil` is returned, or rolled back on error.
- **Batch Writes**: `db.Batch(func(tx *bolt.Tx) error { ... })`. Opportunistically batches multiple write calls into a single transaction for high throughput. The provided function must be idempotent, as it may be called multiple times.

**Data Structuring Patterns**:

- **Nested Buckets**: Good for hierarchical data, like `users` within an `organization`. This allows for logical grouping and easy deletion of all items in a nested bucket.
  ```go
  tx.CreateBucketIfNotExists([]byte("orgs")).CreateBucketIfNotExists([]byte("org-123"))
  ```
- **Flat Buckets with Composite Keys**: Good for large, flat datasets. Use a naming convention for keys (e.g., `org-123:user-456`) to enable efficient prefix scans with a cursor.
- **Secondary Indexes**: To query by a value, create a secondary index in a separate bucket that maps the value back to the primary key (e.g., a bucket `users_by_email` might store `jane@example.com -> user-123`).

**Best Practices**:

- Use `db.View()` for all reads and `db.Update()` for all writes.
- Use `db.Batch()` for high-volume, small writes, but only with idempotent operations.
- Avoid long-running read transactions, as they can prevent the database from reclaiming space.
- Keys and values are only valid _within_ the transaction. Copy them if they need to live longer.
- For auto-incrementing integer keys, use `bucket.NextSequence()`.

#### `modernc.org/sqlite`

**Purpose**: A CGo-free, pure Go implementation of SQLite. Enables powerful embedded SQL querying without requiring a C toolchain, which simplifies cross-platform builds.

**Why `modernc.org/sqlite` over `mattn/go-sqlite3`?**

- **Simplicity**: No CGo means no C compiler is needed for builds, making cross-compilation trivial.
- **Tradeoff**: Performance is 10-100% slower than the CGo version. This is acceptable for read-heavy workloads where build simplicity is prioritized.

**Querying JSON Data**:
SQLite's JSON functions are powerful for querying semi-structured data stored in a text column.

1.  **Direct Extraction**: Use `json_extract()` directly in queries. This is flexible but can be slow on large datasets as it parses the JSON for every row.
    ```sql
    SELECT json_extract(metadata, '$.title') FROM documents;
    ```
2.  **Generated Columns**: For better performance, use generated columns to expose JSON properties as if they were real columns.
    - **`VIRTUAL`**: The value is computed on-the-fly during reads. More flexible as the table can be altered.
    - **`STORED`**: The value is computed on write and stored on disk. Faster reads, but less flexible.
    ```sql
    -- Create a VIRTUAL column to expose the 'type' field from a 'metadata' JSON column
    ALTER TABLE documents ADD COLUMN doc_type TEXT AS (json_extract(metadata, '$.type'));
    ```
3.  **Indexing JSON**: You can create indexes on generated columns or directly on `json_extract` expressions to make queries highly performant.
    ```sql
    -- Index the virtual column from the previous example
    CREATE INDEX idx_doc_type ON documents(doc_type);
    ```

**Best Practices**:

- Use `PRAGMA journal_mode=WAL` to allow concurrent reads during writes.
- Use prepared statements (`db.Prepare()`) for performance-critical, repeated queries.
- Wrap bulk inserts in a single transaction to improve write performance.
- For querying JSON, prefer using `VIRTUAL` generated columns with indexes. This provides a good balance of performance and flexibility.

#### `github.com/rs/zerolog`

- **Role**: Structured Logging.
- **Architectural Pattern**: A base logger is configured in `main.go`. Contextual sub-loggers are created and passed down via dependency injection.
- **Developer Guidance**:
  - **Usage Pattern**: Do not use the global `log` package. Accept a `zerolog.Logger` instance in your service's constructor.
  - **Context**: To add context that persists for the life of a component, create a sub-logger in the constructor: `s.logger = parentLogger.With().Str("component", "my_service").Logger()`.
  - **Key Considerations**: Log data as structured key-value pairs (e.g., `.Str("id", id)`), not as part of the message string (`.Msgf("Processing id %s", id)`). This makes logs queryable. Use `.Err(err)` to log errors.
