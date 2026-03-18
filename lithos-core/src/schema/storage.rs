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
    aggregate::{Schema, SchemaId, SchemaName},
    bank::PropertyBank,
    error::SchemaError,
    property::PropertyName,
    views::{RawPropertyBankView, RawSchemaView},
};
use crate::db::{BatchReader, Database};

/// A schema name-to-ID pair.
pub type NameIdPair = (SchemaName, SchemaId);

/// Inheritance relationship: (`child_id`, `parent_id`, `excludes`).
pub type InheritanceRelation = (SchemaId, Option<SchemaId>, Vec<Box<str>>);

/// Inheritance children map: `parent_id` → Vec<(`child_id`, `excludes`)>.
pub type InheritanceChildren =
    HashMap<SchemaId, Vec<(SchemaId, Vec<Box<str>>)>>;

/// Schema-to-properties usage map: `schema_id` → Vec<`property_name`>.
///
/// Used by `find_schemas_using_properties()` to return which schemas use which
/// properties.
pub type SchemaPropertyUsage = HashMap<SchemaId, Vec<PropertyName>>;

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
    fn list_schema_name_id_pairs(&self)
    -> Result<Vec<NameIdPair>, Self::Error>;

    /// Lists inheritance children for all parent schemas.
    ///
    /// Returns a map of `parent_id` → Vec<(`child_id`, `excludes`)>.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_inheritance_children(
        &self,
    ) -> Result<InheritanceChildren, Self::Error>;

    /// Lists all descendant schema IDs for a given parent.
    ///
    /// Returns transitive children (children, grandchildren, etc.).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn list_descendant_ids(
        &self,
        parent_id: SchemaId,
    ) -> Result<Vec<SchemaId>, Self::Error>;

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
    fn save_schemas(&self, schemas: &[Schema]) -> Result<(), Self::Error>;

    /// Saves inheritance relationships atomically.
    ///
    /// Each relationship is a tuple of (`child_id`, `parent_id`, `excludes`).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_inheritance_relations(
        &self,
        relations: &[InheritanceRelation],
    ) -> Result<(), Self::Error>;

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
    ) -> Result<Option<super::views::RawSchemaView>, Self::Error>;

    /// Saves a raw schema view.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &super::views::RawSchemaView,
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
    ) -> Result<Option<super::views::RawPropertyBankView>, Self::Error>;

    /// Saves the raw property bank view.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_raw_property_bank_view(
        &self,
        view: &super::views::RawPropertyBankView,
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
        file_path: &str,
    ) -> Result<Option<super::views::RawSchemaView>, Self::Error>;

    /// Finds the `SchemaId` for a file path.
    ///
    /// Returns `None` if no schema exists at that path.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn find_schema_id_by_path(
        &self,
        file_path: &str,
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
        file_paths: &[std::path::PathBuf],
    ) -> Result<
        HashMap<std::path::PathBuf, super::views::RawSchemaView>,
        Self::Error,
    >;

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
        file_paths: &[std::path::PathBuf],
    ) -> Result<HashMap<std::path::PathBuf, SchemaId>, Self::Error>;

    // ========================================================================
    // Inheritance Metadata Cache Operations
    // ========================================================================

    /// Gets the inheritance metadata for a given schema ID.
    ///
    /// Returns `None` if no metadata exists (schema never resolved or cache
    /// stale).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<Option<super::views::SchemaInheritanceView>, Self::Error>;

    /// Saves inheritance metadata for a schema.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_inheritance_metadata(
        &self,
        id: SchemaId,
        metadata: &super::views::SchemaInheritanceView,
    ) -> Result<(), Self::Error>;

    /// Deletes inheritance metadata for a schema.
    ///
    /// Used when a schema's inheritance chain changes and the cached metadata
    /// becomes stale.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the deletion fails.
    fn delete_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<(), Self::Error>;

    /// Provides zero-copy access to archived inheritance metadata.
    ///
    /// The closure receives a reference to the archived metadata without
    /// deserialization. This is the most efficient way to read cached metadata.
    ///
    /// Returns `None` if no metadata exists.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Check if metadata is fresh without full deserialization
    /// let is_fresh = repo.with_inheritance_metadata(id, |archived| {
    ///     archived.ancestors_hash == expected_hash
    /// })?.unwrap_or(false);
    /// ```
    fn with_inheritance_metadata<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(
            &'archived rkyv::Archived<super::views::SchemaInheritanceView>,
        ) -> R;

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
    type Error = SchemaError;

    // ========================================================================
    // Schema Read Operations
    // ========================================================================
    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        Ok(self.db.get_owned_by_uuid::<Schema>(SCHEMA_BY_ID, id.into_uuid())?)
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_NAME;

        Ok(self.db.get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())?)
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
                    Err(e) => Some(Err(SchemaError::from(e))),
                }
            })
            .collect()
    }

    #[inline]
    fn list_schemas(&self) -> Result<Vec<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        let pairs: Vec<(Box<str>, Schema)> =
            self.db.list_owned(SCHEMA_BY_ID)?;

        Ok(pairs.into_iter().map(|(_id, schema)| schema).collect())
    }

    #[inline]
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<Vec<NameIdPair>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_NAME;

        self.db
            .list_owned(SCHEMA_ID_BY_NAME)?
            .into_iter()
            .map(|(name_str, id): (Box<str>, SchemaId)| {
                SchemaName::try_new(name_str.as_ref()).map(|name| (name, id))
            })
            .collect()
    }

    #[inline]
    fn list_inheritance_children(
        &self,
    ) -> Result<InheritanceChildren, Self::Error> {
        use std::collections::HashMap;

        // Build children map by scanning all schemas
        // This is simpler than iterating the multimap and since Schema now has
        // parent_id, we can just scan schemas directly
        let mut result = HashMap::new();
        let schemas = self.list_schemas()?;

        for schema in schemas {
            if let Some(parent_id) = schema.parent_id() {
                result
                    .entry(*parent_id)
                    .or_insert_with(Vec::new)
                    .push((*schema.id(), vec![])); // TODO: get excludes from somewhere
            }
        }

        Ok(result)
    }

    #[inline]
    fn list_descendant_ids(
        &self,
        parent_id: SchemaId,
    ) -> Result<Vec<SchemaId>, Self::Error> {
        use std::collections::{HashSet, VecDeque};

        // BFS traversal using Schema.children field
        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(parent_id);

        while let Some(current_id) = queue.pop_front() {
            let Some(schema) = self.find_schema_by_id(current_id)? else {
                continue;
            };

            for &child_id in schema.children() {
                // First time seeing this child - add to queue
                if descendants.insert(child_id) {
                    queue.push_back(child_id);
                }
            }
        }

        Ok(descendants.into_iter().collect())
    }

    // ========================================================================
    // Property Bank Read Operations
    // ========================================================================

    #[inline]
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        use crate::schema::db_table::{PROPERTY_BANK, PROPERTY_BANK_KEY};

        Ok(self
            .db
            .get_owned::<PropertyBank>(PROPERTY_BANK, PROPERTY_BANK_KEY)?)
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

            for property in schema.properties() {
                if target_names.contains(property.name().as_str()) {
                    matching_properties.push(property.name().clone());
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
    fn save_schemas(&self, schemas: &[Schema]) -> Result<(), Self::Error> {
        use crate::schema::db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME};

        self.db.batch_write(|batch| {
            for schema in schemas {
                let id_key = schema.id().to_string();

                // Save schema by ID
                batch.put(SCHEMA_BY_ID, &id_key, schema)?;

                // Save name → ID mapping
                batch.put(
                    SCHEMA_ID_BY_NAME,
                    schema.name().as_str(),
                    schema.id(),
                )?;
            }
            Ok(())
        })?;

        Ok(())
    }

    #[inline]
    fn save_inheritance_relations(
        &self,
        relations: &[InheritanceRelation],
    ) -> Result<(), Self::Error> {
        use std::time::SystemTime;

        use crate::schema::{
            db_table::{SCHEMA_CHILDREN, SCHEMA_PARENT},
            views::inheritance::{ChildSchemaView, ParentSchemaView},
        };

        #[expect(
            clippy::ref_patterns,
            reason = "Destructuring with &(a, b, ref c) is clearest for mixed \
                      Copy/non-Copy fields"
        )]
        self.db.batch_write(|batch| {
            for &(child_id, parent_id, ref excludes) in relations {
                let timestamp = SystemTime::now();
                let child_key = child_id.to_string();

                // Save parent → child mapping in multimap (if not root)
                if let Some(parent) = parent_id {
                    let child_view = ChildSchemaView {
                        child_id,
                        excludes: excludes.clone(),
                        resolved_at: timestamp,
                    };

                    let bytes = child_view.to_bytes()?;
                    batch.multimap_insert_bytes(
                        SCHEMA_CHILDREN,
                        parent.to_string().as_str(),
                        bytes.as_slice(),
                    )?;
                }

                // Save child → parent reference table
                let parent_view = ParentSchemaView {
                    parent_id,
                    excludes: excludes.clone(),
                    resolved_at: timestamp,
                };
                batch.put(SCHEMA_PARENT, child_key.as_str(), &parent_view)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{PROPERTY_BANK, PROPERTY_BANK_KEY};

        self.db.batch_write(|batch| {
            batch.put(PROPERTY_BANK, PROPERTY_BANK_KEY, bank)?;
            Ok(())
        })?;

        Ok(())
    }

    #[expect(
        clippy::unimplemented,
        reason = "Schema deletion is complex and not yet needed"
    )]
    #[inline]
    fn delete_schema(&self, _id: SchemaId) -> Result<(), Self::Error> {
        // Schema deletion is complex and not yet needed - requires:
        // 1. Load schema to get its name
        // 2. Delete from SCHEMA_BY_ID
        // 3. Delete from SCHEMA_ID_BY_NAME
        // 4. Delete from SCHEMA_PARENT
        // 5. Remove from SCHEMA_CHILDREN multimap entries
        unimplemented!("Schema deletion with proper cleanup of all references")
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

    #[inline]
    fn with_batch_reader<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>,
    {
        // Adapt the closure to convert SchemaError -> DbError for batch_read
        // Then convert DbError -> SchemaError for the final result
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Explicitly matching all 27 SchemaError variants would \
                      be fragile and unnecessary - we only care about Storage \
                      variant"
        )]
        let result = self.db.batch_read(|reader| {
            f(reader).map_err(|schema_err| {
                // Extract DbError if it's a Storage variant, otherwise create a
                // generic error This is a workaround for the
                // type mismatch
                match schema_err {
                    SchemaError::Storage(db_err) => db_err,
                    _ => {
                        // This shouldn't happen in practice since f should only
                        // return Storage errors when using the reader
                        crate::db::DbError::Database(schema_err.to_string())
                    }
                }
            })
        });

        result.map_err(SchemaError::from)
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

        let key = id.to_string();
        self.db
            .get_owned(RAW_SCHEMA_VIEWS, key.as_str())
            .map_err(SchemaError::from)
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{
            RAW_SCHEMA_VIEW_BY_PATH, RAW_SCHEMA_VIEWS,
        };

        let key = id.to_string();
        self.db.batch_write(|batch| {
            batch.put(RAW_SCHEMA_VIEWS, &key, view)?;
            batch.put(
                RAW_SCHEMA_VIEW_BY_PATH,
                view.file_path().as_str(),
                &id,
            )?;
            Ok(())
        })?;

        Ok(())
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
    ) -> Result<Option<RawPropertyBankView>, Self::Error> {
        use crate::schema::db_table::{
            RAW_PROPERTY_BANK_KEY, RAW_PROPERTY_BANK_VIEW,
        };

        self.db
            .get_owned(RAW_PROPERTY_BANK_VIEW, RAW_PROPERTY_BANK_KEY)
            .map_err(SchemaError::from)
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        view: &RawPropertyBankView,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{
            RAW_PROPERTY_BANK_KEY, RAW_PROPERTY_BANK_VIEW,
        };

        self.db
            .put(RAW_PROPERTY_BANK_VIEW, RAW_PROPERTY_BANK_KEY, view)
            .map_err(SchemaError::from)
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<RawSchemaView>, Self::Error> {
        use crate::schema::db_table::RAW_SCHEMA_VIEW_BY_PATH;

        // First lookup SchemaId by path
        let id = self
            .db
            .get_owned::<SchemaId>(RAW_SCHEMA_VIEW_BY_PATH, file_path)?;

        match id {
            Some(id) => self.get_raw_schema_view(id),
            None => Ok(None),
        }
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<SchemaId>, Self::Error> {
        use crate::schema::db_table::RAW_SCHEMA_VIEW_BY_PATH;

        self.db
            .get_owned::<SchemaId>(RAW_SCHEMA_VIEW_BY_PATH, file_path)
            .map_err(SchemaError::from)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        file_paths: &[std::path::PathBuf],
    ) -> Result<
        HashMap<std::path::PathBuf, super::views::RawSchemaView>,
        Self::Error,
    > {
        use crate::schema::db_table::{
            RAW_SCHEMA_VIEW_BY_PATH, RAW_SCHEMA_VIEWS,
        };

        // Perform all queries in a single read transaction
        self.db
            .batch_read(|reader| {
                let mut results = HashMap::new();

                for path in file_paths {
                    let path_key = path.to_string_lossy();

                    // Step 1: Look up SchemaId by path
                    let Some(id) = reader.get_owned::<SchemaId>(
                        RAW_SCHEMA_VIEW_BY_PATH,
                        path_key.as_ref(),
                    )?
                    else {
                        continue;
                    };

                    // Step 2: Look up RawSchemaView by ID
                    let id_key = id.to_string();
                    if let Some(view) = reader.get_owned::<RawSchemaView>(
                        RAW_SCHEMA_VIEWS,
                        id_key.as_str(),
                    )? {
                        results.insert(path.clone(), view);
                    }
                }

                Ok(results)
            })
            .map_err(SchemaError::from)
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        file_paths: &[std::path::PathBuf],
    ) -> Result<HashMap<std::path::PathBuf, SchemaId>, Self::Error> {
        use crate::schema::db_table::RAW_SCHEMA_VIEW_BY_PATH;

        // Perform all queries in a single read transaction
        self.db
            .batch_read(|reader| {
                let mut results = HashMap::new();

                for path in file_paths {
                    let path_key = path.to_string_lossy();
                    if let Some(id) = reader.get_owned::<SchemaId>(
                        RAW_SCHEMA_VIEW_BY_PATH,
                        path_key.as_ref(),
                    )? {
                        results.insert(path.clone(), id);
                    }
                }

                Ok(results)
            })
            .map_err(SchemaError::from)
    }

    // ========================================================================
    // Inheritance Metadata Cache Operations
    // ========================================================================

    #[inline]
    fn get_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<Option<super::views::SchemaInheritanceView>, Self::Error> {
        use crate::schema::db_table::SCHEMA_INHERITANCE;

        let key = id.to_string();
        self.db
            .get_owned(SCHEMA_INHERITANCE, key.as_str())
            .map_err(SchemaError::from)
    }

    #[inline]
    fn save_inheritance_metadata(
        &self,
        id: SchemaId,
        metadata: &super::views::SchemaInheritanceView,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::SCHEMA_INHERITANCE;

        let key = id.to_string();
        self.db
            .put(SCHEMA_INHERITANCE, key.as_str(), metadata)
            .map_err(SchemaError::from)
    }

    #[inline]
    fn delete_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::SCHEMA_INHERITANCE;

        let key = id.to_string();
        self.db
            .delete(SCHEMA_INHERITANCE, key.as_str())
            .map(|_deleted| ()) // Discard bool return value
            .map_err(SchemaError::from)
    }

    #[inline]
    fn with_inheritance_metadata<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(
            &'archived rkyv::Archived<super::views::SchemaInheritanceView>,
        ) -> R,
    {
        use crate::schema::db_table::SCHEMA_INHERITANCE;

        let key = id.to_string();
        self.db
            .get::<super::views::SchemaInheritanceView, _, R>(
                SCHEMA_INHERITANCE,
                key.as_str(),
                f,
            )
            .map_err(SchemaError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

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
            PathBuf::from("schemas/foo.json"),
            PathBuf::from("schemas/bar.json"),
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
            PathBuf::from("schemas/foo.json"),
            PathBuf::from("schemas/bar.json"),
        ];

        let results = repo
            .find_schema_ids_by_paths(&paths)
            .expect("bulk query should succeed");

        assert!(results.is_empty(), "should return empty map for no matches");
    }

    #[test]
    #[ignore = "RedbRepository implementation pending - migrate from \
                db_query/db_command"]
    fn redb_repository_skeleton_exists() {
        // This test documents that the skeleton exists but implementation is
        // pending Remove #[ignore] as methods are implemented
    }
}
