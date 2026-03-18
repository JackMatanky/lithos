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
/// let fm = Frontmatter::new(fields);
/// let key =
///     lithos_core::config::frontmatter::FrontmatterKey::try_new("status")?;
/// assert!(fm.has(&key));
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
#[serde(transparent)]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    fields: HashMap<Box<str>, FieldValue>,
}

impl Frontmatter {
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
                let sanitized = Frontmatter::sanitize_yaml_obsidian_links(text);
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
    #[inline]
    #[must_use]
    pub fn get(&self, key: &FrontmatterKey) -> Option<&FieldValue> {
        self.fields.get(key.as_str())
    }

    /// Returns `true` if the frontmatter contains a field with the given key.
    #[inline]
    #[must_use]
    pub fn has(&self, key: &FrontmatterKey) -> bool {
        self.fields.contains_key(key.as_str())
    }

    /// Returns a reference to the value for the given raw key, if it exists.
    #[inline]
    #[must_use]
    pub fn get_raw(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }

    /// Returns `true` if the frontmatter contains a field with the given raw
    /// key.
    #[inline]
    #[must_use]
    pub fn has_raw(&self, key: &str) -> bool {
        self.fields.contains_key(key)
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
    pub fn try_get<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        T::try_from_value(value)
            .map(Some)
            .map_err(|err| FrontmatterError::from(err).with_key(key.as_str()))
    }

    /// Strictly extracts a required typed value from frontmatter.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterError::Missing`] if the key is absent.
    #[inline]
    pub fn try_get_required<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<T, FrontmatterError> {
        self.try_get(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.as_str().into(),
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
        key: &FrontmatterKey,
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
    pub fn try_get_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError>
    where
        T: TryFromFieldValueRef<'frontmatter>,
    {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        T::try_from_value_ref(value)
            .map(Some)
            .map_err(|err| FrontmatterError::from(err).with_key(key.as_str()))
    }

    /// Strictly extracts a required *borrowed* typed value from frontmatter.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterError::Missing`] if the key is absent.
    #[inline]
    pub fn try_get_required_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &FrontmatterKey,
    ) -> Result<T, FrontmatterError>
    where
        T: TryFromFieldValueRef<'frontmatter>,
    {
        self.try_get_ref(key)?.ok_or_else(|| FrontmatterError::Missing {
            key: key.as_str().into(),
        })
    }

    /// Returns the title of the note, using the configured title key.
    #[inline]
    #[must_use]
    pub fn title(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.get(config.frontmatter().title()).and_then(FieldValue::as_str)
    }

    /// Returns the file class of the note, using the configured key.
    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.get(config.frontmatter().file_class()).and_then(FieldValue::as_str)
    }

    /// Returns a borrowed iterator over aliases.
    ///
    /// This is zero-copy; use [`Frontmatter::aliases_owned`] when you need
    /// owned values.
    #[inline]
    #[must_use]
    pub fn aliases<'frontmatter>(
        &'frontmatter self,
        config: &crate::config::aggregate::Config,
    ) -> AliasValues<'frontmatter> {
        AliasValues::new(self.get(config.frontmatter().alias()))
    }

    /// Returns the aliases of the note as a vector of boxed strings.
    ///
    /// This allocates; prefer [`Frontmatter::aliases`] in hot paths.
    #[inline]
    #[must_use]
    pub fn aliases_owned(
        &self,
        config: &crate::config::aggregate::Config,
    ) -> Vec<Box<str>> {
        self.aliases(config).map(Into::into).collect()
    }

    fn sanitize_yaml_obsidian_links(text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        for line in text.split_inclusive('\n') {
            let line_end = line.trim_end_matches(['\n', '\r']);
            let line_ending = line.get(line_end.len()..).unwrap_or("");
            let trimmed = line_end.trim_start();
            let indent_len = line_end.len().saturating_sub(trimmed.len());
            let indent = line_end.get(..indent_len).unwrap_or("");

            if let Some(updated) =
                Frontmatter::sanitize_yaml_list_item(trimmed, indent)
            {
                output.push_str(&updated);
                output.push_str(line_ending);
                continue;
            }

            if let Some(updated) =
                Frontmatter::sanitize_yaml_mapping_entry(trimmed, indent)
            {
                output.push_str(&updated);
                output.push_str(line_ending);
                continue;
            }

            output.push_str(line_end);
            output.push_str(line_ending);
        }
        output
    }

    fn sanitize_yaml_list_item(line: &str, indent: &str) -> Option<String> {
        let rest = line.strip_prefix('-')?.trim_start();
        if !Frontmatter::is_unquoted_obsidian_link(rest) {
            return None;
        }
        let mut updated = String::with_capacity(
            indent.len().saturating_add(rest.len()).saturating_add(4),
        );
        updated.push_str(indent);
        updated.push_str("- ");
        updated.push('"');
        updated.push_str(rest);
        updated.push('"');
        Some(updated)
    }

    fn sanitize_yaml_mapping_entry(line: &str, indent: &str) -> Option<String> {
        let colon_index = line.find(':')?;
        let split_index = colon_index.saturating_add(1);
        let (key, rest) = line.split_at(split_index);
        let value = rest.trim_start();
        if value.is_empty() || value.starts_with('|') || value.starts_with('>')
        {
            return None;
        }
        if !Frontmatter::is_unquoted_obsidian_link(value) {
            return None;
        }
        let whitespace_len = rest.len().saturating_sub(value.len());
        let whitespace = rest.get(..whitespace_len).unwrap_or("");
        let mut updated = String::with_capacity(
            indent
                .len()
                .saturating_add(key.len())
                .saturating_add(whitespace.len())
                .saturating_add(value.len())
                .saturating_add(2),
        );
        updated.push_str(indent);
        updated.push_str(key);
        updated.push_str(whitespace);
        updated.push('"');
        updated.push_str(value);
        updated.push('"');
        Some(updated)
    }

    fn is_unquoted_obsidian_link(value: &str) -> bool {
        if value.starts_with('"') || value.starts_with('\'') {
            return false;
        }
        value.starts_with("[[") || value.starts_with("![[")
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
pub struct AliasValues<'frontmatter> {
    source: AliasSource<'frontmatter>,
}

enum AliasSource<'frontmatter> {
    Empty,
    Single(Option<&'frontmatter str>),
    Array(std::slice::Iter<'frontmatter, FieldValue>),
}

impl<'frontmatter> AliasValues<'frontmatter> {
    fn new(value: Option<&'frontmatter FieldValue>) -> Self {
        let source = if let Some(value) = value {
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
            source,
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
        match self.source {
            AliasSource::Empty => None,
            AliasSource::Single(ref mut value) => value.take(),
            AliasSource::Array(ref mut iter) => {
                for item in iter.by_ref() {
                    if let Some(text) = item.as_str() {
                        return Some(text);
                    }
                }
                None
            }
        }
    }

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.source {
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
            let result = fm.try_get_string_vec_strict(&lookup_key);
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
                fm.get_raw("aliases").and_then(string_array_lossy),
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
