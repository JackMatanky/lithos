//! Task sub-entity and temporal management.
//!
//! Defines the [`crate::note::task::Task`] entity and its specialized
//! components, including semantic timestamp handling and metadata extraction.

//! Task value object for notes.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::{borrow::Borrow, collections::HashMap, fmt};

use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::{NoteError, TaskError},
    position::SourceByteOffset,
    tag::Tag,
    value::FieldValue,
};
use crate::config::{
    task::{StatusName, StatusSymbol, Task as TaskConfig},
    value::{DateSpec, FieldSpec},
};

/// Task entity within a Note.
///
/// Represents a promoted checkbox item from a markdown list. A `Task` carries
/// additional domain semantics such as status, temporal data (due dates, etc.),
/// and rich metadata extracted from inline tags and brackets.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{task::Task, position::SourceByteOffset};
/// # use lithos_core::config::task::StatusName;
/// # use lithos_core::note::task::TaskAttributes;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = StatusName::try_new("todo")?;
/// let task = Task::try_new(
///     status,
///     "Urgent work",
///     SourceByteOffset::new(0),
///     TaskAttributes::default(),
/// )?;
///
/// assert_eq!(task.text(), "Urgent work");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct Task {
    id: TaskId,
    status: StatusName,
    text: TaskText,
    position: SourceByteOffset,
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    schedule: TaskSchedule,
}

/// Unique identifier for a Task (UUID v7).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskId(Uuid);

impl TaskId {
    /// Creates a new random `TaskId` (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for TaskId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for TaskId {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<TaskId> for Uuid {
    #[inline]
    fn from(id: TaskId) -> Uuid {
        id.0
    }
}

/// Parsed task attributes captured from checkbox text.
#[derive(Debug, Clone, Default)]
pub struct TaskAttributes {
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    schedule: TaskSchedule,
}

impl TaskAttributes {
    #[inline]
    #[must_use]
    pub fn builder() -> TaskAttributesBuilder {
        TaskAttributesBuilder::default()
    }

    /// Returns the schedule timestamps for the task attributes.
    #[inline]
    #[must_use]
    pub fn schedule(&self) -> &TaskSchedule {
        &self.schedule
    }
}

/// Builder for [`TaskAttributes`].
#[derive(Debug, Default)]
pub struct TaskAttributesBuilder {
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    schedule: TaskSchedule,
}

/// Builder for promoting checkbox list items into tasks.
pub(crate) struct TaskBuilder<'config> {
    config: &'config TaskConfig,
}

impl<'config> TaskBuilder<'config> {
    #[inline]
    pub(crate) const fn new(config: &'config TaskConfig) -> Self {
        Self {
            config,
        }
    }

    pub(crate) fn promote_from_checkbox(
        &self,
        raw_text: &str,
        tags: Vec<Tag>,
        status_symbol: StatusSymbol,
        position: SourceByteOffset,
    ) -> Result<Option<Task>, NoteError> {
        if !self.should_promote_from_tags(&tags) {
            return Ok(None);
        }

        let status = self
            .config
            .status()
            .name_for_symbol(status_symbol)
            .ok_or_else(|| {
                NoteError::Task(TaskError::UnrecognizedStatusSymbol {
                    symbol: status_symbol.value(),
                })
            })?
            .clone();
        let text = self.extract_clean_text(raw_text)?;
        let parsed = self.parse_inline_fields(raw_text)?;
        let attributes = parsed.into_attributes(tags);

        Task::try_new(status, text, position, attributes).map(Some)
    }

    fn should_promote_from_tags(&self, tags: &[Tag]) -> bool {
        self.config.tags().iter().any(|config_tag| {
            tags.iter().any(|tag| {
                config_tag
                    .as_str()
                    .strip_prefix('#')
                    .is_some_and(|raw| raw == tag.full_path())
            })
        })
    }

