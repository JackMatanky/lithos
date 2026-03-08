//! List extraction from markdown event streams.
//!
//! Builds ordered/unordered lists from pulldown-cmark list events, tracks list
//! depth, and captures list items with source offsets. Checkbox items may be
//! promoted to tasks when configured promotion tags are present; the list item
//! retains the task id linkage.

use std::ops::Range;

use pulldown_cmark::{Event, Tag as CmarkTag, TagEnd};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::{
    config::{
        aggregate::Config,
        task::{StatusSymbol, Task as TaskConfig},
        value::{DateSpec, FieldSpec},
    },
    note::{
        error::{NoteError, TaskError},
        list::{List, ListDepth, ListItem, ListType},
        position::SourceByteOffset,
        tag::{Tag, scan_tags},
        task::{
            Task, TaskAttributes, TaskAttributesBuilder, TaskFieldKey,
            TaskMetadata, TaskTimestamp,
        },
        value::FieldValue,
    },
};

/// Output from list extraction - either a list or a promoted task.
#[derive(Debug)]
pub enum ExtractionOutput {
    /// A complete list with items.
    List(List),
    /// A task promoted from a checkbox item with a promotion tag.
    Task(Box<Task>),
}

/// Extractor for markdown lists, checkboxes, and task promotion.
///
/// Processes markdown list events and builds domain `List` entities.
/// Handles nested lists by maintaining a stack. Checkboxes with promotion
/// tags will be extracted as separate `Task` entities.
///
/// ## Task Promotion
///
/// When a checkbox contains a tag matching the configured task promotion tags,
/// it will be promoted to a `Task` entity and emitted immediately. The list
/// item will link to the task via `task_id`.
pub struct ListExtractor<'config> {
    config: &'config Config,
    list_stack: Vec<List>,
    current_item: Option<ItemBuilder>,
}

/// Builder for accumulating list item data during extraction.
struct ItemBuilder {
    position: SourceByteOffset,
    text: String,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ItemBuilder {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            is_checkbox: false,
            status_symbol: None,
        }
    }

    fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked {
            'x'
        } else {
            ' '
        });
    }

    fn add_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}

impl<'config> ListExtractor<'config> {
    /// Creates a new list extractor bound to the provided configuration.
    ///
    /// This is the standard constructor for creating a list extractor.
    #[inline]
    pub(super) const fn new(config: &'config Config) -> Self {
        Self {
            config,
            list_stack: Vec::new(),
            current_item: None,
        }
    }

