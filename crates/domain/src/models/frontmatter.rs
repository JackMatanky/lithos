//! Frontmatter domain entities and business logic.
//!
//! This module defines the structure and behavior of Note frontmatter (YAML metadata).
//! It provides type-safe accessors and coercion logic for common frontmatter patterns.
//!
//! # Architecture Decision
//!
//! This module uses the same pattern as `serde_json::Value` for runtime type inspection.
//! The `FieldValue` enum supports unknown-type scenarios (inspect then extract) while
//! the `FromFieldValue` trait enables known-type scenarios (schema-driven extraction).

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::errors::DomainError;

/// Represents YAML metadata extracted from a note header.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    pub fields: HashMap<String, FieldValue>,
}

/// Possible values in a frontmatter field.
///
/// This enum represents the runtime type of a value parsed from YAML frontmatter.
/// It mirrors the design of `serde_json::Value` to support dynamic typing scenarios.
///
/// # Usage Patterns
///
/// **Pattern 1: Unknown Type (Runtime Inspection).**
/// ```
/// use lithos_domain::models::frontmatter::FieldValue;
///
/// # let value = FieldValue::String("test".to_string());
/// if value.is_string() {
///     println!("String: {}", value.as_str().unwrap());
/// } else if value.is_number() {
///     println!("Number: {}", value.as_number().unwrap());
/// }
/// ```
///
/// **Pattern 2: Known Type (Schema-Driven).**
/// ```
/// use lithos_domain::models::frontmatter::{FieldValue, FromFieldValue};
///
/// let value = FieldValue::String("test".to_string());
/// let extracted: Option<String> = FromFieldValue::from_value(&value);
/// assert_eq!(extracted, Some("test".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum FieldValue {
    /// Array of values.
    Array(Vec<FieldValue>),
    /// Boolean value.
    Boolean(bool),
    /// Date/time value.
    Date(DateTime<Utc>),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(HashMap<String, FieldValue>),
    /// String value.
    String(String),
}

impl FieldValue {
    /// Returns the array if this is an Array variant.
    ///
    /// Uses Rust 2018 match ergonomics (RFC 2005) for automatic reference binding.
    /// This is the same pattern used by `serde_json::Value::as_array()`.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    ///
    /// let arr = FieldValue::Array(vec![FieldValue::String("item".to_string())]);
    /// assert!(arr.as_array().is_some());
    ///
    /// let not_arr = FieldValue::Boolean(true);
    /// assert!(not_arr.as_array().is_none());
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics RFC 2005: self is &FieldValue, pattern binds &Vec automatically. This is idiomatic Rust (see serde_json::Value)"
    )]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(arr) => Some(arr),
            Self::Boolean(_)
            | Self::Date(_)
            | Self::Number(_)
            | Self::Object(_)
            | Self::String(_) => None,
        }
    }

    /// Returns the boolean value if this is a Boolean variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    ///
    /// let val = FieldValue::Boolean(true);
    /// assert_eq!(val.as_bool(), Some(true));
    ///
    /// let not_bool = FieldValue::String("true".to_string());
    /// assert_eq!(not_bool.as_bool(), None);
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: self is &FieldValue, pattern binds &bool, dereferenced to bool. Idiomatic Rust."
    )]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Array(_)
            | Self::Date(_)
            | Self::Number(_)
            | Self::Object(_)
            | Self::String(_) => None,
        }
    }

    /// Returns the date if this is a Date variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    /// use chrono::{DateTime, Utc, TimeZone};
    ///
    /// let date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    /// let val = FieldValue::Date(date);
    /// assert_eq!(val.as_date(), Some(date));
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: DateTime is Copy, so &DateTime is dereferenced to DateTime. Idiomatic."
    )]
    pub fn as_date(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Date(d) => Some(*d),
            Self::Array(_)
            | Self::Boolean(_)
            | Self::Number(_)
            | Self::Object(_)
            | Self::String(_) => None,
        }
    }

    /// Returns the number if this is a Number variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    ///
    /// let val = FieldValue::Number(42.0);
    /// assert_eq!(val.as_number(), Some(42.0));
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: f64 is Copy, so &f64 is dereferenced to f64. Idiomatic."
    )]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Array(_)
            | Self::Boolean(_)
            | Self::Date(_)
            | Self::Object(_)
            | Self::String(_) => None,
        }
    }

    /// Returns the object if this is an Object variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    /// use std::collections::HashMap;
    ///
    /// let mut obj = HashMap::new();
    /// obj.insert("key".to_string(), FieldValue::String("value".to_string()));
    /// let val = FieldValue::Object(obj.clone());
    /// assert_eq!(val.as_object(), Some(&obj));
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics RFC 2005: self is &FieldValue, pattern binds &HashMap automatically. Matches serde_json::Value."
    )]
    pub fn as_object(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Object(obj) => Some(obj),
            Self::Array(_)
            | Self::Boolean(_)
            | Self::Date(_)
            | Self::Number(_)
            | Self::String(_) => None,
        }
    }

    /// Returns the string value if this is a String variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    ///
    /// let val = FieldValue::String("hello".to_string());
    /// assert_eq!(val.as_str(), Some("hello"));
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: self is &FieldValue, pattern binds &String, coerced to &str. Idiomatic Rust."
    )]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Array(_)
            | Self::Boolean(_)
            | Self::Date(_)
            | Self::Number(_)
            | Self::Object(_) => None,
        }
    }

    /// Checks if this value is an Array variant.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::FieldValue;
    ///
    /// let val = FieldValue::Array(vec![]);
    /// assert!(val.is_array());
    /// assert!(!val.is_string());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this is a Boolean variant.
    #[inline]
    #[must_use]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns true if this is a Date variant.
    #[inline]
    #[must_use]
    pub const fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// Returns true if this is a Number variant.
    #[inline]
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns true if this is an Object variant.
    #[inline]
    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Returns true if this is a String variant.
    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
}

