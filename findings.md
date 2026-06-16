# Findings: Schema Error Redesign

## User-Specified Redesign Points (Must be finalized in artifacts)
1. **Parse**: Use `Parse(#[from] crate::fs::error::ParseError)`.
2. **Read**: `SchemaReadError` wrapping `Read(#[from] crate::fs::error::ReadError)`.
3. **Remove Syntax/Ingestion**: Remove `SchemaSyntaxError` and `SchemaIngestionError`.

4. **SchemaNameError**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SchemaNameError {
    #[error("schema name cannot be empty")]
    NameIsEmpty,
    #[error("schema name too long: {len} (max {max})")]
    NameExceedsMaxLength { len: usize, max: usize },
    #[error("invalid schema name: {name}")]
    ContainsInvalidCharacters { name: Box<str> },
    #[error("invalid schema name regex: {reason}")]
    RegexCompilationFailed { reason: Box<str> },
}
```
5. **PropertyNameError**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PropertyNameError {
    #[error("property name cannot be empty")]
    NameIsEmpty,
    #[error("property name too long: {len} (max {max})")]
    NameExceedsMaxLength { len: usize, max: usize },
    #[error("invalid property name: {name}")]
    ContainsInvalidCharacters { name: Box<str> },
    #[error("invalid property name regex: {reason}")]
    RegexCompilationFailed { reason: Box<str> },
}
```
6. **Property Ref/Builder Errors**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PropertyRefError {
    #[error("invalid property reference target: '{reference}' (expected format: #property_bank/<name>)")]
    MalformedBankReferencePath { reference: Box<str> },
    #[error("property reference not found: {reference}")]
    TargetPropertyNotFoundInBank { reference: Box<str> },
}
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PropertyBuilderError {
    #[error("cannot change property type via override: expected {expected}, got {actual}")]
    OverridePropertyRefSpecTypeMismatch { expected: Box<str>, actual: Box<str> },
}
```
7. **Property Map**: `PropertyBankError` -> `PropertyMapError`.
8. **Property Specification & Value Errors**:
```rust
// --- Umbrella Spec Errors (schema/property_spec/mod.rs) ---

