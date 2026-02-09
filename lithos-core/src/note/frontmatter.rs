//! Frontmatter domain entities and metadata extraction.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
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

    /// Returns a reference to the value for the given key, if it exists.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }

    /// Returns `true` if the frontmatter contains a field with the given key.
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
    /// This fails if an array contains any non-string elements.
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

    /// Strictly extracts a *borrowed* typed value from frontmatter.
    ///
    /// This mirrors [`Self::try_get`], but allows return types that borrow from
    /// the underlying [`FieldValue`] (e.g., `&str`, slices, or object maps).
    ///
    /// # Errors
    ///
    /// Returns an error if the key exists but cannot be converted to `T`.
    #[inline]
    pub fn try_get_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &str,
    ) -> Result<Option<T>, FrontmatterError>
    where
        T: FromFieldValueRef<'frontmatter>,
    {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        T::from_value_ref(value)
            .map(Some)
            .map_err(|err| Self::with_key_context(key, err))
    }

    /// Strictly extracts a required *borrowed* typed value from frontmatter.
    ///
    /// # Errors
    ///
    /// Returns `FrontmatterError::Missing` if the key is absent.
    #[inline]
    pub fn try_get_required_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &str,
    ) -> Result<T, FrontmatterError>
    where
        T: FromFieldValueRef<'frontmatter>,
    {
        self.try_get_ref(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.into(),
        })
    }

    /// Returns the title of the note, using the configured title key.
    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::config::aggregate::Config) -> String {
        self.get(config.frontmatter.title_key().as_str())
            .and_then(FieldValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    /// Returns the file class of the note, using the configured key.
    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> String {
        self.get(config.frontmatter.file_class_key().as_str())
            .and_then(FieldValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    /// Returns the aliases of the note as a vector of strings.
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<String> {
        self.get(config.frontmatter.alias_key().as_str())
            .and_then(FieldValue::as_string_array_lossy)
            .unwrap_or_default()
    }

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

/// Fallible, strict conversions from a borrowed [`FieldValue`].
///
/// This exists to support *non-owning* access patterns like `&str` and slices.
pub trait FromFieldValueRef<'frontmatter>: Sized {
    /// Attempts to extract a value of type `Self` from a borrowed
    /// [`FieldValue`].
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] describing why the conversion failed.
    fn from_value_ref(
        value: &'frontmatter FieldValue,
    ) -> Result<Self, FrontmatterError>;
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

impl<'frontmatter> FromFieldValueRef<'frontmatter> for &'frontmatter str {
    #[inline]
    fn from_value_ref(
        value: &'frontmatter FieldValue,
    ) -> Result<Self, FrontmatterError> {
        value.as_str().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "string".into(),
            actual: value.value_type(),
        })
    }
}

impl<'frontmatter> FromFieldValueRef<'frontmatter>
    for &'frontmatter [FieldValue]
{
    #[inline]
    fn from_value_ref(
        value: &'frontmatter FieldValue,
    ) -> Result<Self, FrontmatterError> {
        value.as_array().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "array".into(),
            actual: value.value_type(),
        })
    }
}

