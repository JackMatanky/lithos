//! Redb-backed implementation of the [`crate::schema::ports::Query`] trait.

use crate::{
    db::{BatchReader, Database, DbError},
    schema::{
        adapter::stored::StoredSchema,
        aggregate::{Schema, SchemaId, SchemaName, Timestamp},
        bank::{BankVersion, PropertyBank, PropertyBankId},
        db_table::{PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
        ports::{NameIdPair, Query},
    },
};

/// Redb-backed schema query adapter.
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a query adapter for a database.
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
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        self.db
            .get_owned_by_uuid::<StoredSchema>(SCHEMA_BY_ID, id.into_uuid())?
            .map(Schema::try_from)
            .transpose()
            .map_err(|e| DbError::Transaction(e.to_string()))
    }

    #[inline]
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let id = PropertyBankId::singleton();
        self.db.get_owned_by_uuid(PROPERTY_BANK, id.into_uuid())
    }

    #[inline]
    fn is_bank_stale(
        &self,
        current_version: BankVersion,
    ) -> Result<bool, Self::Error> {
        let id = PropertyBankId::singleton();
        let Some(bank) = self
            .db
            .get_owned_by_uuid::<PropertyBank>(PROPERTY_BANK, id.into_uuid())?
        else {
            return Ok(true);
        };
        Ok(bank.version() != current_version)
    }

    #[inline]
    fn is_schema_stale(
        &self,
        id: SchemaId,
        file_mtime: Option<Timestamp>,
        current_bank_version: BankVersion,
    ) -> Result<bool, Self::Error> {
        let Some(stored) = self
            .db
            .get_owned_by_uuid::<StoredSchema>(SCHEMA_BY_ID, id.into_uuid())?
        else {
            // No stored record — always stale.
            return Ok(true);
        };

        if stored.bank_version != current_bank_version {
            return Ok(true);
        }

        if let Some(mtime) = file_mtime
            && stored.modified_at.as_secs() < mtime.as_secs()
        {
            return Ok(true);
        }

        Ok(false)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Schema>, Self::Error> {
        let stored: Vec<StoredSchema> = self.db.list_owned(SCHEMA_BY_ID)?;
        stored
            .into_iter()
            .map(|s| {
                Schema::try_from(s)
                    .map_err(|e| DbError::Transaction(e.to_string()))
            })
            .collect()
    }

    #[inline]
    fn list_name_id_pairs(&self) -> Result<Vec<NameIdPair>, Self::Error> {
        self.db.list_key_value_pairs::<SchemaId>(SCHEMA_ID_BY_NAME).map(
            |pairs| {
                pairs
                    .into_iter()
                    .filter_map(|(name, id)| {
                        SchemaName::new(&name).ok().map(|name| (name, id))
                    })
                    .collect()
            },
        )
    }

    #[inline]
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        self.db.get_owned(SCHEMA_ID_BY_NAME, name.as_str())
    }
}
