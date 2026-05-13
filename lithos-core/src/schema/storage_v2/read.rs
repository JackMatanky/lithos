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
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(id)? else {
                    return Ok(None);
                };

                let view = deserialize(guard.value())?;
                Ok(Some(view))
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(path_table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };
                let Some(view_table) =
                    tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?
                else {
                    return Ok(None);
                };

                let path_key = path.as_path().to_string_lossy().to_string();
                let Some(id_guard) = path_table.get(path_key)? else {
                    return Ok(None);
                };
                let id: SchemaId = deserialize(id_guard.value())?;

                let Some(view_guard) = view_table.get(id)? else {
                    return Ok(None);
                };

                let view = deserialize(view_guard.value())?;
                Ok(Some(view))
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
    ) -> Result<crate::schema::index::NameIdPairs, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?
                else {
                    return Ok(crate::schema::index::NameIdPairs::new());
                };

                let mut pairs = crate::schema::index::NameIdPairs::new();
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
    ) -> Result<crate::schema::index::PathIdPairs, SchemaStorageV2Error> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(crate::schema::index::PathIdPairs::new());
                };

                let mut pairs = crate::schema::index::PathIdPairs::new();
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
            bank::PropertyBank,
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

    fn test_raw_view(path: &str, hash_byte: u8) -> RawSchemaView {
        let path = RelativePath::try_from(path).unwrap();
        let info = FileInfo::new(None, None, 100);
        let hashes = HashRecord::new(
            Blake3Hash::new([hash_byte; 32]),
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
        RawSchemaView::new(path, version)
    }

    mod by_id {
        use super::*;

        #[test]
        fn schema_roundtrip_and_missing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let missing = repo.find_schema_by_id(SchemaId::new()).unwrap();
            assert!(missing.is_none());

            let id = SchemaId::new();
            let schema = Schema::new(
                id,
                SchemaName::try_new("schema").unwrap(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            repo.save_schema(&schema).unwrap();

            let found = repo.find_schema_by_id(id).unwrap().unwrap();
            assert_eq!(found, schema);
        }

        #[test]
        fn many_schema_lookup_variants() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let s1 = Schema::new(
                SchemaId::new(),
                SchemaName::try_new("schema-1").unwrap(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            let s2 = Schema::new(
                SchemaId::new(),
                SchemaName::try_new("schema-2").unwrap(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            repo.save_schema(&s1).unwrap();
            repo.save_schema(&s2).unwrap();

            let all =
                repo.find_many_schemas_by_id(&[*s1.id(), *s2.id()]).unwrap();
            assert_eq!(all.len(), 2);
            assert_eq!(all.first().and_then(Option::as_ref), Some(&s1));
            assert_eq!(all.get(1).and_then(Option::as_ref), Some(&s2));

            let missing =
                repo.find_many_schemas_by_id(&[SchemaId::new()]).unwrap();
            assert!(missing.first().is_some_and(Option::is_none));

            let partial = repo
                .find_many_schemas_by_id(&[*s1.id(), SchemaId::new()])
                .unwrap();
            assert_eq!(partial.first().and_then(Option::as_ref), Some(&s1));
            assert!(partial.get(1).is_some_and(Option::is_none));

            let empty = repo.find_many_schemas_by_id(&[]).unwrap();
            assert!(empty.is_empty());
        }
    }

    mod property_bank {
        use super::*;

        #[test]
        fn property_bank_reads() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            assert!(repo.get_property_bank().unwrap().is_none());
            let bank = PropertyBank::new();
            repo.save_property_bank(&bank).unwrap();
            assert_eq!(repo.get_property_bank().unwrap().unwrap(), bank);

            let path = RelativePath::try_from("property-bank.toml").unwrap();
            assert!(repo.get_raw_property_bank_view(&path).unwrap().is_none());
        }
    }

    mod index_lookups {
        use super::*;
        use crate::schema::storage_v2::tables::{
            SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH,
        };

        #[test]
        fn name_path_batch_list_and_unified_index() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let name1 = SchemaName::try_from("note").unwrap();
            let name2 = SchemaName::try_from("task").unwrap();
            let path1 = RelativePath::try_from("schemas/note.json").unwrap();
            let path2 = RelativePath::try_from("schemas/task.json").unwrap();

            store
                .write(|tx| {
                    let mut name_table =
                        tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                    let mut path_table =
                        tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                    name_table.insert(
                        name1.as_str().to_owned(),
                        serialize(&id1)?.as_slice(),
                    )?;
                    name_table.insert(
                        name2.as_str().to_owned(),
                        serialize(&id2)?.as_slice(),
                    )?;
                    path_table.insert(
                        path1.as_path().to_string_lossy().to_string(),
                        serialize(&id1)?.as_slice(),
                    )?;
                    path_table.insert(
                        path2.as_path().to_string_lossy().to_string(),
                        serialize(&id2)?.as_slice(),
                    )?;
                    Ok(())
                })
                .unwrap();

            let repo = SchemaRedbRepository::new(store);
            assert_eq!(repo.find_schema_id_by_name(&name1).unwrap(), Some(id1));
            assert_eq!(repo.find_schema_id_by_path(&path1).unwrap(), Some(id1));

            let batch = repo
                .find_schema_ids_by_paths(&[
                    path1.clone(),
                    RelativePath::try_from("schemas/missing.json").unwrap(),
                    path2.clone(),
                ])
                .unwrap();
            assert_eq!(batch.first().copied().flatten(), Some(id1));
            assert!(batch.get(1).is_some_and(Option::is_none));
            assert_eq!(batch.get(2).copied().flatten(), Some(id2));

            let mut name_pairs =
                repo.list_schema_name_id_pairs().unwrap().into_vec();
            name_pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
            assert_eq!(name_pairs.len(), 2);

            let mut path_pairs =
                repo.list_schema_path_id_pairs().unwrap().into_vec();
            path_pairs.sort_by(|a, b| a.0.as_path().cmp(b.0.as_path()));
            assert_eq!(path_pairs.len(), 2);

            let index = repo.get_schema_index().unwrap();
            assert_eq!(index.get_id_by_name(&name1), Some(id1));
            assert_eq!(index.get_id_by_path(&path1), Some(id1));
        }
    }

    mod raw_views {
        use super::*;

        #[test]
        fn get_by_id_and_path_return_none_when_missing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            assert!(
                repo.get_raw_schema_view(SchemaId::new()).unwrap().is_none()
            );
            let missing_path =
                RelativePath::try_from("schemas/missing.json").unwrap();
            assert!(
                repo.find_raw_schema_view_by_path(&missing_path)
                    .unwrap()
                    .is_none()
            );
        }

        #[test]
        fn get_by_id_and_path_roundtrip_after_save() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id = SchemaId::new();
            let view = test_raw_view("schemas/note.json", 7);
            repo.save_raw_schema_view(id, &view).unwrap();

            let by_id = repo.get_raw_schema_view(id).unwrap();
            assert_eq!(by_id, Some(view.clone()));

            let by_path =
                repo.find_raw_schema_view_by_path(view.path()).unwrap();
            assert_eq!(by_path, Some(view));
        }

        #[test]
        fn by_path_returns_none_when_path_index_points_to_missing_view() {
            use crate::schema::storage_v2::tables::SCHEMA_ID_BY_PATH;

            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            let id = SchemaId::new();
            let path = RelativePath::try_from("schemas/orphan.json").unwrap();
            store
                .write(|tx| {
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
            assert!(
                repo.find_raw_schema_view_by_path(&path).unwrap().is_none()
            );
        }
    }
}
