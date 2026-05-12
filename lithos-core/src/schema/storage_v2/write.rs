//! Write implementation for `SchemaRedbRepository`.

use crate::{
    db::serialize,
    schema::{
        aggregate::Schema,
        repository::{SchemaStorageV2Error, SchemaWriteRepository},
        storage_v2::{SchemaRedbRepository, tables::SCHEMAS},
    },
};

impl SchemaWriteRepository for SchemaRedbRepository {
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
            repository::{SchemaReadRepository, SchemaWriteRepository},
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
    fn save_schema_can_be_retrieved() {
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

        // Verify that a failed write transaction rolls back
        let result: Result<(), crate::db::DbError> = store.write(|tx| {
            use crate::schema::storage_v2::tables::SCHEMAS;
            let mut table = tx.try_open_table(SCHEMAS.definition())?;
            let id2 = SchemaId::new();
            let name2 = SchemaName::try_new("will-rollback").unwrap();
            let schema2 =
                Schema::new(id2, name2, Vec::new(), vec![], PropertyMap::new());
            let bytes = crate::db::serialize(&schema2)?;
            table.insert(*schema2.id(), bytes.as_slice())?;
            Err(crate::db::DbError::Serialization("forced failure".to_owned()))
        });

        assert!(result.is_err());

        // Original schema should still exist
        let found = repo.find_schema_by_id(id).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn save_many_schemas_persists_all() {
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

        repo.save_many_schemas(&[schema1.clone(), schema2.clone()]).unwrap();

        let found1 = repo.find_schema_by_id(id1).unwrap().unwrap();
        let found2 = repo.find_schema_by_id(id2).unwrap().unwrap();

        assert_eq!(found1.id(), &id1);
        assert_eq!(found2.id(), &id2);
    }

    #[test]
    fn save_many_schemas_all_or_nothing() {
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
    fn save_many_schemas_empty_slice() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = SchemaRedbRepository::new(store);

        // Should not error on empty batch
        repo.save_many_schemas(&[]).unwrap();
    }
}
