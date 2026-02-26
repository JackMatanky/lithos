//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input \
              types"
)]

use std::collections::BTreeMap;

use super::{
    error::{SchemaError, SchemaIngestionError},
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, OptionEntry, PropertySpec,
        PropertySpecType, StringSpec,
    },
};

/// Current supported schema version.
pub const SCHEMA_VERSION: &str = "1.0";

/// Raw schema definition loaded from vault files.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawSchema, RawProperty, RawPropertyInline};
/// use lithos_core::schema::raw::{RawPropertySpec, RawBoolSpec};
/// use std::collections::HashMap;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
///
/// let mut properties = HashMap::new();
/// properties.insert(
///     "archived".into(),
///     RawProperty::Inline(RawPropertyInline {
///         required: false,
///         multi: false,
///         spec: RawPropertySpec::Bool(RawBoolSpec),
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
    /// Schema format version (must be "1.0").
    #[serde(rename = "$version")]
    pub version: Box<str>,
    /// Unique schema name.
    pub name: Box<str>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<Box<str>>,
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: Vec<Box<str>>,
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawProperty>,
}

impl RawSchema {
    /// Validate the schema version matches the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match.
    #[inline]
    pub fn validate_version(
        &self,
        path: &str,
    ) -> Result<(), SchemaIngestionError> {
        if self.version.as_ref() != SCHEMA_VERSION {
            return Err(SchemaIngestionError::UnsupportedVersion {
                path: path.into(),
                found: self.version.clone(),
                expected: SCHEMA_VERSION.into(),
            });
        }
        Ok(())
    }
}

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
/// let reference = RawPropertyRef {
///     ref_path: "property_bank#/flag".into(),
///     required: None,
///     multi: None,
///     number: RawNumberSpec::default(),
///     string: RawStringSpec::default(),
///     date: RawDateSpec::default(),
///     file: RawFileSpec::default(),
/// };
/// let _ = reference;
/// ```
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

impl RawPropertySpec {
    /// Get the spec type identifier.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     property_spec::PropertySpecType,
    ///     raw::{RawBoolSpec, RawPropertySpec},
    /// };
    ///
    /// let spec = RawPropertySpec::Bool(RawBoolSpec);
    /// assert_eq!(spec.spec_type(), PropertySpecType::Bool);
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &enum are intentional here for \
                  readability"
    )]
    pub const fn spec_type(&self) -> PropertySpecType {
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
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::raw::{RawBoolSpec, RawPropertySpec};
    ///
    /// let spec = RawPropertySpec::Bool(RawBoolSpec);
    /// let _validated = spec.try_into_validated()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_into_validated(self) -> Result<PropertySpec, SchemaError> {
        match self {
            Self::Bool(_) => Ok(PropertySpec::Bool(BoolSpec::default())),
            Self::Date(def) => {
                let format = def.format.ok_or_else(|| {
                    SchemaError::ValidationFailed(
                        "date format is required".into(),
                    )
                })?;
                Ok(PropertySpec::Date(DateSpec::try_new(&format)?))
            }
            Self::File(def) => Ok(PropertySpec::File(FileSpec::try_new(
                def.directory.map(String::from),
                def.file_class.map(String::from),
            )?)),
            Self::Number(def) => Ok(PropertySpec::Number(NumberSpec::try_new(
                def.min, def.max, def.step,
            )?)),
            Self::String(def) => Ok(PropertySpec::String(StringSpec::try_new(
                def.pattern,
                def.options.map(RawOptions::into_entries),
            )?)),
        }
    }
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

/// String property definition.
///
/// Only `options` and `pattern` are supported per the meta-schema.
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
    /// Optional regex pattern.
    pub pattern: Option<Box<str>>,
}

/// Raw property bank loaded from vault files.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawPropertyBank;
///
/// let bank = RawPropertyBank {
///     properties: std::collections::HashMap::new(),
/// };
/// let _ = bank;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version (must be "1.0").
    #[serde(rename = "$version")]
    pub version: Box<str>,
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawPropertyBankEntry>,
}

impl RawPropertyBank {
    /// Validate the property bank version matches the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match.
    #[inline]
    pub fn validate_version(
        &self,
        path: &str,
    ) -> Result<(), SchemaIngestionError> {
        if self.version.as_ref() != SCHEMA_VERSION {
            return Err(SchemaIngestionError::UnsupportedVersion {
                path: path.into(),
                found: self.version.clone(),
                expected: SCHEMA_VERSION.into(),
            });
        }
        Ok(())
    }
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
            version: SCHEMA_VERSION.into(),
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
            version: SCHEMA_VERSION.into(),
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
        let reference = RawPropertyRef {
            ref_path: "property_bank#/status".into(),
            required: None,
            multi: None,
            number: RawNumberSpec::default(),
            string: RawStringSpec::default(),
            date: RawDateSpec::default(),
            file: RawFileSpec::default(),
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }
}
