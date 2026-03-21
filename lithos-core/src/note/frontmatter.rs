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

// ----------------------------------------------------------- //
//                            Macro                            //
// ----------------------------------------------------------- //

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
            self.find_typed::<$type>(key)
                .map_err(|err| err.with_key(key.as_str()))
        }
    };
}

// ----------------------------------------------------------- //
//                     Main Frontmatter Type                   //
// ----------------------------------------------------------- //

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
    /// Aliases as they appear in the file (preserving singular vs plural).
    aliases: Option<AliasField>,
    /// Tags as they appear in the file.
    tags: Option<Box<[Tag]>>,
    /// File class as it appears in the file.
    file_class: Option<FileClassName>,
    /// Explicit creation date from frontmatter.
    date_created: Option<FrontmatterDateField>,
    /// Explicit modification date from frontmatter.
    date_modified: Option<FrontmatterDateField>,
}

impl Frontmatter {
    frontmatter_get_ops!(find_bool, bool);

    frontmatter_get_ops!(find_str, Box<str>);

    frontmatter_get_ops!(find_number, f64);

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

        let aliases = fields.get(fm_config.alias().as_str()).and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(AliasField::Single(AliasName(s.into())))
            } else if let Some(arr) = v.as_array() {
                let names = arr
                    .iter()
                    .filter_map(|item| {
                        item.as_str().map(|s| AliasName(s.into()))
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                Some(AliasField::List(names))
            } else {
                None
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
        aliases: Option<AliasField>,
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
    /// assert!(fm.find_field("key").is_some());
    /// assert!(fm.find_field("missing").is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn find_field<K: AsRef<str>>(&self, key: K) -> Option<&FieldValue> {
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
    pub(crate) fn list_fields(&self) -> FrontmatterFields<'_> {
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
    /// let value: bool = fm.find_typed(&key).unwrap().unwrap();
    /// assert!(value);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] if the value exists but its type is
    /// incompatible with the requested type `T`.
    #[inline]
    pub fn find_typed<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError> {
        let Some(value) = self.find_field(key.as_str()) else {
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
    /// let title: Box<str> = fm.get_typed(&key).unwrap();
    /// assert_eq!(title.as_ref(), "My Note");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`FrontmatterError::Missing`] if the key is absent.
    /// - [`FrontmatterError::TypeMismatch`] if the type is incompatible.
    #[inline]
    pub fn get_typed<T: TryFromFieldValue>(
        &self,
        key: &FrontmatterKey,
    ) -> Result<T, FrontmatterError> {
        self.find_typed(key)?
            .ok_or_else(|| FrontmatterError::KeyMissing {
                key: key.as_str().into(),
            })
            .map_err(|err| err.with_key(key.as_str()))
    }

    /// Strictly extracts a *borrowed* typed value from frontmatter.
    ///
    /// This is more efficient than [`find_typed`][Self::find_typed] for types
    /// like strings that can be borrowed directly from the frontmatter map.
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
    /// let title: &str = fm.find_typed_ref(&key).unwrap().unwrap();
    /// assert_eq!(title, "My Note");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`FrontmatterError`] if the value exists but its type is
    /// incompatible with the requested type `T`.
    #[inline]
    pub fn find_typed_ref<'frontmatter, T>(
        &'frontmatter self,
        key: &FrontmatterKey,
    ) -> Result<Option<T>, FrontmatterError>
    where
        T: TryFromFieldValueRef<'frontmatter>,
    {
        let Some(value) = self.find_field(key.as_str()) else {
            return Ok(None);
        };
        T::try_from_value_ref(value)
            .map(Some)
            .map_err(|err| err.with_key(key.as_str()))
    }

    /// Returns the title of the note.
    #[inline]
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the file class of the note.
    #[inline]
    #[must_use]
    pub fn file_class(&self) -> Option<&str> {
        self.file_class.as_ref().map(FileClassName::as_str)
    }

    /// Returns an iterator over all aliases.
    #[inline]
    pub fn aliases(&self) -> impl Iterator<Item = &str> {
        self.aliases
            .as_ref()
            .into_iter()
            .flat_map(AliasField::as_slice)
            .map(AliasName::as_str)
    }

    /// Returns the primary alias only if it was provided as a single string.
    #[inline]
    #[must_use]
    pub fn alias_str(&self) -> Option<&str> {
        self.aliases
            .as_ref()
            .and_then(AliasField::as_single)
            .map(AliasName::as_str)
    }

    /// Returns the aliases of the note as a vector of boxed strings.
    #[inline]
    #[must_use]
    pub fn to_aliases(&self) -> Vec<Box<str>> {
        self.aliases().map(Into::into).collect()
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

    fn sanitize_yaml_obsidian_links(text: &str) -> std::borrow::Cow<'_, str> {
        let step1 = YAML_MAP_LINK_RE.replace_all(text, r#"$1"$2""#);
        match step1 {
            std::borrow::Cow::Borrowed(_) => {
                YAML_LIST_LINK_RE.replace_all(text, r#"$1"$2""#)
            }
            std::borrow::Cow::Owned(s1) => {
                let step2 = YAML_LIST_LINK_RE.replace_all(&s1, r#"$1"$2""#);
                match step2 {
                    std::borrow::Cow::Borrowed(_) => {
                        std::borrow::Cow::Owned(s1)
                    }
                    std::borrow::Cow::Owned(s2) => std::borrow::Cow::Owned(s2),
                }
            }
        }
    }
}

// ----------------------------------------------------------- //
//                      Domain Logic Types                     //
// ----------------------------------------------------------- //

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
                reason: "alias cannot be an empty string; omit the value or \
                         delete the key if no alias is desired",
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
                reason: "file class cannot be an empty string; omit the value \
                         or delete the key if no file class is desired",
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

/// Represents the shape of alias declarations in frontmatter.
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
pub enum AliasField {
    /// Provided as a single string: `alias: my-alias`.
    Single(AliasName),
    /// Provided as a list: `aliases: [a, b]`.
    List(Box<[AliasName]>),
}

impl AliasField {
    /// Returns the aliases as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[AliasName] {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Explicit matching on references"
        )]
        match self {
            Self::Single(s) => std::slice::from_ref(s),
            Self::List(arr) => arr,
        }
    }

    /// Returns the alias only if it was provided as a single string.
    #[inline]
    #[must_use]
    pub fn as_single(&self) -> Option<&AliasName> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Explicit matching on references"
        )]
        if let Self::Single(s) = self {
            Some(s)
        } else {
            None
        }
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

// ----------------------------------------------------------- //
//                      Support Iterators                      //
// ----------------------------------------------------------- //

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

// ----------------------------------------------------------- //
//                      Internal Helpers                       //
// ----------------------------------------------------------- //

/// Regex for identifying unquoted Obsidian wikilinks in YAML mapping entries.
///
/// Pattern breakdown:
/// 1. `^(\s*[\w_-]+\s*:\s*)`: Matches the key and colon, including indentation.
/// 2. `([^"'\s|>].*?\[\[.*\]\].*|\[\[.*\]\].*)`: Matches values starting with a
///    non-quote/special char that contain a wikilink, or start with one.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_MAP_LINK_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(
    || {
        Regex::new(r#"(?m)^(\s*[\w_-]+\s*:\s*)([^"'\s|>].*?\[\[.*\]\].*|\[\[.*\]\].*)$"#)
            .expect("valid regex")
    },
);

/// Regex for identifying unquoted Obsidian wikilinks in YAML list items.
///
/// Pattern breakdown:
/// 1. `^(\s*-\s*)`: Matches the list dash and indentation.
/// 2. `([^"'\s].*?\[\[.*\]\].*|\[\[.*\]\].*)`: Matches values starting with a
///    non-quote/space that contain a wikilink, or start with one.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_LIST_LINK_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| {
        Regex::new(r#"(?m)^(\s*-\s*)([^"'\s].*?\[\[.*\]\].*|\[\[.*\]\].*)$"#)
            .expect("valid regex")
    });

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module organization"
)]
mod tests {
    mod fixtures {
        use std::collections::HashMap;

