//! Support module error types.

#![expect(
    clippy::module_name_repetitions,
    reason = "Error suffix is intentional for error types"
)]

use uuid::Version;

/// Errors for UUID v7 validation and parsing.
#[non_exhaustive]
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum UuidV7Error {
    /// Parsing the UUID string failed.
    #[error("failed to parse UUID: {0}")]
    Parse(#[source] uuid::Error),

    /// The UUID is not version 7.
    #[error("expected UUID version 7, got {got:?}")]
    WrongVersion {
        /// Actual UUID version observed from parsed/provided UUID.
        got: Option<Version>,
    },

    /// Invalid byte slice (wrong length).
    #[error("invalid UUID bytes: expected 16 bytes, got {0}")]
    InvalidBytes(#[source] uuid::Error),
}
