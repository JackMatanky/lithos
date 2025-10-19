# Epic 1: Foundational CLI with Hexagonal Architecture

This epic establishes the project's backbone, delivering a runnable Go application that is correctly structured with a hexagonal architecture from the start. It includes the core, non-interactive template rendering capability.

## Story 1.1: Establish Test Vault and Golden Files

As a QA specialist, I want to create a foundational test data set, so that all unit and integration tests can use a consistent and predictable set of "golden" files.

### Acceptance Criteria

- 1.1.1: A `testdata` directory is created in the repository.
- 1.1.2: The directory contains a minimal vault structure with sample templates, schemas, and notes.
- 1.1.3: A "golden" output file is created for a simple template to test against in later stories.

## Story 1.2: Foundational Project Setup & Structure

As a developer, I want to initialize a Go module and create the hexagonal architecture package structure, so that I have a clean, version-controlled, and architecturally-compliant foundation.

### Acceptance Criteria

- 1.2.1: The root directory contains a `go.mod` file and a comprehensive `.gitignore` file.
- 1.2.2: The `internal` directory is created with the core hexagonal structure: `app`, `domain`, `ports/api`, `ports/spi`, `adapters/api`, `adapters/spi`, and `shared`.
- 1.2.3: A root `README.md` documents prerequisites and project structure.
- 1.2.4: The main entrypoint is created at `cmd/lithos/main.go`.

## Story 1.3: Implement Core Domain Models

As a developer, I want to implement the core domain models, so that the application has a typed representation of its core concepts.

### Acceptance Criteria

- 1.3.1: The `File`, `Frontmatter`, `Note`, and `Template` models are created in `internal/domain/` as per `docs/architecture/data-models.md#file`, `#frontmatter`, `#note`, and `#template`.
- 1.3.2: All models include basic unit tests to verify their structure and behavior.

## Story 1.4: Implement Shared Errors Package

As a developer, I want to implement a shared errors package, so that the application has consistent domain-specific error handling.

### Acceptance Criteria

- 1.4.1: `internal/shared/errors/` package is created with domain-specific error types.
- 1.4.2: Error types include ValidationError, NotFoundError, and ConfigurationError.
- 1.4.3: Errors implement proper error interface with structured information.
- 1.4.4: Package includes unit tests for error creation and formatting.

## Story 1.5: Implement Shared Logger Package

As a developer, I want to implement a shared logger package, so that the application has consistent structured logging.

### Acceptance Criteria

- 1.5.1: `internal/shared/logger/` package is created for structured logging.
- 1.5.2: Logger supports log levels (Debug, Info, Warn, Error).
- 1.5.3: Logger outputs structured JSON format with timestamps and context.
- 1.5.4: Package includes unit tests for logging functionality.

## Story 1.6: Implement Shared Registry Package

As a developer, I want to implement a shared registry package, so that the application has thread-safe storage for shared resources.

### Acceptance Criteria

- 1.6.1: `internal/shared/registry/` package is created for thread-safe storage.
- 1.6.2: Registry supports generic types with Get, Set, and Delete operations.
- 1.6.3: Registry uses `sync.RWMutex` for concurrent access.
- 1.6.4: Package includes unit tests for thread safety and operations.

## Story 1.7: Define Core Ports & Implement FileSystem Adapter

As a developer, I want to define the `FileSystemPort` and implement a local file system adapter, so that domain logic is decoupled from direct file I/O.

### Acceptance Criteria

- 1.7.1: The `FileSystemPort` interface is defined in `internal/ports/spi/` with `ReadFile` and `WriteFileAtomic` methods per `docs/architecture/components.md#spi-port-interfaces`.
- 1.7.2: The `LocalFileSystemAdapter` is implemented in `internal/adapters/spi/filesystem/`, satisfying the `FileSystemPort` per `docs/architecture/components.md#spi-adapters`.
- 1.7.3: The adapter correctly uses the `os` package to perform file operations.
- 1.7.4: The adapter and port are unit-tested.

## Story 1.8: Integrate Cobra and the CLI Adapter

As a developer, I want to integrate the Cobra framework into the `cmd/lithos/main.go` entrypoint via a CLI adapter, so that the application has a robust command structure.

### Acceptance Criteria

- 1.8.1: `github.com/spf13/cobra` is added to `go.mod`.
- 1.8.2: A `CobraCLIAdapter` is created in `internal/adapters/api/cli/`.
- 1.8.3: The `main.go` file executes the Cobra adapter.
- 1.8.4: A root `lithos` command and a `version` subcommand are created and functional.
- 1.8.5: Running `lithos version` prints the version string.

## Story 1.9: Implement the `new` Command and Template Reading

As a developer, I want to add a `new` command that reads a template file using the `FileSystemPort`, so that the command is wired into the architecture.

### Acceptance Criteria

- 1.9.1: A `new` subcommand is added to the Cobra adapter.
- 1.9.2: The command accepts a `<template-path>` argument.
- 1.9.3: The command's `RunE` function is injected with the `FileSystemPort`.
- 1.9.4: The command successfully uses the port to read the specified template file into memory.
- 1.9.5: A clear error is shown if the file does not exist, and the error originates from the adapter, not the command logic itself.

## Story 1.10: Implement Static Template Parsing

As a developer, I want to parse the loaded template content using Go's `text/template` engine, so that I can process a template without dynamic functions.

### Acceptance Criteria

- 1.10.1: The `new` command uses `text/template` to parse the template string read in the previous story.
- 1.10.2: A template with only static text is parsed and executed successfully.
- 1.10.3: A syntax error in the template results in a descriptive error message, propagated correctly through the command.

## Story 1.11: Add Basic Template Functions

As a developer, I want to add a function map with `now`, `toLower`, and `toUpper` to the template engine, so that templates can perform basic dynamic operations.

### Acceptance Criteria

- 1.11.1: The parser is initialized with a custom `template.FuncMap`.
- 1.11.2: `{{ "HELLO" | toLower }}` renders as `hello`.
- 1.11.3: `{{ now "2006-01-02" }}` renders the current date.
- 1.11.4: The function registration is extensible for future additions.

## Story 1.12: Generate Rendered Template Content

As a developer, I want the `new` command to generate the final content string from the rendered template, so that the template processing is complete before file writing.

### Acceptance Criteria

- 1.12.1: The command generates a final content string from the rendered template.
- 1.12.2: Template rendering uses the custom function map with now, toLower, and toUpper functions.

## Story 1.13: Write Rendered Template to File

As a user, I want the `lithos new <template-path>` command to create a new Markdown file using the `FileSystemPort`, so that a complete note is generated atomically.

### Acceptance Criteria

- 1.13.1: The `WriteFileAtomic` method on the `FileSystemPort` is used to write the rendered content to a new file.
- 1.13.2: The new filename is identical to the template's base filename and is created in the current working directory.
- 1.13.3: A confirmation message is printed to standard output after the file is successfully written.
