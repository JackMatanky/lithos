//! Application-layer error types for ingestion services.

use crate::{fs::ParseError, schema::error::SchemaCommandError};

/// Errors that can occur during file ingestion operations.
///
/// This error type unifies errors from different layers:
/// - File I/O and parsing errors (infrastructure layer)
/// - Domain validation errors (domain layer)
/// - Persistence errors (storage layer)
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IngestionError {
    /// File parsing failed (I/O or format error).
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Domain validation failed during Raw → Domain conversion.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Schema persistence command failed.
    #[error("Schema command error: {0}")]
    SchemaCommand(#[from] SchemaCommandError),

    /// Template persistence failed.
    ///
    /// Note: This is a placeholder. When template errors are properly defined,
    /// this should use `#[from] TemplateError` instead of String.
    #[error("Template command error: {0}")]
    TemplateCommand(String),

    /// Note persistence failed.
    ///
    /// Note: This is a placeholder. When note errors are properly defined,
    /// this should use `#[from] NoteError` instead of String.
    #[error("Note command error: {0}")]
    NoteCommand(String),
}
