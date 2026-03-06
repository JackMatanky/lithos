//! Stored projection types for task indexing.

#![expect(
    missing_docs,
    reason = "WHAT: rkyv derives create archived/resolver items without docs. \
              WHY: generated storage helpers are internal and not \
              hand-authored. HOW: acknowledge the generated code at module \
              scope to keep signals focused."
)]
#![expect(
    clippy::exhaustive_enums,
    reason = "WHAT: rkyv derives generate archived enums that are exhaustive. \
              WHY: storage enums are internal and evolve with migrations. \
              HOW: set module-level expectation for generated enums."
)]

use std::{fmt::Write as _, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::{
    config::task::{StatusName, StatusSymbol},
    note::{
        aggregate::NoteId,
        frontmatter::Frontmatter,
        link::Link,
        paths::NotePath,
        position::{SourceByteOffset, SourceLocation},
        structure::{Heading, Section},
        tag::Tag,
        task::{TaskId, TaskMetadata, TaskSchedule, TaskText, TaskTimestamp},
        value::FieldValue,
    },
};

/// Stored projection for note-level metadata queries.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StoredNote {
    id: NoteId,
    path: NotePath,
    title: Option<Box<str>>,
    frontmatter: Option<Frontmatter>,
    tags: Vec<Tag>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl StoredNote {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "StoredNote construction needs explicit field values"
    )]
    pub fn new(
        id: NoteId,
        path: NotePath,
        title: Option<Box<str>>,
        frontmatter: Option<Frontmatter>,
        tags: Vec<Tag>,
        headings: Vec<Heading>,
        sections: Vec<Section>,
        links: Vec<Link>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        recorded_at: SystemTime,
    ) -> Self {
        Self {
            id,
            path,
            title,
            frontmatter,
            tags,
            headings,
            sections,
            links,
            created_at,
            modified_at,
            recorded_at,
        }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> NoteId {
        self.id
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    #[inline]
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }
}

/// Event types recorded in the note event log.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum NoteEventKind {
    /// A note was created or first indexed.
    Created,
    /// A note was updated and re-indexed.
    Updated,
    /// A note was removed from the vault.
    Deleted,
}

/// Stored event record for audit and incremental indexing.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StoredNoteEvent {
    id: uuid::Uuid,
    note_id: NoteId,
    path: NotePath,
    kind: NoteEventKind,
    #[rkyv(with = AsUnixTime)]
    timestamp: SystemTime,
    task_count: u32,
    tag_count: u32,
}

impl StoredNoteEvent {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "StoredNoteEvent construction needs explicit field values"
    )]
    pub fn new(
        id: uuid::Uuid,
        note_id: NoteId,
        path: NotePath,
        kind: NoteEventKind,
        timestamp: SystemTime,
        task_count: u32,
        tag_count: u32,
    ) -> Self {
        Self {
            id,
            note_id,
            path,
            kind,
            timestamp,
            task_count,
            tag_count,
        }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> uuid::Uuid {
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
    pub const fn task_count(&self) -> u32 {
        self.task_count
    }

    #[inline]
    #[must_use]
    pub const fn tag_count(&self) -> u32 {
        self.tag_count
    }
}

/// Stored projection for task-level queries.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StoredTask {
    id: TaskId,
    note_id: NoteId,
    path: NotePath,
    heading: Option<Heading>,
    position: SourceByteOffset,
    location: Option<SourceLocation>,
    status_name: StatusName,
    status_symbol: StatusSymbol,
    status_type: Box<str>,
    text: TaskText,
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    schedule: TaskSchedule,
    parent_id: Option<TaskId>,
    block_id: Option<Box<str>>,
    depends_on: Vec<Box<str>>,
}

