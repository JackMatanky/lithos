//! Redb-backed implementation of the [`crate::schema::ports::Query`] trait.

use crate::{
    db::{BatchReader, Database, DbError},
    schema::{
        aggregate::{ResolutionMetadata, Schema, SchemaId, SchemaName},
        bank::{PropertyBank, PropertyBankId},
        db_table::{
            PROPERTY_BANK, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME, SCHEMA_METADATA,
        },
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
    type Archived<'archived> = &'archived rkyv::Archived<Schema>;
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
        self.db.get_owned_by_uuid(SCHEMA_BY_ID, id.into_uuid())
    }

    #[inline]
    fn find_metadata_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<ResolutionMetadata>, Self::Error> {
        self.db.get_owned_by_uuid(SCHEMA_METADATA, id.into_uuid())
    }

    #[inline]
    fn find_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let id = PropertyBankId::singleton();
        self.db.get_owned_by_uuid(PROPERTY_BANK, id.into_uuid())
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

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        self.db.get_by_uuid::<Schema, _, R>(SCHEMA_BY_ID, id.into_uuid(), f)
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
        if let Some(id) =
            self.db.get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())?
        {
            self.with_archived_by_id(id, f)
        } else {
            Ok(None)
        }
    }
}
