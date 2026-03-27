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

use super::{
    error::{FrontmatterError, NoteError},
    tag::Tag,
    value::{FieldValue, TryFromFieldValue, TryFromFieldValueRef},
};
use crate::{
    config::frontmatter::FrontmatterConfigSpec, note::raw::RawFrontmatter,
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
            key: &str,
        ) -> Result<Option<$type>, FrontmatterError> {
            self.find_typed::<$type>(key).map_err(|err| err.with_key(key))
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
    aliases: Option<AliasValueKind>,
    /// Tags as they appear in the file.
    tags: Option<Box<[Tag]>>,
    /// File class as it appears in the file.
    file_class: Option<FileClassValue>,
    /// Explicit creation date from frontmatter.
    date_created: Option<FrontmatterDateValue>,
    /// Explicit modification date from frontmatter.
    date_modified: Option<FrontmatterDateValue>,
}

impl Frontmatter {
    frontmatter_get_ops!(find_bool, bool);

    frontmatter_get_ops!(find_str, Box<str>);

    frontmatter_get_ops!(find_number, f64);

    /// Creates a new [`Frontmatter`] instance from a field map, extracting
    /// explicit attributes using the provided configuration spec.
    #[inline]
    #[must_use]
    pub fn from_fields(
        fields: HashMap<Box<str>, FieldValue>,
        spec: &FrontmatterConfigSpec,
    ) -> Self {
        let title = fields
            .get(spec.title_key.as_ref())
            .and_then(FieldValue::as_str)
            .map(Into::into);

        let aliases = fields.get(spec.alias_key.as_ref()).and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(AliasValueKind::Single(AliasValue(s.into())))
            } else if let Some(arr) = v.as_array() {
                let names = arr
                    .iter()
                    .filter_map(|item| {
                        item.as_str().map(|s| AliasValue(s.into()))
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                Some(AliasValueKind::List(names))
            } else {
                None
            }
        });

        let tags = fields.get(spec.tags_key.as_ref()).map(|v| {
            let mut tags = Vec::new();
            Self::collect_tags_from_value(v, &mut tags);
            tags.into_boxed_slice()
        });

        let file_class = fields
            .get(spec.file_class_key.as_ref())
            .and_then(FieldValue::as_str)
            .map(|s| FileClassValue(s.into()));

        let date_created = fields
            .get(spec.date_created_key.as_ref())
            .and_then(FrontmatterDateValue::try_from_value);

        let date_modified = fields
            .get(spec.date_modified_key.as_ref())
            .and_then(FrontmatterDateValue::try_from_value);

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
                && let Ok(tag) = Tag::try_from(token)
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
        aliases: Option<AliasValueKind>,
        tags: Option<Box<[Tag]>>,
        file_class: Option<FileClassValue>,
        date_created: Option<FrontmatterDateValue>,
        date_modified: Option<FrontmatterDateValue>,
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
    /// let mut fields = HashMap::new();
    /// fields.insert("active".into(), FieldValue::Boolean(true));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let value: bool = fm.find_typed("active").unwrap().unwrap();
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
        key: &str,
    ) -> Result<Option<T>, FrontmatterError> {
        let Some(value) = self.find_field(key) else {
            return Ok(None);
        };
        T::try_from_value(value).map(Some).map_err(|err| err.with_key(key))
    }

    /// Strictly extracts a required typed value from frontmatter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::frontmatter::Frontmatter;
    /// # use lithos_core::note::value::FieldValue;
    /// let mut fields = HashMap::new();
    /// fields.insert("title".into(), FieldValue::String("My Note".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let title: Box<str> = fm.get_typed("title").unwrap();
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
        key: &str,
    ) -> Result<T, FrontmatterError> {
        self.find_typed(key)?
            .ok_or_else(|| FrontmatterError::KeyMissing {
                key: key.into(),
            })
            .map_err(|err| err.with_key(key))
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
    /// let mut fields = HashMap::new();
    /// fields.insert("title".into(), FieldValue::String("My Note".into()));
    /// let fm = Frontmatter::new(fields);
    ///
    /// let title: &str = fm.find_typed_ref("title").unwrap().unwrap();
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
        key: &str,
    ) -> Result<Option<T>, FrontmatterError>
    where
        T: TryFromFieldValueRef<'frontmatter>,
    {
        let Some(value) = self.find_field(key) else {
            return Ok(None);
        };
        T::try_from_value_ref(value).map(Some).map_err(|err| err.with_key(key))
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
        self.file_class.as_ref().map(FileClassValue::as_str)
    }

    /// Returns an iterator over all aliases.
    #[inline]
    pub fn aliases(&self) -> impl Iterator<Item = &str> {
        self.aliases
            .as_ref()
            .into_iter()
            .flat_map(AliasValueKind::as_slice)
            .map(AliasValue::as_str)
    }

    /// Returns the collection of tags extracted from the frontmatter.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> Option<&[Tag]> {
        self.tags.as_deref()
    }

    /// Returns the primary alias only if it was provided as a single string.
    #[inline]
    #[must_use]
    pub fn alias_str(&self) -> Option<&str> {
        self.aliases
            .as_ref()
            .and_then(AliasValueKind::as_single)
            .map(AliasValue::as_str)
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
    pub const fn date_created(&self) -> Option<&FrontmatterDateValue> {
        self.date_created.as_ref()
    }

    /// Returns the explicit modification date of the note, if present in
    /// frontmatter.
    #[inline]
    #[must_use]
    pub const fn date_modified(&self) -> Option<&FrontmatterDateValue> {
        self.date_modified.as_ref()
    }
}

