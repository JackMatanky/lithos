//! Raw property types.
//!
//! This module defines the serde-facing property DTOs used during raw schema
//! ingestion. The design follows serde best practices for strict, structural
//! parsing without custom visitors:
//!
//! - **Internally tagged enum** for inline definitions: `RawPropertyInline` is
//!   tagged by `type`, which makes variant selection explicit and stable.
//! - **Untagged enum** for ref vs inline: `RawProperty` uses
//!   `#[serde(untagged)]` because `$ref` is structurally distinct and required
//!   for references.
//! - **`deny_unknown_fields` everywhere** to mirror `additionalProperties:
//!   false` in the meta-schema.
//! - **Newtype parsing for ref paths**: `RawPropertyRefPath` uses
//!   `#[serde(try_from = "String")]` to validate prefix and target name.
//! - **No `flatten`**: explicit fields avoid ambiguity and allow strictness.
//!
//! Component overview:
//! - `RawPropertyInline`: per-type inline definitions (uses `Raw*Property`
//!   structs from `raw/{bool,date,file,number,string}.rs`)
//! - `RawPropertyRef`: `$ref` plus optional override fields
//! - `RawPropertyBankEntry`: inline definitions used in the property bank
//! - `RawPropertyMap<T>`: validated map keyed by `PropertyName`
//! - `RawPropertyRefPath`: validated property bank reference path

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    bool::RawBoolProperty,
    date::RawDateProperty,
    file::RawFileProperty,
    number::RawNumberProperty,
    string::{RawOptions, RawStringPattern, RawStringProperty},
};
use crate::{
    schema::{
        error::SchemaError, identifier::SchemaName, property::PropertyName,
        views::RawPropertyMapHash,
    },
    support::hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  RawPropertyMap<T>
// ─────────────────────────────────────────────────────────────────────────────

/// Validated property map that guarantees all keys are valid `PropertyNames`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct RawPropertyMap<T> {
    /// Inner `HashMap` with validated `PropertyName` keys.
    inner: HashMap<PropertyName, T>,
}

type SplitPropertyEntries = (
    HashMap<PropertyName, RawPropertyInline>,
    HashMap<PropertyName, RawPropertyRef>,
);

impl<T> RawPropertyMap<T> {
    /// Returns a reference to the inner map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"name": "value"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// let inner = map.as_map();
    /// assert_eq!(inner.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_map(&self) -> &HashMap<PropertyName, T> {
        &self.inner
    }

    /// Consumes self and returns the inner map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"name": "value"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// let inner = map.into_map();
    /// assert_eq!(inner.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn into_map(self) -> HashMap<PropertyName, T> {
        self.inner
    }

    /// Returns a reference to the value for the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use lithos_core::schema::property::PropertyName;
    /// # use serde_json;
    /// let json = r#"{"name": "value"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// let key = PropertyName::try_new("name").unwrap();
    /// assert_eq!(map.get(&key), Some(&"value".to_owned()));
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, name: &PropertyName) -> Option<&T> {
        self.inner.get(name)
    }

    /// Returns an iterator over property entries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"a": "1", "b": "2"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// assert_eq!(map.iter().count(), 2);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&PropertyName, &T)> {
        self.inner.iter()
    }

    /// Returns the number of properties.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"a": "1", "b": "2"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// assert_eq!(map.len(), 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no properties.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over property names (keys).
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"a": "1", "b": "2"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// assert_eq!(map.keys().count(), 2);
    /// ```
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &PropertyName> {
        self.inner.keys()
    }

    /// Returns an iterator over property values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyMap;
    /// # use serde_json;
    /// let json = r#"{"a": "1", "b": "2"}"#;
    /// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
    /// assert_eq!(map.values().count(), 2);
    /// ```
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.inner.values()
    }

    /// Computes per-property hashes for the map values.
    #[inline]
    #[must_use]
    pub fn compute_hashes(&self) -> RawPropertyMapHash
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        self.inner
            .iter()
            .map(|(name, value)| {
                (name.clone(), Blake3Hash::compute_json(value))
            })
            .collect::<HashMap<PropertyName, Blake3Hash>>()
            .into()
    }
}

