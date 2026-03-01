//! Task parsing helpers for markdown ingestion.

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
        let (created_at, due_at, reminder_at, completed_at, metadata) =
            self.parse_inline_fields(raw_text)?;
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

        if let Some(prefix) = Self::strip_inline_fields(text) {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }

        Ok(text.into())
    }

    fn parse_inline_fields(
        self,
        text: &str,
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        Self::for_each_inline_field(text, |keyword, raw_value| {
            state.handle_inline_field(self.config, keyword, raw_value)
        })?;

        state.fill_emoji_dates(self.config, text)?;

        Ok(state.finish())
    }

    fn for_each_inline_field(
        text: &str,
        mut f: impl FnMut(&str, &str) -> Result<(), NoteError>,
    ) -> Result<(), NoteError> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == b'['))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == b']'))
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
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == b'['))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let close_rel = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == b']'))?;
            let close = after_open.saturating_add(close_rel);
            let inner = text.get(after_open..close)?;
            if let Some((key, value)) = inner.split_once("::")
                && !key.trim().is_empty()
                && !value.trim().is_empty()
            {
                return text.get(..open);
            }
            cursor = close.saturating_add(1);
        }
        None
    }
}

type ParsedInlineFields = (
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
    Option<TaskTimestamp>,
    TaskMetadata,
);

#[derive(Debug, Default)]
#[expect(
    clippy::struct_field_names,
    reason = "Temporal slots share consistent suffixes by design."
)]
struct TemporalSlots {
    created_at: Option<TaskTimestamp>,
    due_at: Option<TaskTimestamp>,
    reminder_at: Option<TaskTimestamp>,
    completed_at: Option<TaskTimestamp>,
}

impl TemporalSlots {
    fn finish(self, metadata: TaskMetadata) -> ParsedInlineFields {
        (
            self.created_at,
            self.due_at,
            self.reminder_at,
            self.completed_at,
            metadata,
        )
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
        if let Some(spec) = Self::match_date_spec(config, keyword) {
            let parsed = Self::parse_date_str(raw_value, spec)?;
            Self::assign_date_for_keyword(
                config,
                spec.keyword().as_str(),
                parsed,
                &mut self.slots,
            );
            return Ok(());
        }

        Self::insert_metadata(config, &mut self.metadata, keyword, raw_value)
    }

    fn fill_emoji_dates(
        &mut self,
        config: &TaskConfig,
        text: &str,
    ) -> Result<(), NoteError> {
        if let Some(spec) = config.created()
            && self.slots.created_at.is_none()
            && let Some(emoji) = spec.emoji()
            && let Some(value) = Self::find_emoji_field(text, emoji)
        {
            self.slots.created_at = Some(Self::parse_date_str(value, spec)?);
        }

        if let Some(spec) = config.due()
            && self.slots.due_at.is_none()
            && let Some(emoji) = spec.emoji()
            && let Some(value) = Self::find_emoji_field(text, emoji)
        {
            self.slots.due_at = Some(Self::parse_date_str(value, spec)?);
        }

        if let Some(spec) = config.reminder()
            && self.slots.reminder_at.is_none()
            && let Some(emoji) = spec.emoji()
            && let Some(value) = Self::find_emoji_field(text, emoji)
        {
            self.slots.reminder_at = Some(Self::parse_date_str(value, spec)?);
        }

        if let Some(spec) = config.completed()
            && self.slots.completed_at.is_none()
            && let Some(emoji) = spec.emoji()
            && let Some(value) = Self::find_emoji_field(text, emoji)
        {
            self.slots.completed_at = Some(Self::parse_date_str(value, spec)?);
        }

        Ok(())
    }

    fn finish(self) -> ParsedInlineFields {
        self.slots.finish(self.metadata)
    }

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

    fn assign_date_for_keyword(
        config: &TaskConfig,
        keyword: &str,
        value: TaskTimestamp,
        slots: &mut TemporalSlots,
    ) {
        if config
            .created()
            .is_some_and(|spec| spec.keyword().as_str() == keyword)
        {
            slots.created_at = Some(value);
            return;
        }

        if config.due().is_some_and(|spec| spec.keyword().as_str() == keyword) {
            slots.due_at = Some(value);
            return;
        }

        if config
            .reminder()
            .is_some_and(|spec| spec.keyword().as_str() == keyword)
        {
            slots.reminder_at = Some(value);
            return;
        }

        if config
            .completed()
            .is_some_and(|spec| spec.keyword().as_str() == keyword)
        {
            slots.completed_at = Some(value);
        }
    }

    fn insert_metadata(
        config: &TaskConfig,
        metadata: &mut TaskMetadata,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some(spec) = config.field_spec(keyword) {
            let json_value = Self::parse_metadata_value(raw_value, spec)?;
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

        Ok(())
    }

    fn match_date_spec<'config>(
        config: &'config TaskConfig,
        keyword: &str,
    ) -> Option<&'config DateSpec> {
        if let Some(spec) = config.created()
            && spec.keyword().as_str() == keyword
        {
            return Some(spec);
        }
        if let Some(spec) = config.due()
            && spec.keyword().as_str() == keyword
        {
            return Some(spec);
        }
        if let Some(spec) = config.reminder()
            && spec.keyword().as_str() == keyword
        {
            return Some(spec);
        }
        if let Some(spec) = config.completed()
            && spec.keyword().as_str() == keyword
        {
            return Some(spec);
        }
        None
    }

    fn parse_date_str(
        value: &str,
        spec: &DateSpec,
    ) -> Result<TaskTimestamp, NoteError> {
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
}
