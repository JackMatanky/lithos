//! Error types for the Schema domain and its ingestion pipeline.
//!
//! This module provides a hierarchical, phase-oriented error taxonomy that
//! maps to the Schema lifecycle:
//!
//! 1. **File Access**: Physical file access, filename validation, and format
//!    filtering ([`SchemaReadError`]).
//! 2. **Parsing**: Structured decoding of JSON/TOML/YAML and cached views
//!    ([`SchemaParseError`]).
//! 3. **Versioning**: Schema version compatibility checks
//!    ([`SchemaVersionError`]).
//! 4. **Syntax**: Name and shape validation for raw schema inputs.
//! 5. **Validation/Resolution**: Domain-level validation, property resolution,
//!    and inheritance checks ([`SchemaError`] umbrella).
//! 6. **Persistence**: Storage integrity and lookup behavior
//!    ([`SchemaRepositoryError`]).
//! 7. **Orchestration**: Top-level coordination of the full pipeline
//!    ([`SchemaLoaderError`]).
//!
//! # Hierarchy
//!
//! ```text
//! SchemaLoaderError (Orchestration)
//!  ├── SchemaIngestionError (Ingestion Phase)
//!  │    ├── SchemaReadError (I/O & Naming)
//!  │    ├── SchemaParseError (Syntax & Extraction)
//!  │    ├── SchemaVersionError (Version Gate)
//!  │    └── SchemaError (Domain Validation)
//!  ├── SchemaError (Domain Umbrella)
//!  │    ├── PropertySpecError, PropertyValueError
//!  │    ├── PropertyRefError, PropertyMapError
//!  │    ├── PropertyBuilderError
//!  │    └── SchemaInheritanceError, SchemaResolutionError
//!  └── SchemaRepositoryError (Persistence Phase)
//!       └── DbError
//! ```
//!
//! # Design Principles
//!
//! - **Context Preservation**: Each layer wraps the previous one using
//!   `#[error(transparent)]` to preserve the `source()` chain.
//! - **Ergonomics**: Dynamic error data uses `String` to prioritize ergonomics
//!   and compatibility with standard error patterns.
//! - **Phase Orientation**: Errors are categorized by pipeline stage, avoiding
//!   the “everything is ingestion” anti-pattern.
//!
//! # Usage Guidelines
//!
//! | Error Type                | When to Use                                                         |
//! | :------------------------ | :------------------------------------------------------------------ |
//! | [`SchemaError`]           | Domain validation, property resolution, and inheritance handling.   |
//! | [`SchemaIngestionError`]  | Readers/parsers bridging raw bytes to structured schema data.       |
//! | [`SchemaRepositoryError`] | Storage adapters and persistence integrity checks.                  |
//! | [`SchemaLoaderError`]     | Cross-cutting orchestration across the entire lifecycle.            |
//!
//! # Examples
//!
//! ## Handling a Load Failure
//!
//! ```ignore
//! match loader.load_all() {
//!     Err(SchemaLoaderError::Ingestion(e)) => handle_ingestion_error(e),
//!     Err(SchemaLoaderError::Repository(e)) => handle_storage_error(e),
//!     Err(SchemaLoaderError::Resolution(e)) => handle_schema_error(e),
//!     Ok(count) => println!("Loaded {count} schemas"),
//! }
//! ```

use std::path::PathBuf;

use crate::db::DbError;

