//! Configuration error types.
//!
//! This module defines configuration-specific errors using thiserror for
//! structured error handling.

/// Configuration-related errors.
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

    /// Storage operation failed.
    #[error("Storage error: {0}")]
    Storage(Box<str>),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn config_invalid_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<ConfigError>();
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
        let message = error.to_string();
        for part in expected_parts {
            assert!(
                message.contains(part),
                "Error message '{message}' should contain '{part}'"
            );
        }
    }

    #[test]
    fn config_error_display_is_comprehensive() {
        let errors: Vec<ConfigError> = vec![
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

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
