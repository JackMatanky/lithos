//! Schema repository trait and error types.

use crate::{
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaStorageError,
        identifier::{SchemaId, SchemaName},
        inheritance::InheritanceGraph,
        property::PropertyName,
        views::{RawPropertyBankView, RawSchemaView},
    },
};

/// Segregated read interface for schema persistence.
pub trait SchemaReadRepository {
    /// Find a schema by its unique identifier.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if the database read or deserialization
    /// fails.
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaStorageError>;

    /// Find multiple schemas by ID in a single transaction.
    ///
    /// Returns a vector in the same order as the input IDs.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaStorageError>;

    /// Find multiple schemas by ID, skipping missing entries.
    ///
    /// Returns only found schemas in encounter order.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, SchemaStorageError>;

    /// List all persisted schema aggregates.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn list_schemas(&self) -> Result<Vec<Schema>, SchemaStorageError>;

    /// Find schemas using any of the provided property names.
    ///
    /// Returns a mapping from schema ID to the matching property names for
    /// that schema.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<
        std::collections::HashMap<SchemaId, Vec<PropertyName>>,
        SchemaStorageError,
    >;

    /// Get a raw schema view by schema ID.
    ///
    /// Returns `None` if no view exists for the given ID.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, SchemaStorageError>;

    /// Find a raw schema view by schema file path.
    ///
    /// Performs cross-table lookup: path in `SCHEMA_ID_BY_PATH`, then the view
    /// in `RAW_SCHEMA_VIEWS`.
    ///
    /// Returns `None` if path or view is missing.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_raw_schema_view_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, SchemaStorageError>;

    /// Find raw schema views by file paths in a single transaction.
    ///
    /// Performs cross-table batch read: lookups paths in `SCHEMA_ID_BY_PATH`,
    /// then fetches corresponding views from `RAW_SCHEMA_VIEWS`.
    ///
    /// Returns a vector in the same order as the input paths.
    /// Missing views return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaStorageError>;

    /// Get the Property Bank singleton.
    ///
    /// Returns `None` if the Property Bank has not been saved.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaStorageError>;

    /// Get the topological inheritance graph singleton.
    ///
    /// Returns `None` when no graph has been persisted.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, SchemaStorageError>;

    /// Get the raw property bank view by path.
    ///
    /// Returns `None` if no view exists for the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, SchemaStorageError>;

    /// Find a schema ID by its name.
    ///
    /// Returns `None` if no schema with the given name exists.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaStorageError>;

    /// Find a schema ID by its path.
    ///
    /// Returns `None` if no schema exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<SchemaId>, SchemaStorageError>;

    /// Find multiple schema IDs by their paths in a single transaction.
    ///
    /// Returns a vector in the same order as the input paths.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<SchemaId>>, SchemaStorageError>;

    /// List all schema name to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<crate::schema::index::NameIdPairs, SchemaStorageError>;

    /// List all schema path to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<crate::schema::index::PathIdPairs, SchemaStorageError>;

    /// Get unified index combining name, path, and ID lookups.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database read or deserialization
    /// fails.
    fn get_schema_index(
        &self,
    ) -> Result<crate::schema::index::SchemaIndex, SchemaStorageError>;
}

/// Segregated write interface for schema persistence.
pub trait SchemaWriteRepository {
    /// Persist a schema aggregate to the store.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageError>;

    /// Save multiple schemas in a single transaction.
    ///
    /// If any schema fails to serialize, the entire batch rolls back.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaStorageError>;

    /// Save the Property Bank singleton.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaStorageError>;

    /// Save the raw property bank view for a given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaStorageError>;

    /// Save a raw schema view for a schema ID.
    ///
    /// This operation updates both view storage and path index atomically.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaStorageError>;

    /// Save the topological inheritance graph singleton.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if serialization or database write
    /// fails.
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), SchemaStorageError>;

    /// Delete a schema aggregate and all related indexes/views.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if database write fails.
    fn delete_schema(&self, id: SchemaId) -> Result<(), SchemaStorageError>;
}

/// Unified interface for schema persistence and retrieval.
///
/// This trait extends both [`SchemaReadRepository`] and
/// [`SchemaWriteRepository`] to provide a complete interface for schema storage
/// operations.
pub trait SchemaRepository:
    SchemaReadRepository + SchemaWriteRepository
{
}
