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
    position::{SourceByteOffset, SourceByteRange},
    raw::{RawTaskFields, RawTaskPayload},
    tag::Tag,
    value::FieldValue,
};
use crate::config::task::TaskConfigSpec;

// --- Deleted RawTaskInlineField alias ---

/// Task entity within a Note.
///
/// Represents a promoted checkbox item from a markdown list. A `Task` carries
/// additional domain semantics such as status, temporal data (due dates, etc.),
/// and rich metadata extracted from inline tags and brackets.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{task::Task, position::{SourceByteOffset, SourceByteRange}};
/// # use lithos_core::note::task::{TaskAttributes, TaskText};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = "todo";
/// let range = SourceByteRange::new(SourceByteOffset::new(0), SourceByteOffset::new(10))?;
/// let text = TaskText::try_new("Urgent work".into(), "Urgent work".into())?;
/// let task = Task::try_new(
///     status.into(),
///     text,
///     range,
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
    status: Box<str>,
    text: TaskText,
    range: SourceByteRange,
    tags: Box<[Tag]>,
    metadata: TaskMetadata,
    dates: TaskDates,
}

impl Task {
    /// Creates a new [`Task`] from parsed attributes.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if the task text is empty.
    #[inline]
    pub fn try_new(
        status: Box<str>,
        text: TaskText,
        range: SourceByteRange,
        attributes: TaskAttributes,
    ) -> Result<Self, TaskError> {
        Ok(Self {
            id: TaskId::new(),
            status,
            text,
            range,
            tags: attributes.tags,
            metadata: attributes.metadata,
            dates: attributes.dates,
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
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Returns the task's descriptive text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.clean()
    }

    /// Returns the raw task text (including inline fields/tags).
    #[inline]
    #[must_use]
    pub fn text_full(&self) -> &str {
        self.text.raw()
    }

    /// Returns the byte range of the task in the note source.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the start byte position of the task in the note source.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.range.start()
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
        self.dates.created
    }

    /// Returns the task's due date, if set.
    #[inline]
    #[must_use]
    pub const fn due_at(&self) -> Option<TaskTimestamp> {
        self.dates.due
    }

    /// Returns the task's reminder date, if set.
    #[inline]
    #[must_use]
    pub const fn reminder_at(&self) -> Option<TaskTimestamp> {
        self.dates.reminder
    }

    /// Returns the timestamp when the task was completed, if applicable.
    #[inline]
    #[must_use]
    pub const fn completed_at(&self) -> Option<TaskTimestamp> {
        self.dates.completed
    }

    /// Returns the task's structured metadata fields.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &TaskMetadata {
        &self.metadata
    }

