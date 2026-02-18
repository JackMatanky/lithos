//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input \
              types"
)]

use std::collections::BTreeMap;

use super::{
    error::SchemaError,
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, OptionEntry, PropertySpec,
        PropertySpecType, StringSpec,
    },
};

/// Raw schema definition loaded from vault files.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawSchema, RawPropertyEntry, RawPropertyInline};
/// use lithos_core::schema::raw::{RawPropertySpec, BoolSpecDef};
/// use std::collections::HashMap;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
///
/// let mut properties = HashMap::new();
/// properties.insert(
///     "archived".into(),
///     RawPropertyEntry::Inline(RawPropertyInline {
///         required: false,
///         multi: false,
///         spec: RawPropertySpec::Bool(BoolSpecDef::default()),
///     }),
/// );
/// let schema = RawSchema {
///     name: "note".into(),
///     extends: None,
///     excludes: Vec::new(),
///     properties,
/// };
/// assert_eq!(schema.properties.len(), 1, "Schema should contain one property");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Unique schema name.
    pub name: Box<str>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<Box<str>>,
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: Vec<Box<str>>,
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawPropertyEntry>,
}

/// Raw property input definition (Inline or Ref).
///
/// Discriminated by presence of `$ref` field. Ref is tried first because
/// it has a required `$ref` field that Inline never has.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Raw property entry for schema properties map.
///
/// Used in `RawSchema.properties` where the name is the map key.
/// Discriminated by presence of `$ref` field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawPropertyEntry {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Inline variant of a raw property.
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference path (e.g., `property_bank#/date_iso_8601`).
    #[serde(rename = "$ref")]
    pub ref_path: Box<str>,
    /// Override whether property is required.
    pub required: Option<bool>,
    /// Override whether property accepts multiple values.
    pub multi: Option<bool>,
    /// Override allowed values (string type only).
    pub options: Option<RawOptions>,
    /// Override regex pattern (string type only).
    pub pattern: Option<Box<str>>,
    /// Override minimum value (number type only).
    pub min: Option<f64>,
    /// Override maximum value (number type only).
    pub max: Option<f64>,
    /// Override step increment (number type only).
    pub step: Option<f64>,
    /// Override date format (date type only).
    pub format: Option<Box<str>>,
    /// Override directory restriction (file type only).
    pub directory: Option<Box<str>>,
    /// Override file class restriction (file type only).
    pub file_class: Option<Box<str>>,
}

/// Raw property specification (serde-facing input type).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum RawPropertySpec {
    /// Boolean property definition (marker type).
    Bool(BoolSpecDef),
    /// Date property definition.
    Date(DateSpecDef),
    /// File property definition.
    File(FileSpecDef),
    /// Number property definition.
    Number(NumberSpecDef),
    /// String property definition.
    String(StringSpecDef),
}

impl RawPropertySpec {
    /// Get the spec type identifier.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub fn spec_type(&self) -> PropertySpecType {
        match self {
            Self::Bool(_) => PropertySpecType::Bool,
            Self::Date(_) => PropertySpecType::Date,
            Self::File(_) => PropertySpecType::File,
            Self::Number(_) => PropertySpecType::Number,
            Self::String(_) => PropertySpecType::String,
        }
    }

    /// Validate and compile a persisted definition into a validated spec.
    ///
    /// # Errors
    /// Returns `SchemaError` if the definition is invalid.
    #[inline]
    pub fn try_into_validated(self) -> Result<PropertySpec, SchemaError> {
        match self {
            Self::Bool(_) => Ok(PropertySpec::Bool(BoolSpec::default())),
            Self::Date(def) => {
                Ok(PropertySpec::Date(DateSpec::try_new(&def.format)?))
            }
            Self::File(def) => Ok(PropertySpec::File(FileSpec::try_new(
                def.directory.map(String::from),
                def.file_class.map(String::from),
            )?)),
            Self::Number(def) => Ok(PropertySpec::Number(NumberSpec::try_new(
                def.min, def.max, def.step,
            )?)),
            Self::String(def) => Ok(PropertySpec::String(StringSpec::try_new(
                def.min_length,
                def.max_length,
                def.pattern,
                def.options.map(RawOptions::into_entries),
            )?)),
        }
    }
}

/// Boolean property definition (marker type).
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct BoolSpecDef;

/// Date property definition.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct DateSpecDef {
    /// Date format string (using chrono format tokens).
    pub format: Box<str>,
}

