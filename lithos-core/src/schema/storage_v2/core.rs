//! Core implementation of `SchemaRedbRepository`.

use std::sync::Arc;

use crate::{
    db::{Store, deserialize, serialize},
    schema::{
        aggregate::Schema,
        identifier::SchemaId,
        repository::{SchemaRepository, SchemaStorageV2Error},
        storage_v2::tables::SCHEMAS,
    },
};

/// Repository adapter for `redb`-backed schema storage.
///
/// This adapter implements the
/// [`SchemaRepository`](super::repository::SchemaRepository) trait using `redb`
/// as the underlying storage engine. It manages its own transaction boundaries
/// via the provided [`Store`].
#[derive(Debug)]
pub struct SchemaRedbRepository {
    store: Arc<Store>,
}

impl SchemaRedbRepository {
    /// Create a new repository adapter from a database store.
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

impl SchemaRepository for SchemaRedbRepository {
    #[inline]
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageV2Error> {
        let bytes = serialize(schema).map_err(SchemaStorageV2Error::from)?;

        self.store
            .write(|tx| {
                let mut table = tx.inner.open_table(SCHEMAS.definition())?;
                table.insert(schema.id(), bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let table = match tx.inner.open_table(SCHEMAS.definition()) {
                    Ok(t) => t,
                    Err(redb::TableError::TableDoesNotExist(_)) => {
                        return Ok(None);
                    }
                    Err(e) => return Err(e.into()),
                };

                let Some(guard) = table.get(id)? else {
                    return Ok(None);
                };

                let schema = deserialize(guard.value())?;
                Ok(Some(schema))
            })
            .map_err(SchemaStorageV2Error::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::Store,
        schema::{
            aggregate::Schema,
            identifier::{SchemaId, SchemaName},
            property::PropertyMap,
            repository::SchemaRepository,
            storage_v2::SchemaRedbRepository,
        },
    };

    #[test]
    fn save_schema_persists_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let id = SchemaId::new();
        let name = SchemaName::try_new("test-schema").unwrap();
        let schema =
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

        repo.save_schema(&schema).unwrap();
    }

    #[test]
    fn find_schema_by_id_returns_persisted_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let id = SchemaId::new();
        let name = SchemaName::try_new("test-schema").unwrap();
        let schema =
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

        repo.save_schema(&schema).unwrap();

        let found = repo
            .find_schema_by_id(id)
            .unwrap()
            .expect("Schema should be found");
        assert_eq!(found.id(), schema.id());
        assert_eq!(found.name(), schema.name());
    }

    #[test]
    fn find_schema_by_id_returns_none_for_missing_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let id = SchemaId::new();
        let found = repo.find_schema_by_id(id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn save_schema_rolls_back_on_error() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store.clone());

        let id = SchemaId::new();
        let name = SchemaName::try_new("test-schema").unwrap();
        let schema =
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

        repo.save_schema(&schema).unwrap();

        let result: Result<(), crate::db::DbError> = store.write(|tx| {
            use crate::schema::storage_v2::tables::SCHEMAS;
            let mut table = tx.inner.open_table(SCHEMAS.definition())?;
            let id2 = SchemaId::new();
            table.insert(&id2, &b"invalid data"[..])?;
            Err(crate::db::DbError::Serialization("forced failure".to_owned()))
        });
        assert!(result.is_err());

        let found = repo.find_schema_by_id(id).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn save_schema_auto_commits_on_success() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let id = SchemaId::new();
        let name = SchemaName::try_new("test-schema").unwrap();
        let schema =
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

        {
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);
            repo.save_schema(&schema).unwrap();
        }

        {
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);
            let found = repo
                .find_schema_by_id(id)
                .unwrap()
                .expect("Should be found after reopen");
            assert_eq!(found.id(), schema.id());
        }
    }
}