    fn extract_clean_text(
        &self,
        raw_text: &str,
    ) -> Result<Box<str>, NoteError> {
        let mut text = raw_text.trim();

        let mut stripped = true;
        while stripped {
            stripped = false;
            for tag in self.config.tags() {
                if let Some(rest) = text.strip_prefix(tag.as_str()) {
                    text = rest.trim_start();
                    stripped = true;
                }
            }
        }

        if let Some(prefix) = Self::strip_inline_fields(text) {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }

        Ok(text.into())
    }

    fn parse_inline_fields(
        &self,
        text: &str,
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        Self::for_each_inline_field(text, |keyword, raw_value| {
            state.handle_inline_field(self.config, keyword, raw_value)
        })?;

        state.fill_emoji_dates(self.config, text)?;
        state.fill_default_emoji_dates(text)?;

        Ok(state.finish())
    }

    fn for_each_inline_field(
        text: &str,
        mut f: impl FnMut(&str, &str) -> Result<(), NoteError>,
    ) -> Result<(), NoteError> {
        Self::for_each_inline_field_delim(text, b'[', b']', &mut f)?;
        Self::for_each_inline_field_delim(text, b'(', b')', &mut f)?;
        Ok(())
    }

    fn for_each_inline_field_delim(
        text: &str,
        open_delim: u8,
        close_delim: u8,
        f: &mut impl FnMut(&str, &str) -> Result<(), NoteError>,
    ) -> Result<(), NoteError> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let Some(inner) = text.get(after_open..close) else {
                break;
            };
            if let Some((key, value)) = inner.split_once("::") {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    f(key, value)?;
                }
            }
            cursor = close.saturating_add(1);
        }
        Ok(())
    }

    fn strip_inline_fields(text: &str) -> Option<&str> {
        let bracket = Self::inline_field_start(text, b'[', b']');
        let paren = Self::inline_field_start(text, b'(', b')');
        let start = match (bracket, paren) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }?;
        text.get(..start)
    }

    fn inline_field_start(
        text: &str,
        open_delim: u8,
        close_delim: u8,
    ) -> Option<usize> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let Some(inner) = text.get(after_open..close) else {
                break;
            };
            if let Some((key, value)) = inner.split_once("::")
                && !key.trim().is_empty()
                && !value.trim().is_empty()
            {
                return Some(open);
            }
            cursor = close.saturating_add(1);
        }
        None
    }
}

#[derive(Debug)]
struct ParsedInlineFields {
    slots: TemporalSlots,
    metadata: TaskMetadata,
}

impl ParsedInlineFields {
    fn into_attributes(self, tags: Vec<Tag>) -> TaskAttributes {
        self.slots
            .apply_to_builder(TaskAttributes::builder().tags(tags))
            .metadata(self.metadata)
            .build()
    }
}

#[derive(Debug, Clone, Copy)]
enum DateSlot {
    Created,
    Due,
    Reminder,
    Completed,
}

#[derive(Debug, Default)]
struct TemporalSlots {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
}

impl TemporalSlots {
    fn finish(self, metadata: TaskMetadata) -> ParsedInlineFields {
        ParsedInlineFields {
            slots: self,
            metadata,
        }
    }

    fn get(&self, slot: DateSlot) -> Option<TaskTimestamp> {
        match slot {
            DateSlot::Created => self.created,
            DateSlot::Due => self.due,
            DateSlot::Reminder => self.reminder,
            DateSlot::Completed => self.completed,
        }
    }

    fn set(&mut self, slot: DateSlot, value: TaskTimestamp) {
        match slot {
            DateSlot::Created => self.created = Some(value),
            DateSlot::Due => self.due = Some(value),
            DateSlot::Reminder => self.reminder = Some(value),
            DateSlot::Completed => self.completed = Some(value),
        }
    }

    fn apply_to_builder(
        self,
        builder: TaskAttributesBuilder,
    ) -> TaskAttributesBuilder {
        builder
            .created_at(self.created)
            .due_at(self.due)
            .reminder_at(self.reminder)
            .completed_at(self.completed)
    }
}

