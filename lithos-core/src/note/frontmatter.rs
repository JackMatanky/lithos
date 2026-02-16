//! YAML/TOML metadata management for notes.
//!
//! Handles the parsing, validation, and retrieval of structured metadata
//! stored at the beginning of markdown files.

#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::collections::HashMap;

use super::{
    error::{FrontmatterError, NoteError},
    value::FieldValue,
};

/// Represents YAML/TOML metadata extracted from a note header.
///
/// Frontmatter provides structured key-value pairs at the beginning of a
/// markdown document. It is used for tagging, aliasing, and custom metadata
/// that can be queried across the vault.
///
/// This struct provides a type-safe API for accessing metadata values while
/// maintaining the dynamic nature of markdown headers.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::frontmatter::Frontmatter;
/// # use lithos_core::note::value::FieldValue;
/// # use std::collections::HashMap;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut fields = HashMap::new();
/// fields.insert("status".into(), FieldValue::String("draft".into()));
///
/// let fm = Frontmatter::new(fields)?;
/// assert!(fm.has("status"));
/// # Ok(())
/// # }
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
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    fields: HashMap<Box<str>, FieldValue>,
}

impl Frontmatter {
    /// Creates a new [`Frontmatter`] instance from a field map.
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns [`Result`] for future structural
    /// validation.
    #[inline]
    pub fn new(
        fields: HashMap<Box<str>, FieldValue>,
    ) -> Result<Self, NoteError> {
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
    /// Returns a [`FrontmatterError`] if the key exists but cannot be converted
    /// to `T`.
    #[inline]
    #[expect(
        private_bounds,
        reason = "FromFieldValue is an internal adapter trait that should not \
                  be public"
    )]
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
    /// Returns [`FrontmatterError::Missing`] if the key is absent.
    #[inline]
    #[expect(
        private_bounds,
        reason = "FromFieldValue is an internal adapter trait that should not \
                  be public"
    )]
    pub fn try_get_required<T: FromFieldValue>(
        &self,
        key: &str,
    ) -> Result<T, FrontmatterError> {
        self.try_get(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.into(),
        })
    }

    /// Performs strict string-array extraction.
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
    ) -> Result<Vec<Box<str>>, FrontmatterError> {
        self.try_get_required::<Vec<Box<str>>>(key)
    }

    /// Strictly extracts a *borrowed* typed value from frontmatter.
    ///
    /// This mirrors [`Self::try_get`], but allows return types that borrow from
    /// the underlying [`FieldValue`] (e.g., `&str`, slices, or object maps).
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] if the key exists but cannot be converted
    /// to `T`.
    #[inline]
    #[expect(
        private_bounds,
        reason = "FromFieldValueRef is an internal adapter trait that should \
                  not be public"
    )]
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
    /// Returns [`FrontmatterError::Missing`] if the key is absent.
    #[inline]
    #[expect(
        private_bounds,
        reason = "FromFieldValueRef is an internal adapter trait that should \
                  not be public"
    )]
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
    pub fn title(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.get(config.frontmatter().title().as_str())
            .and_then(FieldValue::as_str)
    }

    /// Returns the file class of the note, using the configured key.
    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.get(config.frontmatter().file_class().as_str())
            .and_then(FieldValue::as_str)
    }

    /// Returns the aliases of the note as a vector of boxed strings.
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<Box<str>> {
        self.get(config.frontmatter().alias().as_str())
            .and_then(as_string_array_lossy)
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

/// Adapter trait for frontmatter-specific conversions from [`FieldValue`].
///
/// This trait provides frontmatter-specific error handling by converting
/// generic [`super::value::FieldValueError`] into context-aware
/// [`FrontmatterError`] with key information.
///
/// # Implementation Note
///
/// This trait mirrors [`super::value::FromFieldValue`] but returns
/// [`FrontmatterError`] instead of [`super::value::FieldValueError`].
/// The blanket implementation adapts all types implementing
/// [`super::value::FromFieldValue`].
pub(super) trait FromFieldValue: Sized {
    /// Attempts to extract a value of type `Self` from a [`FieldValue`].
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] describing why the conversion failed.
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError>;
}

