//! Task configuration schema and validation.
//!
//! This module provides the [`Task`] aggregate and supporting types
//! for defining how Markdown-based tasks are recognized and indexed.

#![allow(
    clippy::allow_attributes,
    clippy::missing_trait_methods,
    dead_code,
    reason = "Internal validation helpers and clippy compatibility"
)]
#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv::Archive derive generates exhaustive archived enums \
              despite #[non_exhaustive]"
)]

use std::{collections::HashMap, sync::Arc};

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    error::ConfigError,
    raw::{RawStatusSpec, RawTaskConfig},
    value::{DateSpec, FieldSpec},
};

// ----------------------------------------------------------- //
//                     Public Domain Types                     //
// ----------------------------------------------------------- //

/// Validated task configuration aggregate.
///
/// This struct defines how tasks (e.g., in Markdown files) are recognized,
/// parsed, and indexed by Lithos. It ensures all field keywords and status
/// symbols are valid and unique.
///
/// # Always Valid Invariants
///
/// - **Promotion Tags**: Task tags must start with `#`.
/// - **Status Mappings**: Checkbox symbols must be printable ASCII and unique.
/// - **Field Keywords**: Custom field keywords must be ASCII alphanumeric.
/// - **Field Integrity**: All indexed fields must exist in the field
///   definitions.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::task::Task;
///
/// let config = Task::default();
/// assert!(config.enabled());
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Config flags are inherently boolean - state machine would add \
              complexity"
)]
pub struct Task {
    /// Whether task processing is enabled.
    enabled: bool,
    /// Configured task promotion tags.
    tags: Vec<TaskTag>,
    /// Status mappings for checkboxes.
    status: CheckboxStatus,
    /// Whether emoji format is used for task metadata.
    use_emoji: bool,
    /// Optional due date field configuration.
    due: Option<DateSpec>,
    /// Optional created date field configuration.
    created: Option<DateSpec>,
    /// Optional reminder date field configuration.
    reminder: Option<DateSpec>,
    /// Optional completed date field configuration.
    completed: Option<DateSpec>,
    /// Optional start date field configuration.
    start: Option<DateSpec>,
    /// Optional scheduled date field configuration.
    scheduled: Option<DateSpec>,
    /// Custom task field specifications.
    fields: HashMap<Box<str>, FieldSpec>,
    /// List of field names to be indexed.
    indexed: Vec<Box<str>>,
    /// Whether task dependency indexing is enabled.
    dependencies_enabled: bool,
}

impl Default for Task {
    #[inline]
    #[expect(
        clippy::unwrap_used,
        reason = "Default config is guaranteed valid"
    )]
    fn default() -> Self {
        Self::try_from_raw(RawTaskConfig::default()).unwrap()
    }
}

impl Task {
    #[inline]
    /// Builds a validated task configuration from raw input.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the configuration
    /// invariants are violated (e.g., duplicate status symbols or unknown
    /// indexed fields).
    pub fn try_from_raw(raw: RawTaskConfig) -> Result<Self, ConfigError> {
        let enabled = raw.enabled.unwrap_or(true);
        let tags = match raw.task_tags {
            Some(tags) => tags
                .into_iter()
                .map(TaskTag::try_new)
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![TaskTag::try_new("#task")?],
        };

        let status = if let Some(mapping) = raw.status {
            CheckboxStatus::try_from_raw(mapping)?
        } else {
            let mut mapping = HashMap::new();
            mapping.insert("todo".to_owned(), RawStatusSpec {
                symbol: ' ',
                status_type: StatusType::Todo,
                next_symbol: None,
            });
            mapping.insert("done".to_owned(), RawStatusSpec {
                symbol: 'x',
                status_type: StatusType::Done,
                next_symbol: None,
            });
            CheckboxStatus::try_from_raw(mapping)?
        };

        let (due, completed, created, reminder, start, scheduled) =
            match raw.dates {
                Some(dates) => (
                    dates.due.map(DateSpec::try_from_raw).transpose()?,
                    dates.completed.map(DateSpec::try_from_raw).transpose()?,
                    dates.created.map(DateSpec::try_from_raw).transpose()?,
                    dates.reminder.map(DateSpec::try_from_raw).transpose()?,
                    dates.start.map(DateSpec::try_from_raw).transpose()?,
                    dates.scheduled.map(DateSpec::try_from_raw).transpose()?,
                ),
                None => (None, None, None, None, None, None),
            };

        let mut fields = HashMap::new();
        if let Some(raw_fields) = raw.fields {
            let mut entries: Vec<_> = raw_fields.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (name, spec) in entries {
                fields.insert(
                    name.clone().into_boxed_str(),
                    FieldSpec::try_from_raw(&name, spec)?,
                );
            }
        }

        let mut indexed = Vec::new();
        if let Some(indexing) = raw.indexing
            && let Some(fields_list) = indexing.indexed_fields
        {
            for field_name in fields_list {
                if !fields.contains_key(field_name.as_str()) {
                    return Err(ConfigError::ValidationFailed {
                        field: "task.indexing.fields".into(),
                        message: format!("unknown field: {field_name}").into(),
                    });
                }
                indexed.push(field_name.into_boxed_str());
            }
        }

        Ok(Self {
            enabled,
            tags,
            status,
            use_emoji: raw.use_emoji.unwrap_or(true),
            due,
            created,
            reminder,
            completed,
            start,
            scheduled,
            fields,
            indexed,
            dependencies_enabled: raw
                .dependencies
                .and_then(|deps| deps.enabled)
                .unwrap_or(false),
        })
    }

