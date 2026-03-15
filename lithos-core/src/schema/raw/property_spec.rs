//! Raw property specification types.
//!
//! Defines the type-specific validation constraints for properties:
//! - Property specs (Bool, Date, File, Number, String)
//! - String options (List, Map, Rich)
//! - String patterns (named formats and custom regex)

use std::collections::BTreeMap;

/// Raw property specification (serde-facing input type).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::{RawBoolSpec, RawPropertySpec};
///
/// let spec = RawPropertySpec::Bool(RawBoolSpec);
/// match spec {
///     RawPropertySpec::Bool(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum RawPropertySpec {
    /// Boolean property definition (marker type).
    Bool(RawBoolSpec),
    /// Date property definition.
    Date(RawDateSpec),
    /// File property definition.
    File(RawFileSpec),
    /// Number property definition.
    Number(RawNumberSpec),
    /// String property definition.
    String(RawStringSpec),
}

/// Boolean property definition (marker type).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawBoolSpec;
///
/// let _spec = RawBoolSpec;
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[expect(
    clippy::exhaustive_structs,
    reason = "Marker type with no fields; non_exhaustive prevents construction"
)]
pub struct RawBoolSpec;

/// Date property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// (where `format` is required) and override contexts (where `None`
/// means "don't override").
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawDateSpec;
///
/// let _spec = RawDateSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawDateSpec {
    /// Date format string (using chrono format tokens).
    pub format: Option<Box<str>>,
}

/// File property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawFileSpec;
///
/// let _spec = RawFileSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawFileSpec {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<Box<str>>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<Box<str>>,
}

/// Number property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawNumberSpec;
///
/// let _spec = RawNumberSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawNumberSpec {
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

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

/// String property definition.
///
/// Supports `options` and `pattern` per the meta-schema.
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawStringSpec;
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

/// Raw options definition supporting three formats.
///
/// # Modes
///
/// - **Mode 1 (List)**: `["a", "b"]` — plain array of string values
/// - **Mode 2 (Map)**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed
///   object
/// - **Mode 3 (Rich)**: `[{"value": "a", "label": "A", "order": 1}]` — rich
///   entries with labels
///
/// # Deserialization Strategy
///
/// Uses custom deserializer with explicit type checking:
/// 1. If sequence: try as List (strings), then Rich (objects)
/// 2. If map: deserialize as Map
/// 3. Fail with clear error for other types
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawOptions;
///
/// let options = RawOptions::List(vec!["open".into(), "closed".into()]);
/// match options {
///     RawOptions::List(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum RawOptions {
    /// Mode 1: Plain array of string values.
    List(Vec<Box<str>>),
    /// Mode 2: Integer-keyed ordered object.
    Map(BTreeMap<Box<str>, Box<str>>),
    /// Mode 3: Rich entries with optional label and order.
    Rich(Vec<RawOptionEntry>),
}

/// Rich option entry with optional label and display order.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawOptionEntry;
///
/// let entry = RawOptionEntry {
///     value: "open".into(),
///     label: Some("Open".into()),
///     order: Some(1),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawOptionEntry {
    /// The option value.
    pub value: Box<str>,
    /// Optional display label.
    pub label: Option<Box<str>>,
    /// Optional display order (lower = earlier).
    pub order: Option<u32>,
}

// Custom deserializer for RawOptions to avoid relying on untagged variant order
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
                // Peek at first element to determine if List or Rich
                let first = seq.next_element::<serde_json::Value>()?;
                if let Some(value) = first {
                    if value.is_string() {
                        // List mode: array of strings
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
                        Ok(RawOptions::List(items))
                    } else if value.is_object() {
                        // Rich mode: array of objects
                        let first_entry: RawOptionEntry =
                            serde_json::from_value(value)
                                .map_err(Error::custom)?;
                        let mut entries = vec![first_entry];
                        while let Some(entry) =
                            seq.next_element::<RawOptionEntry>()?
                        {
                            entries.push(entry);
                        }
                        Ok(RawOptions::Rich(entries))
                    } else {
                        Err(Error::custom(
                            "expected array elements to be strings or objects",
                        ))
                    }
                } else {
                    // Empty array defaults to List
                    Ok(RawOptions::List(vec![]))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                // Map mode: object with string keys and string values
                let mut result = BTreeMap::new();
                while let Some((key, value)) =
                    map.next_entry::<Box<str>, Box<str>>()?
                {
                    result.insert(key, value);
                }
                Ok(RawOptions::Map(result))
            }
        }

        deserializer.deserialize_any(RawOptionsVisitor)
    }
}