/// File property definition.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct FileSpecDef {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<Box<str>>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<Box<str>>,
}

/// Number property definition.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct NumberSpecDef {
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

/// String property definition.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct StringSpecDef {
    /// Optional allowed values in one of three formats.
    pub options: Option<RawOptions>,
    /// Optional max length.
    pub max_length: Option<usize>,
    /// Optional min length.
    pub min_length: Option<usize>,
    /// Optional regex pattern.
    pub pattern: Option<Box<str>>,
}

/// Raw property bank loaded from vault files.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawPropertyBankEntry>,
}

/// Entry in the raw property bank.
///
/// The property name is the map key, not a field here.
/// `required` is not present because the bank is schema-agnostic.
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

/// Raw options definition supporting three formats.
///
/// # Modes
///
/// - **Mode 1 (List)**: `["a", "b"]` — plain array of values
/// - **Mode 2 (Map)**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed
///   object
/// - **Mode 3 (Rich)**: `[{"value": "a", "label": "A", "order": 1}]` — rich
///   entries
///
/// Serde deserializes untagged variants in declaration order. Arrays are tried
/// as `List` first (strings), then `Rich` (objects). Objects are tried as
/// `Map`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawOptionEntry {
    /// The option value.
    pub value: Box<str>,
    /// Optional display label.
    pub label: Option<Box<str>>,
    /// Optional display order (lower = earlier).
    pub order: Option<u32>,
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
    #[inline]
    #[must_use]
    pub fn into_entries(self) -> Vec<OptionEntry> {
        match self {
            Self::List(items) => items
                .into_iter()
                .map(|value| OptionEntry {
                    value,
                    label: None,
                })
                .collect(),
            Self::Map(map) => {
                let mut entries: Vec<_> = map
                    .into_iter()
                    .filter_map(|(key, value)| {
                        key.parse::<u32>().ok().map(|order| (order, value))
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, value)| OptionEntry {
                        value,
                        label: None,
                    })
                    .collect()
            }
            Self::Rich(entries) => {
                let mut entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let order = entry.order.unwrap_or_else(|| {
                            u32::try_from(idx).unwrap_or(u32::MAX)
                        });
                        (order, entry)
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, entry)| OptionEntry {
                        value: entry.value,
                        label: entry.label,
                    })
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn schema_name() -> Box<str> {
        "note".into()
    }

    #[test]
    fn raw_schema_defaults_to_empty_excludes() {
        let schema = RawSchema {
            name: schema_name(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        assert!(
            schema.excludes.is_empty(),
            "RawSchema should have empty excludes by default"
        );
    }

    #[test]
    fn raw_schema_defaults_to_no_extends() {
        let schema = RawSchema {
            name: schema_name(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        assert!(
            schema.extends.is_none(),
            "RawSchema should have no extends by default"
        );
    }

    #[test]
    fn raw_property_inline_variant_constructs() {
        let inline = RawPropertyInline {
            required: false,
            multi: false,
            spec: RawPropertySpec::Bool(BoolSpecDef::default()),
        };
        let inline_variant = RawProperty::Inline(inline);

        assert!(
            matches!(inline_variant, RawProperty::Inline(_)),
            "RawProperty should be Inline variant"
        );
    }

    #[test]
    fn raw_property_ref_variant_constructs() {
        let reference = RawPropertyRef {
            ref_path: "property_bank#/status".into(),
            required: None,
            multi: None,
            options: None,
            pattern: None,
            min: None,
            max: None,
            step: None,
            format: None,
            directory: None,
            file_class: None,
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }

    #[test]
    fn raw_property_entry_inline_constructs() {
        let inline = RawPropertyInline {
            required: false,
            multi: false,
            spec: RawPropertySpec::Bool(BoolSpecDef::default()),
        };
        let entry = RawPropertyEntry::Inline(inline);

        assert!(
            matches!(entry, RawPropertyEntry::Inline(_)),
            "RawPropertyEntry should be Inline variant"
        );
    }

    #[test]
    fn raw_property_entry_ref_constructs() {
        let reference = RawPropertyRef {
            ref_path: "property_bank#/status".into(),
            required: None,
            multi: None,
            options: None,
            pattern: None,
            min: None,
            max: None,
            step: None,
            format: None,
            directory: None,
            file_class: None,
        };
        let entry = RawPropertyEntry::Ref(reference);

        assert!(
            matches!(entry, RawPropertyEntry::Ref(_)),
            "RawPropertyEntry should be Ref variant"
        );
    }
}
