# Epic 2: Configuration & Schema Loading

This epic introduces the "brains" of Lithos. It enables the CLI to read configuration files and understand the structure of the user's data through schema definitions.

## Story 2.1: Integrate Viper for Configuration Management

As a developer, I want to integrate Viper to manage configuration from a `lithos.yaml` file, so that settings are easily configurable.

### Acceptance Criteria

- 2.1.1: `github.com/spf13/viper` is added to `go.mod`.
- 2.1.2: The CLI searches for `lithos.yaml` in the current directory and parent directories.
- 2.1.3: A `Config` struct is defined to hold configuration, and Viper unmarshals the file into it.
- 2.1.4: Default values are used if the config file is not found.

## Story 2.2: Define Core Schema Data Structures

As a developer, I want to define Go structs for `Schema` and `Field`, so that I have a typed structure for schema definitions.

### Acceptance Criteria

- 2.2.1: A `Schema` struct is defined with `Name` and `Extends` fields.
- 2.2.2: A `Field` struct is defined with `Name`, `Type`, and `Required`fields.
- 2.2.3: A set of constants for supported field types is created.

## Story 2.3: Load and Parse Schema Files

As a developer, I want to load and parse all schema `.yaml` files from the configured directory, so that schemas are available in memory.

### Acceptance Criteria

- 2.3.1: The CLI reads the `schemas_dir` path from the config.
- 2.3.2: It scans the directory for `.yaml` files.
- 2.3.3: Each file is parsed into a `Schema` struct.
- 2.3.4: All parsed schemas are stored in a map, keyed by their `Name`.
- 2.3.5: Invalid YAML files produce a clear error message.

## Story 2.4: Implement Single-Level Schema Inheritance

As a developer, I want to resolve the `Extends` property for a schema, so that the child schema correctly includes the parent's fields.

### Acceptance Criteria

- 2.4.1: The system processes schemas with a non-empty `Extends` field.
- 2.4.2: Fields from the parent schema are merged into the child.
- 2.4.3: Fields in the child override fields of the same name from the parent.

## Story 2.5: Implement Multi-Level Schema Inheritance

As a developer, I want to resolve a chain of schema inheritance, so that the final schema contains fields from all ancestors.

### Acceptance Criteria

- 2.5.1: The system correctly resolves a chain like C extends B, B extends A.
- 2.5.2: Field override priority is correctly handled (C > B > A).

## Story 2.6: Add Circular Dependency Detection for Schemas

As a developer, I want the schema inheritance logic to detect circular dependencies, so that the application does not enter an infinite loop.

### Acceptance Criteria

- 2.6.1: The system detects direct (A -> B, B -> A) and indirect (A -> B -> C -> A) circular dependencies.
- 2.6.2: A clear error is reported when a circular dependency is found.
