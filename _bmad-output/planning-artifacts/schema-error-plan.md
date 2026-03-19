# Schema Error Taxonomy and Implementation Plan

## Purpose
Define and implement a clear, complete error taxonomy for the schema module so
each pipeline phase reports precise, actionable errors. This plan covers the
full schema module, including error types that should exist even if not currently
represented in code, and lays out an implementation approach.

## Goals
- Provide a phase-oriented error model aligned with the schema pipeline
  (ingest -> raw validation -> expansion -> inheritance -> resolution -> storage).
- Eliminate generic or uninformative error variants by introducing structured
  error types that carry context (path, name, id, expected vs found, etc.).
- Preserve source chains and avoid lossy string-only errors.
- Maintain a stable public facade for external callers while enabling internal
  precision.

## Non-Goals
- No functional refactors of schema logic beyond error propagation updates.
- No changes to behavior of the pipeline or storage semantics.

## Design Principles
- Prefer structured error enums over free-form strings.
- Use `thiserror` with `#[from]` and `#[error(transparent)]` for conversions.
- Keep immutable string fields as `Box<str>` for memory efficiency.
- Preserve I/O and parsing context (path, format, line, column) where possible.
- Keep public API errors minimal and stable; internal enums can be more granular.

## Current Schema Module Boundaries
These locations define error responsibility boundaries and should map to
distinct error categories:

### Ingestion / File Parsing
- `lithos-core/src/schema/ingestor.rs`
- `lithos-core/src/schema/views/raw.rs`
- `lithos-core/src/fs/error.rs` (ParseError integration)

### Raw Syntax Validation
- `lithos-core/src/schema/raw/mod.rs`
- `lithos-core/src/schema/raw/property.rs`
- `lithos-core/src/schema/raw/property_spec.rs`

### Domain Validation and Specs
- `lithos-core/src/schema/aggregate.rs`
- `lithos-core/src/schema/property.rs`
- `lithos-core/src/schema/property_spec/*.rs`

### Reference Expansion and Resolution
- `lithos-core/src/schema/expander.rs`
- `lithos-core/src/schema/extender.rs`
- `lithos-core/src/schema/merger.rs`
- `lithos-core/src/schema/resolver.rs`

### Storage / Repository
- `lithos-core/src/schema/storage.rs`

### Loader Orchestration
- `lithos-core/src/schema/loader.rs`

## Proposed Error Taxonomy
The following errors are the errors the schema module should have, even if
they do not currently exist.

### 1) Ingestion and Parsing Errors
Used for file I/O, filesystem enumeration, and structured parsing.

- `SchemaFileError`
  - Invalid filename or basename
  - Unsupported extension
  - Missing expected file
  - Filesystem read/list/metadata failures

- `SchemaParseError`
  - JSON parse error
  - TOML parse error
  - YAML parse error
  - Cached view deserialization error (schema view or property bank view)
  - Invalid parse output shape

- `SchemaVersionError`
  - Unsupported schema version (path, expected, found)

### 2) Raw Syntax Validation Errors
Used in the Raw layer for syntactic checks only.

- `SchemaNameError`
  - Empty
  - Too long
  - Invalid characters/pattern

- `PropertyNameError`
  - Empty
  - Too long
  - Invalid characters/pattern
  - Duplicate in raw input

- `RawOptionsError`
  - Invalid list/map/rich option shape
  - Invalid key parsing (map keys to u32)
  - Empty options where not allowed

- `SchemaSyntaxError` (umbrella for raw validation)
  - Wraps the above for convenience in callers

### 3) Domain Validation Errors
Used in domain-level validation and property spec construction.

- `PropertySpecError`
  - Invalid enum value
  - Invalid regex
  - Invalid date format
  - Invalid directory path
  - Invalid file class
  - Invalid type
  - Number out of range
  - Invalid step value
  - Non-finite numeric value

- `PropertyValueError` (optional, if separating runtime validation)
  - Type mismatch for actual value
  - Value not in enum
  - Value fails pattern

- `SchemaValidationError`
  - Domain-level validation failure with context

### 4) Reference Expansion and Resolution Errors
Used during expansion, inheritance, and resolution.

- `PropertyRefError`
  - Invalid `$ref` format
  - Ref not found in property bank
  - Type mismatch on override

- `PropertyBankError`
  - Duplicate property name
  - Duplicate property id
  - Property not found (if needed outside raw layer)
  - Missing property bank when required

- `SchemaInheritanceError`
  - Parent not found
  - Circular inheritance
  - Depth exceeded

- `SchemaResolutionError`
  - Wraps expansion, inheritance, and merge errors

### 5) Storage and Repository Errors
Used for persistent storage and repository operations.

- `SchemaStorageError`
  - Underlying database error
  - Corruption / invalid bytes
  - Not found by id or name
  - Conflict on save/delete

- `SchemaRepositoryError` (public facade)
  - Wraps storage errors
  - Wraps domain errors when repository returns them

### 6) Loader Orchestration Errors
Used at the pipeline boundary.

- `SchemaLoaderError` (public facade)
  - Ingestion error
  - Resolution error
  - Repository error

## Public API Strategy
Keep a stable public facade while enabling internal precision:

