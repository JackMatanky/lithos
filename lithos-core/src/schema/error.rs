//! Schema error types.
//!
//! This module defines schema-specific errors using thiserror for structured
//! error handling across ingestion, validation, resolution, and storage.

use std::path::PathBuf;

use crate::db::DbError;

/// Context for property name validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameContext {
    /// Property name defined in a schema file.
    SchemaProperty,
    /// Property name defined in a property bank file.
    PropertyBank,
    /// Property name used in `excludes` list.
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

/// Schema name validation errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaNameError {
    /// Schema name cannot be empty.
    #[error("schema name cannot be empty")]
    Empty,

    /// Schema name too long.
    #[error("schema name too long: {len} (max {max})")]
    TooLong {
        /// Provided name length.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Schema name is not valid for the expected pattern.
    #[error("invalid schema name: {name}")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
    },
}

/// Property name validation errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameError {
    /// Property name cannot be empty.
    #[error("property name cannot be empty ({context})")]
    Empty {
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Property name too long.
    #[error("property name too long: {len} (max {max}) ({context})")]
    TooLong {
        /// Provided name length.
        len: usize,
        /// Maximum allowed length.
        max: usize,
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Property name is not valid for the expected pattern.
    #[error("invalid property name: {name} ({context})")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
        /// Context of the property name.
        context: PropertyNameContext,
    },
}

/// Internal validation errors for schema construction.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaValidationError {
    /// Schema name regex failed to compile.
    #[error("invalid schema name regex: {reason}")]
    SchemaNameRegex {
        /// Regex error details.
        reason: Box<str>,
    },

    /// Property name regex failed to compile.
    #[error("invalid property name regex: {reason}")]
    PropertyNameRegex {
        /// Regex error details.
        reason: Box<str>,
    },
}

/// Raw syntax validation errors for schema inputs.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaSyntaxError {
    /// Schema name validation error.
    #[error(transparent)]
    SchemaName(#[from] SchemaNameError),

    /// Property name validation error.
    #[error(transparent)]
    PropertyName(#[from] PropertyNameError),
}

/// Property spec configuration errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertySpecError {
    /// Date format is required but missing.
    #[error("date format is required")]
    DateFormatRequired,

    /// Invalid date format string.
    #[error("invalid date format: {format}")]
    InvalidDateFormat {
        /// The invalid format string.
        format: Box<str>,
    },

    /// Invalid regex pattern.
    #[error("invalid regex pattern: {pattern} ({reason})")]
    InvalidRegex {
        /// Regex pattern string.
        pattern: Box<str>,
        /// Regex error details.
        reason: Box<str>,
    },

    /// Options list is empty.
    #[error("options list cannot be empty")]
    OptionsEmpty,

    /// Option value does not match pattern.
    #[error("option value '{value}' does not match pattern {pattern}")]
    OptionPatternMismatch {
        /// Option value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Option value is empty or whitespace.
    #[error("option value cannot be empty")]
    OptionValueEmpty,

    /// Invalid directory path constraint.
    #[error("invalid directory path: {path}")]
    InvalidDirectoryPath {
        /// The invalid directory path.
        path: Box<str>,
    },

    /// Invalid numeric range configuration.
    #[error("invalid range: min {min} cannot be greater than max {max}")]
    InvalidRange {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },

    /// Non-finite numeric constraint.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },

    /// Invalid file class constraint.
    #[error("invalid file class: {class}")]
    InvalidFileClass {
        /// The invalid file class.
        class: Box<str>,
    },

    /// Failure deserializing an archived spec.
    #[error("failed to deserialize {spec}: {reason}")]
    Deserialization {
        /// Spec name.
        spec: &'static str,
        /// Deserialization error details.
        reason: Box<str>,
    },
}

/// Property value validation errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyValueError {
    /// Invalid type for property value.
    #[error("invalid type: {value} (expected: {expected})")]
    InvalidType {
        /// Provided value representation.
        value: Box<str>,
        /// Expected type representation.
        expected: Box<str>,
    },

    /// Invalid enum value.
    #[error("invalid enum value: {value} (allowed: {allowed:?})")]
    InvalidEnumValue {
        /// Provided value.
        value: Box<str>,
        /// Allowed values.
        allowed: Vec<Box<str>>,
    },

    /// Pattern mismatch.
    #[error("value {value} does not match pattern {pattern}")]
    PatternMismatch {
        /// Provided value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Date format mismatch.
    #[error("value {value} does not match format {format}")]
    DateFormatMismatch {
        /// Provided value.
        value: Box<str>,
        /// Expected format.
        format: Box<str>,
    },

    /// Number out of range.
    #[error("number out of range: {value} (min: {min:?}, max: {max:?})")]
    NumberOutOfRange {
        /// Provided value.
        value: f64,
        /// Minimum allowed value.
        min: Option<f64>,
        /// Maximum allowed value.
        max: Option<f64>,
    },

    /// Invalid step value.
    #[error("invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// Non-finite numeric value.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },
}

/// Property reference errors for `$ref` handling.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyRefError {
    /// Invalid `$ref` format.
    #[error(
        "invalid property reference: '{reference}' (expected format: \
         property_bank#/<name>)"
    )]
    InvalidFormat {
        /// The invalid reference string.
        reference: Box<str>,
    },

    /// Property reference not found in property bank.
    #[error("property reference not found: {reference}")]
    NotFound {
        /// The reference string.
        reference: Box<str>,
    },

    /// Property type mismatch on `$ref` override.
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

/// Property bank errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyBankError {
    /// Duplicate property name in the property bank.
    #[error("duplicate property name: {name}")]
    DuplicatePropertyName {
        /// Property name.
        name: Box<str>,
    },

    /// Duplicate property ID in the property bank.
    #[error("duplicate property id: {id}")]
    DuplicatePropertyId {
        /// Property id.
        id: Box<str>,
    },
}

