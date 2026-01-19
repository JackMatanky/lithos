//! Raw schema and property input definitions.

#![allow(
    clippy::module_name_repetitions,
    reason = "RawSchema and RawProperty follow naming conventions for input types"
)]

use std::collections::HashSet;

use uuid::Uuid;

use super::{
    aggregate::SchemaName, property::PropertyName, property_spec::PropertySpec,
};

/// Raw schema definition (Input).
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
