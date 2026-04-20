//! Task entities and temporal metadata management for notes.
//!
//! This module defines the core types for representing **semantic** task
//! entities in markdown notes. Tasks are promoted from checkbox list items
//! ([`crate::note::list::ListItem`]).

#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "Task module uses generated and value types"
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "Pattern matching style is clear in context"
)]

use std::sync::Arc;

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone as _};
use rkyv::{Archive, Deserialize, Serialize};

use super::{
    error::{NoteError, TaskError},
    inline_fields::InlineField,
    list::{ListItem, ListItemBase, ListItemId},
    position::{SourceByteOffset, SourceByteRange},
    tag::Tag,
    value::FieldValue,
};
use crate::config::task::TaskConfigSpec;

// ================================================================
// Core Task Entity
// ================================================================

/// A semantic task entity promoted from a checkbox list item.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Task {
    /// The underlying structural list item base.
    pub base: ListItemBase,
    /// Status name (e.g., "todo", "done", "in-progress").
    pub status: Box<str>,
    /// Task text with raw and cleaned variants.
    pub task_text: TaskText,
    /// Temporal metadata (due, created, completed, etc.).
    pub dates: TaskDates,
    /// Metadata fields associated with the task.
    pub fields: Box<[InlineField]>,
}

impl Task {
    /// Creates a new [`Task`] from validated components.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if components are invalid.
    #[inline]
    pub fn try_new(
        base: ListItemBase,
        status: Box<str>,
        task_text: TaskText,
        dates: TaskDates,
        fields: Box<[InlineField]>,
    ) -> Result<Self, TaskError> {
        Ok(Self {
            base,
            status,
            task_text,
            dates,
            fields,
        })
    }

    /// Returns the unique task identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> ListItemId {
        self.base.id
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
        self.task_text.clean()
    }

    /// Returns the byte range of the task in the note source.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.base.range
    }

    /// Returns the collection of tags associated with this task.
    #[inline]
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.base.tags.iter()
    }

    /// Returns the task's inline field metadata.
    #[inline]
    #[must_use]
    pub fn fields(&self) -> &[InlineField] {
        &self.fields
    }

    /// Returns the task's date slots.
    #[inline]
    #[must_use]
    pub const fn dates(&self) -> &TaskDates {
        &self.dates
    }

    /// Promotes a list item into a fully validated [`Task`].
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if promotion fails.
    #[inline]
    pub fn promote(
        item: ListItem,
        source: &str,
        spec: &TaskConfigSpec,
    ) -> Result<Self, NoteError> {
        // 1. Extract marker character lazily from source
        let status_symbol =
            extract_task_marker_from_source(source, item.range())?;

        // Look up status name from symbol using config
        let status: Box<str> =
            spec.status_mappings.get(&status_symbol).cloned().unwrap_or_else(
                || {
                    let mut buf = [0u8; 4];
                    status_symbol.encode_utf8(&mut buf).into()
                },
            );

        // 2. Extract date slots from fields using spec
        let mut dates = TaskDates::new();
        for field in item.fields() {
            let key = field.key();
            if let Some((kind, date_spec)) =
                TaskDates::match_date_spec(spec, key.as_kebab())
            {
                let date_value = TaskDateValue::from_field_value(
                    field.value(),
                    key.as_str(),
                    Some(date_spec),
                )?;
                dates.set(kind, date_value);
            }
        }

        // 3. Compute clean text (range-based, no re-parsing)
        let mut exclusion_ranges = Vec::new();
        for tag in item.tags() {
            if let Some(range) = tag.range() {
                exclusion_ranges.push(range);
            }
        }
        for field in item.fields() {
            exclusion_ranges.push(field.range());
        }

        // We also exclude the marker range
        if let Some(marker_range) = find_checkbox_range(source, item.range()) {
            exclusion_ranges.push(marker_range);
        }

        let task_text = TaskText::try_new_with_ranges(
            item.text(),
            item.range().start(),
            &exclusion_ranges,
        )?;

        // 4. Construct Task
        let ListItem {
            base,
            fields,
        } = item;

        Task::try_new(base, status, task_text, dates, fields)
            .map_err(Into::into)
    }
}

