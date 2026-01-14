//! Domain error types for Lithos.
//!
//! This module defines all domain-level errors using thiserror for structured
//! error handling following hexagonal architecture principles.
//!
//! # Error Handling Strategy
//! - Use `thiserror::Error` for all domain errors
//! - Each error variant includes descriptive context
//! - Errors are `Send + Sync` for use across async boundaries
//! - Use `#[from]` attribute for automatic error conversions

/// Configuration-related domain errors.
///
/// # Invariants
/// - All errors must be `Send + Sync` for async contexts.
/// - Error messages must be descriptive and actionable.
/// - Use `#[from]` for automatic conversions from underlying errors.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigError;
///
/// let error = ConfigError::ValidationFailed {
///     field: "vault_path".to_string(),
///     message: "path cannot be empty".to_string(),
/// };
/// assert!(error.to_string().contains("vault_path"));
/// ```
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigError {
    /// Configuration dependency violation.
    #[error(
        "Configuration dependency violation: {field} requires {depends_on}"
    )]
    DependencyViolation {
        /// Field that has unmet dependency.
        field: String,
        /// Field that is required.
        depends_on: String,
    },

    /// Encryption-related error for sensitive fields.
    #[error("Encryption error for field {field}: {message}")]
    EncryptionError {
        /// Field that failed encryption/decryption.
        field: String,
        /// Detailed error message.
        message: String,
    },

    /// Invalid enum value for configuration field.
    #[error("Invalid enum value for {field}: {value} not in {allowed:?}")]
    InvalidEnumValue {
        /// Field with invalid enum value.
        field: String,
        /// Value that was provided.
        value: String,
        /// List of allowed values.
        allowed: Vec<String>,
    },

    /// Invalid configuration value type.
    #[error(
        "Invalid configuration value type for {field}: expected {expected}, got {actual}"
    )]
    InvalidType {
        /// Field with type mismatch.
        field: String,
        /// Expected type name.
        expected: String,
        /// Actual type encountered.
        actual: String,
    },

    /// Configuration merge conflict between hierarchical levels.
    #[error(
        "Configuration merge conflict: {field} has incompatible types at {path1} and {path2}"
    )]
    MergeConflict {
        /// Field with merge conflict.
        field: String,
        /// Path to first configuration source.
        path1: String,
        /// Path to second configuration source.
        path2: String,
    },

    /// Required configuration field is missing.
    #[error("Required configuration field missing: {field}")]
    MissingRequiredField {
        /// Name of the missing field.
        field: String,
    },

    /// Configuration value out of valid range.
    #[error(
        "Configuration value out of range for {field}: {value} not in {min:?}..{max:?}"
    )]
    OutOfRange {
        /// Field with out-of-range value.
        field: String,
        /// Actual value provided.
        value: f64,
        /// Minimum allowed value (if any).
        min: Option<f64>,
        /// Maximum allowed value (if any).
        max: Option<f64>,
    },

    /// Configuration validation failed for a specific field.
    #[error("Configuration validation failed: {field} - {message}")]
    ValidationFailed {
        /// Field that failed validation.
        field: String,
        /// Detailed error message.
        message: String,
    },
}