impl RawPropertyMap<RawProperty> {
    /// Returns a `HashMap` containing only `$ref` entries.
    #[inline]
    #[must_use]
    pub fn ref_entries(&self) -> HashMap<PropertyName, RawPropertyRef> {
        let (_, refs) = self.split_entries();
        refs
    }

    /// Returns a `HashMap` containing only inline entries.
    #[inline]
    #[must_use]
    pub fn inline_entries(&self) -> HashMap<PropertyName, RawPropertyInline> {
        let (inline, _) = self.split_entries();
        inline
    }

    /// Returns both inline and `$ref` entries in one pass.
    #[inline]
    #[must_use]
    pub fn split_entries(&self) -> SplitPropertyEntries {
        let mut inline = HashMap::new();
        let mut refs = HashMap::new();
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Ordering is irrelevant when filtering ref entries"
        )]
        for (name, entry) in &self.inner {
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Match ergonomics keeps ref extraction concise"
            )]
            match entry {
                RawProperty::Ref(ref_entry) => {
                    refs.insert(name.clone(), ref_entry.clone());
                }
                RawProperty::Inline(inline_entry) => {
                    inline.insert(name.clone(), inline_entry.clone());
                }
            }
        }
        (inline, refs)
    }
}

impl<'lifetime, T> IntoIterator for &'lifetime RawPropertyMap<T> {
    type IntoIter =
        std::collections::hash_map::Iter<'lifetime, PropertyName, T>;
    type Item = (&'lifetime PropertyName, &'lifetime T);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  RawProperty and Related Types
// ─────────────────────────────────────────────────────────────────────────────

/// Raw property for schema properties map.
///
/// Used in `RawSchema.properties` where the name is the map key. The enum is
/// untagged: references are selected by presence of a required `$ref` field,
/// while inline definitions match the tagged `type` shape.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawProperty, RawPropertyInline};
///
/// let property = RawProperty::Inline(RawPropertyInline::Bool(
///     lithos_core::schema::raw::bool::RawBoolProperty {
///         required: false,
///         multi: false,
///     },
/// ));
/// match property {
///     RawProperty::Inline(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Reference variant of a raw property with optional overrides.
///
/// All override fields are optional. Unknown fields are rejected to mirror the
/// schema's `additionalProperties: false` constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference path (validated during deserialization).
    ///
    /// Format: `#property_bank/<property_name>` where `<property_name>` is a
    /// valid property name. The target property name is pre-extracted for
    /// efficient access via `ref_path.target_name()`.
    #[serde(rename = "$ref")]
    pub ref_path: RawPropertyRefPath,
    /// Override whether property is required.
    pub required: Option<bool>,
    /// Override whether property accepts multiple values.
    pub multi: Option<bool>,
    /// String-type overrides (options, pattern).
    pub options: Option<RawOptions>,
    /// Optional string pattern override.
    pub pattern: Option<RawStringPattern>,
    /// Number-type overrides (min, max, step).
    pub min: Option<f64>,
    /// Optional maximum override.
    pub max: Option<f64>,
    /// Optional step override.
    pub step: Option<f64>,
    /// Date-type overrides (format).
    pub format: Option<Box<str>>,
    /// File-type overrides (directory, `file_class`).
    pub directory: Option<Box<str>>,
    /// Optional file class override (schema name).
    pub file_class: Option<SchemaName>,
}

/// Inline variant of a raw property definition.
///
/// Discriminated by the `type` field. These are full inline definitions used
/// in schema and property bank files (syntax-only validation at this layer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RawPropertyInline {
    /// Boolean property definition.
    #[serde(rename = "bool", alias = "boolean")]
    Bool(RawBoolProperty),
    /// Date property definition.
    #[serde(rename = "date")]
    Date(RawDateProperty),
    /// File property definition.
    #[serde(rename = "file")]
    File(RawFileProperty),
    /// Number property definition.
    #[serde(rename = "number")]
    Number(RawNumberProperty),
    /// String property definition.
    #[serde(rename = "string")]
    String(RawStringProperty),
}

/// Entry in the raw property bank.
///
/// Property bank entries use the same schema-level inline DTOs. `required` is
/// allowed in the input for early warnings and overridden during domain
/// construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct RawPropertyBankEntry(pub RawPropertyInline);

