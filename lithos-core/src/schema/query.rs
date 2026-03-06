//! Schema query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read operations
//! through the schema query port.

use std::time::SystemTime;

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    bank::{BankVersion, PropertyBank},
    error::SchemaQueryError,
    ports as schema_ports,
};

/// Query implementation for schema read operations.
///
/// This struct is generic over a storage port to support multiple backends.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::{self, adapter};
///
/// let db = todo!("Provide a Database instance");
/// let query = schema::Query::new(adapter::Query::new(&db));
/// ```
pub struct Query<Q> {
    query_port: Q,
}

impl<Q> Query<Q> {
    /// Create a new `Query` with a storage port.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::schema::{self, adapter};
    ///
    /// let db = todo!("Provide a Database instance");
    /// let query = schema::Query::new(adapter::Query::new(&db));
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(query_port: Q) -> Self {
        Self {
            query_port,
        }
    }
}

impl<Q> Query<Q>
where
    Q: schema_ports::Query,
    Q::Error: Into<crate::db::DbError>,
{
    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// let _ = query.find_by_id(id)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn find_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaQueryError> {
        self.query_port.find_by_id(id).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Find a schema by its unique name.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// # let name = lithos_core::schema::aggregate::SchemaName::new("task")?;
    /// let _ = query.find_by_name(&name)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, SchemaQueryError> {
        let id = self.query_port.find_id_by_name(name).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.find_by_id(id)
    }

    /// Find a schema name-to-ID mapping.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use lithos_core::schema::aggregate::SchemaName;
    /// # let query = todo!("Provide a Query instance");
    /// # let name = SchemaName::try_new("note")?;
    /// let id = query.find_id_by_name(&name)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn find_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaQueryError> {
        self.query_port.find_id_by_name(name).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Find multiple schemas by their IDs in a single transaction.
    ///
    /// This is more efficient than calling `find_by_id` multiple times.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use std::collections::HashMap;
    /// # let query = todo!("Provide a Query instance");
    /// # let ids = vec![lithos_core::schema::aggregate::SchemaId::new()];
    /// let schemas: HashMap<_, _> = query.find_many_by_ids(&ids)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn find_many_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<std::collections::HashMap<SchemaId, Schema>, SchemaQueryError>
    {
        self.query_port.find_many_by_ids(ids).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Check staleness for multiple schemas in a single transaction.
    ///
    /// This is more efficient than calling `is_schema_stale` multiple times.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use std::collections::HashMap;
    /// # let query = todo!("Provide a Query instance");
    /// # let bank_version = lithos_core::schema::bank::BankVersion::initial();
    /// # let schemas = vec![(lithos_core::schema::aggregate::SchemaId::new(), None, None)];
    /// let staleness: HashMap<_, _> = query.are_many_stale(&schemas, bank_version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn are_many_stale(
        &self,
        schemas: &[super::ports::StalenessCheck],
        bank_version: BankVersion,
    ) -> Result<std::collections::HashMap<SchemaId, bool>, SchemaQueryError>
    {
        self.query_port.are_many_stale(schemas, bank_version).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// let _ = query.list()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn list(&self) -> Result<Vec<Schema>, SchemaQueryError> {
        self.query_port.list().map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// List all schema name-to-ID pairs.
    ///
    /// This is a bulk operation that scans the entire name index in one pass.
    /// Use this instead of `find_by_name` when preloading all mappings.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// let _ = query.list_name_id_pairs()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn list_name_id_pairs(
        &self,
    ) -> Result<Vec<schema_ports::NameIdPair>, SchemaQueryError> {
        self.query_port.list_name_id_pairs().map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Find the `PropertyBank` registry.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// let _ = query.get_property_bank()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaQueryError> {
        self.query_port.get_property_bank().map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Get a property by its ID from the current `PropertyBank`.
    ///
    /// Returns `None` if the property doesn't exist or if `PropertyBank`
    /// is not loaded.
    ///
    /// # Errors
    /// Returns `SchemaQueryError::Storage` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use lithos_core::schema::property::PropertyId;
    /// # let query = todo!("Provide a Query instance");
    /// # let id = PropertyId::new();
    /// let property = query.get_property_by_id(id)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn get_property_by_id(
        &self,
        id: super::property::PropertyId,
    ) -> Result<Option<super::property::Property>, SchemaQueryError> {
        self.query_port.get_property_by_id(id).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Returns the `PropertyBank` or an error if it doesn't exist.
    ///
    /// This is a convenience method that returns a clear error when the
    /// `PropertyBank` is missing, rather than requiring callers to unwrap
    /// the `Option` returned by `get_property_bank()`.
    ///
    /// # Errors
    /// Returns `SchemaQueryError::PropertyBankNotFound` if `PropertyBank`
    /// doesn't exist, or `SchemaQueryError::Storage` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// let bank = query.require_property_bank()?;
    /// // Use bank without unwrapping Option
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn require_property_bank(
        &self,
    ) -> Result<PropertyBank, SchemaQueryError> {
        self.get_property_bank()?.ok_or(SchemaQueryError::PropertyBankNotFound)
    }

    /// Returns `true` if the stored schema for `id` is stale.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// # let bank_version = lithos_core::schema::bank::BankVersion::initial();
    /// let _ = query.is_schema_stale(id, None, None, bank_version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_schema_stale(
        &self,
        id: SchemaId,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        bank_version: BankVersion,
    ) -> Result<bool, SchemaQueryError> {
        self.query_port
            .is_schema_stale(id, created_at, modified_at, bank_version)
            .map_err(|error| {
                SchemaQueryError::Storage(Into::<crate::db::DbError>::into(
                    error,
                ))
            })
    }

    /// Returns `true` if the stored bank version differs from `version`.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// # let version = lithos_core::schema::bank::BankVersion::initial();
    /// let _ = query.is_bank_stale(version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_bank_stale(
        &self,
        version: BankVersion,
    ) -> Result<bool, SchemaQueryError> {
        self.query_port.is_bank_stale(version).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Execute multiple read operations within a single transaction.
    ///
    /// This amortizes transaction creation cost across multiple reads,
    /// improving performance for batch operations.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # let query = todo!("Provide a Query instance");
    /// query.read_many(|_reader| Ok::<_, Box<dyn std::error::Error>>(()))?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn read_many<R, F>(&self, f: F) -> Result<R, SchemaQueryError>
    where
        F: FnOnce(
            &crate::db::BatchReader,
        ) -> Result<R, <Q as schema_ports::Query>::Error>,
    {
        self.query_port.read_many(f).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Find all children of the given parent schemas.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if storage operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use lithos_core::schema::ports::InheritanceMap;
    /// # let query = todo!("Provide a Query instance");
    /// # let parent_id = lithos_core::schema::aggregate::SchemaId::new();
    /// let children_map: InheritanceMap = query.list_children(&[parent_id])?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn list_children(
        &self,
        parent_ids: &[SchemaId],
    ) -> Result<super::ports::InheritanceMap, SchemaQueryError> {
        self.query_port.list_children(parent_ids).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// List all descendants (transitive children) of the given parent schemas.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if storage operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use std::collections::HashSet;
    /// # let query = todo!("Provide a Query instance");
    /// # let parent_id = lithos_core::schema::aggregate::SchemaId::new();
    /// let descendants: HashSet<_> = query.list_descendants(&[parent_id])?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn list_descendants(
        &self,
        parent_ids: &[SchemaId],
    ) -> Result<std::collections::HashSet<SchemaId>, SchemaQueryError> {
        self.query_port.list_descendants(parent_ids).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Cascade staleness to descendants in the staleness map.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if storage operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::query::Query;
    /// # use std::collections::HashMap;
    /// # let query = todo!("Provide a Query instance");
    /// # let mut staleness_map: HashMap<_, _> = HashMap::new();
    /// query.cascade_staleness(&mut staleness_map)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn cascade_staleness(
        &self,
        staleness_map: &mut std::collections::HashMap<SchemaId, bool>,
    ) -> Result<(), SchemaQueryError> {
        self.query_port.cascade_staleness(staleness_map).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }
}

