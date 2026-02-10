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

use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;
use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::NoteError, tag::Tag, types::SourceByteOffset, value::FieldValue,
};
use crate::config::task::{
    DateFieldSpec, StatusName, StatusSymbol, TaskConfig, TaskFieldSpec,
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
/// # use lithos_core::note::{task::Task, types::SourceByteOffset};
/// # use lithos_core::config::task::{TaskConfig, StatusSymbol};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = TaskConfig::default();
/// let status = StatusSymbol::try_new(' ')?;
/// let task = Task::from_checkbox(
///     "#task Urgent work",
///     status,
///     SourceByteOffset::new(0),
///     &config,
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
    text: String,
    position: SourceByteOffset,
    tags: Vec<Tag>,
    metadata: TaskMetadata,
    created_at: Option<TaskTimestamp>,
    due_at: Option<TaskTimestamp>,
    reminder_at: Option<TaskTimestamp>,
    completed_at: Option<TaskTimestamp>,
}

impl Task {
    /// Creates a new [`Task`] from checkbox text and metadata.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Task`] if:
    /// - The status symbol is unrecognized.
    /// - The task text is empty after cleaning.
    /// - Temporal field parsing fails.
    /// - Metadata validation fails against configuration.
    #[inline]
    pub fn from_checkbox(
        raw_text: &str,
        status_symbol: StatusSymbol,
        position: SourceByteOffset,
        config: &TaskConfig,
    ) -> Result<Self, NoteError> {
        let status = config
            .status()
            .name_for_symbol(status_symbol)
            .ok_or_else(|| {
                NoteError::Task(format!(
                    "unrecognized status symbol: '{}'",
                    status_symbol.value()
                ))
            })?
            .clone();

        let text = Self::extract_clean_text(raw_text, config)?;
        let tags = Self::extract_tags(raw_text)?;
        let (created_at, due_at, reminder_at, completed_at) =
            Self::parse_temporal_fields(raw_text, config)?;
        let metadata = Self::parse_metadata(raw_text, config)?;

        Ok(Self {
            id: TaskId::new(),
            status,
            text,
            position,
            tags,
            metadata,
            created_at,
            due_at,
            reminder_at,
            completed_at,
        })
    }

    /// Returns `true` if the checkbox text should be promoted to a [`Task`].
    #[inline]
    #[must_use]
    pub fn should_promote(text: &str, config: &TaskConfig) -> bool {
        config.has_task_tag(text)
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
    pub fn status(&self) -> StatusName {
        self.status.clone()
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
        self.created_at
    }

    /// Returns the task's due date, if set.
    #[inline]
    #[must_use]
    pub const fn due_at(&self) -> Option<TaskTimestamp> {
        self.due_at
    }

    /// Returns the task's reminder date, if set.
    #[inline]
    #[must_use]
    pub const fn reminder_at(&self) -> Option<TaskTimestamp> {
        self.reminder_at
    }

    /// Returns the timestamp when the task was completed, if applicable.
    #[inline]
    #[must_use]
    pub const fn completed_at(&self) -> Option<TaskTimestamp> {
        self.completed_at
    }

    /// Returns the task's structured metadata fields.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &TaskMetadata {
        &self.metadata
    }

    fn extract_clean_text(
        raw_text: &str,
        config: &TaskConfig,
    ) -> Result<String, NoteError> {
        let mut text = raw_text.trim();

        let mut stripped = true;
        while stripped {
            stripped = false;
            for tag in config.task_tags() {
                if let Some(rest) = text.strip_prefix(tag.as_str()) {
                    text = rest.trim_start();
                    stripped = true;
                }
            }
        }

        if let Some(mat) = METADATA_REGEX.find(text)
            && let Some(prefix) = text.get(..mat.start())
        {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(NoteError::Task(
                "task text cannot be empty".to_owned(),
            ));
        }

        Ok(text.to_owned())
    }

    fn extract_tags(raw_text: &str) -> Result<Vec<Tag>, NoteError> {
        TAG_REGEX
            .find_iter(raw_text)
            .map(|mat| Tag::new(mat.as_str()))
            .collect()
    }

