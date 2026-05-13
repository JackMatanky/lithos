//! Configuration error types.
//!
//! This module defines the [`ConfigError`] hierarchy, covering ingestion,
//! validation failures, and storage-layer errors.

use std::path::PathBuf;

use crate::db::DbError;

/// Primary error type for configuration operations.
///
/// This enum covers all domain-level validation failures, dependency
/// violations, and type mismatches that can occur during configuration
/// construction.
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
        value: Box<str>,
        /// Minimum allowed value (if any).
        min: Option<Box<str>>,
        /// Maximum allowed value (if any).
        max: Option<Box<str>>,
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

    /// Configuration ingestion failed.
    #[error("Ingestion error: {0}")]
    Ingestion(Box<str>),
}

/// Errors returned by configuration command operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigCommandError {
    /// Domain-level validation or merge error.
    #[error("Domain error: {0}")]
    Domain(#[from] ConfigError),
    /// Storage-layer error.
    #[error("Storage error: {0}")]
    Storage(#[from] DbError),
    /// Config ingestion error.
    #[error("Ingest error: {0}")]
    Ingest(#[from] Box<ConfigIngestError>),
}

/// Errors returned by configuration query operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigQueryError {
    /// Storage-layer error.
    #[error("Storage error: {0}")]
    Storage(#[from] DbError),
    /// Data corruption or missing read model.
    #[error("Data corruption: {0}")]
    Corruption(Box<str>),
}

/// Errors returned while ingesting raw configuration sources.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigIngestError {
    /// I/O error reading config file.
    #[error("Failed to read config file {path}: {source}")]
    Io {
        /// Path to the config file.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// TOML parsing error.
    #[error("Failed to parse TOML config file {path}: {source}")]
    TomlParse {
        /// Path to the config file.
        path: std::path::PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },

    /// Path was not within the expected base directory.
    #[error("Path {path} is not within base directory {base}")]
    NotInBasePath {
        /// The path that was outside the base.
        path: std::path::PathBuf,
        /// The expected base directory.
        base: std::path::PathBuf,
    },
}

impl From<ConfigIngestError> for ConfigCommandError {
    #[inline]
    fn from(error: ConfigIngestError) -> Self {
        Self::Ingest(Box::new(error))
    }
}

impl From<ConfigIngestError> for ConfigError {
    #[inline]
    fn from(error: ConfigIngestError) -> Self {
        Self::Ingestion(error.to_string().into())
    }
}

impl From<DbError> for ConfigError {
    #[inline]
    fn from(error: DbError) -> Self {
        Self::Storage(error.to_string().into())
    }
}

impl From<crate::fs::ParseError> for ConfigIngestError {
    #[inline]
    fn from(error: crate::fs::ParseError) -> Self {
        match error {
            crate::fs::ParseError::Toml {
                path,
                message,
                line,
                column,
            } => {
                // Create a synthetic toml::de::Error since it doesn't have a
                // public constructor. We parse invalid TOML to
                // get an error instance.
                #[expect(
                    clippy::expect_used,
                    reason = "Intentionally parsing invalid TOML to create \
                              error instance"
                )]
                let source = toml::from_str::<toml::Value>("[")
                    .expect_err("Invalid TOML should always error");

                // Log the original error details for debugging
                tracing::warn!(
                    path = %path.display(),
                    ?line,
                    ?column,
                    message = %message,
                    "TOML parsing error"
                );

                Self::TomlParse {
                    path,
                    source,
                }
            }
            crate::fs::ParseError::Json {
                path,
                ..
            }
            | crate::fs::ParseError::Yaml {
                path,
                ..
            }
            | crate::fs::ParseError::UnsupportedFormat {
                path,
                ..
            } => Self::Io {
                path,
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Unsupported format",
                ),
            },
        }
    }
}

impl From<crate::fs::ReadError> for ConfigIngestError {
    #[inline]
    fn from(error: crate::fs::ReadError) -> Self {
        match error {
            crate::fs::ReadError::Io {
                path,
                source,
            } => Self::Io {
                path,
                source,
            },
            crate::fs::ReadError::NotInBase {
                path,
                base,
            } => Self::NotInBasePath {
                path,
                base,
            },
        }
    }
}

