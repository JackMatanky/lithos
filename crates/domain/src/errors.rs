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
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DomainError {
    /// Configuration error.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Initial placeholder error.
    #[error("Initialization error")]
    Initialize,
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn config_error_validation_failed_message() {
        let error = ConfigError::ValidationFailed {
            field: "vault_path".to_owned(),
            message: "path cannot be empty".to_owned(),
        };

        let message = error.to_string();
        assert!(message.contains("vault_path"));
        assert!(message.contains("path cannot be empty"));
    }

    #[test]
    fn config_error_missing_required_field_message() {
        let error = ConfigError::MissingRequiredField {
            field: "templates_dir".to_owned(),
        };

        let message = error.to_string();
        assert!(message.contains("templates_dir"));
        assert!(message.contains("missing"));
    }

    #[test]
    fn config_error_invalid_type_message() {
        let error = ConfigError::InvalidType {
            field: "log_level".to_owned(),
            expected: "String".to_owned(),
            actual: "Number".to_owned(),
        };

        let message = error.to_string();
        assert!(message.contains("log_level"));
        assert!(message.contains("String"));
        assert!(message.contains("Number"));
    }

    #[test]
    fn domain_error_from_config_error() {
        let config_error = ConfigError::ValidationFailed {
            field: "test".to_owned(),
            message: "test error".to_owned(),
        };

        let domain_error: DomainError = config_error.into();
        assert!(matches!(domain_error, DomainError::Config(_)));
    }
}
