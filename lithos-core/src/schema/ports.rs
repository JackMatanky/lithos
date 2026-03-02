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
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::ports::Command;
///
/// struct MyCommand;
///
/// impl Command for MyCommand {
///     type Error = std::convert::Infallible;
///
///     fn delete(
///         &self,
///         _id: lithos_core::schema::aggregate::SchemaId,
///     ) -> Result<(), Self::Error> {
///         Ok(())
///     }
///
///     fn save_batch(
///         &self,
///         _schemas: &[lithos_core::schema::aggregate::Schema],
///     ) -> Result<(), Self::Error> {
///         Ok(())
///     }
///
///     fn save_property_bank(
///         &self,
///         _bank: &lithos_core::schema::bank::PropertyBank,
///     ) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
/// ```
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if deletion fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Command;
    /// # let command = todo!("Provide a Command implementation");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// command.delete(id)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error>;

    /// Save a batch of schemas to persistence.
    ///
    /// All schemas are saved atomically within a single write transaction.
    /// This is the simple port trait method that adapters implement with
    /// default metadata.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Command;
    /// # let command = todo!("Provide a Command implementation");
    /// # let schemas: Vec<lithos_core::schema::aggregate::Schema> = Vec::new();
    /// command.save_batch(&schemas)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn save_batch(&self, schemas: &[Schema]) -> Result<(), Self::Error>;

    /// Save the `PropertyBank` to persistence.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Command;
    /// # let command = todo!("Provide a Command implementation");
    /// # let bank = lithos_core::schema::bank::PropertyBank::new();
    /// command.save_property_bank(&bank)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error>;
}

/// Query port for Schema read operations.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::ports::Query;
///
/// struct MyQuery;
///
/// impl Query for MyQuery {
///     type Error = std::convert::Infallible;
///
///     fn batch_read<R, F>(&self, f: F) -> Result<R, Self::Error>
///     where
///         F: FnOnce(&lithos_core::db::BatchReader) -> Result<R, Self::Error>,
///     {
///         let reader = todo!("Provide a BatchReader instance");
///         f(&reader)
///     }
///
///     fn find_by_id(
///         &self,
///         _id: lithos_core::schema::aggregate::SchemaId,
///     ) -> Result<Option<lithos_core::schema::aggregate::Schema>, Self::Error> {
///         Ok(None)
///     }
///
///     fn get_property_bank(
///         &self,
///     ) -> Result<Option<lithos_core::schema::bank::PropertyBank>, Self::Error> {
///         Ok(None)
///     }
///
///     fn is_bank_stale(
///         &self,
///         _version: lithos_core::schema::bank::BankVersion,
///     ) -> Result<bool, Self::Error> {
///         Ok(false)
///     }
///
///     fn is_schema_stale(
///         &self,
///         _id: lithos_core::schema::aggregate::SchemaId,
///         _created_at: Option<lithos_core::schema::aggregate::Timestamp>,
///         _modified_at: Option<lithos_core::schema::aggregate::Timestamp>,
///         _bank_version: lithos_core::schema::bank::BankVersion,
///     ) -> Result<bool, Self::Error> {
///         Ok(false)
///     }
///
///     fn list(
///         &self,
///     ) -> Result<Vec<lithos_core::schema::aggregate::Schema>, Self::Error> {
///         Ok(Vec::new())
///     }
///
///     fn list_name_id_pairs(
///         &self,
///     ) -> Result<Vec<lithos_core::schema::ports::NameIdPair>, Self::Error> {
///         Ok(Vec::new())
///     }
///
///     fn lookup_id_by_name(
///         &self,
///         _name: &lithos_core::schema::aggregate::SchemaName,
///     ) -> Result<Option<lithos_core::schema::aggregate::SchemaId>, Self::Error> {
///         Ok(None)
///     }
/// }
/// ```
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// # query.batch_read(|_reader| Ok::<_, Box<dyn std::error::Error>>(()))?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn batch_read<R, F>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>;

    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// let _ = query.find_by_id(id)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;

    /// Get the singleton `PropertyBank` registry.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// let _ = query.get_property_bank()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>;

    /// Returns `true` if the stored bank version differs from `version`.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// # let version = lithos_core::schema::bank::BankVersion::initial();
    /// let _ = query.is_bank_stale(version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn is_bank_stale(&self, version: BankVersion) -> Result<bool, Self::Error>;

    /// Returns `true` if the stored schema for `id` is stale.
    ///
    /// A schema is considered stale when:
    /// - No stored record exists for `id`, or
    /// - `stored.bank_version != bank_version`, or
    /// - `stored.created_at != created_at` when both are present, or
    /// - `stored.modified_at < modified_at` (file changed since last
    ///   ingestion).
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// # let bank_version = lithos_core::schema::bank::BankVersion::initial();
    /// let _ = query.is_schema_stale(id, None, None, bank_version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn is_schema_stale(
        &self,
        id: SchemaId,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
        bank_version: BankVersion,
    ) -> Result<bool, Self::Error>;

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// let _ = query.list()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn list(&self) -> Result<Vec<Schema>, Self::Error>;

    /// List all schema name-to-ID pairs.
    ///
    /// This is a bulk operation that scans the entire name index in one pass.
    /// Use this instead of `lookup_id_by_name` when preloading all mappings.
    ///
    /// # Errors
    /// Returns a storage-specific error if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// let _ = query.list_name_id_pairs()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn list_name_id_pairs(&self) -> Result<Vec<NameIdPair>, Self::Error>;

    /// Lookup a schema ID by name.
    ///
    /// # Errors
    /// Returns a storage-specific error if lookup fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::ports::Query;
    /// # let query = todo!("Provide a Query implementation");
    /// # let name = lithos_core::schema::aggregate::SchemaName::new("task")?;
    /// let _ = query.lookup_id_by_name(&name)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error>;
}
