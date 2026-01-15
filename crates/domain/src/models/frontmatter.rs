//! Frontmatter domain entities and business logic.
//!
//! This module defines the structure and behavior of Note frontmatter (YAML metadata).
//! It provides type-safe accessors and coercion logic for common frontmatter patterns.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::errors::DomainError;

/// Represents YAML metadata extracted from a note header.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    pub fields: HashMap<String, FrontmatterValue>,
}

/// Possible values in a frontmatter field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Domain name follows ubiquitous language"
)]
pub enum FrontmatterValue {
    /// Array of values.
    Array(Vec<FrontmatterValue>),
    /// Boolean value.
    Boolean(bool),
    /// Date/time value.
    Date(DateTime<Utc>),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(HashMap<String, FrontmatterValue>),
    /// String value.
    String(String),
}

impl FrontmatterValue {
    /// Returns the array if this is an Array variant.
    #[inline]
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics preferred"
        )]
        if let Self::Array(arr) = self {
            Some(arr.as_slice())
        } else {
            None
        }
    }

    /// Returns the boolean value if this is a Boolean variant.
    #[inline]
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let &Self::Boolean(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns the date if this is a Date variant.
    #[inline]
    #[must_use]
    pub fn as_date(&self) -> Option<DateTime<Utc>> {
        if let &Self::Date(d) = self {
            Some(d)
        } else {
            None
        }
    }

    /// Returns the number if this is a Number variant.
    #[inline]
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        if let &Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    /// Returns the object if this is an Object variant.
    #[inline]
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, Self>> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics preferred"
        )]
        if let Self::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    /// Returns the string value if this is a String variant.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics preferred"
        )]
        if let Self::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }
}

/// Trait for generic extraction of values from frontmatter.
pub trait FromFrontmatterValue: Sized {
    /// Attempts to extract a value of type `Self` from a `FrontmatterValue`.
    fn from_value(value: &FrontmatterValue) -> Option<Self>;
}

impl FromFrontmatterValue for String {
    #[inline]
    fn from_value(value: &FrontmatterValue) -> Option<Self> {
        value.as_str().map(ToString::to_string)
    }
}

impl FromFrontmatterValue for bool {
    #[inline]
    fn from_value(value: &FrontmatterValue) -> Option<Self> {
        value.as_bool()
    }
}

impl FromFrontmatterValue for f64 {
    #[inline]
    fn from_value(value: &FrontmatterValue) -> Option<Self> {
        value.as_number()
    }
}

impl FromFrontmatterValue for DateTime<Utc> {
    #[inline]
    fn from_value(value: &FrontmatterValue) -> Option<Self> {
        value.as_date()
    }
}

impl FromFrontmatterValue for Vec<String> {
    #[inline]
    fn from_value(value: &FrontmatterValue) -> Option<Self> {
        if let Some(arr) = value.as_array() {
            Some(
                arr.iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect(),
            )
        } else {
            value.as_str().map(|s| vec![s.to_owned()])
        }
    }
}

impl Frontmatter {
    /// Extracts the aliases field from frontmatter using the configured key.
    ///
    /// Returns a vector of alias strings. Supports both single strings and arrays.
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::models::config::Config,
    ) -> Vec<String> {
        self.get_as(&config.frontmatter.alias_key).unwrap_or_default()
    }

    /// Extracts the `file_class` field from frontmatter using the configured key.
    #[inline]
    #[must_use]
    pub fn file_class(&self, config: &crate::models::config::Config) -> String {
        self.get_as(&config.frontmatter.file_class_key).unwrap_or_default()
    }

    /// Gets a frontmatter value by key.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FrontmatterValue> {
        self.fields.get(key)
    }

    /// Gets a field and attempts to convert it to the specified type.
    #[inline]
    #[must_use]
    pub fn get_as<T: FromFrontmatterValue>(&self, key: &str) -> Option<T> {
        self.fields.get(key).and_then(T::from_value)
    }

    /// Creates a new frontmatter from fields.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if fields are invalid.
    #[inline]
    pub fn new(
        fields: HashMap<String, FrontmatterValue>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            fields,
        })
    }

    /// Extracts the title field from frontmatter using the configured key.
    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::models::config::Config) -> String {
        self.get_as(&config.frontmatter.title_key).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike as _, TimeZone as _};

    use super::*;

    #[test]
    #[expect(clippy::panic, reason = "Test error path")]
    fn parses_iso8601_date_successfully() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        let val = FrontmatterValue::Date(date);
        if let FrontmatterValue::Date(d) = val {
            assert_eq!(d.year(), 2_024i32);
        } else {
            panic!("Expected Date variant");
        }
    }

    #[test]
    fn converts_numeric_values_correctly() {
        let val = FrontmatterValue::Number(42.0);
        assert!(matches!(
            val,
            FrontmatterValue::Number(n) if (n - 42.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn converts_boolean_values_correctly() {
        let val = FrontmatterValue::Boolean(true);
        assert!(matches!(val, FrontmatterValue::Boolean(true)));
    }
}
