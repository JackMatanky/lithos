//! Note domain events.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Note created domain event.
///
/// Published when a new note is created, allowing other bounded contexts
/// to react to note creation (e.g., indexing, linking).
///
/// # Examples
/// ```
/// use lithos_core::note::NoteCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event =
///     NoteCreated::new(id, "projects/lithos.md".to_string(), 1234567890);
/// assert_eq!(event.id, id);
/// assert_eq!(event.path, "projects/lithos.md");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NoteCreated {
    /// UUID v7 of the note.
    pub id: Uuid,
    /// Vault-relative path of the note.
    pub path: String,
    /// Unix timestamp when the note was created.
    pub timestamp: i64,
}

impl NoteCreated {
    /// Creates a new note created event.
    #[inline]
    #[must_use]
    pub fn new(id: Uuid, path: String, timestamp: i64) -> Self {
        Self {
            id,
            path,
            timestamp,
        }
    }
}

/// Frontmatter validated domain event.
///
/// Published when frontmatter has been validated against schema in the
/// application layer, allowing other systems to react to validated metadata.
///
/// # Emission Point
/// This event is emitted by the application layer after schema compliance
/// validation, NOT by the domain layer. The domain layer only validates
/// structural consistency.
///
/// # Examples
/// ```
/// use lithos_core::note::FrontmatterValidated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = FrontmatterValidated::new(id, 5, 1234567890);
/// assert_eq!(event.note_id, id);
/// assert_eq!(event.field_count, 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrontmatterValidated {
    /// Number of frontmatter fields validated.
    pub field_count: usize,
    /// UUID v7 of the note containing this frontmatter.
    pub note_id: Uuid,
    /// Unix timestamp when validation occurred.
    pub timestamp: i64,
}

impl FrontmatterValidated {
    /// Creates a new frontmatter validated event.
    #[inline]
    #[must_use]
    pub fn new(note_id: Uuid, field_count: usize, timestamp: i64) -> Self {
        Self {
            field_count,
            note_id,
            timestamp,
        }
    }
}

/// Domain events that can be emitted by the Note aggregate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteEvents {
    /// Frontmatter was validated.
    FrontmatterValidated(FrontmatterValidated),
    /// Note was created.
    NoteCreated(NoteCreated),
}
