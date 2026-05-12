//! Read-only schema repository operations.

use redb::ReadableTable;

use crate::{
    db::{UuidTableReadExt, deserialize},
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        identifier::{SchemaId, SchemaName},
        repository::{SchemaReadRepository, SchemaStorageV2Error},
        storage_v2::{
            SchemaRedbRepository,
            tables::{
                PROPERTY_BANK, PROPERTY_BANK_KEY, RAW_PROPERTY_BANK_VIEW,
                RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH,
                SCHEMAS,
            },
        },
        views::{RawPropertyBankView, RawSchemaView},
    },
};

impl SchemaReadRepository for SchemaRedbRepository {
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
        paths: &[RelativePath],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaStorageV2Error> {
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

    #[inline]
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(PROPERTY_BANK.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(PROPERTY_BANK_KEY.to_owned())?
                else {
                    return Ok(None);
                };

                let bank = deserialize(guard.value())?;
                Ok(Some(bank))
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(RAW_PROPERTY_BANK_VIEW.definition())?
                else {
                    return Ok(None);
                };

                let path_str = path.as_path().to_string_lossy().to_string();
                let Some(guard) = table.get(path_str)? else {
                    return Ok(None);
                };

                let view = deserialize(guard.value())?;
                Ok(Some(view))
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?
                else {
                    return Ok(None);
                };

                let key = name.as_str().to_owned();
                let Some(guard) = table.get(key)? else {
                    return Ok(None);
                };

                let id = deserialize(guard.value())?;
                Ok(Some(id))
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };

                let key = path.as_path().to_string_lossy().to_string();
                let Some(guard) = table.get(key)? else {
                    return Ok(None);
                };

                let id = deserialize(guard.value())?;
                Ok(Some(id))
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<SchemaId>>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(paths.iter().map(|_| None).collect());
                };

