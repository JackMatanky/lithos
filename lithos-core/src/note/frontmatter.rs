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

pub type FrontmatterError = super::error::FrontmatterError;

/// Represents YAML metadata extracted from a note header.
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
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    fields: HashMap<String, FieldValue>,
}

impl Frontmatter {
    #[inline]
    fn with_key_context(
        key: &str,
        mut err: FrontmatterError,
    ) -> FrontmatterError {
        let key_str: Box<str> = key.into();
        match &mut err {
            &mut (FrontmatterError::Missing {
                key: ref mut existing,
            }
            | FrontmatterError::TypeMismatch {
                key: ref mut existing,
                ..
            }
            | FrontmatterError::ArrayElementTypeMismatch {
                key: ref mut existing,
                ..
            }
            | FrontmatterError::InvalidDateTimestamp {
                key: ref mut existing,
                ..
            }) => {
                if existing.is_empty() {
                    *existing = key_str;
                }
            }
        }
        err
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
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }

    #[inline]
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Strictly extracts a typed value from frontmatter.
    ///
    /// Returns:
    /// - `Ok(None)` if the key is missing.
    /// - `Ok(Some(T))` if present and valid.
    /// - `Err(FrontmatterError)` if present but invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the key exists but cannot be converted to `T`.
    #[inline]
    pub fn try_get<T: FromFieldValue>(
        &self,
        key: &str,
    ) -> Result<Option<T>, FrontmatterError> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        T::from_value(value)
            .map(Some)
            .map_err(|err| Self::with_key_context(key, err))
    }

    /// Strictly extracts a required typed value from frontmatter.
    ///
    /// # Errors
    ///
    /// Returns `FrontmatterError::Missing` if the key is absent.
    #[inline]
    pub fn try_get_required<T: FromFieldValue>(
        &self,
        key: &str,
    ) -> Result<T, FrontmatterError> {
        self.try_get(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.into(),
        })
    }

    /// Strict string-array extraction.
    ///
    /// Unlike [`Self::get_string_array`], this fails if an array contains any
    /// non-string elements.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is missing, if the value is not a string or
    /// array of strings, or if any array element is not a string.
    #[inline]
    pub fn try_get_string_vec_strict(
        &self,
        key: &str,
    ) -> Result<Vec<String>, FrontmatterError> {
        self.try_get_required::<Vec<String>>(key)
    }

    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::config::aggregate::Config) -> String {
        self.get(&config.frontmatter.title_key)
            .and_then(FieldValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> String {
        self.get(&config.frontmatter.file_class_key)
            .and_then(FieldValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<String> {
        self.get(&config.frontmatter.alias_key)
            .and_then(FieldValue::as_string_array_lossy)
            .unwrap_or_default()
    }
}

/// A high-level type descriptor for [`FieldValue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValueType {
    Array,
    Boolean,
    Date,
    Number,
    Object,
    String,
}

impl core::fmt::Display for FieldValueType {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match *self {
            Self::Array => "array",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::Number => "number",
            Self::Object => "object",
            Self::String => "string",
        };
        f.write_str(name)
    }
}

/// Fallible, strict conversions from a [`FieldValue`].
///
/// This is intentionally a *local* trait (instead of `TryFrom<&FieldValue>`) to
/// avoid Rust's orphan rules (we can't implement foreign traits for foreign
/// types like `bool`, `f64`, `String`, etc.).
pub trait FromFieldValue: Sized {
    /// Attempts to extract a value of type `Self` from a [`FieldValue`].
    ///
    /// Returns a structured error when the value is present but incompatible.
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] describing why the conversion failed.
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError>;
}

/// Possible values in a frontmatter field.
///
/// This enum represents the runtime type of a value parsed from YAML
/// frontmatter. It mirrors the design of `serde_json::Value` to support dynamic
/// typing scenarios.
///
/// Note: `DateTime` stored as i64 timestamp for rkyv compatibility.
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
#[rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
)))]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum FieldValue {
    /// Array of values.
    Array(#[rkyv(omit_bounds)] Vec<FieldValue>),
    /// Boolean value.
    Boolean(bool),
    /// Date/time value (stored as Unix timestamp for serialization).
    Date(i64),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(#[rkyv(omit_bounds)] HashMap<String, FieldValue>),
    /// String value.
    String(String),
}

impl FromFieldValue for bool {
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_bool().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "boolean".into(),
            actual: value.value_type(),
        })
    }
}

impl FromFieldValue for f64 {
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_number().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "number".into(),
            actual: value.value_type(),
        })
    }
}

impl FromFieldValue for String {
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
            FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: "string".into(),
                actual: value.value_type(),
            }
        })
    }
}

impl FromFieldValue for DateTime<Utc> {
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        use chrono::TimeZone as _;
        let ts =
            value.as_date().ok_or_else(|| FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: "date".into(),
                actual: value.value_type(),
            })?;
        Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
            FrontmatterError::InvalidDateTimestamp {
                key: "".into(),
                timestamp: ts,
            }
        })
    }
}

impl FromFieldValue for Vec<String> {
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(arr) = value.as_array() {
            let mut out = Vec::with_capacity(arr.len());
            for (index, item) in arr.iter().enumerate() {
                let Some(s) = item.as_str() else {
                    return Err(FrontmatterError::ArrayElementTypeMismatch {
                        key: "".into(),
                        index,
                        expected: FieldValueType::String,
                        actual: item.value_type(),
                    });
                };
                out.push(s.to_owned());
            }
            return Ok(out);
        }