/// Extracts the task marker character from the source range.
fn extract_task_marker_from_source(
    source: &str,
    range: SourceByteRange,
) -> Result<char, NoteError> {
    let segment = source
        .get(range.start().as_usize()..range.end().as_usize())
        .ok_or_else(|| NoteError::Internal("invalid source range".into()))?;

    // Look for [X] pattern
    if let Some(start_bracket) = segment.find('[') {
        let after_bracket =
            segment.get(start_bracket.saturating_add(1)..).unwrap_or("");
        let mut chars = after_bracket.chars();
        if let Some(marker_char) = chars.next() {
            let char_len = marker_char.len_utf8();
            if after_bracket.get(char_len..).is_some_and(|s| s.starts_with(']'))
            {
                return Ok(marker_char);
            }
        }
    }

    // Default if not found (shouldn't happen for valid task list items)
    Ok(' ')
}

/// Finds the source byte range of the checkbox [X].
fn find_checkbox_range(
    source: &str,
    range: SourceByteRange,
) -> Option<SourceByteRange> {
    let start_idx = range.start().as_usize();
    let segment = source.get(start_idx..range.end().as_usize())?;

    if let Some(start_bracket) = segment.find('[') {
        let after_bracket =
            segment.get(start_bracket.saturating_add(1)..).unwrap_or("");
        let mut chars = after_bracket.chars();
        if let Some(marker_char) = chars.next() {
            let char_len = marker_char.len_utf8();
            if after_bracket.get(char_len..).is_some_and(|s| s.starts_with(']'))
            {
                let range_start =
                    range.start().add_offset(start_bracket).ok()?;
                let range_end = range_start
                    .add_offset(2usize.saturating_add(char_len))
                    .ok()?;
                return SourceByteRange::new(range_start, range_end).ok();
            }
        }
    }
    None
}

// ================================================================
// Task Identity and References
// ================================================================

/// Lightweight reference to a task by its source byte range.
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

// ================================================================
// Task Text Processing
// ================================================================

/// Task text container with raw and cleaned variants.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskText {
    /// Raw text as it appears in the source markdown.
    raw: Box<str>,
    /// Cleaned text with tags and inline fields removed.
    clean: Box<str>,
}

impl TaskText {
    /// Creates a validated task text from raw and cleaned variants.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if the clean text is empty.
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

