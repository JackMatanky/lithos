//! Task entities and temporal metadata management for notes.
//!
//! This module defines the core types for representing **semantic** task
//! entities in markdown notes. Tasks are promoted from checkbox list items
//! ([`crate::note::list::ListItem`]) and carry rich domain semantics including
//! status tracking, temporal metadata (due dates, reminders), and validated
//! inline field data.
//!
//! # Tasks and Lists
//!
//! In the Lithos domain, there is a clear distinction between **Structural**
//! list items and **Semantic** tasks:
//!
//! 1. **Structural ([`crate::note::list::ListItem`]):** Manages the physical
//!    presence of a checkbox in the source file, including its position,
//!    nesting depth, and raw metadata (tags and inline fields). Every checkbox
//!    starts as a `ListItem`.
//!
//! 2. **Semantic ([`Task`]):** A "promoted" entity that represents a checkbox
//!    with validated domain logic. The promotion process extracts temporal
//!    metadata, resolves status names from checkbox symbols, computes clean
//!    text, and validates field values against configured schemas.
//!
//! The promotion from `ListItem` to `Task` happens via [`Task::promote`], which
//! applies the task configuration ([`crate::config::task::TaskConfigSpec`]) to
//! transform raw structural data into a validated domain entity.
//!
//! # Architecture
//!
//! - **[`Task`]**: The main entity representing a promoted checkbox item
//! - **[`TaskId`]**: UUID v7 identifier for tasks
//! - **[`TaskRef`]**: Lightweight reference to a task by its source range
//! - **[`TaskText`]**: Text container with raw and cleaned variants
//! - **[`TaskDates`]**: Semantic date slots (created, due, reminder, etc.)
//! - **[`TaskDateValue`]**: Typed date field with optional format spec
//! - **[`TaskDateKind`]**: Enum discriminating date field types
//!
//! # Examples
//!
//! ## Promoting from a ListItem
//!
//! ```no_run
//! # use lithos_core::note::{task::Task, list::ListItem};
//! # use lithos_core::config::task::TaskConfigSpec;
//! # fn example(item: &ListItem, spec: &TaskConfigSpec) -> Result<(), Box<dyn std::error::Error>> {
//! // Promote a checkbox ListItem to a semantic Task.
//! // This requires a parsed ListItem and a task configuration spec.
//! let task = Task::promote(item, spec)?;
//!
//! // Access validated domain properties
//! println!("Status: {}", task.status());
//! println!("Text: {}", task.text());
//! if let Some(due) = task.dates().due() {
//!     println!("Due: {:?}", due.as_naive_date());
//! }
//! # Ok(())
//! # }
//! ```

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
#![expect(
    clippy::iter_over_hash_type,
    reason = "Hash iteration order doesn't affect correctness here"
)]

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone as _};
use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::{NoteError, TaskError},
    inline_fields::InlineFieldKey,
    list::ListItem,
    position::{SourceByteOffset, SourceByteRange},
    tag::Tag,
    value::FieldValue,
};
use crate::config::task::TaskConfigSpec;

// ================================================================
// Core Task Entity
// ================================================================

/// A semantic task entity promoted from a checkbox list item.
///
/// `Task` represents a checkbox in a markdown note that has been promoted to a
/// domain entity with validated semantics. Unlike
/// [`crate::note::list::ListItem`] which captures raw structural data, `Task`
/// applies business logic such as:
///
/// - Status name resolution from checkbox symbols (e.g., ' ' -> "todo")
/// - Temporal metadata extraction into typed date slots
/// - Clean text computation by removing tags and inline fields
/// - Field value validation against configured schemas
///
/// # Promotion Process
///
/// Tasks are created via [`Task::promote`] which transforms a checkbox
/// `ListItem` using a [`crate::config::task::TaskConfigSpec`]:
///
/// 1. Resolve status symbol to configured status name
/// 2. Extract temporal fields (due, created, completed, etc.)
/// 3. Compute clean text by removing metadata ranges
/// 4. Validate field values against schema
/// 5. Generate unique UUID v7 identifier
///
/// # Invariants
///
/// - **Component Validity**: `TaskText` and `TaskDates` enforce their own
///   validation rules during construction
/// - **Immutable**: Task properties cannot be modified after creation
/// - **Unique**: Each task has a globally unique UUID v7 identifier
/// - **Clean Text**: Task text excludes tags and inline field syntax
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{task::Task, position::{SourceByteOffset, SourceByteRange}};
/// # use lithos_core::note::task::{TaskText, TaskDates};
/// # use std::collections::HashMap;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = "todo";
/// let range = SourceByteRange::new(SourceByteOffset::new(0), SourceByteOffset::new(10))?;
/// let text = TaskText::try_new("Urgent work".into(), "Urgent work".into())?;
/// let task = Task::try_new(
///     status.into(),
///     text,
///     range,
///     Box::new([]),
///     HashMap::new(),
///     TaskDates::default(),
/// )?;
///
/// assert_eq!(task.text(), "Urgent work");
/// assert_eq!(task.status(), "todo");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Task {
    /// Unique task identifier (UUID v7).
    id: TaskId,
    /// Status name (e.g., "todo", "done", "in-progress").
    status: Box<str>,
    /// Task text with raw and cleaned variants.
    text: TaskText,
    /// Source byte range in the note.
    range: SourceByteRange,
    /// Metadata tags extracted from the task text.
    tags: Box<[Tag]>,
    /// Inline field metadata as key-value pairs.
    fields: Box<[(InlineFieldKey, FieldValue)]>,
    /// Temporal metadata (due, created, completed, etc.).
    dates: TaskDates,
}

