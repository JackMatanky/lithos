//! Task subentity for Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::{collections::HashMap, sync::OnceLock};

use regex::Regex;
use uuid::Uuid;

use super::{error::NoteError, types::SourceByteOffset, value::FieldValue};
use crate::config::task::{
    DateFieldSpec, StatusName, StatusSymbol, TaskConfig, TaskFieldSpec,
};

/// Unique identifier for a Task (UUID v7).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskId(Uuid);

impl TaskId {
    /// Creates a new time-ordered task id.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the inner UUID.
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
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
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<TaskId> for Uuid {
    #[inline]
    fn from(value: TaskId) -> Self {
        value.0
    }
}

/// Task metadata parsed from inline fields.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TaskMetadata {
    fields: HashMap<Box<str>, FieldValue>,
}

impl TaskMetadata {
    /// Creates an empty metadata map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Inserts a metadata field.
    #[inline]
    pub fn insert(&mut self, field: Box<str>, value: FieldValue) {
        self.fields.insert(field, value);
    }

    /// Returns a field value by name.
    #[inline]
    #[must_use]
    pub fn get(&self, field: &str) -> Option<&FieldValue> {
        self.fields.get(field)
    }

    /// Returns a string field value if present.
    #[inline]
    #[must_use]
    pub fn get_string(&self, field: &str) -> Option<&str> {
        self.get(field)?.as_str()
    }

    /// Returns a numeric field value if present.
    #[inline]
    #[must_use]
    pub fn get_number(&self, field: &str) -> Option<f64> {
        self.get(field)?.as_number()
    }

    /// Returns a boolean field value if present.
    #[inline]
    #[must_use]
    pub fn get_boolean(&self, field: &str) -> Option<bool> {
        self.get(field)?.as_bool()
    }

    /// Returns a date timestamp field value if present.
    #[inline]
    #[must_use]
    pub fn get_date(&self, field: &str) -> Option<i64> {
        self.get(field)?.as_date()
    }
}

impl Default for TaskMetadata {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a promoted task entity parsed from a checkbox list item.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Task {
    id: TaskId,
    text: Box<str>,
    status: StatusName,
    created_at: Option<i64>,
    due_at: Option<i64>,
    reminder_at: Option<i64>,
    completed_at: Option<i64>,
    position: SourceByteOffset,
    tags: Vec<Box<str>>,
    metadata: TaskMetadata,
}

type TemporalFields = (Option<i64>, Option<i64>, Option<i64>, Option<i64>);

impl Task {
    /// Creates a task from checkbox text, validating against task config.
    ///
    /// # Errors
    ///
    /// Returns `NoteError::Task` when the status symbol is unknown, when task
    /// text is empty after normalization, or when metadata parsing fails.
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
                    "unknown status symbol '{}'",
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
            text: text.into_boxed_str(),
            status,
            created_at,
            due_at,
            reminder_at,
            completed_at,
            position,
            tags,
            metadata,
        })
    }

    /// Returns true if the checkbox text should be promoted to a Task.
    #[inline]
    #[must_use]
    pub fn should_promote(text: &str, config: &TaskConfig) -> bool {
        config.has_task_tag(text)
    }

    /// Returns the task id.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the normalized task text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the semantic status name.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> &StatusName {
        &self.status
    }

    /// Returns the created timestamp if present.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<i64> {
        self.created_at
    }

    /// Returns the due timestamp if present.
    #[inline]
    #[must_use]
    pub const fn due_at(&self) -> Option<i64> {
        self.due_at
    }

    /// Returns the reminder timestamp if present.
    #[inline]
    #[must_use]
    pub const fn reminder_at(&self) -> Option<i64> {
        self.reminder_at
    }

    /// Returns the completed timestamp if present.
    #[inline]
    #[must_use]
    pub const fn completed_at(&self) -> Option<i64> {
        self.completed_at
    }

    /// Returns the source byte offset of this task.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Returns the tags found in the task text.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }

    /// Returns task metadata.
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

        let regex = metadata_regex()?;
        if let Some(mat) = regex.find(text)
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

    fn extract_tags(raw_text: &str) -> Result<Vec<Box<str>>, NoteError> {
        let regex = tag_regex()?;
        Ok(regex
            .find_iter(raw_text)
            .map(|mat| mat.as_str().to_owned().into_boxed_str())
            .collect())
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
    ) -> Result<Option<i64>, NoteError> {
        if let Some(value) = find_inline_field(text, spec.keyword().as_str()) {
            let naive =
                config.parse_date_value(value, spec).map_err(|error| {
                    NoteError::Task(format!(
                        "invalid date for field '{}': {error}",
                        spec.keyword().as_str()
                    ))
                })?;
            return Ok(Some(naive.and_utc().timestamp()));
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
            return Ok(Some(naive.and_utc().timestamp()));
        }

        Ok(None)
    }

    fn parse_metadata(
        text: &str,
        config: &TaskConfig,
    ) -> Result<TaskMetadata, NoteError> {
        let regex = metadata_regex()?;
        let mut metadata = TaskMetadata::new();

        for caps in regex.captures_iter(text) {
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

fn metadata_regex() -> Result<&'static Regex, NoteError> {
    static REGEX: OnceLock<Result<Regex, NoteError>> = OnceLock::new();
    let cached = REGEX.get_or_init(|| {
        Regex::new(r"\[([^:\]]+)::\s*([^\]]+)\]").map_err(|error| {
            NoteError::Task(format!("metadata regex error: {error}"))
        })
    });
    cached.as_ref().map_err(Clone::clone)
}

fn tag_regex() -> Result<&'static Regex, NoteError> {
    static REGEX: OnceLock<Result<Regex, NoteError>> = OnceLock::new();
    let cached = REGEX.get_or_init(|| {
        Regex::new("#[A-Za-z0-9_-]+").map_err(|error| {
            NoteError::Task(format!("tag regex error: {error}"))
        })
    });
    cached.as_ref().map_err(Clone::clone)
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
    }

    #[test]
    fn from_checkbox_collects_tags() {
        let config = TaskConfig::default();
        let task = Task::from_checkbox(
            "#task #work Fix bug",
            StatusSymbol::try_new(' ').expect("valid status"),
            SourceByteOffset::new(0),
            &config,
        )
        .expect("task should parse");

        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#task"));
        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#work"));
    }
}