/// Failures that occur when defining, building, or overriding a property specification.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PropertySpecError {
    #[error(transparent)]
    String(#[from] StringSpecError),
    #[error(transparent)]
    Number(#[from] NumberSpecError),
    #[error(transparent)]
    Date(#[from] DateSpecError),
    #[error(transparent)]
    File(#[from] FileSpecError),

    #[error("failed to deserialize {spec}: {reason}")]
    ArchivedSpecDeserializationFailed { spec: &'static str, reason: Box<str> },
}

/// Failures that occur when validating a runtime value against a specification.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PropertyValueError {
    #[error("invalid type: {value} (expected: {expected})")]
    IncorrectPrimitiveType { value: Box<str>, expected: Box<str> },

    #[error(transparent)]
    String(#[from] StringValueValidationError),
    #[error(transparent)]
    Number(#[from] NumberValueValidationError),
    #[error(transparent)]
    Date(#[from] DateValueValidationError),
    #[error(transparent)]
    File(#[from] FileValueValidationError),
}

// --- String Spec Errors (schema/property_spec/string.rs) ---

#[derive(Debug, Clone, PartialEq, Error)]
pub enum StringSpecError {
    #[error("invalid regex pattern: {pattern} ({reason})")]
    InvalidCustomRegexPattern { pattern: Box<str>, reason: Box<str> },
    #[error("options list cannot be empty")]
    EmptyOptionsList,
    #[error("option value '{value}' does not match pattern {pattern}")]
    OptionValueViolatesPattern { value: Box<str>, pattern: Box<str> },
    #[error("option value cannot be empty")]
    EmptyOptionValue,
    #[error("option order key must be an integer: {key}")]
    OrderKeyNotAnInteger { key: Box<str> },
    #[error("option order key must be >= 1: {order}")]
    OrderKeyLessThanOne { order: u32 },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum StringValueValidationError {
    #[error("invalid enum value: {value} (allowed: {allowed:?})")]
    ValueNotInAllowedOptions { value: Box<str>, allowed: Vec<Box<str>> },
    #[error("value {value} does not match pattern {pattern}")]
    ValueViolatesPattern { value: Box<str>, pattern: Box<str> },
}

// --- Number Spec Errors (schema/property_spec/number.rs) ---

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NumberSpecError {
    #[error("invalid range: min {min} cannot be greater than max {max}")]
    MinGreaterThanMax { min: f64, max: f64 },
    #[error("{context} must be finite: {value}")]
    NonFiniteConstraintValue { value: f64, context: Box<str> },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NumberValueValidationError {
    #[error("number out of range: {value} (min: {min:?}, max: {max:?})")]
    ValueOutsideAllowedRange { value: f64, min: Option<f64>, max: Option<f64> },
    #[error("invalid step value: {value} (step: {step})")]
    ValueViolatesStepIncrement { value: f64, step: f64 },
    #[error("{context} must be finite: {value}")]
    NonFiniteNumber { value: f64, context: Box<str> },
}

// --- Date Spec Errors (schema/property_spec/date.rs) ---

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DateSpecError {
    #[error("date format is required")]
    MissingFormatString,
    #[error("invalid date format: {format}")]
    InvalidStrftimePattern { format: Box<str> },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DateValueValidationError {
    #[error("value {value} does not match format {format}")]
    ValueDoesNotMatchFormat { value: Box<str>, format: Box<str> },
}

// --- File Spec Errors (schema/property_spec/file.rs) ---

#[derive(Debug, Clone, PartialEq, Error)]
pub enum FileSpecError {
    #[error("invalid directory path: {path}")]
    MalformedDirectoryConstraint { path: Box<str> },
    #[error("invalid file class: {class}")]
    EmptyFileClassConstraint { class: Box<str> },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum FileValueValidationError {
    #[error("file {path} must be inside (not at) directory {directory}")]
    FileOutsideAllowedDirectory { path: Box<str>, directory: Box<str> },
}

```
9. **Orchestration**: Rename `SchemaLoaderError` -> `SchemaBuilderError`.
10. **File Validation**: `SchemaReadError` consolidated.
11. **Clean-up**: Remove duplicate variants in `SchemaResolutionError` and `SchemaInheritanceError`.

### Proposed Revised Hierarchy

```text
SchemaError (The Central Umbrella)
 ├── SchemaBuilderError
 │    ├── SchemaReadError (Wraps fs::error::ReadError)
 │    ├── SchemaParseError (Wraps fs::error::ParseError)
 │    ├── SchemaVersionError
 │    ├── PropertyBuilderError
 │    ├── SchemaInheritanceError (Graph structural constraints)
 │    └── SchemaResolutionError (Final semantic consolidation)
 ├── SchemaNameError (Syntax validation)
 ├── PropertyNameError (Syntax validation)
 ├── PropertySpecError (Structural invalidity of Spec definitions)
 ├── PropertyValueError (Data fails to adhere to Spec validation rules)
 ├── PropertyRefError (Invalid $ref references)
 ├── PropertyMapError (Duplicate Property ID/Name constraints)
 └── SchemaRepositoryError (Persistence Phase)
```

### Revised Domain Responsibilities:
- **SchemaInheritanceError**: Solely responsible for graph-related constraints (cycles, missing nodes, depth limits, directed-graph violations).
- **SchemaResolutionError**: Solely responsible for final entity resolution and conflict detection (duplicate names, missing parent-child link resolution failures, merge conflicts).

### Phase 6c.1 Implementation Design: String Transition
We are replacing `Box<str>` with `String` throughout the `schema` module.
- `error.rs`: All variant fields.
- `identifier.rs`, `property.rs`: Newtype wrappers for names.
- `raw/*.rs`: All raw schema input types.
- `processors/*.rs`: All processor state and logic.

### Phase 6c.2 Implementation Design: Circularity & Repository
- **Circularity**: `SchemaRepositoryError` currently wraps `SchemaError`. Since the new `SchemaError` will be the top-level umbrella wrapping `SchemaRepositoryError`, we must remove `Domain(SchemaError)` from `SchemaRepositoryError`.
- **Audit**: Grep shows no explicit usages of `SchemaRepositoryError::Domain` in the `schema` module. It was likely added for future-proofing or is used implicitly via `#[from]`. We will verify this during Task 6c.2.2.

### Phase 6c.3 Implementation Design: Orchestration Layer

#### Mapping Logic
The migration will follow this mapping from the legacy `SchemaLoaderError` / `SchemaIngestionError` to the new `SchemaError` umbrella:

| Legacy Construction | New Construction |
| :--- | :--- |
| `SchemaLoaderError::Ingestion(SchemaIngestionError::Read(e))` | `SchemaError::Builder(SchemaBuilderError::Read(e))` |
| `SchemaLoaderError::Ingestion(SchemaIngestionError::Parse(e))` | `SchemaError::Builder(SchemaBuilderError::Parse(e))` |
| `SchemaLoaderError::Ingestion(SchemaIngestionError::Version(e))` | `SchemaError::Builder(SchemaBuilderError::Version(e))` |
| `SchemaLoaderError::Ingestion(SchemaIngestionError::Schema { path, source })` | `SchemaError::Builder(SchemaBuilderError::Validation { path, source })` |
| `SchemaLoaderError::Repository(e)` | `SchemaError::Repository(e)` |
| `SchemaLoaderError::Resolution(e)` | `SchemaError::Builder(SchemaBuilderError::Resolution(e))` |
| `SchemaIngestionError::Repository(e)` | `SchemaError::Repository(e)` |

#### `SchemaBuilderError` Structure
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaBuilderError {
    #[error(transparent)]
    Read(#[from] SchemaReadError),

    #[error(transparent)]
    Parse(#[from] SchemaParseError),

    #[error(transparent)]
    Version(#[from] SchemaVersionError),

    #[error(transparent)]
    PropertyBuilder(#[from] PropertyBuilderError),

    #[error(transparent)]
    Inheritance(#[from] SchemaInheritanceError),

    #[error(transparent)]
    Resolution(#[from] SchemaResolutionError),

    /// Validation failed for a specific file during build.
    #[error("validation failed for schema at {path}: {source}")]
    Validation {
        path: PathBuf,
        #[source]
        source: Box<SchemaError>, // Boxed to break recursion
    },
}
```

#### `SchemaError` Umbrella Structure
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaError {
    #[error(transparent)]
    Builder(#[from] Box<SchemaBuilderError>), // Boxed for recursion safety

    #[error(transparent)]
    Repository(#[from] Box<SchemaRepositoryError>),

    #[error(transparent)]
    SchemaName(#[from] SchemaNameError),

    #[error(transparent)]
    PropertyName(#[from] PropertyNameError),

    #[error(transparent)]
    PropertySpec(#[from] PropertySpecError),

    #[error(transparent)]
    PropertyValue(#[from] PropertyValueError),

    #[error(transparent)]
    PropertyRef(#[from] PropertyRefError),

    #[error(transparent)]
    PropertyMap(#[from] PropertyMapError),
}
```

## Phase 6c.1 implementation notes
- Updated `inheritance.rs` to propagate `GraphError` correctly via `Into::into()`.
- Updated `schema_processor.rs` to return `SchemaResolutionError::ParentNotFound` when a parent is missing in `analyze_graph` and `build_new_graph`.
- Updated `schema_processor.rs` to return `SchemaResolutionError::DuplicateSchemaName` in `build_resolution_index` and `build_new_graph`.
