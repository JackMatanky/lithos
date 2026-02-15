//! Field specification and value validation logic.
//!
//! This module provides types for defining and validating custom metadata
//! fields within the configuration.

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

use std::sync::Arc;

use regex::Regex;

use super::{
    error::ConfigError,
    raw::{RawDateFieldSpec, RawFieldSpec},
};
use crate::bounds::Bounds;

// ----------------------------------------------------------- //
//                     Public Domain Types                     //
// ----------------------------------------------------------- //

/// Custom field specification.
///
/// Defines the type and validation rules for a specific metadata field.
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
    /// Build a field spec from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the spec is invalid.
    pub fn from_raw(
        name: &str,
        raw: RawFieldSpec,
    ) -> Result<Self, ConfigError> {
        let name = FieldName::try_new(name)?;
        match raw {
            RawFieldSpec::Enum {
                values,
            } => {
                if values.is_empty() {
                    return Err(ConfigError::ValidationFailed {
                        field: "fields.values".into(),
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
            RawFieldSpec::Integer {
                min,
                max,
            } => {
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "fields".into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Integer {
                    name,
                    bounds,
                })
            }
            RawFieldSpec::Float {
                min,
                max,
            } => {
                let bounds = Bounds::from_options(min, max)
                    .transpose()
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "fields".into(),
                        message: e.to_string().into(),
                    })?
                    .unwrap_or(Bounds::Unbounded);
                Ok(Self::Float {
                    name,
                    bounds,
                })
            }
            RawFieldSpec::DateTime {
                format,
            } => {
                validate_chrono_format(&format, "fields.format")?;
                Ok(Self::DateTime {
                    name,
                    format,
                })
            }
            RawFieldSpec::String {
                pattern,
            } => {
                let mut compiled = None;
                if let Some(pattern_str) = pattern.as_ref() {
                    if pattern_str.len() > 256 {
                        return Err(ConfigError::ValidationFailed {
                            field: "fields.pattern".into(),
                            message: "pattern too long".into(),
                        });
                    }
                    let regex = Regex::new(pattern_str).map_err(|error| {
                        ConfigError::ValidationFailed {
                            field: "fields.pattern".into(),
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
                field: name.as_str().into(),
                expected: "integer".into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: name.as_str().into(),
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
                field: name.as_str().into(),
                expected: "float".into(),
                actual: value_type(value).into(),
            })?;
        if !bounds.validate(number) {
            return Err(ConfigError::OutOfRange {
                field: name.as_str().into(),
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
            field: name.as_str().into(),
            expected: "string".into(),
            actual: value_type(value).into(),
        })?;
        if let Some(regex) = pattern
            && !regex.is_match(text)
        {
            return Err(ConfigError::ValidationFailed {
                field: name.as_str().into(),
                message: "pattern mismatch".into(),
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
            field: name.as_str().into(),
            expected: "string".into(),
            actual: value_type(value).into(),
        })?;
        if !values.iter().any(|v| v.as_ref() == text) {
            return Err(ConfigError::InvalidEnumValue {
                field: name.as_str().into(),
                value: text.into(),
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
            field: name.as_str().into(),
            expected: "string".into(),
            actual: value_type(value).into(),
        })?;
        parse_datetime_value(text, format, name.as_str())?;
        Ok(())
    }
}

/// Field name identifier used in configuration (e.g., `due:`).
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
    /// Creates a validated field name.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty,
    /// too long, or contains non-alphanumeric characters.
    pub fn try_new<T: AsRef<str>>(value: T) -> Result<Self, ConfigError> {
        let text = value.as_ref();
        if text.is_empty() || text.len() > 64 {
            return Err(ConfigError::ValidationFailed {
                field: "fields.name".into(),
                message: "field name must be 1-64 characters".into(),
            });
        }
        if !text
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigError::ValidationFailed {
                field: "fields.name".into(),
                message: "field name must be ASCII alphanumeric, '_' or '-'"
                    .to_owned()
                    .into(),
            });
        }
        Ok(Self(text.to_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the field name as a string slice.
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
    emoji: Option<u32>,
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
            emoji: raw.emoji.map(u32::from),
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
        self.emoji.and_then(char::from_u32)
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

pub(crate) fn validate_chrono_format(
    format: &str,
    field: &'static str,
) -> Result<(), ConfigError> {
    if format.is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.into(),
            message: "format cannot be empty".into(),
        });
    }
    // Simple verification that format is valid for chrono
    let now = chrono::Utc::now().naive_utc();
    if now.format(format).to_string().is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.into(),
            message: "invalid chrono format".into(),
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
                field: field.into(),
                message: error.to_string().into(),
            }
        })?;

    date.and_hms_opt(0, 0, 0).ok_or_else(|| ConfigError::ValidationFailed {
        field: field.into(),
        message: "invalid date time".into(),
    })
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test modules have relaxed rules for unwrapping"
)]
mod tests {
    use super::*;

    #[test]
    fn field_name_accepts_valid_alphanumeric() {
        let name = FieldName::try_new("valid_name");
        assert!(
            name.is_ok(),
            "FieldName 'valid_name' should be valid, but got: {name:?}"
        );
    }

    #[test]
    fn field_name_accepts_hyphens_and_numbers() {
        let name = FieldName::try_new("valid-name-123");
        assert!(
            name.is_ok(),
            "FieldName 'valid-name-123' should be valid, but got: {name:?}"
        );
    }

    #[test]
    fn field_name_rejects_empty_string() {
        let name = FieldName::try_new("");
        assert!(name.is_err(), "FieldName with empty string should be invalid");
    }

    #[test]
    fn field_name_rejects_spaces_and_special_chars() {
        let name = FieldName::try_new("invalid name!");
        assert!(
            name.is_err(),
            "FieldName with spaces/special chars should be invalid"
        );
    }

    #[test]
    fn bounds_rejects_min_greater_than_max() {
        // This is now tested in bounds.rs, but keeping a check here for
        // FieldSpec context
        let result = Bounds::from_options(Some(10i64), Some(0i64));
        assert!(
            matches!(
                result,
                Some(Err(crate::bounds::BoundsError::InvalidRange))
            ),
            "Expected InvalidRange error when min > max, but got: {result:?}"
        );
    }

    #[test]
    fn field_spec_parses_integer_spec() {
        let toml_str = r#"
type = "integer"
min = 0
max = 10
"#;
        let spec: RawFieldSpec =
            toml::from_str(toml_str).expect("Should parse Integer type");
        assert!(
            matches!(spec, RawFieldSpec::Integer { .. }),
            "Expected Integer spec, got {spec:?}"
        );
    }

    #[test]
    fn field_spec_parses_enum_spec() {
        let toml_str = r#"
type = "enum"
values = ["a", "b"]
"#;
        let spec: RawFieldSpec =
            toml::from_str(toml_str).expect("Should parse Enum type");
        assert!(
            matches!(spec, RawFieldSpec::Enum { .. }),
            "Expected Enum spec, got {spec:?}"
        );
    }

    #[test]
    fn field_spec_parses_datetime_spec() {
        let toml_str = r#"
type = "datetime"
format = "%Y-%m-%d"
"#;
        let spec: RawFieldSpec =
            toml::from_str(toml_str).expect("Should parse DateTime type");
        assert!(
            matches!(spec, RawFieldSpec::DateTime { .. }),
            "Expected DateTime spec, got {spec:?}"
        );
    }

    #[test]
    fn field_spec_parses_string_spec() {
        let toml_str = r#"
type = "string"
pattern = "^[a-z]+$"
"#;
        let spec: RawFieldSpec =
            toml::from_str(toml_str).expect("Should parse String type");
        assert!(
            matches!(spec, RawFieldSpec::String { .. }),
            "Expected String spec, got {spec:?}"
        );
    }

    #[test]
    fn field_spec_parses_float_spec() {
        let toml_str = r#"
type = "float"
min = 0.0
max = 1.0
"#;
        let spec: RawFieldSpec =
            toml::from_str(toml_str).expect("Should parse Float type");
        assert!(
            matches!(spec, RawFieldSpec::Float { .. }),
            "Expected Float spec, got {spec:?}"
        );
    }
}
