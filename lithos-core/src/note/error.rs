//! Note error types.
//!
//! This module defines note-specific errors using thiserror for
//! structured error handling.

use super::frontmatter::FieldValueType;

/// Note-related errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteError {
    /// Note already exists.
    #[error("note already exists: {0}")]
    AlreadyExists(String),

    /// Frontmatter parsing error.
    #[error("frontmatter error: {0}")]
    Frontmatter(String),

    /// Frontmatter access/extraction error.
    #[error(transparent)]
    FrontmatterAccess(#[from] FrontmatterError),

    /// Note path is invalid.
    #[error("invalid note path: {0}")]
    InvalidPath(String),

    /// Link parsing error.
    #[error("link error: {0}")]
    Link(String),

    /// Note not found.
    #[error("note not found: {0}")]
    NotFound(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Tag error.
    #[error("tag error: {0}")]
    Tag(String),

    /// Task error.
    #[error("task error: {0}")]
    Task(String),

    /// Note validation failed.
    #[error("note validation failed: {0}")]
    ValidationFailed(String),
}

/// Errors surfaced by strict frontmatter accessors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontmatterError {
    /// A required key was missing from the frontmatter map.
    #[error("missing frontmatter key: {key}")]
    Missing {
        /// The missing key.
        key: Box<str>,
    },

    /// A key exists, but the value has an unexpected runtime type.
    #[error(
        "frontmatter key '{key}' has wrong type (expected {expected}, got \
         {actual})"
    )]
    TypeMismatch {
        /// The key that was requested.
        key: Box<str>,
        /// The expected type description.
        expected: Box<str>,
        /// The actual runtime type.
        actual: FieldValueType,
    },

    /// A key exists and is an array, but at least one element has the wrong
    /// type.
    #[error(
        "frontmatter key '{key}' has wrong array element type at index \
         {index} (expected {expected}, got {actual})"
    )]
    ArrayElementTypeMismatch {
        /// The key that was requested.
        key: Box<str>,
        /// The index of the first mismatched array element.
        index: usize,
        /// The expected element type.
        expected: FieldValueType,
        /// The actual element type.
        actual: FieldValueType,
    },

    /// A key exists and is a date timestamp, but the timestamp is not
    /// representable as a UTC datetime.
    #[error("frontmatter key '{key}' has invalid date timestamp: {timestamp}")]
    InvalidDateTimestamp {
        /// The key that was requested.
        key: Box<str>,
        /// The invalid timestamp.
        timestamp: i64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<NoteError>();
    }

    #[test]
    fn note_error_display_is_comprehensive() {
        let errors = vec![
            NoteError::InvalidPath("test.md".into()),
            NoteError::NotFound("uuid".into()),
            NoteError::AlreadyExists("test.md".into()),
            NoteError::ValidationFailed("invalid".into()),
            NoteError::Frontmatter("parse error".into()),
            NoteError::FrontmatterAccess(FrontmatterError::Missing {
                key: "title".into(),
            }),
            NoteError::Link("broken link".into()),
            NoteError::Tag("invalid tag".into()),
            NoteError::Task("invalid task".into()),
            NoteError::Storage("io error".into()),
        ];

        for err in errors {
            assert!(
                !err.to_string().is_empty(),
                "Error {err:?} should have non-empty display message"
            );
        }
    }
}
