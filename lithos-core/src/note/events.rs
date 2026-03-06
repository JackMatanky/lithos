//! Note events emitted during ingestion and indexing.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates Archived types with public fields/variants"
)]

use std::time::SystemTime;

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};
use uuid::Uuid;

use super::{aggregate::NoteId, paths::NotePath};

/// Event kinds recorded in the note event log.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum NoteEventKind {
    /// Parsed a note from source.
    Parsed,
    /// Persisted note projections.
    Indexed,
    /// Detected a note change or removal.
    Changed,
    /// Failed to ingest or project a note.
    Failed,
}

/// Change classification for note events.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum NoteChangeKind {
    /// Note was created.
    Created,
    /// Note was updated.
    Updated,
    /// Note was deleted.
    Deleted,
}

/// Versioned event payloads stored for note auditability.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum NoteEventPayload {
    /// Version 1 payload.
    V1(NoteEventPayloadV1),
}

/// Version 1 note event payload.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct NoteEventPayloadV1 {
    change: Option<NoteChangeKind>,
    task_count: u32,
    tag_count: u32,
    source_hash: Option<Box<str>>,
    source_bytes: Option<u64>,
    #[rkyv(with = Map<AsUnixTime>)]
    source_modified_at: Option<SystemTime>,
    error_code: Option<Box<str>>,
}

impl NoteEventPayloadV1 {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Payload captures indexing metadata explicitly"
    )]
    pub fn indexed(
        change: NoteChangeKind,
        task_count: u32,
        tag_count: u32,
        source_hash: Option<Box<str>>,
        source_bytes: Option<u64>,
        source_modified_at: Option<SystemTime>,
    ) -> Self {
        Self {
            change: Some(change),
            task_count,
            tag_count,
            source_hash,
            source_bytes,
            source_modified_at,
            error_code: None,
        }
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Payload captures indexing metadata explicitly"
    )]
    pub fn changed(
        change: NoteChangeKind,
        task_count: u32,
        tag_count: u32,
        source_hash: Option<Box<str>>,
        source_bytes: Option<u64>,
        source_modified_at: Option<SystemTime>,
    ) -> Self {
        Self {
            change: Some(change),
            task_count,
            tag_count,
            source_hash,
            source_bytes,
            source_modified_at,
            error_code: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn failed(error_code: Box<str>) -> Self {
        Self {
            change: None,
            task_count: 0,
            tag_count: 0,
            source_hash: None,
            source_bytes: None,
            source_modified_at: None,
            error_code: Some(error_code),
        }
    }

    #[inline]
    #[must_use]
    pub const fn change(&self) -> Option<NoteChangeKind> {
        self.change
    }

    #[inline]
    #[must_use]
    pub const fn task_count(&self) -> u32 {
        self.task_count
    }

    #[inline]
    #[must_use]
    pub const fn tag_count(&self) -> u32 {
        self.tag_count
    }

    #[inline]
    #[must_use]
    pub fn source_hash(&self) -> Option<&str> {
        self.source_hash.as_deref()
    }

    #[inline]
    #[must_use]
    pub const fn source_bytes(&self) -> Option<u64> {
        self.source_bytes
    }

    #[inline]
    #[must_use]
    pub fn source_modified_at(&self) -> Option<SystemTime> {
        self.source_modified_at
    }

    #[inline]
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }
}

/// Stored event record for audit and incremental indexing.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct NoteEvent {
    id: Uuid,
    note_id: NoteId,
    path: NotePath,
    kind: NoteEventKind,
    #[rkyv(with = AsUnixTime)]
    timestamp: SystemTime,
    payload: NoteEventPayload,
}