// Methods specific to the adapter implementation
impl Query<crate::schema::adapter::Query<'_>> {
    /// Get the source file hash for a schema (adapter-specific method).
    ///
    /// Returns `None` if the schema metadata is not found.
    ///
    /// This is used for two-tier staleness detection: if timestamp changed but
    /// hash is the same, it's a touch-only change.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if database access fails.
    pub(crate) fn get_schema_hash(
        &self,
        id: SchemaId,
    ) -> Result<Option<crate::schema::hash::Blake3Hash>, SchemaQueryError> {
        self.query_port.get_schema_hash(id).map_err(SchemaQueryError::Storage)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    use crate::schema::hash::Blake3Hash;

    mod fixtures {
        use uuid::Uuid;

        use super::*;
        use crate::db::Database;

        pub const TEST_SCHEMA_ID_A: SchemaId = SchemaId::from_uuid(
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0201),
        );
        pub const TEST_SCHEMA_ID_B: SchemaId = SchemaId::from_uuid(
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0202),
        );

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("schema.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn schema_fixture(
            id: SchemaId,
            name: &str,
        ) -> Result<Schema, String> {
            let name = SchemaName::try_new(name).map_err(|e| e.to_string())?;
            Schema::try_new(id, name, None, vec![]).map_err(|e| e.to_string())
        }
    }

    use std::collections::HashSet;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::schema::{
        self as schema_mod, adapter,
        adapter::stored::StoredMetadata,
        aggregate::{SchemaId, SchemaName},
    };

    mod queries {
        use std::time::Duration;

        use super::*;

        #[test]
        fn find_by_id_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");

            cmd.save(&schema).expect("Save should succeed");

            let result = qry
                .find_by_id(fixtures::TEST_SCHEMA_ID_A)
                .expect("Query should succeed");
            assert!(result.is_some(), "find_by_id should return schema");
        }

        #[test]
        fn find_by_name_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");

            cmd.save(&schema).expect("Save should succeed");

            let name = SchemaName::try_new("note")
                .expect("Failed to create schema name");
            let stored = qry.find_by_name(&name).expect("Query should succeed");
            let stored_schema = stored.expect("Schema should be found by name");
            assert_eq!(
                stored_schema.name().as_ref(),
                "note",
                "Stored schema name should match"
            );
        }

        #[test]
        fn list_returns_all_saved_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let schema_b =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "project")
                    .expect("Failed to create schema fixture");

            cmd.save_many(&[schema_a, schema_b]).expect("Save should succeed");

            let schemas = qry.list().expect("List should succeed");
            let names: HashSet<&str> =
                schemas.iter().map(|schema| schema.name().as_ref()).collect();
            assert_eq!(
                names,
                HashSet::from(["note", "project"]),
                "List should return all saved schemas"
            );
        }

        #[test]
        fn is_schema_stale_returns_true_for_missing_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let missing_id = fixtures::TEST_SCHEMA_ID_A;
            let stale = qry
                .is_schema_stale(missing_id, None, None, BankVersion::initial())
                .expect("Staleness check should succeed");
            assert!(stale, "Missing schema should be stale");
        }

        #[test]
        fn is_schema_stale_returns_false_for_fresh_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let ts = SystemTime::now();

            cmd.save(&schema).expect("Save should succeed");

            let stale = qry
                .is_schema_stale(
                    fixtures::TEST_SCHEMA_ID_A,
                    None,
                    Some(ts),
                    BankVersion::initial(),
                )
                .expect("Staleness check should succeed");
            assert!(!stale, "Freshly saved schema should not be stale");
        }

        #[test]
        fn is_schema_stale_returns_true_for_created_at_mismatch() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let stored_created = SystemTime::now();
            // Use a different timestamp to simulate mismatch
            let file_created = stored_created + Duration::from_secs(1);

            cmd.save_many_with_metadata(&[schema], &[StoredMetadata::new(
                BankVersion::initial(),
                Blake3Hash::zero(),
                Some(stored_created),
                None,
            )])
            .expect("Save should succeed");

            let stale = qry
                .is_schema_stale(
                    fixtures::TEST_SCHEMA_ID_A,
                    Some(file_created),
                    None,
                    BankVersion::initial(),
                )
                .expect("Staleness check should succeed");
            assert!(stale, "Created-at mismatch should be stale");
        }

        #[test]
        fn require_property_bank_returns_error_when_missing() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let result = qry.require_property_bank();

            assert!(result.is_err(), "Should return error when bank missing");
            assert!(
                matches!(result, Err(SchemaQueryError::PropertyBankNotFound)),
                "Should return PropertyBankNotFound error"
            );
        }

        #[test]
        fn require_property_bank_returns_bank_when_present() {
            use crate::schema::bank::PropertyBank;

            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            // Save PropertyBank
            let bank = PropertyBank::new();
            cmd.save_property_bank(&bank).expect("Save should succeed");

            // WHEN: Requiring PropertyBank
            let result = qry.require_property_bank();

            // THEN: Bank is returned without Option wrapping
            assert!(result.is_ok(), "Should return bank when present");
            let loaded_bank = result.expect("Should unwrap to PropertyBank");
            assert_eq!(loaded_bank.version(), bank.version());
        }

        #[test]
        fn find_many_by_ids_returns_multiple_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            // Save multiple schemas
            let schema1 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "article")
                    .expect("Failed to create schema fixture");
            let schema2 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "note")
                    .expect("Failed to create schema fixture");
            cmd.save_many(&[schema1.clone(), schema2.clone()])
                .expect("Save should succeed");

            // WHEN: Finding many by IDs
            let ids =
                vec![fixtures::TEST_SCHEMA_ID_A, fixtures::TEST_SCHEMA_ID_B];
            let result = qry.find_many_by_ids(&ids);

            // THEN: All schemas are returned
            assert!(result.is_ok(), "Find many should succeed");
            let found_schemas = result.expect("Should unwrap to HashMap");
            assert_eq!(found_schemas.len(), 2, "Should return both schemas");
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_A));
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_B));
        }

        #[test]
        fn find_many_by_ids_skips_missing_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            cmd.save(&schema).expect("Save should succeed");

            // WHEN: Finding many by IDs including a missing one
            let ids =
                vec![fixtures::TEST_SCHEMA_ID_A, fixtures::TEST_SCHEMA_ID_B];
            let result = qry.find_many_by_ids(&ids);

            // THEN: Only found schema is returned
            assert!(result.is_ok(), "Find many should succeed");
            let found_schemas = result.expect("Should unwrap to HashMap");
            assert_eq!(found_schemas.len(), 1, "Should return only one schema");
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_A));
        }

        #[test]
        fn are_many_stale_checks_multiple_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            let ts_old = SystemTime::now();
            let ts_new = ts_old + Duration::from_secs(1);

            // Save two schemas
            let schema1 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note1")
                    .expect("Failed to create schema fixture");
            let schema2 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "note2")
                    .expect("Failed to create schema fixture");

            cmd.save_many_with_metadata(&[schema1, schema2], &[
                StoredMetadata::new(
                    BankVersion::initial(),
                    Blake3Hash::zero(),
                    None,
                    Some(ts_old),
                ),
                StoredMetadata::new(
                    BankVersion::initial(),
                    Blake3Hash::zero(),
                    None,
                    Some(ts_new),
                ),
            ])
            .expect("Save should succeed");

            // WHEN: Checking many for staleness
            let checks = vec![
                (fixtures::TEST_SCHEMA_ID_A, None, Some(ts_new)), /* Stale (file newer) */
                (fixtures::TEST_SCHEMA_ID_B, None, Some(ts_old)), /* Fresh (file older) */
            ];
            let result = qry.are_many_stale(&checks, BankVersion::initial());

            // THEN: Staleness is reported correctly
            assert!(result.is_ok(), "Staleness check should succeed");
            let staleness = result.expect("Should unwrap to HashMap");
            assert_eq!(staleness.get(&fixtures::TEST_SCHEMA_ID_A), Some(&true));
            assert_eq!(
                staleness.get(&fixtures::TEST_SCHEMA_ID_B),
                Some(&false)
            );
        }

        #[test]
        fn are_many_stale_reports_missing_as_stale() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let qry = schema_mod::Query::new(adapter::Query::new(&db));

            // WHEN: Checking staleness for a schema that doesn't exist
            let schemas = vec![(fixtures::TEST_SCHEMA_ID_A, None, None)];
            let result = qry.are_many_stale(&schemas, BankVersion::initial());

            // THEN: It is reported as stale
            assert!(result.is_ok(), "Staleness check should succeed");
            let staleness = result.expect("Should unwrap to HashMap");
            assert_eq!(staleness.get(&fixtures::TEST_SCHEMA_ID_A), Some(&true));
        }
    }
}
