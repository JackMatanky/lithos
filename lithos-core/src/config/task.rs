//! Task configuration schema and validation.
//!
//! This module provides the [`Task`] aggregate and supporting types
//! for defining how Markdown-based tasks are recognized and indexed.

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
pub struct Task {
    /// Whether task processing is enabled.
    enabled: bool,
    /// Configured task promotion tags.
    tags: Vec<TaskTag>,
    /// Status mappings for checkboxes.
    status: CheckboxStatus,
    /// Optional due date field configuration.
    due: Option<DateSpec>,
    /// Optional created date field configuration.
    created: Option<DateSpec>,
    /// Optional reminder date field configuration.
    reminder: Option<DateSpec>,
    /// Optional completed date field configuration.
    completed: Option<DateSpec>,
    /// Custom task field specifications.
    fields: HashMap<Box<str>, FieldSpec>,
    /// List of field names to be indexed.
    indexed: Vec<Box<str>>,
}

impl Default for Task {
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

impl Task {
    #[inline]
    /// Builds a validated task configuration from raw input.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the configuration
    /// invariants are violated (e.g., duplicate status symbols or unknown
    /// indexed fields).
    pub fn from_raw(raw: RawTaskConfig) -> Result<Self, ConfigError> {
        let enabled = raw.enabled.unwrap_or(true);
        let tags = match raw.task_tags {
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

        let (due, completed, created, reminder) = match raw.dates {
            Some(dates) => (
                dates.due.map(DateSpec::from_raw).transpose()?,
                dates.completed.map(DateSpec::from_raw).transpose()?,
                dates.created.map(DateSpec::from_raw).transpose()?,
                dates.reminder.map(DateSpec::from_raw).transpose()?,
            ),
            None => (None, None, None, None),
        };

        let mut fields = HashMap::new();
        if let Some(raw_fields) = raw.fields {
            let mut entries: Vec<_> = raw_fields.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (name, spec) in entries {
                fields.insert(
                    name.clone().into_boxed_str(),
                    FieldSpec::from_raw(&name, spec)?,
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
                        field: "task.indexing.fields".to_owned().into(),
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
            due,
            created,
            reminder,
            completed,
            fields,
            indexed,
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
    /// Look up a field specification by name.
    #[must_use]
    pub fn field_spec(&self, name: &str) -> Option<&FieldSpec> {
        self.fields.get(name)
    }
}

/// Custom task field specification.
///
/// Defines the type and validation rules for a specific task metadata field.
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
pub enum FieldSpec {
    /// Integer field with optional range bounds.
    Integer {
        /// Field name identifier.
        name: FieldName,
        /// Optional bounds.
        bounds: Bounds<i64>,
    },
    /// Floating point field with optional range bounds.
    Float {
        /// Field name identifier.
        name: FieldName,
        /// Optional bounds.
        bounds: Bounds<f64>,
    },
    /// String field with optional regex pattern validation.
    String {
        /// Field name identifier.
        name: FieldName,
        /// Optional validation pattern.
        pattern: Option<String>,
        /// Pre-compiled regex pattern for validation.
        #[rkyv(with = rkyv::with::Skip)]
        #[serde(skip)]
        compiled: Option<Arc<Regex>>,
    },
    /// Categorical field with a fixed set of allowed values.
    Enum {
        /// Field name identifier.
        name: FieldName,
        /// List of allowed values.
        values: Vec<Box<str>>,
    },
    /// Date/time field with a specific Chrono format.
    DateTime {
        /// Field name identifier.
        name: FieldName,
        /// Chrono format string.
        format: String,
    },
}

impl FieldSpec {
    #[inline]
    #[allow(clippy::too_many_lines, reason = "Complex ingestion logic")]
    /// Build a task field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(
        name: &str,
        raw: RawTaskFieldSpec,
    ) -> Result<Self, ConfigError> {
        let name = FieldName::try_new(name)?;
        match raw {
            RawTaskFieldSpec::Enum {
                values,
            } => {
                if values.is_empty() {
                    return Err(ConfigError::ValidationFailed {
                        field: "task.fields.values".to_owned().into(),
                        message: "enum values cannot be empty"
                            .to_owned()
                            .into(),
                    });
                }
                let values =
                    values.into_iter().map(String::into_boxed_str).collect();
                Ok(Self::Enum {
                    name,
                    values,
                })
            }
            RawTaskFieldSpec::Integer {
                min,
                max,
            } => {
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "task.fields".to_owned().into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Integer {
                    name,
                    bounds,
                })
            }
            RawTaskFieldSpec::Float {
                min,
                max,
            } => {
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "task.fields".to_owned().into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Float {
                    name,
                    bounds,
                })
            }
            RawTaskFieldSpec::DateTime {
                format,
            } => {
                validate_chrono_format(&format, "task.fields.format")?;
                Ok(Self::DateTime {
                    name,
                    format,
                })
            }
            RawTaskFieldSpec::String {
                pattern,
            } => {
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
                    name,
                    pattern,
                    compiled,
                })
            }
        }
    }