    /// Adds a completed item to the current list, potentially promoting to
    /// task.
    ///
    /// Takes `ItemBuilder` by value since it's consumed during list item
    /// construction.
    ///
    /// Returns `Some(Task)` if the checkbox was promoted to a task.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ItemBuilder is intentionally consumed to build ListItem"
    )]
    fn add_item_to_list(
        &mut self,
        item: ItemBuilder,
    ) -> Result<Option<Box<Task>>, NoteError> {
        let mut promoted_task = None;
        let mut status = None;

        if item.is_checkbox {
            // Check for task promotion
            let checkbox_status =
                StatusSymbol::try_new(item.status_symbol.unwrap_or(' '))?;
            if !self.config.task().tags().is_empty() {
                let tags = scan_tags(&item.text);
                promoted_task = self.parse_promoted_checkbox_with_tags(
                    &item.text,
                    tags,
                    checkbox_status,
                    item.position,
                )?;
            }
            status = Some(checkbox_status);
        }

        if let Some(list) = self.list_stack.last_mut() {
            if let Some(checkbox_status) = status {
                let task_id = promoted_task.as_ref().map(Task::id);
                list.add_item(ListItem::Checkbox {
                    text: item.text.trim().into(),
                    status: checkbox_status,
                    position: item.position,
                    task_id,
                });
            } else {
                list.add_item(ListItem::Plain {
                    text: item.text.trim().into(),
                    position: item.position,
                });
            }
        }

        Ok(promoted_task.map(Box::new))
    }

    fn parse_promoted_checkbox_with_tags(
        &self,
        raw_text: &str,
        tags: Vec<Tag>,
        status_symbol: StatusSymbol,
        position: SourceByteOffset,
    ) -> Result<Option<Task>, NoteError> {
        let task_config = self.config.task();
        if !Self::should_promote_from_tags(task_config, &tags) {
            return Ok(None);
        }

        let status = task_config
            .status()
            .name_for_symbol(status_symbol)
            .ok_or_else(|| {
                NoteError::Task(TaskError::UnrecognizedStatusSymbol {
                    symbol: status_symbol.value(),
                })
            })?
            .clone();
        let text = Self::extract_clean_text(task_config, raw_text)?;
        let parsed = Self::parse_inline_fields(task_config, raw_text)?;
        let attributes = parsed.into_attributes(tags);

        Task::try_new(status, text, position, attributes).map(Some)
    }

    fn should_promote_from_tags(
        task_config: &TaskConfig,
        tags: &[Tag],
    ) -> bool {
        task_config.tags().iter().any(|config_tag| {
            tags.iter().any(|tag| {
                config_tag
                    .as_str()
                    .strip_prefix('#')
                    .is_some_and(|raw| raw == tag.full_path())
            })
        })
    }

    fn extract_clean_text(
        task_config: &TaskConfig,
        raw_text: &str,
    ) -> Result<Box<str>, NoteError> {
        let mut text = raw_text.trim();

        let mut stripped = true;
        while stripped {
            stripped = false;
            for tag in task_config.tags() {
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
        task_config: &TaskConfig,
        text: &str,
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        Self::for_each_inline_field(text, |keyword, raw_value| {
            state.handle_inline_field(task_config, keyword, raw_value)
        })?;

        state.fill_emoji_dates(task_config, text)?;
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
            let close_rel = bytes.get(after_open..).and_then(|slice| {
                slice.iter().position(|&b| b == close_delim)
            })?;
            let close = after_open.saturating_add(close_rel);
            let inner = text.get(after_open..close)?;
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
        ParsedInlineFields {
            slots: self,
            metadata,
        }
    }

    fn get(&self, slot: DateSlot) -> Option<TaskTimestamp> {
        match slot {
            DateSlot::Created => self.created_at,
            DateSlot::Due => self.due_at,
            DateSlot::Reminder => self.reminder_at,
            DateSlot::Completed => self.completed_at,
        }
    }

    fn set(&mut self, slot: DateSlot, value: TaskTimestamp) {
        match slot {
            DateSlot::Created => self.created_at = Some(value),
            DateSlot::Due => self.due_at = Some(value),
            DateSlot::Reminder => self.reminder_at = Some(value),
            DateSlot::Completed => self.completed_at = Some(value),
        }
    }

    fn apply_to_builder(
        self,
        builder: TaskAttributesBuilder,
    ) -> TaskAttributesBuilder {
        builder
            .created_at(self.created_at)
            .due_at(self.due_at)
            .reminder_at(self.reminder_at)
            .completed_at(self.completed_at)
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
        let Some(emoji) = spec.emoji() else {
            return Ok(());
        };
        let Some(value) = Self::find_emoji_field(text, emoji) else {
            return Ok(());
        };
        slots.set(slot, Self::parse_date_str(value, spec)?);
        Ok(())
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

    fn fill_default_emoji_slot(
        &mut self,
        slot: DateSlot,
        emoji: char,
        keyword: &str,
        text: &str,
    ) -> Result<(), NoteError> {
        if self.slots.get(slot).is_some() {
            return Ok(());
        }
        let Some(value) = Self::find_emoji_field(text, emoji) else {
            return Ok(());
        };
        let parsed = Self::parse_default_date(value, keyword)?;
        self.slots.set(slot, parsed);
        Ok(())
    }

    fn fill_default_emoji_metadata(
        &mut self,
        keyword: &str,
        emoji: char,
        text: &str,
    ) -> Result<(), NoteError> {
        if self.metadata.get(keyword).is_some() {
            return Ok(());
        }
        let Some(value) = Self::find_emoji_field(text, emoji) else {
            return Ok(());
        };
        let parsed = Self::parse_default_date(value, keyword)?;
        let key = TaskFieldKey::try_new(keyword)?;
        self.metadata.insert(key, FieldValue::Date(parsed.as_i64()));
        Ok(())
    }
}

impl Extractor for ListExtractor<'_> {
    type Error = NoteError;
    type Output = ExtractionOutput;

    fn finish(self) -> Result<Vec<ExtractionOutput>, NoteError> {
        // Flush any incomplete lists
        Ok(self.list_stack.into_iter().map(ExtractionOutput::List).collect())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn process(
        &mut self,
        event: &Event<'_>,
        text: &str,
        range: &Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<ExtractionOutput>, NoteError> {
        match event {
            Event::Start(CmarkTag::List(start)) => {
                // Start a new list
                let depth = ListDepth::try_new(self.list_stack.len())?;
                let list_type = match *start {
                    Some(start_num) => ListType::Ordered {
                        start: start_num,
                    },
                    None => ListType::Unordered,
                };
                self.list_stack.push(List::with_depth(list_type, depth));
                Ok(ExtractionState::Continue)
            }

            Event::Start(CmarkTag::Item) => {
                // Start a new list item
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.current_item = Some(ItemBuilder::new(position));
                Ok(ExtractionState::Continue)
            }

            Event::TaskListMarker(checked) => {
                // Mark current item as checkbox
                if let Some(item) = self.current_item.as_mut() {
                    item.mark_as_checkbox(*checked);
                }
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) => {
                // Accumulate text in current item
                if let Some(item) = self.current_item.as_mut() {
                    item.add_text(text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Item) => {
                // Complete current item and add to list (potentially promoting
                // to task)
                if let Some(item) = self.current_item.take()
                    && let Some(task) = self.add_item_to_list(item)?
                {
                    // Checkbox was promoted - emit task immediately
                    return Ok(ExtractionState::Emit(ExtractionOutput::Task(
                        task,
                    )));
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::List(_)) => {
                // Complete list and emit
                if let Some(list) = self.list_stack.pop() {
                    return Ok(ExtractionState::Emit(ExtractionOutput::List(
                        list,
                    )));
                }
                Ok(ExtractionState::Continue)
            }

            // Ignore other events
            Event::Start(_)
            | Event::End(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule => Ok(ExtractionState::Continue),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pulldown_cmark::{Event, Tag as CmarkTag, TagEnd};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::{RawConfig, RawFieldSpec, RawTaskConfig},
            task::StatusSymbol,
            vault::{VaultId, VaultRoot},
        },
        note::{
            list::ListType,
            reader::{ExtractionContext, ExtractionState},
        },
    };

    #[test]
    fn extracts_plain_unordered_list() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        let result1 = extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();
        assert!(matches!(result1, ExtractionState::Continue));

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(4..12),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(12..13), &ctx)
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(13..14), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list emission");
        };
        assert_eq!(list.items().count(), 1);
        assert!(matches!(list.list_type(), ListType::Unordered));
    }

    #[test]
    fn extracts_ordered_list() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list with start number
        extractor
            .process(&Event::Start(CmarkTag::List(Some(1))), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text("First item".into()),
                "First item",
                &(4..14),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(14..15), &ctx)
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(&Event::End(TagEnd::List(true)), "", &(20..21), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list emission");
        };
        assert_eq!(list.items().count(), 1);
        assert!(matches!(list.list_type(), ListType::Ordered {
            start: 1
        }));
    }

    #[test]
    fn extracts_checkbox_unchecked() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..7), &ctx)
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(7..15),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(15..16), &ctx)
            .unwrap();

        // End list
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(16..17), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list");
        };
        let item = list.items().next().unwrap();
        assert!(item.status().is_some()); // Is a checkbox
        assert!(item.task_id().is_none()); // Not promoted
    }

    #[test]
    fn extracts_checkbox_checked() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (checked)
        extractor
            .process(&Event::TaskListMarker(true), "", &(4..7), &ctx)
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text("Done task".into()),
                "Done task",
                &(7..16),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(16..17), &ctx)
            .unwrap();

        // End list
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(17..18), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list");
        };
        let item = list.items().next().unwrap();
        assert!(item.status().is_some()); // Is a checkbox
    }

    #[test]
    fn checkbox_without_promotion_tag_stays_as_list_item() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text without promotion tag
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(5..13),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(13..14), &ctx)
            .unwrap();

        // End list - should emit List (not Task)
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(14..15), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list, not task");
        };

        let item = list.items().next().unwrap();
        assert!(matches!(item, ListItem::Checkbox { .. }));
        assert!(item.task_id().is_none()); // Not promoted
    }

    #[test]
    fn checkbox_with_promotion_tag_becomes_task() {
        let config = test_config_with_task_tag();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text with promotion tag
        extractor
            .process(
                &Event::Text("#task Review PR".into()),
                "#task Review PR",
                &(5..20),
                &ctx,
            )
            .unwrap();

        // End item - should emit Task immediately
        let result = extractor
            .process(&Event::End(TagEnd::Item), "", &(20..21), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::Task(task)) = result else {
            panic!("Expected task emission on item end");
        };

        assert_eq!(task.text(), "Review PR");
        assert!(task.tags().any(|t| t.full_path() == "task"));
    }

    #[test]
    fn promoted_task_links_to_list_item() {
        let config = test_config_with_task_tag();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text with promotion tag
        extractor
            .process(
                &Event::Text("#task Deploy".into()),
                "#task Deploy",
                &(5..17),
                &ctx,
            )
            .unwrap();

        // End item - emit task
        let task_result = extractor
            .process(&Event::End(TagEnd::Item), "", &(17..18), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::Task(task)) = task_result
        else {
            panic!("Expected task");
        };
        let task_id = task.id();

        // End list - emit list
        let list_result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(18..19), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = list_result
        else {
            panic!("Expected list");
        };

        // Verify link
        let item = list.items().next().unwrap();
        assert_eq!(item.task_id(), Some(task_id));
    }

    #[test]
    fn promotes_only_when_task_tag_present() {
        let config = test_config_with_task_tag();
        let extractor = ListExtractor::new(&config);

        let promoted = extractor
            .parse_promoted_checkbox_with_tags(
                "#task Do work",
                scan_tags("#task Do work"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(promoted.is_some());

        let skipped = extractor
            .parse_promoted_checkbox_with_tags(
                "Do work",
                scan_tags("Do work"),
                StatusSymbol::try_new(' ').expect("valid status"),
                SourceByteOffset::new(0),
            )
            .expect("parse should succeed");
        assert!(skipped.is_none());

        let skipped_partial = extractor
            .parse_promoted_checkbox_with_tags(
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
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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

        assert!(task.created_at().is_none() || task.created_at().is_some());
        assert!(task.due_at().is_none() || task.due_at().is_some());
        assert!(task.reminder_at().is_none() || task.reminder_at().is_some());
        assert!(task.completed_at().is_none() || task.completed_at().is_some());
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let config = test_config_with_task_tag();
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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
        let extractor = ListExtractor::new(&config);
        let task = extractor
            .parse_promoted_checkbox_with_tags(
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

    fn test_config() -> Config {
        let raw = RawConfig::default();
        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    fn test_config_with_task_tag() -> Config {
        use crate::config::raw::RawTaskConfig;

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
