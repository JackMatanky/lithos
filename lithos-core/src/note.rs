//! Note management - the core domain entity of Lithos.
//!
//! Notes represent markdown files with frontmatter, tags, links, and tasks.
//! This module provides the Note aggregate and related value objects.

/// Note error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum NoteError {
    /// Note is invalid.
    #[error("invalid note: {0}")]
    Invalid(String),
    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),
}

/// Stub note aggregate root.
///
/// Phase 3 will implement real note types with frontmatter, tags, links, tasks.
#[non_exhaustive]
pub struct Note;
