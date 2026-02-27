//! Error types for note domain and persistence operations.

//! Note error types.
//!
//! This module defines note-specific errors using thiserror for
//! structured error handling.

use super::{
    aggregate::{NoteId, NotePath},
    value::FieldValueType,
};

/// Note-related errors.
///
/// This enum covers domain-level errors related to parsing, validation,
/// and consistency of the [`crate::note::aggregate::Note`] aggregate.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteError {
    /// Note already exists.
    #[error("note already exists: {0}")]
    AlreadyExists(NotePath),

    /// Frontmatter parsing error.
    #[error("frontmatter error: {0}")]
    Frontmatter(Box<str>),

    /// Frontmatter access/extraction error.
    #[error(transparent)]
    FrontmatterAccess(#[from] FrontmatterError),

    /// Note path is invalid.
    #[error("invalid note path: {0}")]
    InvalidPath(Box<str>),

    /// Link parsing error.
    #[error("link error: {0}")]
    Link(Box<str>),

    /// Note not found.
    #[error("note not found: {0}")]
    NotFound(NoteId),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(Box<str>),

    /// Tag error.
    #[error("tag error: {0}")]
    Tag(#[from] TagError),

    /// Task error.
    #[error("task error: {0}")]
    Task(Box<str>),

    /// List nesting depth is out of range.
    #[error("list depth out of range: {depth}")]
    ListDepthOutOfRange {
        /// The observed list depth.
        depth: usize,
        /// Conversion error details.
        reason: Box<str>,
    },

    /// Structural error within a note.
    #[error("note structure error: {0}")]
    Structure(Box<str>),

    /// Note validation failed.
    #[error("note validation failed: {0}")]
    ValidationFailed(Box<str>),
}

/// Errors surfaced when validating or parsing tags.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagError {
    /// Tag does not start with '#'.
    #[error("tag must start with #")]
    MissingHash,
    /// Tag is empty after the hash.
    #[error("tag cannot be empty")]
    EmptyTag,
    /// Tag contains an empty path segment.
    #[error("empty tag segment")]
    EmptySegment,
    /// Tag segment contains invalid characters.
    #[error(
        "invalid tag segment '{segment}': only alphanumeric, underscore, and \
         hyphen allowed"
    )]
    InvalidSegment {
        /// The invalid segment text.
        segment: Box<str>,
    },
}

/// Errors surfaced by Note command operations.
///
/// Combines domain errors with low-level storage errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteCommandError {
    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] NoteError),

    /// Storage operation error.
    #[error(transparent)]
    Storage(#[from] crate::db::DbError),
}

/// Errors surfaced by Note query operations.
///
/// Combines domain errors with low-level storage errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteQueryError {
    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] NoteError),

    /// Storage operation error.
    #[error(transparent)]
    Storage(#[from] crate::db::DbError),
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
    use rstest::rstest;

    use super::*;

    #[test]
    fn note_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<NoteError>();
    }

    #[rstest]
    #[case(NoteError::InvalidPath("test.md".into()))]
    #[case(NoteError::NotFound(NoteId::new()))]
    #[case(NoteError::AlreadyExists(
        NotePath::new("test.md").expect("valid path")
    ))]
    #[case(NoteError::ValidationFailed("invalid".into()))]
    #[case(NoteError::Frontmatter("parse error".into()))]
    #[case(NoteError::FrontmatterAccess(FrontmatterError::Missing {
        key: "title".into(),
    }))]
    #[case(NoteError::Link("broken link".into()))]
    #[case(NoteError::Tag(TagError::MissingHash))]
    #[case(NoteError::Task("invalid task".into()))]
    #[case(NoteError::ListDepthOutOfRange {
        depth: 300,
        reason: "out of range".into(),
    })]
    #[case(NoteError::Storage("io error".into()))]
    fn note_error_display_is_comprehensive(#[case] error: NoteError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
