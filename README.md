# Lithos

Lithos is a CLI tool for managing and processing Obsidian vaults, providing schema-driven lookups, template rendering, interactive input capabilities, and high-performance vault indexing with hybrid BoltDB+SQLite storage.

## Installation

### Prerequisites

- Go 1.23 or later (required for generics support)

### Install via go install

```bash
go install github.com/JackMatanky/lithos@latest
```

This will install the `lithos` binary to your `$GOPATH/bin` directory.

### Troubleshooting

- If you get "no matching versions for query 'latest'", releases have not been published yet. Build from source instead:
  ```bash
  git clone https://github.com/JackMatanky/lithos.git
  cd lithos
  go build ./cmd/lithos
  ```

## Quick Start

1. **Create or navigate to your vault directory:**
   ```bash
   mkdir my-vault
   cd my-vault
   ```

2. **Create configuration file (optional):**
   Create `lithos.json` in vault root:

   ```json
   {
     "vaultPath": ".",
     "templatesDir": "templates/",
     "logLevel": "info"
   }
   ```

   If omitted, lithos uses sensible defaults.

3. **Create templates directory:**

   ```bash
   mkdir templates
   ```

4. **Create your first template:**
   Create `templates/contact.md`:

   ```markdown
   ---
   title: Contact Note
   created: {{ now "2006-01-02" }}
   ---

   # Contact Note

   This contact note was created on {{ now "2006-01-02" }}.

   ## File Information

   - Vault path: {{ vaultPath }}
   - Template location: {{ join (vaultPath) "templates" "contact.md" }}
   ```

    Note: Lithos supports both static template rendering and interactive prompts for dynamic content.

5. **Generate a note from the template:**

   ```bash
   lithos new contact
   ```

6. **Check the generated note:**
    The note will be created in your vault root with the rendered content.

## Schema System

Lithos supports **schemas** to define the structure and validation rules for your notes. Schemas ensure consistent frontmatter properties, automatic validation, and property reuse across your vault.

### Key Features

- **Property Bank:** Single source of truth for reusable property definitions
- **Schema Inheritance:** Extend base schemas with `extends` and customize with `excludes`
- **Type Validation:** Automatic validation for string, number, boolean, date, and file properties
- **Constraint Validation:** Support for regex, enum, min/max, step, directory, and fileClass constraints
- **Property References:** Use `$ref` to reference property bank entries (DRY principle)
- **Actionable Errors:** Error messages include remediation hints for quick fixes

For detailed documentation, see [Schema Documentation](docs/schemas/README.md).

### Quick Example

1. **Create property bank** (`schemas/property_bank.json`):

```json
{
  "standard_title": {
    "type": "string",
    "required": true,
    "metadata": {"description": "Standard title property"}
  },
  "standard_created": {
    "type": "date",
    "required": true,
    "metadata": {"description": "Creation timestamp"}
  }
}
```

2. **Create schema** (`schemas/contact.schema.json`):

```json
{
  "name": "contact",
  "properties": [
    {"$ref": "standard_title"},
    {"$ref": "standard_created"},
    {
      "id": "email",
      "type": "string",
      "required": true,
      "spec": {
        "regex": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
      }
    }
  ]
}
```

3. **Create template** using schema (`templates/contact.md`):

```markdown
---
schema: contact
title: {{ prompt "name" "Contact Name" "" }}
created: {{ now "2006-01-02" }}
email: {{ prompt "email" "Email Address" "" }}
---

# {{ .Frontmatter.title }}

Email: {{ .Frontmatter.email }}
```

4. **Generate note** with automatic validation:

```bash
lithos new contact
```

The frontmatter will be automatically validated against the schema before the note is created.

## Vault Indexing System

Lithos features a high-performance hybrid vault indexing system that combines BoltDB and SQLite for optimal query performance and storage efficiency.

### What is Vault Indexing?

Vault indexing is the process of scanning your Obsidian vault, extracting metadata from notes (frontmatter, links, headings, tags), and building optimized data structures for fast queries. This enables features like:

