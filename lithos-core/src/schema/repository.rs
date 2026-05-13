//! Schema repository trait and error types.

use crate::{
    db::DbError,
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        identifier::{SchemaId, SchemaName},
        views::{RawPropertyBankView, RawSchemaView},
    },
};

/// Error type for schema storage operations in the v2 seam.
///
/// This type wraps the core [`DbError`] to provide a schema-specific error
/// context while allowing callers to branch on stable error kinds.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct SchemaStorageV2Error(#[from] DbTxError);

impl From<DbError> for SchemaStorageV2Error {
    #[inline]
    fn from(err: DbError) -> Self {
        Self(DbTxError::from(err))
    }
}

/// Internal error type for database transactions.
///
/// This is wrapped by [`SchemaStorageV2Error`] to maintain a clean public API.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DbTxError(#[from] DbError);

/// Segregated read interface for schema persistence.
pub trait SchemaReadRepository {
    /// Find a schema by its unique identifier.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if the database read or deserialization
    /// fails.
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaStorageV2Error>;

    /// Find multiple schemas by ID in a single transaction.
    ///
    /// Returns a vector in the same order as the input IDs.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaStorageV2Error>;

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
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaStorageV2Error>;

    /// Get the Property Bank singleton.
    ///
    /// Returns `None` if the Property Bank has not been saved.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaStorageV2Error>;

    /// Get the raw property bank view by path.
    ///
    /// Returns `None` if no view exists for the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, SchemaStorageV2Error>;

    /// Find a schema ID by its name.
    ///
    /// Returns `None` if no schema with the given name exists.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error>;

    /// Find a schema ID by its path.
    ///
    /// Returns `None` if no schema exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn find_schema_id_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error>;

    /// Find multiple schema IDs by their paths in a single transaction.
    ///
    /// Returns a vector in the same order as the input paths.
    /// Missing schemas return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<SchemaId>>, SchemaStorageV2Error>;

    /// List all schema name to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<crate::schema::index::NameIdPairs, SchemaStorageV2Error>;

    /// List all schema path to ID mappings.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<crate::schema::index::PathIdPairs, SchemaStorageV2Error>;

    /// Get unified index combining name, path, and ID lookups.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if database read or deserialization
    /// fails.
    fn get_schema_index(
        &self,
    ) -> Result<crate::schema::index::SchemaIndex, SchemaStorageV2Error>;
}

/// Segregated write interface for schema persistence.
pub trait SchemaWriteRepository {
    /// Persist a schema aggregate to the store.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if serialization or database write
    /// fails.
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageV2Error>;

    /// Save multiple schemas in a single transaction.
    ///
    /// If any schema fails to serialize, the entire batch rolls back.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if serialization or database write
    /// fails.
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaStorageV2Error>;

    /// Save the Property Bank singleton.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if serialization or database write
    /// fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaStorageV2Error>;

    /// Save the raw property bank view for a given path.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageV2Error`] if serialization or database write
    /// fails.
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaStorageV2Error>;
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
