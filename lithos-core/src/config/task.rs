//! Task configuration schema and validation.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums for #[non_exhaustive] \
              source enums"
)]
#![expect(
    missing_docs,
    reason = "rkyv generates undocumented archived struct fields"
)]
#![allow(
    clippy::allow_attributes,
    clippy::missing_trait_methods,
    dead_code,
    reason = "Internal validation helpers and clippy compatibility"
)]

use std::{collections::HashMap, sync::Arc};

use regex::Regex;

use super::{
    error::ConfigError,
    raw::{RawDateFieldSpec, RawTaskConfig, RawTaskFieldSpec},
};
use crate::bounds::Bounds;

/// Validated task configuration aggregate.
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[serde(try_from = "RawTaskConfig")]
#[non_exhaustive]
pub struct TaskConfig {
    /// Whether task processing is enabled.
    enabled: bool,
    /// Configured task promotion tags.
    task_tags: Vec<TaskTag>,
    /// Status mappings for checkboxes.
    status: CheckboxStatus,
    /// Optional due date field configuration.
    due_field: Option<DateFieldSpec>,
    /// Optional created date field configuration.
    created_field: Option<DateFieldSpec>,
    /// Optional reminder date field configuration.
    reminder_field: Option<DateFieldSpec>,
    /// Optional completed date field configuration.
    completed_field: Option<DateFieldSpec>,
    /// Custom task field specifications.
    fields: HashMap<Box<str>, TaskFieldSpec>,
    /// List of field names to be indexed.
    indexed_fields: Vec<Box<str>>,
}

/// Validated status name (e.g., `complete`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct StatusName(Box<str>);

impl StatusName {
    #[inline]
    /// Create a validated status name.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the name is empty or invalid.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.is_empty() || text.len() > 32 {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".to_owned().into(),
                message: "status name must be 1-32 characters"
                    .to_owned()
                    .into(),
            });
        }
        if !text.chars().all(|c: char| c.is_ascii_alphanumeric() || c == '_') {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".to_owned().into(),
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
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct StatusSymbol(char);

impl StatusSymbol {
    #[inline]
    /// Create a validated status symbol.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the symbol is invalid.
    pub fn try_new(value: char) -> Result<Self, ConfigError> {
        if value == ' ' || value.is_ascii_graphic() {
            return Ok(Self(value));
        }
        Err(ConfigError::ValidationFailed {
            field: "task.status".to_owned().into(),
            message: "status symbol must be printable ASCII".to_owned().into(),
        })
    }

    #[inline]
    #[must_use]
    /// Return the underlying status symbol.
    pub fn value(self) -> char {
        self.0
    }
}

/// Checkbox status mappings.
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
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct CheckboxStatus {
    by_name: HashMap<StatusName, StatusSymbol>,
    by_symbol: HashMap<StatusSymbol, StatusName>,
}

impl CheckboxStatus {
    #[inline]
    /// Build status mappings from raw name/symbol data.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if mappings are invalid.
    pub fn from_raw(raw: HashMap<String, char>) -> Result<Self, ConfigError> {
        let mut by_name = HashMap::new();
        let mut by_symbol = HashMap::new();

        let mut entries: Vec<_> = raw.into_iter().collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (name, symbol) in entries {
            let status_name = StatusName::try_new(name.clone())?;
            let status_symbol = StatusSymbol::try_new(symbol)?;

            if by_name.contains_key(&status_name) {
                return Err(ConfigError::ValidationFailed {
                    field: "task.status".to_owned().into(),
                    message: "duplicate status name".to_owned().into(),
                });
            }
            if by_symbol.contains_key(&status_symbol) {
                return Err(ConfigError::ValidationFailed {
                    field: "task.status".to_owned().into(),
                    message: "duplicate status symbol".to_owned().into(),
                });
            }

            by_name.insert(status_name.clone(), status_symbol);
            by_symbol.insert(status_symbol, status_name);
        }

        if by_name.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".to_owned().into(),
                message: "status mapping cannot be empty".to_owned().into(),
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
        self.by_name.get(name).copied()
    }

    #[inline]
    #[must_use]
    /// Look up the status name for a symbol.
    pub fn name_for_symbol(&self, symbol: StatusSymbol) -> Option<&StatusName> {
        self.by_symbol.get(&symbol)
    }
}

/// Validated task tag marker (e.g., `#task`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TaskTag(Box<str>);

impl TaskTag {
    #[inline]
    /// Create a validated task tag.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the tag is not a valid task
    /// marker.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.len() < 2 || !text.starts_with('#') {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".to_owned().into(),
                message: "task tag must start with '#' and be non-empty"
                    .to_owned()
                    .into(),
            });
        }
        if !text
            .chars()
            .skip(1)
            .all(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".to_owned().into(),
                message: "task tag must be ASCII alphanumeric, '_' or '-'"
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

/// Field keyword used in task text (e.g., `due:`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, Hash, PartialEq, Eq))]
pub struct TaskFieldKeyword(Box<str>);

impl TaskFieldKeyword {
    #[inline]
    /// Create a validated task field keyword.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the keyword is invalid.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.is_empty() || text.len() > 64 {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.keyword".to_owned().into(),
                message: "keyword must be 1-64 characters".to_owned().into(),
            });
        }
        if !text
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.keyword".to_owned().into(),
                message: "keyword must be ASCII alphanumeric, '_' or '-'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(text.to_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the keyword as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<TaskFieldKeyword> for String {
    #[inline]
    fn from(keyword: TaskFieldKeyword) -> Self {
        keyword.0.into_string()
    }
}

