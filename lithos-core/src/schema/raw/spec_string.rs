//! String property specification types.

use std::collections::BTreeSet;

use serde::de::{Error, MapAccess, Visitor};

use crate::schema::error::{PropertySpecError, SchemaError};

// ============================================================================
// RawStringSpec
// ============================================================================

/// String property definition.
///
/// Supports `options` and `pattern` per the meta-schema.
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::spec_string::RawStringSpec;
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
/// # Modes
///
/// - **Plain**: `["a", "b"]` — plain array of string values
/// - **Ordered**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed object
/// - **Labeled**: `[{"value": "a", "label": "A", "order": 1}]` — rich entries
///   with labels
///
/// # Deserialization Strategy
///
/// Uses custom deserializer with explicit type checking:
/// 1. If sequence: try as Plain (strings), then Labeled (objects)
/// 2. If map: deserialize as Ordered
/// 3. Fail with clear error for other types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum RawOptions {
    /// Plain array of string values.
    Plain(RawOptionsPlain),
    /// Integer-keyed ordered object.
    Ordered(RawOptionsOrdered),
    /// Rich entries with optional label and order.
    Labeled(RawOptionsLabeled),
}

#[expect(
    clippy::missing_trait_methods,
    clippy::missing_inline_in_public_items,
    clippy::excessive_nesting,
    reason = "Custom serde Visitor requires specific method impls; excessive \
              nesting is inherent to discriminating union variants by peeking"
)]
impl<'de> serde::Deserialize<'de> for RawOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{Error, MapAccess, SeqAccess, Visitor};

        struct RawOptionsVisitor;

        impl<'de> Visitor<'de> for RawOptionsVisitor {
            type Value = RawOptions;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str(
                    "a sequence of strings, a sequence of objects with \
                     'value' field, or a map with string keys and string \
                     values",
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let first = seq.next_element::<serde_json::Value>()?;
                if let Some(value) = first {
                    if value.is_string() {
                        let mut items = vec![
                            value
                                .as_str()
                                .ok_or_else(|| {
                                    Error::custom("expected string")
                                })?
                                .into(),
                        ];
                        while let Some(elem) =
                            seq.next_element::<serde_json::Value>()?
                        {
                            if let Some(s) = elem.as_str() {
                                items.push(s.into());
                            } else {
                                return Err(Error::custom(
                                    "expected all array elements to be strings",
                                ));
                            }
                        }
                        Ok(RawOptions::Plain(RawOptionsPlain(items)))
                    } else if value.is_object() {
                        let first_entry: RawEntryLabeled =
                            serde_json::from_value(value)
                                .map_err(Error::custom)?;
                        let mut entries = vec![first_entry];
                        while let Some(entry) =
                            seq.next_element::<RawEntryLabeled>()?
                        {
                            entries.push(entry);
                        }
                        Ok(RawOptions::Labeled(RawOptionsLabeled(entries)))
                    } else {
                        Err(Error::custom(
                            "expected array elements to be strings or objects",
                        ))
                    }
                } else {
                    Ok(RawOptions::Plain(RawOptionsPlain(vec![])))
                }
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                use serde::Deserialize as _;

                let map = RawOptionsOrdered::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(RawOptions::Ordered(map))
            }
        }

        deserializer.deserialize_any(RawOptionsVisitor)
    }
}

impl RawOptions {
    /// Convert raw options to a normalized vector of `RawEntryLabeled`.
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
    pub fn into_entries(self) -> Vec<RawEntryLabeled> {
        match self {
            Self::Plain(RawOptionsPlain(items)) => items
                .into_iter()
                .map(|value| RawEntryLabeled {
                    value,
                    label: None,
                    order: None,
                })
                .collect(),
            Self::Ordered(RawOptionsOrdered(mut entries)) => {
                entries.sort_by_key(|entry| entry.order);
                entries
                    .into_iter()
                    .map(|entry| RawEntryLabeled {
                        value: entry.value,
                        label: None,
                        order: None,
                    })
                    .collect()
            }
            Self::Labeled(RawOptionsLabeled(entries)) => {
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
                    .map(|(_, entry)| RawEntryLabeled {
                        value: entry.value,
                        label: entry.label,
                        order: None,
                    })
                    .collect()
            }
        }
    }
}

// ============================================================================
// RawOptionsPlain
// ============================================================================

/// Plain array of string values.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct RawOptionsPlain(Vec<Box<str>>);

impl From<Vec<Box<str>>> for RawOptionsPlain {
    #[inline]
    fn from(vec: Vec<Box<str>>) -> Self {
        Self(vec)
    }
}

// ============================================================================
// RawOptionsOrdered
// ============================================================================

/// Integer-keyed ordered object.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawOptionsOrdered(Vec<RawEntryInputOrdered>);