                let mut results = Vec::with_capacity(paths.len());
                for path in paths {
                    let key = path.as_path().to_string_lossy().to_string();
                    match table.get(key)? {
                        Some(guard) => {
                            let id = deserialize(guard.value())?;
                            results.push(Some(id));
                        }
                        None => results.push(None),
                    }
                }
                Ok(results)
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<Vec<(SchemaName, SchemaId)>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut pairs = Vec::new();
                for result in table.iter()? {
                    let (k_guard, v_guard) = result?;
                    let name_str = k_guard.value();
                    let name = SchemaName::try_from(name_str.as_str())
                        .map_err(|e| {
                            crate::db::DbError::Deserialization(e.to_string())
                        })?;
                    let id = deserialize(v_guard.value())?;
                    pairs.push((name, id));
                }
                Ok(pairs)
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<Vec<(RelativePath, SchemaId)>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut pairs = Vec::new();
                for result in table.iter()? {
                    let (k_guard, v_guard) = result?;
                    let path_str = k_guard.value();
                    let path = RelativePath::try_from(path_str.as_str())
                        .map_err(|e| {
                            crate::db::DbError::Deserialization(e.to_string())
                        })?;
                    let id = deserialize(v_guard.value())?;
                    pairs.push((path, id));
                }
                Ok(pairs)
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn get_schema_index(
        &self,
    ) -> Result<crate::schema::index::SchemaIndex, SchemaStorageV2Error> {
        let name_pairs = self.list_schema_name_id_pairs()?;
        let path_pairs = self.list_schema_path_id_pairs()?;

        crate::schema::index::SchemaIndex::from_pairs(name_pairs, path_pairs)
            .map_err(|e| {
                SchemaStorageV2Error::from(crate::db::DbError::Deserialization(
                    e.to_string(),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::{Store, serialize},
        fs::{FileInfo, RelativePath},
        schema::{
            aggregate::Schema,
            identifier::{SchemaId, SchemaName},
            property::PropertyMap,
            raw::{RawPropertyMap, RawSchema, RawSchemaVersion},
            repository::{SchemaReadRepository, SchemaWriteRepository},
            storage_v2::SchemaRedbRepository,
            views::{
                RawPropertyHashIndex, SchemaVersion, hashes::HashRecord,
                raw::RawSchemaView,
            },
        },
        support::Blake3Hash,
    };

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

        // Use write repository to save
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

        #[test]
        fn finds_views_by_paths_in_single_transaction() {
            use crate::schema::storage_v2::tables::{
                RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH,
            };

            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

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

                    let path1_str =
                        path1.as_path().to_string_lossy().to_string();
                    let path2_str =
                        path2.as_path().to_string_lossy().to_string();
                    path_table
                        .insert(path1_str, serialize(&id1)?.as_slice())?;
                    path_table
                        .insert(path2_str, serialize(&id2)?.as_slice())?;

                    view_table.insert(id1, serialize(&view1)?.as_slice())?;
                    view_table.insert(id2, serialize(&view2)?.as_slice())?;

                    Ok(())
                })
                .unwrap();

            let repo = SchemaRedbRepository::new(store);
            let results = repo
                .find_raw_schema_views_by_paths(&[path1.clone(), path2.clone()])
                .unwrap();

            assert_eq!(results.len(), 2);
            assert!(results.first().and_then(Option::as_ref).is_some());
            assert!(results.get(1).and_then(Option::as_ref).is_some());

            let found1 = results.first().and_then(Option::as_ref).unwrap();
            let found2 = results.get(1).and_then(Option::as_ref).unwrap();
            assert_eq!(found1.path(), &path1);
            assert_eq!(found2.path(), &path2);
        }

        #[test]
        fn empty_batch_succeeds() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let results = repo.find_raw_schema_views_by_paths(&[]).unwrap();
            assert!(results.is_empty());
        }

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

    #[test]
    fn get_property_bank_returns_none_when_not_saved() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let result = repo.get_property_bank().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_property_bank_returns_saved_bank() {
        use crate::schema::{
            bank::PropertyBank, repository::SchemaWriteRepository,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let bank = PropertyBank::new();
        repo.save_property_bank(&bank).unwrap();

        let result = repo.get_property_bank().unwrap();
        assert!(result.is_some());
        let retrieved = result.unwrap();
        assert_eq!(retrieved, bank);
    }

    #[test]
    fn get_raw_property_bank_view_returns_none_when_not_saved() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let path = RelativePath::try_from("property-bank.toml").unwrap();
        let result = repo.get_raw_property_bank_view(&path).unwrap();
        assert!(result.is_none());
    }

    // Note: get_raw_property_bank_view is tested via integration tests that
    // exercise the full staleness detection pipeline. Unit testing here would
    // require exposing internal HashRecord construction which violates
    // encapsulation. The None case is covered by the simple test above.

    #[test]
    fn find_schema_id_by_name_returns_none_when_not_saved() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let name = SchemaName::try_from("note").unwrap();
        let result = repo.find_schema_id_by_name(&name).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_schema_id_by_name_returns_id_after_save() {
        use crate::{
            db::serialize, schema::storage_v2::tables::SCHEMA_ID_BY_NAME,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let schema_id = SchemaId::new();
        let name = SchemaName::try_from("note").unwrap();

        // Manually insert into SCHEMA_ID_BY_NAME table to test read path
        store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                let key = name.as_str().to_owned();
                let value_bytes = serialize(&schema_id)?;
                table.insert(key, value_bytes.as_slice())?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let result = repo.find_schema_id_by_name(&name).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), schema_id);
    }

    #[test]
    fn find_schema_id_by_path_returns_none_when_not_saved() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        let path = RelativePath::try_from("schemas/note.json").unwrap();
        let result = repo.find_schema_id_by_path(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_schema_id_by_path_returns_id_after_save() {
        use crate::{
            db::serialize, schema::storage_v2::tables::SCHEMA_ID_BY_PATH,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let schema_id = SchemaId::new();
        let path = RelativePath::try_from("schemas/note.json").unwrap();

        // Manually insert into SCHEMA_ID_BY_PATH table to test read path
        store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                let key = path.as_path().to_string_lossy().to_string();
                let value_bytes = serialize(&schema_id)?;
                table.insert(key, value_bytes.as_slice())?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let result = repo.find_schema_id_by_path(&path).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), schema_id);
    }

