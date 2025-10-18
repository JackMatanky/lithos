# Epic 1: Foundational CLI & Static Template Engine

This epic establishes the project's backbone. It delivers a runnable Go application with a basic command structure and the core, non-interactive template rendering capability.

## Story 1.1: Establish Test Vault and Golden Files

As a QA specialist, I want to create a foundational test data set, so that all unit and integration tests can use a consistent and predictable set of "golden" files.

### Acceptance Criteria

- 1.1.1: A `testdata` directory is created in the repository.
- 1.1.2: The directory contains a minimal vault structure with sample templates, schemas, and notes.
- 1.1.3: A "golden" output file is created for a simple template to test against in Story 1.9.

## Story 1.2: Foundational Project Setup

As a developer, I want to initialize a Go module with a `.gitignore` file, so that I have a clean, version-controlled foundation for the project.

### Acceptance Criteria

- 1.2.1: The root directory contains a `go.mod` file.
- 1.2.2: The root directory contains a `.gitignore` file.
- 1.2.3: The `.gitignore` file includes entries for Go build artifacts and common OS files.
- 1.2.4: A root `README.md` documents prerequisites, `just` automation usage, and quickstart commands for new contributors.

## Story 1.3: Implement a Runnable `main.go` and Basic CI Build

As a developer, I want a `main.go` that prints a version number and a GitHub Action that builds it, so that I can verify the project compiles and have an automated build process.

### Acceptance Criteria

- 1.3.1: A `main.go` file exists in the project root.
- 1.3.2: Executing `go run.` prints a version string (e.g., "lithos version 0.1.0").
- 1.3.3: A GitHub Actions workflow successfully executes `go build./…` and `go test./…` on every push.
- 1.3.4: The CI pipeline runs `go test -cover` and reports test coverage.

## Story 1.4: Integrate Cobra for Basic Command Structure

As a developer, I want to integrate Cobra to create a root `lithos` command and a `version` subcommand, so that I have a robust CLI framework.

### Acceptance Criteria

- 1.4.1: `github.com/spf13/cobra` is added to `go.mod`.
- 1.4.2: Running `lithos` displays default Cobra help.
- 1.4.3: Running `lithos version` prints the version string.

## Story 1.5: Add Placeholder `new` Command

As a developer, I want to add a `new` command that accepts an optional `<template-path>` argument, so that the command's API is defined.

### Acceptance Criteria

- 1.5.1: A `new` subcommand is added.
- 1.5.2: Running `lithos new my-template.md` prints a "Not Implemented" message.
- 1.5.3: Running `lithos new` without an argument also prints a "Not Implemented" message.

## Story 1.6: Implement Core Template File Reading

As a developer, I want the `new` command to read a template file into memory, so that the content is available for parsing.

### Acceptance Criteria

- 1.6.1: The `new` command reads the file specified by the `<template-path>` argument.
- 1.6.2: The file content is loaded into a string variable.
- 1.6.3: A clear error is shown if the file does not exist.

## Story 1.7: Implement Static Template Parsing

As a developer, I want to parse template content using Go's `text/template` engine, so that I can process a template without dynamic functions.

### Acceptance Criteria

- 1.7.1: The `new` command uses `text/template` to parse the template string.
- 1.7.2: A template with only static text is parsed successfully.
- 1.7.3: A syntax error in the template results in a descriptive error message.

## Story 1.8: Add Basic Date & String Functions to Template Engine

As a developer, I want to add a function map with `now`, `toLower`, and `toUpper`, so that templates can perform basic dynamic operations.

### Acceptance Criteria

- 1.8.1: The parser is initialized with a custom function map.
- 1.8.2: `{{ "HELLO" | toLower }}` renders as `hello`.
- 1.8.3: `{{ now "2006-01-02" }}` renders the current date.
- 1.8.4: The function registration is extensible for future additions.

## Story 1.9: Write Rendered Template to a New File

As a user, I want the `lithos new <template-path>` command to create a new Markdown file, so that I can generate a complete note.

### Acceptance Criteria

- 1.9.1: The command generates a final content string from the template.
- 1.9.2: A new file is created in the current working directory.
- 1.9.3: The new filename is identical to the template's filename.
- 1.9.4: A confirmation message is printed after the file is written.