// ─────────────────────────────────────────────────────────────────────────────
//  RawPropertyRefPath
// ─────────────────────────────────────────────────────────────────────────────

/// Validated reference path to a property bank entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
#[non_exhaustive]
pub struct RawPropertyRefPath {
    /// Full reference path (e.g., "`#property_bank/archived`").
    full_path: Box<str>,
    /// Pre-extracted target property name (e.g., "archived").
    target_name: PropertyName,
}

impl serde::Serialize for RawPropertyRefPath {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.full_path.serialize(serializer)
    }
}

impl TryFrom<String> for RawPropertyRefPath {
    type Error = SchemaError;

    #[inline]
    fn try_from(path: String) -> Result<Self, Self::Error> {
        // Validate prefix
        let target = path.strip_prefix("#property_bank/").ok_or_else(|| {
            SchemaError::Syntax(crate::schema::error::SchemaSyntaxError::PropertyName(
                crate::schema::error::PropertyNameError::InvalidFormat {
                    name: path.clone().into(),
                    context: crate::schema::error::PropertyNameContext::PropertyBank,
                },
            ))
        })?;

        // Validate and construct target PropertyName
        let target_name = PropertyName::try_new_with_context(
            target,
            crate::schema::error::PropertyNameContext::PropertyBank,
        )?;

        Ok(RawPropertyRefPath {
            full_path: path.into_boxed_str(),
            target_name,
        })
    }
}

impl RawPropertyRefPath {
    /// Returns the target property name being referenced.
    ///
    /// This is the property name after `#property_bank/` in the path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyRefPath;
    /// # use serde_json;
    /// let json = r##""#property_bank/status""##;
    /// let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
    /// assert_eq!(path.target_name().as_str(), "status");
    /// ```
    #[inline]
    #[must_use]
    pub fn target_name(&self) -> &PropertyName {
        &self.target_name
    }

    /// Returns the full reference path as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyRefPath;
    /// # use serde_json;
    /// let json = r##""#property_bank/flag""##;
    /// let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
    /// assert_eq!(path.as_str(), "#property_bank/flag");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.full_path
    }
}

// Implement AsRef<str> for compatibility with code expecting string references
impl AsRef<str> for RawPropertyRefPath {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.full_path
    }
}

