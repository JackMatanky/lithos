# Lithos

Lithos is a Go-based CLI tool for managing Obsidian vaults with template rendering, schema validation, and vault indexing capabilities. It brings Obsidian-style templating, schema validation, and metadata automation to any terminal workflow.

## Prerequisites

- **Go 1.23+** (minimum version required)
- **Git** (for version control and cloning)
- **Just** task runner (optional but recommended for automation)
- **golangci-lint** (for code quality checks, fetched automatically by provided tasks)

Verify prerequisites with:

```bash
go version
just --version
```

## Quick Start

1. **Clone the repository**

   ```bash
   git clone <repo-url>
   cd lithos
   ```

2. **Install dependencies**

   ```bash
   go mod download
   ```

3. **Run tests and validation**

   ```bash
   # With just (recommended)
   just verify

   # OR without just
   go test ./...
   ```

4. **Build the CLI** (available after Story 1.3)

   ```bash
   go build ./cmd/lithos
   ```

## Development Workflow

| Command              | Purpose                                                    |
|---------------------|------------------------------------------------------------|
| `just --list`       | Discover available automation tasks                        |
| `just verify`       | Run full test suite and linters (recommended)             |
| `just test-unit`    | Run unit tests only                                        |
| `just test-integration` | Run integration tests                                  |
| `just build`        | Compile the CLI into `./bin/lithos`                       |
| `just lint`         | Run `golangci-lint` with project rules                    |

**Alternative without just:**
```bash
go test ./...           # Run all tests
golangci-lint run      # Run linters
go build ./cmd/lithos  # Build binary
```

During development, follow the epic/story sequence outlined in `docs/prd/`. All changes must pass CI pipeline validation.

## Project Structure

```
lithos/
├── cmd/lithos/main.go       # CLI entry point (Story 1.3+)
├── internal/                # Application code following hexagonal architecture
│   ├── domain/              # Core models (File, Note, Schema, Property)
│   ├── app/                 # Domain services & orchestrators
│   ├── ports/               # API/SPI contracts
│   └── adapters/            # External integrations
├── testdata/                # Test fixtures and golden files (Story 1.1)
├── docs/                    # PRD and architecture documentation
├── .lithos/                 # Runtime cache (ignored in version control)
├── go.mod                   # Go module definition
├── .gitignore               # Version control exclusions
└── README.md                # This file
```

For detailed architecture information, see `docs/architecture/source-tree.md`.

## Contributing

All contributions must follow the coding standards defined in `docs/architecture/coding-standards.md`. Key requirements:

- **Pre-commit hooks enforce linting and security checks** (`golangci-lint run`, `gitleaks detect`)
- **All tests must pass** (`go test ./...` or `just verify`)
- **Follow hexagonal architecture patterns** with Result types for error handling
- **Use structured logging** with zerolog (no `fmt.Print*` or `log.*`)

## License

[License information to be added]
