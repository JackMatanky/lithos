//! Note domain events.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates Archived types with public fields/variants"
)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
/// use lithos_core::note::events::FrontmatterValidated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = FrontmatterValidated::new(id, 5, 1234567890);
/// assert_eq!(event.note_id, id);
/// assert_eq!(event.field_count, 5);
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FrontmatterValidated {
    /// Number of frontmatter fields validated.
    pub field_count: usize,
    /// UUID v7 of the note containing this frontmatter.
    pub note_id: Uuid,
    /// Unix timestamp when validation occurred.
    pub timestamp: i64,
}

/// Note created domain event.
///
/// Published when a new note is created, allowing other bounded contexts
/// to react to note creation (e.g., indexing, linking).
///
/// # Examples
/// ```
/// use lithos_core::note::events::NoteCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event =
///     NoteCreated::new(id, "projects/lithos.md".to_string(), 1234567890);
/// assert_eq!(event.id, id);
/// assert_eq!(event.path, "projects/lithos.md");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct NoteCreated {
    /// UUID v7 of the note.
    pub id: Uuid,
    /// Vault-relative path of the note.
    pub path: String,
    /// Unix timestamp when the note was created.
    pub timestamp: i64,
}

/// Domain events that can be emitted by the Note aggregate.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum NoteEvents {
    /// Frontmatter was validated.
    FrontmatterValidated(FrontmatterValidated),
    /// Note was created.
    NoteCreated(NoteCreated),
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_NOTE_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0401);

    #[test]
    fn frontmatter_validated_event_populates_fields() {
        let event = FrontmatterValidated::new(TEST_NOTE_ID, 3, 1_234_567_890);

        assert_eq!(event.note_id, TEST_NOTE_ID, "Note ID should match input");
        assert_eq!(event.field_count, 3, "Field count should match input");
        assert_eq!(
            event.timestamp, 1_234_567_890,
            "Timestamp should match input"
        );
    }

    #[test]
    fn note_created_event_populates_fields() {
        let event =
            NoteCreated::new(TEST_NOTE_ID, "notes/test.md".to_owned(), 42);

        assert_eq!(event.id, TEST_NOTE_ID, "Note ID should match input");
        assert_eq!(event.path, "notes/test.md", "Path should match input");
        assert_eq!(event.timestamp, 42, "Timestamp should match input");
    }

    #[test]
    fn note_events_enum_wraps_variants() {
        let validated = FrontmatterValidated::new(TEST_NOTE_ID, 1, 10);
        let created =
            NoteCreated::new(TEST_NOTE_ID, "notes/test.md".to_owned(), 20);

        let wrapped_validated =
            NoteEvents::FrontmatterValidated(validated.clone());
        let wrapped_created = NoteEvents::NoteCreated(created.clone());

        assert_eq!(
            wrapped_validated,
            NoteEvents::FrontmatterValidated(validated),
            "FrontmatterValidated should wrap in NoteEvents"
        );
        assert_eq!(
            wrapped_created,
            NoteEvents::NoteCreated(created),
            "NoteCreated should wrap in NoteEvents"
        );
    }
}