- Public: `SchemaLoaderError`, `SchemaRepositoryError`, `SchemaError`.
- Internal: `SchemaNameError`, `PropertySpecError`, `SchemaInheritanceError`,
  `SchemaParseError`, etc.
- If multiple internal enums are used, aggregate into `SchemaError` with
  transparent conversions.

## Mapping Current Errors to Target Errors
This is the required mapping activity during implementation:

- `SchemaIngestionError::Io` and `FileSystem` should map to `SchemaFileError`.
- `SchemaIngestionError::{Json,Toml,Yaml}` should map to `SchemaParseError`.
- `SchemaIngestionError::UnsupportedFormat` should map to `SchemaFileError` or
  `SchemaParseError` depending on responsibility.
- `SchemaIngestionError::UnsupportedVersion` should map to `SchemaVersionError`.
- `SchemaError::ValidationFailed` should be replaced by
  `SchemaValidationError` or a more specific domain error.
- `SchemaError::PropertyRefNotFound` should be replaced by `PropertyRefError`.
- `SchemaError::CircularInheritance` and `ParentNotFound` should be replaced by
  `SchemaInheritanceError`.
- `SchemaError::Storage` should be replaced by `SchemaStorageError` and only
  exposed via `SchemaRepositoryError`.

## Implementation Plan (Detailed)

### Step 1: Error Inventory and Call-Site Map
Build a table of all fallible paths and current errors:
- Ingestor: file read, parse, staleness checks, view loads, view saves
- Raw validation: schema name, parent name, exclude names, property names
- Specs: date, file, number, string, bool
- Expansion: property bank lookup and ref format checks
- Inheritance: tree build, parent lookup, cycle detection, depth checks
- Merge: bank property lookup, resolve operations
- Storage: db access, multi-table reads, staleness metadata
- Loader: orchestration-only error boundaries

Deliverable: a mapping table for each call site -> target error enum/variant.

### Step 2: Define Error Enums and Data Shape
Create new enums in `schema/error.rs` or a `schema/error/` submodule:
- Use `Box<str>` for owned string fields.
- Use structured fields instead of free-form strings.
- Expose the facades publicly, keep internal enums private if needed.
- Add documentation for each error and when it is expected to occur.

### Step 3: Conversion Strategy
Define explicit conversions using `thiserror`:
- `SchemaFileError` -> `SchemaIngestionError`
- `SchemaParseError` -> `SchemaIngestionError`
- `SchemaVersionError` -> `SchemaIngestionError`
- Raw validation errors -> `SchemaError`
- Resolution errors -> `SchemaError` or `SchemaResolutionError`
- Storage errors -> `SchemaRepositoryError`
- `SchemaError` -> `SchemaRepositoryError` (domain error during storage ops)
- `SchemaIngestionError` / `SchemaResolutionError` / `SchemaRepositoryError`
  -> `SchemaLoaderError`

### Step 4: Update Call Sites
Refactor error creation and propagation to use new types:
- Replace `SchemaError::ValidationFailed` with specific error types.
- Replace `SchemaIngestionError::Io` use in non-I/O contexts (e.g.,
  `persist_inheritance_metadata` parent missing) with semantic errors.
- Replace ad-hoc error strings in expander/extender/merger/resolver with
  `PropertyRefError` or `SchemaInheritanceError` variants.
- Ensure parsing errors from `fs::ParseError` map to `SchemaParseError`.
- Update view deserialization errors (`RawSchemaView::to_raw`) to a parse
  or cache-specific error, not I/O.

### Step 5: Tests and Documentation Updates
Add or update tests for:
- New error enums and display formatting
- Conversion paths (ParseError -> SchemaParseError, etc.)
- Key failure modes in loader and ingestor

Update documentation:
- Add a section to `schema/error.rs` describing the error hierarchy.
- Update module docs in `schema/loader.rs` and `schema/ingestor.rs` to point
  to the new errors.

### Step 6: Run Quality Gates
- `mise run fmt`
- `mise run test:unit:schema`
- `mise run lint`

## Implementation Sequencing
Recommended order to minimize churn:
1. Add new error enums and conversions (no call site changes).
2. Update ingestion and parsing call sites.
3. Update raw syntax validation call sites.
4. Update property spec validation call sites.
5. Update resolution and inheritance call sites.
6. Update repository and loader call sites.
7. Fix tests and finalize docs.

## Risk and Mitigation
- Risk: Breaking external callers if public error types change.
  - Mitigation: Keep `SchemaError`, `SchemaRepositoryError`, and
    `SchemaLoaderError` as public facades.

- Risk: Error explosion in downstream `match` statements.
  - Mitigation: Use a thin public facade; keep fine-grained enums internal.

- Risk: Loss of context due to conversion.
  - Mitigation: Preserve structured fields and use `#[source]` where needed.

## Completion Criteria
- All schema pipeline phases map to an appropriate error type.
- No remaining usage of generic `SchemaError::ValidationFailed` for distinct
  failure modes.
- All new error types documented with clear expectations.
- Tests updated to match new error variants and conversion paths.
- Quality gates pass (`fmt`, `lint`, `test:unit:schema`).
