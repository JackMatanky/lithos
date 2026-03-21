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

use std::{collections::HashMap, fmt};

use chrono::{DateTime, FixedOffset, NaiveDate};
use regex::Regex;

use super::{
    error::{FrontmatterError, NoteError, NoteParseError},
    tag::Tag,
    value::{FieldValue, TryFromFieldValue, TryFromFieldValueRef},
};
use crate::{
    config::frontmatter::FrontmatterKey, note::raw::RawFrontmatterFormat,
};

/// Validated alias name for a note.
///
/// Aliases provide alternative names for notes, often used in `WikiLinks`
/// for easier discovery and linking.
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
#[rkyv(derive(Debug))]
pub struct AliasName(Box<str>);

impl AliasName {
    /// Creates a validated alias name.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterError::InvalidAlias`] if the alias is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.is_empty() {
            return Err(FrontmatterError::InvalidAlias {
                value: value.into(),
                reason: "alias cannot be empty",
            }
            .into());
        }
        Ok(Self(value.into()))
    }

    /// Returns the alias as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AliasName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated file class name for a note.
///
/// File classes are a convention used in many Obsidian workflows to categorize
/// notes and apply specific schema rules.
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
#[rkyv(derive(Debug))]
pub struct FileClassName(Box<str>);

impl FileClassName {
    /// Creates a validated file class name.
    ///
    /// # Errors
    ///
    /// Returns [`FrontmatterError::InvalidFileClass`] if the class is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.is_empty() {
            return Err(FrontmatterError::InvalidFileClass {
                value: value.into(),
                reason: "file class cannot be empty",
            }
            .into());
        }
        Ok(Self(value.into()))
    }

    /// Returns the file class as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileClassName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A specialized field for handling date and time metadata in frontmatter.
///
/// This type wraps a [`FieldValue`] and provides heuristic parsing for various
/// date and time formats commonly found in Markdown metadata. It is used
/// primarily for `date_created` and `date_modified` attributes.
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
pub struct FrontmatterDateField(FieldValue);

impl FrontmatterDateField {
    /// Attempts to create a date field from any [`FieldValue`].
    ///
    /// 1. If the value is already temporal (Date/DateTime), it is used as-is.
    /// 2. If it is a String, we attempt heuristic parsing against common
    ///    formats.
    #[inline]
    #[must_use]
    pub fn try_from_value(value: &FieldValue) -> Option<Self> {
        if value.is_temporal() {
            return Some(Self(value.clone()));
        }

        if let Some(s) = value.as_str() {
            return Self::parse_heuristically(s).map(Self);
        }

        None
    }

    /// Returns the inner [`FieldValue`].
    #[inline]
    #[must_use]
    pub const fn as_field_value(&self) -> &FieldValue {
        &self.0
    }

    /// Returns the value as a [`NaiveDate`] if possible.
    #[inline]
    #[must_use]
    pub fn as_naive_date(&self) -> Option<NaiveDate> {
        self.0
            .as_naive_date()
            .or_else(|| self.0.as_datetime().map(|dt| dt.date_naive()))
    }

    /// Returns the value as a [`DateTime<FixedOffset>`] if possible.
    #[inline]
    #[must_use]
    pub fn as_datetime(&self) -> Option<DateTime<FixedOffset>> {
        self.0.as_datetime()
    }

    /// Internal heuristic parsing for common Markdown metadata formats.
    fn parse_heuristically(s: &str) -> Option<FieldValue> {
        // 1. Try ISO 8601 / RFC 3339 (handled by chrono native)
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(FieldValue::DateTime(dt.into()));
        }