- **Instant note lookup** by path, title, or content
- **Schema validation** during indexing
- **Link analysis** and backlink computation
- **Tag and heading queries** across your entire vault

### Architecture Overview

The vault indexing system uses a **hybrid storage approach** with two complementary databases:

- **BoltDB**: Fast key-value storage for metadata and quick lookups (<1ms response time)
- **SQLite**: Relational storage for complex queries and aggregations (<50ms response time)
- **Smart Routing**: Automatic query optimization that chooses the best storage backend

### Key Features

- **Automatic Indexing**: Scans vault files, extracts frontmatter, and validates against schemas
- **Hybrid Storage**: BoltDB for fast metadata access, SQLite for complex relational queries
- **Query Optimization**: Smart routing between storage backends based on query patterns
- **Incremental Updates**: Efficient updates without full re-indexing
- **Schema Validation**: Real-time validation during indexing with detailed error reporting
- **Performance Monitoring**: Built-in metrics and statistics for optimization

### Performance Characteristics

- **Indexing Speed**: ~500-1000 files/second depending on file complexity
- **Query Performance**: Sub-millisecond for metadata lookups, milliseconds for complex queries
- **Memory Usage**: Minimal memory footprint with streaming processing
- **Storage Efficiency**: Compressed storage with deduplication

### Configuration Options

The vault indexing system supports configurable storage options:

```json
{
  "vaultPath": ".",
  "cacheDir": ".lithos/cache/",
  "file_class_key": "type",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 4,
    "batchSize": 100
  }
}
```

- **`file_class_key`**: Frontmatter property used for file classification (default: "type")
- **`cacheDir`**: Directory for index storage (default: ".lithos/cache/")
- **`enableValidation`**: Enable/disable schema validation during indexing
- **`maxConcurrency`**: Maximum concurrent file processing workers
- **`batchSize`**: Number of files to process in each batch

## Configuration Reference

Lithos can be configured through a `lithos.json` file, environment variables, or defaults. Configuration follows a hierarchical precedence: CLI flags > environment variables > config file > defaults.

**Configuration File Location:**
Lithos searches for `lithos.json` starting from the current working directory and moving upward until found or reaching the root directory.

### Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vaultPath` | string | `"."` | Root directory of your vault |
| `templatesDir` | string | `"templates/"` | Directory containing template files |
| `schemasDir` | string | `"schemas/"` | Directory containing schema files (Epic 2) |
| `propertyBankFile` | string | `"property_bank.json"` | Property bank filename (Epic 2) |
| `cacheDir` | string | `".lithos/cache/"` | Index cache location (Epic 3) |
| `file_class_key` | string | `"type"` | Frontmatter property for file classification (Epic 3) |
| `logLevel` | string | `"info"` | Logging verbosity (debug, info, warn, error) |
| `indexing.enableValidation` | boolean | `true` | Enable schema validation during indexing |
| `indexing.maxConcurrency` | number | `4` | Maximum concurrent file processing workers |
| `indexing.batchSize` | number | `100` | Number of files to process in each batch |