        value.as_str().map(|s| vec![s.to_owned()]).ok_or_else(|| {
            FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: "array|string".into(),
                actual: value.value_type(),
            }
        })
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
    pub const fn value_type(&self) -> FieldValueType {
        match *self {
            Self::Array(_) => FieldValueType::Array,
            Self::Boolean(_) => FieldValueType::Boolean,
            Self::Date(_) => FieldValueType::Date,
            Self::Number(_) => FieldValueType::Number,
            Self::Object(_) => FieldValueType::Object,
            Self::String(_) => FieldValueType::String,
        }
    }

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
    pub fn as_string_array_lossy(&self) -> Option<Vec<String>> {
        if let Some(arr) = self.as_array() {
            return Some(
                arr.iter()
                    .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                    .collect(),
            );
        }

        self.as_str().map(|s| vec![s.to_owned()])
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
    fn try_get_performs_type_conversion() {
        let mut fields = HashMap::new();
        fields.insert("s".to_owned(), FieldValue::String("text".into()));
        fields.insert("b".to_owned(), FieldValue::Boolean(true));
        fields.insert("n".to_owned(), FieldValue::Number(1.5f64));
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.try_get::<String>("s").unwrap(), Some("text".to_owned()));
        assert_eq!(fm.try_get::<bool>("b").unwrap(), Some(true));
        assert_eq!(fm.try_get::<f64>("n").unwrap(), Some(1.5f64));

        let err =
            fm.try_get::<bool>("s").expect_err("type mismatch should error");
        assert!(matches!(
            err,
            FrontmatterError::TypeMismatch { key, expected, actual: FieldValueType::String }
                if key.as_ref() == "s" && expected.as_ref() == "boolean"
        ));
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

        assert_eq!(
            fm.get("single").and_then(FieldValue::as_string_array_lossy),
            Some(vec!["a".to_owned()])
        );
        assert_eq!(
            fm.get("multi").and_then(FieldValue::as_string_array_lossy),
            Some(vec!["b".to_owned()])
        );
    }

    #[test]
    fn strict_string_vec_errors_on_non_string_array_elements() {
        let mut fields = HashMap::new();
        fields.insert(
            "aliases".to_owned(),
            FieldValue::Array(vec![
                FieldValue::String("ok".into()),
                FieldValue::Number(123.0),
            ]),
        );
        let fm = Frontmatter::new(fields).unwrap();

        let err = fm
            .try_get_string_vec_strict("aliases")
            .expect_err("strict extraction should fail");
        assert!(matches!(
            err,
            FrontmatterError::ArrayElementTypeMismatch {
                key,
                index: 1,
                expected: FieldValueType::String,
                actual: FieldValueType::Number,
            } if key.as_ref() == "aliases"
        ));

        // Lenient extraction keeps today's behavior (drops non-strings).
        assert_eq!(
            fm.get("aliases").and_then(FieldValue::as_string_array_lossy),
            Some(vec!["ok".to_owned()])
        );
    }

    #[test]
    fn strict_get_required_distinguishes_missing_from_mismatch() {
        let mut fields = HashMap::new();
        fields.insert("n".to_owned(), FieldValue::Number(1.0f64));
        let fm = Frontmatter::new(fields).unwrap();

        let missing = fm
            .try_get_required::<String>("missing")
            .expect_err("missing key should error");
        assert!(matches!(
            missing,
            FrontmatterError::Missing { key } if key.as_ref() == "missing"
        ));

        let mismatch = fm
            .try_get_required::<String>("n")
            .expect_err("type mismatch should error");
        assert!(matches!(
            mismatch,
            FrontmatterError::TypeMismatch { key, expected, actual: FieldValueType::Number }
                if key.as_ref() == "n" && expected.as_ref() == "string"
        ));
    }

    #[test]
    fn strict_date_reports_invalid_timestamp() {
        let mut fields = HashMap::new();
        fields.insert("d".to_owned(), FieldValue::Date(i64::MAX));
        let fm = Frontmatter::new(fields).unwrap();

        let err = fm
            .try_get_required::<DateTime<Utc>>("d")
            .expect_err("invalid timestamp should error");
        assert!(matches!(
            err,
            FrontmatterError::InvalidDateTimestamp { key, timestamp: i64::MAX }
                if key.as_ref() == "d"
        ));
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
    fn get_retrieve_and_coerce_values() {
        let mut fields = HashMap::new();
        fields.insert("b".to_owned(), FieldValue::Boolean(true));
        fields.insert("n".to_owned(), FieldValue::Number(1.0f64));
        fields.insert("s".to_owned(), FieldValue::String("s".into()));
        fields.insert("d".to_owned(), FieldValue::Date(Utc::now().timestamp()));
        let fm = Frontmatter::new(fields).unwrap();

        assert_eq!(fm.get("b").and_then(FieldValue::as_bool), Some(true));
        assert_eq!(fm.get("n").and_then(FieldValue::as_number), Some(1.0f64));
        assert_eq!(fm.get("s").and_then(FieldValue::as_str), Some("s"));
        assert!(fm.get("d").and_then(FieldValue::as_datetime).is_some());

        assert!(fm.get("missing").is_none());
        assert!(fm.get("n").and_then(FieldValue::as_bool).is_none());
    }
}
