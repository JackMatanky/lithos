//! Task parsing helpers for markdown ingestion.

use std::sync::LazyLock;

use regex::Regex;

use crate::{
    config::{
        task::{StatusSymbol, Task as TaskConfig},
        value::{DateSpec, FieldSpec},
    },
    note::{
        error::{NoteError, TaskError},
        tag::Tag,
        task::{
            Task, TaskAttributes, TaskFieldKey, TaskMetadata, TaskTimestamp,
        },
        types::SourceByteOffset,
        value::FieldValue,
    },
};

/// Parses markdown task list items into `Task` entities.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskParser<'config> {
    config: &'config TaskConfig,
}

impl<'config> TaskParser<'config> {
    #[inline]
    pub(crate) const fn new(config: &'config TaskConfig) -> Self {
        Self {
            config,
        }
    }

    #[inline]
    pub(crate) fn parse_promoted_checkbox_with_tags(
        self,
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
        let (created_at, due_at, reminder_at, completed_at) =
            self.parse_temporal_fields(raw_text)?;
        let metadata = self.parse_metadata(raw_text)?;
        let attributes = TaskAttributes::builder()
            .tags(tags)
            .metadata(metadata)
            .created_at(created_at)
            .due_at(due_at)
            .reminder_at(reminder_at)
            .completed_at(completed_at)
            .build();

        Task::new(status, text, position, attributes).map(Some)
    }

    #[inline]
    fn should_promote_from_tags(self, tags: &[Tag]) -> bool {
        self.config.tags().iter().any(|config_tag| {
            tags.iter().any(|tag| {
                config_tag
                    .as_str()
                    .strip_prefix('#')
                    .is_some_and(|raw| raw == tag.full_path())
            })
        })
    }

    fn extract_clean_text(self, raw_text: &str) -> Result<Box<str>, NoteError> {
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

        if let Some(mat) = METADATA_REGEX.find(text)
            && let Some(prefix) = text.get(..mat.start())
        {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }

        Ok(text.into())
    }

    fn parse_temporal_fields(
        self,
        text: &str,
    ) -> Result<TemporalFields, NoteError> {
        let created_at = self
            .config
            .created()
            .map(|spec| Self::parse_date_field(text, spec))
            .transpose()?
            .flatten();
        let due_at = self
            .config
            .due()
            .map(|spec| Self::parse_date_field(text, spec))
            .transpose()?
            .flatten();
        let reminder_at = self
            .config
            .reminder()
            .map(|spec| Self::parse_date_field(text, spec))
            .transpose()?
            .flatten();
        let completed_at = self
            .config
            .completed()
            .map(|spec| Self::parse_date_field(text, spec))
            .transpose()?
            .flatten();

        Ok((created_at, due_at, reminder_at, completed_at))
    }

    fn parse_date_field(
        text: &str,
        spec: &DateSpec,
    ) -> Result<Option<TaskTimestamp>, NoteError> {
        let parse_date_str = |value: &str| -> Result<TaskTimestamp, NoteError> {
            if let Ok(naive) =
                chrono::NaiveDateTime::parse_from_str(value, spec.format())
            {
                return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
            }

            let date = chrono::NaiveDate::parse_from_str(value, spec.format())
                .map_err(|error| {
                    NoteError::Task(TaskError::InvalidDate {
                        keyword: spec.keyword().as_str().into(),
                        reason: error.to_string().into(),
                    })
                })?;

            let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                NoteError::Task(TaskError::InvalidDateTime {
                    keyword: spec.keyword().as_str().into(),
                })
            })?;

            Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
        };

        if let Some(value) = find_inline_field(text, spec.keyword().as_str()) {
            return parse_date_str(value).map(Some);
        }

        if let Some(emoji) = spec.emoji()
            && let Some(value) = find_emoji_field(text, emoji)
        {
            return parse_date_str(value).map(Some);
        }

        Ok(None)
    }

    fn parse_metadata(self, text: &str) -> Result<TaskMetadata, NoteError> {
        let mut metadata = TaskMetadata::new();

        for caps in METADATA_REGEX.captures_iter(text) {
            let keyword = caps.get(1).map_or("", |m| m.as_str().trim());
            let raw_value = caps.get(2).map_or("", |m| m.as_str().trim());

            if keyword.is_empty() {
                continue;
            }

            if self.is_temporal_keyword(keyword) {
                continue;
            }

            if let Some(spec) = self.config.field_spec(keyword) {
                let json_value = parse_metadata_value(raw_value, spec)?;
                spec.validate_raw_value(&json_value).map_err(|error| {
                    NoteError::Task(TaskError::InvalidMetadataField {
                        keyword: keyword.into(),
                        reason: error.to_string().into(),
                    })
                })?;
                let field_value =
                    FieldValue::from_json(&json_value).map_err(|error| {
                        NoteError::Task(TaskError::InvalidMetadataField {
                            keyword: keyword.into(),
                            reason: error.to_string().into(),
                        })
                    })?;
                let key = TaskFieldKey::try_new(keyword)?;
                metadata.insert(key, field_value);
            } else {
                let key = TaskFieldKey::try_new(keyword)?;
                metadata.insert(key, FieldValue::String(raw_value.into()));
            }
        }

        Ok(metadata)
    }

    fn is_temporal_keyword(self, keyword: &str) -> bool {
        self.config
            .created()
            .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || self
                .config
                .due()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || self
                .config
                .reminder()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
            || self
                .config
                .completed()
                .is_some_and(|spec| spec.keyword().as_str() == keyword)
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
    #[expect(clippy::expect_used, reason = "Internal regex compilation")]
    Regex::new(r"\[([^:\]]+)::\s*([^\]]+)\]").expect("Invalid metadata regex")
});

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Matching on &TaskFieldSpec keeps call sites concise."
)]
fn parse_metadata_value(
    raw_value: &str,
    spec: &FieldSpec,
) -> Result<serde_json::Value, NoteError> {
    match spec {
        FieldSpec::Integer {
            ..
        } => {
            let value = raw_value.parse::<i64>().map_err(|error| {
                NoteError::Task(TaskError::InvalidInteger {
                    raw: raw_value.into(),
                    reason: error.to_string().into(),
                })
            })?;
            Ok(serde_json::Value::Number(value.into()))
        }
        FieldSpec::Float {
            ..
        } => {
            let value = raw_value.parse::<f64>().map_err(|error| {
                NoteError::Task(TaskError::InvalidFloat {
                    raw: raw_value.into(),
                    reason: error.to_string().into(),
                })
            })?;
            let number =
                serde_json::Number::from_f64(value).ok_or_else(|| {
                    NoteError::Task(TaskError::InvalidFloat {
                        raw: raw_value.into(),
                        reason: "float value is not finite".into(),
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
