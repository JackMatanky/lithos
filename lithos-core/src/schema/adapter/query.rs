//! Redb-backed implementation of the [`crate::schema::ports::Query`] trait.
//!
//! Property bank reads use `bank_metadata` plus versioned rows from
//! `bank_property_by_name`.

use std::collections::HashMap;

use crate::{
    db::{BatchReader, Database, DbError},
    schema::{
        adapter::stored::{
            StoredBankProperty, StoredChildSchema, StoredMetadata,
            StoredPropertyBank, StoredSchema,
        },
        aggregate::{Schema, SchemaId, SchemaName, Timestamp},
        bank::{BankVersion, PropertyBank},
        db_table::{
            BANK_METADATA, BANK_PROPERTY_BY_ID, BANK_PROPERTY_BY_NAME,
            PROPERTY_BANK_KEY, SCHEMA_BY_ID, SCHEMA_CHILDREN,
            SCHEMA_ID_BY_NAME, SCHEMA_METADATA,
        },
        ports::{NameIdPair, Query},
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
    },
};

/// Redb-backed schema query adapter.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::adapter::query::QueryAdapter;
///
/// let db = todo!("Provide a Database instance");
/// let adapter = QueryAdapter::new(&db);
/// let _ = adapter;
/// ```
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a query adapter for a database.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::adapter::query::QueryAdapter;
    ///
    /// let db = todo!("Provide a Database instance");
    /// let adapter = QueryAdapter::new(&db);
    /// let _ = adapter;
    /// ```
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl QueryAdapter<'_> {
    /// Check if a stored schema is fresh based on version and timestamps.
    ///
    /// Returns `Ok(true)` if fresh, `Ok(false)` if stale.
    fn check_schema_freshness(
        stored: &rkyv::Archived<StoredMetadata>,
        id: SchemaId,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
        bank_version: BankVersion,
    ) -> Result<bool, DbError> {
        // Deserialize only the fields we need
        let stored_version: BankVersion = rkyv::deserialize::<
            BankVersion,
            rkyv::rancor::Error,
        >(&stored.bank_version)
        .map_err(|e| DbError::Deserialization(e.to_string()))?;

        if stored_version != bank_version {
            return Ok(false); // Not fresh: version mismatch
        }

        // Verify file identity via created_at when possible
        if let (Some(file_created), Some(archived_created)) =
            (created_at, stored.created_at.as_ref())
        {
            let stored_created: Timestamp = rkyv::deserialize::<
                Timestamp,
                rkyv::rancor::Error,
            >(archived_created)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;

            if file_created.as_secs() != stored_created.as_secs() {
                return Ok(false); // Not fresh: created_at mismatch
            }
        } else if created_at.is_some() != stored.created_at.is_some() {
            tracing::warn!(
                schema_id = %id,
                "Cannot verify schema identity: created_at unavailable"
            );
        } else {
            // Both are None - no timestamp to verify
        }

        // Check modified time
        if let (Some(file_mtime), Some(archived_mtime)) =
            (modified_at, stored.modified_at.as_ref())
        {
            let stored_mtime: Timestamp = rkyv::deserialize::<
                Timestamp,
                rkyv::rancor::Error,
            >(archived_mtime)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;

            if stored_mtime.as_secs() < file_mtime.as_secs() {
                return Ok(false); // Not fresh: file modified
            }
        }

        Ok(true) // Fresh
    }
}

