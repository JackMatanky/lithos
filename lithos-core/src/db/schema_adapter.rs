//! Schema port adapters for the database.

use tracing::instrument;

use crate::{
    db::{Database, DbError},
    schema::{
        aggregate::{Schema, SchemaId, SchemaName, SchemaNameKey},
        db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
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
    fn save(&self, schema: &Schema) -> Result<(), Self::Error> {
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
        self.db.put(SCHEMA_ID_BY_NAME, name_key.as_str(), &id)?;
        Ok(())
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
}
