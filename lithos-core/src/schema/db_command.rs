//! Redb-backed implementation of the [`crate::schema::ports::Command`] trait.
//!
//! Property bank persistence writes:
//! - `bank_metadata` for version/timestamps
//! - `bank_property_by_id` and `bank_property_by_name` for versioned rows

use std::{collections::HashMap, time::SystemTime};

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        aggregate::SchemaId,
        bank::{BankVersion, PropertyBank},
        db_table::{
            BANK_METADATA, BANK_PROPERTY_BY_ID, BANK_PROPERTY_BY_NAME,
            PROPERTY_BANK_KEY, RAW_PROPERTY_BANK_FILE, RAW_PROPERTY_BANK_KEY,
            RAW_SCHEMA_FILES, SCHEMA_BY_ID, SCHEMA_CHILDREN, SCHEMA_ID_BY_NAME,
            SCHEMA_METADATA, SCHEMA_PARENT,
        },
        ports::Command as CommandPort,
        property::{Multiplicity, Optionality, PropertyName},
        storage::{
            RawPropertyBankFile, RawSchemaFile, StoredBankProperty,
            StoredChildSchema, StoredMetadata, StoredParentSchema,
            StoredProperty, StoredPropertyBank, StoredSchema,
        },
    },
};

/// Redb-backed schema command adapter.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::db_command::Command;
///
/// let db = todo!("Provide a Database instance");
/// let adapter = Command::new(&db);
/// let _ = adapter;
/// ```
pub struct Command<'db> {
    db: &'db Database,
}

