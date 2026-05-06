//! Unified repository trait and implementation for schema persistence.
//!
//! This module provides both the [`Repository`] trait and its concrete
//! implementation [`RedbRepository`], replacing the previous CQRS Command/Query
//! pattern.
//!
//! # Architecture
//!
//! Following the unified Repository pattern from the architecture guide:
//! - Single trait combining reads and writes
//! - Zero-copy access via closure-based methods
//! - Concrete `RedbRepository` using redb for persistence

use std::{collections::HashMap, sync::Arc};

use super::{
    aggregate::Schema,
    bank::PropertyBank,
    error::{SchemaRepositoryError, SchemaStorageError},
    identifier::{SchemaId, SchemaName},
    index::{NameIdPairs, PathIdPairs, SchemaIndex},
    inheritance::InheritanceGraph,
    property::PropertyName,
    views::{RawPropertyBankView, RawSchemaView, RawView as _},
};
use crate::{
    db::{BatchReader, Database},
    fs::RelativePath,
};

fn map_db_error(error: crate::db::DbError) -> SchemaRepositoryError {
    SchemaRepositoryError::Storage(SchemaStorageError::Storage(error))
}

/// Schema-to-properties usage map: `schema_id` → Vec<`property_name`>.
///
/// Used by `find_schemas_using_properties()` to return which schemas use which
/// properties.
pub type SchemaPropertyUsage = HashMap<SchemaId, Vec<PropertyName>>;

/// Batch reader adapter for schema tables.
pub trait BatchSchemaReader {
    /// Storage-specific error type for batch reads.
    type Error;

    /// Gets the raw schema view for a given schema ID.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, Self::Error>;

    /// Gets the topological graph singleton.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, Self::Error>;
}

struct RedbBatchSchemaReader<'reader> {
    reader: &'reader BatchReader,
}

impl BatchSchemaReader for RedbBatchSchemaReader<'_> {
    type Error = SchemaRepositoryError;

    #[inline]
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, Self::Error> {
        use crate::schema::db_table::RAW_SCHEMA_VIEWS;

        self.reader
            .get_owned_by_uuid::<RawSchemaView>(
                RAW_SCHEMA_VIEWS,
                id.into_uuid(),
            )
            .map_err(map_db_error)
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, Self::Error> {
        use crate::schema::db_table::{
            SCHEMA_TOPOLOGICAL_GRAPH, TOPOLOGICAL_GRAPH_KEY,
        };

        self.reader
            .get_owned::<InheritanceGraph<()>>(
                SCHEMA_TOPOLOGICAL_GRAPH,
                TOPOLOGICAL_GRAPH_KEY,
            )
            .map_err(map_db_error)
    }
}

/// Unified repository trait for schema domain persistence.
///
/// Combines read and write operations in a single trait, following the
/// unified Repository pattern from the architecture guide.
///
/// # Type Parameters
///
/// - `Error`: Storage-specific error type
///
/// # Naming Conventions
///
/// Following the naming taxonomy from `docs/refs/rust/naming-taxonomy.md`:
/// - **find_***: Optional reads (returns `Option<T>`)
/// - **get_***: Required singleton reads
/// - **list_***: Multiple item reads (returns `Vec<T>`)
/// - **is_***: Boolean checks
/// - **save**, **delete**: Write operations
/// - **with_***: Zero-copy closure-based access
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::Repository;
///
/// fn example<R: Repository>(repo: &R) -> Result<(), R::Error> {
///     // Find optional schema
///     if let Some(schema) = repo.find_schema_by_id(id)? {
///         println!("Found: {}", schema.name);
///     }
///
///     // List all schemas
///     let schemas = repo.list_schemas()?;
///
///     // Save schemas
///     repo.save_schemas(&schemas)?;
///
///     Ok(())
/// }
/// ```
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped by category for better maintainability"
)]
pub trait Repository: Send + Sync {
    /// Storage-specific error type.
    type Error: std::error::Error + Send + Sync;

