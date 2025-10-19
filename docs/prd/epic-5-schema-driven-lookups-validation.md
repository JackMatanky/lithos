# Epic 5: Schema-Driven Lookups & Validation

This epic connects the schema and indexing engines to the template engine, unlocking the full power of dynamic, data-driven note creation. It also implements the final validation step. This epic integrates QueryService with TemplateEngine and implements post-generation validation through CommandOrchestrator following the hexagonal architecture.

**Dependencies:** Epic 4 (Interactive Input Engine)

## Story 5.1: Enhance Query Service with Filter Support

As a developer, I want to enhance the QueryService with sophisticated filtering capabilities, so that template functions can perform complex queries following the architectural patterns.

### Acceptance Criteria

- 5.1.1: QueryService includes Filter struct with Key, Operator, and Value fields for flexible searching.
- 5.1.2: Query method accepts filter parameters and returns filtered Note slices through CacheQueryPort.
- 5.1.3: Initial implementation supports basic iteration with preparation for advanced filtering.
- 5.1.4: Service maintains thread-safe access patterns using `sync.RWMutex` as specified in components documentation.

## Story 5.2: Implement `fileClass` Filter

As a developer, I want to implement the logic for filtering by `fileClass`, so that the query engine can perform its first type of lookup.

### Acceptance Criteria

- 5.2.1: The `Query` function correctly handles a `Filter` where `Key` is "fileClass".
- 5.2.2: The function returns only notes where the frontmatter `fileClass` key matches the filter's `Value`.
- 5.2.3: The filtering is case-insensitive.

## Story 5.3: Integrate Query Service with Template Engine

As a template author, I want to use `query()` and `lookup()` functions in templates, so that I can create dynamic templates that reference indexed vault data.

### Acceptance Criteria

- 5.3.1: TemplateEngine function map includes `query()` function that calls QueryService through closure.
- 5.3.2: Convenience `lookup("fileClass")` function calls QueryService with predefined fileClass filter.
- 5.3.3: Template functions return maps of note names to file paths for template consumption.
- 5.3.4: Functions handle empty/stale index gracefully with appropriate logging through structured logger.

## Story 5.4: Enhance `suggester` to Handle Map Input

As a template author, I want the `suggester` to accept a map, so that I can present users with a list of note names and get back a file path.

### Acceptance Criteria

- 5.4.1: The `suggester` function is updated to accept a `map[string]string` as its `options` argument.
- 5.4.2: The UI displays the *keys* of the map to the user.
- 5.4.3: The function returns the *value* corresponding to the selected key.
- 5.4.4: A template with `{{ suggester "Select project" (lookup "project") }}` works as expected.

## Story 5.5: Implement Basic Schema Validation Enhancement

As a developer, I want to enhance the SchemaValidator with basic validation rules, so that frontmatter validation follows the PropertySpec architecture.

### Acceptance Criteria

- 5.5.1: SchemaValidator implements validation using PropertySpec polymorphism per `docs/architecture/data-models.md#propertyspec`.
- 5.5.2: Validation checks Required fields and Array constraints.

## Story 5.6: Implement PropertySpec Type Validation

As a developer, I want to implement type-specific validation for PropertySpec, so that all PropertySpec types are properly validated.

### Acceptance Criteria

- 5.6.1: Each PropertySpec implements its own Validate method with type-specific rules (enum, pattern, min/max, format).
- 5.6.2: Service returns structured ValidationError instances with detailed field-level information.

## Story 5.7: Implement Post-Generation Validation in Command Orchestrator

As a user, I want the `lithos new` command to validate generated notes through the proper service architecture, so I get immediate feedback on metadata correctness.

### Acceptance Criteria

- 5.7.1: CommandOrchestrator.New method includes post-generation validation step after template rendering.
- 5.7.2: Orchestrator uses SchemaRegistry to lookup schema by fileClass and SchemaValidator for validation.
- 5.7.3: Validation errors are formatted using structured error types and logged through proper service layers.
- 5.7.4: CLI adapter receives validation results and provides appropriate exit codes and user feedback.

## Story 5.8: Implement Advanced FilePropertySpec Validation

As a developer, I want to implement FilePropertySpec validation with dynamic lookups, so that file references can be validated against the vault index.

### Acceptance Criteria

- 5.8.1: FilePropertySpec validation uses QueryService to validate file references against actual vault contents.
- 5.8.2: FileClass and Directory constraints are checked against indexed notes following the specification.
- 5.8.3: Validation supports both absolute paths and wikilink format `[[basename]]` references.
- 5.8.4: Filter conjunction (FileClass AND Directory) and negation patterns are properly implemented.