/// Date field specification.
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct DateFieldSpec {
    /// Keyword used in text.
    keyword: TaskFieldKeyword,
    /// Optional emoji marker.
    emoji: Option<char>,
    /// Chrono format string.
    format: Box<str>,
}

impl DateFieldSpec {
    #[inline]
    /// Build a date field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(raw: RawDateFieldSpec) -> Result<Self, ConfigError> {
        let keyword = TaskFieldKeyword::try_new(raw.keyword)?;
        validate_chrono_format(&raw.format, "task.dates.format")?;
        Ok(Self {
            keyword,
            emoji: raw.emoji,
            format: raw.format.into_boxed_str(),
        })
    }

    #[inline]
    #[must_use]
    /// Return the field keyword.
    pub fn keyword(&self) -> &TaskFieldKeyword {
        &self.keyword
    }

    #[inline]
    #[must_use]
    /// Return the optional emoji marker.
    pub fn emoji(&self) -> Option<char> {
        self.emoji
    }

    #[inline]
    #[must_use]
    /// Return the chrono format string.
    pub fn format(&self) -> &str {
        &self.format
    }
}

/// Custom task field specification.
#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum TaskFieldSpec {
    /// Integer field.
    Integer {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Optional bounds.
        bounds: Bounds<i64>,
    },
    /// Float field.
    Float {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Optional bounds.
        bounds: Bounds<f64>,
    },
    /// String field.
    String {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Optional validation pattern.
        pattern: Option<String>,
        /// Pre-compiled regex pattern for validation.
        #[rkyv(with = rkyv::with::Skip)]
        #[serde(skip)]
        compiled: Option<Arc<Regex>>,
    },
    /// Enumerated field.
    Enum {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// List of allowed values.
        values: Vec<Box<str>>,
    },
    /// Date time field.
    DateTime {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Chrono format string.
        format: String,
    },
}

