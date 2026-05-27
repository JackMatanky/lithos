//! [`WriteRepository`] trait implementation for [`RedbRepository`].
//!
//! Provides write operations for schema persistence backed by `redb`. All
//! writes execute within atomic transactions with automatic rollback on error.
//!
//! # Atomicity Guarantees
//!
//! - **Single transaction per method**: Each write method opens one transaction
//!   via `Store::write()`. If any table operation fails, the entire transaction
//!   rolls back automatically.
//! - **Multi-table coordination**: Methods like `save_schema` atomically update
//!   both the schema aggregate and its indexes ([`SCHEMA_ID_BY_NAME`]).
//! - **Batch operations**: `save_many_schemas` wraps all saves in a single
//!   transaction for atomicity.
//!
//! # Cross-Table Invariants
//!
//! - `save_schema`: Maintains [`SCHEMAS`] ↔ [`SCHEMA_ID_BY_NAME`] consistency
//! - `delete_schema`: Removes schema aggregate + all related indexes (name,
//!   path) and raw view in a single transaction
//!
//! # Rollback Behavior
//!
//! If serialization or table write fails, the transaction is automatically
//! rolled back by `redb`. No partial writes are visible to concurrent readers.
//!
//! # Helper Functions
//!
//! - [`load_delete_context`]: Loads schema name/path before deletion
//! - [`remove_schema`], [`remove_name_id_index`], [`remove_path_id_index`],
//!   [`remove_raw_schema_view`]: Atomic delete operations on individual tables
//!
//! [`WriteRepository`]: crate::schema::repository::WriteRepository
//! [`RedbRepository`]: crate::schema::storage::RedbRepository
//! [`SCHEMAS`]: crate::schema::storage::tables::SCHEMAS
//! [`SCHEMA_ID_BY_NAME`]: crate::schema::storage::tables::SCHEMA_ID_BY_NAME

use redb::ReadableTable;

