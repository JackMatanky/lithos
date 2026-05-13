//! Write implementation for `SchemaRedbRepository`.

use redb::ReadableTable;

use crate::{
    db::serialize,
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        repository::{SchemaStorageV2Error, SchemaWriteRepository},
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

impl SchemaWriteRepository for SchemaRedbRepository {
    #[inline]
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageV2Error> {
        let bytes = serialize(schema).map_err(SchemaStorageV2Error::from)?;
        let id_bytes =
            serialize(schema.id()).map_err(SchemaStorageV2Error::from)?;

        self.store
            .write(|tx| {
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                table.insert(*schema.id(), bytes.as_slice())?;

                let mut name_table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                let name_key = schema.name().as_str().to_owned();
                name_table.insert(name_key, id_bytes.as_slice())?;

                Ok(())
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
                let mut name_table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;

                for schema in schemas {
                    let bytes = serialize(schema)?;
                    let id_bytes = serialize(schema.id())?;

                    table.insert(*schema.id(), bytes.as_slice())?;

                    let name_key = schema.name().as_str().to_owned();
                    name_table.insert(name_key, id_bytes.as_slice())?;
                }
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaStorageV2Error> {
        let bytes = serialize(bank).map_err(SchemaStorageV2Error::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(PROPERTY_BANK.definition())?;
                table.insert(PROPERTY_BANK_KEY.to_owned(), bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaStorageV2Error> {
        let bytes = serialize(view).map_err(SchemaStorageV2Error::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_PROPERTY_BANK_VIEW.definition())?;
                let path_str = path.as_path().to_string_lossy().to_string();
                table.insert(path_str, bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: crate::schema::identifier::SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaStorageV2Error> {
        let view_bytes = serialize(view).map_err(SchemaStorageV2Error::from)?;
        let id_bytes = serialize(&id).map_err(SchemaStorageV2Error::from)?;

        self.store
            .write(|tx| {
                let mut view_table =
                    tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;

                if let Some(existing) = view_table.get(id)? {
                    let existing_view = crate::db::deserialize::<RawSchemaView>(
                        existing.value(),
                    )?;
                    if existing_view.path() != view.path() {
                        let stale_key = existing_view
                            .path()
                            .as_path()
                            .to_string_lossy()
                            .to_string();
                        let _ = path_table.remove(stale_key)?;
                    }
                }

                view_table.insert(id, view_bytes.as_slice())?;
                let path_key =
                    view.path().as_path().to_string_lossy().to_string();
                path_table.insert(path_key, id_bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaStorageV2Error::from)
    }
}

#[cfg(test)]
mod tests {
    mod save_schema {
        use std::sync::Arc;

        use crate::{
            db::Store,
            schema::{
                aggregate::Schema,
                identifier::{SchemaId, SchemaName},
                property::PropertyMap,
                repository::{SchemaReadRepository, SchemaWriteRepository},
                storage_v2::SchemaRedbRepository,
            },
        };

        #[test]
        fn persists_data() {
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
        fn can_be_retrieved() {
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
        fn rolls_back_on_error() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store.clone());

            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

            repo.save_schema(&schema).unwrap();

            // Verify that a failed write transaction rolls back
            let result: Result<(), crate::db::DbError> = store.write(|tx| {
                use crate::schema::storage_v2::tables::SCHEMAS;
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                let id2 = SchemaId::new();
                let name2 = SchemaName::try_new("will-rollback").unwrap();
                let schema2 = Schema::new(
                    id2,
                    name2,
                    Vec::new(),
                    vec![],
                    PropertyMap::new(),
                );
                let bytes = crate::db::serialize(&schema2)?;
                table.insert(*schema2.id(), bytes.as_slice())?;
                Err(crate::db::DbError::Serialization(
                    "forced failure".to_owned(),
                ))
            });

            assert!(result.is_err());

            // Original schema should still exist
            let found = repo.find_schema_by_id(id).unwrap();
            assert!(found.is_some());
        }

        #[test]
        fn updates_name_index() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id = SchemaId::new();
            let name = SchemaName::try_from("test-schema-index").unwrap();
            let schema = Schema::new(
                id,
                name.clone(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );

            repo.save_schema(&schema).unwrap();

            let found_id = repo.find_schema_id_by_name(&name).unwrap();
            assert!(found_id.is_some());
            assert_eq!(found_id.unwrap(), id);
        }
    }

    mod save_many_schemas {
        use std::sync::Arc;

        use crate::{
            db::Store,
            schema::{
                aggregate::Schema,
                identifier::{SchemaId, SchemaName},
                property::PropertyMap,
                repository::{SchemaReadRepository, SchemaWriteRepository},
                storage_v2::SchemaRedbRepository,
            },
        };

        #[test]
        fn persists_all() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let name1 = SchemaName::try_new("schema-1").unwrap();
            let name2 = SchemaName::try_new("schema-2").unwrap();

            let schema1 = Schema::new(
                id1,
                name1.clone(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            let schema2 = Schema::new(
                id2,
                name2.clone(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );

            repo.save_many_schemas(&[schema1.clone(), schema2.clone()])
                .unwrap();

            let found1 = repo.find_schema_by_id(id1).unwrap().unwrap();
            let found2 = repo.find_schema_by_id(id2).unwrap().unwrap();

            assert_eq!(found1, schema1);
            assert_eq!(found2, schema2);

            // Verify name index was updated for both
            assert_eq!(repo.find_schema_id_by_name(&name1).unwrap(), Some(id1));
            assert_eq!(repo.find_schema_id_by_name(&name2).unwrap(), Some(id2));
        }

        #[test]
        fn all_or_nothing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let name1 = SchemaName::try_new("schema-1").unwrap();
            let name2 = SchemaName::try_new("schema-2").unwrap();

            let schema1 =
                Schema::new(id1, name1, Vec::new(), vec![], PropertyMap::new());
            let schema2 =
                Schema::new(id2, name2, Vec::new(), vec![], PropertyMap::new());

            repo.save_many_schemas(&[schema1, schema2]).unwrap();

            let results = repo.find_many_schemas_by_id(&[id1, id2]).unwrap();

            assert_eq!(results.len(), 2);
            assert!(results.first().is_some_and(Option::is_some));
            assert!(results.get(1).is_some_and(Option::is_some));
        }

        #[test]
        fn empty_slice() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            // Should not error on empty batch
            repo.save_many_schemas(&[]).unwrap();
        }
    }

    mod save_property_bank {
        use std::sync::Arc;

        use crate::{
            db::Store,
            schema::{
                bank::PropertyBank,
                repository::{SchemaReadRepository, SchemaWriteRepository},
                storage_v2::SchemaRedbRepository,
            },
        };

        #[test]
        fn persists() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let bank = PropertyBank::new();
            repo.save_property_bank(&bank).unwrap();

            // Verify it was saved by retrieving it
            let retrieved = repo.get_property_bank().unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), bank);
        }

        #[test]
        fn overwrites_previous() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            // Save first bank
            let bank1 = PropertyBank::new();
            repo.save_property_bank(&bank1).unwrap();

            // Save second bank (new instance)
            let bank2 = PropertyBank::new();
            repo.save_property_bank(&bank2).unwrap();

            // Should successfully overwrite (singleton behavior)
            let retrieved = repo.get_property_bank().unwrap();
            assert!(retrieved.is_some());
        }
    }

    mod save_raw_schema_view {
        use std::sync::Arc;

        use crate::{
            db::Store,
            fs::{FileInfo, RelativePath},
            schema::{
                identifier::SchemaId,
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

        #[test]
        fn persists_view_and_path_index() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id = SchemaId::new();
            let view = test_raw_view("schemas/note.json", 9);
            repo.save_raw_schema_view(id, &view).unwrap();

            assert_eq!(
                repo.get_raw_schema_view(id).unwrap(),
                Some(view.clone())
            );
            assert_eq!(
                repo.find_raw_schema_view_by_path(view.path()).unwrap(),
                Some(view)
            );
        }

        #[test]
        fn updates_path_index_when_path_changes() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = SchemaRedbRepository::new(store);

            let id = SchemaId::new();
            let old_view = test_raw_view("schemas/old.json", 3);
            let new_view = test_raw_view("schemas/new.json", 4);

            repo.save_raw_schema_view(id, &old_view).unwrap();
            repo.save_raw_schema_view(id, &new_view).unwrap();

            assert_eq!(
                repo.get_raw_schema_view(id).unwrap(),
                Some(new_view.clone())
            );
            assert!(
                repo.find_raw_schema_view_by_path(old_view.path())
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                repo.find_raw_schema_view_by_path(new_view.path()).unwrap(),
                Some(new_view)
            );
        }
    }
}
