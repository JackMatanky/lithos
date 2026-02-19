//! Redb-backed implementation of the [`crate::schema::ports::Command`] trait.

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        adapter::stored::to_stored,
        aggregate::SchemaId,
        bank::PropertyBank,
        db_table::{PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
        ports::{Command, SchemaRecord},
    },
};

/// Redb-backed schema command adapter.
pub struct CommandAdapter<'db> {
    db: &'db Database,
}

impl<'db> CommandAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a command adapter for a database.
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Command for CommandAdapter<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self, records),
        fields(operation = "save_schema_batch", record_count = records.len())
    )]
    fn save_batch(&self, records: &[SchemaRecord]) -> Result<(), Self::Error> {
        let mut name_index = std::collections::HashMap::new();

        for record in records {
            let schema = &record.schema;
            if let Some(_existing) =
                name_index.insert(schema.name().clone(), schema.id())
            {
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

        self.db.batch_write(|batch| {
            for record in records {
                let schema = &record.schema;
                let stored = to_stored(
                    schema,
                    record.parent_id,
                    record.bank_version,
                    record.created_at,
                    record.modified_at,
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

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "delete_schema", schema_id = %id.as_uuid())
    )]
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error> {
        use crate::schema::adapter::stored::StoredSchema;

        let id_uuid = id.into_uuid();
        if let Some(stored) =
            self.db.get_owned_by_uuid::<StoredSchema>(SCHEMA_BY_ID, id_uuid)?
        {
            self.db.delete(SCHEMA_ID_BY_NAME, stored.name.as_ref())?;
        }

        // Use the string key for deletion (same format as put).
        let id_key = id_uuid.to_string();
        self.db.delete(SCHEMA_BY_ID, id_key.as_str())?;
        Ok(())
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
