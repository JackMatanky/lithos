//! Frontmatter domain entities and business logic.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates Archived types with public fields/variants; docs \
              TODO for new methods"
)]

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::error::NoteError;

/// Possible values in a frontmatter field.
///
/// This enum represents the runtime type of a value parsed from YAML
/// frontmatter. It mirrors the design of `serde_json::Value` to support dynamic
/// typing scenarios.
///
/// Note: `DateTime` stored as i64 timestamp for rkyv compatibility.
/// TODO: Recursive rkyv causes trait solver overflow - will implement custom
/// serialization later.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum FieldValue {
    /// Array of values.
    Array(Vec<FieldValue>),
    /// Boolean value.
    Boolean(bool),
    /// Date/time value (stored as Unix timestamp for serialization).
    Date(i64),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(HashMap<String, FieldValue>),
    /// String value.
    String(String),
}

/// Represents YAML metadata extracted from a note header.
///
/// TODO: Frontmatter rkyv support deferred - recursive `FieldValue` causes
/// trait solver overflow. For Phase 6, we'll serialize Note without frontmatter
/// or use serde fallback.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal builders and tests"
)]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    pub(crate) fields: HashMap<String, FieldValue>,
}

pub trait FromFieldValue: Sized {
    /// Attempts to extract a value of type `Self` from a `FieldValue`.
    ///
    /// Returns `None` if the value cannot be converted to the target type.
    fn from_value(value: &FieldValue) -> Option<Self>;
}

impl FromFieldValue for bool {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_bool()
    }
}

impl FromFieldValue for DateTime<Utc> {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        use chrono::TimeZone as _;
        let ts = value.as_date()?;
        Utc.timestamp_opt(ts, 0).single()
    }
}

impl FromFieldValue for f64 {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_number()
    }
}

impl FromFieldValue for String {
    #[inline]
    fn from_value(value: &FieldValue) -> Option<Self> {
        value.as_str().map(ToOwned::to_owned)
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

#[expect(
    clippy::pattern_type_mismatch,
    clippy::wildcard_enum_match_arm,
    reason = "Accessor methods intentionally use catch-all patterns for \
              forward compatibility"
)]
impl FieldValue {
    #[inline]
    #[must_use]
    pub fn as_array(&self) -> Option<&[FieldValue]> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_date(&self) -> Option<i64> {
        match self {
            Self::Date(timestamp) => Some(*timestamp),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_datetime(&self) -> Option<DateTime<Utc>> {
        use chrono::TimeZone as _;
        let ts = self.as_date()?;
        Utc.timestamp_opt(ts, 0).single()
    }

    #[inline]
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, FieldValue>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
}

impl Frontmatter {
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<String> {
        self.get_string_array(&config.frontmatter.alias_key).unwrap_or_default()
    }

    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> String {
        self.get_str(&config.frontmatter.file_class_key)
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }

    #[inline]
    #[must_use]
    pub fn get_as<T: FromFieldValue>(&self, key: &str) -> Option<T> {
        self.fields.get(key).and_then(T::from_value)
    }

    #[inline]
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    #[inline]
    #[must_use]
    pub fn get_date(&self, key: &str) -> Option<DateTime<Utc>> {
        self.get(key)?.as_datetime()
    }

    #[inline]
    #[must_use]
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_number()
    }

    #[inline]
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

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

    #[inline]
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Creates a new Frontmatter from field map.
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns Result for future validation.
    #[inline]
    pub fn new(fields: HashMap<String, FieldValue>) -> Result<Self, NoteError> {
        Ok(Self {
            fields,
        })
    }

    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::config::aggregate::Config) -> String {
        self.get_str(&config.frontmatter.title_key)
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test code uses unwrap/expect for simplicity"
)]
mod tests {
    use chrono::{Datelike as _, TimeZone as _};

    use super::*;