impl PartialEq for TaskFieldSpec {
    #[inline]
    #[expect(clippy::pattern_type_mismatch, reason = "Enum pattern matching")]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Integer {
                    keyword: k1,
                    bounds: b1,
                },
                Self::Integer {
                    keyword: k2,
                    bounds: b2,
                },
            ) => k1 == k2 && b1 == b2,
            (
                Self::Float {
                    keyword: k1,
                    bounds: b1,
                },
                Self::Float {
                    keyword: k2,
                    bounds: b2,
                },
            ) => k1 == k2 && b1 == b2,
            (
                Self::String {
                    keyword: k1,
                    pattern: p1,
                    ..
                },
                Self::String {
                    keyword: k2,
                    pattern: p2,
                    ..
                },
            ) => k1 == k2 && p1 == p2,
            (
                Self::Enum {
                    keyword: k1,
                    values: v1,
                },
                Self::Enum {
                    keyword: k2,
                    values: v2,
                },
            ) => k1 == k2 && v1 == v2,
            (
                Self::DateTime {
                    keyword: k1,
                    format: f1,
                },
                Self::DateTime {
                    keyword: k2,
                    format: f2,
                },
            ) => k1 == k2 && f1 == f2,
            _ => false,
        }
    }
}

impl Eq for TaskFieldSpec {
    #[inline]
    fn assert_receiver_is_total_eq(&self) {
        // Default implementation is fine
    }
}

impl TaskFieldSpec {
    #[inline]
    #[expect(clippy::too_many_lines, reason = "Complex ingestion logic")]
    /// Build a task field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(raw: RawTaskFieldSpec) -> Result<Self, ConfigError> {
        match raw {
            RawTaskFieldSpec::Enum(crate::config::raw::RawEnumFieldSpec {
                keyword,
                values,
            }) => {
                if values.is_empty() {
                    return Err(ConfigError::ValidationFailed {
                        field: "task.fields.values".to_owned().into(),
                        message: "enum values cannot be empty"
                            .to_owned()
                            .into(),
                    });
                }
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let values =
                    values.into_iter().map(String::into_boxed_str).collect();
                Ok(Self::Enum {
                    keyword,
                    values,
                })
            }
            RawTaskFieldSpec::Integer(
                crate::config::raw::RawIntegerFieldSpec {
                    keyword,
                    min,
                    max,
                },
            ) => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "task.fields".to_owned().into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Integer {
                    keyword,
                    bounds,
                })
            }
            RawTaskFieldSpec::Float(
                crate::config::raw::RawFloatFieldSpec {
                    keyword,
                    min,
                    max,
                },
            ) => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "task.fields".to_owned().into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Float {
                    keyword,
                    bounds,
                })
            }
            RawTaskFieldSpec::DateTime(
                crate::config::raw::RawDateTimeFieldSpec {
                    keyword,
                    format,
                },
            ) => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                validate_chrono_format(&format, "task.fields.format")?;
                Ok(Self::DateTime {
                    keyword,
                    format,
                })
            }
            RawTaskFieldSpec::String(
                crate::config::raw::RawStringFieldSpec {
                    keyword,
                    pattern,
                },
            ) => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let mut compiled = None;
                if let Some(pattern_str) = pattern.as_ref() {
                    if pattern_str.len() > 256 {
                        return Err(ConfigError::ValidationFailed {
                            field: "task.fields.pattern".to_owned().into(),
                            message: "pattern too long".to_owned().into(),
                        });
                    }
                    let regex = Regex::new(pattern_str).map_err(|error| {
                        ConfigError::ValidationFailed {
                            field: "task.fields.pattern".to_owned().into(),
                            message: error.to_string().into(),
                        }
                    })?;
                    compiled = Some(Arc::new(regex));
                }
                Ok(Self::String {
                    keyword,
                    pattern,
                    compiled,
                })
            }
        }
    }

    #[inline]
    #[must_use]
    /// Return the field keyword.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Ergonomic enum pattern for accessing shared field"
    )]
    pub fn keyword(&self) -> &TaskFieldKeyword {
        match self {
            Self::String {
                keyword,
                ..
            }
            | Self::Integer {
                keyword,
                ..
            }
            | Self::Float {
                keyword,
                ..
            }
            | Self::Enum {
                keyword,
                ..
            }
            | Self::DateTime {
                keyword,
                ..
            } => keyword,
        }
    }

    #[inline]
    /// Validate a raw JSON value against this spec.
    ///
    /// # Errors
    /// Returns `ConfigError` if the value does not match the spec.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Ergonomic enum pattern for validation dispatch"
    )]
    pub(crate) fn validate_raw_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), ConfigError> {
        match self {
            Self::Integer {
                keyword,
                bounds,
            } => Self::validate_integer(value, keyword, bounds),
            Self::Float {
                keyword,
                bounds,
            } => Self::validate_float(value, keyword, bounds),
            Self::String {
                keyword,
                compiled,
                ..
            } => Self::validate_string(value, keyword, compiled.as_deref()),
            Self::Enum {
                keyword,
                values,
            } => Self::validate_enum(value, keyword, values),
            Self::DateTime {
                keyword,
                format,
            } => Self::validate_datetime(value, keyword, format),
        }
    }

    fn validate_integer(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        bounds: &Bounds<i64>,
    ) -> Result<(), ConfigError> {
        let number =
            value.as_i64().ok_or_else(|| ConfigError::InvalidType {
                field: keyword.as_str().to_owned().into(),
                expected: "integer".to_owned().into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: keyword.as_str().to_owned().into(),
                value: number.to_string().into(),
                min: bounds.min().map(|v| v.to_string().into()),
                max: bounds.max().map(|v| v.to_string().into()),
            });
        }
        Ok(())
    }

    fn validate_float(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        bounds: &Bounds<f64>,
    ) -> Result<(), ConfigError> {
        let number =
            value.as_f64().ok_or_else(|| ConfigError::InvalidType {
                field: keyword.as_str().to_owned().into(),
                expected: "float".to_owned().into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: keyword.as_str().to_owned().into(),
                value: number.to_string().into(),
                min: bounds.min().map(|v| v.to_string().into()),
                max: bounds.max().map(|v| v.to_string().into()),
            });
        }
        Ok(())
    }

    fn validate_string(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        pattern: Option<&Regex>,
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: keyword.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if let Some(regex) = pattern
            && !regex.is_match(text)
        {
            return Err(ConfigError::ValidationFailed {
                field: keyword.as_str().to_owned().into(),
                message: "pattern mismatch".to_owned().into(),
            });
        }
        Ok(())
    }

    fn validate_enum(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        values: &[Box<str>],
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: keyword.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if !values.iter().any(|v| v.as_ref() == text) {
            return Err(ConfigError::InvalidEnumValue {
                field: keyword.as_str().to_owned().into(),
                value: text.to_owned().into(),
                allowed: values
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>(),
            });
        }
        Ok(())
    }

    fn validate_datetime(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        format: &str,
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: keyword.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        parse_datetime_value(text, format, keyword.as_str())?;
        Ok(())
    }
}

