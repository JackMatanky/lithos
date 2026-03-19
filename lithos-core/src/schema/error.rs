//! Schema error taxonomy for the schema pipeline.
//!
//! This module centralizes error types produced by schema ingestion, parsing,
//! validation, resolution, and storage. Errors are ordered from high-level
//! pipeline wrappers ([`SchemaLoaderError`], [`SchemaIngestionError`],
//! [`SchemaRepositoryError`]) down to domain and leaf errors
//! ([`SchemaError`] and its variants).
//!
//! Use the wrapper errors at API boundaries (loader, ingestor, repository) and
//! the domain errors when validating or resolving in-memory schema data.

use std::path::PathBuf;

use crate::db::DbError;

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

/// Errors produced while reading schema files and building raw/domain models.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaIngestionError {
    /// Returned when filesystem access fails.
    #[error(transparent)]
    File(#[from] SchemaFileError),

    /// Returned when structured parsing fails.
    #[error(transparent)]
    Parse(#[from] SchemaParseError),

    /// Returned when schema version validation fails.
    #[error(transparent)]
    Version(#[from] SchemaVersionError),

    /// Returned when syntax validation fails.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Returned when storage access fails during ingestion.
    #[error(transparent)]
    Storage(#[from] SchemaStorageError),

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
    Storage(#[from] SchemaStorageError),

    /// Returned when domain validation fails while saving or loading.
    #[error(transparent)]
    Domain(#[from] SchemaError),
}

/// Domain-level errors for in-memory schema validation and resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Returned when raw syntax validation fails.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Returned when property specification configuration is invalid.
    #[error(transparent)]
    PropertySpec(#[from] PropertySpecError),

    /// Returned when a value fails validation against a spec.
    #[error(transparent)]
    PropertyValue(#[from] PropertyValueError),

    /// Returned when a property reference is invalid or unresolved.
    #[error(transparent)]
    PropertyRef(#[from] PropertyRefError),

    /// Returned when property bank constraints are violated.
    #[error(transparent)]
    PropertyBank(#[from] PropertyBankError),

    /// Returned when inheritance resolution fails.
    #[error(transparent)]
    Inheritance(#[from] SchemaInheritanceError),

    /// Returned when schema resolution fails outside inheritance concerns.
    #[error(transparent)]
    Resolution(#[from] SchemaResolutionError),
}

/// File-system and filename errors during ingestion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaFileError {
    /// Returned when a schema file cannot be read.
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path to the file that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Returned when a filename or basename is invalid.
    #[error("invalid filename: {path} ({reason})")]
    InvalidFilename {
        /// Path to the file with an invalid name.
        path: PathBuf,
        /// Reason the filename was rejected.
        reason: Box<str>,
    },

    /// Returned when the file format is not supported.
    #[error("unsupported format for {path}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// Path to the file with unsupported format.
        path: PathBuf,
        /// Supported formats.
        supported: Vec<Box<str>>,
    },

    /// Returned for filesystem errors not tied to a specific file.
    #[error("filesystem error: {reason}")]
    FileSystem {
        /// Error details from the filesystem layer.
        reason: Box<str>,
    },
}

/// Structured parse errors with file location context.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaParseError {
    /// Returned when JSON parsing fails.
    #[error(
        "JSON parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Json {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when TOML parsing fails.
    #[error(
        "TOML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Toml {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when YAML parsing fails.
    #[error(
        "YAML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Yaml {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when cached view deserialization fails.
    #[error("cached view parse error in {path}: {reason}")]
    CachedView {
        /// Path to the cached view file.
        path: PathBuf,
        /// Deserialization error details.
        reason: Box<str>,
    },

    /// Returned when serialization of a cached view fails.
    #[error("serialization error in {path}: {reason}")]
    Serialization {
        /// Path to the file being serialized.
        path: PathBuf,
        /// Serialization error details.
        reason: Box<str>,
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
        found: Box<str>,
        /// Expected version value.
        expected: Box<str>,
    },
}

/// Schema name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaNameError {
    /// Returned when the schema name is empty.
    #[error("schema name cannot be empty")]
    Empty,

    /// Returned when the schema name exceeds the maximum length.
    #[error("schema name too long: {len} (max {max})")]
    TooLong {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Returned when the schema name does not match the expected format.
    #[error("invalid schema name: {name}")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
    },

    /// Returned when the schema name regex fails to compile.
    #[error("invalid schema name regex: {reason}")]
    InvalidRegex {
        /// Regex error details.
        reason: Box<str>,
    },
}

/// Identifies where a property name was used for better diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameContext {
    /// Property name defined in a schema file.
    SchemaProperty,
    /// Property name defined in a property bank file.
    PropertyBank,
    /// Property name used in an excludes list.
    Exclude,
}

impl std::fmt::Display for PropertyNameContext {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match *self {
            Self::SchemaProperty => "schema property",
            Self::PropertyBank => "property bank",
            Self::Exclude => "exclude list",
        };
        f.write_str(label)
    }
}

/// Property name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameError {
    /// Returned when the property name is empty.
    #[error("property name cannot be empty ({context})")]
    Empty {
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name exceeds the maximum length.
    #[error("property name too long: {len} (max {max}) ({context})")]
    TooLong {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name does not match the expected format.
    #[error("invalid property name: {name} ({context})")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name regex fails to compile.
    #[error("invalid property name regex: {reason}")]
    InvalidRegex {
        /// Regex error details.
        reason: Box<str>,
    },
}

/// Syntax validation failures in raw schema inputs.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaSyntaxError {
    /// Returned when the schema name is invalid.
    #[error(transparent)]
    SchemaName(#[from] SchemaNameError),

    /// Returned when a property name is invalid.
    #[error(transparent)]
    PropertyName(#[from] PropertyNameError),
}

/// Property spec configuration failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertySpecError {
    /// Returned when a date spec omits the required format.
    #[error("date format is required")]
    DateFormatRequired,

    /// Returned when a date format is invalid.
    #[error("invalid date format: {format}")]
    InvalidDateFormat {
        /// The invalid format string.
        format: Box<str>,
    },

    /// Returned when a regex pattern fails to compile.
    #[error("invalid regex pattern: {pattern} ({reason})")]
    InvalidRegex {
        /// Regex pattern string.
        pattern: Box<str>,
        /// Regex error details.
        reason: Box<str>,
    },

    /// Returned when an options list is empty.
    #[error("options list cannot be empty")]
    OptionsEmpty,

    /// Returned when an option value does not match a pattern constraint.
    #[error("option value '{value}' does not match pattern {pattern}")]
    OptionPatternMismatch {
        /// Option value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Returned when an option value is empty or whitespace.
    #[error("option value cannot be empty")]
    OptionValueEmpty,

    /// Returned when a directory path constraint is invalid.
    #[error("invalid directory path: {path}")]
    InvalidDirectoryPath {
        /// The invalid directory path.
        path: Box<str>,
    },

    /// Returned when a numeric range is invalid.
    #[error("invalid range: min {min} cannot be greater than max {max}")]
    InvalidRange {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },

    /// Returned when a numeric constraint is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },

    /// Returned when a file class constraint is invalid.
    #[error("invalid file class: {class}")]
    InvalidFileClass {
        /// The invalid file class.
        class: Box<str>,
    },

    /// Returned when an archived spec fails to deserialize.
    #[error("failed to deserialize {spec}: {reason}")]
    Deserialization {
        /// Spec name.
        spec: &'static str,
        /// Deserialization error details.
        reason: Box<str>,
    },
}

/// Property value validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyValueError {
    /// Returned when a value has the wrong type.
    #[error("invalid type: {value} (expected: {expected})")]
    InvalidType {
        /// Provided value representation.
        value: Box<str>,
        /// Expected type representation.
        expected: Box<str>,
    },

    /// Returned when a value is not in the allowed options list.
    #[error("invalid enum value: {value} (allowed: {allowed:?})")]
    InvalidEnumValue {
        /// Provided value.
        value: Box<str>,
        /// Allowed values.
        allowed: Vec<Box<str>>,
    },

    /// Returned when a value does not match a pattern constraint.
    #[error("value {value} does not match pattern {pattern}")]
    PatternMismatch {
        /// Provided value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Returned when a value does not match a date format.
    #[error("value {value} does not match format {format}")]
    DateFormatMismatch {
        /// Provided value.
        value: Box<str>,
        /// Expected format.
        format: Box<str>,
    },

    /// Returned when a numeric value is out of range.
    #[error("number out of range: {value} (min: {min:?}, max: {max:?})")]
    NumberOutOfRange {
        /// Provided value.
        value: f64,
        /// Minimum allowed value.
        min: Option<f64>,
        /// Maximum allowed value.
        max: Option<f64>,
    },

    /// Returned when a numeric value does not align with the step constraint.
    #[error("invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// Returned when a numeric value is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },
}

/// Errors for `$ref` property references.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyRefError {
    /// Returned when a `$ref` string is not in the expected format.
    #[error(
        "invalid property reference: '{reference}' (expected format: \
         property_bank#/<name>)"
    )]
    InvalidFormat {
        /// The invalid reference string.
        reference: Box<str>,
    },

    /// Returned when the referenced property does not exist.
    #[error("property reference not found: {reference}")]
    NotFound {
        /// The reference string.
        reference: Box<str>,
    },

    /// Returned when a `$ref` override changes the property type.
    #[error(
        "cannot change property type via $ref override: expected {expected}, \
         got {actual}"
    )]
    TypeMismatch {
        /// Expected type.
        expected: Box<str>,
        /// Actual override type.
        actual: Box<str>,
    },
}

/// Errors raised by property bank operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyBankError {
    /// Returned when a property name is defined more than once.
    #[error("duplicate property name: {name}")]
    DuplicatePropertyName {
        /// Property name.
        name: Box<str>,
    },

    /// Returned when a property ID is reused for a different name.
    #[error("duplicate property id: {id}")]
    DuplicatePropertyId {
        /// Property id.
        id: Box<str>,
    },
}

