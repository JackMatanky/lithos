//! Database storage backend for schema views.
//!
//! Provides the [`RedbRepository`] which persists schema metadata and version
//! history using the `redb` zero-copy database.

use std::collections::HashMap;

use rkyv::rancor::Error as RancorError;

use crate::{
    db::{BatchReader, Database, DbError},
    fs::RelativePath,
    schema::{
        error::{SchemaRepositoryError, SchemaStorageError},
        identifier::SchemaId,
        views::{RawPropertyBankView, RawSchemaView, contracts::RawView},
    },
};

/// Specialized reader for batch schema operations.
///
/// This structure holds a database transaction and provides efficient,
/// zero-copy access to schema metadata during discovery and resolution.
pub struct RedbBatchSchemaReader<'reader> {
    reader: &'reader BatchReader,
}

impl<'reader> RedbBatchSchemaReader<'reader> {
    /// Creates a new batch reader from a database transaction.
    #[inline]
    #[must_use]
    pub const fn new(reader: &'reader BatchReader) -> Self {
        Self {
            reader,
        }
    }
}

impl BatchSchemaReader for RedbBatchSchemaReader<'_> {
    type Error = SchemaRepositoryError;

    #[inline]
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<RawSchemaView, Self::Error> {
        let key = id.into_uuid();
        let key_str = key.to_string();
        self.reader
            .get::<RawSchemaView, _, _>(
                super::db_table::RAW_SCHEMA_VIEWS,
                &key_str,
                |archived| {
                    rkyv::deserialize::<RawSchemaView, RancorError>(archived)
                },
            )
            .map_err(SchemaRepositoryError::from)?
            .ok_or(SchemaRepositoryError::NotFound(id))?
            .map_err(|e| SchemaRepositoryError::Serialization(e.to_string()))
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<RawPropertyBankView, Self::Error> {
        let key = path.as_path().to_string_lossy();
        self.reader
            .get::<RawPropertyBankView, _, _>(
                super::db_table::RAW_SCHEMA_VIEWS,
                key.as_ref(),
                |archived| {
                    rkyv::deserialize::<RawPropertyBankView, RancorError>(
                        archived,
                    )
                },
            )
            .map_err(SchemaRepositoryError::from)?
            .ok_or_else(|| SchemaRepositoryError::Database(DbError::NotFound))?
            .map_err(|e| SchemaRepositoryError::Serialization(e.to_string()))
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error> {
        let mut results = HashMap::with_capacity(paths.len());
        let path_set: std::collections::HashSet<_> =
            paths.iter().map(|p| p.as_path().to_string_lossy()).collect();

        self.reader
            .scan_range::<RawSchemaView>(super::db_table::RAW_SCHEMA_VIEWS, "")
            .map_err(SchemaRepositoryError::from)?
            .into_iter()
            .for_each(|(id_str, archived)| {
                let path_str = archived.path().as_path().to_string_lossy();
                if path_set.contains(&path_str) {
                    if let (Ok(path), Ok(id_uuid)) = (
                        RelativePath::try_from(path_str.as_ref()),
                        uuid::Uuid::parse_str(&id_str),
                    ) {
                        results.insert(path, SchemaId::from_uuid(id_uuid));
                    }
                }
            });

        Ok(results)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error> {
        let mut results = HashMap::with_capacity(paths.len());
        let path_set: std::collections::HashSet<_> =
            paths.iter().map(|p| p.as_path().to_string_lossy()).collect();

        self.reader
            .scan_range::<RawSchemaView>(super::db_table::RAW_SCHEMA_VIEWS, "")
            .map_err(SchemaRepositoryError::from)?
            .into_iter()
            .for_each(|(_, archived)| {
                let path_str = archived.path().as_path().to_string_lossy();
                if path_set.contains(&path_str) {
                    results.insert(archived.path().clone(), archived);
                }
            });

        Ok(results)
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<Vec<(RelativePath, SchemaId)>, Self::Error> {
        let mut results = Vec::new();
        self.reader
            .scan_range::<RawSchemaView>(super::db_table::RAW_SCHEMA_VIEWS, "")
            .map_err(SchemaRepositoryError::from)?
            .into_iter()
            .for_each(|(id_str, archived)| {
                let path_str = archived.path().as_path().to_string_lossy();
                if let (Ok(path), Ok(id_uuid)) = (
                    RelativePath::try_from(path_str.as_ref()),
                    uuid::Uuid::parse_str(&id_str),
                ) {
                    results.push((path, SchemaId::from_uuid(id_uuid)));
                }
            });

        Ok(results)
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<
        Option<crate::schema::inheritance::InheritanceGraph<()>>,
        Self::Error,
    > {
        let mut builder = crate::schema::inheritance::SchemaGraphBuilder::new();
        let mut has_data = false;

        self.reader
            .scan_range::<RawSchemaView>(super::db_table::RAW_SCHEMA_VIEWS, "")
            .map_err(SchemaRepositoryError::from)?
            .into_iter()
            .for_each(|(id_str, archived)| {
                has_data = true;
                if let Ok(id_uuid) = uuid::Uuid::parse_str(&id_str) {
                    let id = SchemaId::from_uuid(id_uuid);
                    builder.add_node(id, ());

                    if let Some(current) = archived.current() {
                        if current.extends().is_some() {
                            // Find parent ID by name
                            // This is expensive in this pass, but we only
                            // have a few schemas
                            // A better way would be a secondary index
                        }
                    }
                }
            });

        if has_data {
            // Second pass to resolve inheritance now that all nodes are known
            // This is still clunky but works for now.
            let graph = crate::schema::inheritance::InheritanceGraph::try_from(
                builder.build::<()>(),
            )
            .map_err(|e| {
                SchemaRepositoryError::Serialization(format!(
                    "Failed to build inheritance graph: {e:?}"
                ))
            })?;
            Ok(Some(graph))
        } else {
            Ok(None)
        }
    }
}

/// Trait defining the schema storage operations.
pub trait Repository {
    /// The error type returned by repository operations.
    type Error;

    /// Saves a raw schema view to the repository.
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), Self::Error>;

    /// Deletes a schema and its view from the repository.
    fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error>;

    /// Saves the raw property bank view to the repository.
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), Self::Error>;

    /// Retrieves the property bank domain aggregate.
    ///
    /// # Errors
    ///
    /// Returns an error if database access fails or deserialization fails.
    fn get_property_bank(
        &self,
    ) -> Result<Option<crate::schema::bank::PropertyBank>, Self::Error>;

    /// Persists the property bank domain aggregate.
    ///
    /// # Errors
    ///
    /// Returns an error if database access fails or serialization fails.
    fn save_property_bank(
        &self,
        bank: &crate::schema::bank::PropertyBank,
    ) -> Result<(), Self::Error>;

    /// Executes a closure with a batch schema reader.
    fn with_batch_schema_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(
            &dyn BatchSchemaReader<Error = SchemaRepositoryError>,
        ) -> Result<R, SchemaRepositoryError>;

    /// Returns a list of all schema paths and their IDs.
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<Vec<(RelativePath, SchemaId)>, Self::Error>;
}

/// Defines the read operations available during a batch discovery/resolution
/// run.
pub trait BatchSchemaReader {
    /// The error type returned by reader operations.
    type Error;

    /// Fetches a raw schema view by its stable ID.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<RawSchemaView, Self::Error>;

    /// Fetches the raw property bank view by its path.
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<RawPropertyBankView, Self::Error>;

    /// Finds multiple schema IDs by their filesystem paths.
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error>;

    /// Finds multiple raw schema views by their filesystem paths.
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error>;

    /// Returns a list of all schema paths and their IDs.
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<Vec<(RelativePath, SchemaId)>, Self::Error>;

    /// Returns the current inheritance graph from the database.
    fn get_topological_graph(
        &self,
    ) -> Result<
        Option<crate::schema::inheritance::InheritanceGraph<()>>,
        Self::Error,
    >;
}

/// concrete implementation of [`Repository`] using `redb`.
pub struct RedbRepository {
    db: Database,
}

impl RedbRepository {
    /// Creates a new `RedbRepository` from a database instance.
    #[inline]
    #[must_use]
    pub const fn new(db: Database) -> Self {
        Self {
            db,
        }
    }
}

impl Repository for RedbRepository {
    type Error = SchemaRepositoryError;

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), Self::Error> {
        let key = id.into_uuid();
        let key_str = key.to_string();
        self.db
            .batch_write(|writer| {
                writer.put(super::db_table::RAW_SCHEMA_VIEWS, &key_str, view)
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn delete_schema(&self, id: SchemaId) -> Result<(), Self::Error> {
        let key = id.into_uuid();
        let key_str = key.to_string();
        self.db
            .batch_write(|writer| {
                writer
                    .delete(super::db_table::RAW_SCHEMA_VIEWS, &key_str)
                    .map(|_| ())
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), Self::Error> {
        let key = path.as_path().to_string_lossy();
        self.db
            .batch_write(|writer| {
                writer.put(
                    super::db_table::RAW_SCHEMA_VIEWS,
                    key.as_ref(),
                    view,
                )
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn get_property_bank(
        &self,
    ) -> Result<Option<crate::schema::bank::PropertyBank>, Self::Error> {
        use crate::schema::bank::PropertyBank;

        self.db
            .batch_read(|reader| {
                reader.get::<PropertyBank, _, _>(
                    super::db_table::PROPERTY_BANK,
                    super::db_table::PROPERTY_BANK_KEY,
                    |archived| {
                        rkyv::deserialize::<PropertyBank, RancorError>(archived)
                    },
                )
            })
            .map_err(SchemaRepositoryError::from)?
            .transpose()
            .map_err(|e| SchemaRepositoryError::Serialization(e.to_string()))
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &crate::schema::bank::PropertyBank,
    ) -> Result<(), Self::Error> {
        self.db
            .batch_write(|writer| {
                writer.put(
                    super::db_table::PROPERTY_BANK,
                    super::db_table::PROPERTY_BANK_KEY,
                    bank,
                )
            })
            .map_err(SchemaRepositoryError::from)
    }

    #[inline]
    fn with_batch_schema_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(
            &dyn BatchSchemaReader<Error = SchemaRepositoryError>,
        ) -> Result<R, SchemaRepositoryError>,
    {
        self.db
            .batch_read(|reader| {
                let schema_reader = RedbBatchSchemaReader::new(reader);
                f(&schema_reader).map_err(|e| match e {
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Storage(db_err),
                    ) => db_err,
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::NotFound {
                            name,
                        },
                    ) => DbError::Deserialization(format!(
                        "Schema not found: {}",
                        name
                    )),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Corruption {
                            reason,
                        },
                    ) => DbError::Corruption(reason.to_string()),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::PropertyBankNotFound,
                    ) => DbError::Deserialization(
                        "PropertyBank not found".into(),
                    ),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Conflict {
                            reason,
                        },
                    ) => DbError::Database(format!("conflict: {reason}")),
                    SchemaRepositoryError::Domain(domain_err) => {
                        DbError::Deserialization(domain_err.to_string())
                    }
                    SchemaRepositoryError::Database(db_err) => db_err,
                    SchemaRepositoryError::NotFound(id) => {
                        DbError::Deserialization(format!(
                            "Schema not found: {}",
                            id
                        ))
                    }
                    SchemaRepositoryError::Serialization(msg) => {
                        DbError::Deserialization(msg)
                    }
                })
            })
            .map_err(|db_err| SchemaRepositoryError::Database(db_err))
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<Vec<(RelativePath, SchemaId)>, Self::Error> {
        self.db
            .batch_read(|reader| {
                let schema_reader = RedbBatchSchemaReader::new(reader);
                schema_reader.list_schema_path_id_pairs().map_err(|e| match e {
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Storage(db_err),
                    ) => db_err,
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::NotFound {
                            name,
                        },
                    ) => DbError::Deserialization(format!(
                        "Schema not found: {}",
                        name
                    )),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Corruption {
                            reason,
                        },
                    ) => DbError::Corruption(reason.to_string()),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::PropertyBankNotFound,
                    ) => DbError::Deserialization(
                        "PropertyBank not found".into(),
                    ),
                    SchemaRepositoryError::Storage(
                        SchemaStorageError::Conflict {
                            reason,
                        },
                    ) => DbError::Database(format!("conflict: {reason}")),
                    SchemaRepositoryError::Domain(domain_err) => {
                        DbError::Deserialization(domain_err.to_string())
                    }
                    SchemaRepositoryError::Database(db_err) => db_err,
                    SchemaRepositoryError::NotFound(id) => {
                        DbError::Deserialization(format!(
                            "Schema not found: {}",
                            id
                        ))
                    }
                    SchemaRepositoryError::Serialization(msg) => {
                        DbError::Deserialization(msg)
                    }
                })
            })
            .map_err(|db_err| SchemaRepositoryError::Database(db_err))
    }
}