        use crate::{
            config::aggregate::Config,
            note::{frontmatter::Frontmatter, value::FieldValue},
        };

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

            #[expect(clippy::unused_self, reason = "Test helper ergonomics")]
            pub fn config(&self) -> Config {
                Self::build_config()
            }

            fn build_config() -> Config {
                use crate::config::{
                    aggregate::Version,
                    raw::RawConfig,
                    vault::{VaultId, VaultRoot},
                };
                crate::config::aggregate::Config::build(
                    &RawConfig::default(),
                    VaultId::new(),
                    VaultRoot::try_new(std::path::PathBuf::from("/v")).unwrap(),
                    Version::initial(),
                )
                .unwrap()
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
    }

    use rstest::rstest;

    use super::*;
    use crate::note::{error::FrontmatterError, frontmatter::FrontmatterKey};

    mod constructor {
        use super::*;

        #[test]
        fn should_create_empty_frontmatter() {
            let fm = Frontmatter::new(HashMap::new());
            assert!(fm.fields.is_empty(), "Frontmatter should be empty");
            assert!(fm.title().is_none());
        }

        #[test]
        fn should_extract_attributes_from_fields() {
            let config = fixtures::config_with_custom_frontmatter_keys();
            let mut fields = HashMap::new();
            fields.insert("subject".into(), FieldValue::String("Title".into()));
            fields.insert("kind".into(), FieldValue::String("Class".into()));

            let fm = Frontmatter::from_fields(fields, &config);

            assert_eq!(fm.title(), Some("Title"));
            assert_eq!(fm.file_class(), Some("Class"));
        }
    }

