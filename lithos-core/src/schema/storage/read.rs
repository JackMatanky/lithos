//! [`ReadRepository`] trait implementation for [`RedbRepository`].
//!
//! Provides read-only schema persistence operations backed by `redb`. All
//! methods execute within independent read transactions managed by the
//! [`Store`].
//!
//! # Transaction Boundaries
//!
//! Each method call opens a new read transaction via `Store::read()`. Methods
//! like `find_raw_schema_views_by_paths` batch multiple lookups into a single
//! transaction for efficiency.
//!
//! # Table Access
//!
//! Uses table definitions from [`crate::schema::storage::tables`]:
//! - [`SCHEMAS`]: Schema aggregates by ID
//! - [`RAW_SCHEMA_VIEWS`]: Raw views by ID
//! - [`SCHEMA_ID_BY_NAME`], [`SCHEMA_ID_BY_PATH`]: Name/path indexes
//! - [`PROPERTY_BANK`], [`SCHEMA_TOPOLOGICAL_GRAPH`]: Singletons
//!
//! # Helper Functions
//!
//! - [`parse_schema_name_key`]: Validates and converts name index keys
//! - [`parse_path_key`]: Validates and converts path index keys
//!
//! These helpers provide structured error messages with context when index
//! keys fail validation (e.g., "invalid schema-name index key 'bad name'").
//!
//! [`ReadRepository`]: crate::schema::repository::ReadRepository
//! [`RedbRepository`]: crate::schema::storage::RedbRepository
//! [`Store`]: crate::db::Store
//! [`SCHEMAS`]: crate::schema::storage::tables::SCHEMAS
//! [`RAW_SCHEMA_VIEWS`]: crate::schema::storage::tables::RAW_SCHEMA_VIEWS
//! [`SCHEMA_ID_BY_NAME`]: crate::schema::storage::tables::SCHEMA_ID_BY_NAME
//! [`SCHEMA_ID_BY_PATH`]: crate::schema::storage::tables::SCHEMA_ID_BY_PATH
//! [`PROPERTY_BANK`]: crate::schema::storage::tables::PROPERTY_BANK
//! [`SCHEMA_TOPOLOGICAL_GRAPH`]: crate::schema::storage::tables::SCHEMA_TOPOLOGICAL_GRAPH

use std::collections::{HashMap, HashSet};

use redb::ReadableTable;