// ----------------------------------------------------------------
// Task - Construction and Accessors
// ----------------------------------------------------------------

impl Task {
    /// Creates a new [`Task`] from validated components.
    ///
    /// This is the primary constructor for creating tasks directly. For
    /// promoting checkbox list items to tasks, use [`Task::promote`] instead.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError`] if validation fails (currently no validation
    /// beyond what the component types enforce).
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::{task::Task, position::{SourceByteOffset, SourceByteRange}};
    /// # use lithos_core::note::task::{TaskText, TaskDates};
    /// # use std::collections::HashMap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let range = SourceByteRange::new(SourceByteOffset::new(0), SourceByteOffset::new(10))?;
    /// let text = TaskText::try_new("Write docs".into(), "Write docs".into())?;
    /// let task = Task::try_new(
    ///     "todo".into(),
    ///     text,
    ///     range,
    ///     Box::new([]),
    ///     HashMap::new(),
    ///     TaskDates::default(),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Task construction uses explicit components"
    )]
    pub fn try_new(
        status: Box<str>,
        text: TaskText,
        range: SourceByteRange,
        tags: Box<[Tag]>,
        fields: HashMap<InlineFieldKey, FieldValue>,
        dates: TaskDates,
    ) -> Result<Self, TaskError> {
        Ok(Self {
            id: TaskId::new(),
            status,
            text,
            range,
            tags,
            fields: fields.into_iter().collect(),
            dates,
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

    /// Returns the byte range of the task in the note source.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the collection of tags associated with this task.
    #[inline]
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.tags.iter()
    }

    /// Returns the task's inline field metadata.
    #[inline]
    #[must_use]
    pub fn fields(&self) -> &[(InlineFieldKey, FieldValue)] {
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
    /// Returns [`NoteError`] if:
    /// - The item is not a checkbox (missing task status marker)
    /// - Status mappings or metadata parsing fail
    #[inline]
    pub fn promote(
        item: &ListItem,
        spec: &TaskConfigSpec,
    ) -> Result<Self, NoteError> {
        // 1. Get status (caller should ensure item is a checkbox)
        let status_symbol =
            item.task_status().ok_or_else(|| TaskError::MissingStatus {
                text: item.text().to_owned().into(),
            })?;

        // Look up status name from symbol using config
        let status: Box<str> = spec
            .status_mappings
            .get(&status_symbol.value())
            .cloned()
            .unwrap_or_else(|| status_symbol.value().to_string().into());

        // 2. Copy all fields from ListItem
        let fields: HashMap<_, _> = item
            .fields()
            .iter()
            .map(|f| (f.key().clone(), f.value().clone()))
            .collect();

        // 3. Extract date slots from fields using spec
        let mut dates = TaskDates::new();
        for (key, value) in &fields {
            if let Some((kind, date_spec)) =
                TaskDates::match_date_spec(spec, key.as_kebab())
            {
                let date_value = TaskDateValue::from_field_value(
                    value,
                    key.as_str(),
                    Some(date_spec),
                )?;
                dates.set(kind, date_value);
            }
        }

        // 4. Compute clean text (range-based, no re-parsing)
        let mut exclusion_ranges = Vec::new();
        for tag in item.tags() {
            if let Some(range) = tag.range() {
                exclusion_ranges.push(range);
            }
        }
        for field in item.fields() {
            exclusion_ranges.push(field.range());
        }

        let text = TaskText::try_new_with_ranges(
            item.text(),
            item.text_range().start(),
            &exclusion_ranges,
        )?;

        // 5. Construct Task
        let tags = if item.tags().is_empty() {
            Box::new([])
        } else {
            item.tags().to_vec().into_boxed_slice()
        };

        Task::try_new(status, text, item.range(), tags, fields, dates)
            .map_err(Into::into)
    }
}

// ================================================================
// Task Identity and References
// ================================================================

/// Unique identifier for a task (UUID v7 timestamp-ordered).
///
/// Each task is assigned a globally unique identifier using UUID version 7,
/// which embeds a timestamp for typical chronological ordering. This makes
/// task IDs suitable for both identity and time-ordered indexing.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::task::TaskId;
/// let id1 = TaskId::new();
/// let id2 = TaskId::new();
///
/// // UUIDv7 is typically time-ordered
/// assert!(id1 != id2);
/// ```
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

/// Lightweight reference to a task by its source byte range.
///
/// `TaskRef` provides a compact way to reference a task without storing the
/// full [`TaskId`]. This is useful for linking [`crate::note::list::ListItem`]s
/// to their promoted tasks, allowing bidirectional navigation between the
/// structural (list) and semantic (task) representations.
///
/// # Usage
///
/// When a checkbox `ListItem` is promoted to a `Task`, the list item stores a
/// `TaskRef` pointing to the task's source range. This enables queries like
/// "which list items have been promoted" without duplicating task data.
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
///
/// `TaskText` stores both the original markdown text and a cleaned version
/// with tags and inline field syntax removed. This dual representation allows:
///
/// - **Raw text**: Preserves the original source for reference
/// - **Clean text**: Provides human-readable text for display and search
///
/// # Cleaning Process
///
/// The cleaning algorithm removes exclusion ranges (tags, inline fields) while
/// preserving the remaining text structure:
///
/// 1. Convert absolute byte ranges to relative offsets
/// 2. Sort and merge overlapping ranges
/// 3. Extract text between exclusion ranges
/// 4. Trim whitespace from the final result
///
/// # Examples
///
/// ```
/// # use lithos_core::note::task::TaskText;
/// let text = TaskText::try_new(
///     "Review PR #urgent [priority:: 1]".into(),
///     "Review PR".into(),
/// )
/// .expect("valid text");
///
/// assert_eq!(text.raw(), "Review PR #urgent [priority:: 1]");
/// assert_eq!(text.clean(), "Review PR");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct TaskText {
    /// Raw text as it appears in the source markdown.
    raw: Box<str>,
    /// Cleaned text with tags and inline fields removed.
    clean: Box<str>,
}

// ----------------------------------------------------------------
// TaskText - Construction and Accessors
// ----------------------------------------------------------------

impl TaskText {
    /// Creates a validated task text from raw and cleaned variants.
    ///
    /// Use this constructor when you have already computed the cleaned text.
    /// For automatic cleaning based on exclusion ranges, use
    /// [`TaskText::try_new_with_ranges`] instead.
    ///
    /// # Errors
    ///
    /// Returns [`TaskError::EmptyText`] if the cleaned text is empty after
    /// trimming whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::task::TaskText;
    /// let text = TaskText::try_new("Do work #tag".into(), "Do work".into())
    ///     .expect("valid text");
    ///
    /// assert_eq!(text.raw(), "Do work #tag");
    /// assert_eq!(text.clean(), "Do work");
    /// ```
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
    /// This method is used during task promotion to remove tags and inline
    /// fields from the raw text. The cleaning algorithm:
    ///
    /// 1. Converts absolute ranges to relative offsets based on `base`
    /// 2. Filters out ranges outside the raw text bounds
    /// 3. Sorts and merges overlapping ranges
    /// 4. Extracts text between exclusion ranges
    /// 5. Trims the final result
    ///
    /// # Errors
    ///
    /// Returns [`TaskError::EmptyText`] if the cleaned text is empty after
    /// removing all exclusion ranges and trimming.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::task::TaskText;
    /// # use lithos_core::note::position::{SourceByteOffset, SourceByteRange};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let raw = "Review PR #urgent [priority:: 1]";
    /// let base = SourceByteOffset::new(0);
    ///
    /// // Exclude "#urgent" (10..17) and "[priority:: 1]" (18..32)
    /// let exclusions = vec![
    ///     SourceByteRange::new(SourceByteOffset::new(10), SourceByteOffset::new(17))?,
    ///     SourceByteRange::new(SourceByteOffset::new(18), SourceByteOffset::new(32))?,
    /// ];
    ///
    /// let text = TaskText::try_new_with_ranges(raw, base, &exclusions)?;
    /// assert_eq!(text.clean(), "Review PR");
    /// # Ok(())
    /// # }
    /// ```
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
        for range in relative {
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
///
/// `TaskDates` stores validated temporal metadata in six standard slots used
/// across task management systems. Each slot is optional and type-safe,
/// holding either a date or datetime value with an associated format spec.
///
/// # Temporal Slots
///
/// - **Created**: When the task was originally created
/// - **Due**: Target completion date/time
/// - **Reminder**: When to send a notification
/// - **Completed**: When the task was marked done
/// - **Start**: Earliest time to begin work
/// - **Scheduled**: Planned work session date/time
///
/// # Usage
///
/// Date slots are extracted during task promotion by matching inline field
/// keywords against the configured temporal specs. For example, with default
/// configuration:
///
/// ```markdown
/// - [ ] #task Review PR [due:: 2024-12-31] [created:: 2024-01-01]
/// ```
///
/// Maps to `due` and `created` slots based on the `TaskConfigSpec`.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::task::TaskDates;
/// let dates = TaskDates::new();
/// assert!(dates.due().is_none());
/// assert!(dates.created().is_none());
/// ```
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

// ----------------------------------------------------------------
// TaskDates - Construction and Accessors
// ----------------------------------------------------------------

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
    ///
    /// Returns the slot kind and the `DateSpec` if the keyword maps to a
    /// temporal slot.
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
///
/// `TaskDateValue` wraps a [`FieldValue`] that is guaranteed to be temporal
/// (date or datetime) and optionally retains the [`DateSpec`]
/// configuration that was used to parse it. This makes it possible to
/// reformat or validate the value consistently with configuration.
///
/// # Parsing Behavior
///
/// When created during task promotion, values are parsed according to the
/// configured date spec if the input is a string. If parsing fails, the logic
/// falls back to RFC3339 or `YYYY-MM-DD` formats. If the value is already a
/// typed temporal [`FieldValue`], it is accepted without additional parsing.
/// Non-temporal values yield a validation error.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::task::TaskDateValue;
/// # use lithos_core::note::value::FieldValue;
/// # use chrono::NaiveDate;
/// let date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
/// let value = TaskDateValue::new(FieldValue::Date(date.into()), None)
///     .expect("temporal value");
/// assert_eq!(value.as_naive_date(), Some(date));
/// ```
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
    /// Returns [`TaskError::InvalidDate`] if the value is not temporal.
    #[inline]
    pub fn new(
        value: FieldValue,
        spec: Option<Arc<crate::config::value::DateSpec>>,
    ) -> Result<Self, TaskError> {
        if !value.is_temporal() {
            return Err(TaskError::InvalidDate {
                keyword: "".into(),
                raw: value.to_json_string().into(),
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

    /// Create from pre-typed `FieldValue` (from `ListItem`).
    fn from_field_value(
        value: &FieldValue,
        key: &str,
        spec: Option<Arc<crate::config::value::DateSpec>>,
    ) -> Result<Self, TaskError> {
        match value {
            FieldValue::Date(_) | FieldValue::DateTime(_) => {
                Self::new(value.clone(), spec)
            }
            FieldValue::String(s) => {
                // Rare fallback: value wasn't typed during parsing
                Self::parse_heuristic(s, key, spec)
            }
            FieldValue::Number(_)
            | FieldValue::Boolean(_)
            | FieldValue::Time(_)
            | FieldValue::Duration(_)
            | FieldValue::Array(_)
            | FieldValue::Object(_)
            | FieldValue::Null => Err(TaskError::InvalidDate {
                keyword: key.into(),
                raw: value.to_json_string().into(),
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
///
/// `TaskDateKind` identifies which temporal slot a value belongs to during
/// promotion and query-time operations (e.g., "list all tasks due today").
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

#[cfg(test)]
#[expect(
    clippy::shadow_unrelated,
    reason = "Test code prioritizes readability"
)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;

    use super::*;
    use crate::{
        config::task::{TaskConfigSpec, TemporalSlot},
        note::{
            raw::{RawListItem, RawListKind, RawTaskMarker},
            scanner::{NoteScanner, ScannedArtifact},
        },
    };

    fn task_spec_fixture() -> TaskConfigSpec {
        use crate::config::{raw::RawDateFieldSpec, value::DateSpec};

        let mut temporal_specs = HashMap::new();

        let due_spec = DateSpec::try_from_raw(RawDateFieldSpec {
            keyword: "due".to_owned(),
            emoji: Some('\u{1f4c5}'),
            format: "%Y-%m-%d".to_owned(),
        })
        .expect("valid due spec");
        temporal_specs.insert(
            "due".into(),
            (TemporalSlot::Due, Arc::new(due_spec), Some('\u{1f4c5}')),
        );

        let created_spec = DateSpec::try_from_raw(RawDateFieldSpec {
            keyword: "created".to_owned(),
            emoji: Some('\u{2795}'),
            format: "%Y-%m-%d".to_owned(),
        })
        .expect("valid created spec");
        temporal_specs.insert(
            "created".into(),
            (TemporalSlot::Created, Arc::new(created_spec), Some('\u{2795}')),
        );

        let completed_spec = DateSpec::try_from_raw(RawDateFieldSpec {
            keyword: "completed".to_owned(),
            emoji: Some('\u{2705}'),
            format: "%Y-%m-%d".to_owned(),
        })
        .expect("valid completed spec");
        temporal_specs.insert(
            "completed".into(),
            (
                TemporalSlot::Completed,
                Arc::new(completed_spec),
                Some('\u{2705}'),
            ),
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

        let priority_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "priority")
            .map(|(_, v)| v);
        assert_eq!(
            priority_field.and_then(super::super::value::FieldValue::as_number),
            Some(2.0f64)
        );

        let project_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "project")
            .map(|(_, v)| v);
        assert_eq!(project_field.and_then(|v| v.as_str()), Some("lithos"));
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

        let created_date =
            NaiveDate::from_ymd_opt(2024, 1, 1).expect("created date");
        let due_date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("due date");

        if let Some(created_at) = task.dates().created() {
            assert_eq!(created_at.as_naive_date(), Some(created_date));
        }

        if let Some(due_at) = task.dates().due() {
            assert_eq!(due_at.as_naive_date(), Some(due_date));
        }

        if let (Some(created_at), Some(due_at)) =
            (task.dates().created(), task.dates().due())
        {
            let created_date = date_of(created_at);
            let due_date = date_of(due_at);
            assert!(created_date < due_date);
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

        let priority_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "priority")
            .map(|(_, v)| v);
        assert_eq!(
            priority_field.and_then(super::super::value::FieldValue::as_number),
            Some(2.0f64)
        );

        let project_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "project")
            .map(|(_, v)| v);
        assert_eq!(project_field.and_then(|v| v.as_str()), Some("lithos"));
    }

    fn date_of(value: &TaskDateValue) -> NaiveDate {
        value.as_naive_date().expect("date")
    }

    fn promote_task(
        promoted_text: &str,
        spec: &TaskConfigSpec,
        emoji_markers: &[char],
    ) -> Task {
        let item = list_item_from_text(promoted_text, emoji_markers);
        Task::promote(&item, spec).expect("task conversion")
    }

    fn list_item_from_text(raw_text: &str, emoji_markers: &[char]) -> ListItem {
        let scanner = NoteScanner::new(emoji_markers.to_vec());
        let start = SourceByteOffset::new(0);
        let end = SourceByteOffset::try_from(raw_text.len()).unwrap_or(start);
        let range = SourceByteRange::new(start, end).expect("valid test range");

        let artifacts = scanner
            .scan_block(raw_text, SourceByteOffset::new(0))
            .expect("scan artifacts");

        let mut tags = Vec::new();
        let mut inline_fields = Vec::new();
        let mut task_marker = None;

        for artifact in artifacts {
            match artifact {
                ScannedArtifact::Tag(tag) => tags.push(tag),
                ScannedArtifact::InlineField(field) => {
                    let crate::note::raw::RawInlineFieldToken {
                        key,
                        value,
                        range,
                    } = field;
                    let typed_value =
                        crate::note::raw::RawFieldValue::from_str_with_spec(
                            value.as_ref(),
                            key.as_ref(),
                            None,
                        )
                        .into_owned();
                    inline_fields.push(crate::note::raw::RawInlineField::new(
                        key,
                        typed_value,
                        range,
                    ));
                }
                ScannedArtifact::TaskMarker {
                    marker,
                    ..
                } => {
                    task_marker = Some(RawTaskMarker::from_char(marker));
                }
                ScannedArtifact::BlockRef(_) => {}
            }
        }

        let mut is_checked = task_marker
            .map(|marker| matches!(marker, RawTaskMarker::Checked(_)));
        if task_marker.is_none() {
            task_marker = Some(RawTaskMarker::Unchecked(' '));
            is_checked = Some(false);
        }

        let raw = RawListItem::new(
            RawListKind::Unordered,
            crate::note::raw::RawListDepth::Root,
            raw_text.into(),
            is_checked,
            task_marker,
            range,
            range,
            None,
            tags,
            inline_fields,
        );

        ListItem::try_from(&raw).expect("valid list item")
    }
}