impl NoteEvent {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "NoteEvent construction needs explicit field values"
    )]
    pub fn new(
        id: Uuid,
        note_id: NoteId,
        path: NotePath,
        kind: NoteEventKind,
        timestamp: SystemTime,
        payload: NoteEventPayload,
    ) -> Self {
        Self {
            id,
            note_id,
            path,
            kind,
            timestamp,
            payload,
        }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    #[must_use]
    pub const fn note_id(&self) -> NoteId {
        self.note_id
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> NoteEventKind {
        self.kind
    }

    #[inline]
    #[must_use]
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    #[inline]
    #[must_use]
    pub fn payload(&self) -> &NoteEventPayload {
        &self.payload
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on reference payload"
    )]
    pub fn payload_v1(&self) -> Option<&NoteEventPayloadV1> {
        match self.payload() {
            NoteEventPayload::V1(payload) => Some(payload),
        }
    }
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
/// let event = NoteCreated::new(id, "projects/lithos.md", 1234567890);
/// assert_eq!(event.id(), id, "Note id should match");
/// assert_eq!(event.path(), "projects/lithos.md", "Path should match");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct NoteCreated {
    /// UUID v7 of the note.
    id: Uuid,
    /// Vault-relative path of the note (immutable).
    path: Box<str>,
    /// Unix timestamp when the note was created.
    timestamp: i64,
}

impl NoteCreated {
    /// Creates a new note created event.
    #[inline]
    #[must_use]
    pub fn new(id: Uuid, path: &str, timestamp: i64) -> Self {
        Self {
            id,
            path: path.into(),
            timestamp,
        }
    }

    /// Returns the note's UUID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the note's vault-relative path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the Unix timestamp when the note was created.
    #[inline]
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
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
/// use lithos_core::note::events::FrontmatterValidated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = FrontmatterValidated::new(id, 5, 1234567890);
/// assert_eq!(event.note_id(), id, "Note id should match");
/// assert_eq!(event.field_count(), 5, "Field count should match");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FrontmatterValidated {
    /// Number of frontmatter fields validated.
    field_count: usize,
    /// UUID v7 of the note containing this frontmatter.
    note_id: Uuid,
    /// Unix timestamp when validation occurred.
    timestamp: i64,
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

    /// Returns the note's UUID.
    #[inline]
    #[must_use]
    pub const fn note_id(&self) -> Uuid {
        self.note_id
    }

    /// Returns the number of frontmatter fields validated.
    #[inline]
    #[must_use]
    pub const fn field_count(&self) -> usize {
        self.field_count
    }

    /// Returns the Unix timestamp when validation occurred.
    #[inline]
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// Domain events that can be emitted by the Note aggregate.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::events::{NoteEvents, NoteCreated};
/// # use uuid::Uuid;
/// let inner = NoteCreated::new(Uuid::now_v7(), "test.md", 0);
/// let event = NoteEvents::NoteCreated(inner);
///
/// match event {
///     NoteEvents::NoteCreated(e) => println!("Note created: {}", e.path()),
///     NoteEvents::FrontmatterValidated(_) => (),
///     _ => (),
/// }
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_NOTE_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0401);

    #[test]
    fn frontmatter_validated_event_populates_fields() {
        let event = FrontmatterValidated::new(TEST_NOTE_ID, 3, 1_234_567_890);

        assert_eq!(event.note_id(), TEST_NOTE_ID, "Note ID should match input");
        assert_eq!(event.field_count(), 3, "Field count should match input");
        assert_eq!(
            event.timestamp(),
            1_234_567_890,
            "Timestamp should match input"
        );
    }

    #[test]
    fn note_created_event_populates_fields() {
        let event = NoteCreated::new(TEST_NOTE_ID, "notes/test.md", 42);

        assert_eq!(event.id(), TEST_NOTE_ID, "Note ID should match input");
        assert_eq!(event.path(), "notes/test.md", "Path should match input");
        assert_eq!(event.timestamp(), 42, "Timestamp should match input");
    }

    #[test]
    fn note_events_enum_wraps_variants() {
        let validated = FrontmatterValidated::new(TEST_NOTE_ID, 1, 10);
        let created = NoteCreated::new(TEST_NOTE_ID, "notes/test.md", 20);

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
