//! Unified repository trait for schema persistence.
//!
//! This module defines the `Repository` trait that combines read and write
//! operations for the schema domain, replacing the previous CQRS-style
//! Command/Query split.
//!
//! ## Architecture
//!
//! Following the unified Repository pattern from the architecture guide:
//! - Single trait combining reads and writes
//! - Zero-copy access via `with_archived()` methods
//! - Concrete implementations: `RedbRepository`, `InMemoryRepository`,
//!   `FakeRepository`
//!
//! ## Usage
//!
//! ```ignore
//! use lithos_core::schema::Repository;
//!
//! fn process_schemas<R: Repository>(repo: &R) -> Result<(), R::Error> {
//!     // Read operations
//!     let schemas = repo.list_schemas()?;
//!
//!     // Write operations
//!     repo.save_schemas(&schemas)?;
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    bank::PropertyBank,
    property::{Property, PropertyId, PropertyName},
};
use crate::db::BatchReader;

/// A schema name-to-ID pair.
pub type NameIdPair = (SchemaName, SchemaId);

/// Inheritance relationship: (`child_id`, `parent_id`, `excludes`).
pub type InheritanceRelation = (SchemaId, Option<SchemaId>, Vec<Box<str>>);

/// Inheritance children map: `parent_id` → Vec<(`child_id`, `excludes`)>.
pub type InheritanceChildren =
    HashMap<SchemaId, Vec<(SchemaId, Vec<Box<str>>)>>;

/// Schema-to-properties usage map: `schema_id` → Vec<`property_name`>.
///
/// Used by `find_schemas_using_properties()` to return which schemas use which
/// properties.
pub type SchemaPropertyUsage = HashMap<SchemaId, Vec<PropertyName>>;

/// Unified repository trait for schema domain persistence.
///
/// Combines read and write operations in a single trait, following the
/// unified Repository pattern from the architecture guide.
///
/// # Type Parameters
///
/// - `Error`: Storage-specific error type
///
/// # Naming Conventions
///
/// Following the naming taxonomy from `docs/refs/rust/naming-taxonomy.md`:
/// - **find_***: Optional reads (returns `Option<T>`)
/// - **get_***: Required singleton reads
/// - **list_***: Multiple item reads (returns `Vec<T>`)
/// - **is_***: Boolean checks
/// - **save**, **delete**: Write operations
/// - **with_***: Zero-copy closure-based access
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::Repository;
///
/// fn example<R: Repository>(repo: &R) -> Result<(), R::Error> {
///     // Find optional schema
///     if let Some(schema) = repo.find_schema_by_id(id)? {
///         println!("Found: {}", schema.name);
///     }
///
///     // List all schemas
///     let schemas = repo.list_schemas()?;
///
///     // Save schemas
///     repo.save_schemas(&schemas)?;
///
///     Ok(())
/// }
/// ```
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped by category for better maintainability"
)]
pub trait Repository: Send + Sync {
    /// Storage-specific error type.
    type Error: std::error::Error + Send + Sync;

    // ========================================================================
    // Schema Read Operations
    // ========================================================================

    /// Finds a schema by ID.
    ///
    /// Returns `None` if the schema does not exist.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, Self::Error>;

    /// Finds a schema ID by name.
    ///
    /// Returns `None` if no schema with the given name exists.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error>;

    /// Finds multiple schemas by IDs.
    ///
    /// Returns only the schemas that exist. Missing schemas are silently
    /// skipped.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, Self::Error>;

    /// Lists all schemas.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_schemas(&self) -> Result<Vec<Schema>, Self::Error>;

    /// Lists schema name-to-ID pairs.
    ///
    /// Useful for building name lookup tables without loading full schema data.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_schema_name_id_pairs(&self)
    -> Result<Vec<NameIdPair>, Self::Error>;

    /// Lists inheritance children for all parent schemas.
    ///
    /// Returns a map of `parent_id` → Vec<(`child_id`, `excludes`)>.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_inheritance_children(
        &self,
    ) -> Result<InheritanceChildren, Self::Error>;

    /// Lists all descendant schema IDs for a given parent.
    ///
    /// Returns transitive children (children, grandchildren, etc.).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_descendant_ids(
        &self,
        parent_id: SchemaId,
    ) -> Result<Vec<SchemaId>, Self::Error>;

    // ========================================================================
    // Property Bank Read Operations
    // ========================================================================

    /// Gets the property bank singleton.
    ///
    /// Returns `None` if the property bank has not been initialized.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>;

    /// Finds a property by ID in the property bank.
    ///
    /// Returns `None` if the property does not exist.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_property_by_id(
        &self,
        id: PropertyId,
    ) -> Result<Option<Property>, Self::Error>;

    /// Finds schemas that use any of the given property names.
    ///
    /// Returns a map of `schema_id` → Vec<`property_name`> for schemas
    /// that reference at least one of the given properties.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<SchemaPropertyUsage, Self::Error>;

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Saves multiple schemas atomically.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_schemas(&self, schemas: &[Schema]) -> Result<(), Self::Error>;

    /// Saves inheritance relationships atomically.
    ///
    /// Each relationship is a tuple of (`child_id`, `parent_id`, `excludes`).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_inheritance_relations(
        &self,
        relations: &[InheritanceRelation],
    ) -> Result<(), Self::Error>;

    /// Saves the property bank singleton.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error>;

    /// Deletes a schema by ID.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the deletion fails.
    fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error>;

    // ========================================================================
    // Batch Operations (for complex multi-table queries)
    // ========================================================================

    /// Provides access to a batch reader for complex multi-table queries.
    ///
    /// This is a lower-level API for operations that need to read from
    /// multiple tables in a single transaction.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn with_batch_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>;
}