    /// Creates task text by automatically computing clean text from exclusion
    /// ranges.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if the resulting clean text is empty.
    #[inline]
    pub fn try_new_with_ranges(
        raw: &str,
        base: SourceByteOffset,
        ranges: &[SourceByteRange],
    ) -> Result<Self, TaskError> {
        let mut relative = Vec::new();
        let base_offset = base.as_usize();
        let raw_len = raw.len();

        for range in ranges {
            let start = range.start().as_usize();
            let end = range.end().as_usize();
            if start < base_offset || end <= start {
                continue;
            }
            let rel_start = start.saturating_sub(base_offset);
            let rel_end = end.saturating_sub(base_offset);
            if rel_start >= raw_len || rel_end > raw_len {
                continue;
            }
            relative.push(rel_start..rel_end);
        }

        relative.sort_by_key(|range| range.start);
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for range in &relative {
            if let Some(last) = merged.last_mut()
                && range.start <= last.1
            {
                last.1 = last.1.max(range.end);
                continue;
            }
            merged.push((range.start, range.end));
        }

        let mut cleaned = String::with_capacity(raw_len);
        let mut cursor = 0usize;
        for range in &merged {
            if let Some(slice) = raw.get(cursor..range.0) {
                cleaned.push_str(slice);
            }
            cursor = range.1;
        }
        if let Some(slice) = raw.get(cursor..raw_len) {
            cleaned.push_str(slice);
        }

        let clean = cleaned.trim();
        TaskText::try_new(raw.into(), clean.into())
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

// ================================================================
// Task Temporal Metadata
// ================================================================

/// Semantic date slots extracted from task inline fields.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskDates {
    created: Option<TaskDateValue>,
    due: Option<TaskDateValue>,
    reminder: Option<TaskDateValue>,
    completed: Option<TaskDateValue>,
    start: Option<TaskDateValue>,
    scheduled: Option<TaskDateValue>,
}

impl TaskDates {
    /// Create a new empty `TaskDates` with no date slots set.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            created: None,
            due: None,
            reminder: None,
            completed: None,
            start: None,
            scheduled: None,
        }
    }

    /// Returns the created date if set.
    #[inline]
    #[must_use]
    pub const fn created(&self) -> Option<&TaskDateValue> {
        self.created.as_ref()
    }

    /// Returns the due date if set.
    #[inline]
    #[must_use]
    pub const fn due(&self) -> Option<&TaskDateValue> {
        self.due.as_ref()
    }

    /// Returns the reminder date if set.
    #[inline]
    #[must_use]
    pub const fn reminder(&self) -> Option<&TaskDateValue> {
        self.reminder.as_ref()
    }

    /// Returns the completed date if set.
    #[inline]
    #[must_use]
    pub const fn completed(&self) -> Option<&TaskDateValue> {
        self.completed.as_ref()
    }

    /// Returns the start date if set.
    #[inline]
    #[must_use]
    pub const fn start(&self) -> Option<&TaskDateValue> {
        self.start.as_ref()
    }

    /// Returns the scheduled date if set.
    #[inline]
    #[must_use]
    pub const fn scheduled(&self) -> Option<&TaskDateValue> {
        self.scheduled.as_ref()
    }

    /// Set a date slot value (private helper for `Task::promote`).
    fn set(&mut self, kind: TaskDateKind, value: TaskDateValue) {
        match kind {
            TaskDateKind::Created => self.created = Some(value),
            TaskDateKind::Due => self.due = Some(value),
            TaskDateKind::Reminder => self.reminder = Some(value),
            TaskDateKind::Completed => self.completed = Some(value),
            TaskDateKind::Start => self.start = Some(value),
            TaskDateKind::Scheduled => self.scheduled = Some(value),
        }
    }

    /// Map keyword to date slot using the task config spec.
    fn match_date_spec(
        spec: &TaskConfigSpec,
        keyword: &str,
    ) -> Option<(TaskDateKind, Arc<crate::config::value::DateSpec>)> {
        use crate::config::task::TemporalSlot;

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics are preferred for mapping lookups"
        )]
        let (slot_enum, date_spec, _emoji) =
            spec.temporal_specs.get(keyword)?;
        let kind = match *slot_enum {
            TemporalSlot::Created => TaskDateKind::Created,
            TemporalSlot::Due => TaskDateKind::Due,
            TemporalSlot::Reminder => TaskDateKind::Reminder,
            TemporalSlot::Completed => TaskDateKind::Completed,
            TemporalSlot::Start => TaskDateKind::Start,
            TemporalSlot::Scheduled => TaskDateKind::Scheduled,
        };
        Some((kind, Arc::clone(date_spec)))
    }
}

impl Default for TaskDates {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ================================================================
// Task Date Values
// ================================================================

/// Typed date field for task temporal metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskDateValue {
    value: FieldValue,
    spec: Option<Arc<crate::config::value::DateSpec>>,
}

