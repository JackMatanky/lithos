//! Raw property types.
//!
//! Defines the property-level structures:
//! - Property variants (Inline vs Ref)
//! - Property bank entries
//! - Overridable fields
//! - Validated property map wrapper (`RawPropertyMap<T>`)
//! - Validated property reference path wrapper (`RawPropertyRefPath`)

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::property_spec::{
    RawDateSpec, RawFileSpec, RawNumberSpec, RawPropertySpec, RawStringSpec,
};
use crate::schema::property::PropertyName;

// ─────────────────────────────────────────────────────────────────────────────
//  RawPropertyMap<T>
// ─────────────────────────────────────────────────────────────────────────────

/// Validated property map that guarantees all keys are valid `PropertyNames`.
///
/// This type provides custom deserialization that validates property names
/// during parsing, making invalid states unrepresentable.
///
/// # Design
///
/// - Keys are guaranteed to be valid `PropertyName` instances
/// - Validation happens during deserialization (fail-fast)
/// - Reusable across `RawPropertyBank` and `RawSchema`
/// - Inner map is private to maintain invariants
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::raw::property::RawPropertyMap;
/// # use serde_json;
/// // Deserialization validates property names automatically
/// let json = r#"{"valid_name": "value", "another_valid": "value2"}"#;
/// let map: RawPropertyMap<String> = serde_json::from_str(json).unwrap();
/// assert_eq!(map.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawPropertyMap<T> {
    /// Inner `HashMap` with validated `PropertyName` keys.
    inner: HashMap<PropertyName, T>,
}

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
}

// Custom Deserialize implementation validates keys during parsing
impl<'de, T> Deserialize<'de> for RawPropertyMap<T>
where
    T: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;

        // Deserialize as HashMap<Box<str>, T>
        let raw_map: HashMap<Box<str>, T> = HashMap::deserialize(deserializer)?;

        // Validate all keys and convert to PropertyName
        let inner: HashMap<PropertyName, T> = raw_map
            .into_iter()
            .map(|(k, v)| {
                PropertyName::try_new(&k)
                    .map(|name| (name, v))
                    .map_err(D::Error::custom)
            })
            .collect::<Result<_, _>>()?;

        Ok(RawPropertyMap {
            inner,
        })
    }

    #[inline]
    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        *place = Self::deserialize(deserializer)?;
        Ok(())
    }
}

// Serialize implementation for symmetry (useful for debugging and testing)
impl<T> Serialize for RawPropertyMap<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as HashMap<String, T> for compatibility
        let string_map: HashMap<String, &T> = self
            .inner
            .iter()
            .map(|(k, v)| (k.as_str().to_owned(), v))
            .collect();
        string_map.serialize(serializer)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  RawPropertyRefPath
// ─────────────────────────────────────────────────────────────────────────────

/// Validated reference path to a property bank entry.
///
/// Ensures the path is properly formatted (e.g., `property_bank#/name`)
/// and allows O(1) extraction of the target property name.
///
/// # Design
///
/// - Validates format during deserialization
/// - Pre-extracts target `PropertyName` for efficient access
/// - Stores full path for error reporting
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::raw::property::RawPropertyRefPath;
/// # use serde_json;
/// let json = r#""property_bank#/archived""#;
/// let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
/// assert_eq!(path.target_name().as_str(), "archived");
/// assert_eq!(path.as_str(), "property_bank#/archived");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RawPropertyRefPath {
    /// Full reference path (e.g., "`property_bank#/archived`").
    full_path: Box<str>,
    /// Pre-extracted target property name (e.g., "archived").
    target_name: PropertyName,
}

impl RawPropertyRefPath {
    /// Returns the target property name being referenced.
    ///
    /// This is the property name after `property_bank#/` in the path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::raw::property::RawPropertyRefPath;
    /// # use serde_json;
    /// let json = r#""property_bank#/status""#;
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
    /// let json = r#""property_bank#/flag""#;
    /// let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
    /// assert_eq!(path.as_str(), "property_bank#/flag");
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

// Custom Deserialize to validate format and extract target name
impl<'de> Deserialize<'de> for RawPropertyRefPath {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;

        let path = String::deserialize(deserializer)?;

        // Validate prefix
        let target = path.strip_prefix("property_bank#/").ok_or_else(|| {
            D::Error::custom(
                "Property reference must start with 'property_bank#/'",
            )
        })?;

        // Validate and construct target PropertyName
        let target_name =
            PropertyName::try_new(target).map_err(D::Error::custom)?;

        Ok(RawPropertyRefPath {
            full_path: path.into_boxed_str(),
            target_name,
        })
    }