/// Schema inheritance errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaInheritanceError {
    /// Parent schema not found.
    #[error("parent not found: {name}")]
    ParentNotFound {
        /// Parent schema name.
        name: Box<str>,
    },

    /// Circular inheritance detected.
    #[error("circular schema inheritance detected: {name}")]
    CircularInheritance {
        /// Schema name in the detected cycle.
        name: Box<str>,
    },

    /// Inheritance chain exceeds maximum depth.
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
    /// Duplicate schema name detected in the batch.
    #[error("duplicate schema name: {name}")]
    DuplicateSchemaName {
        /// Schema name.
        name: Box<str>,
    },

    /// Missing schema node during merge.
    #[error("schema node missing for id {id}")]
    MissingNode {
        /// Schema ID.
        id: crate::schema::aggregate::SchemaId,
    },
}

/// Schema-related domain errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Raw syntax validation error.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Internal validation error.
    #[error(transparent)]
    Validation(#[from] SchemaValidationError),

    /// Property spec configuration error.
    #[error(transparent)]
    PropertySpec(#[from] PropertySpecError),

    /// Property value validation error.
    #[error(transparent)]
    PropertyValue(#[from] PropertyValueError),

    /// Property reference error.
    #[error(transparent)]
    PropertyRef(#[from] PropertyRefError),

    /// Property bank error.
    #[error(transparent)]
    PropertyBank(#[from] PropertyBankError),

    /// Inheritance error.
    #[error(transparent)]
    Inheritance(#[from] SchemaInheritanceError),

    /// Resolution error.
    #[error(transparent)]
    Resolution(#[from] SchemaResolutionError),
}

/// Schema file errors during ingestion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaFileError {
    /// I/O error reading file.
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path to the file.
        path: PathBuf,
        /// I/O error source.
        #[source]
        source: std::io::Error,
    },

    /// Invalid filename or basename.
    #[error("invalid filename: {path} ({reason})")]
    InvalidFilename {
        /// Path to the file.
        path: PathBuf,
        /// Reason for invalid filename.
        reason: Box<str>,
    },

    /// Unsupported file format.
    #[error("unsupported format for {path}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// Path to the file.
        path: PathBuf,
        /// Supported formats.
        supported: Vec<Box<str>>,
    },

    /// File system error not tied to a specific file.
    #[error("filesystem error: {reason}")]
    FileSystem {
        /// Error details.
        reason: Box<str>,
    },
}

/// Schema parsing errors during ingestion.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaParseError {
    /// JSON parsing failed.
    #[error(
        "JSON parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Json {
        /// Path to the file.
        path: PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// TOML parsing failed.
    #[error(
        "TOML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Toml {
        /// Path to the file.
        path: PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// YAML parsing failed.
    #[error(
        "YAML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Yaml {
        /// Path to the file.
        path: PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Cached view deserialization failed.
    #[error("cached view parse error in {path}: {reason}")]
    CachedView {
        /// Path to the file.
        path: PathBuf,
        /// Error details.
        reason: Box<str>,
    },

    /// Serialization failed.
    #[error("serialization error in {path}: {reason}")]
    Serialization {
        /// Path to the file.
        path: PathBuf,
        /// Error details.
        reason: Box<str>,
    },
}

/// Schema version errors during ingestion.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaVersionError {
    /// Unsupported schema version.
    #[error(
        "unsupported schema version in {path}: got '{found}', expected \
         '{expected}'"
    )]
    UnsupportedVersion {
        /// Path to the file.
        path: PathBuf,
        /// Found version.
        found: Box<str>,
        /// Expected version.
        expected: Box<str>,
    },
}

/// Schema storage errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaStorageError {
    /// Storage/database error.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),

    /// Entity not found.
    #[error("not found: {name}")]
    NotFound {
        /// Name or identifier.
        name: Box<str>,
    },

    /// Data corruption detected in storage.
    #[error("data corruption: {reason}")]
    Corruption {
        /// Reason for corruption.
        reason: Box<str>,
    },

    /// `PropertyBank` not found in database.
    #[error(
        "PropertyBank not found in database - initialize by loading schema \
         files or creating properties"
    )]
    PropertyBankNotFound,

    /// Conflict during save/delete operations.
    #[error("conflict: {reason}")]
    Conflict {
        /// Reason for the conflict.
        reason: Box<str>,
    },
}

/// Unified schema repository errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaRepositoryError {
    /// Storage/database error.
    #[error(transparent)]
    Storage(#[from] SchemaStorageError),

    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] SchemaError),
}

/// Schema loader errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaLoaderError {
    /// Ingestion (file I/O or parsing) error.
    #[error(transparent)]
    Ingestion(#[from] SchemaIngestionError),

    /// Repository (storage) error.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Resolution error.
    #[error(transparent)]
    Resolution(#[from] SchemaError),
}

/// Schema ingestion errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaIngestionError {
    /// File system error.
    #[error(transparent)]
    File(#[from] SchemaFileError),

    /// Parse error.
    #[error(transparent)]
    Parse(#[from] SchemaParseError),

    /// Version error.
    #[error(transparent)]
    Version(#[from] SchemaVersionError),

    /// Syntax validation error.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Storage error.
    #[error(transparent)]
    Storage(#[from] SchemaStorageError),

    /// Repository error.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Schema validation error with path context.
    #[error("schema validation failed in {path}: {source}")]
    Schema {
        /// Path to the file.
        path: PathBuf,
        /// Underlying schema error.
        #[source]
        source: SchemaError,
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
