//! String property override types.

use std::collections::BTreeSet;

use crate::schema::error::{PropertySpecError, SchemaError};

// ============================================================================
// RawStringSpec
// ============================================================================

/// String property override bundle.
///
/// Supports `options` and `pattern` per the meta-schema.
/// All fields are `Option<T>` to support override contexts.
/// Inline definitions use `RawPropertyString`.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::string::RawStringSpec;
///
/// let _spec = RawStringSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawStringSpec {
    /// Optional allowed values in one of three formats.
    pub options: Option<RawOptions>,
    /// Optional validation pattern (custom regex or predefined format).
    pub pattern: Option<RawStringPattern>,
}

// ============================================================================
// RawStringPattern
// ============================================================================

/// Raw string pattern supporting both custom regex and predefined formats.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawStringPattern {
    /// Predefined named format.
    Named(RawStringFormat),
    /// Custom regex pattern.
    Custom(Box<str>),
}

// ============================================================================
// RawStringFormat
// ============================================================================

/// Named string format for common validation patterns (raw/syntax layer).
///
/// This is the deserialization type. It gets converted to `StringPattern`
/// in the domain layer during validation.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RawStringFormat {
    /// Email address validation.
    Email,
    /// URL validation.
    Url,
    /// US phone number validation.
    PhoneUs,
    /// Slug validation (kebab-case).
    Slug,
    /// UUID v4 validation.
    UuidV4,
    /// `WikiLink` validation (Obsidian-style).
    WikiLink,
    /// US ZIP code validation.
    ZipCode,
}

// ============================================================================
// RawOptions
// ============================================================================

/// Raw options definition supporting three formats.
///
/// Empty lists or duplicate values are allowed here and handled with warnings
/// during domain construction.
///
/// # Modes
///
/// - **Plain**: `["a", "b"]` — plain array of string values
/// - **Ordered**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed object
/// - **Labeled**: `[{"value": "a", "label": "A", "order": 1}]` — rich entries
///   with labels
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawOptions {
    /// Plain array of string values.
    Plain(Vec<Box<str>>),
    /// Integer-keyed ordered object.
    #[serde(
        deserialize_with = "deserialize_ordered_map",
        serialize_with = "serialize_ordered_map"
    )]
    Ordered(Vec<RawEntryValue>),
    /// Rich entries with optional label and order.
    Labeled(Vec<RawEntryValue>),
}

impl RawOptions {
    /// Convert raw options to a normalized vector of `RawEntryValue`.
    ///
    /// # Modes
    ///
    /// - **Plain**: entries have `value = item`, `label = None`, order
    ///   preserved.
    /// - **Ordered**: keys parsed as integers, sorted by key, entries have
    ///   `value = map_value`, `label = None`.
    /// - **Labeled**: sorted by `order` field (then array position), entries
    ///   have `value` and `label`.
    ///
    /// # Panics
    /// Panics if Labeled mode option list has more than `u32::MAX` entries
    /// (>4 billion). This is unrealistic in practice and indicates a
    /// malformed input.
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Fail-fast on unrealistic >4 billion options prevents silent \
                  data corruption"
    )]
    pub fn into_entries(self) -> Vec<RawEntryValue> {
        match self {
            Self::Plain(items) => items
                .into_iter()
                .map(|value| RawEntryValue {
                    value,
                    label: None,
                    order: None,
                })
                .collect(),
            Self::Ordered(mut entries) => {
                entries.sort_by_key(|entry| entry.order);
                entries
            }
            Self::Labeled(entries) => {
                let mut entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let order = entry.order.unwrap_or_else(|| {
                            RawEntryInputOrder(u32::try_from(idx).expect(
                                "Option list index exceeds u32::MAX (>4 \
                                 billion entries)",
                            ))
                        });
                        (order, entry)
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, entry)| RawEntryValue {
                        value: entry.value,
                        label: entry.label,
                        order: None,
                    })
                    .collect()
            }
        }
    }
}

/// Bridge function to deserialize an integer-keyed map into a sorted vector of
/// entries.
#[expect(
    clippy::missing_trait_methods,
    reason = "Manual Visitor used for structural map-to-vec conversion"
)]
fn deserialize_ordered_map<'de, D>(
    deserializer: D,
) -> Result<Vec<RawEntryValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};

    struct OrderedMapVisitor;

    impl<'de> Visitor<'de> for OrderedMapVisitor {
        type Value = Vec<RawEntryValue>;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            formatter.write_str("a map with integer string keys")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut seen = BTreeSet::new();
            let mut entries = Vec::new();
            while let Some((order, value)) =
                map.next_entry::<RawEntryInputOrder, Box<str>>()?
            {
                if !seen.insert(order) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate order key {order}"
                    )));
                }
                entries.push(RawEntryValue {
                    value,
                    label: None,
                    order: Some(order),
                });
            }
            entries.sort_by_key(|e| e.order);
            Ok(entries)
        }
    }

    deserializer.deserialize_map(OrderedMapVisitor)
}

