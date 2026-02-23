//! Schema bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Schema
//! aggregate and PropertyBank registry.

use super::{
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
    bank::{BankVersion, PropertyBank},
};
use crate::db::BatchReader;

/// A schema name-to-ID pair returned by [`Query::list_name_id_pairs`].
pub type NameIdPair = (SchemaName, SchemaId);

/// Command port for Schema write operations.
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if deletion fails.
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error>;

    /// Save a batch of schemas to persistence.
    ///
    /// All schemas are saved atomically within a single write transaction.
    /// This is the simple port trait method that adapters implement with
    /// default metadata.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    fn save_batch(&self, schemas: &[Schema]) -> Result<(), Self::Error>;

    /// Save the `PropertyBank` to persistence.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error>;
}

/// Query port for Schema read operations.
pub trait Query: Send + Sync {
    /// Storage error type for query operations.
    type Error: std::error::Error;

    /// Execute multiple read operations within a single transaction.
    ///
    /// This amortizes transaction creation cost across multiple reads,
    /// improving performance for batch operations.
    ///
    /// # Errors
    /// Returns a storage-specific error if the transaction fails.
    fn batch_read<R, F>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>;

    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;

    /// Find the `PropertyBank` registry.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>;

    /// Returns `true` if the stored bank version differs from
    /// `current_version`.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn is_bank_stale(
        &self,
        current_version: BankVersion,
    ) -> Result<bool, Self::Error>;

    /// Returns `true` if the stored schema for `id` is stale.
    ///
    /// A schema is considered stale when:
    /// - No stored record exists for `id`, or
    /// - `stored.bank_version != current_bank_version`, or
    /// - `stored.modified_at < file_mtime` (file changed since last ingestion).
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn is_schema_stale(
        &self,
        id: SchemaId,
        file_mtime: Option<Timestamp>,
        current_bank_version: BankVersion,
    ) -> Result<bool, Self::Error>;

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn list(&self) -> Result<Vec<Schema>, Self::Error>;

    /// List all schema name-to-ID pairs.
    ///
    /// This is a bulk operation that scans the entire name index in one pass.
    /// Use this instead of `lookup_id_by_name` when preloading all mappings.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn list_name_id_pairs(&self) -> Result<Vec<NameIdPair>, Self::Error>;

    /// Lookup a schema ID by name.
    ///
    /// # Errors
    /// Returns a storage-specific error if lookup fails.
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error>;
}