    #[inline]
    #[must_use]
    /// Return whether task processing is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    #[must_use]
    /// Return the list of task promotion tags.
    pub fn tags(&self) -> &[TaskTag] {
        &self.tags
    }

    #[inline]
    #[must_use]
    /// Return the checkbox status mappings.
    pub fn status(&self) -> &CheckboxStatus {
        &self.status
    }

    #[inline]
    #[must_use]
    /// Return whether emoji metadata is enabled.
    pub fn use_emoji(&self) -> bool {
        self.use_emoji
    }

    #[inline]
    #[must_use]
    /// Return the due date field spec, if configured.
    pub fn due(&self) -> Option<&DateSpec> {
        self.due.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the created date field spec, if configured.
    pub fn created(&self) -> Option<&DateSpec> {
        self.created.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the reminder date field spec, if configured.
    pub fn reminder(&self) -> Option<&DateSpec> {
        self.reminder.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the completed date field spec, if configured.
    pub fn completed(&self) -> Option<&DateSpec> {
        self.completed.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the start date field spec, if configured.
    pub fn start(&self) -> Option<&DateSpec> {
        self.start.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the scheduled date field spec, if configured.
    pub fn scheduled(&self) -> Option<&DateSpec> {
        self.scheduled.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the custom fields map.
    pub fn fields(&self) -> &HashMap<Box<str>, FieldSpec> {
        &self.fields
    }

    #[inline]
    #[must_use]
    /// Return the list of indexed field names.
    pub fn indexed(&self) -> &[Box<str>] {
        &self.indexed
    }

    #[inline]
    #[must_use]
    /// Return whether task dependency indexing is enabled.
    pub fn dependencies_enabled(&self) -> bool {
        self.dependencies_enabled
    }

    #[inline]
    /// Look up a field specification by name.
    #[must_use]
    pub fn field_spec(&self, name: &str) -> Option<&FieldSpec> {
        self.fields.get(name)
    }
}

// ----------------------------------------------------------- //
//                    Building Block Types                     //
// ----------------------------------------------------------- //

/// Identified temporal slots for task metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalSlot {
    /// When the task was created.
    Created,
    /// When the task is due.
    Due,
    /// When to remind about the task.
    Reminder,
    /// When the task was completed.
    Completed,
    /// When the task should start.
    Start,
    /// When the task is scheduled.
    Scheduled,
}

/// Mapping from keyword to temporal slot, date spec, and optional emoji.
pub type TemporalMapping =
    HashMap<Box<str>, (TemporalSlot, Arc<DateSpec>, Option<char>)>;

/// Lightweight contract for task scanning and promotion.
///
/// This struct consolidates all configuration rules needed to identify and
/// promote checkboxes into domain Tasks, using only basic data types.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TaskConfigSpec {
    /// Whether task processing is enabled globally.
    pub enabled: bool,
    /// Whether emoji format is allowed for task metadata.
    pub use_emoji: bool,
    /// Emojis that trigger colon-less inline fields (e.g., 📅).
    pub emoji_markers: Box<[char]>,
    /// Tags that trigger task promotion (e.g., #task).
    pub promotion_tags: Box<[Box<str>]>,
    /// Maps checkbox symbols to status names (e.g., 'x' -> "done").
    pub status_mappings: HashMap<char, Box<str>>,
    /// Maps keywords to temporal slots and their parsing rules.
    pub temporal_specs: TemporalMapping,
    /// Maps keywords to custom field validation rules.
    pub field_specs: HashMap<Box<str>, FieldSpec>,
}

impl TaskConfigSpec {
    /// Creates a new task configuration spec.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Mapping contract provides exhaustive domain facts"
    )]
    pub fn new(
        enabled: bool,
        use_emoji: bool,
        emoji_markers: Box<[char]>,
        promotion_tags: Box<[Box<str>]>,
        status_mappings: HashMap<char, Box<str>>,
        temporal_specs: TemporalMapping,
        field_specs: HashMap<Box<str>, FieldSpec>,
    ) -> Self {
        Self {
            enabled,
            use_emoji,
            emoji_markers,
            promotion_tags,
            status_mappings,
            temporal_specs,
            field_specs,
        }
    }
}

/// Task status type (aligned with Tasks plugin).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum StatusType {
    /// Task is pending and not yet started.
    Todo,
    /// Task is actively being worked on.
    InProgress,
    /// Task is temporarily paused or blocked.
    OnHold,
    /// Task has been completed.
    Done,
    /// Task was cancelled and will not be completed.
    Cancelled,
    /// Item is not actually a task (e.g., a note or heading).
    NonTask,
}

/// Fully specified status mapping entry.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct StatusSpec {
    name: StatusName,
    symbol: StatusSymbol,
    status_type: StatusType,
    next_symbol: StatusSymbol,
}