impl Query for QueryAdapter<'_> {
    type Error = DbError;

    #[inline]
    fn read_many<R, F>(&self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(&BatchReader) -> Result<R, Self::Error>,
    {
        self.db.batch_read(f)
    }

    #[inline]
    fn find_many_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<std::collections::HashMap<SchemaId, Schema>, Self::Error> {
        use std::collections::HashMap;

        self.db.batch_read(|reader| {
            let mut results = HashMap::with_capacity(ids.len());

            for id in ids {
                let id_key = id.into_uuid().to_string();

                let Some(stored) = reader
                    .get_owned::<StoredSchema>(SCHEMA_BY_ID, id_key.as_str())?
                else {
                    continue; // Skip missing schemas
                };

                // Validate schema-metadata consistency to detect corruption
                let metadata_exists = reader
                    .get_owned::<StoredMetadata>(
                        SCHEMA_METADATA,
                        id_key.as_str(),
                    )?
                    .is_some();

                if !metadata_exists {
                    return Err(DbError::Corruption(format!(
                        "schema {} exists but metadata is missing (database \
                         corruption detected)",
                        id.as_uuid()
                    )));
                }

                let schema = Schema::try_from(stored).map_err(
                    |e: super::super::error::SchemaError| {
                        DbError::Deserialization(e.to_string())
                    },
                )?;
                results.insert(*id, schema);
            }

            Ok(results)
        })
    }

    #[inline]
    fn are_many_stale(
        &self,
        schemas: &[super::super::ports::StalenessCheck],
        bank_version: BankVersion,
    ) -> Result<std::collections::HashMap<SchemaId, bool>, Self::Error> {
        use std::collections::HashMap;

        self.db.batch_read(|reader| {
            let mut results = HashMap::with_capacity(schemas.len());

            for &(id, created_at, modified_at) in schemas {
                // Check if metadata exists (missing = stale)
                let Some(is_fresh_result) = reader
                    .get::<StoredMetadata, _, _>(
                        SCHEMA_METADATA,
                        id.into_uuid().to_string().as_str(),
                        |stored| {
                            Self::check_schema_freshness(
                                stored,
                                id,
                                created_at,
                                modified_at,
                                bank_version,
                            )
                        },
                    )?
                else {
                    // Missing metadata = stale
                    results.insert(id, true);
                    continue;
                };

                // Invert: is_fresh -> is_stale
                let is_fresh = is_fresh_result?;
                results.insert(id, !is_fresh);
            }

            Ok(results)
        })
    }

    #[inline]
    fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error> {
        let Some(metadata) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(None);
        };

        let prefix = StoredBankProperty::prefix(metadata.bank_version);
        let entries = self
            .db
            .scan_range::<StoredBankProperty>(BANK_PROPERTY_BY_NAME, &prefix)?;
        let properties: Vec<_> =
            entries.into_iter().map(|(_, stored)| stored.property).collect();

        let stored = StoredPropertyBank {
            bank_version: metadata.bank_version,
            recorded_at: metadata.recorded_at,
            properties,
        };

        PropertyBank::try_from(stored)
            .map(Some)
            .map_err(|e| DbError::Deserialization(e.to_string()))
    }

    #[inline]
    fn get_property_by_id(
        &self,
        id: PropertyId,
    ) -> Result<Option<Property>, Self::Error> {
        // Get current bank version from metadata
        let Some(metadata) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(None);
        };

        // Query BANK_PROPERTY_BY_ID with versioned key
        let key =
            StoredBankProperty::key(metadata.bank_version, &id.to_string());
        let Some(stored) = self
            .db
            .get_owned::<StoredBankProperty>(BANK_PROPERTY_BY_ID, &key)?
        else {
            return Ok(None);
        };

        // Reconstruct Property from StoredProperty
        let sp = stored.property;
        let prop_name = PropertyName::try_from(sp.name)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let optionality = Optionality::from(sp.required);
        let multiplicity = Multiplicity::from(sp.multi);

        Ok(Some(Property::new(
            sp.id,
            prop_name,
            optionality,
            multiplicity,
            sp.spec,
        )))
    }

    #[inline]
    fn is_bank_stale(&self, version: BankVersion) -> Result<bool, Self::Error> {
        let Some(stored) = self
            .db
            .get_owned::<StoredMetadata>(BANK_METADATA, PROPERTY_BANK_KEY)?
        else {
            return Ok(true);
        };
        Ok(stored.bank_version != version)
    }

    #[inline]
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        let Some(stored) = self
            .db
            .get_owned_by_uuid::<StoredSchema>(SCHEMA_BY_ID, id.into_uuid())?
        else {
            return Ok(None);
        };

        // Validate schema-metadata consistency to detect corruption
        let id_key = id.into_uuid().to_string();
        let metadata_exists = self
            .db
            .get_owned::<StoredMetadata>(SCHEMA_METADATA, id_key.as_str())?
            .is_some();

        if !metadata_exists {
            return Err(DbError::Corruption(format!(
                "schema {} exists but metadata is missing (database \
                 corruption detected)",
                id.as_uuid()
            )));
        }

        Schema::try_from(stored)
            .map(Some)
            .map_err(|e| DbError::Deserialization(e.to_string()))
    }

    #[inline]
    fn is_schema_stale(
        &self,
        id: SchemaId,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
        bank_version: BankVersion,
    ) -> Result<bool, Self::Error> {
        // Zero-copy metadata check using closure-based API.
        // This eliminates the double deserialization in the old implementation:
        // - Old: get() to check existence, then get_owned() to deserialize full
        //   struct
        // - New: Single zero-copy read with minimal field deserialization
        let Some(result) =
            self.with_metadata(id, |stored| -> Result<bool, DbError> {
                // Deserialize only the primitive fields we need to compare.
                // This is faster than get_owned() which allocates and
                // deserializes the entire struct including
                // fields we don't need.
                let stored_version: BankVersion =
                    rkyv::deserialize::<BankVersion, rkyv::rancor::Error>(
                        &stored.bank_version,
                    )
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;

                if stored_version != bank_version {
                    return Ok(false); // Not fresh: version mismatch
                }

                // Verify file identity via created_at when possible.
                if let (Some(file_created), Some(archived_created)) =
                    (created_at, stored.created_at.as_ref())
                {
                    let stored_created: Timestamp =
                        rkyv::deserialize::<Timestamp, rkyv::rancor::Error>(
                            archived_created,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;

                    if file_created.as_secs() != stored_created.as_secs() {
                        return Ok(false); // Not fresh: created_at mismatch
                    }
                } else if created_at.is_some() != stored.created_at.is_some() {
                    tracing::warn!(
                        schema_id = %id,
                        "Cannot verify schema identity: created_at unavailable"
                    );
                } else {
                    // Both are None - no timestamp to verify
                }

                // Compare file mtime with stored mtime.
                if let (Some(file_mtime), Some(archived_mtime)) =
                    (modified_at, stored.modified_at.as_ref())
                {
                    let stored_mtime: Timestamp =
                        rkyv::deserialize::<Timestamp, rkyv::rancor::Error>(
                            archived_mtime,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;

                    if stored_mtime.as_secs() < file_mtime.as_secs() {
                        return Ok(false); // Not fresh: file modified after storage
                    }
                }

                Ok(true) // Fresh: all checks passed
            })?
        else {
            // Metadata not found → schema is stale.
            return Ok(true);
        };

        // Flatten the nested Result
        let is_fresh = result?;
        Ok(!is_fresh)
    }

    #[inline]
    fn with_metadata<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: FnOnce(&rkyv::Archived<StoredMetadata>) -> R,
    {
        let id_key = id.into_uuid().to_string();
        self.db.get::<StoredMetadata, _, _>(SCHEMA_METADATA, id_key.as_str(), f)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Schema>, Self::Error> {
        let stored: Vec<StoredSchema> = self.db.list_owned(SCHEMA_BY_ID)?;
        stored
            .into_iter()
            .map(|s| {
                Schema::try_from(s)
                    .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .collect()
    }

    #[inline]
    fn list_name_id_pairs(&self) -> Result<Vec<NameIdPair>, Self::Error> {
        let pairs =
            self.db.list_key_value_pairs::<SchemaId>(SCHEMA_ID_BY_NAME)?;
        pairs
            .into_iter()
            .map(|(name, id)| {
                SchemaName::try_new(&name)
                    .map(|schema_name| (schema_name, id))
                    .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .collect()
    }

    #[inline]
    fn find_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, Self::Error> {
        self.db.get_owned(SCHEMA_ID_BY_NAME, name.as_str())
    }

    #[inline]
    fn list_children(
        &self,
        parent_ids: &[SchemaId],
    ) -> Result<crate::schema::ports::InheritanceMap, Self::Error> {
        // Direct transaction access for multimap operations
        let tx = self.db.begin_read()?;
        let table = match tx.open_multimap_table(SCHEMA_CHILDREN) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                // Table doesn't exist yet - return empty map
                return Ok(HashMap::new());
            }
            Err(e) => return Err(DbError::from(e)),
        };

        let mut result = HashMap::with_capacity(parent_ids.len());

        for &parent_id in parent_ids {
            let parent_key = parent_id.to_string();

            // O(1) multimap lookup
            let children_iter = table.get(parent_key.as_str())?;

            let mut children = Vec::new();
            for guard_result in children_iter {
                let guard = guard_result?;
                let bytes = guard.value();
                let archived = rkyv::access::<
                    rkyv::Archived<StoredChildSchema>,
                    rkyv::rancor::Error,
                >(bytes)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;

                // Convert archived types to owned types
                // Deserialize the full object to get native types
                let stored: StoredChildSchema = rkyv::deserialize(archived)
                    .map_err(|e: rkyv::rancor::Error| {
                        DbError::Deserialization(e.to_string())
                    })?;

                children.push((stored.child_id, stored.excludes));
            }

            if !children.is_empty() {
                result.insert(parent_id, children);
            }
        }

        Ok(result)
    }

    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "HashMap values() iteration order doesn't affect BFS \
                  correctness"
    )]
    #[expect(
        clippy::excessive_nesting,
        reason = "Standard BFS pattern: while loop + iterate values + iterate \
                  children - flattening would obscure algorithm structure"
    )]
    fn list_descendants(
        &self,
        parent_ids: &[SchemaId],
    ) -> Result<std::collections::HashSet<SchemaId>, Self::Error> {
        use std::collections::HashSet;

        // Compute transitive closure via BFS over SCHEMA_CHILDREN multimap
        // The multimap stores only direct parent→child edges, so we need
        // iterative traversal to find all descendants (children, grandchildren,
        // etc.)
        let mut all_descendants: HashSet<SchemaId> =
            parent_ids.iter().copied().collect();
        let mut frontier: Vec<SchemaId> = parent_ids.to_vec();

        while !frontier.is_empty() {
            // Query direct children of current frontier from multimap
            let children_map = self.list_children(&frontier)?;
            if children_map.is_empty() {
                break;
            }

            // Prepare next level: collect children not yet seen
            frontier.clear();
            for children in children_map.values() {
                for &(child_id, _) in children {
                    if all_descendants.insert(child_id) {
                        frontier.push(child_id);
                    }
                }
            }
        }

        Ok(all_descendants)
    }

    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "HashSet iteration order doesn't matter - all stale IDs must \
                  be marked regardless of order"
    )]
    fn cascade_staleness(
        &self,
        staleness_map: &mut std::collections::HashMap<SchemaId, bool>,
    ) -> Result<(), Self::Error> {
        // Extract IDs of schemas currently marked as stale
        let stale_parent_ids: Vec<SchemaId> = staleness_map
            .iter()
            .filter_map(|(&id, &is_stale)| is_stale.then_some(id))
            .collect();

        if stale_parent_ids.is_empty() {
            return Ok(());
        }

        // Find all descendants of stale schemas
        let all_stale = self.list_descendants(&stale_parent_ids)?;

        // Mark all descendants as stale
        for id in all_stale {
            staleness_map.insert(id, true);
        }

        Ok(())
    }
}
