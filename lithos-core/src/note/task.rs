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
    raw::{RawNote, RawTask},
    tag::Tag,
    value::FieldValue,
};
use crate::config::{
    aggregate::Config,
    task::{StatusName, StatusSymbol},
};

type RawTaskInlineField = (Box<str>, Box<str>);

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

impl Task {
    /// Builds a collection of tasks from raw extraction data.
    ///
    /// # Errors
    /// Returns [`NoteError`] if task promotion or attribute parsing fails.
    pub(crate) fn build_many(
        raw: &RawNote,
        config: &Config,
    ) -> Result<Vec<Task>, NoteError> {
        if raw.tasks().is_empty() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for raw_task in raw.tasks() {
            let ctx = RawTaskContext::new(raw_task, config);
            if let Some(task) = Option::<Task>::try_from(ctx)? {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

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

pub(crate) struct RawTaskContext<'raw> {
    raw: &'raw RawTask,
    config: &'raw Config,
}

impl<'raw> RawTaskContext<'raw> {
    #[inline]
    pub(crate) const fn new(raw: &'raw RawTask, config: &'raw Config) -> Self {
        Self {
            raw,
            config,
        }
    }
}

impl<'raw> TryFrom<RawTaskContext<'raw>> for Option<Task> {
    type Error = NoteError;

    #[inline]
    fn try_from(ctx: RawTaskContext<'raw>) -> Result<Self, Self::Error> {
        let status_symbol =
            StatusSymbol::try_new(ctx.raw.task_kind().marker())?;
        let tags = ctx
            .raw
            .tags()
            .iter()
            .filter_map(|tag| Tag::try_from_token(tag).ok())
            .collect::<Vec<_>>();
        let builder = TaskBuilder::new(ctx.config.task());
        builder.promote_from_raw(ctx.raw, tags, status_symbol)
    }
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

struct TaskBuilder<'config> {
    config: &'config crate::config::task::Task,
}

impl<'config> TaskBuilder<'config> {
    #[inline]
    const fn new(config: &'config crate::config::task::Task) -> Self {
        Self {
            config,
        }
    }

    fn promote_from_raw(
        &self,
        raw: &RawTask,
        tags: Vec<Tag>,
        status_symbol: StatusSymbol,
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
        let text = self.extract_clean_text(raw.text())?;
        let parsed = self.parse_inline_fields(raw.inline_fields())?;
        let attributes = parsed.into_attributes(tags);

        Task::try_new(status, text, raw.position(), attributes).map(Some)
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
        inline_fields: &[RawTaskInlineField],
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        for (keyword, raw_value) in
            inline_fields.iter().map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
        {
            state.handle_any_inline_field(self.config, keyword, raw_value)?;
        }

        Ok(state.finish())
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

    fn handle_any_inline_field(
        &mut self,
        config: &crate::config::task::Task,
        key: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        // 1. Try user-configured emoji mapping
        if let Some((slot, spec)) = Self::match_date_spec_by_emoji(config, key)
        {
            if self.slots.get(slot).is_none() {
                let parsed = Self::parse_date_str(value, spec)?;
                self.slots.set(slot, parsed);
            }
            return Ok(());
        }

        // 2. Try user-configured text mapping
        if let Some((slot, spec)) = Self::match_date_spec(config, key) {
            if self.slots.get(slot).is_none() {
                let parsed = Self::parse_date_str(value, spec)?;
                self.slots.set(slot, parsed);
            }
            return Ok(());
        }

        // 3. Try default emoji mappings
        if self.handle_default_emoji(key, value)? {
            return Ok(());
        }

        // 4. Handle as standard metadata
        Self::insert_metadata(config, &mut self.metadata, key, value)
    }

    fn handle_default_emoji(
        &mut self,
        emoji: &str,
        value: &str,
    ) -> Result<bool, NoteError> {
        match () {
            () if Self::emoji_matches(emoji, '\u{2795}') => {
                self.fill_default_slot_value(
                    DateSlot::Created,
                    "created",
                    value,
                )?;
                Ok(true)
            }
            () if Self::emoji_matches(emoji, '\u{1f4c5}') => {
                self.fill_default_slot_value(DateSlot::Due, "due", value)?;
                Ok(true)
            }
            () if Self::emoji_matches(emoji, '\u{2705}') => {
                self.fill_default_slot_value(
                    DateSlot::Completed,
                    "completed",
                    value,
                )?;
                Ok(true)
            }
            () if Self::emoji_matches(emoji, '\u{23f3}') => {
                self.fill_default_metadata_value("scheduled", value)?;
                Ok(true)
            }
            () if Self::emoji_matches(emoji, '\u{1f6eb}') => {
                self.fill_default_metadata_value("start", value)?;
                Ok(true)
            }
            () if Self::emoji_matches(emoji, '\u{274c}') => {
                self.fill_default_metadata_value("cancelled", value)?;
                Ok(true)
            }
            () => Ok(false),
        }
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
        spec: &crate::config::value::FieldSpec,
    ) -> Result<serde_json::Value, NoteError> {
        match spec {
            crate::config::value::FieldSpec::Integer {
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
            crate::config::value::FieldSpec::Float {
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
            crate::config::value::FieldSpec::Enum {
                ..
            }
            | crate::config::value::FieldSpec::String {
                ..
            }
            | crate::config::value::FieldSpec::DateTime {
                ..
            } => Ok(serde_json::Value::String(raw_value.into())),
        }
    }

    fn match_date_spec<'config>(
        config: &'config crate::config::task::Task,
        keyword: &str,
    ) -> Option<(DateSlot, &'config crate::config::value::DateSpec)> {
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
        config: &crate::config::task::Task,
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
        raw_value: &str,
        spec: &crate::config::value::DateSpec,
    ) -> Result<TaskTimestamp, NoteError> {
        if let Ok(naive) =
            chrono::NaiveDateTime::parse_from_str(raw_value, spec.format())
        {
            return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
        }

        let date = chrono::NaiveDate::parse_from_str(raw_value, spec.format())
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
        raw_value: &str,
        field: &str,
    ) -> Result<TaskTimestamp, NoteError> {
        let formats = ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"];
        for format in formats {
            if let Ok(naive) =
                chrono::NaiveDateTime::parse_from_str(raw_value, format)
            {
                return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
            }
        }

        let date = chrono::NaiveDate::parse_from_str(raw_value, "%Y-%m-%d")
            .map_err(|_error| {
                NoteError::Task(TaskError::InvalidDate {
                    keyword: field.into(),
                    reason: "failed to parse date string",
                })
            })?;
        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            NoteError::Task(TaskError::InvalidDateTime {
                keyword: field.into(),
            })
        })?;
        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn match_date_spec_by_emoji<'config>(
        config: &'config crate::config::task::Task,
        emoji: &str,
    ) -> Option<(DateSlot, &'config crate::config::value::DateSpec)> {
        if let Some(spec) = config.created()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Created, spec));
        }
        if let Some(spec) = config.due()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Due, spec));
        }
        if let Some(spec) = config.reminder()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Reminder, spec));
        }
        if let Some(spec) = config.completed()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Completed, spec));
        }
        None
    }

    fn emoji_matches(token: &str, emoji: char) -> bool {
        let mut chars = token.chars();
        matches!(chars.next(), Some(first) if first == emoji)
            && chars.next().is_none()
    }

    fn fill_default_slot_value(
        &mut self,
        slot: DateSlot,
        label: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        if self.slots.get(slot).is_some() {
            return Ok(());
        }
        let parsed = Self::parse_default_date(value, label)?;
        self.slots.set(slot, parsed);
        Ok(())
    }

    fn fill_default_metadata_value(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        if self.metadata.get(key).is_some() {
            return Ok(());
        }
        let parsed = Self::parse_default_date(value, key)?;
        let key = TaskFieldKey::try_new(key)?;
        self.metadata.insert(key, FieldValue::Date(parsed.as_i64()));
        Ok(())
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
            vault::{VaultId, VaultRoot},
        },
        note::{
            inline_fields::InlineFieldCollection,
            position::SourceByteOffset,
            raw::{RawTag, RawTask, RawTaskKind},
        },
    };

    #[test]
    fn promotes_only_when_task_tag_present() {
        let config = test_config_with_task_tag();

        let promoted = promote_task("#task Do work", &config, &[])
            .expect("task should be promoted");
        assert_eq!(promoted.text(), "Do work");

        let skipped = promote_task("Do work", &config, &[]);
        assert!(skipped.is_none());

        let skipped_partial = promote_task("#tasker Do work", &config, &[]);
        assert!(skipped_partial.is_none());
    }

    #[test]
    fn promoted_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let task = promote_task(
            "#task Review PR [priority:: 2] [project:: lithos]",
            &config,
            &[],
        )
        .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let config = test_config_with_task_tag();
        let task =
            promote_task("#task Fix #work/project/urgent issue", &config, &[])
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
        let task = promote_task("#task Review #bad/ tags", &config, &[])
            .expect("task should be promoted");

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().count(), 1);
    }

    #[test]
    fn promoted_checkbox_parses_dates() {
        let config = test_config_with_task_tag();
        let task = promote_task(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
            &config,
            &[],
        )
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
        let task = promote_task(
            "#task Review PR (priority:: 2) (project:: lithos)",
            &config,
            &[],
        )
        .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_parses_default_emoji_dates() {
        let config = test_config_with_task_tag();
        let emojis = default_emoji_markers();
        let task = promote_task(
            "#task Do work \u{2795}2024-01-01 \u{1f4c5}2024-12-31 \
             \u{2705}2025-01-01",
            &config,
            &emojis,
        )
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

    fn promote_task(
        text: &str,
        config: &Config,
        emoji_markers: &[char],
    ) -> Option<Task> {
        let raw = raw_task_from_text(text, emoji_markers);
        let ctx = RawTaskContext::new(&raw, config);
        Option::<Task>::try_from(ctx).expect("task conversion")
    }

    fn raw_task_from_text(text: &str, emoji_markers: &[char]) -> RawTask {
        let base = SourceByteOffset::new(0);
        let tags = scan_raw_tags(text, base)
            .expect("raw tags")
            .into_iter()
            .map(|tag| tag.value().into())
            .collect();
        let tokens = InlineFieldCollection::parse(text, emoji_markers);
        RawTask::new(
            RawTaskKind::Unchecked(' '),
            text.into(),
            tags,
            tokens.inline_fields().to_vec(),
            base,
        )
    }

    fn scan_raw_tags(
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<RawTag>, NoteError> {
        let mut tags = Vec::new();
        let mut chars = text.char_indices().peekable();
        let mut prev_is_alnum = false;
        let base =
            usize::try_from(u32::from(base_offset)).map_err(|_error| {
                NoteError::Structure("tag offset out of range")
            })?;

        while let Some((start_idx, ch)) = chars.next() {
            if ch != '#' || prev_is_alnum {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            }

            let Some(mut end_idx) = start_idx.checked_add(ch.len_utf8()) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };
            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if !(next_ch.is_alphanumeric()
                    || matches!(next_ch, '_' | '-' | '/'))
                {
                    break;
                }
                chars.next();
                let Some(updated) = next_idx.checked_add(next_ch.len_utf8())
                else {
                    break;
                };
                end_idx = updated;
            }

            let Some(raw) = text.get(start_idx..end_idx) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };

            if raw.len() > 1 {
                let offset = base.saturating_add(start_idx);
                let position = SourceByteOffset::try_from_usize(offset)?;
                tags.push(RawTag::new(raw.into(), position));
            }

            prev_is_alnum =
                raw.chars().last().is_some_and(char::is_alphanumeric);
        }

        Ok(tags)
    }

    fn default_emoji_markers() -> Vec<char> {
        vec![
            '\u{2795}',
            '\u{1f4c5}',
            '\u{2705}',
            '\u{23f3}',
            '\u{1f6eb}',
            '\u{274c}',
        ]
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