/// General domain errors.
///
/// # Invariants.
/// - Must remain backwards compatible (use #[`non_exhaustive`]).
/// - All variants must be Send + Sync.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainError {
    /// Circular schema inheritance detected.
    #[error("Circular schema inheritance detected: {0}")]
    CircularInheritance(String),

    /// Circular template composition detected.
    #[error("Circular template composition detected: {0}")]
    CircularComposition(String),

    /// Composition depth limit exceeded.
    #[error("Composition depth limit exceeded: {0} (max 5)")]
    CompositionDepthExceeded(usize),

    /// Configuration error.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Duplicate property name.
    #[error("Duplicate property name: {0}")]
    DuplicatePropertyName(String),

    /// Embed target path cannot be empty.
    #[error("Embed target path cannot be empty")]
    EmptyEmbedTarget,

    /// Link target path cannot be empty.
    #[error("Link target path cannot be empty")]
    EmptyLinkTarget,

    /// Path cannot be empty.
    #[error("Path cannot be empty")]
    EmptyPath,

    /// Property name cannot be empty.
    #[error("Property name cannot be empty")]
    EmptyPropertyName,

    /// Schema name cannot be empty.
    #[error("Schema name cannot be empty")]
    EmptySchemaName,

    /// Tag segment cannot be empty.
    #[error("Tag segment cannot be empty")]
    EmptyTagSegment,

    /// Initial placeholder error.
    #[error("Initialization error")]
    Initialize,

    /// Invalid date format.
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    /// Invalid directory path.
    #[error("Invalid directory path: {0}")]
    InvalidDirectoryPath(String),

    /// Invalid enum value.
    #[error("Invalid enum value: {value} (allowed: {allowed:?})")]
    InvalidEnumValue {
        /// The value that was provided.
        value: String,
        /// The list of allowed values.
        allowed: Vec<String>,
    },

    /// Invalid file class.
    #[error("Invalid file class: {0}")]
    InvalidFileClass(String),

    /// Invalid heading level.
    #[error("Invalid heading level: {0} (must be 1-6)")]
    InvalidHeadingLevel(u8),

    /// Invalid note path.
    #[error("Invalid note path: {0}")]
    InvalidPath(String),

    /// Invalid property name.
    #[error("Invalid property name: {0}")]
    InvalidPropertyName(String),

    /// Invalid regex pattern.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegexPattern(String),

    /// Invalid schema name.
    #[error("Invalid schema name: {0}")]
    InvalidSchemaName(String),

    /// Invalid step value.
    #[error("Invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// The value that was provided.
        value: f64,
        /// The expected step increment.
        step: f64,
    },

    /// Invalid tag format.
    #[error("Invalid tag format: {0}")]
    InvalidTag(String),

    /// Invalid task status.
    #[error("Invalid task status: {0}")]
    InvalidTaskStatus(String),

    /// Invalid UUID format.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    /// Maximum variables exceeded.
    #[error("Maximum variables exceeded: {0} (max 50)")]
    MaxVariablesExceeded(usize),

    /// Number out of range.
    #[error("Number out of range: {value} (min: {min:?}, max: {max:?})")]
    NumberOutOfRange {
        /// The value that was provided.
        value: f64,
        /// Minimum allowed value (if any).
        min: Option<f64>,
        /// Maximum allowed value (if any).
        max: Option<f64>,
    },

    /// Parent schema not found.
    #[error("Parent schema not found: {0}")]
    ParentSchemaNotFound(String),

    /// Property name too long.
    #[error("Property name too long: {0} (max 64)")]
    PropertyNameTooLong(usize),

    /// Schema name too long.
    #[error("Schema name too long: {0} (max 64)")]
    SchemaNameTooLong(usize),

    /// String too long.
    #[error("String too long: {actual} (max: {max})")]
    StringTooLong {
        /// Maximum allowed length.
        max: usize,
        /// Actual length encountered.
        actual: usize,
    },

    /// String too short.
    #[error("String too short: {actual} (min: {min})")]
    StringTooShort {
        /// Minimum required length.
        min: usize,
        /// Actual length encountered.
        actual: usize,
    },

    /// Template content too large.
    #[error("Template content too large: {0} (max 1MB)")]
    TemplateContentTooLarge(usize),

    /// Template not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// General validation failure.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Variable not found.
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Variable type mismatch.
    #[error(
        "Variable type mismatch for {name}: expected {expected}, got {actual}"
    )]
    VariableTypeMismatch {
        /// Variable name.
        name: String,
        /// Expected type name.
        expected: String,
        /// Actual type encountered.
        actual: String,
    },
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn config_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<ConfigError>();
    }

    #[test]
    fn domain_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<DomainError>();
    }

    #[rstest]
    #[case::validation(
        ConfigError::ValidationFailed {
            field: "vault_path".to_owned(),
            message: "cannot be empty".to_owned()
        },
        &["vault_path", "cannot be empty"]
    )]
    #[case::missing_field(
        ConfigError::MissingRequiredField {
            field: "templates_dir".to_owned()
        },
        &["templates_dir", "missing"]
    )]
    #[case::invalid_type(
        ConfigError::InvalidType {
            field: "log_level".to_owned(),
            expected: "String".to_owned(),
            actual: "Number".to_owned()
        },
        &["log_level", "String", "Number"]
    )]
    fn should_display_correct_error_messages(
        #[case] error: ConfigError,
        #[case] expected_parts: &[&str],
    ) {
        let message = error.to_string();
        for part in expected_parts {
            assert!(
                message.contains(part),
                "Error message '{message}' should contain '{part}'"
            );
        }
    }

    #[test]
    fn should_convert_config_to_domain_error() {
        let config_error = ConfigError::ValidationFailed {
            field: "test".to_owned(),
            message: "test error".to_owned(),
        };

        let domain_error: DomainError = config_error.into();
        assert!(
            matches!(domain_error, DomainError::Config(_)),
            "Expected DomainError::Config variant"
        );
    }
}