    #[inline]
    #[must_use]
    /// Return the spec name identifier.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Ergonomic enum pattern for accessing shared field"
    )]
    pub fn name(&self) -> &FieldName {
        match self {
            Self::String {
                name,
                ..
            }
            | Self::Integer {
                name,
                ..
            }
            | Self::Float {
                name,
                ..
            }
            | Self::Enum {
                name,
                ..
            }
            | Self::DateTime {
                name,
                ..
            } => name,
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
                name,
                bounds,
            } => Self::validate_integer(value, name, bounds),
            Self::Float {
                name,
                bounds,
            } => Self::validate_float(value, name, bounds),
            Self::String {
                name,
                compiled,
                ..
            } => Self::validate_string(value, name, compiled.as_deref()),
            Self::Enum {
                name,
                values,
            } => Self::validate_enum(value, name, values),
            Self::DateTime {
                name,
                format,
            } => Self::validate_datetime(value, name, format),
        }
    }

    fn validate_integer(
        value: &serde_json::Value,
        name: &FieldName,
        bounds: &Bounds<i64>,
    ) -> Result<(), ConfigError> {
        let number =
            value.as_i64().ok_or_else(|| ConfigError::InvalidType {
                field: name.as_str().to_owned().into(),
                expected: "integer".to_owned().into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: name.as_str().to_owned().into(),
                value: number.to_string().into(),
                min: bounds.min().map(|v| v.to_string().into()),
                max: bounds.max().map(|v| v.to_string().into()),
            });
        }
        Ok(())
    }

    fn validate_float(
        value: &serde_json::Value,
        name: &FieldName,
        bounds: &Bounds<f64>,
    ) -> Result<(), ConfigError> {
        let number =
            value.as_f64().ok_or_else(|| ConfigError::InvalidType {
                field: name.as_str().to_owned().into(),
                expected: "float".to_owned().into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: name.as_str().to_owned().into(),
                value: number.to_string().into(),
                min: bounds.min().map(|v| v.to_string().into()),
                max: bounds.max().map(|v| v.to_string().into()),
            });
        }
        Ok(())
    }

    fn validate_string(
        value: &serde_json::Value,
        name: &FieldName,
        pattern: Option<&Regex>,
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: name.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if let Some(regex) = pattern
            && !regex.is_match(text)
        {
            return Err(ConfigError::ValidationFailed {
                field: name.as_str().to_owned().into(),
                message: "pattern mismatch".to_owned().into(),
            });
        }
        Ok(())
    }

    fn validate_enum(
        value: &serde_json::Value,
        name: &FieldName,
        values: &[Box<str>],
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: name.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if !values.iter().any(|v| v.as_ref() == text) {
            return Err(ConfigError::InvalidEnumValue {
                field: name.as_str().to_owned().into(),
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
        name: &FieldName,
        format: &str,
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: name.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        parse_datetime_value(text, format, name.as_str())?;
        Ok(())
    }
}

// ----------------------------------------------------------- //
//                    Building Block Types                     //
// ----------------------------------------------------------- //

/// Bi-directional mapping between status names and checkbox symbols.
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
    /// Forward mapping (name -> symbol).
    by_name: HashMap<StatusName, StatusSymbol>,
    /// Reverse mapping (symbol -> name).
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

    #[inline]
    #[must_use]
    /// Returns an iterator over the status name and symbol pairs.
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, StatusName, StatusSymbol> {
        self.by_name.iter()
    }
}

impl<'status> IntoIterator for &'status CheckboxStatus {
    type IntoIter =
        std::collections::hash_map::Iter<'status, StatusName, StatusSymbol>;
    type Item = (&'status StatusName, &'status StatusSymbol);

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
    /// Creates a validated status name.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty, too
    /// long, or contains non-alphanumeric characters.
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
///
/// # Invariants
///
/// - Must be a printable ASCII character.
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
    /// Creates a validated status symbol.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the symbol is not a
    /// printable ASCII character.
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

/// Validated task tag marker (e.g., `#task`).
///
/// # Invariants
///
/// - Must start with `#`.
/// - Must be at least 2 characters long.
/// - Remaining characters must be ASCII alphanumeric, `_`, or `-`.
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
    /// Creates a validated task tag.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the tag does not start with
    /// `#` or contains invalid characters.
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

/// Field name identifier used in task text (e.g., `due:`).
///
/// # Invariants
///
/// - Must be 1-64 characters long.
/// - Must be ASCII alphanumeric, `_`, or `-`.
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
pub struct FieldName(Box<str>);

impl FieldName {
    #[inline]
    /// Creates a validated task spec name.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty,
    /// too long, or contains non-alphanumeric characters.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.is_empty() || text.len() > 64 {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.name".to_owned().into(),
                message: "field name must be 1-64 characters".to_owned().into(),
            });
        }
        if !text
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.name".to_owned().into(),
                message: "field name must be ASCII alphanumeric, '_' or '-'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(text.to_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the spec name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<FieldName> for String {
    #[inline]
    fn from(name: FieldName) -> Self {
        name.0.into_string()
    }
}

