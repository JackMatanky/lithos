//! Schema repository trait and error types.

use crate::{
    fs::PathKey,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaRepositoryError,
        identifier::{SchemaId, SchemaName},
        inheritance::InheritanceGraph,
        property::PropertyName,
        views::{RawPropertyBankView, RawSchemaView},
    },
};

/// Segregated read interface for schema persistence.
pub trait ReadRepository {
    /// Find a schema by its unique identifier.
    ///
    /// Returns `None` if no schema exists with the given ID.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if the database read or
    /// deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::schema::repository::ReadRepository;
    /// use lithos_core::schema::storage::RedbRepository;
    /// use std::sync::Arc;
    ///
    /// # let store = Arc::new(lithos_core::db::Store::open_temp()?);
    /// let repo = RedbRepository::new(store);
    ///
    /// match repo.find_schema_by_id(schema_id)? {
    ///     Some(schema) => println!("Found: {}", schema.name()),
    ///     None => println!("Schema not found"),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaRepositoryError>;

    /// Find multiple schemas by ID in a single transaction.
    ///
    /// Returns a vector in the same order as the input IDs.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaRepositoryError>;

    /// Find multiple schemas by ID, skipping missing entries.
    ///
    /// Returns only found schemas in encounter order.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, SchemaRepositoryError>;

    /// List all persisted schema aggregates.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn list_schemas(&self) -> Result<Vec<Schema>, SchemaRepositoryError>;

    /// Find schemas using any of the provided property names.
    ///
    /// Returns a mapping from schema ID to the matching property names for
    /// that schema.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<
        std::collections::HashMap<SchemaId, Vec<PropertyName>>,
        SchemaRepositoryError,
    >;

    /// Get a raw schema view by schema ID.
    ///
    /// Returns `None` if no view exists for the given ID.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, SchemaRepositoryError>;

    /// Find a raw schema view by schema file path.
    ///
    /// Performs cross-table lookup: path in `SCHEMA_ID_BY_PATH`, then the view
    /// in `RAW_SCHEMA_VIEWS`.
    ///
    /// Returns `None` if path or view is missing.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::schema::repository::ReadRepository;
    /// use lithos_core::schema::storage::RedbRepository;
    /// use lithos_core::fs::PathKey;
    /// use std::sync::Arc;
    ///
    /// # let store = Arc::new(lithos_core::db::Store::open_temp()?);
    /// let repo = RedbRepository::new(store);
    /// let path = PathKey::try_new("schemas/note.json")?;
    ///
    /// // Cross-table lookup: path → ID → view
    /// if let Some(view) = repo.find_raw_schema_view_by_path(&path)? {
    ///     println!("Found view with version: {}", view.version());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn find_raw_schema_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawSchemaView>, SchemaRepositoryError>;

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
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaRepositoryError>;

    /// Get the Property Bank singleton.
    ///
    /// Returns `None` if the Property Bank has not been saved.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaRepositoryError>;

    /// Get the topological inheritance graph singleton.
    ///
    /// Returns `None` when no graph has been persisted.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, SchemaRepositoryError>;

    /// Get the raw property bank view by path.
    ///
    /// Returns `None` if no view exists for the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn get_raw_property_bank_view(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawPropertyBankView>, SchemaRepositoryError>;

    /// Find a schema ID by its name.
    ///
    /// Returns `None` if no schema with the given name exists.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaRepositoryError>;

    /// Find a schema ID by its path.
    ///
    /// Returns `None` if no schema exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<SchemaId>, SchemaRepositoryError>;

    /// Find multiple schema IDs by their paths in a single transaction.
    ///
    /// Returns a vector in the same order as the input paths.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn find_schema_ids_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<SchemaId>>, SchemaRepositoryError>;

    /// List all schema name to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<crate::schema::index::NameIdPairs, SchemaRepositoryError>;

    /// List all schema path to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<crate::schema::index::PathIdPairs, SchemaRepositoryError>;