// Implement Display for error messages and debugging
impl std::fmt::Display for RawPropertyRefPath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // RawPropertyMap Tests
    // -------------------------------------------------------------------------

    mod raw_property_map {
        use super::*;

        #[test]
        fn deserializes_valid_property_names() {
            let json = r#"{"valid_name": "value1", "another_valid": "value2"}"#;
            let result: Result<RawPropertyMap<String>, _> =
                serde_json::from_str(json);

            assert!(
                result.is_ok(),
                "Should deserialize map with valid property names"
            );
            let map = result.unwrap();
            assert_eq!(map.len(), 2, "Should have 2 properties");
        }

        #[test]
        fn rejects_invalid_property_names() {
            let json = r#"{"Invalid Name!": "value"}"#;
            let result: Result<RawPropertyMap<String>, _> =
                serde_json::from_str(json);

            assert!(
                result.is_err(),
                "Should reject map with invalid property names"
            );
        }

        #[test]
        fn rejects_empty_property_names() {
            let json = r#"{"": "value"}"#;
            let result: Result<RawPropertyMap<String>, _> =
                serde_json::from_str(json);

            assert!(result.is_err(), "Should reject empty property names");
        }

        #[test]
        fn as_map_returns_inner_hashmap() {
            let json = r#"{"name": "value"}"#;
            let map: RawPropertyMap<String> =
                serde_json::from_str(json).unwrap();
            let inner = map.as_map();

            assert_eq!(inner.len(), 1, "Inner map should have 1 entry");
        }

        #[test]
        fn get_returns_value_for_existing_key() {
            let json = r#"{"name": "value"}"#;
            let map: RawPropertyMap<String> =
                serde_json::from_str(json).unwrap();
            let key = PropertyName::try_new("name").unwrap();

            assert_eq!(
                map.get(&key),
                Some(&"value".to_owned()),
                "Should return value for existing key"
            );
        }

        #[test]
        fn iter_returns_all_entries() {
            let json = r#"{"name": "value"}"#;
            let map: RawPropertyMap<String> =
                serde_json::from_str(json).unwrap();
            let entries: Vec<_> = map.iter().collect();

            assert_eq!(entries.len(), 1, "Should iterate over 1 entry");
            #[expect(
                clippy::indexing_slicing,
                reason = "Test access known-fixed index"
            )]
            {
                assert_eq!(entries[0].0.as_str(), "name");
                assert_eq!(entries[0].1, "value");
            }
        }

        #[test]
        fn len_and_is_empty_work_as_expected() {
            let map: RawPropertyMap<String> =
                serde_json::from_str("{}").unwrap();
            assert_eq!(map.len(), 0);
            assert!(map.is_empty());

            let map2: RawPropertyMap<String> =
                serde_json::from_str(r#"{"a":"b"}"#).unwrap();
            assert_eq!(map2.len(), 1);
            assert!(!map2.is_empty());
        }

        #[test]
        fn keys_returns_all_property_names() {
            let map: RawPropertyMap<String> =
                serde_json::from_str(r#"{"a":"1","b":"2"}"#).unwrap();
            let keys: Vec<_> = map.keys().map(PropertyName::as_str).collect();
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"a"));
            assert!(keys.contains(&"b"));
        }

        #[test]
        fn values_returns_all_values() {
            let map: RawPropertyMap<String> =
                serde_json::from_str(r#"{"a":"1","b":"2"}"#).unwrap();
            let values: Vec<_> = map.values().collect();
            assert_eq!(values.len(), 2);
            assert!(values.contains(&&"1".to_owned()));
            assert!(values.contains(&&"2".to_owned()));
        }

        #[test]
        fn compute_hashes_returns_per_property_hashes() {
            let map: RawPropertyMap<String> =
                serde_json::from_str(r#"{"a":"1","b":"2"}"#).unwrap();

            let hashes = map.compute_hashes();

            assert_eq!(hashes.len(), 2);
            assert_eq!(
                hashes.get(&PropertyName::try_new("a").unwrap()),
                Some(&Blake3Hash::compute_json(&"1".to_owned()))
            );
            assert_eq!(
                hashes.get(&PropertyName::try_new("b").unwrap()),
                Some(&Blake3Hash::compute_json(&"2".to_owned()))
            );
        }

        #[test]
        fn serializes_back_to_json() {
            let json = r#"{"name":"value"}"#;
            let map: RawPropertyMap<String> =
                serde_json::from_str(json).unwrap();
            let serialized = serde_json::to_string(&map).unwrap();

            // Deserialize both to compare (order may differ)
            let original: std::collections::HashMap<String, String> =
                serde_json::from_str(json).unwrap();
            let reserialized: std::collections::HashMap<String, String> =
                serde_json::from_str(&serialized).unwrap();

            assert_eq!(
                original, reserialized,
                "Serialized map should match original"
            );
        }

        #[test]
        fn into_map_consumes_and_returns_inner() {
            let json = r#"{"name": "value"}"#;
            let map: RawPropertyMap<String> =
                serde_json::from_str(json).unwrap();
            let inner = map.into_map();

            assert_eq!(inner.len(), 1, "Inner map should have 1 entry");
        }

        #[test]
        fn ref_entries_returns_only_refs() {
            let json = r##"{
                "inline_prop": {"type": "bool"},
                "ref_prop": {"$ref": "#property_bank/name"}
            }"##;
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_str(json).unwrap();

            let refs = map.ref_entries();

            assert_eq!(refs.len(), 1);
            assert!(
                refs.contains_key(&PropertyName::try_new("ref_prop").unwrap())
            );
        }

        #[test]
        fn inline_entries_returns_only_inline() {
            let json = r##"{
                "inline_prop": {"type": "bool"},
                "ref_prop": {"$ref": "#property_bank/name"}
            }"##;
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_str(json).unwrap();

            let inline = map.inline_entries();

            assert_eq!(inline.len(), 1);
            assert!(
                inline.contains_key(
                    &PropertyName::try_new("inline_prop").unwrap()
                )
            );
        }

        #[test]
        fn split_entries_partitions_in_one_call() {
            let json = r##"{
                "inline_prop": {"type": "bool"},
                "ref_prop": {"$ref": "#property_bank/name"}
            }"##;
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_str(json).unwrap();

            let (inline, refs) = map.split_entries();

            assert_eq!(inline.len(), 1);
            assert_eq!(refs.len(), 1);
        }
    }

    // -------------------------------------------------------------------------
    // RawPropertyRefPath Tests
    // -------------------------------------------------------------------------

    mod raw_property_ref_path {
        use super::*;

        #[test]
        fn deserializes_valid_reference_path() {
            let json = r##""#property_bank/valid_name""##;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(result.is_ok(), "Should deserialize valid reference path");
            let path = result.unwrap();
            assert_eq!(
                path.target_name().as_str(),
                "valid_name",
                "Should extract target name"
            );
            assert_eq!(
                path.as_str(),
                "#property_bank/valid_name",
                "Should preserve full path"
            );
        }

        #[test]
        fn rejects_invalid_prefix() {
            let json = r#""invalid_prefix#/name""#;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(result.is_err(), "Should reject path with invalid prefix");
        }

        #[test]
        fn rejects_missing_hash_separator() {
            let json = r#""property_bank/name""#;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(
                result.is_err(),
                "Should reject path without hash separator"
            );
        }

        #[test]
        fn rejects_invalid_target_property_name() {
            let json = r##""#property_bank/Invalid Name!""##;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(
                result.is_err(),
                "Should reject path with invalid target property name"
            );
        }

        #[test]
        fn rejects_empty_target_name() {
            let json = r##""#property_bank/""##;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(result.is_err(), "Should reject empty target name");
        }

        #[test]
        fn displays_as_full_path() {
            let json = r##""#property_bank/name""##;
            let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
            assert_eq!(format!("{path}"), "#property_bank/name");
        }

        #[test]
        fn serializes_to_full_path_string() {
            let json = r##""#property_bank/name""##;
            let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
            let serialized = serde_json::to_string(&path).unwrap();
            assert_eq!(serialized, json);
        }
    }

    mod raw_property_inline {
        use super::*;

        #[test]
        fn deserializes_bool_and_boolean_tags() {
            let json_bool = r#"{"type": "bool"}"#;
            let json_boolean = r#"{"type": "boolean"}"#;

            let bool_inline: RawPropertyInline =
                serde_json::from_str(json_bool).unwrap();
            let boolean_inline: RawPropertyInline =
                serde_json::from_str(json_boolean).unwrap();

            assert!(matches!(bool_inline, RawPropertyInline::Bool(_)));
            assert!(matches!(boolean_inline, RawPropertyInline::Bool(_)));
        }

        #[test]
        fn serializes_bool_tag() {
            let inline = RawPropertyInline::Bool(RawBoolProperty {
                required: false,
                multi: false,
            });
            let value = serde_json::to_value(&inline).unwrap();
            let tag = value.get("type").and_then(|value| value.as_str());
            assert_eq!(tag, Some("bool"));
        }

        #[test]
        fn rejects_unknown_fields() {
            let json = r#"{"type": "string", "extra": "nope"}"#;
            let result: Result<RawPropertyInline, _> =
                serde_json::from_str(json);
            assert!(result.is_err(), "Unknown fields must be rejected");
        }
    }

    #[test]
    fn raw_property_inline_variant_constructs() {
        let inline_variant =
            RawProperty::Inline(RawPropertyInline::Bool(RawBoolProperty {
                required: false,
                multi: false,
            }));

        assert!(
            matches!(inline_variant, RawProperty::Inline(_)),
            "RawProperty should be Inline variant"
        );
    }

    #[test]
    fn raw_property_ref_variant_constructs() {
        // RawPropertyRef now uses RawPropertyRefPath which validates during
        // deserialization
        let json = r##"{
            "$ref": "#property_bank/status"
        }"##;
        let reference: RawPropertyRef =
            serde_json::from_str(json).expect("Valid ref should deserialize");
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }

    #[test]
    fn raw_property_ref_rejects_unknown_fields() {
        let json = r##"{
            "$ref": "#property_bank/status",
            "extra": true
        }"##;
        let result: Result<RawPropertyRef, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Unknown fields must be rejected");
    }
}