use crate::{
    db::{ArchivedEntity, DbError},
    fs::PathKey,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaRepositoryError,
        identifier::SchemaName,
        inheritance::InheritanceGraph,
        repository::WriteRepository,
        storage::{
            RedbRepository,
            tables::{
                PROPERTY_BANK, PROPERTY_BANK_KEY, RAW_PROPERTY_BANK_VIEW,
                RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH,
                SCHEMA_TOPOLOGICAL_GRAPH, SCHEMAS, TOPOLOGICAL_GRAPH_KEY,
            },
        },
        views::{RawPropertyBankView, RawSchemaView},
    },
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_schema(
        &self,
        schema: &Schema,
    ) -> Result<(), SchemaRepositoryError> {
        let bytes = schema.to_bytes().map_err(SchemaRepositoryError::from)?;
        let id_bytes =
            schema.id().to_bytes().map_err(SchemaRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                table.insert(*schema.id(), bytes.as_slice())?;

                let mut name_table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                name_table
                    .insert(schema.name().as_str(), id_bytes.as_slice())?;

                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaRepositoryError> {
        self.store
            .write(|tx| {
                let mut table = tx.try_open_table(SCHEMAS.definition())?;
                let mut name_table =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;

                for schema in schemas {
                    let bytes = schema.to_bytes()?;
                    let id_bytes = schema.id().to_bytes()?;

                    table.insert(*schema.id(), bytes.as_slice())?;

                    name_table
                        .insert(schema.name().as_str(), id_bytes.as_slice())?;
                }
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaRepositoryError> {
        let bytes = bank.to_bytes().map_err(SchemaRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(PROPERTY_BANK.definition())?;
                table.insert(PROPERTY_BANK_KEY, bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &PathKey,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaRepositoryError> {
        let bytes = view.to_bytes().map_err(SchemaRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_PROPERTY_BANK_VIEW.definition())?;
                table.insert(path, bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: crate::schema::identifier::SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaRepositoryError> {
        let view_bytes =
            view.to_bytes().map_err(SchemaRepositoryError::from)?;
        let id_bytes = id.to_bytes().map_err(SchemaRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut view_table =
                    tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;

                if let Some(existing) = view_table.get(id)? {
                    let existing_view =
                        RawSchemaView::from_bytes(existing.value())?;
                    if existing_view.path() != view.path() {
                        let _ = path_table.remove(existing_view.path())?;
                    }
                }

                view_table.insert(id, view_bytes.as_slice())?;
                path_table.insert(view.path(), id_bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), SchemaRepositoryError> {
        let bytes = graph.to_bytes().map_err(SchemaRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(SCHEMA_TOPOLOGICAL_GRAPH.definition())?;
                table.insert(TOPOLOGICAL_GRAPH_KEY, bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn delete_schema(
        &self,
        id: crate::schema::identifier::SchemaId,
    ) -> Result<(), SchemaRepositoryError> {
        self.store
            .write(|tx| {
                let ctx = load_delete_context(tx, id)?;
                remove_name_id_index(
                    tx,
                    ctx.schema_name.as_ref().map(SchemaName::as_str),
                )?;
                remove_path_id_index(tx, ctx.view_path.as_ref())?;
                remove_schema(tx, id)?;
                remove_raw_schema_view(tx, id)?;
                Ok(())
            })
            .map_err(SchemaRepositoryError::from)
    }
}

struct DeleteContext {
    schema_name: Option<SchemaName>,
    view_path: Option<PathKey>,
}

/// Loads schema name and path for deletion context.
///
/// Queries [`SCHEMAS`] and [`RAW_SCHEMA_VIEWS`] tables to extract the schema
/// name and file path needed for index cleanup during deletion.
///
/// Returns `None` for name/path if the corresponding table entry is missing.
/// This gracefully handles partial deletion (e.g., schema exists but no raw
/// view).
///
/// [`SCHEMAS`]: crate::schema::storage::tables::SCHEMAS
/// [`RAW_SCHEMA_VIEWS`]: crate::schema::storage::tables::RAW_SCHEMA_VIEWS
fn load_delete_context(
    tx: &crate::db::WriteTx,
    id: crate::schema::identifier::SchemaId,
) -> Result<DeleteContext, DbError> {
    let schemas = tx.try_open_table(SCHEMAS.definition())?;
    let raw_views = tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;

    let schema_name = if let Some(schema_guard) = schemas.get(id)? {
        let schema = Schema::from_bytes(schema_guard.value())?;
        Some(schema.name().clone())
    } else {
        None
    };

    let view_path = if let Some(view_guard) = raw_views.get(id)? {
        let view = RawSchemaView::from_bytes(view_guard.value())?;
        Some(view.path().clone())
    } else {
        None
    };

    Ok(DeleteContext {
        schema_name,
        view_path,
    })
}

/// Removes schema aggregate from [`SCHEMAS`] table.
///
/// Idempotent: returns `Ok(())` if schema ID does not exist.
///
/// [`SCHEMAS`]: crate::schema::storage::tables::SCHEMAS
fn remove_schema(
    tx: &crate::db::WriteTx,
    id: crate::schema::identifier::SchemaId,
) -> Result<(), DbError> {
    let mut schemas = tx.try_open_table(SCHEMAS.definition())?;
    let _ = schemas.remove(id)?;
    Ok(())
}

/// Removes name-to-ID index entry from [`SCHEMA_ID_BY_NAME`].
///
/// No-op if `schema_name` is `None` (e.g., schema was partially saved).
///
/// [`SCHEMA_ID_BY_NAME`]: crate::schema::storage::tables::SCHEMA_ID_BY_NAME
fn remove_name_id_index(
    tx: &crate::db::WriteTx,
    schema_name: Option<&str>,
) -> Result<(), DbError> {
    let mut name_index = tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;

    if let Some(name) = schema_name {
        let _ = name_index.remove(name)?;
    }

    Ok(())
}

/// Removes path-to-ID index entry from [`SCHEMA_ID_BY_PATH`].
///
/// No-op if `view_path` is `None` (e.g., no raw view exists).
///
/// [`SCHEMA_ID_BY_PATH`]: crate::schema::storage::tables::SCHEMA_ID_BY_PATH
fn remove_path_id_index(
    tx: &crate::db::WriteTx,
    view_path: Option<&PathKey>,
) -> Result<(), DbError> {
    let mut path_index = tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;

    if let Some(path) = view_path {
        let _ = path_index.remove(path)?;
    }

    Ok(())
}

/// Removes raw schema view from [`RAW_SCHEMA_VIEWS`] table.
///
/// Idempotent: returns `Ok(())` if view does not exist.
///
/// [`RAW_SCHEMA_VIEWS`]: crate::schema::storage::tables::RAW_SCHEMA_VIEWS
fn remove_raw_schema_view(
    tx: &crate::db::WriteTx,
    id: crate::schema::identifier::SchemaId,
) -> Result<(), DbError> {
    let mut raw_views = tx.try_open_table(RAW_SCHEMA_VIEWS.definition())?;
    let _ = raw_views.remove(id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    mod save_schema {
        use std::sync::Arc;

        use crate::{
            db::{ArchivedEntity, Store},
            schema::{
                aggregate::Schema,
                identifier::{SchemaId, SchemaName},
                property::PropertyMap,
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
            },
        };

        #[test]
        fn persists_data() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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
            let repo = RedbRepository::new(store);

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
            let repo = RedbRepository::new(store.clone());

            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

            repo.save_schema(&schema).unwrap();

            // Verify that a failed write transaction rolls back
            let result: Result<(), crate::db::DbError> = store.write(|tx| {
                use crate::schema::storage::tables::SCHEMAS;
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
                let bytes = schema2.to_bytes()?;
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
            let repo = RedbRepository::new(store);

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
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
            },
        };

        #[test]
        fn persists_all() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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

            WriteRepository::save_many_schemas(&repo, &[
                schema1.clone(),
                schema2.clone(),
            ])
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
            let repo = RedbRepository::new(store);

            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let name1 = SchemaName::try_new("schema-1").unwrap();
            let name2 = SchemaName::try_new("schema-2").unwrap();

            let schema1 =
                Schema::new(id1, name1, Vec::new(), vec![], PropertyMap::new());
            let schema2 =
                Schema::new(id2, name2, Vec::new(), vec![], PropertyMap::new());

            WriteRepository::save_many_schemas(&repo, &[schema1, schema2])
                .unwrap();

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
            let repo = RedbRepository::new(store);

            // Should not error on empty batch
            WriteRepository::save_many_schemas(&repo, &[]).unwrap();
        }
    }

    mod save_property_bank {
        use std::sync::Arc;

        use crate::{
            db::Store,
            schema::{
                bank::PropertyBank,
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
            },
        };

        #[test]
        fn persists() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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
            let repo = RedbRepository::new(store);

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
            fs::{
                PathKey,
                metadata::{FileMetadata, FsTimes},
            },
            schema::{
                identifier::SchemaId,
                raw::{RawPropertyMap, RawSchema, RawSchemaVersion},
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
                views::{
                    RawPropertyHashIndex, SchemaVersion, hashes::HashRecord,
                    raw::RawSchemaView,
                },
            },
            support::Blake3Hash,
        };

        fn test_raw_view(path: &str, hash_byte: u8) -> RawSchemaView {
            let path = PathKey::try_new(path).unwrap();
            let info = FileMetadata::new(FsTimes::new(None, None), 100, false);
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
                metadata: FileMetadata::new(FsTimes::new(None, None), 0, false),
            };
            let version = SchemaVersion::new(info, hashes, &raw).unwrap();
            RawSchemaView::new(path, version)
        }

        fn key(path: &str) -> PathKey {
            PathKey::try_new(path).expect("valid path key")
        }

        #[test]
        fn persists_view_and_path_index() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let id = SchemaId::new();
            let view = test_raw_view("schemas/note.json", 9);
            repo.save_raw_schema_view(id, &view).unwrap();

            assert_eq!(
                repo.get_raw_schema_view(id).unwrap(),
                Some(view.clone())
            );
            assert_eq!(
                repo.find_raw_schema_view_by_path(&key("schemas/note.json"))
                    .unwrap(),
                Some(view)
            );
        }

        #[test]
        fn updates_path_index_when_path_changes() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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
                repo.find_raw_schema_view_by_path(&key("schemas/old.json"))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                repo.find_raw_schema_view_by_path(&key("schemas/new.json"))
                    .unwrap(),
                Some(new_view)
            );
        }
    }

    mod save_topological_graph {
        use std::sync::Arc;

        use crate::{
            db::Store,
            schema::{
                identifier::SchemaId,
                inheritance::{InheritanceGraph, SchemaGraphBuilder},
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
            },
        };

        fn build_graph(
            root: SchemaId,
            child: SchemaId,
        ) -> InheritanceGraph<()> {
            let mut builder = SchemaGraphBuilder::new();
            builder.add_node(root, ());
            builder.add_node(child, ());
            builder.add_parent(child, root);
            InheritanceGraph::try_from(builder.build()).unwrap()
        }

        #[test]
        fn persists_across_reopen() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            let root = SchemaId::new();
            let child = SchemaId::new();
            {
                let store = Arc::new(Store::open(&db_path).unwrap());
                let repo = RedbRepository::new(store);
                let graph = build_graph(root, child);
                repo.save_topological_graph(&graph).unwrap();
            }

            let reopened_store = Arc::new(Store::open(&db_path).unwrap());
            let reopened_repo = RedbRepository::new(reopened_store);
            let loaded =
                reopened_repo.get_topological_graph().unwrap().unwrap();

            assert_eq!(loaded.parents_of(child), &[root]);
            assert_eq!(loaded.children_of(root), &[child]);
        }
    }

    mod delete_schema {
        use std::sync::Arc;

        use crate::{
            db::Store,
            fs::{
                PathKey,
                metadata::{FileMetadata, FsTimes},
            },
            schema::{
                aggregate::Schema,
                identifier::{SchemaId, SchemaName},
                property::PropertyMap,
                raw::{RawPropertyMap, RawSchema, RawSchemaVersion},
                repository::{ReadRepository, WriteRepository},
                storage::RedbRepository,
                views::{
                    RawPropertyHashIndex, SchemaVersion, hashes::HashRecord,
                    raw::RawSchemaView,
                },
            },
            support::Blake3Hash,
        };

        fn test_raw_view(path: &str, hash_byte: u8) -> RawSchemaView {
            let path = PathKey::try_new(path).unwrap();
            let info = FileMetadata::new(FsTimes::new(None, None), 100, false);
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
                metadata: FileMetadata::new(FsTimes::new(None, None), 0, false),
            };
            let version = SchemaVersion::new(info, hashes, &raw).unwrap();
            RawSchemaView::new(path, version)
        }

        #[test]
        fn removes_schema_indexes_and_raw_view() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-delete").unwrap();
            let path = PathKey::try_new("schemas/delete.json").unwrap();
            let schema = Schema::new(
                id,
                name.clone(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            let view = test_raw_view(path.as_str(), 11);

            repo.save_schema(&schema).unwrap();
            repo.save_raw_schema_view(id, &view).unwrap();

            repo.delete_schema(id).unwrap();

            assert!(repo.find_schema_by_id(id).unwrap().is_none());
            assert!(repo.get_raw_schema_view(id).unwrap().is_none());
            assert!(repo.find_schema_id_by_name(&name).unwrap().is_none());
            assert!(
                repo.find_schema_id_by_path(
                    &PathKey::try_new("schemas/delete.json").unwrap()
                )
                .unwrap()
                .is_none()
            );
        }

        #[test]
        fn deleting_missing_schema_is_idempotent() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let missing = SchemaId::new();
            repo.delete_schema(missing).unwrap();
            repo.delete_schema(missing).unwrap();
        }

        #[test]
        fn removes_name_index_even_without_raw_view() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let id = SchemaId::new();
            let name = SchemaName::try_new("schema-name-only").unwrap();
            let schema = Schema::new(
                id,
                name.clone(),
                Vec::new(),
                vec![],
                PropertyMap::new(),
            );
            repo.save_schema(&schema).unwrap();

            repo.delete_schema(id).unwrap();

            assert!(repo.find_schema_id_by_name(&name).unwrap().is_none());
            assert!(repo.find_schema_by_id(id).unwrap().is_none());
        }
    }
}