impl TryFrom<RawTaskConfig> for TaskConfig {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTaskConfig) -> Result<Self, Self::Error> {
        Self::from_raw(raw)
    }
}

impl TaskConfig {
    #[inline]
    /// Build a validated task configuration from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the configuration is invalid.
    pub fn from_raw(raw: RawTaskConfig) -> Result<Self, ConfigError> {
        let enabled = raw.enabled.unwrap_or(true);
        let task_tags = match raw.task_tags {
            Some(tags) => tags
                .into_iter()
                .map(TaskTag::try_new)
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![TaskTag::try_new("#task")?],
        };

        let status = if let Some(mapping) = raw.status {
            CheckboxStatus::from_raw(mapping)?
        } else {
            let mut mapping = HashMap::new();
            mapping.insert("todo".to_owned(), ' ');
            mapping.insert("done".to_owned(), 'x');
            CheckboxStatus::from_raw(mapping)?
        };

        let (due_field, completed_field, created_field, reminder_field) =
            match raw.dates {
                Some(dates) => (
                    dates.due.map(DateFieldSpec::from_raw).transpose()?,
                    dates.completed.map(DateFieldSpec::from_raw).transpose()?,
                    dates.created.map(DateFieldSpec::from_raw).transpose()?,
                    dates.reminder.map(DateFieldSpec::from_raw).transpose()?,
                ),
                None => (None, None, None, None),
            };

        let mut fields = HashMap::new();
        if let Some(raw_fields) = raw.fields {
            let mut entries: Vec<_> = raw_fields.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (name, spec) in entries {
                fields.insert(
                    name.into_boxed_str(),
                    TaskFieldSpec::from_raw(spec)?,
                );
            }
        }

        let mut indexed_fields = Vec::new();
        if let Some(indexing) = raw.indexing
            && let Some(fields_list) = indexing.indexed_fields
        {
            for field_name in fields_list {
                if !fields.contains_key(field_name.as_str()) {
                    return Err(ConfigError::ValidationFailed {
                        field: "task.indexing.fields".to_owned().into(),
                        message: format!("unknown field: {field_name}").into(),
                    });
                }
                indexed_fields.push(field_name.into_boxed_str());
            }
        }

        Ok(Self {
            enabled,
            task_tags,
            status,
            due_field,
            created_field,
            reminder_field,
            completed_field,
            fields,
            indexed_fields,
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
    pub fn task_tags(&self) -> &[TaskTag] {
        &self.task_tags
    }

    #[inline]
    #[must_use]
    /// Return the checkbox status mappings.
    pub fn status(&self) -> &CheckboxStatus {
        &self.status
    }

    #[inline]
    #[must_use]
    /// Return the due date field spec, if configured.
    pub fn due_field(&self) -> Option<&DateFieldSpec> {
        self.due_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the created date field spec, if configured.
    pub fn created_field(&self) -> Option<&DateFieldSpec> {
        self.created_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the reminder date field spec, if configured.
    pub fn reminder_field(&self) -> Option<&DateFieldSpec> {
        self.reminder_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the completed date field spec, if configured.
    pub fn completed_field(&self) -> Option<&DateFieldSpec> {
        self.completed_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the custom fields map.
    pub fn fields(&self) -> &HashMap<Box<str>, TaskFieldSpec> {
        &self.fields
    }

    #[inline]
    #[must_use]
    /// Return the list of indexed field names.
    pub fn indexed_fields(&self) -> &[Box<str>] {
        &self.indexed_fields
    }

    #[inline]
    /// Look up a field specification by name.
    #[must_use]
    pub fn field_spec(&self, name: &str) -> Option<&TaskFieldSpec> {
        self.fields.get(name)
    }
}

impl Default for TaskConfig {
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        clippy::unwrap_used,
        reason = "Default config is guaranteed valid"
    )]
    fn default() -> Self {
        Self::from_raw(RawTaskConfig::default()).unwrap()
    }
}

fn validate_chrono_format(
    format: &str,
    field: &'static str,
) -> Result<(), ConfigError> {
    if format.is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: "format cannot be empty".to_owned().into(),
        });
    }
    // Simple verification that format is valid for chrono
    let now = chrono::Utc::now().naive_utc();
    if now.format(format).to_string().is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: "invalid chrono format".to_owned().into(),
        });
    }
    Ok(())
}