impl From<Vec<RawEntryInputOrdered>> for RawOptionsOrdered {
    #[inline]
    fn from(vec: Vec<RawEntryInputOrdered>) -> Self {
        Self(vec)
    }
}

#[expect(
    clippy::missing_inline_in_public_items,
    reason = "Serialize impl requires specific method signature"
)]
impl serde::Serialize for RawOptionsOrdered {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for entry in &self.0 {
            map.serialize_entry(
                &entry.order.value().to_string(),
                &entry.value,
            )?;
        }
        map.end()
    }
}

#[expect(
    clippy::missing_trait_methods,
    clippy::missing_inline_in_public_items,
    reason = "Custom serde Visitor requires specific method impls"
)]
impl<'de> serde::Deserialize<'de> for RawOptionsOrdered {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        struct RawOptionsOrderedVisitor;

        impl<'de> Visitor<'de> for RawOptionsOrderedVisitor {
            type Value = RawOptionsOrdered;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str(
                    "a map with integer string keys and string values",
                )
            }

            #[expect(
                clippy::excessive_nesting,
                reason = "Deserializing map entries with duplicate checks \
                          inherently requires some nesting"
            )]
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
                        return Err(Error::custom(format!(
                            "duplicate order key {}",
                            order.value()
                        )));
                    }
                    entries.push(RawEntryInputOrdered {
                        order,
                        value,
                    });
                }
                entries.sort_by_key(|entry| entry.order);
                Ok(RawOptionsOrdered(entries))
            }
        }

        deserializer.deserialize_map(RawOptionsOrderedVisitor)
    }
}

// ============================================================================
// RawOptionsLabeled
// ============================================================================

/// Rich entries with optional label and order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct RawOptionsLabeled(Vec<RawEntryLabeled>);

impl From<Vec<RawEntryLabeled>> for RawOptionsLabeled {
    #[inline]
    fn from(vec: Vec<RawEntryLabeled>) -> Self {
        Self(vec)
    }
}

// ============================================================================
// RawEntryLabeled
// ============================================================================

/// Rich option entry with optional label and input order.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::spec_string::RawEntryLabeled;
///
/// let entry = RawEntryLabeled {
///     value: "open".into(),
///     label: Some("Open".into()),
///     order: Some(1),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawEntryLabeled {
    /// The option value.
    pub value: Box<str>,
    /// Optional display label.
    pub label: Option<Box<str>>,
    /// Optional input order for sorting (lower = earlier).
    pub order: Option<RawEntryInputOrder>,
}

// ============================================================================
// RawEntryInputOrdered
// ============================================================================

/// Ordered map entry parsed from integer-keyed objects.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawEntryInputOrdered {
    /// The order key parsed from the map key.
    pub order: RawEntryInputOrder,
    /// The option value.
    pub value: Box<str>,
}

// ============================================================================
// RawEntryInputOrder
// ============================================================================

/// Input order position parsed from map keys.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize,
)]
#[non_exhaustive]
pub struct RawEntryInputOrder(u32);

#[expect(
    clippy::missing_trait_methods,
    clippy::missing_inline_in_public_items,
    reason = "Custom serde Visitor requires specific method impls"
)]
impl<'de> serde::Deserialize<'de> for RawEntryInputOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        struct RawEntryInputOrderVisitor;

        impl Visitor<'_> for RawEntryInputOrderVisitor {
            type Value = RawEntryInputOrder;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("a string integer >= 1")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                RawEntryInputOrder::try_from(v).map_err(Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(&v)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value: u32 = v.try_into().map_err(|_e| {
                    Error::custom("order key exceeds u32::MAX")
                })?;
                if value == 0 {
                    return Err(Error::custom("order key must be >= 1"));
                }
                Ok(RawEntryInputOrder(value))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v <= 0 {
                    return Err(Error::custom("order key must be >= 1"));
                }
                let value: u32 = v.try_into().map_err(|_e| {
                    Error::custom("order key exceeds u32::MAX")
                })?;
                Ok(RawEntryInputOrder(value))
            }
        }

        deserializer.deserialize_any(RawEntryInputOrderVisitor)
    }
}

