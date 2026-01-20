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

use std::borrow::Cow;

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
///     field: "vault_path".to_string().into(),
///     message: "path cannot be empty".to_string().into(),
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
        field: Box<str>,
        /// Field that is required.
        depends_on: Box<str>,
    },

    /// Encryption-related error for sensitive fields.
    #[error("Encryption error for field {field}: {message}")]
    EncryptionError {
        /// Field that failed encryption/decryption.
        field: Box<str>,
        /// Detailed error message.
        message: Box<str>,
    },

    /// Invalid enum value for configuration field.
    #[error("Invalid enum value for {field}: {value} not in {allowed:?}")]
    InvalidEnumValue {
        /// Field with invalid enum value.
        field: Box<str>,
        /// Value that was provided.
        value: Box<str>,
        /// List of allowed values.
        allowed: Vec<String>,
    },

    /// Invalid configuration value type.
    #[error(
        "Invalid configuration value type for {field}: expected {expected}, \
         got {actual}"
    )]
    InvalidType {
        /// Field with type mismatch.
        field: Box<str>,
        /// Expected type name.
        expected: Box<str>,
        /// Actual type encountered.
        actual: Box<str>,
    },

    /// Configuration merge conflict between hierarchical levels.
    #[error(
        "Configuration merge conflict: {field} has incompatible types at \
         {path1} and {path2}"
    )]
    MergeConflict {
        /// Field with merge conflict.
        field: Box<str>,
        /// Path to first configuration source.
        path1: Box<str>,
        /// Path to second configuration source.
        path2: Box<str>,
    },

    /// Required configuration field is missing.
    #[error("Required configuration field missing: {field}")]
    MissingRequiredField {
        /// Name of the missing field.
        field: Box<str>,
    },

    /// Configuration value out of valid range.
    #[error(
        "Configuration value out of range for {field}: {value} not in \
         {min:?}..{max:?}"
    )]
    OutOfRange {
        /// Field with out-of-range value.
        field: Box<str>,
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
        field: Box<str>,
        /// Detailed error message.
        message: Box<str>,
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
    /// Circular template composition detected.
    #[error("Circular template composition detected: {0}")]
    CircularComposition(String),

    /// Circular schema inheritance detected.
    #[error("Circular schema inheritance detected: {0}")]
    CircularInheritance(String),

    /// Composition depth limit exceeded.
    #[error("Composition depth limit exceeded: {0} (max 5)")]
    CompositionDepthExceeded(usize),

    /// Configuration error.
    #[error(transparent)]
    Config(Box<ConfigError>),

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

    /// Template name cannot be empty.
    #[error("Template name cannot be empty")]
    EmptyTemplateName,

    /// Variable name cannot be empty.
    #[error("Variable name cannot be empty")]
    EmptyVariableName,

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

    /// Invalid link type for operation.
    #[error("Invalid link type for operation")]
    InvalidLinkType,

    /// Invalid note path.
    ///
    /// Uses `Cow<'static, str>` to avoid allocation for static error messages
    /// while still supporting dynamic messages when needed.
    #[error("Invalid note path: {0}")]
    InvalidPath(Cow<'static, str>),

    /// Invalid property name.
    #[error("Invalid property name: {0}")]
    InvalidPropertyName(String),

    /// Invalid regex pattern.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),

    /// Invalid schema name.
    #[error("Invalid schema name: {0}")]
    InvalidSchemaName(String),

    /// Invalid step value.
    #[error("Invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// Invalid tag.
    #[error("Invalid tag: {0}")]
    InvalidTag(String),

    /// Invalid template name.
    #[error("Invalid template name: {0}")]
    InvalidTemplateName(String),

    /// Invalid type.
    #[error("Invalid type: {value} (expected: {expected})")]
    InvalidType {
        /// The value that was provided.
        value: String,
        /// The expected type.
        expected: String,
    },

    /// Invalid variable name.
    #[error("Invalid variable name: {0}")]
    InvalidVariableName(String),

    /// Invalid YAML.
    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(String),

    /// Maximum number of template variables exceeded.
    #[error("Maximum number of template variables exceeded: {0}")]
    MaxVariablesExceeded(usize),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),

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

    /// Parent schema not found.
    #[error("Parent schema not found: {0}")]
    ParentSchemaNotFound(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Property bank error.
    #[error("Property bank error: {0}")]
    PropertyBank(String),

    /// Property name too long.
    #[error("Property name too long: {0} (max 64)")]
    PropertyNameTooLong(usize),

    /// Property not found.
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    /// Schema name too long.
    #[error("Schema name too long: {0} (max 64)")]
    SchemaNameTooLong(usize),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

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

    /// Template content too large.
    #[error("Template content too large: {0} bytes (max: {1})")]
    TemplateContentTooLarge(usize, usize),

    /// Template name too long.
    #[error("Template name too long: {0} (max 64)")]
    TemplateNameTooLong(usize),

    /// Unexpected error.
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    /// Validation failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Variable name too long.
    #[error("Variable name too long: {0} (max 32)")]
    VariableNameTooLong(usize),

    /// Variable not found.
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Variable type mismatch.
    #[error(
        "Variable type mismatch for {name}: expected {expected}, got {actual}"
    )]
    VariableTypeMismatch {
        /// Variable name.
        name: Box<str>,
        /// Expected type name.
        expected: Box<str>,
        /// Actual type encountered.
        actual: Box<str>,
    },
}

