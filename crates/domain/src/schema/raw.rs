//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input \
              types"
)]

use std::collections::HashSet;

use uuid::Uuid;

use super::{
    aggregate::SchemaName, property::PropertyName, property_spec::PropertySpec,
};

/// Raw schema definition (Input).
///
/// # Examples
/// ```ignore
/// use lithos_domain::{RawSchema, RawProperty, RawPropertyInline, SchemaName, PropertySpec, BoolSpec};
/// use std::collections::HashSet;
/// use uuid::Uuid;
///
/// let schema = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("note".to_string()).unwrap(),
///     None,
///     HashSet::new(),
///     vec![RawProperty::Inline(RawPropertyInline {
///         id: Uuid::now_v7(),
///         name: "archived".to_string(),
///         required: false,
///         array: false,
///         spec: PropertySpec::Bool(BoolSpec::default()),
///     })],
/// );
/// assert_eq!(schema.properties.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: HashSet<PropertyName>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<SchemaName>,
    /// Unique identity for the schema definition.
    pub id: Uuid,
    /// Unique schema name.
    pub name: SchemaName,
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
            excludes,
            extends,
            id,
            name,
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

/// Reference variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference string (e.g., "#/properties/title").
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

/// Inline variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Unique identity assigned by adapter.
    pub id: Uuid,
    /// Property name.
    pub name: String,
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use expect for readability"
)]
mod tests {
    use super::*;
    use crate::BoolSpec;

    #[test]
    fn raw_schema_initializes_fields() {
        // GIVEN: a raw schema definition
        let schema = RawSchema::new(
            Uuid::now_v7(),
            SchemaName::new("note".to_owned()).expect("valid schema name"),
            None,
            HashSet::new(),
            Vec::new(),
        );

        // THEN: fields are preserved
        assert!(schema.excludes.is_empty());
        assert!(schema.extends.is_none());
    }

    #[test]
    fn raw_property_variants_construct() {
        // GIVEN: inline and reference properties
        let inline = RawPropertyInline {
            id: Uuid::now_v7(),
            name: "archived".to_owned(),
            required: false,
            array: false,
            spec: PropertySpec::Bool(BoolSpec::default()),
        };
        let reference = RawPropertyRef {
            ref_path: "status".to_owned(),
        };

        // WHEN: wrapping into enum variants
        let inline_variant = RawProperty::Inline(inline);
        let reference_variant = RawProperty::Ref(reference);

        // THEN: variants hold expected values
        assert!(matches!(inline_variant, RawProperty::Inline(_)));
        assert!(matches!(reference_variant, RawProperty::Ref(_)));
    }
}