### Example Configuration

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "file_class_key": "type",
  "logLevel": "info",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 4,
    "batchSize": 100
  }
}
```

### Environment Variables

All configuration fields can be overridden using environment variables with the `LITHOS_` prefix:

```bash
export LITHOS_VAULT_PATH="/path/to/vault"
export LITHOS_TEMPLATES_DIR="my-templates/"
export LITHOS_SCHEMAS_DIR="my-schemas/"
export LITHOS_PROPERTY_BANK_FILE="my-property-bank.json"
export LITHOS_CACHE_DIR=".lithos/cache/"
export LITHOS_FILE_CLASS_KEY="category"
export LITHOS_LOG_LEVEL="debug"
export LITHOS_INDEXING_ENABLE_VALIDATION="true"
export LITHOS_INDEXING_MAX_CONCURRENCY="8"
export LITHOS_INDEXING_BATCH_SIZE="200"
```

### Configuration File Search

Lithos searches for `lithos.json` starting from the current working directory and moving upward until the file is found or the root directory is reached.

## Schema Quick Start

Get started with schema-based note creation in 6 steps:

1. **Create schemas/ directory:**
   ```bash
   mkdir schemas
   ```

2. **Create property_bank.json:**
   ```json
   {
     "standard_title": {
       "type": "string",
       "required": true,
       "metadata": {"description": "Standard title property"}
     },
     "standard_created": {
       "type": "date",
       "required": true,
       "metadata": {"description": "Creation timestamp"}
     }
   }
   ```

3. **Create contact.schema.json:**
   ```json
   {
     "name": "contact",
     "properties": [
       {"$ref": "standard_title"},
       {"$ref": "standard_created"},
       {
         "id": "email",
         "type": "string",
         "required": true,
         "spec": {
           "regex": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
         }
       }
     ]
   }
   ```

4. **Create template using schema:**
   ```markdown
   ---
   schema: contact
   title: {{ prompt "name" "Contact Name" "" }}
   created: {{ now "2006-01-02" }}
   email: {{ prompt "email" "Email Address" "" }}
   ---

   # {{ .Frontmatter.title }}

   Email: {{ .Frontmatter.email }}
   ```

5. **Run lithos new contact:**
   ```bash
   lithos new contact
   ```

6. **Check generated file:**
   The output file will have frontmatter matching the schema structure and validated properties.

## Error Messages

Lithos provides actionable error messages with remediation hints to help you quickly resolve issues.

### Common Schema Errors

**Missing Property Bank:**
```
Error: Property bank file not found at schemas/property_bank.json
Hint: Create the property bank file with your reusable property definitions
```

**Circular Inheritance:**
```
Error: Circular inheritance detected in schema chain: contact -> person -> contact
Hint: Remove the circular reference by changing the extends field
```

**Missing $ref Target:**
```
Error: Property reference '$ref: "missing_prop"' not found in property bank
Hint: Add the missing property to schemas/property_bank.json or fix the reference
```

### Vault Indexing Errors

**Vault Path Not Found:**
```
Error: vault indexing failed: vault path does not exist
Hint: Ensure the vaultPath in lithos.json points to a valid directory
```

**Permission Denied:**
```
Error: vault indexing failed: permission denied accessing vault
Hint: Check file permissions and ensure read access to vault directory
```

**Schema Validation Failures:**
```
Warning: Validation failures: 3
Hint: Run with --verbose flag to see detailed validation errors
```

**Cache Write Errors:**
```
Warning: Cache failures: 2
Hint: Check disk space and permissions for cache directory
```

### Configuration Errors

**Invalid File Class Key:**
```
Error: fileClassKey cannot be empty
Hint: Set file_class_key to a valid frontmatter property name (e.g., "type")
```

**Invalid Concurrency:**
```
Error: indexing.maxConcurrency must be between 1 and 16
Hint: Set maxConcurrency to a valid value based on your system capabilities
```

**Missing Required Configuration:**
```
Error: vaultPath is required
Hint: Set vaultPath in lithos.json or LITHOS_VAULT_PATH environment variable
```

### Template Errors

**Template Not Found:**
```
Error: template 'contact' not found in templates/
Hint: Ensure the template file exists in the templates directory
```

**Template Parsing Error:**
```
Error: template error in 'contact': parse error near line 15
Hint: Check template syntax and ensure all functions are properly closed
```

### Troubleshooting Guide

#### Slow Indexing Performance

**Symptoms:**
- `lithos index` takes longer than expected
- High CPU usage during indexing

**Solutions:**
1. **Reduce concurrency:** Set `indexing.maxConcurrency` to 2-4
2. **Increase batch size:** Set `indexing.batchSize` to 200-500
3. **Disable validation:** Set `indexing.enableValidation` to false for bulk operations
4. **Check disk I/O:** Ensure fast storage for cache directory

#### Outdated Query Results

**Symptoms:**
- Queries return old or missing data
- Recent notes don't appear in search results

**Solutions:**
1. **Rebuild index:** Run `lithos index` to refresh the cache
2. **Check file permissions:** Ensure write access to cache directory
3. **Verify configuration:** Check `file_class_key` matches your frontmatter properties

#### High Memory Usage

**Symptoms:**
- Application uses excessive RAM
- System becomes unresponsive during indexing

**Solutions:**
1. **Reduce batch size:** Lower `indexing.batchSize` to 50-100
2. **Process incrementally:** Index smaller vault sections separately
3. **Monitor system resources:** Ensure adequate RAM for vault size

#### Schema Validation Issues

**Symptoms:**
- Notes fail validation during creation
- Indexing shows validation failures

**Solutions:**
1. **Check schema syntax:** Validate JSON syntax in schema files
2. **Verify property references:** Ensure all `$ref` targets exist in property bank
3. **Review inheritance:** Check for circular dependencies in `extends` fields
4. **Update frontmatter:** Ensure note frontmatter matches schema requirements

#### Template Rendering Problems

**Symptoms:**
- Template execution fails
- Unexpected output from `lithos new`

**Solutions:**
1. **Check template syntax:** Validate Go template syntax
2. **Verify function usage:** Ensure template functions are properly called
3. **Test interactively:** Use `lithos new --view` to see rendering progress
4. **Check dependencies:** Ensure required templates and schemas exist

For detailed error handling strategies, see [Error Handling Strategy](docs/architecture/error-handling-strategy.md).

## Template Function Reference

Lithos templates use Go's text/template syntax with custom functions for dynamic content generation.

### Basic Functions

- **`now(format string) string`** - Current timestamp with Go time layout
  ```go
  {{ now "2006-01-02" }} // 2025-10-28
  {{ now "2006-01-02T15:04:05Z" }} // 2025-10-28T14:30:00Z
  ```

- **`toLower(s string) string`** - Convert string to lowercase
  ```go
  {{ toLower "HELLO WORLD" }} // hello world
  ```

- **`toUpper(s string) string`** - Convert string to uppercase
  ```go
  {{ toUpper "hello world" }} // HELLO WORLD
  ```

### File Path Control Functions

- **`path() string`** - Target file path for the note being created
  ```go
  {{ path }} // "/vault/notes/my-note.md"
  ```

- **`folder(p string) string`** - Parent directory of path
  ```go
  {{ folder "/vault/notes/contact.md" }} // /vault/notes
  ```

- **`basename(p string) string`** - Filename without extension
  ```go
  {{ basename "/vault/notes/contact.md" }} // contact
  ```

- **`extension(p string) string`** - File extension with dot
  ```go
  {{ extension "/vault/notes/contact.md" }} // .md
  ```

- **`join(parts ...string) string`** - Join path segments (OS-appropriate separator)
  ```go
  {{ join (vaultPath) "templates" "contact.md" }} // /vault/templates/contact.md
  ```

- **`vaultPath() string`** - Vault root directory from configuration
  ```go
  {{ vaultPath }} // /vault
  ```

## CLI Commands

### version

Print version information.

```bash
lithos version
# Output: lithos v0.1.0
```

### new

Create a new note from a template.

```bash
# Create note from template
lithos new contact