/// Domain-level errors for in-memory schema validation and resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Returned when the schema name is invalid.
    #[error(transparent)]
    SchemaName(#[from] SchemaNameError),

    /// Returned when a property name is invalid.
    #[error(transparent)]
    PropertyName(#[from] PropertyNameError),

    /// Returned when property specification configuration is invalid.
    #[error(transparent)]
    PropertySpec(#[from] PropertySpecError),

    /// Returned when a value fails validation against a spec.
    #[error(transparent)]
    PropertyValue(#[from] PropertyValueError),

    /// Returned when a property reference is invalid or unresolved.
    #[error(transparent)]
    PropertyRef(#[from] PropertyRefError),

    /// Returned when property map constraints are violated.
    #[error(transparent)]
    PropertyMap(#[from] PropertyMapError),

    /// Returned when property building or overriding fails.
    #[error(transparent)]
    PropertyBuilder(#[from] PropertyBuilderError),

    /// Returned when inheritance resolution fails.
    #[error(transparent)]
    Inheritance(#[from] SchemaInheritanceError),

    /// Returned when schema resolution fails outside inheritance concerns.
    #[error(transparent)]
    Resolution(#[from] SchemaResolutionError),
}

/// Errors related to schema file reading and root-boundary safety.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaReadError {
    /// Failure at the filesystem layer.
    #[error(transparent)]
    Read(#[from] crate::fs::error::ReadError),

    /// Returned when a filename or basename is invalid.
    #[error("invalid filename: {path} ({reason})")]
    InvalidFileName {
        /// Path to the file with an invalid name.
        path: PathBuf,
        /// Reason the filename was rejected.
        reason: String,
    },

    /// Returned for filesystem errors not tied to a specific file.
    #[error("filesystem error: {reason}")]
    FileSystem {
        /// Error details from the filesystem layer.
        reason: String,
    },
}

/// Errors related to structured schema parsing (JSON, TOML, YAML).
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaParseError {
    /// Failure at the parsing layer.
    #[error(transparent)]
    Parse(#[from] crate::fs::error::ParseError),

    /// Returned when cached view deserialization fails.
    #[error("cached view parse error in {path}: {reason}")]
    CachedView {
        /// Path to the cached view file.
        path: PathBuf,
        /// Deserialization error details.
        reason: String,
    },

    /// Returned when serialization of a cached view fails.
    #[error("serialization error in {path}: {reason}")]
    Serialization {
        /// Path to the file being serialized.
        path: PathBuf,
        /// Serialization error details.
        reason: String,
    },
}

/// Schema ingestion error with file context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaIngestionError {
    /// Returned when filesystem access fails.
    #[error(transparent)]
    Read(#[from] SchemaReadError),

    /// Returned when structured parsing fails.
    #[error(transparent)]
    Parse(#[from] SchemaParseError),

    /// Returned when schema version validation fails.
    #[error(transparent)]
    Version(#[from] SchemaVersionError),

    /// Returned when repository access fails during ingestion.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Returned when schema validation fails with file context.
    #[error("schema validation failed in {path}: {source}")]
    Schema {
        /// Path to the schema file that failed validation.
        path: PathBuf,
        /// Underlying schema error.
        #[source]
        source: SchemaError,
    },
}

/// Errors returned by schema repository implementations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaRepositoryError {
    /// Returned when the underlying storage layer fails.
    #[error(transparent)]
    Storage(#[from] DbError),

    /// Returned when domain validation fails while saving or loading.
    #[error(transparent)]
    Domain(#[from] SchemaError),

    /// Returned when an expected entity is missing by ID.
    #[error("schema not found: {0}")]
    NotFoundById(crate::schema::identifier::SchemaId),

    /// Returned when an expected entity is missing by name.
    #[error("schema name not found: {0}")]
    NotFoundByName(crate::schema::identifier::SchemaName),

    /// Returned when an expected entity is missing by path.
    #[error("schema path not found: {0}")]
    NotFoundByPath(crate::fs::PathKey),

    /// Returned when the property bank has not been initialized.
    #[error(
        "PropertyBank not found in database - initialize by loading schema \
         files or creating properties"
    )]
    PropertyBankNotFound,

    /// Returned when the version history is missing or empty for a view.
    #[error(
        "version history missing for {0} - cached view is corrupt or empty"
    )]
    EmptyVersionHistory(crate::fs::PathKey),
}

/// High-level errors returned by schema loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaLoaderError {
    /// Returned when ingestion (file I/O or parsing) fails.
    #[error(transparent)]
    Ingestion(#[from] SchemaIngestionError),

    /// Returned when repository access fails.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Returned when schema resolution fails.
    #[error(transparent)]
    Resolution(#[from] SchemaError),
}

impl From<SchemaInheritanceError> for SchemaLoaderError {
    #[inline]
    fn from(value: SchemaInheritanceError) -> Self {
        Self::Resolution(SchemaError::Inheritance(value))
    }
}

/// Errors raised by property builder and override operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyBuilderError {
    /// Returned when a `$ref` override changes the property type.
    #[error(
        "cannot change property type via override: expected {expected}, got \
         {actual}"
    )]
    OverridePropertyRefSpecTypeMismatch {
        /// Expected type.
        expected: String,
        /// Actual override type.
        actual: String,
    },
}

/// Schema version validation errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaVersionError {
    /// Returned when the schema version is unsupported.
    #[error(
        "unsupported schema version in {path}: got '{found}', expected \
         '{expected}'"
    )]
    UnsupportedVersion {
        /// Path to the file with an unsupported version.
        path: PathBuf,
        /// Version found in the file.
        found: String,
        /// Expected version value.
        expected: String,
    },
}

/// Schema name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaNameError {
    /// Returned when the schema name is empty.
    #[error("schema name cannot be empty")]
    NameIsEmpty,

    /// Returned when the schema name exceeds the maximum length.
    #[error("schema name too long: {len} (max {max})")]
    NameExceedsMaxLength {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Returned when the schema name does not match the expected format.
    #[error("invalid schema name: {name}")]
    ContainsInvalidCharacters {
        /// The invalid name.
        name: String,
    },

    /// Returned when the schema name regex fails to compile.
    #[error("invalid schema name regex: {reason}")]
    RegexCompilationFailed {
        /// Regex error details.
        reason: String,
    },
}

/// Property name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameError {
    /// Returned when the property name is empty.
    #[error("property name cannot be empty")]
    NameIsEmpty,

    /// Returned when the property name exceeds the maximum length.
    #[error("property name too long: {len} (max {max})")]
    NameExceedsMaxLength {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Returned when the property name does not match the expected format.
    #[error("invalid property name: {name}")]
    ContainsInvalidCharacters {
        /// The invalid name.
        name: String,
    },

    /// Returned when the property name regex fails to compile.
    #[error("invalid property name regex: {reason}")]
    RegexCompilationFailed {
        /// Regex error details.
        reason: String,
    },
}

/// Errors raised by property map constraints.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyMapError {
    /// Returned when a property name is defined more than once.
    #[error("duplicate property name: {name}")]
    DuplicatePropertyName {
        /// Property name.
        name: String,
    },

    /// Returned when a property ID is reused for a different name.
    #[error("duplicate property id: {id}")]
    DuplicatePropertyId {
        /// Property id.
        id: String,
    },
}

/// Property spec configuration failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertySpecError {
    /// Returned when a string spec is invalid.
    #[error(transparent)]
    String(#[from] StringSpecError),

    /// Returned when a numeric spec is invalid.
    #[error(transparent)]
    Number(#[from] NumberSpecError),

    /// Returned when a date spec is invalid.
    #[error(transparent)]
    Date(#[from] DateSpecError),

    /// Returned when a file spec is invalid.
    #[error(transparent)]
    File(#[from] FileSpecError),

    /// Returned when an archived spec fails to deserialize.
    #[error("failed to deserialize {spec}: {reason}")]
    ArchivedSpecDeserializationFailed {
        /// Spec name.
        spec: &'static str,
        /// Deserialization error details.
        reason: String,
    },
}

/// Failures that occur when defining, building, or overriding a string property
/// specification.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StringSpecError {
    /// Returned when a regex pattern fails to compile.
    #[error("invalid regex pattern: {pattern} ({reason})")]
    InvalidCustomRegexPattern {
        /// Regex pattern string.
        pattern: String,
        /// Regex error details.
        reason: String,
    },

    /// Returned when an options list is empty.
    #[error("options list cannot be empty")]
    EmptyOptionsList,

    /// Returned when an option value does not match a pattern constraint.
    #[error("option value '{value}' does not match pattern {pattern}")]
    OptionValueViolatesPattern {
        /// Option value.
        value: String,
        /// Pattern string.
        pattern: String,
    },

    /// Returned when an option value is empty or whitespace.
    #[error("option value cannot be empty")]
    EmptyOptionValue,

    /// Returned when an option order key is not a valid integer.
    #[error("option order key must be an integer: {key}")]
    OrderKeyNotAnInteger {
        /// The invalid key.
        key: String,
    },

    /// Returned when an option order key is less than 1.
    #[error("option order key must be >= 1: {order}")]
    OrderKeyLessThanOne {
        /// The invalid order value.
        order: u32,
    },
}

/// Failures that occur when defining or overriding a numeric property
/// specification.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NumberSpecError {
    /// Returned when a numeric range is invalid.
    #[error("invalid range: min {min} cannot be greater than max {max}")]
    MinGreaterThanMax {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },

    /// Returned when a numeric constraint is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFiniteConstraintValue {
        /// Provided value.
        value: f64,
        /// Context label.
        context: String,
    },
}

/// Failures that occur when defining or overriding a date property
/// specification.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DateSpecError {
    /// Returned when a date spec omits the required format.
    #[error("date format is required")]
    MissingFormatString,

    /// Returned when a date format is invalid.
    #[error("invalid date format: {format}")]
    InvalidStrftimePattern {
        /// The invalid format string.
        format: String,
    },
}

/// Failures that occur when defining or overriding a file property
/// specification.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileSpecError {
    /// Returned when a directory path constraint is invalid.
    #[error("invalid directory path: {path}")]
    MalformedDirectoryConstraint {
        /// The invalid directory path.
        path: String,
    },

    /// Returned when a file class constraint is invalid.
    #[error("invalid file class: {class}")]
    EmptyFileClassConstraint {
        /// The invalid file class.
        class: String,
    },
}