    // ========================================================================
    // Schema Read Operations
    // ========================================================================

    /// Finds a schema by ID.
    ///
    /// Returns `None` if the schema does not exist.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, Self::Error>;

    /// Finds a schema ID by name.
    ///
    /// Returns `None` if no schema with the given name exists.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error>;

    /// Finds multiple schemas by IDs.
    ///
    /// Returns only the schemas that exist. Missing schemas are silently
    /// skipped.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, Self::Error>;

    /// Lists all schemas.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_schemas(&self) -> Result<Vec<Schema>, Self::Error>;

    /// Lists schema name-to-ID pairs.
    ///
    /// Useful for building name lookup tables without loading full schema data.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_schema_name_id_pairs(&self) -> Result<NameIdPairs, Self::Error>;

    /// Lists schema path-to-ID pairs.
    ///
    /// Useful for discovery stage to detect deleted schemas without loading
    /// full schema data.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_schema_path_id_pairs(&self) -> Result<PathIdPairs, Self::Error>;

    /// Gets a unified index of all schemas.
    ///
    /// The index provides O(1) lookups by name, ID, and path. It is derived
    /// from the repository's path and name tables.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the index construction fails.
    fn get_schema_index(&self) -> Result<SchemaIndex, Self::Error>;

    // ========================================================================
    // Property Bank Read Operations
    // ========================================================================

    /// Gets the property bank singleton.
    ///
    /// Returns `None` if the property bank has not been initialized.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>;

    /// Finds schemas that use any of the given property names.
    ///
    /// Returns a map of `schema_id` → Vec<`property_name`> for schemas
    /// that reference at least one of the given properties.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<SchemaPropertyUsage, Self::Error>;

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Saves multiple schemas atomically.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_schemas(&self, schemas: &[&Schema]) -> Result<(), Self::Error>;

    /// Saves the property bank singleton.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error>;

    /// Deletes a schema by ID.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the deletion fails.
    fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error>;

    // ========================================================================
    // Raw View Operations (for staleness detection)
    // ========================================================================

    /// Gets the raw schema view for a given schema ID.
    ///
    /// Returns `None` if no view exists (schema never loaded).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, Self::Error>;

    /// Saves a raw schema view.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), Self::Error>;

    /// Gets the raw property bank view.
    ///
    /// Returns `None` if no view exists (bank never loaded).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, Self::Error>;

    /// Saves the raw property bank view.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), Self::Error>;

    /// Finds a raw schema view by file path.
    ///
    /// Returns `None` if no view exists for the given path.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_raw_schema_view_by_path(
        &self,
        file_path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, Self::Error>;

    /// Finds the `SchemaId` for a file path.
    ///
    /// Returns `None` if no schema exists at that path.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_id_by_path(
        &self,
        file_path: &RelativePath,
    ) -> Result<Option<SchemaId>, Self::Error>;

    /// Finds multiple raw schema views by file paths (bulk query).
    ///
    /// More efficient than N individual queries as it performs a single
    /// transaction. Returns a map of path → view for paths that have views.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_raw_schema_views_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error>;

    /// Finds multiple schema IDs by file paths (bulk query).
    ///
    /// More efficient than N individual queries as it performs a single
    /// transaction. Returns a map of path → `SchemaId` for paths that have
    /// schemas.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_ids_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error>;

    /// Gets the topological graph singleton.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, Self::Error>;

    /// Saves the topological graph singleton.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), Self::Error>;

    // ========================================================================
    // Batch Operations (for complex multi-table queries)
    // ========================================================================

    /// Provides access to a batch reader for complex multi-table queries.
    ///
    /// This is a lower-level API for operations that need to read from
    /// multiple tables in a single transaction.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn with_batch_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>;

    /// Provides access to a batch reader scoped to schema tables.
    ///
    /// This is a convenience wrapper that keeps schema pipelines from
    /// depending on raw table names.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn with_batch_schema_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        for<'reader> F: FnOnce(
            &'reader dyn BatchSchemaReader<Error = Self::Error>,
        ) -> Result<R, Self::Error>;
}

