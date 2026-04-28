//! Schema aggregate and identifier types.
//!
//! Core aggregate and value objects for the schema domain:
//! - [`Schema`] - Main schema aggregate (previously `StoredSchema`)
//!
//! ## Schema Aggregate
//!
//! The `Schema` type is the main domain aggregate for the schema system.
//! It represents a fully resolved schema with all properties merged from
//! parent schemas according to inheritance rules.
//!
//! ## Architecture Notes
//!
//! This module follows the unified Repository pattern:
//! - Files are the source of truth
//! - Domain types are used as storage shape (no separate view types unless
//!   profiling shows need)
//! - `recorded_at` field is private (ingestion metadata, not exposed in public
//!   API)

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    identifier::{SchemaId, SchemaName},
    property::{Property, PropertyId, PropertyMap, PropertyName},
};

// ============================================================================
// Schema Aggregate
// ============================================================================

/// Main schema aggregate.
///
/// Represents a fully resolved schema with all properties merged from parent
/// schemas. This is the primary domain type used throughout the schema system.
///
/// ## Fields
///
/// - `id`: Unique schema identifier
/// - `name`: Validated schema name
/// - `parents`: Parent schema IDs for inheritance
/// - `children`: Child schema IDs (for fast inheritance traversal)
/// - `properties`: Resolved properties after inheritance merge
/// - `recorded_at`: Ingestion timestamp (**private** - not part of public API)
///
/// ## Storage
///
/// Persisted to the `schema_by_id` table using `rkyv` serialization.
/// The domain type serves as the storage shape.
///
/// # Examples
///
/// ```
/// use lithos_core::schema::{
///     aggregate::Schema,
///     identifier::{SchemaId, SchemaName},
///     property::PropertyMap,
/// };
///
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("project-note")?;
/// let schema = Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());
/// assert_eq!(schema.id(), &id);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// Schema identity.
    id: SchemaId,
    /// Schema name.
    name: SchemaName,
    /// Parent schema IDs, for inheritance.
    parents: Vec<SchemaId>,
    /// Child schema IDs (for fast inheritance traversal).
    ///
    /// Stores IDs only. Full relationship metadata (extends/excludes) is
    /// managed via inheritance views in the repository layer.
    children: Vec<SchemaId>,
    /// Resolved properties (`PropertyMap` for O(1) lookup by name).
    properties: PropertyMap,
    /// Ingestion timestamp (private - not exposed in public API).
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl Schema {
    /// Creates a new `Schema` with current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::Schema,
    ///     identifier::{SchemaId, SchemaName},
    ///     property::PropertyMap,
    /// };
    ///
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("note")?;
    /// let schema = Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());
    /// assert_eq!(schema.id(), &id);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: SchemaId,
        name: SchemaName,
        parents: Vec<SchemaId>,
        children: Vec<SchemaId>,
        properties: PropertyMap,
    ) -> Self {
        Self {
            id,
            name,
            parents,
            children,
            properties,
            recorded_at: SystemTime::now(),
        }
    }

    /// Returns the schema ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &SchemaId {
        &self.id
    }

    /// Returns the schema name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the parent schema IDs.
    #[inline]
    #[must_use]
    pub fn parents(&self) -> &[SchemaId] {
        &self.parents
    }

    /// Returns the child schema IDs.
    ///
    /// This provides fast access to direct children for inheritance traversal.
    /// Full relationship metadata (extends/excludes) is managed separately
    /// via inheritance views in the repository layer.
    #[inline]
    #[must_use]
    pub fn children(&self) -> &[SchemaId] {
        &self.children
    }

    /// Returns the resolved properties.
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    /// Finds a property by name (O(1) lookup).
    #[inline]
    #[must_use]
    pub fn find_property_by_name(
        &self,
        name: &PropertyName,
    ) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Finds a property by ID (O(n) - iterates all properties).
    #[inline]
    #[must_use]
    pub fn find_property(&self, id: &PropertyId) -> Option<&Property> {
        self.properties.values().find(|p| p.id() == *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Schema Tests ==========

    #[test]
    fn schema_new_creates_with_empty_properties() {
        let id = SchemaId::new();
        let name = SchemaName::try_new("test").unwrap();
        let schema = Schema::new(
            id,
            name.clone(),
            Vec::new(),
            vec![],
            PropertyMap::new(),
        );

        assert_eq!(schema.id(), &id);
        assert_eq!(schema.name(), &name);
        assert!(schema.parents().is_empty());
        assert!(schema.properties().is_empty());
    }

    #[test]
    fn schema_accessors_work() {
        let id = SchemaId::new();
        let parent_id = SchemaId::new();
        let name = SchemaName::try_new("child-schema").unwrap();
        let schema = Schema::new(
            id,
            name.clone(),
            vec![parent_id],
            vec![],
            PropertyMap::new(),
        );

        assert_eq!(schema.id(), &id);
        assert_eq!(schema.name(), &name);
        assert_eq!(schema.parents(), &[parent_id]);
    }
}
