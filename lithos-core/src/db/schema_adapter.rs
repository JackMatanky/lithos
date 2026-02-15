//! Schema port adapters for the database.

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        aggregate::{
            PropertyBank, PropertyBankId, ResolutionMetadata, Schema, SchemaId,
            SchemaName, SchemaNameKey,
        },
        db_table::{
            PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME, SCHEMA_METADATA,
        },
        ports::{Command, Query},
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
        skip(self, schema),
        fields(operation = "save_schema", schema_id = %schema.id().as_uuid())
    )]
    fn save_with_metadata(
        &self,
        schema: &Schema,
        metadata: &ResolutionMetadata,
    ) -> Result<(), Self::Error> {
        let id = schema.id();
        let id_uuid = id.into_uuid();
        let name_key = SchemaNameKey::from(schema.name());

        if let Some(existing) = self
            .db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())?
            && existing != id
        {
            return Err(DbError::Transaction(format!(
                "schema name already exists: {}",
                schema.name().as_str()
            )));
        }

        self.db.put_by_uuid(SCHEMA_BY_ID, id_uuid, schema)?;
        self.db.put_by_uuid(SCHEMA_METADATA, id_uuid, metadata)?;
        self.db.put(SCHEMA_ID_BY_NAME, name_key.as_str(), &id)?;
        Ok(())
    }

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
            let name_key = SchemaNameKey::from(schema.name());
            if let Some(_existing) =
                name_index.insert(name_key.clone(), schema.id())
            {
                return Err(DbError::Transaction(format!(
                    "schema name already exists in batch: {}",
                    schema.name().as_str()
                )));
            }

            if let Some(existing) = self
                .db
                .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())?
                && existing != schema.id()
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
                let name_key = SchemaNameKey::from(schema.name());

                batch.put(SCHEMA_BY_ID, id_key.as_str(), schema)?;
                batch.put(SCHEMA_METADATA, id_key.as_str(), metadata)?;
                batch.put(
                    SCHEMA_ID_BY_NAME,
                    name_key.as_str(),
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
            let name_key = SchemaNameKey::from(schema.name());
            self.db.delete(SCHEMA_ID_BY_NAME, name_key.as_str())?;
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
    type Archived<'archived> = &'archived rkyv::Archived<Schema>;
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "find_schema_by_id", schema_id = %id.as_uuid())
    )]
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        let key = id.as_uuid().to_string();
        self.db.get_owned(SCHEMA_BY_ID, &key)
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "list_schemas")
    )]
    fn list(&self) -> Result<Vec<Schema>, Self::Error> {
        self.db.list_owned(SCHEMA_BY_ID)
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "lookup_schema_id", schema_name = %name)
    )]
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        let name_key = SchemaNameKey::from(name);
        self.db.get_owned(SCHEMA_ID_BY_NAME, name_key.as_str())
    }

    #[inline]
    #[instrument(
        skip(self, f),
        level = "debug",
        fields(operation = "with_archived_schema", schema_id = %id.as_uuid())
    )]
    fn with_archived_by_id<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let key = id.as_uuid().to_string();
        self.db.get::<Schema, _, _>(SCHEMA_BY_ID, &key, f)
    }

    #[inline]
    #[instrument(
        skip(self, f),
        level = "debug",
        fields(operation = "with_archived_schema_by_name", schema_name = %name)
    )]
    fn with_archived_by_name<F, R>(
        &self,
        name: &SchemaName,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let name_key = SchemaNameKey::from(name);
        let Some(id) = self
            .db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())?
        else {
            return Ok(None);
        };

        self.with_archived_by_id(id, f)
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "list_schema_metadata")
    )]
    fn list_metadata(&self) -> Result<Vec<ResolutionMetadata>, Self::Error> {
        self.db.list_owned(SCHEMA_METADATA)
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "find_schema_metadata", schema_id = %id.as_uuid())
    )]
    fn find_metadata_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<ResolutionMetadata>, Self::Error> {
        let key = id.as_uuid().to_string();
        self.db.get_owned(SCHEMA_METADATA, &key)
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "find_property_bank")
    )]
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let id = PropertyBankId::singleton();
        let key = id.as_uuid().to_string();
        self.db.get_owned(PROPERTY_BANK, &key)
    }
}