impl From<ConfigError> for DomainError {
    #[inline]
    fn from(err: ConfigError) -> Self {
        Self::Config(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn config_error_is_send_and_sync() {
        // GIVEN: the ConfigError type
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN: checking Send + Sync bounds
        // THEN: it satisfies the bounds
        is_send_sync::<ConfigError>();
    }

    #[test]
    fn domain_error_is_send_and_sync() {
        // GIVEN: the DomainError type
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN: checking Send + Sync bounds
        // THEN: it satisfies the bounds
        is_send_sync::<DomainError>();
    }

    #[rstest]
    #[case::validation(
        ConfigError::ValidationFailed {
            field: "vault_path".to_owned().into(),
            message: "cannot be empty".to_owned().into()
        },
        &["vault_path", "cannot be empty"]
    )]
    #[case::missing_field(
        ConfigError::MissingRequiredField {
            field: "templates_dir".to_owned().into()
        },
        &["templates_dir", "missing"]
    )]
    #[case::invalid_type(
        ConfigError::InvalidType {
            field: "log_level".to_owned().into(),
            expected: "String".to_owned().into(),
            actual: "Number".to_owned().into()
        },
        &["log_level", "String", "Number"]
    )]
    fn should_display_correct_error_messages(
        #[case] error: ConfigError,
        #[case] expected_parts: &[&str],
    ) {
        // GIVEN: a configuration error variant
        // WHEN: formatting the error message
        let message = error.to_string();

        // THEN: it contains the expected context parts
        for part in expected_parts {
            assert!(
                message.contains(part),
                "Error message '{message}' should contain '{part}'"
            );
        }
    }

    #[test]
    fn domain_error_display_is_comprehensive() {
        // GIVEN: various domain error variants
        let errors = vec![
            DomainError::CircularComposition("A".into()),
            DomainError::CircularInheritance("B".into()),
            DomainError::DuplicatePropertyName("C".into()),
            DomainError::EmptyPath,
            DomainError::InvalidHeadingLevel(7),
            DomainError::InvalidPath(Cow::Borrowed("err")),
            DomainError::InvalidType {
                value: "v".into(),
                expected: "e".into(),
            },
            DomainError::MaxVariablesExceeded(10),
            DomainError::MissingField("f".into()),
            DomainError::NumberOutOfRange {
                value: 1.0f64,
                min: Some(2.0f64),
                max: None,
            },
            DomainError::ParentSchemaNotFound("p".into()),
            DomainError::PropertyNameTooLong(100),
            DomainError::PropertyNotFound("p".into()),
            DomainError::SchemaNameTooLong(100),
            DomainError::StringTooLong {
                max: 5,
                actual: 10,
            },
            DomainError::StringTooShort {
                min: 5,
                actual: 2,
            },
            DomainError::VariableTypeMismatch {
                name: "v".into(),
                expected: "e".into(),
                actual: "a".into(),
            },
        ];

        // WHEN: formatting them as strings
        // THEN: they all produce non-empty messages
        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn config_error_display_is_comprehensive() {
        // GIVEN: all configuration error variants
        let errors = vec![
            ConfigError::DependencyViolation {
                field: "f".into(),
                depends_on: "d".into(),
            },
            ConfigError::EncryptionError {
                field: "f".into(),
                message: "m".into(),
            },
            ConfigError::InvalidEnumValue {
                field: "f".into(),
                value: "v".into(),
                allowed: vec!["a".into()],
            },
            ConfigError::InvalidType {
                field: "f".into(),
                expected: "e".into(),
                actual: "a".into(),
            },
            ConfigError::MergeConflict {
                field: "f".into(),
                path1: "p1".into(),
                path2: "p2".into(),
            },
            ConfigError::MissingRequiredField {
                field: "f".into(),
            },
            ConfigError::OutOfRange {
                field: "f".into(),
                value: 1.0f64,
                min: Some(0.0f64),
                max: Some(2.0f64),
            },
            ConfigError::ValidationFailed {
                field: "f".into(),
                message: "m".into(),
            },
        ];

        // WHEN: formatting them as strings
        // THEN: they all produce non-empty messages
        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
