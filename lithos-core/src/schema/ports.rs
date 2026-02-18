//! Schema bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Schema
//! aggregate and PropertyBank registry.

use super::{
    aggregate::{ResolutionMetadata, Schema, SchemaId, SchemaName},
    bank::PropertyBank,
};

/// Command port for Schema write operations.
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if deletion fails.
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error>;

    /// Save a batch of schemas and resolution metadata to persistence.
    ///
    /// All saves are atomic within a single write transaction.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    fn save_batch(
        &self,
        schemas: &[(Schema, ResolutionMetadata)],
    ) -> Result<(), Self::Error>;

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
    /// Archived schema type for zero-copy reads.
    type Archived<'archived>;
    /// Storage error type for query operations.
    type Error: std::error::Error;

    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;

    /// Find resolution metadata by schema ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn find_metadata_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<ResolutionMetadata>, Self::Error>;

    /// Find the `PropertyBank` registry.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>;

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn list(&self) -> Result<Vec<Schema>, Self::Error>;

    /// List all resolution metadata entries.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn list_metadata(&self) -> Result<Vec<ResolutionMetadata>, Self::Error>;

    /// Lookup a schema ID by name.
    ///
    /// # Errors
    /// Returns a storage-specific error if lookup fails.
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error>;

    /// Access a schema by ID as archived data.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn with_archived_by_id<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R;

    /// Access a schema by name as archived data.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    fn with_archived_by_name<F, R>(
        &self,
        name: &SchemaName,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R;
}