#[derive(Debug, Default)]
struct InlineFieldState {
    slots: TemporalSlots,
    metadata: TaskMetadata,
}

impl InlineFieldState {
    fn new() -> Self {
        Self::default()
    }

    fn handle_inline_field(
        &mut self,
        config: &TaskConfig,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some((slot, spec)) = Self::match_date_spec(config, keyword) {
            let parsed = Self::parse_date_str(raw_value, spec)?;
            self.slots.set(slot, parsed);
            return Ok(());
        }

        Self::insert_metadata(config, &mut self.metadata, keyword, raw_value)
    }

    fn fill_emoji_dates(
        &mut self,
        config: &TaskConfig,
        text: &str,
    ) -> Result<(), NoteError> {
        Self::fill_emoji_slot(
            DateSlot::Created,
            config.created(),
            text,
            &mut self.slots,
        )?;
        Self::fill_emoji_slot(
            DateSlot::Due,
            config.due(),
            text,
            &mut self.slots,
        )?;
        Self::fill_emoji_slot(
            DateSlot::Reminder,
            config.reminder(),
            text,
            &mut self.slots,
        )?;
        Self::fill_emoji_slot(
            DateSlot::Completed,
            config.completed(),
            text,
            &mut self.slots,
        )?;

        Ok(())
    }

    fn fill_default_emoji_dates(
        &mut self,
        text: &str,
    ) -> Result<(), NoteError> {
        self.fill_default_emoji_slot(
            DateSlot::Created,
            '\u{2795}',
            "created",
            text,
        )?;
        self.fill_default_emoji_slot(DateSlot::Due, '\u{1f4c5}', "due", text)?;
        self.fill_default_emoji_slot(
            DateSlot::Completed,
            '\u{2705}',
            "completed",
            text,
        )?;

        self.fill_default_emoji_metadata("scheduled", '\u{23f3}', text)?;
        self.fill_default_emoji_metadata("start", '\u{1f6eb}', text)?;
        self.fill_default_emoji_metadata("cancelled", '\u{274c}', text)?;

        Ok(())
    }