/// Property value validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyValueError {
    /// Returned when a value has the wrong type.
    #[error("invalid type: {value} (expected: {expected})")]
    IncorrectPrimitiveType {
        /// Provided value representation.
        value: String,
        /// Expected type representation.
        expected: String,
    },

    /// Returned when a string value fails validation.
    #[error(transparent)]
    String(#[from] StringValueValidationError),

    /// Returned when a numeric value fails validation.
    #[error(transparent)]
    Number(#[from] NumberValueValidationError),

    /// Returned when a date value fails validation.
    #[error(transparent)]
    Date(#[from] DateValueValidationError),

    /// Returned when a file value fails validation.
    #[error(transparent)]
    File(#[from] FileValueValidationError),
}

/// Failures that occur when validating a string value.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StringValueValidationError {
    /// Returned when a value is not in the allowed options list.
    #[error("invalid enum value: {value} (allowed: {allowed:?})")]
    ValueNotInAllowedOptions {
        /// Provided value.
        value: String,
        /// Allowed values.
        allowed: Vec<String>,
    },

    /// Returned when a value does not match a pattern constraint.
    #[error("value {value} does not match pattern {pattern}")]
    ValueViolatesPattern {
        /// Provided value.
        value: String,
        /// Pattern string.
        pattern: String,
    },
}

/// Failures that occur when validating a numeric value.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NumberValueValidationError {
    /// Returned when a numeric value is out of range.
    #[error("number out of range: {value} (min: {min:?}, max: {max:?})")]
    ValueOutsideAllowedRange {
        /// Provided value.
        value: f64,
        /// Minimum allowed value.
        min: Option<f64>,
        /// Maximum allowed value.
        max: Option<f64>,
    },

    /// Returned when a numeric value does not align with the step constraint.
    #[error("invalid step value: {value} (step: {step})")]
    ValueViolatesStepIncrement {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// Returned when a numeric value is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFiniteNumber {
        /// Provided value.
        value: f64,
        /// Context label.
        context: String,
    },
}

/// Failures that occur when validating a date value.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DateValueValidationError {
    /// Returned when a value does not match a date format.
    #[error("value {value} does not match format {format}")]
    ValueDoesNotMatchFormat {
        /// Provided value.
        value: String,
        /// Expected format.
        format: String,
    },
}