        // 2. Try common YMD formats (Standard)
        let ymd_formats = ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d"];
        for fmt in ymd_formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
                return Some(FieldValue::Date(d.into()));
            }
        }

        // 3. Try common DMY formats (International)
        let dmy_formats = ["%d-%m-%Y", "%d/%m/%Y", "%d.%m.%Y"];
        for fmt in dmy_formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
                return Some(FieldValue::Date(d.into()));
            }
        }

        // 4. Try common MDY formats (US)
        let mdy_formats = ["%m-%d-%Y", "%m/%d/%Y"];
        for fmt in mdy_formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
                return Some(FieldValue::Date(d.into()));
            }
        }

        None
    }
}

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
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    fields: HashMap<Box<str>, FieldValue>,
    /// Title as it appears in the file.
    title: Option<Box<str>>,
    /// Aliases as they appear in the file.
    aliases: Option<Box<[AliasName]>>,
    /// Tags as they appear in the file.
    tags: Option<Box<[Tag]>>,
    /// File class as it appears in the file.
    file_class: Option<FileClassName>,
    /// Explicit creation date from frontmatter.
    date_created: Option<FrontmatterDateField>,
    /// Explicit modification date from frontmatter.
    date_modified: Option<FrontmatterDateField>,
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
            self.try_get::<$type>(key).map_err(|err| err.with_key(key.as_str()))
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
    /// Returns [`NoteParseError`] if the content cannot be parsed or
    /// converted into supported field values.
    pub(crate) fn parse(
        format: RawFrontmatterFormat,
        text: &str,
        config: &crate::config::aggregate::Config,
    ) -> Result<Self, NoteParseError> {
        let fields: HashMap<Box<str>, FieldValue> = match format {
            RawFrontmatterFormat::Yaml => {
                let sanitized = Self::sanitize_yaml_obsidian_links(text);
                serde_yaml::from_str(&sanitized).map_err(|e| {
                    let location = e.location();
                    NoteParseError::Frontmatter {
                        format: "YAML",
                        line: location.as_ref().map(serde_yaml::Location::line),
                        column: location
                            .as_ref()
                            .map(serde_yaml::Location::column),
                        reason: e.to_string().into(),
                    }
                })?
            }
            RawFrontmatterFormat::Toml => {
                toml::from_str(text).map_err(|e| {
                    NoteParseError::Frontmatter {
                        format: "TOML",
                        line: None,
                        column: None,
                        reason: e.to_string().into(),
                    }
                })?
            }
        };

        Ok(Self::from_fields(fields, config))
    }

    /// Creates a new [`Frontmatter`] instance from a field map, extracting
    /// explicit attributes using the provided configuration.
    #[inline]
    #[must_use]
    pub fn from_fields(
        fields: HashMap<Box<str>, FieldValue>,
        config: &crate::config::aggregate::Config,
    ) -> Self {
        let fm_config = config.frontmatter();

        let title = fields
            .get(fm_config.title().as_str())
            .and_then(FieldValue::as_str)
            .map(Into::into);

        let aliases = fields.get(fm_config.alias().as_str()).map(|v| {
            if let Some(s) = v.as_str() {
                vec![AliasName(s.into())].into_boxed_slice()
            } else if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|item| {
                        item.as_str().map(|s| AliasName(s.into()))
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            } else {
                Box::new([])
            }
        });

        let tags = fields.get(fm_config.tags().as_str()).map(|v| {
            let mut tags = Vec::new();
            Self::collect_tags_from_value(v, &mut tags);
            tags.into_boxed_slice()
        });

        let file_class = fields
            .get(fm_config.file_class().as_str())
            .and_then(FieldValue::as_str)
            .map(|s| FileClassName(s.into()));

        let date_created = fields
            .get(fm_config.date_created().as_str())
            .and_then(FrontmatterDateField::try_from_value);

        let date_modified = fields
            .get(fm_config.date_modified().as_str())
            .and_then(FrontmatterDateField::try_from_value);

        Self {
            fields,
            title,
            aliases,
            tags,
            file_class,
            date_created,
            date_modified,
        }
    }

    fn collect_tags_from_value(value: &FieldValue, out: &mut Vec<Tag>) {
        if let Some(text) = value.as_str() {
            Self::collect_tags_from_str(text, out);
        } else if let Some(values) = value.as_array() {
            for item in values {
                if let Some(text) = item.as_str() {
                    Self::collect_tags_from_str(text, out);
                }
            }
        } else {
            // Non-taggable value types are ignored.
        }
    }

    fn collect_tags_from_str(text: &str, out: &mut Vec<Tag>) {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if !token.is_empty()
                && let Ok(tag) = Tag::try_from_token(token)
            {
                out.push(tag);
            }
        }
    }

    /// Creates a new [`Frontmatter`] instance with explicit values.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if any invariant is violated.
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Explicit attribute construction"
    )]
    pub fn try_new(
        fields: HashMap<Box<str>, FieldValue>,
        title: Option<Box<str>>,
        aliases: Option<Box<[AliasName]>>,
        tags: Option<Box<[Tag]>>,
        file_class: Option<FileClassName>,
        date_created: Option<FrontmatterDateField>,
        date_modified: Option<FrontmatterDateField>,
    ) -> Result<Self, NoteError> {
        Ok(Self {
            fields,
            title,
            aliases,
            tags,
            file_class,
            date_created,
            date_modified,
        })
    }

    /// Creates a new [`Frontmatter`] instance from a field map.
    ///
    /// Use [`from_fields`][Self::from_fields] to extract explicit attributes
    /// using a configuration.
    #[inline]
    #[must_use]
    pub fn new(fields: HashMap<Box<str>, FieldValue>) -> Self {
        Self {
            fields,
            title: None,
            aliases: None,
            tags: None,
            file_class: None,
            date_created: None,
            date_modified: None,
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
            .map_err(|err| err.with_key(key.as_str()))
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
        self.try_get(key)?
            .ok_or_else(|| FrontmatterError::KeyMissing {
                key: key.as_str().into(),
            })
            .map_err(|err| err.with_key(key.as_str()))
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
            .map_err(|err| err.with_key(key.as_str()))
    }

    /// Returns the title of the note, using the configured title key.
    ///
    /// Uses the title key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
    #[inline]
    #[must_use]
    pub fn title(
        &self,
        _config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the file class of the note, using the configured key.
    ///
    /// Uses the file class key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
    #[inline]
    #[must_use]
    pub fn file_class(
        &self,
        _config: &crate::config::aggregate::Config,
    ) -> Option<&str> {
        self.file_class.as_ref().map(FileClassName::as_str)
    }

    /// Returns a borrowed iterator over aliases.
    ///
    /// Handles both single string values and arrays of strings. Uses the
    /// alias key defined in the provided
    /// [`Config`][crate::config::aggregate::Config].
    #[inline]
    #[must_use]
    pub fn aliases<'frontmatter>(
        &'frontmatter self,
        _config: &crate::config::aggregate::Config,
    ) -> AliasValues<'frontmatter> {
        AliasValues::new(self.aliases.as_deref())
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

    /// Returns the explicit creation date of the note, if present in
    /// frontmatter.
    #[inline]
    #[must_use]
    pub const fn date_created(&self) -> Option<&FrontmatterDateField> {
        self.date_created.as_ref()
    }

    /// Returns the explicit modification date of the note, if present in
    /// frontmatter.
    #[inline]
    #[must_use]
    pub const fn date_modified(&self) -> Option<&FrontmatterDateField> {
        self.date_modified.as_ref()
    }

    fn sanitize_yaml_obsidian_links(text: &str) -> String {
        let step1 = YAML_MAP_LINK_RE.replace_all(text, r#"$1"$2""#);
        YAML_LIST_LINK_RE.replace_all(&step1, r#"$1"$2""#).into_owned()
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
    Array(std::slice::Iter<'frontmatter, AliasName>),
}

impl<'frontmatter> AliasValues<'frontmatter> {
    fn new(aliases: Option<&'frontmatter [AliasName]>) -> Self {
        let inner = if let Some(aliases) = aliases {
            AliasSource::Array(aliases.iter())
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
            AliasSource::Array(ref mut iter) => {
                iter.next().map(AliasName::as_str)
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
            AliasSource::Empty => (0, Some(0)),
            AliasSource::Array(iter) => iter.size_hint(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module organization"
)]
mod tests {
    /// Test fixtures and builders for Frontmatter tests.
    #[expect(
        dead_code,
        reason = "Fixture helpers are used by multiple test modules"
    )]
    mod fixtures {
        use std::collections::HashMap;

        use chrono::{FixedOffset, TimeZone as _, Utc};

        use crate::{
            config::aggregate::Config,
            note::{frontmatter::Frontmatter, value::FieldValue},
        };

        pub const TEST_TIMESTAMP: i64 = 1_700_000_000;

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
                let dt = Utc
                    .timestamp_opt(timestamp, 0)
                    .single()
                    .expect("valid timestamp");
                self.fields.insert(
                    key.into(),
                    FieldValue::DateTime(
                        dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
                            .into(),
                    ),
                );
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

            pub fn build(self) -> Frontmatter {
                use crate::config::{
                    aggregate::Version,
                    raw::RawConfig,
                    vault::{VaultId, VaultRoot},
                };
                let config = crate::config::aggregate::Config::build(
                    &RawConfig::default(),
                    VaultId::new(),
                    VaultRoot::try_new(std::path::PathBuf::from("/v")).unwrap(),
                    Version::initial(),
                )
                .unwrap();
                Frontmatter::from_fields(self.fields, &config)
            }

            pub fn build_with_config(self, config: &Config) -> Frontmatter {
                Frontmatter::from_fields(self.fields, config)
            }
        }

        pub fn config_with_custom_frontmatter_keys() -> Config {
            use crate::config::{
                aggregate::Version,
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
                Version::initial(),
            )
            .expect("Config build should succeed")
        }

        pub fn frontmatter_with_custom_keys() -> Frontmatter {
            let config = config_with_custom_frontmatter_keys();
            FrontmatterBuilder::new()
                .with_string("subject", "Subj")
                .with_string("kind", "Note")
                .with_string("names", "Alias")
                .build_with_config(&config)
        }

        pub fn frontmatter_with_title() -> Frontmatter {
            FrontmatterBuilder::new().with_string("title", "Test").build()
        }

        pub fn frontmatter_with_string_arrays() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_string("single", "a")
                .with_array("multi", vec![FieldValue::String("b".into())])
                .build()
        }

        pub fn frontmatter_with_scalar_values() -> Frontmatter {
            FrontmatterBuilder::new()
                .with_boolean("b", true)
                .with_number("n", 1.0)
                .with_string("s", "s")
                .with_date("d", TEST_TIMESTAMP)
                .build()
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
            // Boolean is not a temporal type
            fields.insert("d".into(), FieldValue::Boolean(true));
            // Actually Frontmatter struct now caches these, so we use try_new.
            Frontmatter::try_new(fields, None, None, None, None, None, None)
                .unwrap()
        }

        pub fn sample_datetime() -> chrono::DateTime<Utc> {
            Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0)
                .single()
                .expect("Valid date should be created")
        }
    }

    use chrono::{DateTime, NaiveDate, Utc};

    use super::*;
    use crate::note::{error::FrontmatterError, frontmatter::FrontmatterKey};

    mod field_value {
        #![allow(
            clippy::items_after_statements,
            clippy::no_effect_underscore_binding,
            reason = "Test code style"
        )]

        use super::*;

        #[test]
        fn date_field_parses_iso_8601_string() {
            let s = "2024-03-21T14:30:00+01:00";
            let val = FieldValue::String(s.into());
            let field = FrontmatterDateField::try_from_value(&val).unwrap();
            assert!(matches!(field.as_field_value(), FieldValue::DateTime(_)));
            assert_eq!(
                field.as_datetime().unwrap().to_rfc3339(),
                "2024-03-21T14:30:00+01:00"
            );
        }

        #[test]
        fn date_field_parses_standard_ymd_variants() {
            let variants = ["2024-03-21", "2024/03/21", "2024.03.21"];
            for s in variants {
                let val = FieldValue::String(s.into());
                let field = FrontmatterDateField::try_from_value(&val).unwrap();
                assert!(
                    matches!(field.as_field_value(), FieldValue::Date(_)),
                    "Failed to parse variant: {s}"
                );
                assert_eq!(
                    field.as_naive_date().unwrap(),
                    NaiveDate::from_ymd_opt(2024, 3, 21).unwrap()
                );
            }
        }

        #[test]
        fn date_field_parses_international_dmy_variants() {
            let variants = ["21-03-2024", "21/03/2024", "21.03.2024"];
            for s in variants {
                let val = FieldValue::String(s.into());
                let field = FrontmatterDateField::try_from_value(&val).unwrap();
                assert!(
                    matches!(field.as_field_value(), FieldValue::Date(_)),
                    "Failed to parse variant: {s}"
                );
                assert_eq!(
                    field.as_naive_date().unwrap(),
                    NaiveDate::from_ymd_opt(2024, 3, 21).unwrap()
                );
            }
        }

        #[test]
        fn date_field_ignores_invalid_format_strings() {
            let val = FieldValue::String("not a date".into());
            let field = FrontmatterDateField::try_from_value(&val);
            assert!(field.is_none());
        }

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
        fn date_coerces_to_naive_date() {
            let date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
            let value = FieldValue::Date(date.into());
            assert!(
                value.as_naive_date().is_some(),
                "Date should coerce to NaiveDate"
            );
        }

        #[test]
        fn date_coerces_to_datetime_via_try_get() {
            let date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
            let mut fields = HashMap::new();
            fields.insert("d".into(), FieldValue::Date(date.into()));
            let fm = Frontmatter::new(fields);
            let key = FrontmatterKey::try_new("d").unwrap();
            let dt: Result<Option<DateTime<Utc>>, _> = fm.try_get(&key);
            assert!(dt.is_ok());
            assert!(dt.unwrap().is_some());
        }

        #[test]
        fn date_does_not_coerce_to_number() {
            let date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
            let value = FieldValue::Date(date.into());
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
                value.as_naive_date().is_none(),
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
        fn date_field_returns_naive_date() {
            let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            let val = FieldValue::Date(date.into());
            assert_eq!(
                val.as_naive_date(),
                Some(date),
                "Date field should return NaiveDate"
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

    mod conversions {
        use super::*;

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
                    Err(FrontmatterError::TypeMismatch {
                        key: error_key,
                        expected: "string",
                        ..
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
            if let Some(items) = value.as_array() {
                let out = items
                    .iter()
                    .filter_map(FieldValue::as_str)
                    .map(Into::into)
                    .collect();
                return Some(out);
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
                    Err(FrontmatterError::KeyMissing { key: error_key })
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
        fn strict_date_reports_type_mismatch_on_non_date() {
            let fm = fixtures::frontmatter_with_invalid_date();
            let lookup_key = FrontmatterKey::try_new("d").expect("valid key");
            let result = fm.try_get_required::<DateTime<Utc>>(&lookup_key);
            assert!(
                matches!(
                    &result,
                    Err(FrontmatterError::TypeMismatch {
                        key: error_key,
                        expected: "datetime",
                        ..
                    }) if error_key.as_ref() == "d"
                ),
                "type mismatch should error: {result:?}"
            );
        }
    }
}