    #[test]
    fn parses_iso8601_date_successfully() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
        let timestamp = date.timestamp();
        let val = FieldValue::Date(timestamp);
        assert_eq!(val.as_date(), Some(timestamp));
        assert_eq!(val.as_datetime().unwrap().year(), 2_024i32);
    }

    #[test]
    fn converts_numeric_values_correctly() {
        let val = FieldValue::Number(42.0f64);
        let observed = val.as_number();
        assert_eq!(observed, Some(42.0f64));
    }

    #[test]
    fn converts_boolean_values_correctly() {
        let val = FieldValue::Boolean(true);
        let observed = val.as_bool();
        assert_eq!(observed, Some(true));
    }

    #[test]
    fn has_method_detects_field_presence() {
        let mut fields = HashMap::new();
        fields
            .insert("title".to_owned(), FieldValue::String("Test".to_owned()));
        let fm = Frontmatter::new(fields).unwrap();
        assert!(fm.has("title"));
        assert!(!fm.has("missing"));
    }

    #[test]
    fn accessors_handle_configured_keys() {
        let mut global = crate::config::global::Global::default();
        global.frontmatter.title_key = "subject".to_owned();
        global.frontmatter.file_class_key = "kind".to_owned();
        global.frontmatter.alias_key = "names".to_owned();
        let config = crate::config::aggregate::Config::build(
            Some(&global),
            "/v",
            crate::config::vault::Vault::default(),
        )
        .unwrap();

        let mut fields = HashMap::new();
        fields.insert("subject".to_owned(), FieldValue::String("Subj".into()));
        fields.insert("kind".to_owned(), FieldValue::String("Note".into()));
        fields.insert("names".to_owned(), FieldValue::String("Alias".into()));
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.title(&config), "Subj");
        assert_eq!(fm.file_class(&config), "Note");
        assert_eq!(fm.aliases(&config), vec!["Alias".to_owned()]);
    }

    #[test]
    fn get_as_performs_type_conversion() {
        let mut fields = HashMap::new();
        fields.insert("s".to_owned(), FieldValue::String("text".into()));
        fields.insert("b".to_owned(), FieldValue::Boolean(true));
        fields.insert("n".to_owned(), FieldValue::Number(1.5f64));
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.get_as::<String>("s"), Some("text".to_owned()));
        assert_eq!(fm.get_as::<bool>("b"), Some(true));
        assert_eq!(fm.get_as::<f64>("n"), Some(1.5f64));
        assert_eq!(fm.get_as::<bool>("s"), None);
    }

    #[test]
    fn get_string_array_handles_single_and_multiple() {
        let mut fields = HashMap::new();
        fields.insert("single".to_owned(), FieldValue::String("a".into()));
        fields.insert(
            "multi".to_owned(),
            FieldValue::Array(vec![FieldValue::String("b".into())]),
        );
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.get_string_array("single"), Some(vec!["a".to_owned()]));
        assert_eq!(fm.get_string_array("multi"), Some(vec!["b".to_owned()]));
    }

    #[test]
    fn field_value_coercion_covers_all_variants() {
        let arr_val = FieldValue::Array(vec![FieldValue::Boolean(true)]);
        let bool_val = FieldValue::Boolean(true);
        let date_val = FieldValue::Date(Utc::now().timestamp());
        let num_val = FieldValue::Number(1.0f64);
        let mut obj_map = HashMap::new();
        obj_map.insert("k".to_owned(), FieldValue::Boolean(false));
        let obj_val = FieldValue::Object(obj_map);
        let str_val = FieldValue::String("s".into());

        assert!(arr_val.as_array().is_some());
        assert!(bool_val.as_bool().is_some());
        assert!(date_val.as_date().is_some());
        assert!(date_val.as_datetime().is_some());
        assert!(num_val.as_number().is_some());
        assert!(obj_val.as_object().is_some());
        assert!(str_val.as_str().is_some());

        assert!(arr_val.as_bool().is_none());
        assert!(bool_val.as_array().is_none());
        assert!(date_val.as_number().is_none());
        assert!(num_val.as_date().is_none());
        assert!(obj_val.as_str().is_none());
        assert!(str_val.as_object().is_none());
    }

    #[test]
    fn get_typed_helpers_retrieve_values() {
        let mut fields = HashMap::new();
        fields.insert("b".to_owned(), FieldValue::Boolean(true));
        fields.insert("n".to_owned(), FieldValue::Number(1.0f64));
        fields.insert("s".to_owned(), FieldValue::String("s".into()));
        fields.insert("d".to_owned(), FieldValue::Date(Utc::now().timestamp()));
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.get_bool("b"), Some(true));
        assert_eq!(fm.get_number("n"), Some(1.0f64));
        assert_eq!(fm.get_str("s"), Some("s"));
        assert!(fm.get_date("d").is_some());

        assert!(fm.get_bool("missing").is_none());
        assert!(fm.get_bool("n").is_none());
    }
}