impl<'frontmatter> FromFieldValueRef<'frontmatter>
    for &'frontmatter HashMap<
        String,
        FieldValue,
        ::std::collections::hash_map::RandomState,
    >
{
    #[inline]
    fn from_value_ref(
        value: &'frontmatter FieldValue,
    ) -> Result<Self, FrontmatterError> {
        value.as_object().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "object".into(),
            actual: value.value_type(),
        })
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Accessor methods intentionally use match ergonomics on `&self` \
              (e.g., `if let Self::Array(arr) = self`) to avoid `ref` \
              patterns and keep the code concise"
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
        if let Self::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let &Self::Boolean(b) = self {
            Some(b)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn as_date(&self) -> Option<i64> {
        if let &Self::Date(timestamp) = self {
            Some(timestamp)
        } else {
            None
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
        if let &Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, FieldValue>> {
        if let Self::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
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

/// A high-level type descriptor for [`FieldValue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValueType {
    /// Array of field values.
    Array,
    /// Boolean value.
    Boolean,
    /// Date timestamp.
    Date,
    /// Floating point number.
    Number,
    /// Map of string keys to field values.
    Object,
    /// String value.
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

#[cfg(test)]
mod tests {
    /// Test fixtures and builders for Frontmatter tests.
    #[expect(
        clippy::disallowed_methods,
        reason = "Fixture helpers use expect for deterministic setup."
    )]
    mod fixtures {
        use chrono::TimeZone as _;

        use super::{super::*, TEST_TIMESTAMP};
        use crate::config::aggregate::Config;

        /// Builder for creating test Frontmatter instances.
        pub struct FrontmatterBuilder {
            fields: HashMap<String, FieldValue>,
        }

        impl FrontmatterBuilder {
            pub fn new() -> Self {
                Self {
                    fields: HashMap::new(),
                }
            }

            pub fn with_string(mut self, key: &str, value: &str) -> Self {
                self.fields
                    .insert(key.to_owned(), FieldValue::String(value.into()));
                self
            }

            pub fn with_boolean(mut self, key: &str, value: bool) -> Self {
                self.fields.insert(key.to_owned(), FieldValue::Boolean(value));
                self
            }

            pub fn with_number(mut self, key: &str, value: f64) -> Self {
                self.fields.insert(key.to_owned(), FieldValue::Number(value));
                self
            }

            pub fn with_date(mut self, key: &str, timestamp: i64) -> Self {
                self.fields.insert(key.to_owned(), FieldValue::Date(timestamp));
                self
            }

            pub fn with_array(
                mut self,
                key: &str,
                values: Vec<FieldValue>,
            ) -> Self {
                self.fields.insert(key.to_owned(), FieldValue::Array(values));
                self
            }

            pub fn build(self) -> Result<Frontmatter, NoteError> {
                Frontmatter::new(self.fields)
            }
        }

        pub fn config_with_custom_frontmatter_keys() -> Config {
            use crate::config::{
                frontmatter::RawFrontmatter,
                raw,
                vault::{VaultId, VaultRoot},
            };

            let raw = raw::RawConfig {
                frontmatter: Some(RawFrontmatter {
                    alias_key: Some("names".to_owned()),
                    date_created_key: Some("date_created".to_owned()),
                    date_modified_key: Some("date_modified".to_owned()),
                    file_class_key: Some("kind".to_owned()),
                    title_key: Some("subject".to_owned()),
                }),
                ..Default::default()
            };

            Config::build(
                &raw,
                VaultId::new(),
                VaultRoot::try_new(std::path::PathBuf::from("/v"))
                    .expect("vault_root"),
            )
            .expect("Config build should succeed")
        }

        pub fn frontmatter_with_custom_keys() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_string("subject", "Subj")
                .with_string("kind", "Note")
                .with_string("names", "Alias")
                .build()
                .expect("Frontmatter build should succeed")
        }

        pub fn frontmatter_with_title() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_string("title", "Test")
                .build()
                .expect("Frontmatter build should succeed")
        }

        pub fn frontmatter_with_string_arrays() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_string("single", "a")
                .with_array("multi", vec![FieldValue::String("b".into())])
                .build()
                .expect("Frontmatter build should succeed")
        }

        pub fn frontmatter_with_scalar_values() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_boolean("b", true)
                .with_number("n", 1.0)
                .with_string("s", "s")
                .with_date("d", TEST_TIMESTAMP)
                .build()
                .expect("Frontmatter build should succeed")
        }

        pub fn frontmatter_for_try_get() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("s".to_owned(), FieldValue::String("text".into()));
            fields.insert("b".to_owned(), FieldValue::Boolean(true));
            fields.insert("n".to_owned(), FieldValue::Number(1.5f64));
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn frontmatter_with_aliases_mixed() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert(
                "aliases".to_owned(),
                FieldValue::Array(vec![
                    FieldValue::String("ok".into()),
                    FieldValue::Number(123.0),
                ]),
            );
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn frontmatter_with_number() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("n".to_owned(), FieldValue::Number(1.0f64));
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn frontmatter_with_invalid_date() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("d".to_owned(), FieldValue::Date(i64::MAX));
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn sample_datetime() -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0)
                .single()
                .expect("Valid date should be created")
        }
    }

    mod field_value {
        use chrono::Datelike as _;

        use super::{super::*, TEST_TIMESTAMP, fixtures};

        #[test]
        fn array_coerces_to_array() {
            let value = FieldValue::Array(vec![FieldValue::Boolean(true)]);
            assert!(value.as_array().is_some(), "Array should coerce to array");
        }

        #[test]
        fn array_does_not_coerce_to_bool() {
            let value = FieldValue::Array(vec![FieldValue::Boolean(true)]);
            assert!(
                value.as_bool().is_none(),
                "Array should not coerce to bool"
            );
        }

        #[test]
        fn boolean_coerces_to_bool() {
            let value = FieldValue::Boolean(true);
            assert!(value.as_bool().is_some(), "Boolean should coerce to bool");
        }

        #[test]
        fn boolean_does_not_coerce_to_array() {
            let value = FieldValue::Boolean(true);
            assert!(
                value.as_array().is_none(),
                "Boolean should not coerce to array"
            );
        }

        #[test]
        fn date_coerces_to_timestamp() {
            let value = FieldValue::Date(TEST_TIMESTAMP);
            assert!(
                value.as_date().is_some(),
                "Date should coerce to timestamp"
            );
        }

        #[test]
        fn date_coerces_to_datetime() {
            let value = FieldValue::Date(TEST_TIMESTAMP);
            assert!(
                value.as_datetime().is_some(),
                "Date should coerce to DateTime"
            );
        }

        #[test]
        fn date_does_not_coerce_to_number() {
            let value = FieldValue::Date(TEST_TIMESTAMP);
            assert!(
                value.as_number().is_none(),
                "Date should not coerce to number"
            );
        }

        #[test]
        fn number_coerces_to_number() {
            let value = FieldValue::Number(1.0f64);
            assert!(value.as_number().is_some(), "Number should coerce to f64");
        }

        #[test]
        fn number_does_not_coerce_to_date() {
            let value = FieldValue::Number(1.0f64);
            assert!(
                value.as_date().is_none(),
                "Number should not coerce to date"
            );
        }

        #[test]
        fn object_coerces_to_object() {
            let mut obj_map = HashMap::new();
            obj_map.insert("k".to_owned(), FieldValue::Boolean(false));
            let value = FieldValue::Object(obj_map);
            assert!(
                value.as_object().is_some(),
                "Object should coerce to HashMap"
            );
        }

        #[test]
        fn object_does_not_coerce_to_string() {
            let mut obj_map = HashMap::new();
            obj_map.insert("k".to_owned(), FieldValue::Boolean(false));
            let value = FieldValue::Object(obj_map);
            assert!(
                value.as_str().is_none(),
                "Object should not coerce to string"
            );
        }

        #[test]
        fn string_coerces_to_str() {
            let value = FieldValue::String("s".into());
            assert!(value.as_str().is_some(), "String should coerce to str");
        }

        #[test]
        fn string_does_not_coerce_to_object() {
            let value = FieldValue::String("s".into());
            assert!(
                value.as_object().is_none(),
                "String should not coerce to object"
            );
        }

        #[test]
        fn date_field_returns_timestamp() {
            let timestamp = fixtures::sample_datetime().timestamp();
            let val = FieldValue::Date(timestamp);
            assert_eq!(
                val.as_date(),
                Some(timestamp),
                "Date field should return timestamp"
            );
        }

        #[test]
        fn date_field_returns_datetime_with_expected_year() {
            let timestamp = fixtures::sample_datetime().timestamp();
            let val = FieldValue::Date(timestamp);
            assert!(
                matches!(
                    val.as_datetime(),
                    Some(dt) if dt.year() == 2_024i32
                ),
                "Date field should convert to DateTime with expected year"
            );
        }

        #[test]
        fn converts_numeric_values_correctly() {
            let val = FieldValue::Number(42.0f64);
            let observed = val.as_number();
            assert_eq!(
                observed,
                Some(42.0f64),
                "Numeric field should convert to f64"
            );
        }

        #[test]
        fn converts_boolean_values_correctly() {
            let val = FieldValue::Boolean(true);
            let observed = val.as_bool();
            assert_eq!(
                observed,
                Some(true),
                "Boolean field should convert to bool"
            );
        }
    }

    mod accessors {
        use super::{super::*, fixtures};

        #[test]
        fn title_uses_configured_key() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let fm = fixtures::frontmatter_with_custom_keys();
            assert_eq!(
                fm.title(&config),
                "Subj",
                "Title should use configured key"
            );
        }

        #[test]
        fn file_class_uses_configured_key() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let fm = fixtures::frontmatter_with_custom_keys();
            assert_eq!(
                fm.file_class(&config),
                "Note",
                "File class should use configured key"
            );
        }

        #[test]
        fn aliases_use_configured_key() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let fm = fixtures::frontmatter_with_custom_keys();
            assert_eq!(
                fm.aliases(&config),
                vec!["Alias".to_owned()],
                "Aliases should use configured key"
            );
        }

        #[test]
        fn has_returns_true_for_existing_field() {
            let fm = fixtures::frontmatter_with_title();
            assert!(fm.has("title"), "Should find existing field 'title'");
        }

        #[test]
        fn has_returns_false_for_missing_field() {
            let fm = fixtures::frontmatter_with_title();
            assert!(!fm.has("missing"), "Should not find non-existent field");
        }

        #[test]
        fn string_array_lossy_converts_single_string() {
            let fm = fixtures::frontmatter_with_string_arrays();
            assert_eq!(
                fm.get("single").and_then(FieldValue::as_string_array_lossy),
                Some(vec!["a".to_owned()]),
                "Single string should convert to array"
            );
        }

        #[test]
        fn string_array_lossy_returns_array_values() {
            let fm = fixtures::frontmatter_with_string_arrays();
            assert_eq!(
                fm.get("multi").and_then(FieldValue::as_string_array_lossy),
                Some(vec!["b".to_owned()]),
                "Array should be returned as-is"
            );
        }

        #[test]
        fn get_returns_boolean_value() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert_eq!(
                fm.get("b").and_then(FieldValue::as_bool),
                Some(true),
                "Boolean field should be returned"
            );
        }

        #[test]
        fn get_returns_number_value() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert_eq!(
                fm.get("n").and_then(FieldValue::as_number),
                Some(1.0f64),
                "Number field should be returned"
            );
        }

        #[test]
        fn get_returns_string_value() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert_eq!(
                fm.get("s").and_then(FieldValue::as_str),
                Some("s"),
                "String field should be returned"
            );
        }

        #[test]
        fn get_returns_datetime_value() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert!(
                fm.get("d").and_then(FieldValue::as_datetime).is_some(),
                "Date field should convert to DateTime"
            );
        }

        #[test]
        fn get_returns_none_for_missing_field() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert!(
                fm.get("missing").is_none(),
                "Missing field should return None"
            );
        }

        #[test]
        fn get_returns_none_for_type_mismatch() {
            let fm = fixtures::frontmatter_with_scalar_values();
            assert!(
                fm.get("n").and_then(FieldValue::as_bool).is_none(),
                "Type mismatch should return None"
            );
        }
    }

    mod conversions {
        use super::{super::*, fixtures};

        #[test]
        fn try_get_returns_string_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let result = fm.try_get::<String>("s");
            assert_eq!(
                result,
                Ok(Some("text".to_owned())),
                "Should retrieve and convert String field"
            );
        }

        #[test]
        fn try_get_returns_boolean_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let result = fm.try_get::<bool>("b");
            assert_eq!(
                result,
                Ok(Some(true)),
                "Should retrieve and convert Boolean field"
            );
        }

        #[test]
        fn try_get_returns_number_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let result = fm.try_get::<f64>("n");
            assert_eq!(
                result,
                Ok(Some(1.5f64)),
                "Should retrieve and convert Number field"
            );
        }

        #[test]
        fn try_get_returns_type_mismatch_error() {
            let fm = fixtures::frontmatter_for_try_get();
            let result = fm.try_get::<bool>("s");
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::TypeMismatch {
                        key,
                        expected,
                        actual,
                    })
                        if key.as_ref() == "s"
                            && expected.as_ref() == "boolean"
                            && *actual == FieldValueType::String
                ),
                "type mismatch should error: {result:?}"
            );
        }

        #[test]
        fn strict_string_vec_errors_on_non_string_array_elements() {
            let fm = fixtures::frontmatter_with_aliases_mixed();
            let result = fm.try_get_string_vec_strict("aliases");
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::ArrayElementTypeMismatch {
                        key,
                        index: 1,
                        expected: FieldValueType::String,
                        actual: FieldValueType::Number,
                    }) if key.as_ref() == "aliases"
                ),
                "strict extraction should fail: {result:?}"
            );
        }

        #[test]
        fn lenient_string_vec_drops_non_string_elements() {
            let fm = fixtures::frontmatter_with_aliases_mixed();
            assert_eq!(
                fm.get("aliases").and_then(FieldValue::as_string_array_lossy),
                Some(vec!["ok".to_owned()])
            );
        }

        #[test]
        fn strict_get_required_reports_missing_key() {
            let fm = fixtures::frontmatter_with_number();
            let result = fm.try_get_required::<String>("missing");
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::Missing { key })
                        if key.as_ref() == "missing"
                ),
                "missing key should error: {result:?}"
            );
        }

        #[test]
        fn strict_get_required_reports_type_mismatch() {
            let fm = fixtures::frontmatter_with_number();
            let result = fm.try_get_required::<String>("n");
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::TypeMismatch {
                        key,
                        expected,
                        actual,
                    })
                        if key.as_ref() == "n"
                            && expected.as_ref() == "string"
                            && *actual == FieldValueType::Number
                ),
                "type mismatch should error: {result:?}"
            );
        }

        #[test]
        fn strict_date_reports_invalid_timestamp() {
            let fm = fixtures::frontmatter_with_invalid_date();
            let result = fm.try_get_required::<DateTime<Utc>>("d");
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::InvalidDateTimestamp {
                        key,
                        timestamp,
                    })
                        if key.as_ref() == "d" && *timestamp == i64::MAX
                ),
                "invalid timestamp should error: {result:?}"
            );
        }
    }

    const TEST_TIMESTAMP: i64 = 1_700_000_000;
}
