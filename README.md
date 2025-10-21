# Lithos

Lithos is a CLI tool for managing and processing Obsidian vaults, providing schema-driven lookups, template rendering, and interactive input capabilities.

## Prerequisites

- Go 1.23 or later
- Git for version control

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