impl StatusSpec {
    /// Creates a new status specification.
    #[inline]
    #[must_use]
    pub fn new(
        name: StatusName,
        symbol: StatusSymbol,
        status_type: StatusType,
        next_symbol: StatusSymbol,
    ) -> Self {
        Self {
            name,
            symbol,
            status_type,
            next_symbol,
        }
    }

    /// Returns the status name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &StatusName {
        &self.name
    }

    /// Returns the checkbox symbol.
    #[inline]
    #[must_use]
    pub fn symbol(&self) -> StatusSymbol {
        self.symbol
    }

    /// Returns the status type.
    #[inline]
    #[must_use]
    pub fn status_type(&self) -> StatusType {
        self.status_type
    }

    /// Returns the next symbol in the cycle.
    #[inline]
    #[must_use]
    pub fn next_symbol(&self) -> StatusSymbol {
        self.next_symbol
    }
}

/// Bi-directional mapping between status names and checkbox symbols.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct CheckboxStatus {
    /// Forward mapping (name -> status spec).
    by_name: HashMap<StatusName, StatusSpec>,
    /// Reverse mapping (symbol -> status spec).
    by_symbol: HashMap<StatusSymbol, StatusSpec>,
}

impl CheckboxStatus {
    #[inline]
    /// Build status mappings from raw name/symbol data.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if mappings are invalid.
    pub fn try_from_raw(
        raw: HashMap<String, RawStatusSpec>,
    ) -> Result<Self, ConfigError> {
        let mut by_name = HashMap::new();
        let mut by_symbol = HashMap::new();

        let mut entries: Vec<_> = raw.into_iter().collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (name, spec) in entries {
            let status_name = StatusName::try_new(name.clone())?;
            let status_symbol = StatusSymbol::try_new(spec.symbol)?;
            let next_symbol = if let Some(next) = spec.next_symbol {
                StatusSymbol::try_new(next)?
            } else {
                status_symbol
            };
            let status_spec = StatusSpec::new(
                status_name.clone(),
                status_symbol,
                spec.status_type,
                next_symbol,
            );

            if by_name.contains_key(&status_name) {
                return Err(ConfigError::ValidationFailed {
                    field: "task.status".into(),
                    message: "duplicate status name".into(),
                });
            }
            if by_symbol.contains_key(&status_symbol) {
                return Err(ConfigError::ValidationFailed {
                    field: "task.status".into(),
                    message: "duplicate status symbol".into(),
                });
            }

            by_name.insert(status_name, status_spec.clone());
            by_symbol.insert(status_symbol, status_spec);
        }

        if by_name.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".into(),
                message: "status mapping cannot be empty".into(),
            });
        }

        Ok(Self {
            by_name,
            by_symbol,
        })
    }

    #[inline]
    #[must_use]
    /// Look up the symbol for a status name.
    pub fn symbol_for_name(&self, name: &StatusName) -> Option<StatusSymbol> {
        self.by_name.get(name).map(StatusSpec::symbol)
    }

    #[inline]
    #[must_use]
    /// Look up the status name for a symbol.
    pub fn name_for_symbol(&self, symbol: StatusSymbol) -> Option<&StatusName> {
        self.by_symbol.get(&symbol).map(StatusSpec::name)
    }

    #[inline]
    #[must_use]
    /// Returns an iterator over the status name and symbol pairs.
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, StatusName, StatusSpec> {
        self.by_name.iter()
    }
}

