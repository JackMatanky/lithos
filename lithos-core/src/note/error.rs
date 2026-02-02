//! Note error types.
//!
//! This module defines note-specific errors using thiserror for
//! structured error handling.

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
    FrontmatterAccess(#[from] super::frontmatter::FrontmatterError),

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::frontmatter::FrontmatterError;

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
            assert!(!err.to_string().is_empty());
        }
    }
}
