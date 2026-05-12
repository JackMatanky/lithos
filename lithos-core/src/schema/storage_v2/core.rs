//! Core implementation of `SchemaRedbRepository`.

use std::sync::Arc;

use crate::{
    db::{Store, UuidTableReadExt, deserialize, serialize},
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
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                table.insert(*schema.id(), bytes.as_slice())?;
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
                let Some(table) = tx.try_open_table(SCHEMAS.definition())?
                else {
                    return Ok(None);
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
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                for schema in schemas {
                    let bytes = serialize(schema)?;
                    table.insert(*schema.id(), bytes.as_slice())?;
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
                let Some(table) = tx.try_open_table(SCHEMAS.definition())?
                else {
                    return Ok(ids.iter().map(|_| None).collect());
                };

                let results = table
                    .get_many(ids)?
                    .into_iter()
                    .map(|guard_opt| {
                        guard_opt
                            .map(|g| deserialize::<Schema>(g.value()))
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(results)
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[crate::fs::RelativePath],
    ) -> Result<
        Vec<Option<crate::schema::views::RawSchemaView>>,
        SchemaStorageV2Error,
    > {
        use crate::schema::storage_v2::tables::{
            RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH,
        };

        self.store
            .read(|tx| {
                // Open both tables
                let path_table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                let view_table =
                    tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;

                let (Some(path_table), Some(view_table)) =
                    (path_table, view_table)
                else {
                    return Ok(paths.iter().map(|_| None).collect());
                };

                let mut results = Vec::with_capacity(paths.len());
                for path in paths {
                    // Step 1: path → SchemaId lookup
                    let path_str = path.as_path().to_string_lossy().to_string();
                    let id_guard = path_table.get(path_str)?;

                    let view = if let Some(id_bytes) = id_guard {
                        // Step 2: SchemaId → RawSchemaView lookup
                        let id: SchemaId = deserialize(id_bytes.value())?;
                        let view_guard = view_table.get(&id)?;
                        view_guard
                            .map(|g| deserialize(g.value()))
                            .transpose()?
                    } else {
                        None
                    };

                    results.push(view);
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
            let mut table = tx.try_open_table(SCHEMAS.definition())?;
            let id2 = SchemaId::new();
            table.insert(id2, &b"invalid data"[..])?;
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
            let mut table = tx.try_open_table(SCHEMAS.definition())?;
            let id2 = SchemaId::new();
            table.insert(id2, &b"invalid"[..])?;
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

    mod cross_table_batch {
        use super::*;
        use crate::{
            db::serialize,
            fs::{FileInfo, RelativePath},
            schema::{
                raw::{RawPropertyMap, RawSchema, RawSchemaVersion},
                views::{
                    RawPropertyHashIndex, SchemaVersion, hashes::HashRecord,
                    raw::RawSchemaView,
                },
            },
            support::hash::Blake3Hash,
        };

        /// Cross-table batch read: path → `SchemaId` → `RawSchemaView`.
        ///
        /// Behavior: Given paths, lookup IDs in `SCHEMA_ID_BY_PATH`, then fetch
        /// views from `RAW_SCHEMA_VIEWS` in single transaction.
        /// Verification: Inserting path→ID→view mappings, then batch read
        /// returns views in correct order.
        #[test]
        fn finds_views_by_paths_in_single_transaction() {
            use crate::schema::storage_v2::tables::{
                RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH,
            };

            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            // Setup: Insert test data
            let path1 = RelativePath::try_from("schemas/note.json").unwrap();
            let path2 = RelativePath::try_from("schemas/task.json").unwrap();
            let id1 = SchemaId::new();
            let id2 = SchemaId::new();

            let view1 = {
                let info = FileInfo::new(None, None, 100);
                let hashes = HashRecord::new(
                    Blake3Hash::new([1; 32]),
                    RawPropertyHashIndex::default(),
                );
                let raw = RawSchema {
                    version: RawSchemaVersion::default(),
                    name: "Note".into(),
                    extends: None,
                    excludes: vec![],
                    properties: RawPropertyMap::new(),
                    info: FileInfo::new(None, None, 0),
                };
                let version = SchemaVersion::new(info, hashes, &raw).unwrap();
                RawSchemaView::new(path1.clone(), version)
            };

            let view2 = {
                let info = FileInfo::new(None, None, 200);
                let hashes = HashRecord::new(
                    Blake3Hash::new([2; 32]),
                    RawPropertyHashIndex::default(),
                );
                let raw = RawSchema {
                    version: RawSchemaVersion::default(),
                    name: "Task".into(),
                    extends: None,
                    excludes: vec![],
                    properties: RawPropertyMap::new(),
                    info: FileInfo::new(None, None, 0),
                };
                let version = SchemaVersion::new(info, hashes, &raw).unwrap();
                RawSchemaView::new(path2.clone(), version)
            };

            store
                .write(|tx| {
                    let mut path_table =
                        tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                    let mut view_table =
                        tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;

                    // Insert path → ID mappings
                    let path1_str =
                        path1.as_path().to_string_lossy().to_string();
                    let path2_str =
                        path2.as_path().to_string_lossy().to_string();
                    path_table
                        .insert(path1_str, serialize(&id1)?.as_slice())?;
                    path_table
                        .insert(path2_str, serialize(&id2)?.as_slice())?;

                    // Insert ID → view mappings
                    view_table.insert(id1, serialize(&view1)?.as_slice())?;
                    view_table.insert(id2, serialize(&view2)?.as_slice())?;

                    Ok(())
                })
                .unwrap();

            // Execute
            let repo = SchemaRedbRepository::new(store);
            let results = repo
                .find_raw_schema_views_by_paths(&[path1.clone(), path2.clone()])
                .unwrap();

            // Verify
            assert_eq!(results.len(), 2);
            assert!(results.first().and_then(Option::as_ref).is_some());
            assert!(results.get(1).and_then(Option::as_ref).is_some());

            let found1 = results.first().and_then(Option::as_ref).unwrap();
            let found2 = results.get(1).and_then(Option::as_ref).unwrap();
            assert_eq!(found1.path(), &path1);
            assert_eq!(found2.path(), &path2);
        }

        /// Empty batch succeeds.
        ///
        /// Behavior: Calling with empty slice returns empty results.
        /// Verification: Result vector is empty.
        #[test]
        fn empty_batch_succeeds() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let results = repo.find_raw_schema_views_by_paths(&[]).unwrap();
            assert!(results.is_empty());
        }

        /// Missing paths return None.
        ///
        /// Behavior: Paths not in `SCHEMA_ID_BY_PATH` return None.
        /// Verification: All results are None for non-existent paths.
        #[test]
        fn missing_paths_return_none() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let path1 = RelativePath::try_from("schemas/missing.json").unwrap();
            let path2 =
                RelativePath::try_from("schemas/also-missing.json").unwrap();

            let results =
                repo.find_raw_schema_views_by_paths(&[path1, path2]).unwrap();

            assert_eq!(results.len(), 2);
            assert!(results.first().and_then(Option::as_ref).is_none());
            assert!(results.get(1).and_then(Option::as_ref).is_none());
        }

        /// Partial found: mix of Some/None.
        ///
        /// Behavior: Some paths exist, others don't - preserves order.
        /// Verification: Existing path returns Some, missing returns None.
        #[test]
        fn partial_found_preserves_order() {
            use crate::schema::storage_v2::tables::{
                RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH,
            };

            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            let existing_path =
                RelativePath::try_from("schemas/note.json").unwrap();
            let missing_path =
                RelativePath::try_from("schemas/missing.json").unwrap();
            let id = SchemaId::new();

            let view = {
                let info = FileInfo::new(None, None, 100);
                let hashes = HashRecord::new(
                    Blake3Hash::new([1; 32]),
                    RawPropertyHashIndex::default(),
                );
                let raw = RawSchema {
                    version: RawSchemaVersion::default(),
                    name: "Note".into(),
                    extends: None,
                    excludes: vec![],
                    properties: RawPropertyMap::new(),
                    info: FileInfo::new(None, None, 0),
                };
                let version = SchemaVersion::new(info, hashes, &raw).unwrap();
                RawSchemaView::new(existing_path.clone(), version)
            };

            store
                .write(|tx| {
                    let mut path_table =
                        tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                    let mut view_table =
                        tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;

                    let path_str =
                        existing_path.as_path().to_string_lossy().to_string();
                    path_table.insert(path_str, serialize(&id)?.as_slice())?;
                    view_table.insert(id, serialize(&view)?.as_slice())?;

                    Ok(())
                })
                .unwrap();

            let repo = SchemaRedbRepository::new(store);
            let results = repo
                .find_raw_schema_views_by_paths(&[
                    existing_path.clone(),
                    missing_path,
                ])
                .unwrap();

            assert_eq!(results.len(), 2);
            assert!(results.first().and_then(Option::as_ref).is_some());
            assert!(results.get(1).and_then(Option::as_ref).is_none());

            let found = results.first().and_then(Option::as_ref).unwrap();
            assert_eq!(found.path(), &existing_path);
        }
    }
}