impl From<crate::fs::FsError> for ConfigIngestError {
    #[inline]
    fn from(error: crate::fs::FsError) -> Self {
        match error {
            crate::fs::FsError::Read(e) => Self::from(e),
            crate::fs::FsError::Scan(e) => Self::from(e),
            crate::fs::FsError::Parse(e) => Self::from(e),
            crate::fs::FsError::Path(e) => Self::from(e),
            crate::fs::FsError::Validation(e) => Self::Io {
                path: PathBuf::from("unknown"),
                source: std::io::Error::other(e.to_string()),
            },
        }
    }
}

impl From<crate::fs::ScanError> for ConfigIngestError {
    #[inline]
    fn from(error: crate::fs::ScanError) -> Self {
        match error {
            crate::fs::ScanError::Traversal {
                path,
                source,
            } => Self::Io {
                path,
                source,
            },
            crate::fs::ScanError::InvalidPattern {
                pattern,
                message,
            } => Self::Io {
                path: PathBuf::from(pattern.as_ref()),
                source: std::io::Error::other(message.as_ref()),
            },
            crate::fs::ScanError::UnsupportedEntryType(path) => Self::Io {
                path,
                source: std::io::Error::other("Unsupported entry type"),
            },
            crate::fs::ScanError::Path(e) => Self::from(e),
        }
    }
}

impl From<crate::fs::PathError> for ConfigIngestError {
    #[inline]
    fn from(error: crate::fs::PathError) -> Self {
        Self::Io {
            path: PathBuf::from("unknown"),
            source: std::io::Error::other(error.to_string()),
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    mod fixtures {
        // Shared fixtures for error tests
    }

    use rstest::rstest;

    use super::*;

    #[test]
    fn config_invalid_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<ConfigError>();
    }

    #[rstest]
    #[case::validation_field(
        ConfigError::ValidationFailed {
            field: "vault_path".into(),
            message: "cannot be empty".into()
        },
        "vault_path"
    )]
    #[case::validation_message(
        ConfigError::ValidationFailed {
            field: "vault_path".into(),
            message: "cannot be empty".into()
        },
        "cannot be empty"
    )]
    #[case::missing_field_name(
        ConfigError::MissingRequiredField {
            field: "templates_dir".into()
        },
        "templates_dir"
    )]
    #[case::missing_field_message(
        ConfigError::MissingRequiredField {
            field: "templates_dir".into()
        },
        "missing"
    )]
    #[case::invalid_type_field(
        ConfigError::InvalidType {
            field: "log_level".into(),
            expected: "String".into(),
            actual: "Number".into()
        },
        "log_level"
    )]
    #[case::invalid_type_expected(
        ConfigError::InvalidType {
            field: "log_level".into(),
            expected: "String".into(),
            actual: "Number".into()
        },
        "String"
    )]
    #[case::invalid_type_actual(
        ConfigError::InvalidType {
            field: "log_level".into(),
            expected: "String".into(),
            actual: "Number".into()
        },
        "Number"
    )]
    fn should_display_correct_error_messages(
        #[case] error: ConfigError,
        #[case] expected_part: &str,
    ) {
        let message = error.to_string();
        assert!(
            message.contains(expected_part),
            "Error message '{message}' should contain '{expected_part}'"
        );
    }

    #[rstest]
    #[case(ConfigError::DependencyViolation {
        field: "f".into(),
        depends_on: "d".into(),
    })]
    #[case(ConfigError::EncryptionError {
        field: "f".into(),
        message: "m".into(),
    })]
    #[case(ConfigError::InvalidEnumValue {
        field: "f".into(),
        value: "v".into(),
        allowed: vec!["a".into()],
    })]
    #[case(ConfigError::InvalidType {
        field: "f".into(),
        expected: "e".into(),
        actual: "a".into(),
    })]
    #[case(ConfigError::MergeConflict {
        field: "f".into(),
        path1: "p1".into(),
        path2: "p2".into(),
    })]
    #[case(ConfigError::MissingRequiredField { field: "f".into() })]
    #[case(ConfigError::OutOfRange {
        field: "f".into(),
        value: "1".into(),
        min: Some("0".into()),
        max: Some("2".into()),
    })]
    #[case(ConfigError::ValidationFailed {
        field: "f".into(),
        message: "m".into(),
    })]
    fn config_error_display_is_comprehensive(#[case] error: ConfigError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
