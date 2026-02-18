//! Schema error types.
//!
//! This module defines schema-specific errors using thiserror for
//! structured error handling.

use crate::db::DbError;

/// Schema-related errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaError {
    // --- Schema Identity Errors ---
    /// Schema name cannot be empty.
    #[error("Schema name cannot be empty")]
    EmptySchemaName,

    /// Schema name too long.
    #[error("Schema name too long: {0} (max 64)")]
    SchemaNameTooLong(usize),

    /// Invalid schema name.
    #[error("Invalid schema name: {0}")]
    InvalidSchemaName(String),

    /// Schema not found.
    #[error("schema not found: {0}")]
    NotFound(String),

    /// Schema already exists.
    #[error("schema already exists: {0}")]
    AlreadyExists(String),

    // --- Property Identity Errors ---
    /// Property name cannot be empty.
    #[error("Property name cannot be empty")]
    EmptyPropertyName,

    /// Property name too long.
    #[error("Property name too long: {0} (max 64)")]
    PropertyNameTooLong(usize),

    /// Invalid property name.
    #[error("Invalid property name: {0}")]
    InvalidPropertyName(String),

    /// Duplicate property name.
    #[error("Duplicate property name: {0}")]
    DuplicatePropertyName(String),

    /// Property not found.
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    /// Property reference not found.
    #[error("Property reference not found: {0}")]
    PropertyRefNotFound(String),

    /// Duplicate property in schema.
    #[error("Duplicate property: {0}")]
    DuplicateProperty(String),

    // --- Inheritance & Resolution Errors ---
    /// Circular schema inheritance detected.
    #[error("Circular schema inheritance detected: {0}")]
    CircularInheritance(String),

    /// Parent schema not found.
    #[error("Parent not found: {0}")]
    ParentNotFound(String),

    // --- Validation Errors ---
    /// Schema validation failed.
    #[error("schema validation failed: {0}")]
    ValidationFailed(String),

    /// Invalid type.
    #[error("Invalid type: {value} (expected: {expected})")]
    InvalidType {
        /// The value that was provided.
        value: String,
        /// The expected type.
        expected: String,
    },

    /// Invalid date format.
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    /// Invalid directory path.
    #[error("Invalid directory path: {0}")]
    InvalidDirectoryPath(String),

    /// Invalid file class.
    #[error("Invalid file class: {0}")]
    InvalidFileClass(String),

    /// Number out of range.
    #[error("Number out of range: {value} (min: {min:?}, max: {max:?})")]
    NumberOutOfRange {
        /// Provided value.
        value: f64,
        /// Minimum allowed value.
        min: Option<f64>,
        /// Maximum allowed value.
        max: Option<f64>,
    },

    /// Invalid step value.
    #[error("Invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// String too long.
    #[error("String too long: {actual} (max: {max})")]
    StringTooLong {
        /// Maximum length allowed.
        max: usize,
        /// Actual length provided.
        actual: usize,
    },

    /// String too short.
    #[error("String too short: {actual} (min: {min})")]
    StringTooShort {
        /// Minimum length required.
        min: usize,
        /// Actual length provided.
        actual: usize,
    },

    /// Invalid enum value.
    #[error("Invalid enum value: {value} (allowed: {allowed:?})")]
    InvalidEnumValue {
        /// The value that was provided.
        value: String,
        /// The list of allowed values.
        allowed: Vec<String>,
    },

    /// Invalid regex pattern.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),

    /// Property validation error.
    #[error("property error: {0}")]
    Property(String),
}

/// Schema command errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaCommandError {
    /// Domain validation failed.
    #[error("domain validation failed: {0}")]
    Domain(#[from] SchemaError),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),

    /// Conflict during save/delete operations.
    #[error("conflict: {reason}")]
    Conflict {
        /// Reason for the conflict.
        reason: Box<str>,
    },
}

/// Schema query errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaQueryError {
    /// Storage error.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),

    /// Data corruption detected in storage.
    #[error("data corruption: {reason}")]
    Corruption {
        /// Reason for corruption.
        reason: Box<str>,
    },

    /// Entity not found.
    #[error("not found: {name}")]
    NotFound {
        /// Name or identifier.
        name: Box<str>,
    },
}

/// Schema ingestion errors.
///
/// Errors that occur during file-to-raw translation (loading schema files
/// from the filesystem).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaIngestionError {
    /// I/O error reading file.
    #[error("Failed to read file {path}: {reason}")]
    Io {
        /// Path to the file.
        path: Box<str>,
        /// Reason for failure.
        reason: Box<str>,
    },

    /// JSON parsing failed.
    #[error("JSON parse error in {path}: {message}")]
    Json {
        /// Path to the file.
        path: Box<str>,
        /// Error message from parser.
        message: Box<str>,
    },

    /// TOML parsing failed.
    #[error("TOML parse error in {path}: {message}")]
    Toml {
        /// Path to the file.
        path: Box<str>,
        /// Error message from parser.
        message: Box<str>,
    },

    /// YAML parsing failed.
    #[error("YAML parse error in {path}: {message}")]
    Yaml {
        /// Path to the file.
        path: Box<str>,
        /// Error message from parser.
        message: Box<str>,
    },

    /// Unsupported file format.
    #[error("Unsupported format for {path}: expected one of {supported}")]
    UnsupportedFormat {
        /// Path to the file.
        path: Box<str>,
        /// Supported formats.
        supported: Box<str>,
    },

    /// File system error.
    #[error("File system error: {0}")]
    FileSystem(Box<str>),
}

impl From<crate::fs::error::ParseError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ParseError) -> Self {
        use crate::fs::error::ParseError;
        match err {
            ParseError::Io {
                path,
                source,
            } => Self::Io {
                path: path.to_string_lossy().into(),
                reason: source.to_string().into(),
            },
            ParseError::Json {
                path,
                message,
                ..
            } => Self::Json {
                path: path.to_string_lossy().into(),
                message,
            },
            ParseError::Toml {
                path,
                message,
                ..
            } => Self::Toml {
                path: path.to_string_lossy().into(),
                message,
            },
            ParseError::Yaml {
                path,
                message,
                ..
            } => Self::Yaml {
                path: path.to_string_lossy().into(),
                message,
            },
            ParseError::UnsupportedFormat {
                path,
                supported,
            } => Self::UnsupportedFormat {
                path: path.to_string_lossy().into(),
                supported: supported.join(", ").into(),
            },
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
    fn schema_command_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaCommandError>();
    }

    #[test]
    fn schema_query_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaQueryError>();
    }

    #[test]
    fn schema_ingestion_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaIngestionError>();
    }

    #[rstest]
    #[case(SchemaError::NotFound("schema".into()))]
    #[case(SchemaError::AlreadyExists("schema".into()))]
    #[case(SchemaError::ValidationFailed("invalid".into()))]
    #[case(SchemaError::CircularInheritance("cycle".into()))]
    #[case(SchemaError::DuplicateProperty("prop".into()))]
    #[case(SchemaError::ParentNotFound("parent".into()))]
    #[case(SchemaError::PropertyRefNotFound("ref".into()))]
    #[case(SchemaError::Property("invalid property".into()))]
    fn schema_error_display_is_comprehensive(#[case] error: SchemaError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