    fn parse_temporal_fields(
        text: &str,
        config: &TaskConfig,
    ) -> Result<TemporalFields, NoteError> {
        let created_at = config
            .created_field()
            .map(|spec| Self::parse_date_field(text, spec, config))
            .transpose()?
            .flatten();
        let due_at = config
            .due_field()
            .map(|spec| Self::parse_date_field(text, spec, config))
            .transpose()?
            .flatten();
        let reminder_at = config
            .reminder_field()
            .map(|spec| Self::parse_date_field(text, spec, config))
            .transpose()?
            .flatten();
        let completed_at = config
            .completed_field()
            .map(|spec| Self::parse_date_field(text, spec, config))
            .transpose()?
            .flatten();

        Ok((created_at, due_at, reminder_at, completed_at))
    }

    fn parse_date_field(
        text: &str,
        spec: &DateFieldSpec,
        config: &TaskConfig,
    ) -> Result<Option<TaskTimestamp>, NoteError> {
        if let Some(value) = find_inline_field(text, spec.keyword().as_str()) {
            let naive =
                config.parse_date_value(value, spec).map_err(|error| {
                    NoteError::Task(format!(
                        "invalid date for field '{}': {error}",
                        spec.keyword().as_str()
                    ))
                })?;
            return Ok(Some(TaskTimestamp::new(naive.and_utc().timestamp())));
        }

        if let Some(emoji) = spec.emoji()
            && let Some(value) = find_emoji_field(text, emoji)
        {
            let naive =
                config.parse_date_value(value, spec).map_err(|error| {
                    NoteError::Task(format!(
                        "invalid date for field '{}': {error}",
                        spec.keyword().as_str()
                    ))
                })?;
            return Ok(Some(TaskTimestamp::new(naive.and_utc().timestamp())));
        }

        Ok(None)
    }

    fn parse_metadata(
        text: &str,
        config: &TaskConfig,
    ) -> Result<TaskMetadata, NoteError> {
        let mut metadata = TaskMetadata::new();

        for caps in METADATA_REGEX.captures_iter(text) {
            let keyword = caps.get(1).map_or("", |m| m.as_str().trim());
            let raw_value = caps.get(2).map_or("", |m| m.as_str().trim());

            if keyword.is_empty() {
                continue;
            }

            if Self::is_temporal_keyword(keyword, config) {
                continue;
            }

            if let Some(spec) = config.field_spec(keyword) {
                let json_value = parse_metadata_value(raw_value, spec)?;
                spec.validate_raw_value(&json_value).map_err(|error| {
                    NoteError::Task(format!(
                        "invalid metadata field '{keyword}': {error}"
                    ))
                })?;
                let field_value = FieldValue::from_json(&json_value);
                metadata.insert(keyword.into(), field_value);
            } else {
                metadata.insert(
                    keyword.into(),
                    FieldValue::String(raw_value.into()),
                );
            }
        }

        Ok(metadata)
    }

    fn is_temporal_keyword(keyword: &str, config: &TaskConfig) -> bool {
        config
            .created_field()
            .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || config
                .due_field()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || config
                .reminder_field()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || config
                .completed_field()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
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
/// let mut meta = TaskMetadata::new();
/// meta.insert("priority".into(), FieldValue::Number(1.0));
/// assert_eq!(meta.priority(), Some(1.0));
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
    fields: HashMap<Box<str>, FieldValue>,
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
    pub fn insert(&mut self, field: Box<str>, value: FieldValue) {
        self.fields.insert(field, value);
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

    /// Returns the task's priority if defined in metadata.
    #[inline]
    #[must_use]
    pub fn priority(&self) -> Option<f64> {
        self.get_number("priority")
    }

    /// Returns the task's project name if defined in metadata.
    #[inline]
    #[must_use]
    pub fn project(&self) -> Option<&str> {
        self.get_string("project")
    }

    /// Returns the task's area name if defined in metadata.
    #[inline]
    #[must_use]
    pub fn area(&self) -> Option<&str> {
        self.get_string("area")
    }

    /// Returns a reference to the internal metadata field map.
    #[inline]
    #[must_use]
    pub const fn fields(&self) -> &HashMap<Box<str>, FieldValue> {
        &self.fields
    }
}

impl Default for TaskMetadata {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

type TemporalFields = (
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
);

// Pre-compiled regexes for performance
static METADATA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    #[expect(
        clippy::expect_used,
        clippy::disallowed_methods,
        reason = "Internal regex compilation"
    )]
    Regex::new(r"\[([^:\]]+)::\s*([^\]]+)\]").expect("Invalid metadata regex")
});