use crate::{
    db::{ArchivedEntity, UuidTableReadExt},
    fs::PathKey,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaRepositoryError,
        identifier::{SchemaId, SchemaName},
        index::{NameIdPairs, PathIdPairs},
        property::PropertyName,
        repository::ReadRepository,
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

impl ReadRepository for RedbRepository {
    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(SCHEMAS.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(id)? else {
                    return Ok(None);
                };

                let schema = Schema::from_bytes(guard.value())?;
                Ok(Some(schema))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaRepositoryError> {
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
                            .map(|g| Schema::from_bytes(g.value()))
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(results)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, SchemaRepositoryError> {
        self.find_many_schemas_by_id(ids)
            .map(|schemas| schemas.into_iter().flatten().collect())
    }

    #[inline]
    fn list_schemas(&self) -> Result<Vec<Schema>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(SCHEMAS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut schemas = Vec::new();
                for result in table.iter()? {
                    let (_id_guard, schema_guard) = result?;
                    schemas.push(Schema::from_bytes(schema_guard.value())?);
                }

                Ok(schemas)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<HashMap<SchemaId, Vec<PropertyName>>, SchemaRepositoryError>
    {
        let target_names: HashSet<&str> =
            property_names.iter().map(PropertyName::as_str).collect();

        let mut usage = HashMap::new();
        for schema in self.list_schemas()? {
            let mut matching = Vec::new();
            for name in schema.properties().keys() {
                if target_names.contains(name.as_str()) {
                    matching.push(name.clone());
                }
            }
            if !matching.is_empty() {
                usage.insert(*schema.id(), matching);
            }
        }

        Ok(usage)
    }

    #[inline]
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, SchemaRepositoryError> {
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

                let view = RawSchemaView::from_bytes(guard.value())?;
                Ok(Some(view))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawSchemaView>, SchemaRepositoryError> {
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

                let Some(id_guard) = path_table.get(path)? else {
                    return Ok(None);
                };
                let id = SchemaId::from_bytes(id_guard.value())?;

                let Some(view_guard) = view_table.get(id)? else {
                    return Ok(None);
                };

                let view = RawSchemaView::from_bytes(view_guard.value())?;
                Ok(Some(view))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaRepositoryError> {
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
                    let id_guard = path_table.get(path)?;

                    let view = if let Some(id_bytes) = id_guard {
                        // Step 2: SchemaId → RawSchemaView lookup
                        let id = SchemaId::from_bytes(id_bytes.value())?;
                        let view_guard = view_table.get(&id)?;
                        view_guard
                            .map(|g| RawSchemaView::from_bytes(g.value()))
                            .transpose()?
                    } else {
                        None
                    };

                    results.push(view);
                }
                Ok(results)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(PROPERTY_BANK.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(PROPERTY_BANK_KEY)? else {
                    return Ok(None);
                };

                let bank = PropertyBank::from_bytes(guard.value())?;
                Ok(Some(bank))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<
        Option<crate::schema::inheritance::InheritanceGraph<()>>,
        SchemaRepositoryError,
    > {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_TOPOLOGICAL_GRAPH.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(TOPOLOGICAL_GRAPH_KEY)? else {
                    return Ok(None);
                };

                let graph = crate::schema::inheritance::InheritanceGraph::<()>::from_bytes(guard.value())?;
                Ok(Some(graph))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawPropertyBankView>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(RAW_PROPERTY_BANK_VIEW.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(path)? else {
                    return Ok(None);
                };

                let view = RawPropertyBankView::from_bytes(guard.value())?;
                Ok(Some(view))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(name.as_str())? else {
                    return Ok(None);
                };

                let id = SchemaId::from_bytes(guard.value())?;
                Ok(Some(id))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<SchemaId>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(path)? else {
                    return Ok(None);
                };

                let id = SchemaId::from_bytes(guard.value())?;
                Ok(Some(id))
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<SchemaId>>, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(paths.iter().map(|_| None).collect());
                };

                let mut results = Vec::with_capacity(paths.len());
                for path in paths {
                    match table.get(path)? {
                        Some(guard) => {
                            let id = SchemaId::from_bytes(guard.value())?;
                            results.push(Some(id));
                        }
                        None => results.push(None),
                    }
                }
                Ok(results)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<NameIdPairs, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?
                else {
                    return Ok(NameIdPairs::new());
                };

                let mut pairs = NameIdPairs::new();
                for result in table.iter()? {
                    let (k_guard, v_guard) = result?;
                    let name = parse_schema_name_key(k_guard.value())?;
                    let id = SchemaId::from_bytes(v_guard.value())?;
                    pairs.push((name, id));
                }
                Ok(pairs)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<PathIdPairs, SchemaRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?
                else {
                    return Ok(PathIdPairs::new());
                };

                let mut pairs = PathIdPairs::new();
                for result in table.iter()? {
                    let (k_guard, v_guard) = result?;
                    let path = parse_path_key(k_guard.value().as_str())?;
                    let id = SchemaId::from_bytes(v_guard.value())?;
                    pairs.push((path, id));
                }
                Ok(pairs)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn get_schema_index(
        &self,
    ) -> Result<crate::schema::index::SchemaIndex, SchemaRepositoryError> {
        let name_pairs = self.list_schema_name_id_pairs()?;
        let path_pairs = self.list_schema_path_id_pairs()?;

        crate::schema::index::SchemaIndex::from_pairs(name_pairs, path_pairs)
            .map_err(|e| {
                SchemaRepositoryError::from(
                    crate::db::DbError::Deserialization(e.to_string()),
                )
            })
    }
}

/// Parses and validates a schema-name index key.
///
/// Converts a raw string key from [`SCHEMA_ID_BY_NAME`] table into a validated
/// [`SchemaName`]. Returns a descriptive error if the key violates schema
/// naming rules (e.g., contains spaces, invalid characters).
///
/// # Errors
///
/// Returns [`DbError::Deserialization`] with context if the key is invalid.
/// Error message includes the invalid key for debugging (e.g.,
/// `"invalid schema-name index key 'bad name': ..."`).
///
/// # Example Error
///
/// ```text
/// invalid schema-name index key 'my schema': schema names cannot contain spaces
/// ```
///
/// [`SCHEMA_ID_BY_NAME`]: crate::schema::storage::tables::SCHEMA_ID_BY_NAME
/// [`DbError::Deserialization`]: crate::db::DbError::Deserialization
#[inline]
fn parse_schema_name_key(key: &str) -> Result<SchemaName, crate::db::DbError> {
    SchemaName::try_from(key).map_err(|error| {
        crate::db::DbError::Deserialization(format!(
            "invalid schema-name index key '{key}': {error}"
        ))
    })
}

/// Parses and validates a schema-path index key.
///
/// Converts a raw string key from [`SCHEMA_ID_BY_PATH`] table into a validated
/// [`PathKey`]. Returns a descriptive error if the key violates path
/// constraints (e.g., empty string, absolute path).
///
/// # Errors
///
/// Returns [`DbError::Deserialization`] with context if the key is invalid.
/// Error message includes the invalid key for debugging (e.g.,
/// `"invalid schema-path index key '': ..."`).
///
/// # Example Error
///
/// ```text
/// invalid schema-path index key '': path cannot be empty
/// ```
///
/// [`SCHEMA_ID_BY_PATH`]: crate::schema::storage::tables::SCHEMA_ID_BY_PATH
/// [`DbError::Deserialization`]: crate::db::DbError::Deserialization
#[inline]
fn parse_path_key(key: &str) -> Result<PathKey, crate::db::DbError> {
    PathKey::try_new(key).map_err(|error| {
        crate::db::DbError::Deserialization(format!(
            "invalid schema-path index key '{key}': {error}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::{ArchivedEntity, Store},
        fs::{
            PathKey,
            metadata::{FileMetadata, FsTimes},
        },
        schema::{
            aggregate::Schema,
            bank::PropertyBank,
            identifier::{SchemaId, SchemaName},
            inheritance::{InheritanceGraph, SchemaGraphBuilder},
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

    fn key(path: &str) -> PathKey {
        PathKey::try_new(path).expect("valid path key")
    }

    mod by_id {
        use super::*;

        #[test]
        fn schema_roundtrip_and_missing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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
            let repo = RedbRepository::new(store);

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

    mod schema_collections {
        use super::*;

        #[test]
        fn find_schemas_by_ids_skips_missing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

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

            let found = repo
                .find_schemas_by_ids(&[*s1.id(), SchemaId::new(), *s2.id()])
                .unwrap();
            assert_eq!(found, vec![s1, s2]);
        }

        #[test]
        fn list_schemas_empty_and_non_empty() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            assert!(repo.list_schemas().unwrap().is_empty());

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

            repo.save_many_schemas(&[s1.clone(), s2.clone()]).unwrap();

            let mut listed = repo.list_schemas().unwrap();
            listed.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
            assert_eq!(listed, vec![s1, s2]);
        }
    }

    mod property_usage {
        use super::*;
        use crate::schema::{
            property::{
                Multiplicity, Optionality, Property, PropertyId, PropertyName,
            },
            property_spec::{BoolSpec, PropertySpec},
        };

        #[test]
        fn finds_schemas_using_target_properties() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let status = PropertyName::try_new("status").unwrap();
            let owner = PropertyName::try_new("owner").unwrap();
            let other = PropertyName::try_new("other").unwrap();

            let mut p1 = PropertyMap::new();
            p1.insert(
                status.clone(),
                Property::new(
                    PropertyId::new(),
                    Optionality::Required,
                    Multiplicity::Single,
                    PropertySpec::Bool(BoolSpec::default()),
                ),
            );

            let mut p2 = PropertyMap::new();
            p2.insert(
                owner.clone(),
                Property::new(
                    PropertyId::new(),
                    Optionality::Required,
                    Multiplicity::Single,
                    PropertySpec::Bool(BoolSpec::default()),
                ),
            );

            let s1 = Schema::new(
                SchemaId::new(),
                SchemaName::try_new("schema-1").unwrap(),
                Vec::new(),
                vec![],
                p1,
            );
            let s2 = Schema::new(
                SchemaId::new(),
                SchemaName::try_new("schema-2").unwrap(),
                Vec::new(),
                vec![],
                p2,
            );

            repo.save_many_schemas(&[s1.clone(), s2]).unwrap();

            let usage = repo
                .find_schemas_using_properties(&[status.clone(), other])
                .unwrap();
            assert_eq!(usage.get(s1.id()), Some(&vec![status]));
        }

        #[test]
        fn returns_empty_when_no_matches() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let missing = PropertyName::try_new("missing").unwrap();
            let usage = repo.find_schemas_using_properties(&[missing]).unwrap();
            assert!(usage.is_empty());
        }
    }

    mod property_bank {
        use super::*;

        #[test]
        fn property_bank_reads() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            assert!(repo.get_property_bank().unwrap().is_none());
            let bank = PropertyBank::new();
            repo.save_property_bank(&bank).unwrap();
            assert_eq!(repo.get_property_bank().unwrap().unwrap(), bank);

            let path = key("property-bank.toml");
            assert!(repo.get_raw_property_bank_view(&path).unwrap().is_none());
        }
    }

    mod index_lookups {
        use super::*;
        use crate::schema::storage::tables::{
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
            let path1 = PathKey::try_new("schemas/note.json").unwrap();
            let path2 = PathKey::try_new("schemas/task.json").unwrap();

            store
                .write(|tx| {
                    let mut name_table =
                        tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                    let mut path_table =
                        tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                    name_table
                        .insert(name1.as_str(), id1.to_bytes()?.as_slice())?;
                    name_table
                        .insert(name2.as_str(), id2.to_bytes()?.as_slice())?;
                    path_table.insert(&path1, id1.to_bytes()?.as_slice())?;
                    path_table.insert(&path2, id2.to_bytes()?.as_slice())?;
                    Ok(())
                })
                .unwrap();

            let repo = RedbRepository::new(store);
            assert_eq!(repo.find_schema_id_by_name(&name1).unwrap(), Some(id1));
            assert_eq!(
                repo.find_schema_id_by_path(&key("schemas/note.json")).unwrap(),
                Some(id1)
            );

            let batch = repo
                .find_schema_ids_by_paths(&[
                    key("schemas/note.json"),
                    key("schemas/missing.json"),
                    key("schemas/task.json"),
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
            path_pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
            assert_eq!(path_pairs.len(), 2);

            let index = repo.get_schema_index().unwrap();
            assert_eq!(index.get_id_by_name(&name1), Some(id1));
            assert_eq!(index.get_id_by_path(&path1), Some(id1));
        }

        #[test]
        fn list_schema_name_id_pairs_reports_context_for_invalid_name_key() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            store
                .write(|tx| {
                    let mut name_table =
                        tx.try_open_table(SCHEMA_ID_BY_NAME.definition())?;
                    name_table.insert(
                        "bad name with spaces",
                        SchemaId::new().to_bytes()?.as_slice(),
                    )?;
                    Ok(())
                })
                .unwrap();

            let repo = RedbRepository::new(store);
            let err = repo.list_schema_name_id_pairs().unwrap_err();
            let msg = err.to_string();

            assert!(msg.contains("invalid schema-name index key"));
            assert!(msg.contains("bad name with spaces"));
        }
    }

    mod raw_views {
        use super::*;

        #[test]
        fn get_by_id_and_path_return_none_when_missing() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            assert!(
                repo.get_raw_schema_view(SchemaId::new()).unwrap().is_none()
            );
            let _missing_path =
                PathKey::try_new("schemas/missing.json").unwrap();
            assert!(
                repo.find_raw_schema_view_by_path(&key("schemas/missing.json"))
                    .unwrap()
                    .is_none()
            );
        }

        #[test]
        fn get_by_id_and_path_roundtrip_after_save() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let id = SchemaId::new();
            let view = test_raw_view("schemas/note.json", 7);
            repo.save_raw_schema_view(id, &view).unwrap();

            let by_id = repo.get_raw_schema_view(id).unwrap();
            assert_eq!(by_id, Some(view.clone()));

            let by_path = repo
                .find_raw_schema_view_by_path(&key("schemas/note.json"))
                .unwrap();
            assert_eq!(by_path, Some(view));
        }

        #[test]
        fn by_path_returns_none_when_path_index_points_to_missing_view() {
            use crate::schema::storage::tables::SCHEMA_ID_BY_PATH;

            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());

            let id = SchemaId::new();
            let path = PathKey::try_new("schemas/orphan.json").unwrap();
            store
                .write(|tx| {
                    let mut path_table =
                        tx.try_open_table(SCHEMA_ID_BY_PATH.definition())?;
                    path_table.insert(&path, id.to_bytes()?.as_slice())?;
                    Ok(())
                })
                .unwrap();

            let repo = RedbRepository::new(store);
            assert!(
                repo.find_raw_schema_view_by_path(&key("schemas/orphan.json"))
                    .unwrap()
                    .is_none()
            );
        }
    }

    mod topology_graph {
        use super::*;

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
        fn returns_none_when_not_saved() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            assert!(repo.get_topological_graph().unwrap().is_none());
        }

        #[test]
        fn roundtrip_and_overwrite_preserve_structure() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let root1 = SchemaId::new();
            let child1 = SchemaId::new();
            let graph1 = build_graph(root1, child1);
            repo.save_topological_graph(&graph1).unwrap();

            let loaded1 = repo.get_topological_graph().unwrap().unwrap();
            assert_eq!(loaded1.parents_of(child1), &[root1]);
            assert_eq!(loaded1.children_of(root1), &[child1]);

            let root2 = SchemaId::new();
            let child2 = SchemaId::new();
            let graph2 = build_graph(root2, child2);
            repo.save_topological_graph(&graph2).unwrap();

            let loaded2 = repo.get_topological_graph().unwrap().unwrap();
            assert_eq!(loaded2.parents_of(child2), &[root2]);
            assert_eq!(loaded2.children_of(root2), &[child2]);
            assert!(loaded2.parents_of(child1).is_empty());
        }
    }

    mod batch_path_lookups {
        use super::*;

        #[test]
        fn schema_read_repository_preserves_order_for_path_lookups() {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);

            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let view1 = test_raw_view("schemas/note.json", 7);
            let view2 = test_raw_view("schemas/task.json", 8);
            let _missing = PathKey::try_new("schemas/missing.json").unwrap();

            repo.save_raw_schema_view(id1, &view1).unwrap();
            repo.save_raw_schema_view(id2, &view2).unwrap();

            let paths = vec![
                key("schemas/note.json"),
                key("schemas/missing.json"),
                key("schemas/task.json"),
            ];

            let id_hits =
                ReadRepository::find_schema_ids_by_paths(&repo, &paths)
                    .unwrap();
            assert_eq!(id_hits, vec![Some(id1), None, Some(id2)]);

            let view_hits =
                ReadRepository::find_raw_schema_views_by_paths(&repo, &paths)
                    .unwrap();
            assert_eq!(view_hits, vec![Some(view1), None, Some(view2)]);
        }
    }
}