    #[test]
    fn find_schema_ids_by_paths_returns_ids_and_nones() {
        use crate::{
            db::serialize, schema::storage_v2::tables::SCHEMA_ID_BY_PATH,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let id1 = SchemaId::new();
        let path1 = RelativePath::try_from("schemas/one.json").unwrap();
        let id2 = SchemaId::new();
        let path2 = RelativePath::try_from("schemas/two.json").unwrap();
        let path3 = RelativePath::try_from("schemas/missing.json").unwrap();

        store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                table.insert(
                    path1.as_path().to_string_lossy().to_string(),
                    serialize(&id1)?.as_slice(),
                )?;
                table.insert(
                    path2.as_path().to_string_lossy().to_string(),
                    serialize(&id2)?.as_slice(),
                )?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let results =
            repo.find_schema_ids_by_paths(&[path1, path3, path2]).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results.first().unwrap(), &Some(id1));
        assert_eq!(results.get(1).unwrap(), &None);
        assert_eq!(results.get(2).unwrap(), &Some(id2));
    }

    #[test]
    fn list_schema_name_id_pairs_returns_all_pairs() {
        use crate::{
            db::serialize, schema::storage_v2::tables::SCHEMA_ID_BY_NAME,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let id1 = SchemaId::new();
        let name1 = SchemaName::try_from("schema1").unwrap();
        let id2 = SchemaId::new();
        let name2 = SchemaName::try_from("schema2").unwrap();

        store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                table.insert(
                    name1.as_str().to_owned(),
                    serialize(&id1)?.as_slice(),
                )?;
                table.insert(
                    name2.as_str().to_owned(),
                    serialize(&id2)?.as_slice(),
                )?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let mut results = repo.list_schema_name_id_pairs().unwrap();

        // Sort to ensure consistent test comparison since DB iteration order
        // might vary
        results.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

        assert_eq!(results.len(), 2);
        assert_eq!(results.first().unwrap().0, name1);
        assert_eq!(results.first().unwrap().1, id1);
        assert_eq!(results.get(1).unwrap().0, name2);
        assert_eq!(results.get(1).unwrap().1, id2);
    }

    #[test]
    fn list_schema_path_id_pairs_returns_all_pairs() {
        use crate::{
            db::serialize, schema::storage_v2::tables::SCHEMA_ID_BY_PATH,
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let id1 = SchemaId::new();
        let path1 = RelativePath::try_from("schemas/schema1.json").unwrap();
        let id2 = SchemaId::new();
        let path2 = RelativePath::try_from("schemas/schema2.json").unwrap();

        store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                table.insert(
                    path1.as_path().to_string_lossy().to_string(),
                    serialize(&id1)?.as_slice(),
                )?;
                table.insert(
                    path2.as_path().to_string_lossy().to_string(),
                    serialize(&id2)?.as_slice(),
                )?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let mut results = repo.list_schema_path_id_pairs().unwrap();

        // Sort to ensure consistent test comparison
        results.sort_by(|a, b| a.0.as_path().cmp(b.0.as_path()));

        assert_eq!(results.len(), 2);
        assert_eq!(results.first().unwrap().0, path1);
        assert_eq!(results.first().unwrap().1, id1);
        assert_eq!(results.get(1).unwrap().0, path2);
        assert_eq!(results.get(1).unwrap().1, id2);
    }

    #[test]
    fn get_schema_index_returns_unified_index() {
        use crate::{
            db::serialize,
            schema::storage_v2::tables::{
                SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH,
            },
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());

        let id = SchemaId::new();
        let name = SchemaName::try_from("note").unwrap();
        let path = RelativePath::try_from("schemas/note.json").unwrap();

        store
            .write(|tx| {
                let mut name_table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                name_table.insert(
                    name.as_str().to_owned(),
                    serialize(&id)?.as_slice(),
                )?;

                let mut path_table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                path_table.insert(
                    path.as_path().to_string_lossy().to_string(),
                    serialize(&id)?.as_slice(),
                )?;
                Ok(())
            })
            .unwrap();

        let repo = SchemaRedbRepository::new(store);
        let index = repo.get_schema_index().unwrap();

        assert_eq!(index.get_id_by_name(&name), Some(id));
        assert_eq!(index.get_id_by_path(&path), Some(id));
    }
}
