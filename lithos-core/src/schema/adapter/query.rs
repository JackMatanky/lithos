//! Redb-backed implementation of the [`crate::schema::ports::Query`] trait.
//!
//! Property bank reads use `bank_metadata` plus versioned rows from
//! `bank_property_by_name`.

use crate::{
    db::{BatchReader, Database, DbError},
    schema::{
        adapter::stored::{
            StoredBankProperty, StoredMetadata, StoredPropertyBank,
            StoredSchema,
        },
        aggregate::{Schema, SchemaId, SchemaName, Timestamp},
        bank::{BankVersion, PropertyBank},
        db_table::{
            BANK_METADATA, BANK_PROPERTY_BY_ID, BANK_PROPERTY_BY_NAME,
            PROPERTY_BANK_KEY, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME,
            SCHEMA_METADATA,
        },
        ports::{NameIdPair, Query},
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
    },
};

/// Redb-backed schema query adapter.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::adapter::query::QueryAdapter;
///
/// let db = todo!("Provide a Database instance");
/// let adapter = QueryAdapter::new(&db);
/// let _ = adapter;
/// ```
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a query adapter for a database.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::adapter::query::QueryAdapter;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = QueryAdapter::new(&db);
    /// let _ = adapter;
    /// ```
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Query for QueryAdapter<'_> {
    type Error = DbError;

    #[inline]
    fn batch_read<R, F>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>,
    {
        self.db.batch_read(f)
    }

    #[inline]
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let Some(metadata) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(None);
        };

        let prefix = StoredBankProperty::prefix(metadata.bank_version);
        let entries = self
            .db
            .scan_range::<StoredBankProperty>(BANK_PROPERTY_BY_NAME, &prefix)?;
        let properties: Vec<_> =
            entries.into_iter().map(|(_, stored)| stored.property).collect();

        let stored = StoredPropertyBank {
            bank_version: metadata.bank_version,
            recorded_at: metadata.recorded_at,
            properties,
        };

        PropertyBank::try_from(stored)
            .map(Some)
            .map_err(|e| DbError::Deserialization(e.to_string()))
    }

    #[inline]
    fn get_property_by_id(
        &self,
        id: PropertyId,
    ) -> Result<Option<Property>, Self::Error> {
        // Get current bank version from metadata
        let Some(metadata) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(None);
        };

        // Query BANK_PROPERTY_BY_ID with versioned key
        let key =
            StoredBankProperty::key(metadata.bank_version, &id.to_string());
        let Some(stored) = self
            .db
            .get_owned::<StoredBankProperty>(BANK_PROPERTY_BY_ID, &key)?
        else {
            return Ok(None);
        };

        // Reconstruct Property from StoredProperty
        let sp = stored.property;
        let prop_name = PropertyName::try_from(sp.name)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let optionality = Optionality::from(sp.required);
        let multiplicity = Multiplicity::from(sp.multi);

        Ok(Some(Property::new(
            sp.id,
            prop_name,
            optionality,
            multiplicity,
            sp.spec,
        )))
    }

    #[inline]
    fn is_bank_stale(&self, version: BankVersion) -> Result<bool, Self::Error> {
        let Some(stored) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(true);
        };
        Ok(stored.bank_version != version)
    }

    #[inline]
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        let Some(stored) = self
            .db
            .get_owned_by_uuid::<StoredSchema>(SCHEMA_BY_ID, id.into_uuid())?
        else {
            return Ok(None);
        };

        // Validate schema-metadata consistency to detect corruption
        let id_key = id.into_uuid().to_string();
        let metadata_exists = self
            .db
            .get_owned::<StoredMetadata>(SCHEMA_METADATA, id_key.as_str())?
            .is_some();

        if !metadata_exists {
            return Err(DbError::Corruption(format!(
                "schema {} exists but metadata is missing (database \
                 corruption detected)",
                id.as_uuid()
            )));
        }

        Schema::try_from(stored)
            .map(Some)
            .map_err(|e| DbError::Deserialization(e.to_string()))
    }

    #[inline]
    fn is_schema_stale(
        &self,
        id: SchemaId,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
        bank_version: BankVersion,
    ) -> Result<bool, Self::Error> {
        // Zero-copy metadata check using closure-based API.
        // This eliminates the double deserialization in the old implementation:
        // - Old: get() to check existence, then get_owned() to deserialize full
        //   struct
        // - New: Single zero-copy read with minimal field deserialization
        let Some(result) =
            self.with_metadata(id, |stored| -> Result<bool, DbError> {
                // Deserialize only the primitive fields we need to compare.
                // This is faster than get_owned() which allocates and
                // deserializes the entire struct including
                // fields we don't need.
                let stored_version: BankVersion =
                    rkyv::deserialize::<BankVersion, rkyv::rancor::Error>(
                        &stored.bank_version,
                    )
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;

                if stored_version != bank_version {
                    return Ok(false); // Not fresh: version mismatch
                }

                // Verify file identity via created_at when possible.
                if let (Some(file_created), Some(archived_created)) =
                    (created_at, stored.created_at.as_ref())
                {
                    let stored_created: Timestamp =
                        rkyv::deserialize::<Timestamp, rkyv::rancor::Error>(
                            archived_created,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;

                    if file_created.as_secs() != stored_created.as_secs() {
                        return Ok(false); // Not fresh: created_at mismatch
                    }
                } else if created_at.is_some() != stored.created_at.is_some() {
                    tracing::warn!(
                        schema_id = %id,
                        "Cannot verify schema identity: created_at unavailable"
                    );
                } else {
                    // Both are None - no timestamp to verify
                }

                // Compare file mtime with stored mtime.
                if let (Some(file_mtime), Some(archived_mtime)) =
                    (modified_at, stored.modified_at.as_ref())
                {
                    let stored_mtime: Timestamp =
                        rkyv::deserialize::<Timestamp, rkyv::rancor::Error>(
                            archived_mtime,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;

                    if stored_mtime.as_secs() < file_mtime.as_secs() {
                        return Ok(false); // Not fresh: file modified after storage
                    }
                }

                Ok(true) // Fresh: all checks passed
            })?
        else {
            // Metadata not found → schema is stale.
            return Ok(true);
        };

        // Flatten the nested Result
        let is_fresh = result?;
        Ok(!is_fresh)
    }

    #[inline]
    fn with_metadata<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: FnOnce(&rkyv::Archived<StoredMetadata>) -> R,
    {
        let id_key = id.into_uuid().to_string();
        self.db.get::<StoredMetadata, _, _>(SCHEMA_METADATA, id_key.as_str(), f)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Schema>, Self::Error> {
        let stored: Vec<StoredSchema> = self.db.list_owned(SCHEMA_BY_ID)?;
        stored
            .into_iter()
            .map(|s| {
                Schema::try_from(s)
                    .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .collect()
    }

    #[inline]
    fn list_name_id_pairs(&self) -> Result<Vec<NameIdPair>, Self::Error> {
        let pairs =
            self.db.list_key_value_pairs::<SchemaId>(SCHEMA_ID_BY_NAME)?;
        pairs
            .into_iter()
            .map(|(name, id)| {
                SchemaName::new(&name)
                    .map(|schema_name| (schema_name, id))
                    .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .collect()
    }

    #[inline]
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        self.db.get_owned(SCHEMA_ID_BY_NAME, name.as_str())
    }
}