    mod parsing {
        use super::*;

        fn config_fixture() -> crate::config::aggregate::Config {
            fixtures::FrontmatterBuilder::new().config()
        }

        #[rstest]
        #[case::simple_yaml(
            RawFrontmatterFormat::Yaml,
            "title: Hello\ncount: 42",
            "title",
            FieldValue::String("Hello".into())
        )]
        #[case::simple_toml(
            RawFrontmatterFormat::Toml,
            "title = \"Hello\"\ncount = 42",
            "title",
            FieldValue::String("Hello".into())
        )]
        fn should_parse_valid_frontmatter(
            #[case] format: RawFrontmatterFormat,
            #[case] text: &str,
            #[case] key: &str,
            #[case] expected: FieldValue,
        ) {
            let config = config_fixture();
            let fm = Frontmatter::parse(format, text, &config)
                .expect("Parse should succeed");
            assert_eq!(fm.find_field(key), Some(&expected));
        }

        #[test]
        fn should_report_yaml_syntax_error_with_location() {
            let config = config_fixture();
            let text = "key: : invalid";
            let result =
                Frontmatter::parse(RawFrontmatterFormat::Yaml, text, &config);

            assert!(
                matches!(
                    result,
                    Err(NoteParseError::Frontmatter {
                        format: "YAML",
                        line: Some(_),
                        ..
                    })
                ),
                "Expected YAML error with location, got: {result:?}"
            );
        }

        #[rstest]
        #[case::map_link("link: [[Note]]", "link: \"[[Note]]\"")]
        #[case::list_link("- [[Note]]", "- \"[[Note]]\"")]
        #[case::mixed(
            "key: val\nlink: [[Note]]\n- [[Other]]",
            "key: val\nlink: \"[[Note]]\"\n- \"[[Other]]\""
        )]
        #[case::already_quoted("link: \"[[Note]]\"", "link: \"[[Note]]\"")]
        #[case::map_with_alias(
            "link: [[Note|Alias]]",
            "link: \"[[Note|Alias]]\""
        )]
        fn should_sanitize_obsidian_links(
            #[case] input: &str,
            #[case] expected: &str,
        ) {
            let sanitized = Frontmatter::sanitize_yaml_obsidian_links(input);
            assert_eq!(sanitized, expected);
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn alias_name_should_reject_empty_string() {
            let result = AliasName::try_new("");
            assert!(
                matches!(
                    result,
                    Err(NoteError::Frontmatter(
                        FrontmatterError::InvalidAlias { .. }
                    ))
                ),
                "Expected invalid alias error, got: {result:?}"
            );
        }

        #[test]
        fn file_class_name_should_reject_empty_string() {
            let result = FileClassName::try_new("");
            assert!(
                matches!(
                    result,
                    Err(NoteError::Frontmatter(
                        FrontmatterError::InvalidFileClass { .. }
                    ))
                ),
                "Expected invalid file class error, got: {result:?}"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn should_retrieve_typed_values() {
            let mut fields = HashMap::new();
            fields.insert("b".into(), FieldValue::Boolean(true));
            fields.insert("s".into(), FieldValue::String("text".into()));
            fields.insert("n".into(), FieldValue::Number(1.5f64));
            let fm = Frontmatter::new(fields);

            let key_b = FrontmatterKey::try_new("b").unwrap();
            let key_s = FrontmatterKey::try_new("s").unwrap();
            let key_n = FrontmatterKey::try_new("n").unwrap();

            assert_eq!(fm.find_typed::<bool>(&key_b).unwrap(), Some(true));
            assert_eq!(
                fm.find_typed::<Box<str>>(&key_s).unwrap(),
                Some("text".into())
            );
            assert_eq!(fm.find_typed::<f64>(&key_n).unwrap(), Some(1.5f64));
        }

        #[test]
        fn should_error_on_type_mismatch() {
            let mut fields = HashMap::new();
            fields.insert("s".into(), FieldValue::String("text".into()));
            let fm = Frontmatter::new(fields);
            let key = FrontmatterKey::try_new("s").unwrap();

            let result = fm.find_typed::<bool>(&key);
            assert!(
                matches!(
                    result,
                    Err(FrontmatterError::TypeMismatch {
                        expected: "boolean",
                        actual: "string",
                        ..
                    })
                ),
                "Expected type mismatch error, got: {result:?}"
            );
        }

        #[test]
        fn should_borrow_typed_values() {
            let mut fields = HashMap::new();
            fields.insert("s".into(), FieldValue::String("text".into()));
            let fm = Frontmatter::new(fields);
            let key = FrontmatterKey::try_new("s").unwrap();

            let val: &str = fm.find_typed_ref(&key).unwrap().unwrap();
            assert_eq!(val, "text");
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn aliases_should_flatten_single_and_list() {
            let config = fixtures::config_with_custom_frontmatter_keys();

            // Single
            let fm_single = fixtures::FrontmatterBuilder::new()
                .with_string("names", "A")
                .build_with_config(&config);
            assert_eq!(fm_single.aliases().collect::<Vec<_>>(), vec!["A"]);
            assert_eq!(fm_single.alias_str(), Some("A"));

            // List
            let fm_list = fixtures::FrontmatterBuilder::new()
                .with_array("names", vec![
                    FieldValue::String("A".into()),
                    FieldValue::String("B".into()),
                ])
                .build_with_config(&config);
            assert_eq!(fm_list.aliases().collect::<Vec<_>>(), vec!["A", "B"]);
            assert_eq!(fm_list.alias_str(), None);
        }

        #[test]
        fn tags_should_parse_from_string_or_list() {
            let config = fixtures::config_with_custom_frontmatter_keys();

            // String
            let fm_str = fixtures::FrontmatterBuilder::new()
                .with_string("labels", "#a, #b #c")
                .build_with_config(&config);
            assert_eq!(fm_str.tags.as_ref().unwrap().len(), 3);

            // List
            let fm_list = fixtures::FrontmatterBuilder::new()
                .with_array("labels", vec![
                    FieldValue::String("#a".into()),
                    FieldValue::String("#b".into()),
                ])
                .build_with_config(&config);
            assert_eq!(fm_list.tags.as_ref().unwrap().len(), 2);
        }
    }

    mod temporal {
        use super::*;

        #[rstest]
        #[case::iso("2024-03-21T14:30:00Z")]
        #[case::ymd_dash("2024-03-21")]
        #[case::ymd_slash("2024/03/21")]
        #[case::ymd_dot("2024.03.21")]
        #[case::dmy_dash("21-03-2024")]
        #[case::mdy_slash("03/21/2024")]
        fn should_parse_dates_heuristically(#[case] input: &str) {
            let val = FieldValue::String(input.into());
            let field = FrontmatterDateField::try_from_value(&val);
            assert!(
                field.is_some(),
                "Failed to parse date heuristically: {input}"
            );
        }

        #[test]
        fn should_reject_invalid_date_strings() {
            let val = FieldValue::String("not-a-date".into());
            let field = FrontmatterDateField::try_from_value(&val);
            assert!(field.is_none());
        }
    }

    mod proptests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            #[test]
            fn sanitize_is_idempotent(s in ".*") {
                let s1 = Frontmatter::sanitize_yaml_obsidian_links(&s);
                let s2 = Frontmatter::sanitize_yaml_obsidian_links(&s1);
                prop_assert_eq!(s1.as_ref(), s2.as_ref());
            }
        }
    }
}
