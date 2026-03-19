//! YAML/TOML metadata management for notes.
//!
//! This module handles the parsing, validation, and retrieval of structured
//! metadata stored at the beginning of markdown files (frontmatter).
//! It provides a type-safe API for accessing common fields like titles,
//! aliases, and custom metadata, while handling Obsidian-specific nuances
//! like unquoted WikiLinks in YAML.
//!
//! The primary type in this module is [`Frontmatter`].

#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::collections::HashMap;

use regex::Regex;

use super::{
    error::{FrontmatterError, FrontmatterParseError},
    value::{FieldValue, TryFromFieldValue, TryFromFieldValueRef},
};
use crate::{
    config::frontmatter::FrontmatterKey,
    note::raw::{RawFrontmatter, RawFrontmatterFormat},
};

/// Represents YAML/TOML metadata extracted from a note header.
///
/// Frontmatter provides structured key-value pairs at the beginning of a
/// markdown document. It is used for tagging, aliasing, and custom metadata
/// that can be queried across the vault.
///
/// This struct provides a type-safe API for accessing metadata values while
/// maintaining the dynamic nature of markdown headers. It supports both
/// YAML and TOML formats.
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
#[serde(transparent)]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    fields: HashMap<Box<str>, FieldValue>,
}

/// Regex for identifying unquoted Obsidian wikilinks in YAML mapping entries.
///
/// Pattern breakdown:
/// 1. `^(\s*[\w_-]+\s*:\s*)`: Matches the key and colon, including indentation.
/// 2. `([^"'\s|>].*\[\[.*\]\].*)`: Matches values starting with a
///    non-quote/special char that contain a wikilink.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_MAP_LINK_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| {
        Regex::new(r#"(?m)^(\s*[\w_-]+\s*:\s*)([^"'\s|>].*\[\[.*\]\].*)$"#)
            .expect("valid regex")
    });

/// Regex for identifying unquoted Obsidian wikilinks in YAML list items.
///
/// Pattern breakdown:
/// 1. `^(\s*-\s*)`: Matches the list dash and indentation.
/// 2. `([^"'\s].*\[\[.*\]\].*)`: Matches values starting with a non-quote/space
///    that contain a wikilink.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_LIST_LINK_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| {
        Regex::new(r#"(?m)^(\s*-\s*)([^"'\s].*\[\[.*\]\].*)$"#)
            .expect("valid regex")
    });

macro_rules! frontmatter_get_ops {
    ($name:ident, $type:ty) => {
        /// Extracts a value of the specified type.
        ///
        /// # Errors
        ///
        /// Returns a [`FrontmatterError`] if the type is incompatible.
        #[inline]
        pub fn $name(
            &self,
            key: &FrontmatterKey,
        ) -> Result<Option<$type>, FrontmatterError> {
            self.try_get::<$type>(key)
        }
    };
}

impl Frontmatter {
    frontmatter_get_ops!(try_get_bool, bool);

    frontmatter_get_ops!(try_get_str, Box<str>);

    frontmatter_get_ops!(try_get_number, f64);

    /// Parses a frontmatter block into structured fields.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterParseError`] if the content cannot be parsed or
    /// converted into supported field values.
    pub(crate) fn parse(
        format: RawFrontmatterFormat,
        text: &str,
    ) -> Result<Self, FrontmatterParseError> {
        match format {
            RawFrontmatterFormat::Yaml => {
                if let Ok(fm) = serde_yaml::from_str::<Self>(text) {
                    return Ok(fm);
                }
                let sanitized = Self::sanitize_yaml_obsidian_links(text);
                serde_yaml::from_str(&sanitized).map_err(|_e| {
                    FrontmatterParseError::InvalidYaml {
                        reason: "failed to parse yaml",
                    }
                })
            }
            RawFrontmatterFormat::Toml => toml::from_str(text).map_err(|_e| {
                FrontmatterParseError::InvalidToml {
                    reason: "failed to parse toml",
                }
            }),
        }
    }

    /// Creates a new [`Frontmatter`] instance from a field map.
    #[inline]
    #[must_use]
    pub fn new(fields: HashMap<Box<str>, FieldValue>) -> Self {
        Self {
            fields,
        }
    }