fn value_type(value: &serde_json::Value) -> &'static str {
    match *value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn parse_datetime_value(
    text: &str,
    format: &str,
    field: &str,
) -> Result<chrono::NaiveDateTime, ConfigError> {
    if let Ok(value) = chrono::NaiveDateTime::parse_from_str(text, format) {
        return Ok(value);
    }

    let date =
        chrono::NaiveDate::parse_from_str(text, format).map_err(|error| {
            ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: error.to_string().into(),
            }
        })?;

    date.and_hms_opt(0, 0, 0).ok_or_else(|| ConfigError::ValidationFailed {
        field: field.to_owned().into(),
        message: "invalid date time".to_owned().into(),
    })
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::disallowed_methods,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use super::*;
    use crate::{
        bounds::BoundsError,
        config::raw::{RawIndexingConfig, RawTaskDates},
    };

    mod fixtures {
        use super::*;

        pub fn sample_raw_task_config() -> RawTaskConfig {
            let mut fields = HashMap::new();
            fields.insert(
                "priority".to_owned(),
                RawTaskFieldSpec::Integer(
                    crate::config::raw::RawIntegerFieldSpec {
                        keyword: "priority".to_owned(),
                        min: Some(0),
                        max: Some(5),
                    },
                ),
            );
            RawTaskConfig {
                enabled: Some(true),
                task_tags: Some(vec!["#task".to_owned()]),
                status: None,
                dates: Some(RawTaskDates {
                    due: Some(crate::config::raw::RawDateFieldSpec {
                        keyword: "due".to_owned(),
                        emoji: Some('\u{1f4c5}'),
                        format: "%Y-%m-%d".to_owned(),
                    }),
                    ..Default::default()
                }),
                fields: Some(fields),
                indexing: None,
            }
        }
    }

    #[test]
    fn task_tag_requires_hash_prefix() {
        let tag1 = TaskTag::try_new("#work");
        tag1.unwrap();

        let tag2 = TaskTag::try_new("work");
        tag2.unwrap_err();
    }

    #[test]
    fn status_mapping_rejects_duplicates() {
        let mut mapping = HashMap::new();
        mapping.insert("todo".to_owned(), ' ');
        mapping.insert("other".to_owned(), ' '); // Duplicate symbol

        let result = CheckboxStatus::from_raw(mapping);
        result.unwrap_err();
    }

    #[test]
    fn task_config_from_raw_full_valid() {
        let raw = fixtures::sample_raw_task_config();
        let config = TaskConfig::from_raw(raw).unwrap();

        assert!(config.enabled());
        assert_eq!(config.task_tags().len(), 1);
        assert_eq!(config.due_field().unwrap().keyword().as_str(), "due");
        assert_eq!(config.fields().len(), 1);
    }

    #[test]
    fn task_config_from_raw_invalid_bounds() {
        let mut raw = fixtures::sample_raw_task_config();
        let mut fields = HashMap::new();
        fields.insert(
            "invalid".to_owned(),
            RawTaskFieldSpec::Integer(
                crate::config::raw::RawIntegerFieldSpec {
                    keyword: "invalid".to_owned(),
                    min: Some(10),
                    max: Some(0), // min > max
                },
            ),
        );
        raw.fields = Some(fields);

        let result = TaskConfig::from_raw(raw);
        assert!(
            result.is_err(),
            "Expected validation error for invalid bounds"
        );
    }

    #[test]
    fn bounds_rejects_min_greater_than_max() {
        // This is now tested in bounds.rs, but keeping a task-specific check
        let result = Bounds::from_options(Some(10i64), Some(0i64));
        assert!(matches!(result, Some(Err(BoundsError::InvalidRange))));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        clippy::shadow_unrelated,
        reason = "Test utilities"
    )]
    fn task_field_spec_type_inference() {
        let toml = r#"
keyword = "priority"
min = 0
max = 10
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should infer Integer type");
        assert!(matches!(spec, RawTaskFieldSpec::Integer(_)));

        let toml = r#"
keyword = "tags"
values = ["a", "b"]
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should infer Enum type");
        assert!(matches!(spec, RawTaskFieldSpec::Enum(_)));

        let toml = r#"
keyword = "due"
format = "%Y-%m-%d"
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should infer DateTime type");
        assert!(matches!(spec, RawTaskFieldSpec::DateTime(_)));

        let toml = r#"
keyword = "name"
pattern = "^[a-z]+$"
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should infer String type");
        assert!(matches!(spec, RawTaskFieldSpec::String(_)));

        let toml = r#"
keyword = "generic"
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should infer String type as fallback");
        assert!(matches!(spec, RawTaskFieldSpec::String(_)));
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
        };

        let result = TaskConfig::from_raw(raw);
        assert!(result.is_err(), "Expected validation error for unknown field");
    }
}