/// Bridge function to serialize a vector of entries back into an integer-keyed
/// map.
fn serialize_ordered_map<S>(
    entries: &[RawEntryValue],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap as _;

    let mut map = serializer.serialize_map(Some(entries.len()))?;
    for entry in entries {
        let key = entry
            .order
            .map_or_else(|| "0".to_owned(), |o| o.value().to_string());
        map.serialize_entry(&key, &entry.value)?;
    }
    map.end()
}

// ============================================================================
// RawEntryValue
// ============================================================================

/// A rich option entry with optional label and input order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawEntryValue {
    /// The option value.
    pub value: Box<str>,
    /// Optional display label.
    pub label: Option<Box<str>>,
    /// Optional input order for sorting (lower = earlier).
    pub order: Option<RawEntryInputOrder>,
}

// ============================================================================
// RawEntryInputOrder
// ============================================================================

/// Input order position parsed from map keys or attributes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize,
)]
#[non_exhaustive]
pub struct RawEntryInputOrder(u32);

impl RawEntryInputOrder {
    /// Returns the order value.
    #[inline]
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for RawEntryInputOrder {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for RawEntryInputOrder {
    type Err = SchemaError;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed: u32 = value.parse().map_err(|_e| {
            SchemaError::PropertySpec(
                PropertySpecError::InvalidOptionsEntryOrderType {
                    key: value.into(),
                },
            )
        })?;
        if parsed == 0 {
            return Err(SchemaError::PropertySpec(
                PropertySpecError::InvalidOptionsEntryOrderValue {
                    order: 0,
                },
            ));
        }
        Ok(Self(parsed))
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Manual Visitor used for flexible parsing from string or integer"
)]
impl<'de> serde::Deserialize<'de> for RawEntryInputOrder {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        struct RawEntryInputOrderVisitor;

        impl serde::de::Visitor<'_> for RawEntryInputOrderVisitor {
            type Value = RawEntryInputOrder;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("a string integer or number >= 1")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value: u32 = v.try_into().map_err(|_e| {
                    serde::de::Error::custom("order key exceeds u32::MAX")
                })?;
                if value == 0 {
                    return Err(serde::de::Error::custom(
                        "order key must be >= 1",
                    ));
                }
                Ok(RawEntryInputOrder(value))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v <= 0 {
                    return Err(serde::de::Error::custom(
                        "order key must be >= 1",
                    ));
                }
                let value = u64::try_from(v)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                self.visit_u64(value)
            }
        }

        deserializer.deserialize_any(RawEntryInputOrderVisitor)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::items_after_statements,
    reason = "Test code: indexing is safe after len check, panic shows test \
              failure clearly, wildcard for unknown future variants is fine"
)]
mod tests {
    use super::*;

    #[test]
    fn raw_options_deserializes_list_from_json() {
        let json = r#"["open", "closed", "archived"]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Plain(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "open");
                assert_eq!(items[1].as_ref(), "closed");
                assert_eq!(items[2].as_ref(), "archived");
            }
            _ => panic!("Expected Plain variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_empty_list() {
        let json = "[]";
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Plain(items) => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected Plain variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_map_from_json() {
        let json = r#"{"1": "todo", "2": "done", "3": "archived"}"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Ordered(entries) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(entries[0].order.unwrap().value(), 1);
                assert_eq!(entries[0].value.as_ref(), "todo");
                assert_eq!(entries[1].order.unwrap().value(), 2);
                assert_eq!(entries[1].value.as_ref(), "done");
                assert_eq!(entries[2].order.unwrap().value(), 3);
                assert_eq!(entries[2].value.as_ref(), "archived");
            }
            _ => panic!("Expected Ordered variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_rich_from_json() {
        let json = r#"[
            {"value": "open", "label": "Open", "order": 1},
            {"value": "closed", "label": "Closed", "order": 2}
        ]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Labeled(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "open");
                assert_eq!(entries[0].label.as_deref(), Some("Open"));
                assert_eq!(entries[0].order, Some(RawEntryInputOrder(1)));
                assert_eq!(entries[1].value.as_ref(), "closed");
                assert_eq!(entries[1].label.as_deref(), Some("Closed"));
                assert_eq!(entries[1].order, Some(RawEntryInputOrder(2)));
            }
            _ => panic!("Expected Labeled variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_rich_with_minimal_fields() {
        let json = r#"[{"value": "open"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Labeled(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].value.as_ref(), "open");
                assert_eq!(entries[0].label, None);
                assert_eq!(entries[0].order, None);
            }
            _ => panic!("Expected Labeled variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_from_yaml_list() {
        let yaml = "- open\n- closed\n- archived\n";
        let options: RawOptions = serde_yaml::from_str(yaml).unwrap();
        match options {
            RawOptions::Plain(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "open");
            }
            _ => panic!("Expected Plain variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_from_toml_inline_array() {
        let toml_str = r#"options = ["open", "closed"]"#;
        #[derive(serde::Deserialize)]
        struct Wrapper {
            options: RawOptions,
        }
        let wrapper: Wrapper = toml::from_str(toml_str).unwrap();
        match wrapper.options {
            RawOptions::Plain(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_ref(), "open");
                assert_eq!(items[1].as_ref(), "closed");
            }
            _ => panic!("Expected Plain variant"),
        }
    }