/// Errors related to schema inheritance resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaInheritanceError {
    /// Returned when a declared parent schema is missing.
    #[error("parent not found: {name}")]
    ParentNotFound {
        /// Parent schema name.
        name: Box<str>,
    },

    /// Returned when inheritance cycles are detected.
    #[error("circular schema inheritance detected: {name}")]
    CircularInheritance {
        /// Schema name involved in the cycle.
        name: Box<str>,
    },

    /// Returned when inheritance depth exceeds the maximum.
    #[error("inheritance depth exceeded: {depth} (max: {max})")]
    DepthExceeded {
        /// Actual depth.
        depth: usize,
        /// Maximum allowed depth.
        max: usize,
    },
}

/// Resolution errors not covered by other categories.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaResolutionError {
    /// Returned when duplicate schema names are detected.
    #[error("duplicate schema name: {name}")]
    DuplicateSchemaName {
        /// Schema name.
        name: Box<str>,
    },

    /// Returned when a schema node is missing during merge.
    #[error("schema node missing for id {id}")]
    MissingNode {
        /// Schema ID.
        id: crate::schema::aggregate::SchemaId,
    },
}

/// Storage-related errors for schema persistence.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaStorageError {
    /// Returned when the database layer fails.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),

    /// Returned when an expected entity is missing.
    #[error("not found: {name}")]
    NotFound {
        /// Name or identifier for the missing entity.
        name: Box<str>,
    },

    /// Returned when storage corruption is detected.
    #[error("data corruption: {reason}")]
    Corruption {
        /// Reason for corruption.
        reason: Box<str>,
    },

    /// Returned when the property bank has not been initialized.
    #[error(
        "PropertyBank not found in database - initialize by loading schema \
         files or creating properties"
    )]
    PropertyBankNotFound,

    /// Returned when storage operations conflict.
    #[error("conflict: {reason}")]
    Conflict {
        /// Reason for the conflict.
        reason: Box<str>,
    },
}

