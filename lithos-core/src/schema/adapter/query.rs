//! Redb-backed implementation of the [`crate::schema::ports::Query`] trait.

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
        ports::Query,
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
    type Archived<'archived> = &'archived rkyv::Archived<Schema>;
    type Error = DbError;

    #[inline]
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        self.db.get_owned(SCHEMA_BY_ID, &id.into_uuid().to_string())
    }

    #[inline]
    fn find_metadata_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<ResolutionMetadata>, Self::Error> {
        self.db.get_owned(SCHEMA_METADATA, &id.into_uuid().to_string())
    }

    #[inline]
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let id = PropertyBankId::singleton();
        let key = id.into_uuid().to_string();
        self.db.get_owned(PROPERTY_BANK, &key)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Schema>, Self::Error> {
        self.db.list_owned(SCHEMA_BY_ID)
    }

    #[inline]
    fn list_metadata(&self) -> Result<Vec<ResolutionMetadata>, Self::Error> {
        self.db.list_owned(SCHEMA_METADATA)
    }

    #[inline]
    fn lookup_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        let name_key = SchemaNameKey::from(name);
        self.db.get_owned(SCHEMA_ID_BY_NAME, name_key.as_str())
    }

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        self.db.get::<Schema, _, R>(
            SCHEMA_BY_ID,
            &id.into_uuid().to_string(),
            f,
        )
    }

    #[inline]
    fn with_archived_by_name<F, R>(
        &self,
        name: &SchemaName,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let name_key = SchemaNameKey::from(name);
        if let Some(id) = self
            .db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())?
        {
            self.with_archived_by_id(id, f)
        } else {
            Ok(None)
        }
    }
}
