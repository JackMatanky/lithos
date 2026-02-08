//! Task configuration schema and validation.
//!
//! Task configuration is cross-cutting infrastructure used by the note context.

use std::collections::{HashMap, HashSet};

use regex::Regex;

use super::{
    error::ConfigError,
    raw::{
        RawDateFieldSpec, RawIndexingConfig, RawTaskConfig, RawTaskDates,
        RawTaskFieldSpec,
    },
};

/// Validated task tag for promotion (e.g., "#task").
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
#[rkyv(derive(Debug, Hash, Eq, PartialEq))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct TaskTag(
    /// Internal tag storage.
    Box<str>,
);

/// Validated keyword used in task metadata (e.g., "priority").
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
#[rkyv(derive(Debug, Hash, Eq, PartialEq))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct TaskFieldKeyword(
    /// Internal keyword storage.
    Box<str>,
);

/// Semantic status name (e.g., "complete").
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
#[rkyv(derive(Debug, Hash, Eq, PartialEq))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct StatusName(
    /// Internal status name storage.
    Box<str>,
);

/// Single-character status symbol used in markdown checkboxes.
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
#[rkyv(derive(Debug, Hash, Eq, PartialEq))]
#[serde(try_from = "char", into = "char")]
#[non_exhaustive]
pub struct StatusSymbol(
    /// Internal status symbol storage.
    char,
);

/// Numeric bounds for integer/float task fields.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub enum Bounds<T> {
    /// No bounds constraint.
    Unbounded,
    /// Minimum value constraint.
    Min(
        /// Minimum allowed value.
        T,
    ),
    /// Maximum value constraint.
    Max(
        /// Maximum allowed value.
        T,
    ),
    /// Inclusive minimum and maximum constraints.
    Range {
        /// Minimum allowed value.
        min: T,
        /// Maximum allowed value.
        max: T,
    },
}

/// First-class date field specification (due, created, reminder, completed).
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
#[non_exhaustive]
pub struct DateFieldSpec {
    /// Field keyword (e.g., "due").
    keyword: TaskFieldKeyword,
    /// Optional emoji marker (e.g., "📅").
    emoji: Option<char>,
    /// Chrono-compatible format string.
    format: Box<str>,
}

/// Validated task field specification.
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
#[non_exhaustive]
pub enum TaskFieldSpec {
    /// String field with optional regex pattern.
    String {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Optional regex pattern constraint.
        pattern: Option<Box<str>>,
    },
    /// Integer field with numeric bounds.
    Integer {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Bounds for allowed values.
        bounds: Bounds<i64>,
    },
    /// Floating-point field with numeric bounds.
    Float {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Bounds for allowed values.
        bounds: Bounds<f64>,
    },
    /// Enum field with allowed values.
    Enum {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Allowed values.
        values: Vec<Box<str>>,
    },
    /// Date/time field with format string.
    DateTime {
        /// Field keyword.
        keyword: TaskFieldKeyword,
        /// Chrono datetime format.
        format: Box<str>,
    },
}

/// Bidirectional status mapping between names and symbols.
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
#[non_exhaustive]
pub struct CheckboxStatus {
    /// Mapping from status name to symbol.
    by_name: HashMap<StatusName, StatusSymbol>,
    /// Mapping from status symbol to name.
    by_symbol: HashMap<StatusSymbol, StatusName>,
}

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