    fn finish(self) -> ParsedInlineFields {
        self.slots.finish(self.metadata)
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics keep spec parsing concise."
    )]
    fn parse_metadata_value(
        raw_value: &str,
        spec: &FieldSpec,
    ) -> Result<serde_json::Value, NoteError> {
        match spec {
            FieldSpec::Integer {
                ..
            } => {
                let value = raw_value.parse::<i64>().map_err(|_error| {
                    NoteError::Task(TaskError::InvalidInteger {
                        raw: raw_value.into(),
                        reason: "failed to parse integer",
                    })
                })?;
                Ok(serde_json::Value::Number(value.into()))
            }
            FieldSpec::Float {
                ..
            } => {
                let value = raw_value.parse::<f64>().map_err(|_error| {
                    NoteError::Task(TaskError::InvalidFloat {
                        raw: raw_value.into(),
                        reason: "failed to parse float",
                    })
                })?;
                let number =
                    serde_json::Number::from_f64(value).ok_or_else(|| {
                        NoteError::Task(TaskError::InvalidFloat {
                            raw: raw_value.into(),
                            reason: "float value is not finite",
                        })
                    })?;
                Ok(serde_json::Value::Number(number))
            }
            FieldSpec::Enum {
                ..
            }
            | FieldSpec::String {
                ..
            }
            | FieldSpec::DateTime {
                ..
            } => Ok(serde_json::Value::String(raw_value.into())),
        }
    }

    fn match_date_spec<'config>(
        config: &'config TaskConfig,
        keyword: &str,
    ) -> Option<(DateSlot, &'config DateSpec)> {
        if let Some(spec) = config.created()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Created, spec));
        }
        if let Some(spec) = config.due()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Due, spec));
        }
        if let Some(spec) = config.reminder()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Reminder, spec));
        }
        if let Some(spec) = config.completed()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Completed, spec));
        }
        None
    }

    fn insert_metadata(
        config: &TaskConfig,
        metadata: &mut TaskMetadata,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some(spec) = config.field_spec(keyword) {
            let json_value = Self::parse_metadata_value(raw_value, spec)?;
            spec.validate_raw_value(&json_value).map_err(|_error| {
                NoteError::Task(TaskError::InvalidMetadataField {
                    keyword: keyword.into(),
                    reason: "failed validation",
                })
            })?;
            let field_value =
                FieldValue::try_from_json(&json_value).map_err(|_error| {
                    NoteError::Task(TaskError::InvalidMetadataField {
                        keyword: keyword.into(),
                        reason: "failed conversion",
                    })
                })?;
            let key = TaskFieldKey::try_new(keyword)?;
            metadata.insert(key, field_value);
        } else {
            let key = TaskFieldKey::try_new(keyword)?;
            metadata.insert(key, FieldValue::String(raw_value.into()));
        }

        Ok(())
    }

    fn parse_date_str(
        raw: &str,
        spec: &DateSpec,
    ) -> Result<TaskTimestamp, NoteError> {
        if let Ok(naive) =
            chrono::NaiveDateTime::parse_from_str(raw, spec.format())
        {
            return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
        }

        let date = chrono::NaiveDate::parse_from_str(raw, spec.format())
            .map_err(|_error| {
                NoteError::Task(TaskError::InvalidDate {
                    keyword: spec.keyword().as_str().into(),
                    reason: "failed to parse date string",
                })
            })?;

        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            NoteError::Task(TaskError::InvalidDateTime {
                keyword: spec.keyword().as_str().into(),
            })
        })?;

        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn parse_default_date(
        value: &str,
        keyword: &str,
    ) -> Result<TaskTimestamp, NoteError> {
        let formats = ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"];
        for format in formats {
            if let Ok(naive) =
                chrono::NaiveDateTime::parse_from_str(value, format)
            {
                return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
            }
        }

        let date = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_error| {
                NoteError::Task(TaskError::InvalidDate {
                    keyword: keyword.into(),
                    reason: "failed to parse date string",
                })
            })?;
        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            NoteError::Task(TaskError::InvalidDateTime {
                keyword: keyword.into(),
            })
        })?;
        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn find_emoji_field(text: &str, emoji: char) -> Option<&str> {
        let start = text.find(emoji)?;
        let value_start = start.saturating_add(emoji.len_utf8());
        let tail = text.get(value_start..)?;
        let value = tail.split_whitespace().next()?;
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    fn fill_emoji_slot(
        slot: DateSlot,
        spec: Option<&DateSpec>,
        text: &str,
        slots: &mut TemporalSlots,
    ) -> Result<(), NoteError> {
        let Some(spec) = spec else {
            return Ok(());
        };
        if slots.get(slot).is_some() {
            return Ok(());
        }
        let Some(date) = spec.emoji() else {
            return Ok(());
        };
        let Some(value) = Self::find_emoji_field(text, date) else {
            return Ok(());
        };
        let parsed = Self::parse_date_str(value, spec)?;
        slots.set(slot, parsed);
        Ok(())
    }

    fn fill_default_emoji_slot(
        &mut self,
        slot: DateSlot,
        emoji: char,
        label: &str,
        text: &str,
    ) -> Result<(), NoteError> {
        if self.slots.get(slot).is_some() {
            return Ok(());
        }
        let Some(value) = Self::find_emoji_field(text, emoji) else {
            return Ok(());
        };
        let parsed = Self::parse_default_date(value, label)?;
        self.slots.set(slot, parsed);
        Ok(())
    }

    fn fill_default_emoji_metadata(
        &mut self,
        key: &str,
        emoji: char,
        text: &str,
    ) -> Result<(), NoteError> {
        if self.metadata.get(key).is_some() {
            return Ok(());
        }
        let Some(value) = Self::find_emoji_field(text, emoji) else {
            return Ok(());
        };
        let parsed = Self::parse_default_date(value, key)?;
        let key = TaskFieldKey::try_new(key)?;
        self.metadata.insert(key, FieldValue::Date(parsed.as_i64()));
        Ok(())
    }
}