impl<'status> IntoIterator for &'status CheckboxStatus {
    type IntoIter =
        std::collections::hash_map::Iter<'status, StatusName, StatusSpec>;
    type Item = (&'status StatusName, &'status StatusSpec);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Validated status name (e.g., `complete`).
///
/// # Invariants
///
/// - Must be 1-32 characters long.
/// - Must be ASCII alphanumeric or `_`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct StatusName(Box<str>);

impl StatusName {
    #[inline]
    /// Creates a validated status name.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty, too
    /// long, or contains non-alphanumeric characters.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.is_empty() || text.len() > 32 {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".into(),
                message: "status name must be 1-32 characters"
                    .to_owned()
                    .into(),
            });
        }
        if !text.chars().all(|c: char| c.is_ascii_alphanumeric() || c == '_') {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".into(),
                message: "status name must be ASCII alphanumeric or '_'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(text.to_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the status name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<StatusName> for String {
    #[inline]
    fn from(name: StatusName) -> Self {
        name.0.into_string()
    }
}
/// Validated status symbol (e.g., `x`).
///
/// # Invariants
///
/// - Must be a printable ASCII character.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct StatusSymbol(u8);

impl StatusSymbol {
    #[inline]
    /// Creates a validated status symbol.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the symbol is not a
    /// printable ASCII character.
    pub fn try_new(value: char) -> Result<Self, ConfigError> {
        if (value == ' ' || value.is_ascii_graphic()) && value.is_ascii() {
            let byte = u8::try_from(value).map_err(|e| {
                ConfigError::ValidationFailed {
                    field: "task.status".into(),
                    message: format!("invalid status symbol: {e}").into(),
                }
            })?;
            return Ok(Self(byte));
        }
        Err(ConfigError::ValidationFailed {
            field: "task.status".into(),
            message: "status symbol must be printable ASCII".into(),
        })
    }

    #[inline]
    #[must_use]
    /// Return the underlying status symbol.
    pub fn value(self) -> char {
        char::from(self.0)
    }
}

/// Validated task tag marker (e.g., `#task`).
///
/// # Invariants
///
/// - Must start with `#`.
/// - Must be at least 2 characters long.
/// - Segments are separated by `/`.
/// - Segments must be alphanumeric, `_`, or `-`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TaskTag(Box<str>);