# Create note and display content
lithos new contact --view
```

#### Options

- `--view, -v`: Display note content after creation

#### Examples

```bash
# Create a contact note
lithos new contact
# Output: ✓ Created: contact.md

# Create and view a meeting note
lithos new meeting --view
# Output: ✓ Created: meeting.md
#         ===================================================
#         [note content displayed]
#         ===================================================
```

#### Error Handling

- Template not found: `template 'contact' not found in templates/`
- Template parsing error: `template error in 'contact': parse error...`

### index

Rebuild the vault cache and query indices using the hybrid BoltDB+SQLite storage system. This command scans your vault, extracts frontmatter, validates against schemas, and updates both storage backends for optimal query performance.

**When should you run this command?**
- After adding new notes to your vault
- After modifying existing note frontmatter
- After changing schema definitions
- Before running queries that require up-to-date indices
- After vault restructuring or reorganization

```bash
# Rebuild vault cache and indices
lithos index
```

#### Output

```bash
$ lithos index
✓ Vault indexed successfully

Statistics:
  Scanned:    150 files
  Indexed:    142 notes
  ⚠ Validation failures: 5
  ⚠ Cache failures:      3
  Duration:   234ms
```

#### Storage Details

The index command updates two storage systems:

- **BoltDB Cache**: Fast key-value storage for metadata and file locations
- **SQLite Database**: Relational storage for complex queries and aggregations
- **Hybrid Optimization**: Automatic data distribution based on access patterns

#### When to Use

- After adding new notes to the vault
- After modifying existing note frontmatter
- After changing schema definitions
- After updating the `file_class_key` configuration
- After manual cache corruption recovery
- Before running queries that require up-to-date indices

#### Performance Notes

- **Large Vaults**: Consider increasing `maxConcurrency` for better performance
- **Validation**: Schema validation can be disabled with `enableValidation: false` for faster indexing
- **Incremental**: Only changed files are re-processed when possible
- **Memory**: Uses streaming processing to handle large vaults efficiently

#### Error Handling

- Vault path not found: `vault indexing failed: vault path does not exist`
- Permission denied: `vault indexing failed: permission denied accessing vault`
- Schema validation errors: Displayed as "Validation failures" with count
- Cache write errors: Displayed as "Cache failures" with count
- Configuration errors: Detailed messages for invalid `file_class_key` or storage paths

## Architecture

Lithos follows hexagonal architecture principles to ensure clean separation of concerns and testability.

### Core Principles

- **Domain Layer**: Core business logic with no external dependencies
- **Ports**: Interfaces defining contracts between layers
- **Adapters**: Implementations of ports with external concerns
- **Clean Separation**: Business logic independent of infrastructure

### Key Components

- **Domain Models**: Note, Template, Config, Frontmatter, IndexStats
- **Domain Services**: TemplateEngine, CommandOrchestrator, VaultIndexer, QueryService
- **Ports**: CLIPort, CommandPort, TemplatePort, ConfigPort, CachePort, QueryPort
- **Adapters**: CobraCLIAdapter, TemplateLoaderAdapter, ViperAdapter, BoltDBCacheAdapter, SQLiteAdapter

For detailed architecture documentation, see [docs/architecture/](docs/architecture/).

## Contributing

### Development Setup

1. Ensure Go 1.23+ is installed
2. Clone the repository: `git clone https://github.com/JackMatanky/lithos.git`
3. Install dependencies: `go mod tidy`
4. Build: `go build ./cmd/lithos`
5. Run tests: `go test ./...`

