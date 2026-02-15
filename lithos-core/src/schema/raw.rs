//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input \
              types"
)]

use std::collections::HashSet;

use uuid::Uuid;

use super::{
    aggregate::SchemaName, property::PropertyName,
    property_spec::PropertySpecDef,
};

/// Raw schema definition (Input).
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawSchema, RawProperty, RawPropertyInline};
/// use lithos_core::schema::aggregate::SchemaName;
/// use lithos_core::schema::property_spec::{PropertySpecDef, BoolSpecDef};
/// use std::collections::HashSet;
/// use uuid::Uuid;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
///
/// let schema = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("note")?,
///     None,
///     HashSet::new(),
///     vec![RawProperty::Inline(RawPropertyInline {
///         id: Uuid::now_v7(),
///         name: "archived".to_string(),
///         required: false,
///         array: false,
///         spec: PropertySpecDef::Bool(BoolSpecDef::default()),
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
    pub name: SchemaName,
    /// Optional parent schema name for inheritance.
    pub extends: Option<SchemaName>,
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: HashSet<PropertyName>,
    /// List of raw property definitions.
    pub properties: Vec<RawProperty>,
}

impl RawSchema {
    /// Create a new `RawSchema`.
    #[inline]
    #[must_use]
    pub fn new(
        id: Uuid,
        name: SchemaName,
        extends: Option<SchemaName>,
        excludes: HashSet<PropertyName>,
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
    pub name: String,
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpecDef,
}

/// Reference variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference string (e.g., "#/properties/title").
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::property_spec::BoolSpecDef;

    const TEST_SCHEMA_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0601);
    const TEST_PROPERTY_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0602);

    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture uses expect for deterministic setup. Failure \
                  indicates invalid test data. Expect is idiomatic in setup."
    )]
    fn schema_name() -> SchemaName {
        SchemaName::new("note").expect("valid schema name")
    }

    #[test]
    fn raw_schema_defaults_to_empty_excludes() {
        let schema = RawSchema::new(
            TEST_SCHEMA_ID,
            schema_name(),
            None,
            HashSet::new(),
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
            HashSet::new(),
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
            name: "archived".to_owned(),
            required: false,
            array: false,
            spec: PropertySpecDef::Bool(BoolSpecDef::default()),
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
            ref_path: "status".to_owned(),
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }
}