/// Trait for generic extraction of values from frontmatter fields.
///
/// This trait enables type-safe extraction when you know the expected type,
/// while still allowing runtime type inspection via `FieldValue` methods.
///
/// # Design Rationale
///
/// This trait exists for **known-type scenarios** where schema validation has already
/// determined the expected type. For **unknown-type scenarios**, use the `FieldValue`
/// methods (`is_*()`, `as_*()`) directly.
///
/// # Examples
///
/// **Known Type (Schema-Driven):**
/// ```
/// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
/// use std::collections::HashMap;
///
/// let mut fields = HashMap::new();
/// fields.insert("title".to_string(), FieldValue::String("Hello".to_string()));
/// let fm = Frontmatter::new(fields).unwrap();
///
/// // When schema says "title is string", use get_as:
/// let title: Option<String> = fm.get_as("title");
/// assert_eq!(title, Some("Hello".to_string()));
/// ```
///
/// **Unknown Type (Runtime Inspection):**
/// ```
/// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
/// use std::collections::HashMap;
///
/// let mut fields = HashMap::new();
/// fields.insert("mystery".to_string(), FieldValue::Number(42.0));
/// let fm = Frontmatter::new(fields).unwrap();
///
/// // When type is unknown, inspect then extract:
/// if let Some(value) = fm.get("mystery") {
///     if value.is_number() {
///         println!("It's a number: {}", value.as_number().unwrap());
///     }
/// }
/// ```
pub trait FromFieldValue: Sized {
    /// Attempts to extract a value of type `Self` from a `FieldValue`.
    ///
    /// Returns `None` if the value cannot be converted to the target type.
    fn from_value(value: &FieldValue) -> Option<Self>;
}

impl FromFieldValue for String {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_str().map(ToOwned::to_owned)
    }
}

impl FromFieldValue for bool {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_bool()
    }
}

impl FromFieldValue for f64 {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_number()
    }
}

impl FromFieldValue for DateTime<Utc> {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_date()
    }
}