/// Failures that occur when validating a file value.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileValueValidationError {
    /// Returned when a file is outside the allowed directory.
    #[error("file {path} must be inside (not at) directory {directory}")]
    FileOutsideAllowedDirectory {
        /// Path to the file.
        path: String,
        /// Allowed directory.
        directory: String,
    },
}

/// Errors for `$ref` property references.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyRefError {
    /// Returned when a `$ref` string is not in the expected format.
    #[error(
        "invalid property reference target: '{reference}' (expected format: \
         #property_bank/<name>)"
    )]
    MalformedBankReferencePath {
        /// The invalid reference string.
        reference: String,
    },

    /// Returned when the referenced property does not exist in the bank.
    #[error("property reference not found: {reference}")]
    TargetPropertyNotFoundInBank {
        /// The reference string.
        reference: String,
    },
}

/// Errors related to schema inheritance graph structure.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaInheritanceError {
    /// Returned when a schema node is missing from the graph.
    #[error("missing schema node for id {id}")]
    MissingNode {
        /// Schema identifier.
        id: crate::schema::identifier::SchemaId,
    },

    /// Returned when a cycle is detected in the inheritance graph.
    #[error("cycle detected in schema inheritance graph: {nodes:?}")]
    CycleDetected {
        /// Nodes involved in the cycle.
        nodes: Vec<crate::schema::identifier::SchemaId>,
    },

    /// Returned when the inheritance graph is not directed.
    #[error("inheritance graph is not directed")]
    NotDirected,

    /// Returned when inheritance depth exceeds the maximum.
    #[error("inheritance depth exceeded: {depth} (max: {max})")]
    DepthExceeded {
        /// Actual depth.
        depth: usize,
        /// Maximum allowed depth.
        max: usize,
    },
}