impl TaskDateValue {
    /// Create from a typed `FieldValue` with optional spec.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if the value is not a valid date or datetime.
    #[inline]
    pub fn new(
        value: FieldValue,
        spec: Option<Arc<crate::config::value::DateSpec>>,
    ) -> Result<Self, TaskError> {
        if !value.is_temporal() {
            return Err(TaskError::InvalidDate {
                keyword: "".into(),
                raw: value.to_string().into(),
                reason: "expected date or datetime value",
            });
        }
        Ok(Self {
            value,
            spec,
        })
    }

    /// Returns the inner `FieldValue`.
    #[inline]
    #[must_use]
    pub const fn as_field_value(&self) -> &FieldValue {
        &self.value
    }

    /// Returns the configured format spec if available.
    #[inline]
    #[must_use]
    pub fn spec(&self) -> Option<&crate::config::value::DateSpec> {
        self.spec.as_deref()
    }

    /// Returns as `NaiveDate` (extracts date from `DateTime` if needed).
    #[inline]
    #[must_use]
    pub fn as_naive_date(&self) -> Option<NaiveDate> {
        self.value
            .as_naive_date()
            .or_else(|| self.value.as_datetime().map(|dt| dt.date_naive()))
    }

    /// Returns as `DateTime` (promotes Date to `DateTime` at 00:00:00 if
    /// needed).
    #[inline]
    #[must_use]
    pub fn as_datetime(&self) -> Option<DateTime<FixedOffset>> {
        // Try as DateTime first
        if let Some(dt) = self.value.as_datetime() {
            return Some(dt);
        }

        // Promote Date -> DateTime with 00:00:00 time
        if let Some(d) = self.value.as_naive_date() {
            let naive_dt = d.and_hms_opt(0, 0, 0)?;
            let utc = chrono::Utc.from_local_datetime(&naive_dt).single()?;
            return Some(utc.fixed_offset());
        }

        None
    }

    /// Create from pre-typed `FieldValue`.
    fn from_field_value(
        value: &FieldValue,
        key: &str,
        spec: Option<Arc<crate::config::value::DateSpec>>,
    ) -> Result<Self, TaskError> {
        match value {
            FieldValue::Date(_) | FieldValue::DateTime(_) => {
                Self::new(value.clone(), spec)
            }
            FieldValue::String(s) => Self::parse_heuristic(s, key, spec),
            FieldValue::Number(_)
            | FieldValue::Boolean(_)
            | FieldValue::Time(_)
            | FieldValue::Duration(_)
            | FieldValue::Array(_)
            | FieldValue::Object(_)
            | FieldValue::Null => Err(TaskError::InvalidDate {
                keyword: key.into(),
                raw: value.to_string().into(),
                reason: "expected date or datetime value",
            }),
        }
    }

    /// Heuristic parsing fallback for string values.
    fn parse_heuristic(
        s: &str,
        key: &str,
        spec: Option<Arc<crate::config::value::DateSpec>>,
    ) -> Result<Self, TaskError> {
        // Try spec format first
        if let Some(date_spec) = &spec {
            if let Ok(d) = NaiveDate::parse_from_str(s, date_spec.format()) {
                return Self::new(FieldValue::Date(d.into()), spec);
            }
            if let Ok(dt) = DateTime::parse_from_str(s, date_spec.format()) {
                return Self::new(FieldValue::DateTime(dt.into()), spec);
            }
            return Err(TaskError::InvalidDate {
                keyword: key.into(),
                raw: s.into(),
                reason: "does not match configured format",
            });
        }

        // Try RFC3339 datetime
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Self::new(FieldValue::DateTime(dt.into()), spec);
        }

        // Try YYYY-MM-DD date
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Self::new(FieldValue::Date(d.into()), spec);
        }

        Err(TaskError::InvalidDate {
            keyword: key.into(),
            raw: s.into(),
            reason: "unrecognized date format",
        })
    }
}

// ================================================================
// Task Date Kinds
// ================================================================

/// Task date fields used for date-based queries and indexing.
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
    /// Start timestamp.
    Start,
    /// Scheduled timestamp.
    Scheduled,
}