impl From<SchemaNameError> for SchemaError {
    #[inline]
    fn from(err: SchemaNameError) -> Self {
        Self::Syntax(err.into())
    }
}

impl From<PropertyNameError> for SchemaError {
    #[inline]
    fn from(err: PropertyNameError) -> Self {
        Self::Syntax(err.into())
    }
}

impl From<crate::fs::error::ParseError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ParseError) -> Self {
        use crate::fs::error::ParseError;
        match err {
            ParseError::Io {
                path,
                source,
            } => Self::File(SchemaFileError::Io {
                path,
                source,
            }),
            ParseError::Json {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Json {
                path,
                message,
                line,
                column,
            }),
            ParseError::Toml {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Toml {
                path,
                message,
                line,
                column,
            }),
            ParseError::Yaml {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Yaml {
                path,
                message,
                line,
                column,
            }),
            ParseError::UnsupportedFormat {
                path,
                supported,
            } => Self::File(SchemaFileError::UnsupportedFormat {
                path,
                supported: supported.iter().map(|s| (*s).into()).collect(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn schema_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaError>();
    }

    #[test]
    fn schema_repository_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaRepositoryError>();
    }

    #[test]
    fn schema_loader_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaLoaderError>();
    }

    #[test]
    fn schema_ingestion_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaIngestionError>();
    }

    #[rstest]
    #[case(SchemaError::Syntax(SchemaSyntaxError::SchemaName(
        SchemaNameError::Empty
    )))]
    #[case(SchemaError::Syntax(SchemaSyntaxError::PropertyName(
        PropertyNameError::Empty {
            context: PropertyNameContext::SchemaProperty
        }
    )))]
    #[case(SchemaError::PropertyRef(PropertyRefError::InvalidFormat {
        reference: "property_bank#/title".into()
    }))]
    #[case(SchemaError::Inheritance(SchemaInheritanceError::ParentNotFound {
        name: "parent".into()
    }))]
    fn schema_error_display_is_comprehensive(#[case] error: SchemaError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