impl RawEntryInputOrder {
    /// Returns the order value.
    #[inline]
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl TryFrom<&str> for RawEntryInputOrder {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
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
            RawOptions::Plain(RawOptionsPlain(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "open");
                assert_eq!(items[1].as_ref(), "closed");
                assert_eq!(items[2].as_ref(), "archived");
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_empty_list() {
        let json = "[]";
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Plain(RawOptionsPlain(items)) => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_map_from_json() {
        let json = r#"{"1": "todo", "2": "done", "3": "archived"}"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Ordered(RawOptionsOrdered(entries)) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(entries[0].order.value(), 1);
                assert_eq!(entries[0].value.as_ref(), "todo");
                assert_eq!(entries[1].order.value(), 2);
                assert_eq!(entries[1].value.as_ref(), "done");
                assert_eq!(entries[2].order.value(), 3);
                assert_eq!(entries[2].value.as_ref(), "archived");
            }
            _ => panic!("Expected Map variant"),
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
            RawOptions::Labeled(RawOptionsLabeled(entries)) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "open");
                assert_eq!(entries[0].label.as_deref(), Some("Open"));
                assert_eq!(entries[0].order, Some(RawEntryInputOrder(1)));
                assert_eq!(entries[1].value.as_ref(), "closed");
                assert_eq!(entries[1].label.as_deref(), Some("Closed"));
                assert_eq!(entries[1].order, Some(RawEntryInputOrder(2)));
            }
            _ => panic!("Expected Rich variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_rich_with_minimal_fields() {
        let json = r#"[{"value": "open"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Labeled(RawOptionsLabeled(entries)) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].value.as_ref(), "open");
                assert_eq!(entries[0].label, None);
                assert_eq!(entries[0].order, None);
            }
            _ => panic!("Expected Rich variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_from_yaml_list() {
        let yaml = "- open\n- closed\n- archived\n";
        let options: RawOptions = serde_yaml::from_str(yaml).unwrap();
        match options {
            RawOptions::Plain(RawOptionsPlain(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "open");
            }
            _ => panic!("Expected List variant"),
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
            RawOptions::Plain(RawOptionsPlain(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_ref(), "open");
                assert_eq!(items[1].as_ref(), "closed");
            }
            _ => panic!("Expected List variant"),
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
        let options = RawOptions::Plain(RawOptionsPlain(vec![
            "a".into(),
            "b".into(),
            "c".into(),
        ]));
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "a");
        assert_eq!(entries[1].value.as_ref(), "b");
        assert_eq!(entries[2].value.as_ref(), "c");
        assert!(entries.iter().all(|e| e.label.is_none()));
    }

    #[test]
    fn raw_options_into_entries_map_sorts_by_key() {
        let options = RawOptions::Ordered(RawOptionsOrdered(vec![
            RawEntryInputOrdered {
                order: RawEntryInputOrder(3),
                value: "third".into(),
            },
            RawEntryInputOrdered {
                order: RawEntryInputOrder(1),
                value: "first".into(),
            },
            RawEntryInputOrdered {
                order: RawEntryInputOrder(2),
                value: "second".into(),
            },
        ]));
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "first");
        assert_eq!(entries[1].value.as_ref(), "second");
        assert_eq!(entries[2].value.as_ref(), "third");
    }

    #[test]
    fn raw_options_into_entries_rich_sorts_by_order() {
        let options = RawOptions::Labeled(RawOptionsLabeled(vec![
            RawEntryLabeled {
                value: "c".into(),
                label: Some("Third".into()),
                order: Some(RawEntryInputOrder(3)),
            },
            RawEntryLabeled {
                value: "a".into(),
                label: Some("First".into()),
                order: Some(RawEntryInputOrder(1)),
            },
            RawEntryLabeled {
                value: "b".into(),
                label: Some("Second".into()),
                order: Some(RawEntryInputOrder(2)),
            },
        ]));
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "a");
        assert_eq!(entries[0].label.as_deref(), Some("First"));
        assert_eq!(entries[1].value.as_ref(), "b");
        assert_eq!(entries[2].value.as_ref(), "c");
    }

    #[test]
    fn raw_options_into_entries_rich_uses_array_position_when_no_order() {
        let options = RawOptions::Labeled(RawOptionsLabeled(vec![
            RawEntryLabeled {
                value: "first".into(),
                label: None,
                order: None,
            },
            RawEntryLabeled {
                value: "second".into(),
                label: None,
                order: None,
            },
        ]));
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
            RawOptions::Labeled(RawOptionsLabeled(entries)) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "a");
                assert_eq!(entries[0].label, None);
            }
            _ => panic!("Expected Rich variant for array of objects"),
        }
    }

    #[test]
    fn raw_options_disambiguates_strings_as_list() {
        let json = r#"["value", "label", "order"]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Plain(RawOptionsPlain(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "value");
                assert_eq!(items[1].as_ref(), "label");
                assert_eq!(items[2].as_ref(), "order");
            }
            _ => panic!("Expected List variant for string array"),
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
    fn raw_options_rich_with_extra_fields_ignored() {
        let json = r#"[{"value": "a", "label": "A", "extra": "ignored"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Labeled(RawOptionsLabeled(entries)) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].value.as_ref(), "a");
                assert_eq!(entries[0].label.as_deref(), Some("A"));
            }
            _ => panic!("Expected Rich variant"),
        }
    }

    #[test]
    fn raw_options_empty_map() {
        let json = "{}";
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Ordered(RawOptionsOrdered(entries)) => {
                assert!(entries.is_empty());
            }
            _ => panic!("Expected Map variant for empty object"),
        }
    }
}