impl TaskTag {
    #[inline]
    /// Create a validated task tag.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the tag is not a valid task
    /// marker.
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
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
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "task_tags".to_owned().into(),
                message: "task tag must be ASCII alphanumeric, '_' or '-'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(value))
    }

    #[inline]
    #[must_use]
    /// Return the tag as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TaskFieldKeyword {
    #[inline]
    /// Create a validated task field keyword.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the keyword is invalid.
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        let text = value.as_ref();
        if text.is_empty() || text.len() > 64 {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.keyword".to_owned().into(),
                message: "keyword must be 1-64 characters".to_owned().into(),
            });
        }
        if !text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "task.fields.keyword".to_owned().into(),
                message: "keyword must be ASCII alphanumeric, '_' or '-'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(value))
    }

    #[inline]
    #[must_use]
    /// Return the keyword as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StatusName {
    #[inline]
    /// Create a validated status name.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the name is invalid.
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        let text = value.as_ref();
        if text.is_empty() || text.len() > 32 {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".to_owned().into(),
                message: "status name must be 1-32 characters"
                    .to_owned()
                    .into(),
            });
        }
        if !text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(ConfigError::ValidationFailed {
                field: "task.status".to_owned().into(),
                message: "status name must be ASCII alphanumeric or '_'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(value))
    }

    #[inline]
    #[must_use]
    /// Return the status name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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

impl<T: PartialOrd + Copy> Bounds<T> {
    #[inline]
    /// Build bounds from optional min/max values.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if min is greater than max.
    pub fn try_from_options(
        min: Option<T>,
        max: Option<T>,
        field: &'static str,
    ) -> Result<Self, ConfigError> {
        match (min, max) {
            (None, None) => Ok(Self::Unbounded),
            (Some(min), None) => Ok(Self::Min(min)),
            (None, Some(max)) => Ok(Self::Max(max)),
            (Some(min), Some(max)) => {
                if min <= max {
                    Ok(Self::Range {
                        min,
                        max,
                    })
                } else {
                    Err(ConfigError::ValidationFailed {
                        field: field.to_owned().into(),
                        message: "min must be <= max".to_owned().into(),
                    })
                }
            }
        }
    }

    #[inline]
    #[must_use]
    /// Return true when the value satisfies the bounds.
    pub fn validate(&self, value: T) -> bool {
        match *self {
            Bounds::Unbounded => true,
            Bounds::Min(min) => value >= min,
            Bounds::Max(max) => value <= max,
            Bounds::Range {
                min,
                max,
            } => value >= min && value <= max,
        }
    }
}

impl DateFieldSpec {
    #[inline]
    /// Build a date field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the format is invalid.
    pub fn from_raw(raw: RawDateFieldSpec) -> Result<Self, ConfigError> {
        let keyword = TaskFieldKeyword::try_new(raw.keyword)?;
        validate_chrono_format(&raw.format, "task.dates.format")?;
        Ok(Self {
            keyword,
            emoji: raw.emoji,
            format: raw.format.into(),
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

impl TaskFieldSpec {
    #[inline]
    /// Build a task field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(raw: RawTaskFieldSpec) -> Result<Self, ConfigError> {
        match raw {
            RawTaskFieldSpec::Enum {
                keyword,
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
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let values =
                    values.into_iter().map(String::into_boxed_str).collect();
                Ok(Self::Enum {
                    keyword,
                    values,
                })
            }
            RawTaskFieldSpec::Integer {
                keyword,
                min,
                max,
            } => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let bounds = Bounds::try_from_options(min, max, "task.fields")?;
                Ok(Self::Integer {
                    keyword,
                    bounds,
                })
            }
            RawTaskFieldSpec::Float {
                keyword,
                min,
                max,
            } => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let bounds = Bounds::try_from_options(min, max, "task.fields")?;
                Ok(Self::Float {
                    keyword,
                    bounds,
                })
            }
            RawTaskFieldSpec::DateTime {
                keyword,
                format,
            } => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                validate_chrono_format(&format, "task.fields.format")?;
                Ok(Self::DateTime {
                    keyword,
                    format: format.into(),
                })
            }
            RawTaskFieldSpec::String {
                keyword,
                pattern,
            } => {
                let keyword = TaskFieldKeyword::try_new(keyword)?;
                let pattern = match pattern {
                    Some(pattern) => {
                        if pattern.len() > 256 {
                            return Err(ConfigError::ValidationFailed {
                                field: "task.fields.pattern".to_owned().into(),
                                message: "pattern too long".to_owned().into(),
                            });
                        }
                        Regex::new(&pattern).map_err(|error| {
                            ConfigError::ValidationFailed {
                                field: "task.fields.pattern".to_owned().into(),
                                message: error.to_string().into(),
                            }
                        })?;
                        Some(pattern.into_boxed_str())
                    }
                    None => None,
                };
                Ok(Self::String {
                    keyword,
                    pattern,
                })
            }
        }
    }

    #[inline]
    #[must_use]
    /// Return the field keyword.
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
    pub fn validate_raw_value(
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
                pattern,
            } => Self::validate_string(value, keyword, pattern.as_deref()),
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
                min: min_bound(bounds).map(|v| v.to_string().into_boxed_str()),
                max: max_bound(bounds).map(|v| v.to_string().into_boxed_str()),
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
                min: min_bound(bounds).map(|v| v.to_string().into_boxed_str()),
                max: max_bound(bounds).map(|v| v.to_string().into_boxed_str()),
            });
        }
        Ok(())
    }

    fn validate_string(
        value: &serde_json::Value,
        keyword: &TaskFieldKeyword,
        pattern: Option<&str>,
    ) -> Result<(), ConfigError> {
        let text = value.as_str().ok_or_else(|| ConfigError::InvalidType {
            field: keyword.as_str().to_owned().into(),
            expected: "string".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if let Some(pattern) = pattern {
            let regex = Regex::new(pattern).map_err(|error| {
                ConfigError::ValidationFailed {
                    field: keyword.as_str().to_owned().into(),
                    message: error.to_string().into(),
                }
            })?;
            if !regex.is_match(text) {
                return Err(ConfigError::ValidationFailed {
                    field: keyword.as_str().to_owned().into(),
                    message: "pattern mismatch".to_owned().into(),
                });
            }
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
            expected: "string (enum)".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        if !values.iter().any(|entry| entry.as_ref() == text) {
            return Err(ConfigError::InvalidEnumValue {
                field: keyword.as_str().to_owned().into(),
                value: text.to_owned().into(),
                allowed: values.iter().map(ToString::to_string).collect(),
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
            expected: "string (datetime)".to_owned().into(),
            actual: value_type(value).into(),
        })?;
        parse_datetime_value(text, format, keyword.as_str())?;
        Ok(())
    }
}

impl TaskConfig {
    #[inline]
    /// Build a task config from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError` if configuration validation fails.
    pub fn from_raw(raw: RawTaskConfig) -> Result<Self, ConfigError> {
        let mut config = TaskConfig::default();

        if let Some(enabled) = raw.enabled {
            config.enabled = enabled;
        }

        if let Some(tags) = raw.task_tags {
            let mut parsed = Vec::with_capacity(tags.len());
            for tag in tags {
                parsed.push(TaskTag::try_new(tag)?);
            }
            if parsed.is_empty() {
                return Err(ConfigError::ValidationFailed {
                    field: "task.task_tags".to_owned().into(),
                    message: "task_tags cannot be empty".to_owned().into(),
                });
            }
            config.task_tags = parsed;
        }

        if let Some(status) = raw.status {
            config.status = CheckboxStatus::from_raw(status)?;
        }

        if let Some(dates) = raw.dates {
            config.apply_dates(dates)?;
        }

        if let Some(fields) = raw.fields {
            let mut parsed = HashMap::new();
            let mut entries: Vec<_> = fields.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (name, spec) in entries {
                let field_spec = TaskFieldSpec::from_raw(spec)?;
                parsed.insert(name.into_boxed_str(), field_spec);
            }
            config.fields = parsed;
        }

        if let Some(indexing) = raw.indexing {
            config.apply_indexing(indexing);
        }

        config.validate_indexed_fields()?;
        Ok(config)
    }

    #[inline]
    #[must_use]
    /// Return whether task parsing is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    #[must_use]
    /// Return configured task tags.
    pub fn task_tags(&self) -> &[TaskTag] {
        &self.task_tags
    }

    #[inline]
    #[must_use]
    /// Return true if a string contains any configured task tag.
    pub fn has_task_tag(&self, text: &str) -> bool {
        self.task_tags.iter().any(|tag| text.contains(tag.as_str()))
    }

    #[inline]
    #[must_use]
    /// Return checkbox status mappings.
    pub fn status(&self) -> &CheckboxStatus {
        &self.status
    }

    #[inline]
    #[must_use]
    /// Return the due date field spec, if set.
    pub fn due_field(&self) -> Option<&DateFieldSpec> {
        self.due_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the created date field spec, if set.
    pub fn created_field(&self) -> Option<&DateFieldSpec> {
        self.created_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the reminder date field spec, if set.
    pub fn reminder_field(&self) -> Option<&DateFieldSpec> {
        self.reminder_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the completed date field spec, if set.
    pub fn completed_field(&self) -> Option<&DateFieldSpec> {
        self.completed_field.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return a field spec by name.
    pub fn field_spec(&self, field_name: &str) -> Option<&TaskFieldSpec> {
        self.fields.get(field_name)
    }

    #[inline]
    #[must_use]
    /// Return the list of indexed field names.
    pub fn indexed_fields(&self) -> &[Box<str>] {
        &self.indexed_fields
    }

    /// Parse a date value using a date field spec.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if parsing fails.
    #[inline]
    pub fn parse_date_value(
        &self,
        text: &str,
        spec: &DateFieldSpec,
    ) -> Result<chrono::NaiveDateTime, ConfigError> {
        parse_datetime_value(text, spec.format(), "task.dates")
    }

    fn apply_dates(&mut self, raw: RawTaskDates) -> Result<(), ConfigError> {
        self.due_field = raw.due.map(DateFieldSpec::from_raw).transpose()?;
        self.created_field =
            raw.created.map(DateFieldSpec::from_raw).transpose()?;
        self.reminder_field =
            raw.reminder.map(DateFieldSpec::from_raw).transpose()?;
        self.completed_field =
            raw.completed.map(DateFieldSpec::from_raw).transpose()?;
        Ok(())
    }

    fn apply_indexing(&mut self, raw: RawIndexingConfig) {
        if let Some(fields) = raw.indexed_fields {
            let mut seen = HashSet::new();
            let mut indexed = Vec::new();
            for field in fields {
                if seen.insert(field.as_str().to_owned()) {
                    indexed.push(field.into_boxed_str());
                }
            }
            self.indexed_fields = indexed;
        }
    }

    fn validate_indexed_fields(&self) -> Result<(), ConfigError> {
        for field in &self.indexed_fields {
            if !self.fields.contains_key(field) {
                return Err(ConfigError::ValidationFailed {
                    field: "task.indexing.indexed_fields".to_owned().into(),
                    message: format!("unknown field '{field}'").into(),
                });
            }
        }
        Ok(())
    }
}

impl Default for TaskConfig {
    #[inline]
    fn default() -> Self {
        let mut status = HashMap::new();
        status.insert("complete".to_owned(), 'x');
        status.insert("incomplete".to_owned(), ' ');
        status.insert("cancelled".to_owned(), '-');

        let status = CheckboxStatus::from_raw(status).unwrap_or_else(|_| {
            let mut by_name = HashMap::new();
            let mut by_symbol = HashMap::new();
            by_name.insert(StatusName("complete".into()), StatusSymbol('x'));
            by_symbol.insert(StatusSymbol('x'), StatusName("complete".into()));
            CheckboxStatus {
                by_name,
                by_symbol,
            }
        });

        let task_tags = vec![
            TaskTag::try_new("#task")
                .unwrap_or_else(|_| TaskTag("#task".into())),
        ];

        Self {
            enabled: false,
            task_tags,
            status,
            due_field: None,
            created_field: None,
            reminder_field: None,
            completed_field: None,
            fields: HashMap::new(),
            indexed_fields: Vec::new(),
        }
    }
}

impl TryFrom<String> for TaskTag {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<TaskTag> for String {
    #[inline]
    fn from(value: TaskTag) -> Self {
        value.0.into()
    }
}

impl TryFrom<String> for TaskFieldKeyword {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<TaskFieldKeyword> for String {
    #[inline]
    fn from(value: TaskFieldKeyword) -> Self {
        value.0.into()
    }
}

impl TryFrom<String> for StatusName {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<StatusName> for String {
    #[inline]
    fn from(value: StatusName) -> Self {
        value.0.into()
    }
}

impl TryFrom<char> for StatusSymbol {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<StatusSymbol> for char {
    #[inline]
    fn from(value: StatusSymbol) -> Self {
        value.0
    }
}

impl TryFrom<RawTaskConfig> for TaskConfig {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: RawTaskConfig) -> Result<Self, Self::Error> {
        Self::from_raw(value)
    }
}

fn validate_chrono_format(
    format: &str,
    field: &'static str,
) -> Result<(), ConfigError> {
    let items = chrono::format::strftime::StrftimeItems::new(format);
    for item in items {
        if matches!(item, chrono::format::Item::Error) {
            return Err(ConfigError::ValidationFailed {
                field: field.to_owned().into(),
                message: "invalid chrono format".to_owned().into(),
            });
        }
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

fn min_bound<T: Copy>(bounds: &Bounds<T>) -> Option<T> {
    match *bounds {
        Bounds::Unbounded | Bounds::Max(_) => None,
        Bounds::Min(value) => Some(value),
        Bounds::Range {
            min,
            ..
        } => Some(min),
    }
}

fn max_bound<T: Copy>(bounds: &Bounds<T>) -> Option<T> {
    match *bounds {
        Bounds::Unbounded | Bounds::Min(_) => None,
        Bounds::Max(value) => Some(value),
        Bounds::Range {
            max,
            ..
        } => Some(max),
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
mod tests {
    use super::*;

    #[test]
    fn task_tag_requires_hash_prefix() {
        let result = TaskTag::try_new("task");
        assert!(result.is_err(), "Expected validation error");
    }

    #[test]
    fn status_mapping_rejects_duplicates() {
        let mut raw = HashMap::new();
        raw.insert("complete".to_owned(), 'x');
        raw.insert("done".to_owned(), 'x');

        let result = CheckboxStatus::from_raw(raw);
        assert!(result.is_err(), "Expected validation error");
    }

    #[test]
    fn bounds_rejects_min_greater_than_max() {
        let result = Bounds::try_from_options(Some(10i64), Some(2i64), "field");
        assert!(result.is_err(), "Expected validation error");
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
                indexed_fields: Some(vec!["priority".to_owned()]),
            }),
        };

        let result = TaskConfig::from_raw(raw);
        assert!(result.is_err(), "Expected validation error");
    }
}
