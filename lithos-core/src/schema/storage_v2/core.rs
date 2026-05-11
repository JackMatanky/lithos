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

    #[inline]
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaStorageV2Error> {
        self.store
            .write(|tx| {
                let mut table = tx.inner.open_table(SCHEMAS.definition())?;
                for schema in schemas {
                    let bytes = serialize(schema)?;
                    table.insert(schema.id(), bytes.as_slice())?;
                }
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let table = match tx.inner.open_table(SCHEMAS.definition()) {
                    Ok(t) => t,
                    Err(redb::TableError::TableDoesNotExist(_)) => {
                        return Ok(ids.iter().map(|_| None).collect());
                    }
                    Err(e) => return Err(e.into()),
                };

                let mut results = Vec::with_capacity(ids.len());
                for id in ids {
                    let guard = table.get(*id)?;
                    let schema =
                        guard.map(|g| deserialize(g.value())).transpose()?;
                    results.push(schema);
                }
                Ok(results)
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
    fn save_many_schemas_persists_all() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let schema1 = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-1").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };
        let schema2 = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-2").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };

        repo.save_many_schemas(&[schema1.clone(), schema2.clone()]).unwrap();

        let found1 = repo
            .find_schema_by_id(*schema1.id())
            .unwrap()
            .expect("Schema 1 should be found");
        let found2 = repo
            .find_schema_by_id(*schema2.id())
            .unwrap()
            .expect("Schema 2 should be found");

        assert_eq!(found1.id(), schema1.id());
        assert_eq!(found2.id(), schema2.id());
    }

    #[test]
    fn save_many_schemas_empty_batch_succeeds() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        repo.save_many_schemas(&[]).unwrap();
    }

    #[test]
    fn save_many_schemas_rolls_back_on_failure() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store.clone());

        let schema = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };

        repo.save_many_schemas(std::slice::from_ref(&schema)).unwrap();

        let result: Result<(), crate::db::DbError> = store.write(|tx| {
            use crate::schema::storage_v2::tables::SCHEMAS;
            let mut table = tx.inner.open_table(SCHEMAS.definition())?;
            let id2 = SchemaId::new();
            table.insert(&id2, &b"invalid"[..])?;
            Err(crate::db::DbError::Serialization("forced".to_owned()))
        });
        assert!(result.is_err());

        let found = repo.find_schema_by_id(*schema.id()).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn save_many_schemas_auto_commits() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let schema = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };

        {
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);
            repo.save_many_schemas(std::slice::from_ref(&schema)).unwrap();
        }

        {
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);
            let found = repo
                .find_schema_by_id(*schema.id())
                .unwrap()
                .expect("Should be found after reopen");
            assert_eq!(found.id(), schema.id());
        }
    }

    #[test]
    fn find_many_schemas_by_id_returns_all_found() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let schema1 = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-1").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };
        let schema2 = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-2").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };

        repo.save_many_schemas(&[schema1.clone(), schema2.clone()]).unwrap();

        let results = repo
            .find_many_schemas_by_id(&[*schema1.id(), *schema2.id()])
            .unwrap();

        assert!(results.first().and_then(Option::as_ref).is_some());
        assert!(results.get(1).and_then(Option::as_ref).is_some());
        assert_eq!(
            results.first().and_then(Option::as_ref).map(Schema::id),
            Some(schema1.id())
        );
        assert_eq!(
            results.get(1).and_then(Option::as_ref).map(Schema::id),
            Some(schema2.id())
        );
    }

    #[test]
    fn find_many_schemas_by_id_returns_none_for_missing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let id1 = SchemaId::new();
        let id2 = SchemaId::new();

        let results = repo.find_many_schemas_by_id(&[id1, id2]).unwrap();

        assert!(results.first().and_then(Option::as_ref).is_none());
        assert!(results.get(1).and_then(Option::as_ref).is_none());
    }

    #[test]
    fn find_many_schemas_by_id_partial_found() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let existing = {
            let id = SchemaId::new();
            let name = SchemaName::try_new("existing").unwrap();
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new())
        };
        let missing = SchemaId::new();

        repo.save_many_schemas(std::slice::from_ref(&existing)).unwrap();

        let results =
            repo.find_many_schemas_by_id(&[*existing.id(), missing]).unwrap();

        assert!(results.first().and_then(Option::as_ref).is_some());
        assert!(results.get(1).and_then(Option::as_ref).is_none());
    }

    #[test]
    fn find_many_schemas_by_id_empty_batch_succeeds() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let results = repo.find_many_schemas_by_id(&[]).unwrap();
        assert!(results.is_empty());
    }
}