/// Task schedule timestamps.
#[derive(Debug, Clone, Default, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskSchedule {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
}

impl TaskSchedule {
    #[inline]
    #[must_use]
    pub const fn created(&self) -> Option<TaskTimestamp> {
        self.created
    }

    #[inline]
    #[must_use]
    pub const fn due(&self) -> Option<TaskTimestamp> {
        self.due
    }

    #[inline]
    #[must_use]
    pub const fn reminder(&self) -> Option<TaskTimestamp> {
        self.reminder
    }

    #[inline]
    #[must_use]
    pub const fn completed(&self) -> Option<TaskTimestamp> {
        self.completed
    }
}

/// Validated task text content.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskText(Box<str>);

impl TaskText {
    /// Creates a validated task text value.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the text is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }
        Self::try_from_boxed(value.into())
    }

    /// Creates a validated task text value from a boxed string.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the text is empty.
    #[inline]
    pub fn try_from_boxed(value: Box<str>) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }
        Ok(Self(value))
    }

    /// Returns the underlying text as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Borrowed iterator over task tags.
pub struct TaskTags<'task> {
    inner: std::slice::Iter<'task, Tag>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'task> Iterator for TaskTags<'task> {
    type Item = &'task Tag;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl TaskAttributesBuilder {
    #[inline]
    #[must_use]
    pub fn tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    #[inline]
    #[must_use]
    pub fn metadata(mut self, metadata: TaskMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the task schedule timestamps.
    #[inline]
    #[must_use]
    pub fn schedule(mut self, schedule: TaskSchedule) -> Self {
        self.schedule = schedule;
        self
    }

    #[inline]
    #[must_use]
    pub fn created_at(mut self, created_at: Option<TaskTimestamp>) -> Self {
        self.schedule.created = created_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn due_at(mut self, due_at: Option<TaskTimestamp>) -> Self {
        self.schedule.due = due_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn reminder_at(mut self, reminder_at: Option<TaskTimestamp>) -> Self {
        self.schedule.reminder = reminder_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn completed_at(mut self, completed_at: Option<TaskTimestamp>) -> Self {
        self.schedule.completed = completed_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn build(self) -> TaskAttributes {
        TaskAttributes {
            tags: self.tags,
            metadata: self.metadata,
            schedule: self.schedule,
        }
    }
}

impl Task {
    /// Creates a new [`Task`] from parsed attributes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the task text is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(
        status: StatusName,
        text: T,
        position: SourceByteOffset,
        attributes: TaskAttributes,
    ) -> Result<Self, NoteError> {
        let text = TaskText::try_from_boxed(text.into())?;

        Ok(Self {
            id: TaskId::new(),
            status,
            text,
            position,
            tags: attributes.tags,
            metadata: attributes.metadata,
            schedule: attributes.schedule,
        })
    }

    /// Returns the unique task identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the current task status.
    #[inline]
    #[must_use]
    pub fn status(&self) -> &StatusName {
        &self.status
    }

    /// Returns the task's descriptive text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    /// Returns the byte position of the task in the note source.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Returns the collection of tags associated with this task.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> TaskTags<'_> {
        TaskTags {
            inner: self.tags.iter(),
        }
    }

    /// Returns the task's creation timestamp, if known.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<TaskTimestamp> {
        self.schedule.created
    }

    /// Returns the task's due date, if set.
    #[inline]
    #[must_use]
    pub const fn due_at(&self) -> Option<TaskTimestamp> {
        self.schedule.due
    }

    /// Returns the task's reminder date, if set.
    #[inline]
    #[must_use]
    pub const fn reminder_at(&self) -> Option<TaskTimestamp> {
        self.schedule.reminder
    }

    /// Returns the timestamp when the task was completed, if applicable.
    #[inline]
    #[must_use]
    pub const fn completed_at(&self) -> Option<TaskTimestamp> {
        self.schedule.completed
    }

    /// Returns the task's structured metadata fields.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &TaskMetadata {
        &self.metadata
    }

    /// Returns the schedule timestamps for the task.
    #[inline]
    #[must_use]
    pub fn schedule(&self) -> &TaskSchedule {
        &self.schedule
    }
}

