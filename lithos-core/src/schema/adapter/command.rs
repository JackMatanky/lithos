//! Redb-backed implementation of the [`crate::schema::ports::Command`] trait.

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        adapter::stored::to_stored,
        aggregate::{Schema, SchemaId, Timestamp},
        bank::{BankVersion, PropertyBank},
        db_table::{PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
        ports::Command,
    },
};

/// Metadata bundle for persisting a schema.
///
/// This adapter-specific type carries the storage metadata needed to build
/// `StoredSchema`. It lives in the adapter layer and is never exposed to
/// the domain.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::adapter::command::SaveMetadata;
/// use lithos_core::schema::bank::BankVersion;
///
/// let metadata = SaveMetadata {
///     bank_version: BankVersion::initial(),
///     created_at: None,
///     modified_at: None,
/// };
/// let _ = metadata;
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SaveMetadata {
    /// Property bank version at time of resolution.
    pub bank_version: BankVersion,
    /// Filesystem birthtime (from `Metadata::created()`), if available.
    pub created_at: Option<Timestamp>,
    /// Filesystem mtime (from `Metadata::modified()`), if available.
    pub modified_at: Option<Timestamp>,
}

/// Redb-backed schema command adapter.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::adapter::command::CommandAdapter;
///
/// let db = todo!("Provide a Database instance");
/// let adapter = CommandAdapter::new(&db);
/// let _ = adapter;
/// ```
pub struct CommandAdapter<'db> {
    db: &'db Database,
}

impl<'db> CommandAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a command adapter for a database.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::adapter::command::CommandAdapter;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = CommandAdapter::new(&db);
    /// let _ = adapter;
    /// ```
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Save a batch of schemas with explicit storage metadata.
    ///
    /// This is an adapter-specific method that allows the caller to provide
    /// storage metadata (bank version, file timestamps). The application
    /// service uses this method to preserve filesystem times.
    ///
    /// Metadata must be provided for each schema in the same order.
    ///
    /// # Errors
    /// Returns a storage-specific error if saving fails.
    ///
    /// # Panics
    /// Panics if `schemas.len() != metadata.len()`.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::adapter::command::{CommandAdapter, SaveMetadata};
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = CommandAdapter::new(&db);
    /// let schemas = Vec::new();
    /// let metadata: Vec<SaveMetadata> = Vec::new();
    /// adapter.save_batch_with_metadata(&schemas, &metadata)?;
    /// # Ok::<_, lithos_core::db::DbError>(())
    /// ```
    #[inline]
    #[instrument(
        skip(self, schemas, metadata),
        fields(operation = "save_schema_batch_with_metadata", record_count = schemas.len())
    )]
    pub fn save_batch_with_metadata(
        &self,
        schemas: &[Schema],
        metadata: &[SaveMetadata],
    ) -> Result<(), DbError> {
        assert_eq!(
            schemas.len(),
            metadata.len(),
            "schemas and metadata must have the same length"
        );

        // Validate uniqueness
        let mut name_index = std::collections::HashMap::new();

        for schema in schemas {
            if name_index.insert(schema.name().clone(), schema.id()).is_some() {
                return Err(DbError::Transaction(format!(
                    "schema name already exists in batch: {}",
                    schema.name().as_str()
                )));
            }

            if let Some(existing) = self.db.get_owned::<SchemaId>(
                SCHEMA_ID_BY_NAME,
                schema.name().as_str(),
            )? && existing != schema.id()
            {
                return Err(DbError::Transaction(format!(
                    "schema name already exists: {}",
                    schema.name().as_str()
                )));
            }
        }

        // Atomic write
        self.db.batch_write(|batch| {
            for (schema, meta) in schemas.iter().zip(metadata.iter()) {
                let stored = to_stored(
                    schema,
                    meta.bank_version,
                    meta.created_at,
                    meta.modified_at,
                );
                let id_key = schema.id().into_uuid().to_string();
                batch.put(SCHEMA_BY_ID, id_key.as_str(), &stored)?;
                batch.put(
                    SCHEMA_ID_BY_NAME,
                    schema.name().as_str(),
                    &schema.id(),
                )?;
            }
            Ok(())
        })
    }
}

impl Command for CommandAdapter<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self, schemas),
        fields(operation = "save_schema_batch", record_count = schemas.len())
    )]
    fn save_batch(&self, schemas: &[Schema]) -> Result<(), Self::Error> {
        // Use default metadata for port trait implementation
        // (tests and simple use cases don't need file timestamps)
        let metadata: Vec<SaveMetadata> = schemas
            .iter()
            .map(|_| SaveMetadata {
                bank_version: BankVersion::initial(),
                created_at: None,
                modified_at: None,
            })
            .collect();

        self.save_batch_with_metadata(schemas, &metadata)
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "delete_schema", schema_id = %id.as_uuid())
    )]
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error> {
        use crate::schema::adapter::stored::StoredSchema;

        let id_uuid = id.into_uuid();
        let id_key = id_uuid.to_string();

        // Atomic delete: read + delete name index + delete schema in single tx
        self.db.read_write_unit_of_work(|tx| {
            if let Some(stored) =
                tx.get_owned::<StoredSchema>(SCHEMA_BY_ID, id_key.as_str())?
            {
                tx.delete(SCHEMA_ID_BY_NAME, stored.name.as_ref())?;
            }
            tx.delete(SCHEMA_BY_ID, id_key.as_str())?;
            Ok(())
        })
    }

    #[inline]
    #[instrument(
        skip(self, bank),
        fields(operation = "save_property_bank", bank_id = %bank.id().as_uuid())
    )]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error> {
        let id_uuid = bank.id().into_uuid();
        self.db.put_by_uuid(PROPERTY_BANK, id_uuid, bank)?;
        Ok(())
    }
}