impl From<crate::graph::GraphError<crate::schema::identifier::SchemaId>>
    for SchemaInheritanceError
{
    #[inline]
    fn from(
        err: crate::graph::GraphError<crate::schema::identifier::SchemaId>,
    ) -> Self {
        match err {
            crate::graph::GraphError::CycleDetected {
                nodes,
            } => Self::CycleDetected {
                nodes,
            },
            crate::graph::GraphError::NotDirected => Self::NotDirected,
            crate::graph::GraphError::MissingNode {
                id,
            } => Self::MissingNode {
                id,
            },
        }
    }
}

/// Errors related to final entity resolution and conflict detection.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaResolutionError {
    /// Returned when duplicate schema names are detected.
    #[error("duplicate schema name: {name}")]
    DuplicateSchemaName {
        /// Schema name.
        name: String,
    },

    /// Returned when a parent schema is not found.
    #[error("parent schema '{parent}' not found for schema '{child}'")]
    ParentNotFound {
        /// Child schema name.
        child: crate::schema::identifier::SchemaName,
        /// Parent schema name.
        parent: crate::schema::identifier::SchemaName,
    },

    /// Returned when a cycle is detected in the inheritance graph.
    #[error("cycle detected in schema inheritance: {}", schemas.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "))]
    CycleDetected {
        /// Schemas involved in the cycle.
        schemas: Vec<crate::schema::identifier::SchemaName>,
    },
}

impl From<crate::fs::error::ParseError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ParseError) -> Self {
        Self::Parse(SchemaParseError::Parse(err))
    }
}

impl From<crate::fs::error::ReadError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ReadError) -> Self {
        Self::Read(SchemaReadError::Read(err))
    }
}

impl From<crate::fs::error::FsError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::FsError) -> Self {
        use crate::fs::error::FsError;
        match err {
            FsError::Read(e) => Self::from(e),
            FsError::Scan(e) => Self::from(e),
            FsError::Parse(e) => Self::from(e),
            FsError::Path(e) => Self::Read(SchemaReadError::FileSystem {
                reason: e.to_string(),
            }),
            FsError::Validation(e) => Self::Read(SchemaReadError::FileSystem {
                reason: e.to_string(),
            }),
            FsError::RootScope(e) => {
                Self::Read(SchemaReadError::Read(e.into()))
            }
        }
    }
}

impl From<crate::fs::error::ScanError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ScanError) -> Self {
        use crate::fs::error::ScanError;
        match err {
            ScanError::Traversal {
                path,
                source,
            } => Self::Read(SchemaReadError::Read(
                crate::fs::error::ReadError::Io {
                    path,
                    source,
                },
            )),
            ScanError::InvalidPattern {
                pattern,
                message,
            } => Self::Read(SchemaReadError::FileSystem {
                reason: format!("Invalid pattern {pattern}: {message}"),
            }),
            ScanError::UnsupportedEntryType(path) => {
                Self::Read(SchemaReadError::FileSystem {
                    reason: format!(
                        "Unsupported entry type at {}",
                        path.display()
                    ),
                })
            }
            ScanError::Path(e) => Self::Read(SchemaReadError::FileSystem {
                reason: e.to_string(),
            }),
        }
    }
}