impl<'db> Command<'db> {
    #[inline]
    #[must_use]
    /// Create a command adapter for a database.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::db_command::Command;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = Command::new(&db);
    /// let _ = adapter;
    /// ```
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Save many schemas with explicit storage metadata.
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
    /// use lithos_core::schema::db_command::CommandAdapter;
    /// use lithos_core::schema::storage::StoredMetadata;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = CommandAdapter::new(&db);
    /// let schemas = Vec::new();
    /// let metadata: Vec<StoredMetadata> = Vec::new();
    /// adapter.save_many_with_metadata(&schemas, &metadata)?;
    /// # Ok::<_, lithos_core::db::DbError>(())
    /// ```
    #[inline]
    #[instrument(
        skip(self, schemas, metadata),
        fields(operation = "save_schema_many_with_metadata", record_count = schemas.len())
    )]
    pub fn save_many_with_metadata(
        &self,
        schemas: &[StoredSchema],
        metadata: &[StoredMetadata],
    ) -> Result<(), DbError> {
        if schemas.len() != metadata.len() {
            return Err(DbError::Transaction(format!(
                "schemas and metadata must have the same length: schemas={}, \
                 metadata={}",
                schemas.len(),
                metadata.len()
            )));
        }

        // Validate uniqueness
        let mut name_index = std::collections::HashMap::new();

        for stored in schemas {
            if name_index.insert(stored.name.clone(), stored.id).is_some() {
                return Err(DbError::Transaction(format!(
                    "schema name already exists in batch: {}",
                    stored.name.as_ref()
                )));
            }

            if let Some(existing) = self.db.get_owned::<SchemaId>(
                SCHEMA_ID_BY_NAME,
                stored.name.as_ref(),
            )? && existing != stored.id
            {
                return Err(DbError::Transaction(format!(
                    "schema name already exists: {}",
                    stored.name.as_ref()
                )));
            }
        }

        // Validate property references against PropertyBank
        if let Some(bank_metadata) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        {
            // Load PropertyBank to validate property references
            let prefix = StoredBankProperty::prefix(bank_metadata.bank_version);
            let entries = self.db.scan_range::<StoredBankProperty>(
                BANK_PROPERTY_BY_NAME,
                &prefix,
            )?;
            let properties: Vec<_> = entries
                .into_iter()
                .map(|(_, stored)| stored.property)
                .collect();

            let stored_bank = StoredPropertyBank {
                bank_version: bank_metadata.bank_version,
                recorded_at: bank_metadata.recorded_at,
                properties,
            };

            let bank = PropertyBank::try_from(stored_bank)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;

            // Validate each schema's property references
            for stored in schemas {
                for property in &stored.properties {
                    let prop_name = PropertyName::try_new(&property.name)
                        .map_err(|e| DbError::Transaction(e.to_string()))?;
                    if !bank.has(&prop_name) {
                        return Err(DbError::Transaction(format!(
                            "property '{}' in schema '{}' not found in \
                             PropertyBank",
                            property.name.as_ref(),
                            stored.name.as_ref()
                        )));
                    }
                }
            }
        }
        // If PropertyBank doesn't exist yet, allow saving schemas without
        // validation. This handles the initial bootstrap case where schemas
        // might be saved before the PropertyBank is initialized.

        // Atomic write
        self.db.batch_write(|batch| {
            for (stored, meta) in schemas.iter().zip(metadata.iter()) {
                let id_key = stored.id.into_uuid().to_string();
                batch.put(SCHEMA_BY_ID, id_key.as_str(), stored)?;
                batch.put(
                    SCHEMA_ID_BY_NAME,
                    stored.name.as_ref(),
                    &stored.id,
                )?;
                batch.put(SCHEMA_METADATA, id_key.as_str(), meta)?;
            }
            Ok(())
        })
    }

    /// Save a raw schema file to the database.
    ///
    /// Stores the file with its version history in the raw storage table.
    ///
    /// # Errors
    /// Returns `DbError` if the database operation fails.
    #[inline]
    #[instrument(skip(self, file), fields(file_path = file.file_path()))]
    pub fn save_raw_schema_file(
        &self,
        file: &RawSchemaFile,
    ) -> Result<(), DbError> {
        self.db.read_write_unit_of_work(|tx| {
            tx.put(RAW_SCHEMA_FILES, file.file_path(), file)?;
            Ok(())
        })
    }

    /// Save the raw property bank file to the database.
    ///
    /// Stores the singleton property bank file with its version history.
    ///
    /// # Errors
    /// Returns `DbError` if the database operation fails.
    #[inline]
    #[instrument(skip(self, file))]
    pub fn save_raw_property_bank_file(
        &self,
        file: &RawPropertyBankFile,
    ) -> Result<(), DbError> {
        self.db.read_write_unit_of_work(|tx| {
            tx.put(RAW_PROPERTY_BANK_FILE, RAW_PROPERTY_BANK_KEY, file)?;
            Ok(())
        })
    }

    /// Collect old version keys for retention cleanup.
    ///
    /// Keeps the last 3 versions to prevent unbounded disk growth.
    /// Returns (`id_keys`, `name_keys`) tuples to delete.
    fn collect_old_version_keys(
        &self,
        previous_metadata: Option<StoredMetadata>,
    ) -> (Vec<String>, Vec<String>) {
        const VERSION_RETENTION_COUNT: u64 = 3;

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
        if let Some(old_version) = version_to_delete {
            let prefix = StoredBankProperty::prefix(old_version);
            let id_keys = self
                .db
                .batch_read(|reader| {
                    reader.scan_range::<StoredBankProperty>(
                        BANK_PROPERTY_BY_ID,
                        &prefix,
                    )
                })
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        version = %old_version.as_u64(),
                        table = "BANK_PROPERTY_BY_ID",
                        %error,
                        "Failed to scan old property bank version for cleanup, \
                         retention may not work correctly"
                    );
                    Vec::new()
                });

            let name_keys = self
                .db
                .batch_read(|reader| {
                    reader.scan_range::<StoredBankProperty>(
                        BANK_PROPERTY_BY_NAME,
                        &prefix,
                    )
                })
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        version = %old_version.as_u64(),
                        table = "BANK_PROPERTY_BY_NAME",
                        %error,
                        "Failed to scan old property bank version for cleanup, \
                         retention may not work correctly"
                    );
                    Vec::new()
                });

            (
                id_keys.into_iter().map(|(k, _)| k).collect::<Vec<_>>(),
                name_keys.into_iter().map(|(k, _)| k).collect::<Vec<_>>(),
            )
        } else {
            (Vec::new(), Vec::new())
        }
    }
}