impl RawOptions {
    /// Convert raw options to a normalized vector of `OptionEntry`.
    ///
    /// # Modes
    ///
    /// - **List**: entries have `value = item`, `label = None`, order
    ///   preserved.
    /// - **Map**: keys parsed as integers, sorted by key, entries have `value =
    ///   map_value`, `label = None`.
    /// - **Rich**: sorted by `order` field (then array position), entries have
    ///   `value` and `label`.
    ///
    /// # Panics
    /// Panics if Rich mode option list has more than `u32::MAX` entries (>4
    /// billion). This is unrealistic in practice and indicates a malformed
    /// input.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::raw::RawOptions;
    ///
    /// let entries = RawOptions::List(vec!["open".into()]).into_entries();
    /// assert_eq!(entries.len(), 1);
    /// assert_eq!(entries[0].value.as_ref(), "open");
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Fail-fast on unrealistic >4 billion options prevents silent \
                  data corruption"
    )]
    pub fn into_entries(self) -> Vec<RawOptionEntry> {
        match self {
            Self::List(items) => items
                .into_iter()
                .map(|value| RawOptionEntry {
                    value,
                    label: None,
                    order: None,
                })
                .collect(),
            Self::Map(map) => {
                let mut entries: Vec<_> = map
                    .into_iter()
                    .filter_map(|(key, value)| {
                        key.parse::<u32>()
                            .inspect_err(|e| {
                                tracing::debug!(
                                    key = %key,
                                    error = %e,
                                    "Option map key is not a valid u32, entry will be skipped"
                                );
                            })
                            .ok()
                            .map(|order| (order, value))
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, value)| RawOptionEntry {
                        value,
                        label: None,
                        order: None,
                    })
                    .collect()
            }
            Self::Rich(entries) => {
                let mut entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let order = entry.order.unwrap_or_else(|| {
                            u32::try_from(idx).expect(
                                "Option list index exceeds u32::MAX (>4 \
                                 billion entries)",
                            )
                        });
                        (order, entry)
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, entry)| RawOptionEntry {
                        value: entry.value,
                        label: entry.label,
                        order: None,
                    })
                    .collect()
            }
        }
    }
}

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
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn raw_options_deserializes_list_from_json() {
        let json = r#"["open", "closed", "archived"]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::List(items) => {
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
            RawOptions::List(items) => {
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
            RawOptions::Map(map) => {
                assert_eq!(map.len(), 3);
                assert_eq!(
                    map.get("1").map(std::convert::AsRef::as_ref),
                    Some("todo")
                );
                assert_eq!(
                    map.get("2").map(std::convert::AsRef::as_ref),
                    Some("done")
                );
                assert_eq!(
                    map.get("3").map(std::convert::AsRef::as_ref),
                    Some("archived")
                );
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
            RawOptions::Rich(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "open");
                assert_eq!(entries[0].label.as_deref(), Some("Open"));
                assert_eq!(entries[0].order, Some(1));
                assert_eq!(entries[1].value.as_ref(), "closed");
                assert_eq!(entries[1].label.as_deref(), Some("Closed"));
                assert_eq!(entries[1].order, Some(2));
            }
            _ => panic!("Expected Rich variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_rich_with_minimal_fields() {
        let json = r#"[{"value": "open"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Rich(entries) => {
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
        let yaml = "
- open
- closed
- archived
";
        let options: RawOptions = serde_yaml::from_str(yaml).unwrap();
        match options {
            RawOptions::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_ref(), "open");
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn raw_options_deserializes_from_toml_inline_array() {
        // TOML requires inline arrays to be in a table context
        let toml_str = r#"options = ["open", "closed"]"#;
        #[derive(serde::Deserialize)]
        struct Wrapper {
            options: RawOptions,
        }
        let wrapper: Wrapper = toml::from_str(toml_str).unwrap();
        match wrapper.options {
            RawOptions::List(items) => {
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
        let options =
            RawOptions::List(vec!["a".into(), "b".into(), "c".into()]);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "a");
        assert_eq!(entries[1].value.as_ref(), "b");
        assert_eq!(entries[2].value.as_ref(), "c");
        assert!(entries.iter().all(|e| e.label.is_none()));
    }

    #[test]
    fn raw_options_into_entries_map_sorts_by_key() {
        let mut map = BTreeMap::new();
        map.insert("3".into(), "third".into());
        map.insert("1".into(), "first".into());
        map.insert("2".into(), "second".into());
        let options = RawOptions::Map(map);
        let entries = options.into_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value.as_ref(), "first");
        assert_eq!(entries[1].value.as_ref(), "second");
        assert_eq!(entries[2].value.as_ref(), "third");
    }

    #[test]
    fn raw_options_into_entries_rich_sorts_by_order() {
        let options = RawOptions::Rich(vec![
            RawOptionEntry {
                value: "c".into(),
                label: Some("Third".into()),
                order: Some(3),
            },
            RawOptionEntry {
                value: "a".into(),
                label: Some("First".into()),
                order: Some(1),
            },
            RawOptionEntry {
                value: "b".into(),
                label: Some("Second".into()),
                order: Some(2),
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
        let options = RawOptions::Rich(vec![
            RawOptionEntry {
                value: "first".into(),
                label: None,
                order: None,
            },
            RawOptionEntry {
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
        // Array of objects with only `value` field should deserialize as Rich
        let json = r#"[{"value": "a"}, {"value": "b"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Rich(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].value.as_ref(), "a");
                assert_eq!(entries[0].label, None);
            }
            _ => panic!("Expected Rich variant for array of objects"),
        }
    }

    #[test]
    fn raw_options_disambiguates_strings_as_list() {
        // Array of strings should always deserialize as List
        let json = r#"["value", "label", "order"]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::List(items) => {
                assert_eq!(items.len(), 3);
                // These are literal strings, not field names
                assert_eq!(items[0].as_ref(), "value");
                assert_eq!(items[1].as_ref(), "label");
                assert_eq!(items[2].as_ref(), "order");
            }
            _ => panic!("Expected List variant for string array"),
        }
    }

    #[test]
    fn raw_options_rejects_array_of_numbers() {
        // Numbers are not valid option values
        let json = "[1, 2, 3]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of numbers (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_array_of_bools() {
        // Booleans are not valid option values
        let json = "[true, false]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of booleans (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_array_of_nulls() {
        // Nulls are not valid option values
        let json = "[null, null]";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject array of nulls (not strings or objects)"
        );
    }

    #[test]
    fn raw_options_rejects_nested_arrays() {
        // Nested arrays are not a valid mode
        let json = r#"[["a", "b"], ["c", "d"]]"#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject nested arrays (not a valid mode)"
        );
    }

    #[test]
    fn raw_options_rejects_string_primitive() {
        // Single string is not an array or map
        let json = r#""single_value""#;
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject string primitive (must be array or map)"
        );
    }

    #[test]
    fn raw_options_rejects_number_primitive() {
        // Single number is not an array or map
        let json = "42";
        let result: Result<RawOptions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Should reject number primitive (must be array or map)"
        );
    }

    #[test]
    fn raw_options_map_with_non_numeric_keys() {
        // Map keys can be any string, not just numeric
        let json = r#"{"open": "Open", "closed": "Closed"}"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Map(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(
                    map.get("open").map(std::convert::AsRef::as_ref),
                    Some("Open")
                );
            }
            _ => panic!("Expected Map variant"),
        }
    }

    #[test]
    fn raw_options_rich_with_extra_fields_ignored() {
        // Extra fields in Rich entries should be ignored (forward compat)
        let json = r#"[{"value": "a", "label": "A", "extra": "ignored"}]"#;
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Rich(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].value.as_ref(), "a");
                assert_eq!(entries[0].label.as_deref(), Some("A"));
            }
            _ => panic!("Expected Rich variant"),
        }
    }

    #[test]
    fn raw_options_empty_map() {
        // Empty map should deserialize successfully
        let json = "{}";
        let options: RawOptions = serde_json::from_str(json).unwrap();
        match options {
            RawOptions::Map(map) => {
                assert!(map.is_empty());
            }
            _ => panic!("Expected Map variant for empty object"),
        }
    }
}
