//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input \
              types"
)]
#![expect(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types despite \
              #[non_exhaustive] on source types."
)]

use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;

use super::{
    error::SchemaError,
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, PropertySpec,
        PropertySpecType, StringSpec,
    },
};

/// Raw schema definition (Input).
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawSchema, RawProperty, RawPropertyInline};
/// use lithos_core::schema::raw::{RawPropertySpec, BoolSpecDef};
/// use std::collections::BTreeSet;
/// use uuid::Uuid;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
///
/// let schema = RawSchema::new(
///     Uuid::now_v7(),
///     "note".into(),
///     None,
///     BTreeSet::new(),
///     vec![RawProperty::Inline(RawPropertyInline {
///         id: Uuid::now_v7(),
///         name: "archived".into(),
///         required: false,
///         array: false,
///         spec: RawPropertySpec::Bool(BoolSpecDef::default()),
///     })],
/// );
/// assert_eq!(schema.properties.len(), 1, "Schema should contain one property");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Unique identity for the schema definition.
    pub id: Uuid,
    /// Unique schema name.
    pub name: Box<str>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<Box<str>>,
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: BTreeSet<Box<str>>,
    /// List of raw property definitions.
    pub properties: Vec<RawProperty>,
}

impl RawSchema {
    /// Create a new `RawSchema`.
    #[inline]
    #[must_use]
    pub fn new(
        id: Uuid,
        name: Box<str>,
        extends: Option<Box<str>>,
        excludes: BTreeSet<Box<str>>,
        properties: Vec<RawProperty>,
    ) -> Self {
        Self {
            id,
            name,
            extends,
            excludes,
            properties,
        }
    }
}

/// Raw property input definition (Inline or Ref).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// An inline property definition.
    Inline(RawPropertyInline),
    /// A reference to a property in the `PropertyBank`.
    Ref(RawPropertyRef),
}

/// Inline variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Unique identity assigned by adapter.
    pub id: Uuid,
    /// Property name.
    pub name: Box<str>,
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Type-specific validation constraints.
    pub spec: RawPropertySpec,
}

/// Reference variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference string (e.g., "#/properties/title").
    #[serde(rename = "$ref")]
    pub ref_path: Box<str>,
}

/// Raw property specification (serde-facing input type).
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
                def.enum_values,
            )?)),
        }
    }
}

/// Boolean property definition (marker type).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct BoolSpecDef;

/// Date property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct DateSpecDef {
    /// Date format string (using chrono format tokens).
    pub format: Box<str>,
}

/// File property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FileSpecDef {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<Box<str>>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<Box<str>>,
}

/// Number property definition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct StringSpecDef {
    /// Optional enum of allowed values.
    pub enum_values: Option<Vec<Box<str>>>,
    /// Optional max length.
    pub max_length: Option<usize>,
    /// Optional min length.
    pub min_length: Option<usize>,
    /// Optional regex pattern.
    pub pattern: Option<Box<str>>,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::schema::raw::BoolSpecDef;

    const TEST_SCHEMA_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0601);
    const TEST_PROPERTY_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0602);

    fn schema_name() -> Box<str> {
        "note".into()
    }

    #[test]
    fn raw_schema_defaults_to_empty_excludes() {
        let schema = RawSchema::new(
            TEST_SCHEMA_ID,
            schema_name(),
            None,
            BTreeSet::new(),
            Vec::new(),
        );

        assert!(
            schema.excludes.is_empty(),
            "RawSchema should have empty excludes by default"
        );
    }

    #[test]
    fn raw_schema_defaults_to_no_extends() {
        let schema = RawSchema::new(
            TEST_SCHEMA_ID,
            schema_name(),
            None,
            BTreeSet::new(),
            Vec::new(),
        );

        assert!(
            schema.extends.is_none(),
            "RawSchema should have no extends by default"
        );
    }

    #[test]
    fn raw_property_inline_variant_constructs() {
        let inline = RawPropertyInline {
            id: TEST_PROPERTY_ID,
            name: "archived".into(),
            required: false,
            array: false,
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
            ref_path: "status".into(),
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }
}