/// A timestamp representing task temporal data.
///
/// Wraps an `i64` Unix timestamp for semantic clarity while maintaining
/// zero-copy compatibility with `rkyv` serialization.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::task::TaskTimestamp;
/// let now = TaskTimestamp::now();
/// assert!(!now.is_future(None));
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskTimestamp(i64);

impl TaskTimestamp {
    /// Creates a new [`TaskTimestamp`] from a Unix timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - Unix timestamp in seconds since epoch.
    #[inline]
    #[must_use]
    pub const fn new(timestamp: i64) -> Self {
        Self(timestamp)
    }

    /// Creates a new [`TaskTimestamp`] representing the current system time.
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(i64::try_from(secs).unwrap_or(i64::MAX))
    }

    /// Returns the raw Unix timestamp value.
    #[inline]
    #[must_use]
    pub const fn as_i64(&self) -> i64 {
        self.0
    }

    /// Returns `true` if this timestamp represents a time in the future.
    ///
    /// # Arguments
    /// * `relative_to` - Optional reference time; defaults to system 'now'.
    #[inline]
    #[must_use]
    pub fn is_future(&self, relative_to: Option<Self>) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        self.0 > reference.0
    }

    /// Returns `true` if this timestamp represents a time in the past.
    ///
    /// # Arguments
    /// * `relative_to` - Optional reference time; defaults to system 'now'.
    #[inline]
    #[must_use]
    pub fn is_past(&self, relative_to: Option<Self>) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        self.0 < reference.0
    }

    /// Returns the duration from now in seconds (positive for future, negative
    /// for past).
    #[inline]
    #[must_use]
    pub fn seconds_from_now(&self) -> i64 {
        self.0.saturating_sub(Self::now().0)
    }

    /// Returns the duration in seconds between this timestamp and another.
    #[inline]
    #[must_use]
    pub const fn duration_from(&self, other: Self) -> i64 {
        self.0.saturating_sub(other.0)
    }

    /// Returns `true` if this timestamp is within the specified duration
    /// window.
    ///
    /// # Arguments
    /// * `duration_seconds` - Duration window in seconds.
    /// * `relative_to` - Optional reference time; defaults to system 'now'.
    #[inline]
    #[must_use]
    pub fn is_within(
        &self,
        duration_seconds: i64,
        relative_to: Option<Self>,
    ) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        let diff = self.0.saturating_sub(reference.0).unsigned_abs();
        diff <= duration_seconds.unsigned_abs()
    }
}

impl Default for TaskTimestamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl From<i64> for TaskTimestamp {
    #[inline]
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}

impl From<TaskTimestamp> for i64 {
    #[inline]
    fn from(timestamp: TaskTimestamp) -> i64 {
        timestamp.0
    }
}

impl From<std::time::SystemTime> for TaskTimestamp {
    #[inline]
    fn from(time: std::time::SystemTime) -> Self {
        let secs = time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(i64::try_from(secs).unwrap_or(i64::MAX))
    }
}

impl From<TaskTimestamp> for std::time::SystemTime {
    #[inline]
    fn from(timestamp: TaskTimestamp) -> Self {
        let secs = u64::try_from(timestamp.0).unwrap_or_default();
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(secs))
            .unwrap_or(std::time::UNIX_EPOCH)
    }
}