/// Validated date field specification.
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
pub struct DateSpec {
    /// Field name used in text.
    keyword: FieldName,
    /// Optional emoji marker (e.g., 📅).
    emoji: Option<char>,
    /// Chrono format string (e.g., `%Y-%m-%d`).
    format: Box<str>,
}

impl DateSpec {
    #[inline]
    /// Build a date field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(raw: RawDateFieldSpec) -> Result<Self, ConfigError> {
        let keyword = FieldName::try_new(raw.keyword)?;
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
    pub fn keyword(&self) -> &FieldName {
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

// ----------------------------------------------------------- //
//               Standard Trait Implementations                //
// ----------------------------------------------------------- //

impl TryFrom<RawTaskConfig> for Task {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTaskConfig) -> Result<Self, Self::Error> {
        Self::from_raw(raw)
    }
}

impl PartialEq for FieldSpec {
    #[inline]
    #[expect(clippy::pattern_type_mismatch, reason = "Enum pattern matching")]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Integer {
                    name: n1,
                    bounds: b1,
                },
                Self::Integer {
                    name: n2,
                    bounds: b2,
                },
            ) => n1 == n2 && b1 == b2,
            (
                Self::Float {
                    name: n1,
                    bounds: b1,
                },
                Self::Float {
                    name: n2,
                    bounds: b2,
                },
            ) => n1 == n2 && b1 == b2,
            (
                Self::String {
                    name: n1,
                    pattern: p1,
                    ..
                },
                Self::String {
                    name: n2,
                    pattern: p2,
                    ..
                },
            ) => n1 == n2 && p1 == p2,
            (
                Self::Enum {
                    name: n1,
                    values: v1,
                },
                Self::Enum {
                    name: n2,
                    values: v2,
                },
            ) => n1 == n2 && v1 == v2,
            (
                Self::DateTime {
                    name: n1,
                    format: f1,
                },
                Self::DateTime {
                    name: n2,
                    format: f2,
                },
            ) => n1 == n2 && f1 == f2,
            _ => false,
        }
    }
}

impl Eq for FieldSpec {
    #[inline]
    fn assert_receiver_is_total_eq(&self) {
        // Default implementation is fine
    }
}

// ----------------------------------------------------------- //
//                Low-Level Validation Helpers                 //
// ----------------------------------------------------------- //

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

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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
            fields.insert("priority".to_owned(), RawTaskFieldSpec::Integer {
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
        let config = Task::from_raw(raw).unwrap();

        assert!(config.enabled());
        assert_eq!(config.tags().len(), 1);
        assert_eq!(config.due().unwrap().keyword().as_str(), "due");
        assert_eq!(config.fields().len(), 1);
    }

    #[test]
    fn task_config_from_raw_invalid_bounds() {
        let mut raw = fixtures::sample_raw_task_config();
        let mut fields = HashMap::new();
        fields.insert("invalid".to_owned(), RawTaskFieldSpec::Integer {
            min: Some(10),
            max: Some(0), // min > max
        });
        raw.fields = Some(fields);

        let result = Task::from_raw(raw);
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
    fn task_field_spec_parses_typed_specs() {
        let toml = r#"
type = "integer"
min = 0
max = 10
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should parse Integer type");
        assert!(matches!(spec, RawTaskFieldSpec::Integer { .. }));

        let toml = r#"
type = "enum"
values = ["a", "b"]
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should parse Enum type");
        assert!(matches!(spec, RawTaskFieldSpec::Enum { .. }));

        let toml = r#"
type = "datetime"
format = "%Y-%m-%d"
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should parse DateTime type");
        assert!(matches!(spec, RawTaskFieldSpec::DateTime { .. }));

        let toml = r#"
type = "string"
pattern = "^[a-z]+$"
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should parse String type");
        assert!(matches!(spec, RawTaskFieldSpec::String { .. }));

        let toml = r#"
type = "float"
min = 0.0
max = 1.0
"#;
        let spec: RawTaskFieldSpec =
            toml::from_str(toml).expect("Should parse Float type");
        assert!(matches!(spec, RawTaskFieldSpec::Float { .. }));
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

        let result = Task::from_raw(raw);
        assert!(result.is_err(), "Expected validation error for unknown field");
    }
}