    #[test]
    fn raw_options_rejects_mixed_array_types() {
        let json = r#"["string", 123]"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Should reject mixed types in array");
    }

    #[test]
    fn raw_options_rejects_invalid_rich_structure() {
        let json = r#"[{"label": "Missing value field"}]"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject rich entries missing 'value' field"
        );
    }

    #[test]
    fn raw_options_into_entries_list_preserves_order() {
        let options =
            RawOptions::Plain(vec!["a".into(), "b".into(), "c".into()]);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "a");
        assert_eq!(entries[1].value.as_ref(), "b");
        assert_eq!(entries[2].value.as_ref(), "c");
        assert!(entries.iter().all(|e| e.label.is_none()));
    }

    #[test]
    fn raw_options_into_entries_map_sorts_by_key() {
        let options = RawOptions::Ordered(vec![
            RawEntryValue {
                order: Some(RawEntryInputOrder(3)),
                value: "third".into(),
                label: None,
            },
            RawEntryValue {
                order: Some(RawEntryInputOrder(1)),
                value: "first".into(),
                label: None,
            },
            RawEntryValue {
                order: Some(RawEntryInputOrder(2)),
                value: "second".into(),
                label: None,
            },
        ]);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "first");
        assert_eq!(entries[1].value.as_ref(), "second");
        assert_eq!(entries[2].value.as_ref(), "third");
    }

    #[test]
    fn raw_options_into_entries_rich_sorts_by_order() {
        let options = RawOptions::Labeled(vec![
            RawEntryValue {
                value: "c".into(),
                label: Some("Third".into()),
                order: Some(RawEntryInputOrder(3)),
            },
            RawEntryValue {
                value: "a".into(),
                label: Some("First".into()),
                order: Some(RawEntryInputOrder(1)),
            },
            RawEntryValue {
                value: "b".into(),
                label: Some("Second".into()),
                order: Some(RawEntryInputOrder(2)),
            },
        ]);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "a");
        assert_eq!(entries[0].label.as_deref(), Some("First"));
        assert_eq!(entries[1].value.as_ref(), "b");
        assert_eq!(entries[2].value.as_ref(), "c");
    }

    #[test]
    fn raw_options_into_entries_rich_uses_array_position_when_no_order() {
        let options = RawOptions::Labeled(vec![
            RawEntryValue {
                value: "first".into(),
                label: None,
                order: None,
            },
            RawEntryValue {
                value: "second".into(),
                label: None,
                order: None,
            },
        ]);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].value.as_ref(), "first");
        assert_eq!(entries[1].value.as_ref(), "second");
    }

    // --- Edge Case Tests for E-04 (Deserialization Disambiguation) ---

    #[test]
    fn raw_options_disambiguates_empty_object_array_as_rich() {
        let json = r#"[{"value": "a"}, {"value": "b"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Labeled(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "a");
                assert_eq!(entries[0].label, None);
            }
            _ => panic!("Expected Labeled variant for array of objects"),
        }
    }

    #[test]
    fn raw_options_disambiguates_strings_as_list() {
        let json = r#"["value", "label", "order"]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Plain(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "value");
                assert_eq!(items[1].as_ref(), "label");
                assert_eq!(items[2].as_ref(), "order");
            }
            _ => panic!("Expected Plain variant for string array"),
        }
    }

    #[test]
    fn raw_options_rejects_array_of_numbers() {
        let json = "[1, 2, 3]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of numbers (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_array_of_bools() {
        let json = "[true, false]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of booleans (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_array_of_nulls() {
        let json = "[null, null]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of nulls (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_nested_arrays() {
        let json = r#"[["a", "b"], ["c", "d"]]"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject nested arrays (not a valid mode)"
        );
    }

    #[test]
    fn raw_options_rejects_string_primitive() {
        let json = r#""single_value""#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject string primitive (must be array or map)"
        );
    }

    #[test]
    fn raw_options_rejects_number_primitive() {
        let json = "42";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject number primitive (must be array or map)"
        );
    }

    #[test]
    fn raw_options_map_with_non_numeric_keys() {
        let json = r#"{"open": "Open", "closed": "Closed"}"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Non-numeric keys must be rejected");
    }

    #[test]
    fn raw_options_map_rejects_zero_key() {
        let json = r#"{"0": "zero"}"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Order key 0 must be rejected");
    }

    #[test]
    fn raw_options_map_rejects_duplicate_keys() {
        let json = r#"{"1": "first", "1": "second"}"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Duplicate order keys must be rejected");
    }

    #[test]
    fn raw_options_rich_rejects_extra_fields() {
        let json = r#"[{"value": "a", "label": "A", "extra": "ignored"}]"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Extra fields must be rejected");
    }

    #[test]
    fn raw_options_empty_map() {
        let json = "{}";
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Ordered(entries) => {
                assert!(entries.is_empty());
            }
            _ => panic!("Expected Ordered variant for empty object"),
        }
    }
}