### Code Standards

- Follow Go coding standards and effective Go practices
- Use the Result pattern for error handling in application core
- Maintain hexagonal architecture separation
- Write comprehensive unit tests
- Document packages and public functions

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality:

- `golangci-lint run` - Linting and static analysis
- `go vet` - Go static analysis
- `gofmt` and `goimports` - Code formatting
- `gitleaks` - Secret detection

Run hooks manually: `pre-commit run --all-files`

### Architecture Guidelines

- Domain models in `internal/domain/`
- Business logic in `internal/app/`
- Interfaces in `internal/ports/`
- Implementations in `internal/adapters/`
- No circular dependencies between adapters

### Testing

- Unit tests for all business logic
- Integration tests for adapter interactions
- Use table-driven tests where appropriate
- Maintain high test coverage (>70% for domain/app layers)

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make changes following the architecture guidelines
4. Add tests for new functionality
5. Ensure all tests pass and linting is clean
6. Update documentation if needed
7. Submit a pull request with a clear description

For more details, see the [architecture documentation](docs/architecture/).

## Project Structure

This project follows hexagonal architecture principles to ensure clean separation of concerns and testability.

```
lithos/
├── cmd/
│   └── lithos/
│       └── main.go                 # Application entrypoint with dependency injection
├── internal/
│   ├── domain/                     # Core business models (File, Frontmatter, Note, Schema, Property, IndexStats)
│   ├── app/                        # Domain services & orchestrators
│   │   ├── command/                # Command orchestration
│   │   ├── vault/                  # Vault indexing services (VaultIndexer)
│   │   ├── query/                  # Query services (hybrid BoltDB+SQLite)
│   │   ├── schema/                 # Schema services
│   │   └── template/               # Template services
│   ├── ports/
│   │   ├── api/                    # Driving port interfaces (CLICommandPort and related contracts)
│   │   └── spi/                    # Driven port interfaces (FileSystemPort, Cache ports, SchemaLoaderPort, etc.)
│   ├── adapters/
│   │   ├── api/                    # Driving adapters (Cobra CLI today; Bubble Tea/LSP post-MVP)
│   │   └── spi/
│   │       ├── cache/              # Cache adapters (BoltDB, SQLite)
│   │       ├── config/             # Configuration adapters
│   │       ├── filesystem/         # Filesystem adapters
│   │       ├── interactive/        # Interactive UI adapters
│   │       ├── schema/             # Schema loading adapters
│   │       └── template/           # Template repository adapters
│   └── shared/                     # Cross-cutting concerns (logger, errors, registry, utilities)
├── pkg/                            # Reserved for future public modules
├── templates/                      # Default template pack shipped with CLI
├── schemas/                        # User-defined schemas + property banks
├── testdata/                       # Golden vault used in automated tests (from Story 1.1)
├── .lithos/                        # Runtime cache (BoltDB + SQLite files)
└── docs/                           # PRD, architecture, elicitation summaries
```