impl TaskTag {
    #[inline]
    /// Creates a validated task tag.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the tag does not start with
    /// `#` or contains invalid characters.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        let Some(tag_body) = text.strip_prefix('#') else {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".into(),
                message: "task tag must start with '#' and be non-empty"
                    .to_owned()
                    .into(),
            });
        };
        if tag_body.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".into(),
                message: "task tag must be non-empty".to_owned().into(),
            });
        }

        for segment in tag_body.split('/') {
            if segment.is_empty() {
                return Err(ConfigError::ValidationFailed {
                    field: "task_tags".into(),
                    message: "task tag cannot contain empty segments"
                        .to_owned()
                        .into(),
                });
            }
            if !segment
                .chars()
                .all(|c: char| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(ConfigError::ValidationFailed {
                    field: "task_tags".into(),
                    message: "task tag segments must be alphanumeric, '_' or \
                              '-'"
                    .to_owned()
                    .into(),
                });
            }
        }

        if tag_body.chars().any(|c| c.is_ascii() && c.is_ascii_whitespace()) {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".into(),
                message: "task tag must not contain whitespace"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(text.to_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the tag as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<TaskTag> for String {
    #[inline]
    fn from(tag: TaskTag) -> Self {
        tag.0.into_string()
    }
}

// ----------------------------------------------------------- //
//               Standard Trait Implementations                //
// ----------------------------------------------------------- //

impl TryFrom<RawTaskConfig> for Task {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTaskConfig) -> Result<Self, Self::Error> {
        Self::try_from_raw(raw)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use super::*;
    use crate::config::raw::RawIndexingConfig;

    mod fixtures {
        use super::*;
        use crate::config::raw::{
            RawDateFieldSpec, RawFieldSpec, RawTaskDates,
        };

        pub fn sample_raw_task_config() -> RawTaskConfig {
            let mut fields = HashMap::new();
            fields.insert("priority".to_owned(), RawFieldSpec::Integer {
                min: Some(0),
                max: Some(5),
            });
            RawTaskConfig {
                enabled: Some(true),
                task_tags: Some(vec!["#task".to_owned()]),
                status: None,
                dates: Some(RawTaskDates {
                    due: Some(RawDateFieldSpec {
                        keyword: "due".to_owned(),
                        emoji: Some('\u{1f4c5}'),
                        format: "%Y-%m-%d".to_owned(),
                    }),
                    ..Default::default()
                }),
                fields: Some(fields),
                indexing: None,
                dependencies: None,
                use_emoji: None,
            }
        }
    }

    #[test]
    fn task_tag_accepts_valid_prefix() {
        let tag = TaskTag::try_new("#work");
        assert!(
            tag.is_ok(),
            "TaskTag '#work' should be valid, but got: {tag:?}"
        );
    }

    #[test]
    fn task_tag_accepts_hierarchical() {
        let tag = TaskTag::try_new("#work/project");
        assert!(
            tag.is_ok(),
            "TaskTag '#work/project' should be valid, but got: {tag:?}"
        );
    }

    #[test]
    fn task_tag_accepts_unicode_segments() {
        let tag = TaskTag::try_new("#\u{8a08}\u{753b}/\u{6b21}");
        assert!(
            tag.is_ok(),
            "TaskTag '#\u{8a08}\u{753b}/\u{6b21}' should be valid, but got: \
             {tag:?}"
        );
    }

    #[test]
    fn task_tag_rejects_missing_hash_prefix() {
        let tag = TaskTag::try_new("work");
        assert!(tag.is_err(), "TaskTag without hash prefix should be invalid");
    }

    #[test]
    fn task_tag_rejects_empty_segments() {
        let tag = TaskTag::try_new("#work//proj");
        assert!(tag.is_err(), "TaskTag should reject empty segments");
    }

    #[test]
    fn status_mapping_rejects_duplicates() {
        let mut mapping = HashMap::new();
        mapping.insert("todo".to_owned(), RawStatusSpec {
            symbol: ' ',
            status_type: StatusType::Todo,
            next_symbol: None,
        });
        mapping.insert("other".to_owned(), RawStatusSpec {
            symbol: ' ',
            status_type: StatusType::Todo,
            next_symbol: None,
        });

        let result = CheckboxStatus::try_from_raw(mapping);
        assert!(
            result.is_err(),
            "CheckboxStatus should reject duplicate symbols"
        );
    }

    #[test]
    fn task_config_from_raw_is_enabled_by_default() {
        let raw = fixtures::sample_raw_task_config();
        let config =
            Task::try_from_raw(raw).expect("Task::from_raw should succeed");
        assert!(config.enabled(), "Task processing should be enabled");
    }

    #[test]
    fn task_config_from_raw_parses_tags() {
        let raw = fixtures::sample_raw_task_config();
        let config =
            Task::try_from_raw(raw).expect("Task::from_raw should succeed");
        let tags_len = config.tags().len();
        assert_eq!(tags_len, 1, "Expected 1 task tag, got {tags_len}");
    }

    #[test]
    fn task_config_from_raw_parses_due_field() {
        let raw = fixtures::sample_raw_task_config();
        let config =
            Task::try_from_raw(raw).expect("Task::from_raw should succeed");
        assert_eq!(
            config.due().expect("due field should exist").keyword().as_str(),
            "due",
            "Expected 'due' keyword"
        );
    }

    #[test]
    fn task_config_from_raw_parses_fields() {
        let raw = fixtures::sample_raw_task_config();
        let config =
            Task::try_from_raw(raw).expect("Task::from_raw should succeed");
        let fields_len = config.fields().len();
        assert_eq!(fields_len, 1, "Expected 1 custom field, got {fields_len}");
    }

    #[test]
    fn task_config_from_raw_rejects_invalid_bounds() {
        use crate::config::raw::RawFieldSpec;
        let mut raw = fixtures::sample_raw_task_config();
        let mut fields = HashMap::new();
        fields.insert("invalid".to_owned(), RawFieldSpec::Integer {
            min: Some(10),
            max: Some(0), // min > max
        });
        raw.fields = Some(fields);

        let result = Task::try_from_raw(raw);
        assert!(
            result.is_err(),
            "Expected validation error for invalid field bounds"
        );
    }

    #[test]
    fn task_config_rejects_unknown_indexed_field() {
        let raw = RawTaskConfig {
            enabled: None,
            task_tags: None,
            status: None,
            dates: None,
            fields: None,
            indexing: Some(RawIndexingConfig {
                indexed_fields: Some(vec!["unknown".to_owned()]),
            }),
            dependencies: None,
            use_emoji: None,
        };

        let result = Task::try_from_raw(raw);
        assert!(
            result.is_err(),
            "Expected validation error for unknown indexed field"
        );
    }
}