impl From<crate::fs::error::PathError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::PathError) -> Self {
        Self::Read(SchemaReadError::FileSystem {
            reason: err.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod thread_safety {
        use super::*;

        #[test]
        fn errors_are_send_and_sync() {
            fn assert_send_sync<T: Send + Sync>() {}

            assert_send_sync::<SchemaError>();
            assert_send_sync::<SchemaIngestionError>();
            assert_send_sync::<SchemaRepositoryError>();
            assert_send_sync::<SchemaLoaderError>();
            assert_send_sync::<SchemaReadError>();
            assert_send_sync::<SchemaParseError>();
            assert_send_sync::<SchemaVersionError>();
            assert_send_sync::<SchemaNameError>();
            assert_send_sync::<PropertyNameError>();
            assert_send_sync::<PropertySpecError>();
            assert_send_sync::<PropertyValueError>();
            assert_send_sync::<StringSpecError>();
            assert_send_sync::<StringValueValidationError>();
            assert_send_sync::<NumberSpecError>();
            assert_send_sync::<NumberValueValidationError>();
            assert_send_sync::<DateSpecError>();
            assert_send_sync::<DateValueValidationError>();
            assert_send_sync::<FileSpecError>();
            assert_send_sync::<FileValueValidationError>();
            assert_send_sync::<PropertyRefError>();
            assert_send_sync::<PropertyBuilderError>();
            assert_send_sync::<PropertyMapError>();
            assert_send_sync::<SchemaInheritanceError>();
            assert_send_sync::<SchemaResolutionError>();
        }
    }

    mod formatting {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::schema_name_empty(
            SchemaError::SchemaName(SchemaNameError::NameIsEmpty),
            "schema name cannot be empty"
        )]
        #[case::property_name_empty(
            SchemaError::PropertyName(PropertyNameError::NameIsEmpty),
            "property name cannot be empty"
        )]
        #[case::property_ref_invalid(
            SchemaError::PropertyRef(PropertyRefError::MalformedBankReferencePath {
                reference: "#property_bank/title".into(),
            }),
            "invalid property reference target"
        )]
        #[case::property_builder_type_mismatch(
            SchemaError::PropertyBuilder(PropertyBuilderError::OverridePropertyRefSpecTypeMismatch {
                expected: "string".into(),
                actual: "number".into(),
            }),
            "cannot change property type via override"
        )]
        #[case::resolution_parent_missing(
            SchemaError::Resolution(SchemaResolutionError::ParentNotFound {
                child: crate::schema::identifier::SchemaName::try_new("child").unwrap(),
                parent: crate::schema::identifier::SchemaName::try_new("parent").unwrap(),
            }),
            "parent schema 'parent' not found"
        )]
        fn schema_error_display_contains_message(
            #[case] error: SchemaError,
            #[case] expected_fragment: &str,
        ) {
            let rendered = error.to_string();
            assert!(
                rendered.contains(expected_fragment),
                "Expected display to contain '{expected_fragment}', got: \
                 {rendered}"
            );
        }

        #[rstest]
        #[case::not_found_by_id(
            SchemaRepositoryError::NotFoundById(
                crate::schema::identifier::SchemaId::new()
            ),
            "schema not found:"
        )]
        #[case::not_found_by_name(
            SchemaRepositoryError::NotFoundByName(crate::schema::identifier::SchemaName::try_new("test").unwrap()),
            "schema name not found: test"
        )]
        #[case::not_found_by_path(
            SchemaRepositoryError::NotFoundByPath(crate::fs::PathKey::try_new("test.json").unwrap()),
            "schema path not found: test.json"
        )]
        #[case::empty_version_history(
            SchemaRepositoryError::EmptyVersionHistory(crate::fs::PathKey::try_new("test.json").unwrap()),
            "version history missing for test.json"
        )]
        fn repository_error_display_contains_message(
            #[case] error: SchemaRepositoryError,
            #[case] expected_fragment: &str,
        ) {
            let rendered = error.to_string();
            assert!(
                rendered.contains(expected_fragment),
                "Expected display to contain '{expected_fragment}', got: \
                 {rendered}"
            );
        }
    }

    mod sources {
        use std::error::Error as StdError;

        use super::*;

        #[test]
        fn schema_read_io_exposes_source() {
            let error =
                SchemaReadError::Read(crate::fs::error::ReadError::Io {
                    path: PathBuf::from("schemas/bad.json"),
                    source: std::io::Error::other("disk failed"),
                });

            let source = StdError::source(&error);
            assert!(
                source.is_some(),
                "Expected source for SchemaReadError::Read"
            );

            if let Some(source) = source {
                assert!(
                    source.to_string().contains("disk failed"),
                    "Expected source message to include 'disk failed', got: \
                     {source}"
                );
            }
        }

        #[test]
        fn ingestion_schema_exposes_source() {
            let error = SchemaIngestionError::Schema {
                path: PathBuf::from("schemas/bad.json"),
                source: SchemaError::PropertyRef(
                    PropertyRefError::TargetPropertyNotFoundInBank {
                        reference: "#property_bank/title".into(),
                    },
                ),
            };

            let source = StdError::source(&error);
            assert!(
                source.is_some(),
                "Expected source for SchemaIngestionError::Schema"
            );

            if let Some(source) = source {
                assert!(
                    source.to_string().contains("property reference not found"),
                    "Expected source message to mention missing reference, \
                     got: {source}"
                );
            }
        }

        #[test]
        fn schema_read_filesystem_has_no_source() {
            let error = SchemaReadError::FileSystem {
                reason: "vault unavailable".into(),
            };

            assert!(
                StdError::source(&error).is_none(),
                "Expected no source for SchemaReadError::FileSystem"
            );
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn schema_error_equality_compares_variants() {
            let left = SchemaError::PropertyRef(
                PropertyRefError::TargetPropertyNotFoundInBank {
                    reference: "#property_bank/title".into(),
                },
            );
            let right = SchemaError::PropertyRef(
                PropertyRefError::TargetPropertyNotFoundInBank {
                    reference: "#property_bank/title".into(),
                },
            );

            assert_eq!(
                left, right,
                "Expected identical property reference errors to be equal"
            );
        }

        #[test]
        fn schema_error_equality_distinguishes_variants() {
            let left = SchemaError::PropertyRef(
                PropertyRefError::TargetPropertyNotFoundInBank {
                    reference: "#property_bank/title".into(),
                },
            );
            let right = SchemaError::PropertyRef(
                PropertyRefError::TargetPropertyNotFoundInBank {
                    reference: "#property_bank/status".into(),
                },
            );

            assert!(
                left != right,
                "Expected different references to be unequal: {left:?} vs \
                 {right:?}"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn schema_name_error_converts_into_schema_error() {
            let error: SchemaError = SchemaNameError::NameIsEmpty.into();

            assert!(
                matches!(
                    error,
                    SchemaError::SchemaName(SchemaNameError::NameIsEmpty)
                ),
                "Expected SchemaNameError::NameIsEmpty to convert into \
                 SchemaError::SchemaName"
            );
        }

        #[test]
        fn property_name_error_converts_into_schema_error() {
            let error: SchemaError = PropertyNameError::NameIsEmpty.into();

            assert!(
                matches!(
                    error,
                    SchemaError::PropertyName(PropertyNameError::NameIsEmpty)
                ),
                "Expected PropertyNameError::NameIsEmpty to convert into \
                 SchemaError::PropertyName"
            );
        }

        #[test]
        fn parse_error_maps_to_schema_ingestion_error() {
            let error: SchemaIngestionError =
                crate::fs::error::ParseError::UnsupportedFormat {
                    path: PathBuf::from("schemas/schema.xml"),
                    supported: &["json", "yaml"],
                }
                .into();

            assert!(matches!(
                error,
                SchemaIngestionError::Parse(SchemaParseError::Parse(
                    crate::fs::error::ParseError::UnsupportedFormat {
                        path: _,
                        supported: _,
                    },
                ))
            ));

            if let SchemaIngestionError::Parse(SchemaParseError::Parse(
                crate::fs::error::ParseError::UnsupportedFormat {
                    path,
                    supported,
                },
            )) = error
            {
                assert_eq!(path, PathBuf::from("schemas/schema.xml"));
                assert_eq!(supported, &["json", "yaml"]);
            }
        }

        #[test]
        fn db_error_converts_into_schema_repository_error() {
            let db_error = DbError::Serialization("test".into());
            let repo_error: SchemaRepositoryError = db_error.into();

            assert!(
                matches!(
                    repo_error,
                    SchemaRepositoryError::Storage(DbError::Serialization(_))
                ),
                "Expected DbError::Serialization to convert into \
                 SchemaRepositoryError::Storage(_)"
            );
        }
    }
}