impl StoredTask {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "StoredTask construction needs explicit field values"
    )]
    pub fn new(
        id: TaskId,
        note_id: NoteId,
        path: NotePath,
        heading: Option<Heading>,
        position: SourceByteOffset,
        location: Option<SourceLocation>,
        status_name: StatusName,
        status_symbol: StatusSymbol,
        status_type: Box<str>,
        text: TaskText,
        tags: Vec<Tag>,
        metadata: TaskMetadata,
        schedule: TaskSchedule,
        parent_id: Option<TaskId>,
        block_id: Option<Box<str>>,
        depends_on: Vec<Box<str>>,
    ) -> Self {
        Self {
            id,
            note_id,
            path,
            heading,
            position,
            location,
            status_name,
            status_symbol,
            status_type,
            text,
            tags,
            metadata,
            schedule,
            parent_id,
            block_id,
            depends_on,
        }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> TaskId {
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
    pub fn heading(&self) -> Option<&Heading> {
        self.heading.as_ref()
    }

    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    #[inline]
    #[must_use]
    pub fn location(&self) -> Option<&SourceLocation> {
        self.location.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn status_name(&self) -> &StatusName {
        &self.status_name
    }

    #[inline]
    #[must_use]
    pub const fn status_symbol(&self) -> StatusSymbol {
        self.status_symbol
    }

    #[inline]
    #[must_use]
    pub fn status_type(&self) -> &str {
        &self.status_type
    }

    #[inline]
    #[must_use]
    pub fn text(&self) -> &TaskText {
        &self.text
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &TaskMetadata {
        &self.metadata
    }

    #[inline]
    #[must_use]
    pub fn schedule(&self) -> &TaskSchedule {
        &self.schedule
    }

    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<TaskId> {
        self.parent_id
    }

    #[inline]
    #[must_use]
    pub fn block_id(&self) -> Option<&str> {
        self.block_id.as_deref()
    }

    #[inline]
    #[must_use]
    pub fn depends_on(&self) -> &[Box<str>] {
        &self.depends_on
    }

    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<TaskTimestamp> {
        self.schedule.created()
    }

    #[inline]
    #[must_use]
    pub fn due_at(&self) -> Option<TaskTimestamp> {
        self.schedule.due()
    }

    #[inline]
    #[must_use]
    pub fn reminder_at(&self) -> Option<TaskTimestamp> {
        self.schedule.reminder()
    }

    #[inline]
    #[must_use]
    pub fn completed_at(&self) -> Option<TaskTimestamp> {
        self.schedule.completed()
    }
}

/// Build typed metadata index keys for the provided field/value pair.
#[inline]
#[must_use]
pub fn metadata_index_keys(field: &str, value: &FieldValue) -> Vec<Box<str>> {
    let mut keys = Vec::new();
    push_metadata_keys(field, value, &mut keys);
    keys
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &FieldValue are intentional"
)]
fn push_metadata_keys(
    field: &str,
    value: &FieldValue,
    out: &mut Vec<Box<str>>,
) {
    match value {
        FieldValue::String(value) => {
            out.push(encode_metadata_key(field, "s:", value).into());
        }
        FieldValue::Number(value) => {
            let mut buffer = ryu::Buffer::new();
            let encoded = buffer.format(*value);
            out.push(encode_metadata_key(field, "n:", encoded).into());
        }
        FieldValue::Boolean(value) => {
            let encoded = if *value {
                "true"
            } else {
                "false"
            };
            out.push(encode_metadata_key(field, "b:", encoded).into());
        }
        FieldValue::Date(value) => {
            let mut buffer = itoa::Buffer::new();
            let encoded = buffer.format(*value);
            out.push(encode_metadata_key(field, "d:", encoded).into());
        }
        FieldValue::Array(values) => {
            for item in values {
                push_metadata_keys(field, item, out);
            }
        }
        FieldValue::Object(_) => {
            let encoded = value.to_json_string();
            out.push(encode_metadata_key(field, "o:", encoded.as_str()).into());
        }
    }
}

fn encode_metadata_key(field: &str, prefix: &str, value: &str) -> String {
    let capacity = field
        .len()
        .saturating_add(prefix.len())
        .saturating_add(value.len())
        .saturating_add(1);
    let mut out = String::with_capacity(capacity);
    #[expect(
        clippy::let_underscore_must_use,
        reason = "Writing to String is infallible"
    )]
    let _ = write!(&mut out, "{field}\0{prefix}{value}");
    out
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Assertions are used to fail tests"
)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::note::{
        error::NoteError, structure::HeadingLevel, value::FieldValue,
    };

    fn system_time_after(seconds: u64) -> Result<SystemTime, NoteError> {
        SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(seconds))
            .ok_or_else(|| NoteError::Storage("timestamp overflow".into()))
    }

    fn build_note() -> Result<StoredNote, NoteError> {
        let id = NoteId::from(Uuid::from_u128(
            0x018C_0000_0000_7000_8000_0000_0000_0002,
        ));
        let path = NotePath::try_new("notes/example.md")?;
        let title = Some("Example".into());

        let frontmatter = Frontmatter::new(HashMap::from([(
            "category".into(),
            FieldValue::String("docs".into()),
        )]));
        let tags = vec![Tag::try_new("#tag")?];
        let headings = vec![Heading::try_new(
            HeadingLevel::try_new(1)?,
            "Heading",
            SourceByteOffset::new(0),
        )?];
        let sections = vec![Section::new(
            headings.first().cloned(),
            crate::note::position::SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(10),
            )?,
        )];
        let links = Vec::new();
        let created_at = Some(system_time_after(10)?);
        let modified_at = Some(system_time_after(20)?);
        let recorded_at = system_time_after(30)?;

        Ok(StoredNote::new(
            id,
            path,
            title,
            Some(frontmatter),
            tags,
            headings,
            sections,
            links,
            created_at,
            modified_at,
            recorded_at,
        ))
    }

    #[test]
    fn stored_note_accessors_expose_fields() -> Result<(), NoteError> {
        let note = build_note()?;

        assert_eq!(note.path().as_str(), "notes/example.md");
        assert_eq!(note.title(), Some("Example"));
        assert_eq!(note.tags().len(), 1);
        assert_eq!(note.headings().len(), 1);
        assert_eq!(note.sections().len(), 1);
        assert_eq!(note.links().len(), 0);
        assert!(note.frontmatter().is_some());
        assert_eq!(note.created_at(), Some(system_time_after(10)?));
        assert_eq!(note.modified_at(), Some(system_time_after(20)?));
        assert_eq!(note.recorded_at(), system_time_after(30)?);
        Ok(())
    }

    #[test]
    fn stored_note_round_trips_through_rkyv() -> Result<(), NoteError> {
        let original = build_note()?;

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original)
            .map_err(|error| NoteError::Storage(error.to_string().into()))?;
        let archived = rkyv::access::<
            rkyv::Archived<StoredNote>,
            rkyv::rancor::Error,
        >(&bytes)
        .map_err(|error| NoteError::Storage(error.to_string().into()))?;
        let deserialized: StoredNote = rkyv::deserialize::<
            StoredNote,
            rkyv::rancor::Error,
        >(archived)
        .map_err(|error| NoteError::Storage(error.to_string().into()))?;

        assert_eq!(deserialized, original);
        Ok(())
    }

    #[test]
    fn stored_note_event_accessors_expose_fields() -> Result<(), NoteError> {
        let id = Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0501);
        let note_id = NoteId::from(Uuid::from_u128(
            0x018C_0000_0000_7000_8000_0000_0000_0502,
        ));
        let path = NotePath::try_new("notes/event.md")?;
        let event = StoredNoteEvent::new(
            id,
            note_id,
            path,
            NoteEventKind::Created,
            system_time_after(123)?,
            2,
            3,
        );

        assert_eq!(event.id(), id);
        assert_eq!(event.note_id(), note_id);
        assert_eq!(event.path().as_str(), "notes/event.md");
        assert_eq!(event.kind(), NoteEventKind::Created);
        let event_secs = event
            .timestamp()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|error| NoteError::Storage(error.to_string().into()))?
            .as_secs();
        assert_eq!(event_secs, 123);
        assert_eq!(event.task_count(), 2);
        assert_eq!(event.tag_count(), 3);
        Ok(())
    }

    #[test]
    fn stored_note_event_round_trips_through_rkyv() -> Result<(), NoteError> {
        let event = StoredNoteEvent::new(
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0503),
            NoteId::from(Uuid::from_u128(
                0x018C_0000_0000_7000_8000_0000_0000_0504,
            )),
            NotePath::try_new("notes/event.md")?,
            NoteEventKind::Updated,
            system_time_after(456)?,
            1,
            0,
        );

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&event)
            .map_err(|error| NoteError::Storage(error.to_string().into()))?;
        let archived = rkyv::access::<
            rkyv::Archived<StoredNoteEvent>,
            rkyv::rancor::Error,
        >(&bytes)
        .map_err(|error| NoteError::Storage(error.to_string().into()))?;
        let deserialized: StoredNoteEvent = rkyv::deserialize::<
            StoredNoteEvent,
            rkyv::rancor::Error,
        >(archived)
        .map_err(|error| NoteError::Storage(error.to_string().into()))?;

        assert_eq!(deserialized, event);
        Ok(())
    }
}