/// Task date fields used for date-based queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskDateKind {
    /// Created timestamp.
    Created,
    /// Due timestamp.
    Due,
    /// Reminder timestamp.
    Reminder,
    /// Completed timestamp.
    Completed,
}

/// Validated task priority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaskPriority(f64);

impl TaskPriority {
    /// Creates a validated task priority.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the value is not finite.
    #[inline]
    pub fn try_new(value: f64) -> Result<Self, NoteError> {
        if !value.is_finite() {
            return Err(NoteError::Task(TaskError::InvalidPriority {
                reason: "task priority must be finite",
            }));
        }
        Ok(Self(value))
    }

    #[inline]
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

/// Validated key for task metadata fields.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct TaskFieldKey(Box<str>);

impl TaskFieldKey {
    /// Creates a validated task field key.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the key is empty or contains
    /// non-ASCII alphanumeric characters outside `_` and `-`.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(NoteError::Task(TaskError::FieldKeyEmpty));
        }
        if !text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(NoteError::Task(TaskError::FieldKeyInvalidChars));
        }
        Ok(Self(text.into()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TaskFieldKey {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for TaskFieldKey {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for TaskFieldKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Task metadata fields.
///
/// Stores dynamic key-value pairs extracted from task text using the
/// `[key:: value]` syntax.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{task::TaskMetadata, value::FieldValue};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut meta = TaskMetadata::new();
/// meta.insert_raw("priority", FieldValue::Number(1.0))?;
/// assert_eq!(meta.get_number("priority"), Some(1.0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskMetadata {
    fields: HashMap<TaskFieldKey, FieldValue>,
}

impl TaskMetadata {
    /// Creates a new, empty [`TaskMetadata`] map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Inserts a new metadata field into the collection.
    #[inline]
    pub fn insert(&mut self, field: TaskFieldKey, value: FieldValue) {
        self.fields.insert(field, value);
    }

    /// Inserts a new metadata field by raw key string.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if the key is invalid.
    #[inline]
    pub fn insert_raw(
        &mut self,
        field: &str,
        value: FieldValue,
    ) -> Result<(), NoteError> {
        let key = TaskFieldKey::try_new(field)?;
        self.fields.insert(key, value);
        Ok(())
    }

    /// Returns a reference to the value for the given metadata field.
    #[inline]
    #[must_use]
    pub fn get(&self, field: &str) -> Option<&FieldValue> {
        self.fields.get(field)
    }

    /// Returns the string value for the given field, if it exists and is a
    /// string.
    #[inline]
    #[must_use]
    pub fn get_string(&self, field: &str) -> Option<&str> {
        self.get(field)?.as_str()
    }

    /// Returns the numeric value for the given field, if it exists and is a
    /// number.
    #[inline]
    #[must_use]
    pub fn get_number(&self, field: &str) -> Option<f64> {
        self.get(field)?.as_number()
    }

    /// Returns an iterator over all metadata fields.
    #[inline]
    #[must_use]
    pub fn fields(&self) -> TaskMetadataFields<'_> {
        TaskMetadataFields {
            inner: self.fields.iter(),
        }
    }
}

/// Borrowed iterator over task metadata fields.
pub struct TaskMetadataFields<'meta> {
    inner: std::collections::hash_map::Iter<'meta, TaskFieldKey, FieldValue>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'meta> Iterator for TaskMetadataFields<'meta> {
    type Item = (&'meta TaskFieldKey, &'meta FieldValue);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl Default for TaskMetadata {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::{RawConfig, RawFieldSpec, RawTaskConfig},
            task::StatusSymbol,
            vault::{VaultId, VaultRoot},
        },
        note::{position::SourceByteOffset, tag::scan_tags},
    };

    #[test]
    fn promotes_only_when_task_tag_present() {
        let config = test_config_with_task_tag();
        let builder = TaskBuilder::new(config.task());

        let promoted = builder
            .promote_from_checkbox(
                "#task Do work",
                scan_tags("#task Do work"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(promoted.is_some());

        let skipped = builder
            .promote_from_checkbox(
                "Do work",
                scan_tags("Do work"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(skipped.is_none());

        let skipped_partial = builder
            .promote_from_checkbox(
                "#tasker Do work",
                scan_tags("#tasker Do work"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(skipped_partial.is_none());
    }

    #[test]
    fn promoted_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Review PR [priority:: 2] [project:: lithos]",
                scan_tags("#task Review PR [priority:: 2] [project:: lithos]"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(12),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let config = test_config_with_task_tag();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Fix #work/project/urgent issue",
                scan_tags("#task Fix #work/project/urgent issue"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert!(
            task.tags().any(|tag| tag.full_path() == "work/project/urgent")
        );
        assert_eq!(task.tags().count(), 2);
    }

    #[test]
    fn promoted_checkbox_ignores_invalid_tags() {
        let config = test_config_with_task_tag();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Review #bad/ tags",
                scan_tags("#task Review #bad/ tags"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().count(), 1);
    }

    #[test]
    fn promoted_checkbox_parses_dates() {
        let config = test_config_with_task_tag();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Test task with dates [created:: 2024-01-01] [due:: \
                 2024-12-31]",
                scan_tags(
                    "#task Test task with dates [created:: 2024-01-01] [due:: \
                     2024-12-31]",
                ),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        if let Some(created_at) = task.created_at() {
            assert_eq!(created_at.as_i64(), 1_704_067_200);
            if let Some(due_at) = task.due_at() {
                assert!(created_at.is_past(Some(due_at)));
            }
        }

        if let Some(due_at) = task.due_at() {
            assert_eq!(due_at.as_i64(), 1_735_689_600);
            if let Some(created_at) = task.created_at() {
                assert!(due_at.is_future(Some(created_at)));
            }
        }
    }

    #[test]
    fn promoted_checkbox_parses_paren_inline_fields() {
        let config = config_with_fields();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Review PR (priority:: 2) (project:: lithos)",
                scan_tags("#task Review PR (priority:: 2) (project:: lithos)"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(12),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_parses_default_emoji_dates() {
        let config = test_config_with_task_tag();
        let builder = TaskBuilder::new(config.task());
        let task = builder
            .promote_from_checkbox(
                "#task Do work \u{2795}2024-01-01 \u{1f4c5}2024-12-31 \
                 \u{2705}2025-01-01",
                scan_tags(
                    "#task Do work \u{2795}2024-01-01 \u{1f4c5}2024-12-31 \
                     \u{2705}2025-01-01",
                ),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert_eq!(
            task.created_at().map(|ts| ts.as_i64()),
            Some(1_704_067_200)
        );
        assert_eq!(task.due_at().map(|ts| ts.as_i64()), Some(1_735_603_200));
        assert_eq!(
            task.completed_at().map(|ts| ts.as_i64()),
            Some(1_735_689_600)
        );
    }

    fn test_config_with_task_tag() -> Config {
        let raw = RawConfig {
            task: Some(RawTaskConfig {
                task_tags: Some(vec!["#task".into()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    fn config_with_fields() -> Config {
        let mut fields = HashMap::new();
        fields.insert("priority".into(), RawFieldSpec::Integer {
            min: None,
            max: None,
        });
        fields.insert("project".into(), RawFieldSpec::String {
            pattern: None,
        });

        let raw = RawConfig {
            task: Some(RawTaskConfig {
                enabled: Some(true),
                task_tags: Some(vec!["#task".into()]),
                status: None,
                dates: None,
                fields: Some(fields),
                indexing: None,
                dependencies: None,
                use_emoji: None,
            }),
            ..Default::default()
        };

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }
}
