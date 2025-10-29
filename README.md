# Lithos

Lithos is a CLI tool for managing and processing Obsidian vaults, providing schema-driven lookups, template rendering, and interactive input capabilities.

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

   Note: Epic 1 supports static template rendering. Interactive prompts are available in Epic 4.

5. **Generate a note from the template:**

   ```bash
   lithos new contact
   ```

6. **Check the generated note:**
   The note will be created in your vault root with the rendered content.

## Configuration Reference

Lithos can be configured through a `lithos.json` file, environment variables, or defaults. The configuration file is searched upward from the current working directory.

### Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vaultPath` | string | `"."` | Root directory of your vault |
| `templatesDir` | string | `"templates/"` | Directory containing template files |
| `schemasDir` | string | `"schemas/"` | Directory containing schema files (Epic 2) |
| `propertyBankFile` | string | `"property_bank.json"` | Property bank filename (Epic 2) |
| `cacheDir` | string | `".lithos/cache/"` | Index cache location (Epic 3) |
| `logLevel` | string | `"info"` | Logging verbosity (debug, info, warn, error) |

### Example Configuration

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "logLevel": "info"
}
```

### Environment Variables

All configuration fields can be overridden using environment variables with the `LITHOS_` prefix:

```bash
export LITHOS_VAULT_PATH="/path/to/vault"
export LITHOS_TEMPLATES_DIR="my-templates/"
export LITHOS_LOG_LEVEL="debug"
```

### Configuration File Search

Lithos searches for `lithos.json` starting from the current working directory and moving upward until the file is found or the root directory is reached.

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

- **`path() string`** - Target file path (empty in Epic 1, populated in Epic 3)
  ```go
  {{ path }} // "" (Epic 1)
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

## Architecture

Lithos follows hexagonal architecture principles to ensure clean separation of concerns and testability.

### Core Principles

- **Domain Layer**: Core business logic with no external dependencies
- **Ports**: Interfaces defining contracts between layers
- **Adapters**: Implementations of ports with external concerns
- **Clean Separation**: Business logic independent of infrastructure

### Key Components

- **Domain Models**: Note, Template, Config, Frontmatter
- **Domain Services**: TemplateEngine, CommandOrchestrator
- **Ports**: CLIPort, CommandPort, TemplatePort, ConfigPort
- **Adapters**: CobraCLIAdapter, TemplateLoaderAdapter, ViperAdapter

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
│       └── main.go                 # Application entrypoint
├── internal/
│   ├── domain/                     # Core business models (File, Frontmatter, Note, Schema, Property)
│   ├── app/                        # Domain services & orchestrators
│   │   ├── command/                # Command orchestration
│   │   ├── indexing/               # Vault indexing services
│   │   ├── query/                  # Query services
│   │   ├── schema/                 # Schema services
│   │   └── template/               # Template services
│   ├── ports/
│   │   ├── api/                    # Driving port interfaces (CLICommandPort and related contracts)
│   │   └── spi/                    # Driven port interfaces (FileSystemPort, Cache ports, SchemaLoaderPort, etc.)
│   ├── adapters/
│   │   ├── api/                    # Driving adapters (Cobra CLI today; Bubble Tea/LSP post-MVP)
│   │   └── spi/
│   │       ├── cache/              # Cache adapters
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
├── .lithos/                        # Runtime cache (ignored in version control)
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

## License

[To be determined]
