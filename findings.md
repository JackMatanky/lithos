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