## Architecture Principles

### Hexagonal Architecture

- **Domain**: Core business logic with no external dependencies
- **Ports**: Interfaces defining contracts between layers
- **Adapters**: Implementations of ports with external concerns
- **Shared**: Common utilities and cross-cutting concerns

### Key Principles

- Clear separation between business logic and infrastructure
- Dependency inversion through ports and adapters
- Testability through interface-based design
- Standard library first approach (minimal external dependencies)

## Build and Development

### Building

```bash
# Build the main binary
go build ./cmd/lithos

# Run tests
go test ./...

# Run with race detection
go test -race ./...
```

### Development Setup

1. Clone the repository
2. Ensure Go 1.23+ is installed
3. Run `go mod tidy` to download dependencies
4. Build and test: `go build ./cmd/lithos && go test ./...`

## Usage

Basic usage (to be expanded as features are implemented):

```bash
# Display help
./lithos --help

# Process a vault (placeholder)
./lithos process --vault /path/to/vault
```

## Contributing

### Code Standards

- Follow Go coding standards and effective Go practices
- Use the Result pattern for error handling in application core
- Maintain hexagonal architecture separation
- Write comprehensive unit tests
- Document packages and public functions

### Architecture Guidelines

- Domain models in `internal/domain/`
- Business logic in `internal/app/`
- Interfaces in `internal/ports/`
- Implementations in `internal/adapters/`
- No circular dependencies between adapters

### Testing

- Unit tests for all business logic
- Integration tests for adapter interactions
- Use table-driven tests where appropriate
- Maintain high test coverage (>80%)

### Production Test Data

**Real Obsidian Vault**: `docs/refs/obsidian/` (gitignored, 70+ MB)
- Contains Jack's real Obsidian vault with production frontmatter patterns
- Use for performance validation and realistic testing scenarios
- Access via: `ls docs/refs/obsidian/` and `find docs/refs/obsidian/ -name "*.md"`
- Extract subsets to `testdata/` for specific test scenarios

## License

[To be determined]