    /// Returns the date fields for the task.
    #[inline]
    #[must_use]
    pub fn dates(&self) -> &TaskDates {
        &self.dates
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

/// Reference to a task derived from its source range.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskRef(SourceByteRange);

impl TaskRef {
    /// Creates a new task reference from a source range.
    #[inline]
    #[must_use]
    pub const fn new(range: SourceByteRange) -> Self {
        Self(range)
    }

    /// Returns the source range for this task reference.
    #[inline]
    #[must_use]
    pub const fn range(self) -> SourceByteRange {
        self.0
    }
}

/// Parsed task attributes captured from checkbox text.
#[derive(Debug, Clone, Default)]
pub struct TaskAttributes {
    tags: Box<[Tag]>,
    metadata: TaskMetadata,
    dates: TaskDates,
}

impl TaskAttributes {
    #[inline]
    #[must_use]
    pub fn builder() -> TaskAttributesBuilder {
        TaskAttributesBuilder::default()
    }

    /// Returns the date fields for the task attributes.
    #[inline]
    #[must_use]
    pub fn dates(&self) -> &TaskDates {
        &self.dates
    }
}

/// Builder for [`TaskAttributes`].
#[derive(Debug, Default)]
pub struct TaskAttributesBuilder {
    tags: Box<[Tag]>,
    metadata: TaskMetadata,
    dates: TaskDates,
}

impl TaskAttributesBuilder {
    #[inline]
    #[must_use]
    pub fn tags<T: Into<Box<[Tag]>>>(mut self, tags: T) -> Self {
        self.tags = tags.into();
        self
    }

    #[inline]
    #[must_use]
    pub fn metadata(mut self, metadata: TaskMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the task date fields.
    #[inline]
    #[must_use]
    pub fn dates(mut self, dates: TaskDates) -> Self {
        self.dates = dates;
        self
    }

    #[inline]
    #[must_use]
    pub fn created_at(mut self, created_at: Option<TaskTimestamp>) -> Self {
        self.dates.created = created_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn due_at(mut self, due_at: Option<TaskTimestamp>) -> Self {
        self.dates.due = due_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn reminder_at(mut self, reminder_at: Option<TaskTimestamp>) -> Self {
        self.dates.reminder = reminder_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn completed_at(mut self, completed_at: Option<TaskTimestamp>) -> Self {
        self.dates.completed = completed_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn start_at(mut self, start_at: Option<TaskTimestamp>) -> Self {
        self.dates.start = start_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn scheduled_at(mut self, scheduled_at: Option<TaskTimestamp>) -> Self {
        self.dates.scheduled = scheduled_at;
        self
    }

    #[inline]
    #[must_use]
    pub fn build(self) -> TaskAttributes {
        TaskAttributes {
            tags: self.tags,
            metadata: self.metadata,
            dates: self.dates,
        }
    }
}

/// Task date fields.
#[derive(Debug, Clone, Default, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskDates {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
    start: Option<TaskTimestamp>,
    scheduled: Option<TaskTimestamp>,
}

impl TaskDates {
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

    #[inline]
    #[must_use]
    pub const fn start(&self) -> Option<TaskTimestamp> {
        self.start
    }

    #[inline]
    #[must_use]
    pub const fn scheduled(&self) -> Option<TaskTimestamp> {
        self.scheduled
    }
}

/// Task text with raw and cleaned variants.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskText {
    raw: Box<str>,
    clean: Box<str>,
}

impl TaskText {
    /// Creates a validated task text value.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError::EmptyText`] if the cleaned text is empty.
    #[inline]
    pub fn try_new(raw: Box<str>, clean: Box<str>) -> Result<Self, TaskError> {
        if clean.trim().is_empty() {
            return Err(TaskError::EmptyText);
        }
        Ok(Self {
            raw,
            clean,
        })
    }

    /// Returns the raw task text.
    #[inline]
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the cleaned task text.
    #[inline]
    #[must_use]
    pub fn clean(&self) -> &str {
        &self.clean
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
    /// Returns [`TaskError::InvalidPriority`] if the value is not finite.
    #[inline]
    pub fn try_new(value: f64) -> Result<Self, TaskError> {
        if !value.is_finite() {
            return Err(TaskError::InvalidPriority {
                value,
            });
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
    /// Returns [`TaskError`] if the key is empty or contains
    /// non-ASCII alphanumeric characters outside `_` and `-`.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, TaskError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(TaskError::EmptyText);
        }
        if !text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(TaskError::InvalidMetadataField {
                key: text.into(),
                reason: "must be ASCII alphanumeric, '_' or '-'",
            });
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
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
    /// Returns [`TaskError`] if the key is invalid.
    #[inline]
    pub fn insert_raw(
        &mut self,
        field: &str,
        value: FieldValue,
    ) -> Result<(), TaskError> {
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

pub(crate) struct TaskBuilder<'spec> {
    spec: &'spec TaskConfigSpec,
}

impl<'spec> TaskBuilder<'spec> {
    #[inline]
    pub(crate) const fn new(spec: &'spec TaskConfigSpec) -> Self {
        Self {
            spec,
        }
    }

    pub fn promote_from_payload(
        &self,
        raw: &RawTaskPayload<'_>,
    ) -> Result<Task, NoteError> {
        let mut tags = Vec::with_capacity(raw.tags.len());
        for raw_tag in &raw.tags {
            if let Ok(tag) = Tag::try_from(raw_tag.value.as_ref()) {
                tags.push(tag);
            }
        }
        let tags = tags.into_boxed_slice();

        let symbol = raw.task_marker.marker();
        let status = self
            .spec
            .status_mappings
            .get(&symbol)
            .ok_or(TaskError::UnrecognizedStatus {
                symbol,
            })?
            .clone();

        let clean = self.extract_clean_text(raw.text_full.as_ref())?;
        let parsed = self.parse_task_fields(&raw.fields)?;
        let attributes = parsed.into_attributes(tags);

        let text = TaskText::try_new(raw.text_full.as_ref().into(), clean)?;
        Task::try_new(status, text, raw.range, attributes).map_err(Into::into)
    }

    fn extract_clean_text(
        &self,
        raw_text: &str,
    ) -> Result<Box<str>, NoteError> {
        let mut text = raw_text.trim();

        let mut stripped = true;
        while stripped {
            stripped = false;
            for tag in self.spec.promotion_tags.as_ref() {
                if let Some(rest) = text.strip_prefix(tag.as_ref()) {
                    text = rest.trim_start();
                    stripped = true;
                }
                // Also check with # prefix for fidelity if needed, but spec
                // contains raw tag names.
                if let Some(rest) = text
                    .strip_prefix('#')
                    .and_then(|s| s.strip_prefix(tag.as_ref()))
                {
                    text = rest.trim_start();
                    stripped = true;
                }
            }
        }

        if let Some(prefix) = Self::strip_inline_fields(text) {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(TaskError::EmptyText.into());
        }

        Ok(text.into())
    }

    fn parse_task_fields(
        &self,
        fields: &RawTaskFields<'_>,
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        for field in &fields.fields {
            state.handle_any_inline_field(
                self.spec,
                field.key.as_ref(),
                field.value.as_ref(),
            )?;
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
    fn into_attributes(self, tags: Box<[Tag]>) -> TaskAttributes {
        self.slots
            .apply_to_builder(TaskAttributes::builder().tags(tags))
            .metadata(self.metadata)
            .build()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateSlot {
    Created,
    Due,
    Reminder,
    Completed,
    Start,
    Scheduled,
}

#[derive(Debug, Default)]
struct TemporalSlots {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
    start: Option<TaskTimestamp>,
    scheduled: Option<TaskTimestamp>,
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
            DateSlot::Start => self.start,
            DateSlot::Scheduled => self.scheduled,
        }
    }

    fn set(&mut self, slot: DateSlot, value: TaskTimestamp) {
        match slot {
            DateSlot::Created => self.created = Some(value),
            DateSlot::Due => self.due = Some(value),
            DateSlot::Reminder => self.reminder = Some(value),
            DateSlot::Completed => self.completed = Some(value),
            DateSlot::Start => self.start = Some(value),
            DateSlot::Scheduled => self.scheduled = Some(value),
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
            .start_at(self.start)
            .scheduled_at(self.scheduled)
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
        spec: &TaskConfigSpec,
        key: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        // 1. Try user-configured mapping
        if let Some((slot, format, _emoji)) = Self::match_date_spec(spec, key) {
            if self.slots.get(slot).is_none() {
                let parsed = Self::parse_date_str(value, key, format)?;
                self.slots.set(slot, parsed);
            }
            return Ok(());
        }

        // 2. Try configured emoji mappings
        if spec.use_emoji
            && let Some((slot, format, keyword)) =
                Self::match_date_spec_by_emoji(spec, key)
        {
            if self.slots.get(slot).is_none() {
                let parsed = Self::parse_date_str(value, keyword, format)?;
                self.slots.set(slot, parsed);
            }
            return Ok(());
        }

        // 3. Handle as standard metadata
        Self::insert_metadata(spec, &mut self.metadata, key, value)
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
                    TaskError::InvalidDate {
                        keyword: "integer".into(),
                        raw: raw_value.into(),
                        reason: "failed to parse integer",
                    }
                })?;
                Ok(serde_json::Value::Number(value.into()))
            }
            crate::config::value::FieldSpec::Float {
                ..
            } => {
                let value = raw_value.parse::<f64>().map_err(|_error| {
                    TaskError::InvalidPriority {
                        value: f64::NAN,
                    }
                })?;
                let number = serde_json::Number::from_f64(value).ok_or(
                    TaskError::InvalidPriority {
                        value,
                    },
                )?;

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

    #[expect(
        clippy::type_complexity,
        reason = "Match ergonomics are preferred for domain facts"
    )]
    fn match_date_spec<'spec>(
        spec: &'spec TaskConfigSpec,
        keyword: &str,
    ) -> Option<(DateSlot, &'spec str, Option<char>)> {
        use crate::config::task::TemporalSlot;

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics are preferred for mapping lookups"
        )]
        let (slot_enum, format, emoji) = spec.temporal_specs.get(keyword)?;
        let slot = match *slot_enum {
            TemporalSlot::Created => DateSlot::Created,
            TemporalSlot::Due => DateSlot::Due,
            TemporalSlot::Reminder => DateSlot::Reminder,
            TemporalSlot::Completed => DateSlot::Completed,
            TemporalSlot::Start => DateSlot::Start,
            TemporalSlot::Scheduled => DateSlot::Scheduled,
        };
        Some((slot, format.as_str(), *emoji))
    }

    fn match_date_spec_by_emoji<'spec>(
        spec: &'spec TaskConfigSpec,
        emoji: &str,
    ) -> Option<(DateSlot, &'spec str, &'spec str)> {
        use crate::config::task::TemporalSlot;

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Borrowed tuple destructuring keeps lookups concise"
        )]
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Order is irrelevant for emoji lookup"
        )]
        for (keyword, value) in &spec.temporal_specs {
            let (slot_enum, format, maybe_emoji) = value;
            let Some(spec_emoji) = *maybe_emoji else {
                continue;
            };
            if !Self::emoji_matches(emoji, spec_emoji) {
                continue;
            }
            let slot = match *slot_enum {
                TemporalSlot::Created => DateSlot::Created,
                TemporalSlot::Due => DateSlot::Due,
                TemporalSlot::Reminder => DateSlot::Reminder,
                TemporalSlot::Completed => DateSlot::Completed,
                TemporalSlot::Start => DateSlot::Start,
                TemporalSlot::Scheduled => DateSlot::Scheduled,
            };
            return Some((slot, format.as_str(), keyword.as_ref()));
        }
        None
    }

    fn insert_metadata(
        spec: &TaskConfigSpec,
        metadata: &mut TaskMetadata,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some(field_spec) = spec.field_specs.get(keyword) {
            let json_value = Self::parse_metadata_value(raw_value, field_spec)?;
            field_spec.validate_raw_value(&json_value).map_err(|_error| {
                TaskError::InvalidMetadataField {
                    key: keyword.into(),
                    reason: "failed validation",
                }
            })?;
            let field_value = serde_json::from_value::<FieldValue>(json_value)
                .map_err(|_error| TaskError::InvalidMetadataField {
                    key: keyword.into(),
                    reason: "failed conversion",
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
        keyword: &str,
        format: &str,
    ) -> Result<TaskTimestamp, NoteError> {
        if let Ok(naive) =
            chrono::NaiveDateTime::parse_from_str(raw_value, format)
        {
            return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
        }

        let date = chrono::NaiveDate::parse_from_str(raw_value, format)
            .map_err(|_error| TaskError::InvalidDate {
                keyword: keyword.into(),
                raw: raw_value.into(),
                reason: "failed to parse date string",
            })?;
        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            TaskError::InvalidDate {
                keyword: keyword.into(),
                raw: raw_value.into(),
                reason: "invalid time",
            }
        })?;

        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn emoji_matches(token: &str, emoji: char) -> bool {
        let mut chars = token.chars();
        matches!(chars.next(), Some(first) if first == emoji)
            && chars.next().is_none()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        config::task::{TaskConfigSpec, TemporalSlot},
        note::{
            raw::{
                RawInlineField, RawTag, RawTaskFields, RawTaskMarker,
                RawTaskPayload,
            },
            scanner::{NoteScanner, ScannedArtifact},
        },
    };

    fn task_spec_fixture() -> TaskConfigSpec {
        let mut temporal_specs = HashMap::new();
        temporal_specs.insert(
            "due".into(),
            (TemporalSlot::Due, "%Y-%m-%d".to_owned(), Some('\u{1f4c5}')),
        );
        temporal_specs.insert(
            "created".into(),
            (TemporalSlot::Created, "%Y-%m-%d".to_owned(), Some('\u{2795}')),
        );
        temporal_specs.insert(
            "completed".into(),
            (TemporalSlot::Completed, "%Y-%m-%d".to_owned(), Some('\u{2705}')),
        );

        TaskConfigSpec {
            enabled: true,
            use_emoji: true,
            emoji_markers: vec!['\u{1f4c5}', '\u{2795}', '\u{2705}'].into(),
            promotion_tags: vec!["task".into()].into(),
            status_mappings: [(' ', "todo".into()), ('x', "done".into())]
                .into_iter()
                .collect(),
            temporal_specs,
            field_specs: HashMap::new(),
        }
    }

    #[test]
    fn promoted_task_strips_promotion_tags() {
        let spec = task_spec_fixture();

        let promoted = promote_task("#task Do work", &spec, &[]);
        assert_eq!(promoted.text(), "Do work");

        let untagged = promote_task("Do work", &spec, &[]);
        assert_eq!(untagged.text(), "Do work");
    }

    #[test]
    fn promoted_checkbox_extracts_text_and_metadata() {
        let mut spec = task_spec_fixture();
        spec.field_specs.insert(
            "priority".into(),
            crate::config::value::FieldSpec::Integer {
                name: crate::config::value::FieldName::try_new("priority")
                    .unwrap(),
                bounds: crate::bounds::Bounds::Unbounded,
            },
        );
        spec.field_specs.insert(
            "project".into(),
            crate::config::value::FieldSpec::String {
                name: crate::config::value::FieldName::try_new("project")
                    .unwrap(),
                pattern: None,
                compiled: None,
            },
        );

        let task = promote_task(
            "#task Review PR [priority:: 2] [project:: lithos]",
            &spec,
            &[],
        );

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let spec = task_spec_fixture();
        let task =
            promote_task("#task Fix #work/project/urgent issue", &spec, &[]);

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert!(
            task.tags().any(|tag| tag.full_path() == "work/project/urgent")
        );
        assert_eq!(task.tags().count(), 2);
    }

    #[test]
    fn promoted_checkbox_ignores_invalid_tags() {
        let spec = task_spec_fixture();
        let task = promote_task("#task Review #bad/ tags", &spec, &[]);

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().count(), 1);
    }

    #[test]
    fn promoted_checkbox_parses_dates() {
        let spec = task_spec_fixture();
        let task = promote_task(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
            &spec,
            &[],
        );

        if let Some(created_at) = task.created_at() {
            assert_eq!(created_at.as_i64(), 1_704_067_200);
            if let Some(due_at) = task.due_at() {
                assert!(created_at.is_past(Some(due_at)));
            }
        }

        if let Some(due_at) = task.due_at() {
            assert_eq!(due_at.as_i64(), 1_735_603_200);
            if let Some(created_at) = task.created_at() {
                assert!(due_at.is_future(Some(created_at)));
            }
        }
    }

    #[test]
    fn promoted_checkbox_parses_paren_inline_fields() {
        let mut spec = task_spec_fixture();
        spec.field_specs.insert(
            "priority".into(),
            crate::config::value::FieldSpec::Integer {
                name: crate::config::value::FieldName::try_new("priority")
                    .unwrap(),
                bounds: crate::bounds::Bounds::Unbounded,
            },
        );
        spec.field_specs.insert(
            "project".into(),
            crate::config::value::FieldSpec::String {
                name: crate::config::value::FieldName::try_new("project")
                    .unwrap(),
                pattern: None,
                compiled: None,
            },
        );

        let task = promote_task(
            "#task Review PR (priority:: 2) (project:: lithos)",
            &spec,
            &[],
        );

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_parses_default_emoji_dates() {
        let spec = task_spec_fixture();
        let emojis = default_emoji_markers();
        let task = promote_task(
            "#task Do work \u{2795}2024-01-01 \u{1f4c5}2024-12-31 \
             \u{2705}2025-01-01",
            &spec,
            &emojis,
        );

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
        promoted_text: &str,
        spec: &TaskConfigSpec,
        emoji_markers: &[char],
    ) -> Task {
        let raw = raw_task_payload_from_text(promoted_text, emoji_markers);
        TaskBuilder::new(spec)
            .promote_from_payload(&raw)
            .expect("task conversion")
    }

    fn raw_task_payload_from_text<'source>(
        raw_text: &'source str,
        emoji_markers: &[char],
    ) -> RawTaskPayload<'source> {
        let scanner = NoteScanner::new(emoji_markers.to_vec());
        let start = SourceByteOffset::new(0);
        let end = SourceByteOffset::try_from(raw_text.len()).unwrap_or(start);
        let range = SourceByteRange::new(start, end).expect("valid test range");

        let artifacts = scanner
            .scan_block(raw_text, SourceByteOffset::new(0))
            .expect("scan artifacts");

        let mut tags = Vec::new();
        let mut inline_fields = Vec::new();

        for artifact in &artifacts {
            match *artifact {
                ScannedArtifact::Tag {
                    text: tag_text,
                    position,
                } => tags.push(RawTag::new(tag_text.into(), position)),
                ScannedArtifact::InlineField {
                    key,
                    value,
                    position,
                } => inline_fields.push(RawInlineField::new(
                    key.into(),
                    value.into(),
                    position,
                )),
                ScannedArtifact::BlockRef {
                    ..
                }
                | ScannedArtifact::TaskMarker {
                    ..
                } => {}
            }
        }

        RawTaskPayload::new(
            RawTaskFields::new(inline_fields),
            raw_text.into(),
            RawTaskMarker::Unchecked(' '),
            tags,
            range,
        )
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
}
