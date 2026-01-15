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
    /// ```ignore
    /// let aliases = frontmatter.aliases(&config);
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
    /// ```ignore
    /// let file_class = frontmatter.file_class(&config);
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
    /// ```ignore
    /// let title = frontmatter.title(&config);
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
    use std::collections::HashMap;

    use chrono::{Datelike as _, TimeZone as _, Utc};

    use super::*;

    /// 3.2-UNIT-017: `FieldValue` Extraction - Date.
    /// P1.
    #[test]
    fn as_date_returns_date_when_variant_is_date() {
        // GIVEN a Date variant
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        let val = FieldValue::Date(date);

        // WHEN the date is extracted
        let result = val.as_date();

        // THEN it returns the correct date
        assert_eq!(result, Some(date));
        assert_eq!(date.year(), 2_024i32);
    }

    /// 3.2-UNIT-018: `FieldValue` Extraction - Number.
    /// P1.
    #[test]
    fn as_number_returns_float_when_variant_is_number() {
        // GIVEN a Number variant
        let val = FieldValue::Number(42.0f64);

        // WHEN the number is extracted
        let result = val.as_number();

        // THEN it returns the correct float
        assert_eq!(result, Some(42.0f64));
        assert!(matches!(
            val,
            FieldValue::Number(n) if (n - 42.0f64).abs() < f64::EPSILON
        ));
    }

    /// 3.2-UNIT-019: `FieldValue` Extraction - Boolean.
    /// P1.
    #[test]
    fn as_bool_returns_bool_when_variant_is_boolean() {
        // GIVEN a Boolean variant
        let val = FieldValue::Boolean(true);

        // WHEN the boolean is extracted
        let result = val.as_bool();

        // THEN it returns the correct value
        assert_eq!(result, Some(true));
        assert!(matches!(val, FieldValue::Boolean(true)));
    }

    /// 3.2-UNIT-020: Frontmatter Inspection - Field Presence.
    /// P1.
    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture creation, unwrap is appropriate for test clarity"
    )]
    fn frontmatter_has_returns_true_when_field_exists() {
        // GIVEN frontmatter with a 'title' field
        let mut fields = HashMap::new();
        fields
            .insert("title".to_owned(), FieldValue::String("Test".to_owned()));
        let fm = Frontmatter::new(fields).expect("Valid fields");

        // WHEN checking for field existence
        let has_title = fm.has("title");
        let has_missing = fm.has("missing");

        // THEN it correctly identifies present and missing fields
        assert!(has_title);
        assert!(!has_missing);
    }

    /// 3.2-UNIT-021: `FieldValue` Type Inspection - Variant Identification.
    /// P1.
    #[test]
    fn field_value_is_variant_returns_true_when_types_match() {
        // GIVEN various FieldValue variants
        let string_val = FieldValue::String("test".to_owned());
        let number_val = FieldValue::Number(42.0f64);
        let bool_val = FieldValue::Boolean(true);
        let array_val = FieldValue::Array(vec![]);
        let obj_val = FieldValue::Object(HashMap::new());
        let date_val = FieldValue::Date(
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        );

        // WHEN inspecting their types
        // THEN they correctly identify their own variants
        assert!(string_val.is_string());
        assert!(number_val.is_number());
        assert!(bool_val.is_bool());
        assert!(array_val.is_array());
        assert!(obj_val.is_object());
        assert!(date_val.is_date());
    }

    /// 3.2-UNIT-026: `FieldValue` Extraction - Complex Types.
    /// P1.
    #[test]
    fn as_methods_return_references_for_complex_variants() {
        // GIVEN Array and Object variants
        let mut obj_map = HashMap::new();
        obj_map.insert("k".to_owned(), FieldValue::Boolean(true));
        let obj = FieldValue::Object(obj_map.clone());
        let arr = FieldValue::Array(vec![FieldValue::Number(1.0f64)]);

        // WHEN extracting references
        let obj_ref = obj.as_object().expect("Not an object");
        let arr_ref = arr.as_array().expect("Not an array");

        // THEN they match the original data
        assert_eq!(obj_ref, &obj_map);
        assert_eq!(arr_ref.len(), 1);
    }

    /// 3.2-UNIT-027: `FromFieldValue` Implementation.
    /// P1.
    #[test]
    #[expect(clippy::similar_names, reason = "Test coverage")]
    fn from_field_value_converts_to_target_types() {
        // GIVEN various variants
        let string_val = FieldValue::String("val".into());
        let bool_val = FieldValue::Boolean(false);
        let num_val = FieldValue::Number(1.5f64);
        let arr = FieldValue::Array(vec![FieldValue::String("a".into())]);
        let mixed_arr = FieldValue::Array(vec![
            FieldValue::String("a".into()),
            FieldValue::Boolean(true),
        ]);

        // WHEN using the trait for extraction
        let res_s: Option<String> = FromFieldValue::from_value(&string_val);
        let res_b: Option<bool> = FromFieldValue::from_value(&bool_val);
        let res_n: Option<f64> = FromFieldValue::from_value(&num_val);
        let res_v: Option<Vec<String>> = FromFieldValue::from_value(&arr);
        let res_vs: Option<Vec<String>> =
            FromFieldValue::from_value(&string_val);
        let res_mixed: Option<Vec<String>> =
            FromFieldValue::from_value(&mixed_arr);

        // THEN conversions succeed
        assert_eq!(res_s, Some("val".to_owned()));
        assert_eq!(res_b, Some(false));
        assert_eq!(res_n, Some(1.5f64));
        assert_eq!(res_v, Some(vec!["a".to_owned()]));
        assert_eq!(res_vs, Some(vec!["val".to_owned()]));
        assert_eq!(res_mixed, Some(vec!["a".to_owned()]));

        // AND mismatched types return None
        assert!(<String as FromFieldValue>::from_value(&bool_val).is_none());
        assert!(<f64 as FromFieldValue>::from_value(&string_val).is_none());
        assert!(<bool as FromFieldValue>::from_value(&num_val).is_none());
        assert!(
            <DateTime<Utc> as FromFieldValue>::from_value(&string_val)
                .is_none()
        );
        assert!(
            <Vec<String> as FromFieldValue>::from_value(&bool_val).is_none()
        );
    }

    /// 3.2-UNIT-028: `Frontmatter` Accessors.
    /// P1.
    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test setup")]
    fn frontmatter_accessors_return_typed_values() {
        // GIVEN frontmatter with mixed fields
        let mut fields = HashMap::new();
        fields.insert("s".into(), FieldValue::String("str".into()));
        fields.insert("b".into(), FieldValue::Boolean(true));
        fields.insert("n".into(), FieldValue::Number(10.0f64));
        let date = Utc::now();
        fields.insert("d".into(), FieldValue::Date(date));
        let fm = Frontmatter::new(fields).expect("Valid fields");

        // WHEN using convenience accessors
        // THEN they return the expected types
        assert_eq!(fm.get_str("s"), Some("str"));
        assert_eq!(fm.get_bool("b"), Some(true));
        assert_eq!(fm.get_number("n"), Some(10.0f64));
        assert_eq!(fm.get_date("d"), Some(date));
        assert_eq!(fm.get_as::<String>("s"), Some("str".to_owned()));
    }

    /// 3.2-UNIT-029: `Frontmatter` Collection Accessors.
    /// P1.
    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test setup")]
    fn frontmatter_collection_accessors_handle_arrays_and_fallbacks() {
        // GIVEN frontmatter with arrays and single values
        let mut fields = HashMap::new();
        fields.insert(
            "tags".into(),
            FieldValue::Array(vec![
                FieldValue::String("t1".into()),
                FieldValue::Number(2.0f64), // Non-string in array
            ]),
        );
        fields.insert("alias".into(), FieldValue::String("a1".into()));
        let fm = Frontmatter::new(fields).expect("Valid fields");

        // WHEN extracting string arrays
        let tags = fm.get_string_array("tags").expect("Missing tags");
        let aliases = fm.get_string_array("alias").expect("Missing alias");

        // THEN it handles both cases correctly and filters non-strings
        assert_eq!(tags, vec!["t1".to_owned()]);
        assert_eq!(aliases, vec!["a1".to_owned()]);
    }

    /// 3.2-UNIT-030: `FieldValue` Extraction - Error Paths.
    /// P1.
    #[test]
    #[expect(
        clippy::many_single_char_names,
        reason = "Coverage exhausting variants"
    )]
    fn as_methods_return_none_when_variants_mismatch() {
        // GIVEN all variants
        let b = FieldValue::Boolean(true);
        let n = FieldValue::Number(1.0f64);
        let s = FieldValue::String(String::new());
        let d = FieldValue::Date(
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        );
        let o = FieldValue::Object(HashMap::new());
        let a = FieldValue::Array(vec![]);

        let variants = vec![&b, &n, &s, &d, &o, &a];

        // WHEN calling extraction methods on mismatched variants
        // THEN they all return None (exhaustively covering match arms)
        for v in variants {
            if !v.is_array() {
                assert!(v.as_array().is_none());
            }
            if !v.is_bool() {
                assert!(v.as_bool().is_none());
            }
            if !v.is_date() {
                assert!(v.as_date().is_none());
            }
            if !v.is_number() {
                assert!(v.as_number().is_none());
            }
            if !v.is_object() {
                assert!(v.as_object().is_none());
            }
            if !v.is_string() {
                assert!(v.as_str().is_none());
            }
        }
    }

    /// 3.2-UNIT-031: `Frontmatter` Config-based Extraction.
    /// P1.
    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test setup")]
    fn frontmatter_extracts_mapped_keys_from_config() {
        // GIVEN a config and frontmatter with mapped keys
        use crate::models::config::Config;
        let config = Config::default();
        let mut fields = HashMap::new();
        fields.insert("title".into(), FieldValue::String("My Title".into()));
        fields.insert("aliases".into(), FieldValue::String("alias1".into()));
        fields.insert("file_class".into(), FieldValue::String("task".into()));
        let fm = Frontmatter::new(fields).expect("Valid fields");

        // WHEN extracting using config-based methods
        // THEN it uses the correct keys from Config
        assert_eq!(fm.title(&config), "My Title");
        assert_eq!(fm.aliases(&config), vec!["alias1".to_owned()]);
        assert_eq!(fm.file_class(&config), "task");
    }
}