    /// Returns a reference to the value for the given key, if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// let mut fields = HashMap::new();
    /// fields.insert("key".into(), FieldValue::String("value".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// assert!(fm.get("key").is_some());
    /// assert!(fm.get("missing").is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&FieldValue> {
        self.fields.get(key.as_ref())
    }

    /// Returns `true` if the frontmatter contains a field with the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// let mut fields = HashMap::new();
    /// fields.insert("key".into(), FieldValue::String("value".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// assert!(fm.has("key"));
    /// assert!(!fm.has("missing"));
    /// ```
    #[inline]
    #[must_use]
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.fields.contains_key(key.as_ref())
    }

    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> FrontmatterFields<'_> {
        FrontmatterFields {
            inner: self.fields.iter(),
        }
    }

    /// Strictly extracts a typed value from frontmatter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// # use lithos_core::config::frontmatter::FrontmatterKey;
    /// let mut fields = HashMap::new();
    /// fields.insert("active".into(), FieldValue::Boolean(true));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let key = FrontmatterKey::try_new("active").unwrap();
    /// let value: bool = fm.try_get(&key).unwrap().unwrap();
    /// assert!(value);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] if the value exists but its type is
    /// incompatible with the requested type `T`.
    #[inline]
    pub fn try_get<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError> {
        let Some(value) = self.get(key.as_str()) else {
            return Ok(None);
        };
        T::try_from_value(value)
            .map(Some)
            .map_err(|err| FrontmatterError::from(err).with_key(key.as_str()))
    }

    /// Strictly extracts a required typed value from frontmatter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// # use lithos_core::config::frontmatter::FrontmatterKey;
    /// let mut fields = HashMap::new();
    /// fields.insert("title".into(), FieldValue::String("My Note".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let key = FrontmatterKey::try_new("title").unwrap();
    /// let title: Box<str> = fm.try_get_required(&key).unwrap();
    /// assert_eq!(title.as_ref(), "My Note");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`FrontmatterError::Missing`] if the key is absent.
    /// - [`FrontmatterError::TypeMismatch`] if the type is incompatible.
    #[inline]
    pub fn try_get_required<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<T, FrontmatterError> {
        self.try_get(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.as_str().into(),
        })
    }

    /// Strictly extracts a *borrowed* typed value from frontmatter.
    ///
    /// This is more efficient than [`try_get`][Self::try_get] for types like
    /// strings that can be borrowed directly from the frontmatter map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// # use lithos_core::config::frontmatter::FrontmatterKey;
    /// let mut fields = HashMap::new();
    /// fields.insert("title".into(), FieldValue::String("My Note".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let key = FrontmatterKey::try_new("title").unwrap();
    /// let title: &str = fm.try_get_ref(&key).unwrap().unwrap();
    /// assert_eq!(title, "My Note");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] if the value exists but its type is
    /// incompatible with the requested type `T`.
    #[inline]
    pub fn try_get_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError>
    where
        T: TryFromFieldValueRef<'frontmatter>,
    {
        let Some(value) = self.get(key.as_str()) else {
            return Ok(None);
        };
        T::try_from_value_ref(value)
            .map(Some)
            .map_err(|err| FrontmatterError::from(err).with_key(key.as_str()))
    }

    /// Returns the title of the note, using the configured title key.
    ///
    /// Uses the title key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
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
    ///
    /// Uses the file class key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.get(config.frontmatter().file_class().as_str())
            .and_then(FieldValue::as_str)
    }

    /// Returns a borrowed iterator over aliases.
    ///
    /// Handles both single string values and arrays of strings. Uses the
    /// alias key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// # use lithos_core::config::aggregate::{Config, Version};
    /// # use lithos_core::config::vault::{VaultId, VaultRoot};
    /// # use lithos_core::config::raw::RawConfig;
    /// # let config = Config::build(&RawConfig::default(), VaultId::new(), VaultRoot::try_new("/v".into()).unwrap(), Version::initial()).unwrap();
    /// let mut fields = HashMap::new();
    /// fields.insert("aliases".into(), FieldValue::Array(vec![FieldValue::String("a".into()), FieldValue::String("b".into())].into_boxed_slice()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let aliases: Vec<&str> = fm.aliases(&config).collect();
    /// assert_eq!(aliases, vec!["a", "b"]);
    /// ```
    #[inline]
    #[must_use]
    pub fn aliases<'frontmatter>(
        &'frontmatter self,
        config: &crate::config::aggregate::Config,
    ) -> AliasValues<'frontmatter> {
        AliasValues::new(self.get(config.frontmatter().alias().as_str()))
    }

    /// Returns the aliases of the note as a vector of boxed strings.
    ///
    /// This method allocates a [`Vec`] and converts each alias to an owned
    /// string. Prefer [`Frontmatter::aliases`] in performance-critical
    /// paths.
    #[inline]
    #[must_use]
    pub fn aliases_owned(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<Box<str>> {
        self.aliases(config).map(Into::into).collect()
    }

    fn sanitize_yaml_obsidian_links(text: &str) -> String {
        let step1 = YAML_MAP_LINK_RE.replace_all(text, r#"$1"$2""#);
        YAML_LIST_LINK_RE.replace_all(&step1, r#"$1"$2""#).into_owned()
    }
}

impl TryFrom<RawFrontmatter> for Frontmatter {
    type Error = FrontmatterParseError;

    #[inline]
    fn try_from(raw: RawFrontmatter) -> Result<Self, Self::Error> {
        Frontmatter::parse(raw.kind(), raw.text())
    }
}

/// Borrowed frontmatter fields iterator.
pub(crate) struct FrontmatterFields<'frontmatter> {
    inner: std::collections::hash_map::Iter<'frontmatter, Box<str>, FieldValue>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'frontmatter> Iterator for FrontmatterFields<'frontmatter> {
    type Item = (&'frontmatter str, &'frontmatter FieldValue);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, value)| (key.as_ref(), value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Borrowed alias iterator returned by [`Frontmatter::aliases`].
///
/// This iterator can handle multiple frontmatter alias formats, including:
/// - A single string value: `alias: my-alias`
/// - An array of string values: `aliases: [a, b]`
pub struct AliasValues<'frontmatter> {
    inner: AliasSource<'frontmatter>,
}

enum AliasSource<'frontmatter> {
    Empty,
    Single(Option<&'frontmatter str>),
    Array(std::slice::Iter<'frontmatter, FieldValue>),
}

impl<'frontmatter> AliasValues<'frontmatter> {
    fn new(value: Option<&'frontmatter FieldValue>) -> Self {
        let inner = if let Some(value) = value {
            if let Some(text) = value.as_str() {
                AliasSource::Single(Some(text))
            } else if let Some(values) = value.as_array() {
                AliasSource::Array(values.iter())
            } else {
                AliasSource::Empty
            }
        } else {
            AliasSource::Empty
        };
        Self {
            inner,
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'frontmatter> Iterator for AliasValues<'frontmatter> {
    type Item = &'frontmatter str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            AliasSource::Empty => None,
            AliasSource::Single(ref mut value) => value.take(),
            AliasSource::Array(ref mut iter) => {
                iter.find_map(|item| item.as_str())
            }
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            AliasSource::Empty | AliasSource::Single(None) => (0, Some(0)),
            AliasSource::Single(Some(_)) => (1, Some(1)),
            AliasSource::Array(iter) => {
                let (_, upper) = iter.size_hint();
                (0, upper)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Test fixtures and builders for Frontmatter tests.
    #[expect(
        dead_code,
        reason = "Fixture helpers are used by multiple test modules"
    )]
    mod fixtures {
        use chrono::{DateTime, TimeZone as _, Utc};

        use super::{super::*, TEST_TIMESTAMP};
        use crate::{config::aggregate::Config, note::error::NoteError};

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
                self.fields.insert(
                    key.into(),
                    FieldValue::Array(values.into_boxed_slice()),
                );
                self
            }

            #[expect(
                clippy::unnecessary_wraps,
                reason = "Fixture builder keeps Result parity with fallible \
                          builders"
            )]
            pub fn build(self) -> Result<Frontmatter, NoteError> {
                Ok(Frontmatter::new(self.fields))
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
                    tags_key: Some("labels".to_owned()),
                    title_key: Some("subject".to_owned()),
                }),
                ..Default::default()
            };

            crate::config::aggregate::Config::build(
                &raw,
                VaultId::new(),
                VaultRoot::try_new(std::path::PathBuf::from("/v"))
                    .expect("vault_root"),
                crate::config::aggregate::Version::initial(),
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
        }

        pub fn frontmatter_with_aliases_mixed() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert(
                "aliases".into(),
                FieldValue::Array(
                    vec![
                        FieldValue::String("ok".into()),
                        FieldValue::Number(123.0),
                    ]
                    .into_boxed_slice(),
                ),
            );
            Frontmatter::new(fields)
        }

        pub fn frontmatter_with_number() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("n".into(), FieldValue::Number(1.0f64));
            Frontmatter::new(fields)
        }

        pub fn frontmatter_with_invalid_date() -> Frontmatter {
            let mut fields = HashMap::new();
            fields.insert("d".into(), FieldValue::Date(i64::MAX));
            Frontmatter::new(fields)
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

        use super::{super::*, TEST_TIMESTAMP, fixtures};

        #[test]
        fn array_coerces_to_array() {
            let value = FieldValue::Array(
                vec![FieldValue::Boolean(true)].into_boxed_slice(),
            );
            assert!(
                value.array_items().is_some(),
                "Array should coerce to items"
            );
        }

        #[test]
        fn array_does_not_coerce_to_bool() {
            let value = FieldValue::Array(
                vec![FieldValue::Boolean(true)].into_boxed_slice(),
            );
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
                value.array_items().is_none(),
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
            use chrono::{TimeZone as _, Utc};
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
                value.object_fields().is_some(),
                "Object should coerce to fields"
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
                value.object_fields().is_none(),
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
            use chrono::{TimeZone as _, Utc};
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

    mod conversions {
        use chrono::{DateTime, Utc};

        use super::{super::*, fixtures};

        #[test]
        fn try_get_returns_string_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let key = FrontmatterKey::try_new("s").expect("valid key");
            let result = fm.try_get::<Box<str>>(&key);
            assert_eq!(
                result,
                Ok(Some("text".into())),
                "Should retrieve and convert String field"
            );
        }

        #[test]
        fn try_get_returns_boolean_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let key = FrontmatterKey::try_new("b").expect("valid key");
            let result = fm.try_get::<bool>(&key);
            assert_eq!(
                result,
                Ok(Some(true)),
                "Should retrieve and convert Boolean field"
            );
        }

        #[test]
        fn try_get_returns_number_value() {
            let fm = fixtures::frontmatter_for_try_get();
            let key = FrontmatterKey::try_new("n").expect("valid key");
            let result = fm.try_get::<f64>(&key);
            assert_eq!(
                result,
                Ok(Some(1.5f64)),
                "Should retrieve and convert Number field"
            );
        }

        #[test]
        fn try_get_returns_type_mismatch_error() {
            let fm = fixtures::frontmatter_for_try_get();
            let lookup_key = FrontmatterKey::try_new("s").expect("valid key");
            let result = fm.try_get::<bool>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::TypeMismatch {
                        key: error_key,
                        expected,
                        actual,
                    })
                        if error_key.as_ref() == "s"
                            && *expected == "boolean"
                            && *actual == "string"
                ),
                "type mismatch should error: {result:?}"
            );
        }

        #[test]
        fn strict_string_vec_errors_on_non_string_array_elements() {
            let fm = fixtures::frontmatter_with_aliases_mixed();
            let lookup_key =
                FrontmatterKey::try_new("aliases").expect("valid key");
            let result = fm.try_get_required::<Vec<Box<str>>>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::ArrayElementTypeMismatch {
                        key: error_key,
                        index: 1,
                        expected: "string",
                        actual: "number",
                    }) if error_key.as_ref() == "aliases"
                ),
                "strict extraction should fail: {result:?}"
            );
        }

        #[test]
        fn lenient_string_vec_drops_non_string_elements() {
            let fm = fixtures::frontmatter_with_aliases_mixed();
            assert_eq!(
                fm.get("aliases").and_then(string_array_lossy),
                Some(vec!["ok".into()])
            );
        }

        fn string_array_lossy(value: &FieldValue) -> Option<Vec<Box<str>>> {
            if let Some(items) = value.array_items() {
                return Some(
                    items
                        .filter_map(|item| item.as_str().map(Into::into))
                        .collect(),
                );
            }

            value.as_str().map(|s| vec![s.into()])
        }

        #[test]
        fn strict_get_required_reports_missing_key() {
            let fm = fixtures::frontmatter_with_number();
            let lookup_key =
                FrontmatterKey::try_new("missing").expect("valid key");
            let result = fm.try_get_required::<Box<str>>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::Missing { key: error_key })
                        if error_key.as_ref() == "missing"
                ),
                "missing key should error: {result:?}"
            );
        }

        #[test]
        fn strict_get_required_reports_type_mismatch() {
            let fm = fixtures::frontmatter_with_number();
            let lookup_key = FrontmatterKey::try_new("n").expect("valid key");
            let result = fm.try_get_required::<Box<str>>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::TypeMismatch {
                        key: error_key,
                        expected,
                        actual,
                    })
                        if error_key.as_ref() == "n"
                            && *expected == "string"
                            && *actual == "number"
                ),
                "type mismatch should error: {result:?}"
            );
        }

        #[test]
        fn strict_date_reports_invalid_timestamp() {
            let fm = fixtures::frontmatter_with_invalid_date();
            let lookup_key = FrontmatterKey::try_new("d").expect("valid key");
            let result = fm.try_get_required::<DateTime<Utc>>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::InvalidDateTimestamp {
                        key: error_key,
                        timestamp,
                    })
                        if error_key.as_ref() == "d" && *timestamp == i64::MAX
                ),
                "invalid timestamp should error: {result:?}"
            );
        }
    }

    const TEST_TIMESTAMP: i64 = 1_700_000_000;
}