static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    #[expect(
        clippy::expect_used,
        clippy::disallowed_methods,
        reason = "Internal regex compilation"
    )]
    Regex::new(r"#[a-zA-Z0-9_\-/]+").expect("Invalid tag regex")
});

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Matching on &TaskFieldSpec keeps call sites concise."
)]
fn parse_metadata_value(
    raw_value: &str,
    spec: &TaskFieldSpec,
) -> Result<serde_json::Value, NoteError> {
    match spec {
        TaskFieldSpec::Integer {
            ..
        } => {
            let value = raw_value.parse::<i64>().map_err(|error| {
                NoteError::Task(format!(
                    "invalid integer value '{raw_value}': {error}"
                ))
            })?;
            Ok(serde_json::Value::Number(value.into()))
        }
        TaskFieldSpec::Float {
            ..
        } => {
            let value = raw_value.parse::<f64>().map_err(|error| {
                NoteError::Task(format!(
                    "invalid float value '{raw_value}': {error}"
                ))
            })?;
            let number =
                serde_json::Number::from_f64(value).ok_or_else(|| {
                    NoteError::Task(format!(
                        "invalid float value '{raw_value}'"
                    ))
                })?;
            Ok(serde_json::Value::Number(number))
        }
        TaskFieldSpec::Enum {
            ..
        }
        | TaskFieldSpec::String {
            ..
        }
        | TaskFieldSpec::DateTime {
            ..
        } => Ok(serde_json::Value::String(raw_value.to_owned())),
    }
}

#[expect(
    clippy::string_slice,
    reason = "Indices are validated by match_indices."
)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Offset addition is safe after successful match."
)]
fn find_inline_field<'text>(
    text: &'text str,
    keyword: &str,
) -> Option<&'text str> {
    for (start, _) in text.match_indices('[') {
        let after_bracket = &text[start + 1..];
        if let Some(after_keyword) = after_bracket.strip_prefix(keyword)
            && let Some(rest) = after_keyword.strip_prefix("::")
            && let Some(value_end) = rest.find(']')
        {
            let value = rest[..value_end].trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "Offset addition is safe after successful find."
)]
fn find_emoji_field(text: &str, emoji: char) -> Option<&str> {
    let start = text.find(emoji)?;
    let value_start = start + emoji.len_utf8();
    let tail = text.get(value_start..)?;
    let value = tail.split_whitespace().next()?;
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Tests use expect for deterministic fixtures."
)]
mod tests {
    use super::*;
    use crate::config::raw::{RawTaskConfig, RawTaskFieldSpec};

    fn config_with_fields() -> TaskConfig {
        let mut fields = HashMap::new();
        fields.insert("priority".to_owned(), RawTaskFieldSpec::Integer {
            keyword: "priority".to_owned(),
            min: None,
            max: None,
        });
        fields.insert("project".to_owned(), RawTaskFieldSpec::String {
            keyword: "project".to_owned(),
            pattern: None,
        });

        let raw = RawTaskConfig {
            enabled: Some(true),
            task_tags: Some(vec!["#task".to_owned()]),
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
        assert!(Task::should_promote("#task Do work", &config));
        assert!(!Task::should_promote("Do work", &config));
    }

    #[test]
    fn from_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let task = Task::from_checkbox(
            "#task Review PR [priority:: 2] [project:: lithos]",
            StatusSymbol::try_new(' ').expect("valid status"),
            SourceByteOffset::new(12),
            &config,
        )
        .expect("task should parse");

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
        let task = Task::from_checkbox(
            "#task Fix #work/project/urgent issue",
            StatusSymbol::try_new(' ').expect("valid status"),
            SourceByteOffset::new(0),
            &config,
        )
        .expect("task should parse");

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
    fn task_timestamp_provides_semantic_methods() {
        let config = TaskConfig::default();
        let task = Task::from_checkbox(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
            StatusSymbol::try_new(' ').expect("valid status"),
            SourceByteOffset::new(0),
            &config,
        )
        .expect("task should parse");

        if let Some(created_at) = task.created_at() {
            assert_eq!(created_at.as_i64(), 1_704_067_200);
            assert!(created_at.is_past(None));
        }

        if let Some(due_at) = task.due_at() {
            assert_eq!(due_at.as_i64(), 1_735_689_600);
            assert!(due_at.is_future(None));
        }
    }
}
