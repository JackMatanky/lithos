//! Schema error types.
//!
//! This module defines schema-specific errors using thiserror for
//! structured error handling.

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

    // --- Inheritance & Resolution Errors ---
    /// Circular schema inheritance detected.
    #[error("Circular schema inheritance detected: {0}")]
    CircularInheritance(String),

    /// Parent schema not found.
    #[error("Parent schema not found: {0}")]
    ParentSchemaNotFound(String),

    /// Resolver error.
    #[error("resolver error: {0}")]
    Resolver(String),

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

    // --- System Errors ---
    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<SchemaError>();
    }

    #[test]
    fn schema_error_display_is_comprehensive() {
        let errors = vec![
            SchemaError::NotFound("schema".into()),
            SchemaError::AlreadyExists("schema".into()),
            SchemaError::ValidationFailed("invalid".into()),
            SchemaError::CircularInheritance("cycle".into()),
            SchemaError::Property("invalid property".into()),
            SchemaError::Resolver("missing reference".into()),
            SchemaError::Storage("io error".into()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