// ========================================================================
// RedbRepository Implementation
// ========================================================================

/// Production repository implementation using redb.
pub struct RedbRepository {
    db: Arc<Database>,
}

impl RedbRepository {
    /// Creates a new `RedbRepository` with the given database.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::schema::RedbRepository;
    /// use redb::Database;
    ///
    /// let db = Database::create("schemas.db")?;
    /// let repo = RedbRepository::new(db);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
        }
    }
}

impl Repository for RedbRepository {
    type Error = SchemaRepositoryError;

    // ========================================================================
    // Schema Read Operations
    // ========================================================================
    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        self.db
            .get_owned_by_uuid::<Schema>(SCHEMA_BY_ID, id.into_uuid())
            .map_err(map_db_error)
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_NAME;

        self.db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())
            .map_err(map_db_error)
    }

    #[inline]
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        ids.iter()
            .filter_map(|id| {
                match self
                    .db
                    .get_owned_by_uuid::<Schema>(SCHEMA_BY_ID, id.into_uuid())
                {
                    Ok(Some(schema)) => Some(Ok(schema)),
                    Ok(None) => None,
                    Err(e) => Some(Err(map_db_error(e))),
                }
            })
            .collect()
    }

    #[inline]
    fn list_schemas(&self) -> Result<Vec<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        let pairs: Vec<(String, Schema)> =
            self.db.list_key_value_pairs(SCHEMA_BY_ID).map_err(map_db_error)?;

        Ok(pairs.into_iter().map(|(_id, schema)| schema).collect())
    }

    #[inline]
    fn list_schema_name_id_pairs(&self) -> Result<NameIdPairs, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_NAME;

        let pairs: Vec<_> = self
            .db
            .list_key_value_pairs(SCHEMA_ID_BY_NAME)
            .map_err(map_db_error)?
            .into_iter()
            .map(|(name_str, id): (String, SchemaId)| {
                SchemaName::try_new(&name_str)
                    .map(|name| (name, id))
                    .map_err(SchemaRepositoryError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(pairs.into())
    }

    #[inline]
    fn list_schema_path_id_pairs(&self) -> Result<PathIdPairs, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_PATH;

        let pairs: Vec<_> = self
            .db
            .list_key_value_pairs(SCHEMA_ID_BY_PATH)
            .map_err(map_db_error)?
            .into_iter()
            .map(|(path_str, id): (String, SchemaId)| {
                RelativePath::try_from(path_str.as_str()).map(|path| (path, id))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                SchemaRepositoryError::Storage(SchemaStorageError::Corruption {
                    reason: format!("invalid schema path in index: {error}")
                        .into(),
                })
            })?;

        Ok(pairs.into())
    }

    #[inline]
    fn get_schema_index(&self) -> Result<SchemaIndex, Self::Error> {
        use crate::schema::db_table::{SCHEMA_ID_BY_NAME, SCHEMA_ID_BY_PATH};

        self.db
            .batch_read(|reader| {
                let name_pairs: Vec<_> = reader
                    .list_key_value_pairs::<SchemaId>(SCHEMA_ID_BY_NAME)?
                    .into_iter()
                    .map(|(name_str, id)| {
                        SchemaName::try_new(&name_str)
                            .map(|name| (name, id))
                            .map_err(|e| {
                                crate::db::DbError::Database(e.to_string())
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let path_pairs: Vec<_> = reader
                    .list_key_value_pairs::<SchemaId>(SCHEMA_ID_BY_PATH)?
                    .into_iter()
                    .map(|(path_str, id)| {
                        RelativePath::try_from(path_str.as_str())
                            .map(|path| (path, id))
                            .map_err(|e| {
                                crate::db::DbError::Database(e.to_string())
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                SchemaIndex::from_pairs(name_pairs, path_pairs)
                    .map_err(|e| crate::db::DbError::Database(e.to_string()))
            })
            .map_err(map_db_error)
    }

    // ========================================================================
    // Property Bank Read Operations
    // ========================================================================

    #[inline]
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        use crate::schema::db_table::{PROPERTY_BANK, PROPERTY_BANK_KEY};

        self.db
            .get_owned::<PropertyBank>(PROPERTY_BANK, PROPERTY_BANK_KEY)
            .map_err(map_db_error)
    }

    #[inline]
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<SchemaPropertyUsage, Self::Error> {
        use std::collections::{HashMap, HashSet};

        // Convert property names to a set for fast lookup
        let target_names: HashSet<&str> =
            property_names.iter().map(PropertyName::as_str).collect();

        // Scan all schemas and check which properties they use
        let mut usage = HashMap::new();
        let schemas = self.list_schemas()?;

        for schema in schemas {
            let mut matching_properties = Vec::new();

            for name in schema.properties().keys() {
                if target_names.contains(name.as_str()) {
                    matching_properties.push(name.clone());
                }
            }

            if !matching_properties.is_empty() {
                usage.insert(*schema.id(), matching_properties);
            }
        }

        Ok(usage)
    }

    // ========================================================================
    // Write Operations
    // ========================================================================

    #[inline]
    fn save_schemas(&self, schemas: &[&Schema]) -> Result<(), Self::Error> {
        use crate::schema::db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME};

        self.db
            .batch_write(|batch| {
                for schema in schemas {
                    // Save schema by ID
                    batch.put_by_uuid(
                        SCHEMA_BY_ID,
                        schema.id().into_uuid(),
                        *schema,
                    )?;

                    // Save name → ID mapping
                    batch.put(
                        SCHEMA_ID_BY_NAME,
                        schema.name().as_str(),
                        schema.id(),
                    )?;
                }
                Ok(())
            })
            .map_err(map_db_error)?;

        Ok(())
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{PROPERTY_BANK, PROPERTY_BANK_KEY};

        self.db
            .batch_write(|batch| {
                batch.put(PROPERTY_BANK, PROPERTY_BANK_KEY, bank)?;
                Ok(())
            })
            .map_err(map_db_error)?;

        Ok(())
    }

    #[inline]
    fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error> {
        use crate::schema::db_table::{
            RAW_SCHEMA_VIEWS, SCHEMA_BY_ID, SCHEMA_ID_BY_NAME,
            SCHEMA_ID_BY_PATH,
        };

        let schema = self.find_schema_by_id(id)?;
        let view = self.get_raw_schema_view(id)?;

        self.db
            .batch_write(|batch| {
                batch.delete_by_uuid(SCHEMA_BY_ID, id.into_uuid())?;
                batch.delete_by_uuid(RAW_SCHEMA_VIEWS, id.into_uuid())?;
                if let Some(schema) = schema.as_ref() {
                    batch.delete(SCHEMA_ID_BY_NAME, schema.name().as_str())?;
                }
                if let Some(view) = view.as_ref() {
                    let path_key = view.file_path().as_path().to_string_lossy();
                    batch.delete(SCHEMA_ID_BY_PATH, path_key.as_ref())?;
                }
                Ok(())
            })
            .map_err(map_db_error)?;

        Ok(())
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

    #[inline]
    fn with_batch_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>,
    {
        // Adapt the closure to convert SchemaRepositoryError -> DbError for
        // batch_read, then convert DbError -> SchemaRepositoryError.
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Explicitly matching all SchemaRepositoryError variants \
                      would be fragile and unnecessary - we only care about \
                      storage variant"
        )]
        let result = self.db.batch_read(|reader| {
            f(reader).map_err(|schema_err| {
                // Extract DbError if it's a Storage variant, otherwise create a
                // generic error.
                match schema_err {
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Storage(db_err),
                    ) => db_err,
                    _ => {
                        // This shouldn't happen in practice since f should only
                        // return Storage errors when using the reader
                        crate::db::DbError::Database(schema_err.to_string())
                    }
                }
            })
        });

        result.map_err(map_db_error)
    }

    #[inline]
    fn with_batch_schema_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        for<'reader> F: FnOnce(
            &'reader dyn BatchSchemaReader<Error = Self::Error>,
        ) -> Result<R, Self::Error>,
    {
        self.with_batch_reader(|reader| {
            let schema_reader = RedbBatchSchemaReader {
                reader,
            };
            f(&schema_reader)
        })
    }

    // ========================================================================
    // Raw View Operations
    // ========================================================================

    #[inline]
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, Self::Error> {
        use crate::schema::db_table::RAW_SCHEMA_VIEWS;

        self.db
            .get_owned_by_uuid(RAW_SCHEMA_VIEWS, id.into_uuid())
            .map_err(map_db_error)
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH};

        self.db
            .batch_write(|batch| {
                let path_key = view.file_path().as_path().to_string_lossy();
                batch.put_by_uuid(RAW_SCHEMA_VIEWS, id.into_uuid(), view)?;
                batch.put(SCHEMA_ID_BY_PATH, path_key.as_ref(), &id)?;
                Ok(())
            })
            .map_err(map_db_error)?;

        Ok(())
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, Self::Error> {
        use crate::schema::db_table::RAW_PROPERTY_BANK_VIEW;

        self.db
            .get_owned(
                RAW_PROPERTY_BANK_VIEW,
                path.as_path().to_string_lossy().as_ref(),
            )
            .map_err(map_db_error)
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::RAW_PROPERTY_BANK_VIEW;

        self.db
            .put(
                RAW_PROPERTY_BANK_VIEW,
                path.as_path().to_string_lossy().as_ref(),
                view,
            )
            .map_err(map_db_error)
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        file_path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_PATH;

        let path_key = file_path.as_path().to_string_lossy();

        // First lookup SchemaId by path
        let id = self
            .db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_PATH, path_key.as_ref())
            .map_err(map_db_error)?;

        match id {
            Some(id) => self.get_raw_schema_view(id),
            None => Ok(None),
        }
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        file_path: &RelativePath,
    ) -> Result<Option<SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_PATH;

        let path_key = file_path.as_path().to_string_lossy();

        self.db
            .get_owned::<SchemaId>(SCHEMA_ID_BY_PATH, path_key.as_ref())
            .map_err(map_db_error)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error> {
        use crate::schema::db_table::{RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH};

        // Perform all queries in a single read transaction
        self.db
            .batch_read(|reader| {
                let mut results = HashMap::new();

                for path in file_paths {
                    let path_key = path.as_path().to_string_lossy();

                    // Step 1: Look up SchemaId by path
                    let Some(id) = reader.get_owned::<SchemaId>(
                        SCHEMA_ID_BY_PATH,
                        path_key.as_ref(),
                    )?
                    else {
                        continue;
                    };

                    // Step 2: Look up RawSchemaView by ID
                    if let Some(view) = reader
                        .get_owned_by_uuid::<RawSchemaView>(
                            RAW_SCHEMA_VIEWS,
                            id.into_uuid(),
                        )?
                    {
                        results.insert(path.clone(), view);
                    }
                }

                Ok(results)
            })
            .map_err(map_db_error)
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_PATH;

        // Perform all queries in a single read transaction
        self.db
            .batch_read(|reader| {
                let mut results = HashMap::new();

                for path in file_paths {
                    let path_key = path.as_path().to_string_lossy();
                    if let Some(id) = reader.get_owned::<SchemaId>(
                        SCHEMA_ID_BY_PATH,
                        path_key.as_ref(),
                    )? {
                        results.insert(path.clone(), id);
                    }
                }

                Ok(results)
            })
            .map_err(map_db_error)
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, Self::Error> {
        use crate::schema::db_table::{
            SCHEMA_TOPOLOGICAL_GRAPH, TOPOLOGICAL_GRAPH_KEY,
        };

        self.db
            .get_owned(SCHEMA_TOPOLOGICAL_GRAPH, TOPOLOGICAL_GRAPH_KEY)
            .map_err(map_db_error)
    }

    #[inline]
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{
            SCHEMA_TOPOLOGICAL_GRAPH, TOPOLOGICAL_GRAPH_KEY,
        };

        self.db
            .put(SCHEMA_TOPOLOGICAL_GRAPH, TOPOLOGICAL_GRAPH_KEY, graph)
            .map_err(map_db_error)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::raw::RawSchema;

    /// Helper to create test repository.
    fn setup_test_repo() -> (TempDir, RedbRepository) {
        use std::sync::Arc;

        use crate::db::Database;

        let tmp = TempDir::new().expect("create temp dir");
        let db_path = tmp.path().join("test.db");
        let db = Database::open(&db_path).expect("create database");
        let repo = RedbRepository::new(Arc::new(db));
        (tmp, repo)
    }

    #[test]
    fn find_raw_schema_views_by_paths_returns_empty_for_no_matches() {
        let (_tmp, repo) = setup_test_repo();

        let paths = vec![
            RelativePath::try_from("schemas/foo.json").unwrap(),
            RelativePath::try_from("schemas/bar.json").unwrap(),
        ];

        let results = repo
            .find_raw_schema_views_by_paths(&paths)
            .expect("bulk query should succeed");

        assert!(results.is_empty(), "should return empty map for no matches");
    }

    #[test]
    fn find_schema_ids_by_paths_returns_empty_for_no_matches() {
        let (_tmp, repo) = setup_test_repo();

        let paths = vec![
            RelativePath::try_from("schemas/foo.json").unwrap(),
            RelativePath::try_from("schemas/bar.json").unwrap(),
        ];

        let results = repo
            .find_schema_ids_by_paths(&paths)
            .expect("bulk query should succeed");

        assert!(results.is_empty(), "should return empty map for no matches");
    }

    #[test]
    fn list_schema_path_id_pairs_returns_empty_for_no_matches() {
        let (_tmp, repo) = setup_test_repo();

        let results = repo
            .list_schema_path_id_pairs()
            .expect("list schema path/id pairs should succeed");

        assert!(results.is_empty(), "should return empty list for no matches");
    }

    #[test]
    fn list_schema_path_id_pairs_includes_saved_view() {
        use crate::{
            fs::FileInfo,
            schema::views::{HashRecord, RawPropertyMapHash, SchemaVersion},
            support::hash::Blake3Hash,
        };

        let (_tmp, repo) = setup_test_repo();

        let raw_json = r#"{
            "$version": "1.0",
            "properties": {}
        }"#;
        let raw = serde_json::from_str::<RawSchema>(raw_json)
            .expect("valid schema should deserialize")
            .with_name("test".into());

        let file_stats = FileInfo::new(None, None, 0);
        let hashes = HashRecord::new(
            Blake3Hash::new([0; 32]),
            RawPropertyMapHash::default(),
        );
        let version = SchemaVersion::new(file_stats, hashes, &raw).unwrap();

        let schema_path = RelativePath::try_from("schemas/test.json").unwrap();
        let view = RawSchemaView::new(schema_path.clone(), version);
        let schema_id = SchemaId::new();
        repo.save_raw_schema_view(schema_id, &view)
            .expect("save view should succeed");

        let results = repo
            .list_schema_path_id_pairs()
            .expect("list schema path/id pairs should succeed");

        assert_eq!(results.len(), 1);
        let (path, id) =
            results.first().cloned().expect("should have one entry");
        assert_eq!(id, schema_id);
        assert_eq!(path, schema_path);
    }
}