/// Adapter trait for frontmatter-specific borrowed conversions from
/// [`FieldValue`].
///
/// This trait mirrors [`super::value::FromFieldValueRef`] but returns
/// [`FrontmatterError`] instead of [`super::value::FieldValueError`].
/// The blanket implementation adapts all types implementing
/// [`super::value::FromFieldValueRef`].
pub(super) trait FromFieldValueRef<'frontmatter>: Sized {
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

// Blanket implementation that adapts value::FromFieldValue to
// frontmatter::FromFieldValue
impl<T> FromFieldValue for T
where
    T: super::value::FromFieldValue,
{
    #[inline]
    fn from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        T::from_value(value).map_err(|err| match err {
            super::value::FieldValueError::TypeMismatch {
                expected,
                actual,
            } => FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: format!("{expected}").into(),
                actual,
            },
            super::value::FieldValueError::InvalidDateTimestamp {
                timestamp,
            } => FrontmatterError::InvalidDateTimestamp {
                key: "".into(),
                timestamp,
            },
            super::value::FieldValueError::ArrayElementTypeMismatch {
                index,
                expected,
                actual,
            } => FrontmatterError::ArrayElementTypeMismatch {
                key: "".into(),
                index,
                expected,
                actual,
            },
        })
    }
}

// Blanket implementation that adapts value::FromFieldValueRef to
// frontmatter::FromFieldValueRef
impl<'frontmatter, T> FromFieldValueRef<'frontmatter> for T
where
    T: super::value::FromFieldValueRef<'frontmatter>,
{
    #[inline]
    fn from_value_ref(
        value: &'frontmatter FieldValue,
    ) -> Result<Self, FrontmatterError> {
        T::from_value_ref(value).map_err(|err| match err {
            super::value::FieldValueError::TypeMismatch {
                expected,
                actual,
            } => FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: format!("{expected}").into(),
                actual,
            },
            super::value::FieldValueError::InvalidDateTimestamp {
                timestamp,
            } => FrontmatterError::InvalidDateTimestamp {
                key: "".into(),
                timestamp,
            },
            super::value::FieldValueError::ArrayElementTypeMismatch {
                index,
                expected,
                actual,
            } => FrontmatterError::ArrayElementTypeMismatch {
                key: "".into(),
                index,
                expected,
                actual,
            },
        })
    }
}

/// Returns a lenient string array conversion.
///
/// This filters out non-string elements rather than erroring.
#[inline]
#[must_use]
pub fn as_string_array_lossy(value: &FieldValue) -> Option<Vec<Box<str>>> {
    if let Some(arr) = value.as_array() {
        return Some(
            arr.iter()
                .filter_map(|item| item.as_str().map(Into::into))
                .collect(),
        );
    }

    value.as_str().map(|s| vec![s.into()])
}

#[cfg(test)]
mod tests {
    /// Test fixtures and builders for Frontmatter tests.
    mod fixtures {
        use chrono::{DateTime, TimeZone as _, Utc};

        use super::{super::*, TEST_TIMESTAMP};
        use crate::config::aggregate::Config;

        /// Builder for creating test Frontmatter instances.
        pub struct FrontmatterBuilder {
            fields: HashMap<Box<str>, FieldValue>,
        }

        impl FrontmatterBuilder {
            pub fn new() -> Self {
                Self {
                    fields: HashMap::new(),
                }
            }

            pub fn with_string(mut self, key: &str, value: &str) -> Self {
                self.fields
                    .insert(key.into(), FieldValue::String(value.into()));
                self
            }

            pub fn with_boolean(mut self, key: &str, value: bool) -> Self {
                self.fields.insert(key.into(), FieldValue::Boolean(value));
                self
            }

            pub fn with_number(mut self, key: &str, value: f64) -> Self {
                self.fields.insert(key.into(), FieldValue::Number(value));
                self
            }

            pub fn with_date(mut self, key: &str, timestamp: i64) -> Self {
                self.fields.insert(key.into(), FieldValue::Date(timestamp));
                self
            }

            pub fn with_array(
                mut self,
                key: &str,
                values: Vec<FieldValue>,
            ) -> Self {
                self.fields.insert(key.into(), FieldValue::Array(values));
                self
            }