impl<'source> TryFrom<(&RawFrontmatter<'source>, &FrontmatterConfigSpec)>
    for Frontmatter
{
    type Error = NoteError;

    #[inline]
    fn try_from(
        (raw, spec): (&RawFrontmatter<'source>, &FrontmatterConfigSpec),
    ) -> Result<Self, Self::Error> {
        let fields = raw.parse_fields().map_err(NoteError::from)?;
        Ok(Self::from_fields(fields, spec))
    }
}

impl<'source> TryFrom<(RawFrontmatter<'source>, &FrontmatterConfigSpec)>
    for Frontmatter
{
    type Error = NoteError;

    #[inline]
    fn try_from(
        (raw, spec): (RawFrontmatter<'source>, &FrontmatterConfigSpec),
    ) -> Result<Self, Self::Error> {
        Self::try_from((&raw, spec))
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
pub struct AliasValue(Box<str>);

impl AliasValue {
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

impl fmt::Display for AliasValue {
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
pub struct FileClassValue(Box<str>);

impl FileClassValue {
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

impl fmt::Display for FileClassValue {
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
pub enum AliasValueKind {
    /// Provided as a single string: `alias: my-alias`.
    Single(AliasValue),
    /// Provided as a list: `aliases: [a, b]`.
    List(Box<[AliasValue]>),
}

impl AliasValueKind {
    /// Returns the aliases as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[AliasValue] {
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
    pub fn as_single(&self) -> Option<&AliasValue> {
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
pub struct FrontmatterDateValue(FieldValue);

impl FrontmatterDateValue {
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
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[fixture]
    fn default_spec() -> FrontmatterConfigSpec {
        FrontmatterConfigSpec::new(
            "title".into(),
            "aliases".into(),
            "tags".into(),
            "file_class".into(),
            "date_created".into(),
            "date_modified".into(),
        )
    }

    #[fixture]
    fn custom_spec() -> FrontmatterConfigSpec {
        FrontmatterConfigSpec::new(
            "subject".into(),
            "names".into(),
            "labels".into(),
            "kind".into(),
            "created".into(),
            "updated".into(),
        )
    }

    mod constructor {
        use super::*;

        #[test]
        fn should_create_empty_frontmatter() {
            let fm = Frontmatter::new(HashMap::new());
            assert!(
                fm.fields.is_empty(),
                "Expected empty fields, got: {:?}",
                fm.fields
            );
            assert!(fm.title().is_none(), "Expected no title");
        }

        #[rstest]
        fn should_extract_attributes_from_fields_with_default_spec(
            default_spec: FrontmatterConfigSpec,
        ) {
            let mut fields = HashMap::new();
            fields
                .insert("title".into(), FieldValue::String("Test Note".into()));
            fields.insert(
                "aliases".into(),
                FieldValue::Array(
                    vec![FieldValue::String("A1".into())].into_boxed_slice(),
                ),
            );
            fields.insert("tags".into(), FieldValue::String("t1, t2".into()));

            let fm = Frontmatter::from_fields(fields, &default_spec);

            assert_eq!(
                fm.title(),
                Some("Test Note"),
                "Title mismatch with default spec"
            );
            assert_eq!(
                fm.aliases().collect::<Vec<_>>(),
                vec!["A1"],
                "Aliases mismatch"
            );
            assert_eq!(
                fm.tags().map(<[Tag]>::len),
                Some(2),
                "Tags count mismatch"
            );
        }

        #[rstest]
        fn should_extract_attributes_from_fields_with_custom_spec(
            custom_spec: FrontmatterConfigSpec,
        ) {
            let mut fields = HashMap::new();
            fields.insert(
                "subject".into(),
                FieldValue::String("Custom Subject".into()),
            );
            fields.insert("names".into(), FieldValue::String("Alias1".into()));

            let fm = Frontmatter::from_fields(fields, &custom_spec);

            assert_eq!(
                fm.title(),
                Some("Custom Subject"),
                "Title mismatch with custom spec"
            );
            assert_eq!(
                fm.aliases().collect::<Vec<_>>(),
                vec!["Alias1"],
                "Alias mismatch (single string)"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn alias_name_should_reject_empty_string() {
            let res = AliasValue::try_new("");
            assert!(
                res.is_err(),
                "Expected error for empty alias, got: {res:?}"
            );
        }

        #[test]
        fn file_class_name_should_reject_empty_string() {
            let res = FileClassValue::try_new("");
            assert!(
                res.is_err(),
                "Expected error for empty file class, got: {res:?}"
            );
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn find_typed_should_handle_type_mismatch() {
            let mut fields = HashMap::new();
            fields.insert("num".into(), FieldValue::Number(42.0f64));
            let fm = Frontmatter::new(fields);

            let res = fm.find_typed::<bool>("num");
            assert!(res.is_err(), "Expected type mismatch error, got: {res:?}");
            if let Err(FrontmatterError::TypeMismatch {
                key,
                ..
            }) = res
            {
                assert_eq!(key.as_ref(), "num");
            }
        }

        #[test]
        fn get_typed_should_handle_missing_key() {
            let fm = Frontmatter::new(HashMap::new());
            let res = fm.get_typed::<bool>("missing");
            assert!(
                matches!(res, Err(FrontmatterError::KeyMissing { .. })),
                "Expected KeyMissing error, got: {res:?}"
            );
        }

        #[test]
        fn find_typed_ref_should_borrow_strings() {
            let mut fields = HashMap::new();
            fields.insert("s".into(), FieldValue::String("hello".into()));
            let fm = Frontmatter::new(fields);

            let s: &str = fm.find_typed_ref("s").unwrap().unwrap();
            assert_eq!(s, "hello");
        }

        #[rstest]
        fn aliases_iterator_should_handle_all_shapes(
            default_spec: FrontmatterConfigSpec,
        ) {
            let mut fields = HashMap::new();

            // Case 1: Single string
            fields.insert("aliases".into(), FieldValue::String("A1".into()));
            let fm1 = Frontmatter::from_fields(fields.clone(), &default_spec);
            assert_eq!(fm1.aliases().collect::<Vec<_>>(), vec!["A1"]);

            // Case 2: List of strings
            fields.insert(
                "aliases".into(),
                FieldValue::Array(
                    vec![
                        FieldValue::String("A2".into()),
                        FieldValue::String("A3".into()),
                    ]
                    .into_boxed_slice(),
                ),
            );
            let fm2 = Frontmatter::from_fields(fields, &default_spec);
            assert_eq!(fm2.aliases().collect::<Vec<_>>(), vec!["A2", "A3"]);
        }

        #[rstest]
        fn tags_should_be_collected_from_mixed_formats(
            default_spec: FrontmatterConfigSpec,
        ) {
            let mut fields = HashMap::new();

            // Mixed: string with commas and an array of strings
            fields.insert(
                "tags".into(),
                FieldValue::Array(
                    vec![
                        FieldValue::String("t1, t2".into()),
                        FieldValue::String("t3".into()),
                    ]
                    .into_boxed_slice(),
                ),
            );

            let fm = Frontmatter::from_fields(fields, &default_spec);
            let tags = fm.tags().unwrap();
            let tag_names: Vec<String> =
                tags.iter().map(|t| format!("#{}", t.full_path())).collect();
            assert_eq!(tag_names, vec!["#t1", "#t2", "#t3"]);
        }
    }

    mod temporal {
        use super::*;

        #[rstest]
        #[case::iso("2024-03-21T12:00:00Z")]
        #[case::ymd_dash("2024-03-21")]
        #[case::ymd_slash("2024/03/21")]
        #[case::dmy_dash("21-03-2024")]
        #[case::mdy_slash("03/21/2024")]
        fn should_parse_dates_heuristically(#[case] input: &str) {
            let val = FieldValue::String(input.into());
            let field = FrontmatterDateValue::try_from_value(&val);
            assert!(
                field.is_some(),
                "Failed to parse date heuristically: {input}"
            );
        }

        #[test]
        fn should_reject_invalid_dates() {
            let val = FieldValue::String("2024-02-31".into());
            let field = FrontmatterDateValue::try_from_value(&val);
            assert!(field.is_none(), "Should have rejected invalid date");
        }

        #[test]
        fn should_handle_native_temporal_values() {
            let dt = chrono::Utc::now().fixed_offset();
            let val = FieldValue::DateTime(dt.into());
            let field = FrontmatterDateValue::try_from_value(&val).unwrap();
            assert_eq!(field.as_datetime(), Some(dt));
        }
    }

    mod proptests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            #[test]
            fn alias_name_accepts_valid_strings(s in "[a-zA-Z0-9_-]+") {
                let res = AliasValue::try_new(&s);
                prop_assert!(res.is_ok());
            }
        }
    }
}