impl CommandPort for Command<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self, schemas),
        fields(operation = "save_schema_many", record_count = schemas.len())
    )]
    fn save_many(&self, schemas: &[StoredSchema]) -> Result<(), Self::Error> {
        // Use default metadata for port trait implementation
        // (tests and simple use cases don't need file timestamps)
        let metadata: Vec<StoredMetadata> = schemas
            .iter()
            .map(|_| {
                StoredMetadata::new(
                    BankVersion::initial(),
                    [0u8; 32],
                    None,
                    None,
                )
            })
            .collect();

        self.save_many_with_metadata(schemas, &metadata)
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "delete_schema", schema_id = %id.as_uuid())
    )]
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error> {
        use crate::schema::{
            aggregate::SchemaName,
            events::{Events, SchemaDeleted},
            storage::StoredSchema,
        };

        let id_uuid = id.into_uuid();
        let id_key = id_uuid.to_string();

        // Atomic delete: read + delete name index + delete schema + delete
        // metadata in single tx
        self.db.read_write_unit_of_work(|tx| {
            if let Some(stored) =
                tx.get_owned::<StoredSchema>(SCHEMA_BY_ID, id_key.as_str())?
            {
                // Emit SchemaDeleted event
                let schema_name = SchemaName::try_new(stored.name.as_ref())
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;

                tracing::info!(
                    schema_id = %id,
                    schema_name = %schema_name.as_str(),
                    "Schema deleted"
                );

                // TODO(EVENT-001): Persist event to SCHEMA_EVENTS table once
                // event store is implemented (Phase 2)
                let timestamp = SystemTime::now();
                let _event = Events::SchemaDeleted(SchemaDeleted::new(
                    id,
                    &schema_name,
                    timestamp,
                ));

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
        let bank_version = bank.version();
        let recorded_at = SystemTime::now();

        // Read current metadata to determine old versions to delete
        let previous_metadata = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?;

        // Collect keys to delete from both tables before write transaction
        let keys_to_delete = self.collect_old_version_keys(previous_metadata);

        let metadata = StoredMetadata {
            bank_version,
            source_file_hash: [0u8; 32],
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

    #[inline]
    fn save_inheritance_many(
        &self,
        relationships: &[crate::schema::ports::InheritanceRelationship],
    ) -> Result<(), Self::Error> {
        let old_parents = self.load_old_parent_refs(relationships)?;

        #[expect(
            clippy::ref_patterns,
            reason = "Destructuring with &(a, b, ref c) is clearest for mixed \
                      Copy/non-Copy fields"
        )]
        self.db.batch_write(|writer| {
            for &(child_id, parent_id, ref excludes) in relationships {
                let child_key = child_id.to_string();
                let timestamp = SystemTime::now();

                // Remove old parent→child multimap entry if parent changed
                if let Some(old_ref) = old_parents.get(&child_id)
                    && let Some(old_parent_id) = old_ref.parent_id
                {
                    let old_schema = StoredChildSchema {
                        child_id,
                        excludes: old_ref.excludes.clone(),
                        resolved_at: old_ref.resolved_at,
                    };
                    let old_bytes = old_schema.to_bytes()?;
                    writer.multimap_remove_bytes(
                        SCHEMA_CHILDREN,
                        old_parent_id.to_string().as_str(),
                        old_bytes.as_slice(),
                    )?;
                }

                // Insert new parent→child multimap entry (if not root)
                if let Some(pid) = parent_id {
                    let child_schema = StoredChildSchema {
                        child_id,
                        excludes: excludes.clone(),
                        resolved_at: timestamp,
                    };
                    let bytes = child_schema.to_bytes()?;
                    writer.multimap_insert_bytes(
                        SCHEMA_CHILDREN,
                        pid.to_string().as_str(),
                        bytes.as_slice(),
                    )?;
                }

                // Update child→parent reference table
                let parent_schema = StoredParentSchema {
                    parent_id,
                    excludes: excludes.clone(),
                    resolved_at: timestamp,
                };
                writer.put(
                    SCHEMA_PARENT,
                    child_key.as_str(),
                    &parent_schema,
                )?;
            }

            Ok(())
        })
    }
}

// Private helper methods for inheritance tracking
impl Command<'_> {
    /// Load existing parent references for all children in the batch.
    fn load_old_parent_refs(
        &self,
        relationships: &[crate::schema::ports::InheritanceRelationship],
    ) -> Result<HashMap<SchemaId, StoredParentSchema>, DbError> {
        let mut old_parents = HashMap::with_capacity(relationships.len());

        for &(child_id, _, _) in relationships {
            let child_key = child_id.to_string();
            if let Some(old_ref) = self.db.get_owned::<StoredParentSchema>(
                SCHEMA_PARENT,
                child_key.as_str(),
            )? {
                old_parents.insert(child_id, old_ref);
            }
        }

        Ok(old_parents)
    }
}