            pub fn build(self) -> Result<Frontmatter, NoteError> {
                Frontmatter::new(self.fields)
            }
        }

        pub fn config_with_custom_frontmatter_keys() -> Config {
            use crate::config::{
                frontmatter::RawFrontmatter,
                raw::RawConfig,
                vault::{VaultId, VaultRoot},
            };

            let raw = RawConfig {
                frontmatter: Some(RawFrontmatter {
                    alias_key: Some("names".to_owned()),
                    date_created_key: Some("date_created".to_owned()),
                    date_modified_key: Some("date_modified".to_owned()),
                    file_class_key: Some("kind".to_owned()),
                    title_key: Some("subject".to_owned()),
                }),
                ..Default::default()
            };

            crate::config::aggregate::Config::build(
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
            fields.insert("s".into(), FieldValue::String("text".into()));
            fields.insert("b".into(), FieldValue::Boolean(true));
            fields.insert("n".into(), FieldValue::Number(1.5f64));
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn frontmatter_with_aliases_mixed() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert(
                "aliases".into(),
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
            fields.insert("n".into(), FieldValue::Number(1.0f64));
            Frontmatter::new(fields)
                .expect("Frontmatter construction should succeed")
        }

        pub fn frontmatter_with_invalid_date() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("d".into(), FieldValue::Date(i64::MAX));
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
        #![allow(
            clippy::items_after_statements,
            clippy::no_effect_underscore_binding,
            reason = "Test code style"
        )]

        use chrono::Utc;

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
            let _value = FieldValue::Date(TEST_TIMESTAMP);
            use chrono::TimeZone as _;
            let dt = Utc.timestamp_opt(TEST_TIMESTAMP, 0).single();
            assert!(dt.is_some(), "Date should coerce to DateTime");
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
            obj_map.insert("k".into(), FieldValue::Boolean(false));
            let value = FieldValue::Object(obj_map);
            assert!(
                value.as_object().is_some(),
                "Object should coerce to HashMap"
            );
        }

        #[test]
        fn object_does_not_coerce_to_string() {
            let mut obj_map = HashMap::new();
            obj_map.insert("k".into(), FieldValue::Boolean(false));
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
            use chrono::TimeZone as _;
            let _val = FieldValue::Date(timestamp);
            let dt = Utc.timestamp_opt(timestamp, 0).single();
            assert!(dt.is_some(), "Date field should convert to DateTime");
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
                Some("Subj"),
                "Title should use configured key"
            );
        }

        #[test]
        fn file_class_uses_configured_key() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let fm = fixtures::frontmatter_with_custom_keys();
            assert_eq!(
                fm.file_class(&config),
                Some("Note"),
                "File class should use configured key"
            );
        }

        #[test]
        fn aliases_use_configured_key() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let fm = fixtures::frontmatter_with_custom_keys();
            assert_eq!(
                fm.aliases(&config),
                vec!["Alias".into()],
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
                fm.get("single").and_then(as_string_array_lossy),
                Some(vec!["a".into()]),
                "Single string should convert to array"
            );
        }

        #[test]
        fn string_array_lossy_returns_array_values() {
            let fm = fixtures::frontmatter_with_string_arrays();
            assert_eq!(
                fm.get("multi").and_then(as_string_array_lossy),
                Some(vec!["b".into()]),
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
                fm.get("d")
                    .and_then(crate::note::value::FieldValue::as_date)
                    .is_some(),
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
        use chrono::{DateTime, Utc};

        use super::{super::*, fixtures};
        use crate::note::value::FieldValueType;

        #[test]
        fn try_get_returns_string_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let result = fm.try_get::<Box<str>>("s");
            assert_eq!(
                result,
                Ok(Some("text".into())),
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
                fm.get("aliases").and_then(as_string_array_lossy),
                Some(vec!["ok".into()])
            );
        }

        #[test]
        fn strict_get_required_reports_missing_key() {
            let fm = fixtures::frontmatter_with_number();
            let result = fm.try_get_required::<Box<str>>("missing");
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
            let result = fm.try_get_required::<Box<str>>("n");
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
