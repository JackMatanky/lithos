//! Note events emitted during ingestion and indexing.
#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived enums that are exhaustive"
)]
use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};
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
    error_code: Option<Box<str>>,
}

impl NoteEventPayloadV1 {
    #[inline]
    #[must_use]
    /// Creates a payload for an indexed note event.
    pub fn indexed(
        change: NoteChangeKind,
        task_count: u32,
        tag_count: u32,
    ) -> Self {
        Self {
            change: Some(change),
            task_count,
            tag_count,
            error_code: None,
        }
    }

    #[inline]
    #[must_use]
    /// Creates a payload for a change-detected note event.
    pub fn changed(
        change: NoteChangeKind,
        task_count: u32,
        tag_count: u32,
    ) -> Self {
        Self {
            change: Some(change),
            task_count,
            tag_count,
            error_code: None,
        }
    }

    #[inline]
    #[must_use]
    /// Creates a payload for a failed ingestion event.
    pub fn failed(error_code: Box<str>) -> Self {
        Self {
            change: None,
            task_count: 0,
            tag_count: 0,
            error_code: Some(error_code),
        }
    }

    #[inline]
    #[must_use]
    /// Returns the change classification, if present.
    pub const fn change(&self) -> Option<NoteChangeKind> {
        self.change
    }

    #[inline]
    #[must_use]
    /// Returns the task count recorded in the payload.
    pub const fn task_count(&self) -> u32 {
        self.task_count
    }

    #[inline]
    #[must_use]
    /// Returns the tag count recorded in the payload.
    pub const fn tag_count(&self) -> u32 {
        self.tag_count
    }

    #[inline]
    #[must_use]
    /// Returns the error code, if recorded.
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
    /// Creates a new note event record.
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
    /// Returns the event id.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    #[must_use]
    /// Returns the note id associated with the event.
    pub const fn note_id(&self) -> NoteId {
        self.note_id
    }

    #[inline]
    #[must_use]
    /// Returns the note path associated with the event.
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    #[inline]
    #[must_use]
    /// Returns the event kind.
    pub const fn kind(&self) -> NoteEventKind {
        self.kind
    }

    #[inline]
    #[must_use]
    /// Returns the event timestamp.
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    #[inline]
    #[must_use]
    /// Returns the event payload.
    pub fn payload(&self) -> &NoteEventPayload {
        &self.payload
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on reference payload"
    )]
    /// Returns the V1 payload, if present.
    pub fn payload_v1(&self) -> Option<&NoteEventPayloadV1> {
        match self.payload() {
            NoteEventPayload::V1(payload) => Some(payload),
        }
    }
}