    /// Get unified index combining name, path, and ID lookups.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database read or deserialization
    /// fails.
    fn get_schema_index(
        &self,
    ) -> Result<crate::schema::index::SchemaIndex, SchemaRepositoryError>;
}

/// Segregated write interface for schema persistence.
pub trait WriteRepository {
    /// Persist a schema aggregate to the store.
    ///
    /// Atomically writes the schema and updates the name index. If the write
    /// fails, no partial state is visible to readers.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::schema::repository::{ReadRepository, WriteRepository};
    /// use lithos_core::schema::storage::RedbRepository;
    /// use std::sync::Arc;
    ///
    /// # let store = Arc::new(lithos_core::db::Store::open_temp()?);
    /// let repo = RedbRepository::new(store);
    ///
    /// // Save schema and verify index was updated atomically
    /// repo.save_schema(&schema)?;
    /// assert_eq!(
    ///     repo.find_schema_id_by_name(schema.name())?,
    ///     Some(schema.id())
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn save_schema(&self, schema: &Schema)
    -> Result<(), SchemaRepositoryError>;

    /// Save multiple schemas in a single transaction.
    ///
    /// If any schema fails to serialize, the entire batch rolls back.
    /// All-or-nothing atomicity: either all schemas are saved or none are.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::schema::repository::{ReadRepository, WriteRepository};
    /// use lithos_core::schema::storage::RedbRepository;
    /// use std::sync::Arc;
    ///
    /// # let store = Arc::new(lithos_core::db::Store::open_temp()?);
    /// let repo = RedbRepository::new(store);
    ///
    /// // Atomically save multiple schemas
    /// let schemas = vec![schema1, schema2, schema3];
    /// repo.save_many_schemas(&schemas)?;
    ///
    /// // All schemas are now persisted
    /// assert_eq!(repo.list_schemas()?.len(), 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaRepositoryError>;

    /// Save the Property Bank singleton.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaRepositoryError>;

    /// Save the raw property bank view for a given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    fn save_raw_property_bank_view(
        &self,
        path: &PathKey,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaRepositoryError>;

    /// Save a raw schema view for a schema ID.
    ///
    /// This operation updates both view storage and path index atomically.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaRepositoryError>;

    /// Save the topological inheritance graph singleton.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if serialization or database write
    /// fails.
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), SchemaRepositoryError>;

    /// Delete a schema aggregate and all related indexes/views.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaRepositoryError`] if database write fails.
    fn delete_schema(&self, id: SchemaId) -> Result<(), SchemaRepositoryError>;
}

/// Unified interface for schema persistence and retrieval.
///
/// This trait extends both [`ReadRepository`] and [`WriteRepository`] to
/// provide a complete interface for schema storage operations. It is
/// automatically implemented via blanket impl for any type implementing
/// both read and write traits.
///
/// # When to Use
///
/// - **Use `Repository`** when you need both read and write capabilities (e.g.,
///   orchestration logic like schema discovery processors).
/// - **Use [`ReadRepository`]** when only reads are required (e.g., query
///   handlers, read-only views).
/// - **Use [`WriteRepository`]** when only writes are required (rare; most
///   write operations need reads for validation).
///
/// # Blanket Implementation
///
/// ```rust,ignore
/// impl<T> Repository for T
/// where
///     T: ReadRepository + WriteRepository
/// {}
/// ```
///
/// This means [`RedbRepository`] and `InMemoryRepository` automatically
/// implement `Repository` since they implement both segregated traits.
///
/// # Example
///
/// ```rust,ignore
/// use lithos_core::schema::repository::Repository;
/// use lithos_core::schema::storage::RedbRepository;
///
/// fn process_schemas<R: Repository>(repo: &R) {
///     // Can use both read and write methods
///     let schemas = repo.list_schemas().unwrap();
///     // ... process ...
///     repo.save_schema(&updated_schema).unwrap();
/// }
/// ```
///
/// [`RedbRepository`]: crate::schema::storage::RedbRepository
pub trait Repository: ReadRepository + WriteRepository {}
