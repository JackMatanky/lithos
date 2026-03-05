//! Stored projection types for task indexing.

#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::fmt::Write as _;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    config::task::{StatusName, StatusSymbol},
    note::{
        aggregate::NoteId,
        paths::NotePath,
        position::{SourceByteOffset, SourceLocation},
        structure::Heading,
        tag::Tag,
        task::{TaskId, TaskMetadata, TaskSchedule, TaskText, TaskTimestamp},
        value::FieldValue,
    },
};

/// Stored projection for task-level queries.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
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
