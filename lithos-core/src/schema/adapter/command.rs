//! Redb-backed implementation of the [`crate::schema::ports::Command`] trait.

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        aggregate::{ResolutionMetadata, Schema, SchemaId},
        bank::PropertyBank,
        db_table::{
            PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME, SCHEMA_METADATA,
        },
        ports::Command,
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
        skip(self, schemas),
        fields(operation = "save_schema_batch", schema_count = schemas.len())
    )]
    fn save_batch(
        &self,
        schemas: &[(Schema, ResolutionMetadata)],
    ) -> Result<(), Self::Error> {
        let mut name_index = std::collections::HashMap::new();

        for pair in schemas {
            let schema = &pair.0;
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
            for pair in schemas {
                let schema = &pair.0;
                let metadata = &pair.1;
                let id_uuid = schema.id().into_uuid();
                let id_key = id_uuid.to_string();

                batch.put(SCHEMA_BY_ID, id_key.as_str(), schema)?;
                batch.put(SCHEMA_METADATA, id_key.as_str(), metadata)?;
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
        let id_uuid = id.into_uuid();
        let key = id_uuid.to_string();

        if let Some(schema) = self.db.get_owned::<Schema>(SCHEMA_BY_ID, &key)? {
            self.db.delete(SCHEMA_ID_BY_NAME, schema.name().as_str())?;
        }

        self.db.delete_by_uuid(SCHEMA_BY_ID, id_uuid)?;
        self.db.delete_by_uuid(SCHEMA_METADATA, id_uuid)?;
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