    #[inline]
    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        *place = Self::deserialize(deserializer)?;
        Ok(())
    }
}

// Serialize implementation for symmetry
impl Serialize for RawPropertyRefPath {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.full_path.serialize(serializer)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  RawProperty and Related Types
// ─────────────────────────────────────────────────────────────────────────────

/// Raw property for schema properties map.
///
/// Used in `RawSchema.properties` where the name is the map key.
/// Discriminated by presence of `$ref` field. Ref is tried first because
/// it has a required `$ref` field that Inline never has.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawProperty, RawPropertyInline, RawPropertySpec,
/// };
///
/// let property = RawProperty::Inline(RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// });
/// match property {
///     RawProperty::Inline(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Inline variant of a raw property.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawPropertyInline, RawPropertySpec,
/// };
///
/// let inline = RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = inline;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}

/// Reference variant of a raw property with optional overrides.
///
/// Override fields are grouped by type via flattened `Raw*Spec` structs.
/// All override fields are `Option<T>` — `None` means "don't override".
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawDateSpec, RawFileSpec, RawNumberSpec, RawPropertyRef, RawStringSpec,
/// };
///
/// // Note: RawPropertyRef is typically deserialized from files, not constructed directly.
/// // The ref_path field is now a RawPropertyRefPath which validates during deserialization.
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference path (validated during deserialization).
    ///
    /// Format: `property_bank#/<property_name>` where `<property_name>` is a
    /// valid property name. The target property name is pre-extracted for
    /// efficient access via `ref_path.target_name()`.
    #[serde(rename = "$ref")]
    pub ref_path: RawPropertyRefPath,
    /// Override whether property is required.
    pub required: Option<bool>,
    /// Override whether property accepts multiple values.
    pub multi: Option<bool>,
    /// Number-type overrides (min, max, step).
    #[serde(flatten)]
    pub number: RawNumberSpec,
    /// String-type overrides (options, pattern).
    #[serde(flatten)]
    pub string: RawStringSpec,
    /// Date-type overrides (format).
    #[serde(flatten)]
    pub date: RawDateSpec,
    /// File-type overrides (directory, `file_class`).
    #[serde(flatten)]
    pub file: RawFileSpec,
}

/// Entry in the raw property bank.
///
/// The property name is the map key, not a field here.
/// `required` is not present because the bank is schema-agnostic.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawBoolSpec, RawPropertyBankEntry, RawPropertySpec};
///
/// let entry = RawPropertyBankEntry {
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBankEntry {
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
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
    }

    // -------------------------------------------------------------------------
    // RawPropertyRefPath Tests
    // -------------------------------------------------------------------------

    mod raw_property_ref_path {
        use super::*;

        #[test]
        fn deserializes_valid_reference_path() {
            let json = r#""property_bank#/valid_name""#;
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
                "property_bank#/valid_name",
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
            let json = r#""property_bank#/Invalid Name!""#;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(
                result.is_err(),
                "Should reject path with invalid target property name"
            );
        }

        #[test]
        fn rejects_empty_target_name() {
            let json = r#""property_bank#/""#;
            let result: Result<RawPropertyRefPath, _> =
                serde_json::from_str(json);

            assert!(result.is_err(), "Should reject empty target name");
        }

        #[test]
        fn displays_as_full_path() {
            let json = r#""property_bank#/name""#;
            let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
            assert_eq!(format!("{path}"), "property_bank#/name");
        }

        #[test]
        fn serializes_to_full_path_string() {
            let json = r#""property_bank#/name""#;
            let path: RawPropertyRefPath = serde_json::from_str(json).unwrap();
            let serialized = serde_json::to_string(&path).unwrap();
            assert_eq!(serialized, json);
        }
    }

    #[test]
    fn raw_property_inline_variant_constructs() {
        use super::super::property_spec::RawBoolSpec;

        let inline = RawPropertyInline {
            required: false,
            multi: false,
            spec: RawPropertySpec::Bool(RawBoolSpec),
        };
        let inline_variant = RawProperty::Inline(inline);

        assert!(
            matches!(inline_variant, RawProperty::Inline(_)),
            "RawProperty should be Inline variant"
        );
    }

    #[test]
    fn raw_property_ref_variant_constructs() {
        // RawPropertyRef now uses RawPropertyRefPath which validates during
        // deserialization
        let json = r#"{
            "$ref": "property_bank#/status"
        }"#;
        let reference: RawPropertyRef =
            serde_json::from_str(json).expect("Valid ref should deserialize");
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }
}
