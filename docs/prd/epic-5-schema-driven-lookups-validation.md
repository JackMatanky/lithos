# Epic 5: Schema-Driven Lookups & Validation

This epic connects the schema and indexing engines to the template engine, unlocking the full power of dynamic, data-driven note creation. It also implements the final validation step.

## Story 5.1: Define Generic Query Engine

As a developer, I want to define a `Query(filters …Filter)` function and a `Filter` struct, so that I have a flexible way to search the index.

### Acceptance Criteria

- 5.1.1: An `indexer/query.go` file defines a `Filter` struct with `Key`, `Operator`, and `Value` fields.
- 5.1.2: A `Query(storage, filters …Filter)`function is defined that returns a `[]Note` slice.
- 5.1.3: In its initial implementation, the function iterates through all notes and prepares to apply filters.

## Story 5.2: Implement `fileClass` Filter

As a developer, I want to implement the logic for filtering by `fileClass`, so that the query engine can perform its first type of lookup.

### Acceptance Criteria

- 5.2.1: The `Query` function correctly handles a `Filter` where `Key` is "fileClass".
- 5.2.2: The function returns only notes where the frontmatter `fileClass` key matches the filter's `Value`.
- 5.2.3: The filtering is case-insensitive.

## Story 5.3: Implement `query()` and `lookup()` Template Functions

As a template author, I want to use a `query()` and `lookup()` function to retrieve data from the index, so that I can create dynamic templates.

### Acceptance Criteria

- 5.3.1: A generic `query()` function is added to the template engine that takes filters and returns a map of note names to file paths.
- 5.3.2: A convenience `lookup("fileClass")` function is added that calls the `query` function with a predefined `fileClass`filter.
- 5.3.3: If the index is empty or stale, both functions return an empty map and log a warning to the console suggesting the user run `lithos index`.

## Story 5.4: Enhance `suggester` to Handle Map Input

As a template author, I want the `suggester` to accept a map, so that I can present users with a list of note names and get back a file path.

### Acceptance Criteria

- 5.4.1: The `suggester` function is updated to accept a `map[string]string` as its `options` argument.
- 5.4.2: The UI displays the *keys* of the map to the user.
- 5.4.3: The function returns the *value* corresponding to the selected key.
- 5.4.4: A template with `{{ suggester "Select project" (lookup "project") }}` works as expected.

## Story 5.5: Implement Validator Scaffolding

As a developer, I want to create the basic structure for the validation engine, so that I have a framework for adding rules.

### Acceptance Criteria

- 5.5.1: A `validator/validator.go` file is created.
- 5.5.2: It defines a `Validate(note Note, schema Schema)` function that returns a `[]error`slice.
- 5.5.3: Initially, the function always returns an empty slice.

## Story 5.6: Implement "Required Field" Validation Rule

As a developer, I want to add logic to check for missing required fields, so that I can enforce basic schema constraints.

### Acceptance Criteria

- 5.6.1: The `Validate` function iterates through the fields in the `Schema`.
- 5.6.2: If a `Field` is marked as `Required`, the function checks for its presence in the `note.Frontmatter`.
- 5.6.3: A descriptive error is returned if a required field is missing.

## Story 5.7: Implement "Simple Type" Validation Rule

As a developer, I want to add logic to check basic types, so that I can enforce data integrity.

### Acceptance Criteria

- 5.7.1: The `Validate` function checks for `string`, `boolean`, `integer`, and `float` types.
- 5.7.2: It also validates `date` types against Go's standard library `time` package layouts.
- 5.7.3: A type mismatch error is returned if a frontmatter value does not match the schema's `Type`.

## Story 5.8: Add Post-Generation Validation to `new` Command

As a user, I want the `lithos new` command to validate the generated note, so I get immediate feedback on metadata correctness.

### Acceptance Criteria

- 5.8.1: After writing a new file, the `new` command identifies its `fileClass` and loads the corresponding `Schema`.
- 5.8.2: It calls the `Validate` function with the new note's data.
- 5.8.3: If validation fails, each error is printed to the console, formatted as 'Error: <file_path>: <error_message>'.
- 5.8.4: The command exits with a non-zero status code upon validation failure.