impl FromFieldValue for Vec<String> {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        // Support both arrays and single strings (Obsidian compatibility)
        if let Some(arr) = value.as_array() {
            Some(
                arr.iter()
                    .filter_map(|v| v.as_str().map(ToOwned::to_owned))
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
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// # use std::collections::HashMap;
    /// # let mut fields = HashMap::new();
    /// # fields.insert("aliases".to_string(), FieldValue::String("My Alias".to_string()));
    /// # let frontmatter = Frontmatter::new(fields).unwrap();
    /// # let global = GlobalConfig::default();
    /// # let mut vault = VaultConfig::default();
    /// # vault.filesystem.vault_path = "/vault".to_string();
    /// # let config = Config::build(&global, vault).unwrap();
    /// let aliases = frontmatter.aliases(&config);
    /// assert_eq!(aliases, vec!["My Alias".to_string()]);
    /// ```
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::models::config::Config,
    ) -> Vec<String> {
        self.get_string_array(&config.frontmatter.alias_key).unwrap_or_default()
    }

    /// Extracts the `file_class` field from frontmatter using the configured key.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// # use std::collections::HashMap;
    /// # let mut fields = HashMap::new();
    /// # fields.insert("file_class".to_string(), FieldValue::String("note".to_string()));
    /// # let frontmatter = Frontmatter::new(fields).unwrap();
    /// # let global = GlobalConfig::default();
    /// # let mut vault = VaultConfig::default();
    /// # vault.filesystem.vault_path = "/vault".to_string();
    /// # let config = Config::build(&global, vault).unwrap();
    /// let file_class = frontmatter.file_class(&config);
    /// assert_eq!(file_class, "note");
    /// ```
    #[inline]
    #[must_use]
    pub fn file_class(&self, config: &crate::models::config::Config) -> String {
        self.get_str(&config.frontmatter.file_class_key)
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    }

    /// Gets a frontmatter value by key without type conversion.
    ///
    /// Use this when you need to inspect the type or handle multiple types.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("title".to_string(), FieldValue::String("My Note".to_string()));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// let value = fm.get("title");
    /// assert!(value.is_some());
    /// assert!(value.unwrap().is_string());
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }

    /// Gets a field and attempts to convert it to the specified type.
    ///
    /// Use this when you know the expected type (e.g., from schema validation).
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("priority".to_string(), FieldValue::Number(5.0));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// let priority: Option<f64> = fm.get_as("priority");
    /// assert_eq!(priority, Some(5.0));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_as<T: FromFieldValue>(&self, key: &str) -> Option<T> {
        self.fields.get(key).and_then(T::from_value)
    }

    /// Gets a boolean field value.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("published".to_string(), FieldValue::Boolean(true));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// assert_eq!(fm.get_bool("published"), Some(true));
    /// assert_eq!(fm.get_bool("missing"), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    /// Gets a date field value.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    /// use chrono::{DateTime, Utc, TimeZone};
    ///
    /// let date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    /// let mut fields = HashMap::new();
    /// fields.insert("created".to_string(), FieldValue::Date(date));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// assert_eq!(fm.get_date("created"), Some(date));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_date(&self, key: &str) -> Option<DateTime<Utc>> {
        self.get(key)?.as_date()
    }

    /// Gets a number field value.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("rating".to_string(), FieldValue::Number(4.5));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// assert_eq!(fm.get_number("rating"), Some(4.5));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_number()
    }

    /// Gets a string field value.
    ///
    /// Returns a reference to avoid allocation. Use `.map(ToOwned::to_owned)` if you need an owned `String`.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("title".to_string(), FieldValue::String("Hello".to_string()));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// assert_eq!(fm.get_str("title"), Some("Hello"));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    /// Gets an array of strings, with fallback to single string.
    ///
    /// This method handles both array fields and single string fields,
    /// making it useful for fields like "tags" or "aliases" that can be
    /// either format in Obsidian.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// // Array case
    /// let mut fields = HashMap::new();
    /// fields.insert("tags".to_string(), FieldValue::Array(vec![
    ///     FieldValue::String("rust".to_string()),
    ///     FieldValue::String("programming".to_string()),
    /// ]));
    /// let fm = Frontmatter::new(fields).unwrap();
    /// assert_eq!(fm.get_string_array("tags"), Some(vec!["rust".to_string(), "programming".to_string()]));
    ///
    /// // Single string case
    /// let mut fields2 = HashMap::new();
    /// fields2.insert("tag".to_string(), FieldValue::String("rust".to_string()));
    /// let fm2 = Frontmatter::new(fields2).unwrap();
    /// assert_eq!(fm2.get_string_array("tag"), Some(vec!["rust".to_string()]));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        let v = self.get(key)?;
        if let Some(arr) = v.as_array() {
            Some(
                arr.iter()
                    .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                    .collect(),
            )
        } else {
            v.as_str().map(|s| vec![s.to_owned()])
        }
    }

    /// Checks if a field exists in the frontmatter.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("title".to_string(), FieldValue::String("Hello".to_string()));
    /// let fm = Frontmatter::new(fields).unwrap();
    ///
    /// assert!(fm.has("title"));
    /// assert!(!fm.has("missing"));
    /// ```
    #[inline]
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Creates a new frontmatter from fields.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if fields are invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut fields = HashMap::new();
    /// fields.insert("title".to_string(), FieldValue::String("My Note".to_string()));
    /// let fm = Frontmatter::new(fields).unwrap();
    /// assert!(fm.has("title"));
    /// ```
    #[inline]
    pub fn new(
        fields: HashMap<String, FieldValue>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            fields,
        })
    }

    /// Extracts the title field from frontmatter using the configured key.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::models::frontmatter::{Frontmatter, FieldValue};
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// # use std::collections::HashMap;
    /// # let mut fields = HashMap::new();
    /// # fields.insert("title".to_string(), FieldValue::String("My Note".to_string()));
    /// # let frontmatter = Frontmatter::new(fields).unwrap();
    /// # let global = GlobalConfig::default();
    /// # let mut vault = VaultConfig::default();
    /// # vault.filesystem.vault_path = "/vault".to_string();
    /// # let config = Config::build(&global, vault).unwrap();
    /// let title = frontmatter.title(&config);
    /// assert_eq!(title, "My Note");
    /// ```
    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::models::config::Config) -> String {
        self.get_str(&config.frontmatter.title_key)
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike as _, TimeZone as _};

    use super::*;

    #[test]
    fn parses_iso8601_date_successfully() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        let val = FieldValue::Date(date);
        assert_eq!(val.as_date(), Some(date));
        assert_eq!(date.year(), 2_024i32);
    }

    #[test]
    fn converts_numeric_values_correctly() {
        let val = FieldValue::Number(42.0f64);
        assert_eq!(val.as_number(), Some(42.0f64));
        assert!(matches!(
            val,
            FieldValue::Number(n) if (n - 42.0f64).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn converts_boolean_values_correctly() {
        let val = FieldValue::Boolean(true);
        assert_eq!(val.as_bool(), Some(true));
        assert!(matches!(val, FieldValue::Boolean(true)));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture creation, unwrap is appropriate for test clarity"
    )]
    fn has_method_detects_field_presence() {
        let mut fields = HashMap::new();
        fields
            .insert("title".to_owned(), FieldValue::String("Test".to_owned()));
        let fm = Frontmatter::new(fields).unwrap();

        assert!(fm.has("title"));
        assert!(!fm.has("missing"));
    }

    #[test]
    fn is_methods_identify_variants() {
        let string_val = FieldValue::String("test".to_owned());
        let number_val = FieldValue::Number(42.0);
        let bool_val = FieldValue::Boolean(true);

        assert!(string_val.is_string());
        assert!(!string_val.is_number());

        assert!(number_val.is_number());
        assert!(!number_val.is_string());

        assert!(bool_val.is_bool());
        assert!(!bool_val.is_string());
    }
}
