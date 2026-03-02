//! Redb-backed implementation of the [`crate::schema::ports::Command`] trait.
//!
//! Property bank persistence writes:
//! - `bank_metadata` for version/timestamps
//! - `bank_property_by_id` and `bank_property_by_name` for versioned rows

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        adapter::stored::{
            StoredBankProperty, StoredMetadata, StoredProperty, StoredSchema,
        },
        aggregate::{Schema, SchemaId, Timestamp},
        bank::{BankVersion, PropertyBank},
        db_table::{
            BANK_METADATA, BANK_PROPERTY_BY_ID, BANK_PROPERTY_BY_NAME,
            PROPERTY_BANK_KEY, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME,
            SCHEMA_METADATA,
        },
        ports::Command,
        property::{Multiplicity, Optionality},
    },
};

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
    /// use lithos_core::schema::adapter::command::CommandAdapter;
    /// use lithos_core::schema::adapter::stored::StoredMetadata;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = CommandAdapter::new(&db);
    /// let schemas = Vec::new();
    /// let metadata: Vec<StoredMetadata> = Vec::new();
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
        metadata: &[StoredMetadata],
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
                let stored = StoredSchema::from_schema(schema);
                let id_key = schema.id().into_uuid().to_string();
                batch.put(SCHEMA_BY_ID, id_key.as_str(), &stored)?;
                batch.put(
                    SCHEMA_ID_BY_NAME,
                    schema.name().as_str(),
                    &schema.id(),
                )?;
                batch.put(SCHEMA_METADATA, id_key.as_str(), meta)?;
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
        let metadata: Vec<StoredMetadata> = schemas
            .iter()
            .map(|_| StoredMetadata::new(BankVersion::initial(), None, None))
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

        // Atomic delete: read + delete name index + delete schema + delete
        // metadata in single tx
        self.db.read_write_unit_of_work(|tx| {
            if let Some(stored) =
                tx.get_owned::<StoredSchema>(SCHEMA_BY_ID, id_key.as_str())?
            {
                tx.delete(SCHEMA_ID_BY_NAME, stored.name.as_ref())?;
            }
            tx.delete(SCHEMA_BY_ID, id_key.as_str())?;
            tx.delete(SCHEMA_METADATA, id_key.as_str())?;
            Ok(())
        })
    }

    #[inline]
    #[instrument(skip(self, bank), fields(operation = "save_property_bank"))]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error> {
        // Persist metadata plus versioned property rows with version retention.
        // Keeps last 3 versions to prevent unbounded disk growth.
        const VERSION_RETENTION_COUNT: u64 = 3;

        let bank_version = bank.version();
        let recorded_at = Timestamp::now();

        // Read current metadata to determine old versions to delete
        let previous_metadata = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?;

        // Determine which version to delete (keep last 3: current-2, current-1,
        // current)
        let version_to_delete = previous_metadata.and_then(|meta| {
            let current = meta.bank_version.as_u64();
            // Only delete if we have accumulated enough versions
            // Example: current=5, keep [3,4,5], delete 2
            (current >= VERSION_RETENTION_COUNT).then(|| {
                BankVersion::from_u64(
                    current
                        .saturating_sub(VERSION_RETENTION_COUNT)
                        .saturating_add(1),
                )
            })
        });

        // Collect keys to delete from both tables before write transaction
        let keys_to_delete = if let Some(old_version) = version_to_delete {
            let prefix = StoredBankProperty::prefix(old_version);
            let id_keys = self
                .db
                .batch_read(|reader| {
                    reader.scan_range::<StoredBankProperty>(
                        BANK_PROPERTY_BY_ID,
                        &prefix,
                    )
                })
                .unwrap_or_default();

            let name_keys = self
                .db
                .batch_read(|reader| {
                    reader.scan_range::<StoredBankProperty>(
                        BANK_PROPERTY_BY_NAME,
                        &prefix,
                    )
                })
                .unwrap_or_default();

            (
                id_keys.into_iter().map(|(k, _)| k).collect::<Vec<_>>(),
                name_keys.into_iter().map(|(k, _)| k).collect::<Vec<_>>(),
            )
        } else {
            (Vec::new(), Vec::new())
        };

        let metadata = StoredMetadata {
            bank_version,
            created_at: None,
            modified_at: None,
            recorded_at,
        };

        // Atomic write: delete old versions + write new metadata + write new
        // properties
        self.db.batch_write(|batch| {
            // Delete old versioned properties
            for id_key in &keys_to_delete.0 {
                batch.delete(BANK_PROPERTY_BY_ID, id_key)?;
            }
            for name_key in &keys_to_delete.1 {
                batch.delete(BANK_PROPERTY_BY_NAME, name_key)?;
            }

            // Write new metadata
            batch.put(BANK_METADATA, PROPERTY_BANK_KEY, &metadata)?;

            // Write new versioned properties
            for property in bank.all() {
                let stored_property = StoredProperty {
                    id: property.id(),
                    name: property.name().as_str().into(),
                    required: property.optionality() == Optionality::Required,
                    multi: property.multiplicity() == Multiplicity::Many,
                    spec: property.spec().clone(),
                };

                let stored = StoredBankProperty {
                    bank_version,
                    recorded_at,
                    property: stored_property,
                };

                let id_key = StoredBankProperty::key(
                    bank_version,
                    &property.id().to_string(),
                );
                let name_key = StoredBankProperty::key(
                    bank_version,
                    property.name().as_str(),
                );

                batch.put(BANK_PROPERTY_BY_ID, &id_key, &stored)?;
                batch.put(BANK_PROPERTY_BY_NAME, &name_key, &stored)?;
            }

            Ok(())
        })
    }
}
