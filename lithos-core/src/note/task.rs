//! Task sub-entity and temporal management.
//!
//! Defines the [`crate::note::task::Task`] entity and its specialized
//! components, including semantic timestamp handling and metadata extraction.

//! Task subentity for Note aggregate.
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
    tag::Tag,
    types::SourceByteOffset,
    value::FieldValue,
};
use crate::config::task::StatusName;

/// Task entity within a Note.
///
/// Represents a promoted checkbox item from a markdown list. A `Task` carries
/// additional domain semantics such as status, temporal data (due dates, etc.),
/// and rich metadata extracted from inline tags and brackets.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{task::Task, types::SourceByteOffset};
/// # use lithos_core::config::task::StatusName;
/// # use lithos_core::note::task::TaskAttributes;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = StatusName::try_new("todo")?;
/// let task = Task::new(
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
pub struct Task {
    id: TaskId,
    status: StatusName,
    text: Box<str>,
    position: SourceByteOffset,
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    schedule: TaskSchedule,
}

/// Validated task priority.
#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize,
)]
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
                reason: "task priority must be finite".into(),
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
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
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

/// Task schedule timestamps.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskSchedule {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
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
    pub fn new<T: Into<Box<str>>>(
        status: StatusName,
        text: T,
        position: SourceByteOffset,
        attributes: TaskAttributes,
    ) -> Result<Self, NoteError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }

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
        &self.text
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
    pub fn tags(&self) -> &[Tag] {
        &self.tags
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
    serde::Serialize,
    serde::Deserialize,
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
    serde::Serialize,
    serde::Deserialize,
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
        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
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
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub fn seconds_from_now(&self) -> i64 {
        self.0 - Self::now().0
    }

    /// Returns the duration in seconds between this timestamp and another.
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub const fn duration_from(&self, other: Self) -> i64 {
        self.0 - other.0
    }

    /// Returns `true` if this timestamp is within the specified duration
    /// window.
    ///
    /// # Arguments
    /// * `duration_seconds` - Duration window in seconds.
    /// * `relative_to` - Optional reference time; defaults to system 'now'.
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Timestamp arithmetic is safe"
    )]
    pub fn is_within(
        &self,
        duration_seconds: i64,
        relative_to: Option<Self>,
    ) -> bool {
        let reference = relative_to.unwrap_or_else(Self::now);
        let diff = (self.0 - reference.0).abs();
        diff <= duration_seconds
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
        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "Unix timestamp fits in i64 for Lithos time range"
        )]
        Self(
            time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
    }
}

impl From<TaskTimestamp> for std::time::SystemTime {
    #[inline]
    fn from(timestamp: TaskTimestamp) -> Self {
        #[expect(
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "Timestamp is non-negative for Duration conversion"
        )]
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(timestamp.0 as u64))
            .unwrap_or(std::time::UNIX_EPOCH)
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

    /// Returns a reference to the internal metadata field map.
    #[inline]
    #[must_use]
    pub const fn fields(&self) -> &HashMap<TaskFieldKey, FieldValue> {
        &self.fields
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
    use super::*;
    use crate::{
        config::{
            raw::{RawFieldSpec, RawTaskConfig},
            task::{StatusSymbol, Task as TaskConfig},
        },
        note::adapter::task_parser::TaskParser,
    };

    fn config_with_fields() -> TaskConfig {
        let mut fields = HashMap::new();
        fields.insert("priority".into(), RawFieldSpec::Integer {
            min: None,
            max: None,
        });
        fields.insert("project".into(), RawFieldSpec::String {
            pattern: None,
        });

        let raw = RawTaskConfig {
            enabled: Some(true),
            task_tags: Some(vec!["#task".into()]),
            status: None,
            dates: None,
            fields: Some(fields),
            indexing: None,
        };

        TaskConfig::from_raw(raw).expect("valid task config")
    }

    #[test]
    fn should_promote_requires_task_tag() {
        let config = TaskConfig::default();
        let parser = TaskParser::new(&config);
        let promoted_tags =
            crate::note::adapter::tag_scanner::TagScanner::new("#task Do work")
                .collect_tags();
        let promoted = parser
            .parse_promoted_checkbox_with_tags(
                "#task Do work",
                promoted_tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(promoted.is_some());

        let skipped_tags =
            crate::note::adapter::tag_scanner::TagScanner::new("Do work")
                .collect_tags();
        let skipped = parser
            .parse_promoted_checkbox_with_tags(
                "Do work",
                skipped_tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(skipped.is_none());

        let skipped_partial_tags =
            crate::note::adapter::tag_scanner::TagScanner::new(
                "#tasker Do work",
            )
            .collect_tags();
        let skipped_partial = parser
            .parse_promoted_checkbox_with_tags(
                "#tasker Do work",
                skipped_partial_tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(skipped_partial.is_none());
    }

    #[test]
    fn from_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let parser = TaskParser::new(&config);
        let tags = crate::note::adapter::tag_scanner::TagScanner::new(
            "#task Review PR [priority:: 2] [project:: lithos]",
        )
        .collect_tags();
        let task = parser
            .parse_promoted_checkbox_with_tags(
                "#task Review PR [priority:: 2] [project:: lithos]",
                tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(12),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));

        // Test temporal fields are accessible
        assert!(task.created_at().is_none() || task.created_at().is_some());
        assert!(task.due_at().is_none() || task.due_at().is_some());
        assert!(task.reminder_at().is_none() || task.reminder_at().is_some());
        assert!(task.completed_at().is_none() || task.completed_at().is_some());
    }

    #[test]
    fn from_checkbox_collects_hierarchical_tags() {
        let config = TaskConfig::default();
        let parser = TaskParser::new(&config);
        let tags = crate::note::adapter::tag_scanner::TagScanner::new(
            "#task Fix #work/project/urgent issue",
        )
        .collect_tags();
        let task = parser
            .parse_promoted_checkbox_with_tags(
                "#task Fix #work/project/urgent issue",
                tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        // Verify hierarchical tags are properly extracted
        assert!(task.tags().iter().any(|tag| tag.full_path() == "task"));
        assert!(
            task.tags()
                .iter()
                .any(|tag| tag.full_path() == "work/project/urgent")
        );
        assert_eq!(task.tags().len(), 2);
    }

    #[test]
    fn from_checkbox_ignores_invalid_tags() {
        let config = TaskConfig::default();
        let parser = TaskParser::new(&config);
        let tags = crate::note::adapter::tag_scanner::TagScanner::new(
            "#task Review #bad/ tags",
        )
        .collect_tags();
        let task = parser
            .parse_promoted_checkbox_with_tags(
                "#task Review #bad/ tags",
                tags,
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("task should parse")
            .expect("task should be promoted");

        assert!(task.tags().iter().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().len(), 1);
    }

    #[test]
    fn task_timestamp_provides_semantic_methods() {
        let config = TaskConfig::default();
        let parser = TaskParser::new(&config);
        let tags = crate::note::adapter::tag_scanner::TagScanner::new(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
        )
        .collect_tags();
        let task = parser
            .parse_promoted_checkbox_with_tags(
                "#task Test task with dates [created:: 2024-01-01] [due:: \
                 2024-12-31]",
                tags,
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
}
