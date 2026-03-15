//! Redb-backed repository implementation for schema persistence.
//!
//! This module provides the concrete `RedbRepository` implementation of the
//! `Repository` trait, using redb for storage.
//!
//! ## Migration Note
//!
//! This is a partial implementation created during the CQRS → Repository
//! refactor. Methods are being migrated incrementally from `db_query.rs`
//! and `db_command.rs`.
//!
//! **Status**: Initial skeleton - core methods implemented, full migration
//! pending

#![allow(
    clippy::todo,
    clippy::missing_inline_in_public_items,
    dead_code,
    unused_imports,
    reason = "Skeleton implementation - methods being migrated incrementally"
)]

use std::sync::Arc;

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    bank::{BankVersion, PropertyBank},
    error::SchemaError,
    property::{Property, PropertyId, PropertyName},
    repository::{
        InheritanceChildren, InheritanceRelation, NameIdPair, Repository,
        SchemaPropertyUsage,
    },
};
use crate::db::{BatchReader, Database};

/// Redb-backed repository implementation.
///
/// Provides persistent storage for schemas and property banks using the
/// redb embedded database.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::RedbRepository;
/// use redb::Database;
///
/// let db = Database::create("schemas.db")?;
/// let repo = RedbRepository::new(db);
///
/// // Use repository
/// let schemas = repo.list_schemas()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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

    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        Ok(self.db.get_owned_by_uuid::<Schema>(SCHEMA_BY_ID, id.into_uuid())?)
    }

    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_NAME;

        Ok(self.db.get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())?)
    }

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

    fn list_schemas(&self) -> Result<Vec<Schema>, Self::Error> {
        use crate::schema::db_table::SCHEMA_BY_ID;

        let pairs: Vec<(Box<str>, Schema)> =
            self.db.list_owned(SCHEMA_BY_ID)?;

        Ok(pairs.into_iter().map(|(_id, schema)| schema).collect())
    }

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

    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        use crate::schema::db_table::{BANK_METADATA, PROPERTY_BANK_KEY};

        Ok(self
            .db
            .get_owned::<PropertyBank>(BANK_METADATA, PROPERTY_BANK_KEY)?)
    }

    fn find_property_by_id(
        &self,
        id: PropertyId,
    ) -> Result<Option<Property>, Self::Error> {
        use crate::schema::db_table::BANK_PROPERTY_BY_ID;

        Ok(self.db.get_owned_by_uuid::<Property>(
            BANK_PROPERTY_BY_ID,
            id.into_uuid(),
        )?)
    }

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

    fn save_inheritance_relations(
        &self,
        relations: &[InheritanceRelation],
    ) -> Result<(), Self::Error> {
        use std::time::SystemTime;

        use crate::schema::{
            db_table::{SCHEMA_CHILDREN, SCHEMA_PARENT},
            views::{ChildSchemaView, ParentSchemaView},
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

    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), Self::Error> {
        use crate::schema::db_table::{BANK_METADATA, PROPERTY_BANK_KEY};

        self.db.batch_write(|batch| {
            batch.put(BANK_METADATA, PROPERTY_BANK_KEY, bank)?;
            Ok(())
        })?;

        Ok(())
    }

    fn delete_schema(&self, _id: SchemaId) -> Result<(), Self::Error> {
        use crate::schema::db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME};

        // Need to:
        // 1. Load schema to get its name
        // 2. Delete from SCHEMA_BY_ID
        // 3. Delete from SCHEMA_ID_BY_NAME
        // 4. Delete from SCHEMA_PARENT
        // 5. Remove from SCHEMA_CHILDREN multimap entries
        // This is complex and needs careful implementation to avoid orphans
        todo!("Schema deletion with proper cleanup of all references")
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "RedbRepository implementation pending - migrate from \
                db_query/db_command"]
    fn redb_repository_skeleton_exists() {
        // This test documents that the skeleton exists but implementation is
        // pending Remove #[ignore] as methods are implemented
    }
}
