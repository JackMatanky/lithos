//! Schema query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read operations
//! through the schema query port.

use super::{
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
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
/// use lithos_core::schema::{
///     RedbSchemaQuery,
///     adapter::query::QueryAdapter,
/// };
///
/// let db = todo!("Provide a Database instance");
/// let query = RedbSchemaQuery::new(QueryAdapter::new(&db));
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
    /// use lithos_core::schema::{
    ///     RedbSchemaQuery,
    ///     adapter::query::QueryAdapter,
    /// };
    ///
    /// let db = todo!("Provide a Database instance");
    /// let query = RedbSchemaQuery::new(QueryAdapter::new(&db));
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
        let id = self.query_port.lookup_id_by_name(name).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.find_by_id(id)
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
    /// let schemas: HashMap<_, _> = query.batch_find_by_ids(&ids)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn batch_find_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<std::collections::HashMap<SchemaId, Schema>, SchemaQueryError>
    {
        self.query_port.batch_find_by_ids(ids).map_err(|error| {
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
    /// let staleness: HashMap<_, _> = query.batch_is_stale(&schemas, bank_version)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn batch_is_stale(
        &self,
        schemas: &[super::ports::StalenessCheck],
        bank_version: BankVersion,
    ) -> Result<std::collections::HashMap<SchemaId, bool>, SchemaQueryError>
    {
        self.query_port.batch_is_stale(schemas, bank_version).map_err(|error| {
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
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
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
    /// query.batch_read(|_reader| Ok::<_, Box<dyn std::error::Error>>(()))?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn batch_read<R, F>(&self, f: F) -> Result<R, SchemaQueryError>
    where
        F: FnOnce(
            &crate::db::BatchReader,
        ) -> Result<R, <Q as schema_ports::Query>::Error>,
    {
        self.query_port.batch_read(f).map_err(|error| {
            SchemaQueryError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
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
            let name = SchemaName::new(name).map_err(|e| e.to_string())?;
            Schema::new(id, name, None, vec![]).map_err(|e| e.to_string())
        }
    }

    use std::collections::HashSet;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::schema::{
        RedbSchemaCommand, RedbSchemaQuery,
        adapter::{
            command::CommandAdapter, query::QueryAdapter,
            stored::StoredMetadata,
        },
        aggregate::{SchemaId, SchemaName},
    };

    mod queries {
        use super::*;

        #[test]
        fn find_by_id_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");

            cmd.save_one(&schema).expect("Save should succeed");

            let result = qry
                .find_by_id(fixtures::TEST_SCHEMA_ID_A)
                .expect("Query should succeed");
            assert!(result.is_some(), "find_by_id should return schema");
        }

        #[test]
        fn find_by_name_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");

            cmd.save_one(&schema).expect("Save should succeed");

            let name =
                SchemaName::new("note").expect("Failed to create schema name");
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
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let schema_b =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "project")
                    .expect("Failed to create schema fixture");

            cmd.save_batch(&[schema_a, schema_b]).expect("Save should succeed");

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
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

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
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let ts = Timestamp::from_secs(1_000_000);

            cmd.save_one(&schema).expect("Save should succeed");

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
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let stored_created = Timestamp::from_secs(1_000_000);
            let file_created = Timestamp::from_secs(2_000_000);

            cmd.save_batch_with_metadata(&[schema], &[StoredMetadata::new(
                BankVersion::initial(),
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
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            let result = qry.require_property_bank();

            assert!(result.is_err(), "Should return error when bank missing");
            assert!(
                matches!(result, Err(SchemaQueryError::PropertyBankNotFound)),
                "Should return PropertyBankNotFound error"
            );
        }

        #[test]
        fn require_property_bank_returns_bank_when_present() {
            use crate::schema::{
                RedbSchemaCommand, adapter::command::CommandAdapter,
                bank::PropertyBank,
            };

            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

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
        fn batch_find_by_ids_returns_multiple_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            // Save multiple schemas
            let schema1 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "article")
                    .expect("Failed to create schema fixture");
            let schema2 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "note")
                    .expect("Failed to create schema fixture");
            cmd.save_batch(&[schema1.clone(), schema2.clone()])
                .expect("Save should succeed");

            // WHEN: Batch finding by IDs
            let ids =
                vec![fixtures::TEST_SCHEMA_ID_A, fixtures::TEST_SCHEMA_ID_B];
            let result = qry.batch_find_by_ids(&ids);

            // THEN: All schemas are returned
            assert!(result.is_ok(), "Batch find should succeed");
            let found_schemas = result.expect("Should unwrap to HashMap");
            assert_eq!(found_schemas.len(), 2, "Should return both schemas");
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_A));
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_B));
        }

        #[test]
        fn batch_find_by_ids_skips_missing_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            // Save only one schema
            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "article")
                    .expect("Failed to create schema fixture");
            cmd.save_one(&schema).expect("Save should succeed");

            // WHEN: Batch finding with one missing ID
            let ids =
                vec![fixtures::TEST_SCHEMA_ID_A, fixtures::TEST_SCHEMA_ID_B];
            let result = qry.batch_find_by_ids(&ids);

            // THEN: Only existing schema is returned
            assert!(result.is_ok(), "Batch find should succeed");
            let found_schemas = result.expect("Should unwrap to HashMap");
            assert_eq!(
                found_schemas.len(),
                1,
                "Should return only existing schema"
            );
            assert!(found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_A));
            assert!(!found_schemas.contains_key(&fixtures::TEST_SCHEMA_ID_B));
        }

        #[test]
        fn batch_is_stale_checks_multiple_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            // Save two schemas with different timestamps
            let schema1 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "article")
                    .expect("Failed to create schema fixture");
            let schema2 =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "note")
                    .expect("Failed to create schema fixture");

            let ts_old = Timestamp::from_secs(1_000_000);
            let ts_new = Timestamp::from_secs(2_000_000);

            cmd.save_batch_with_metadata(&[schema1, schema2], &[
                StoredMetadata::new(BankVersion::initial(), None, Some(ts_old)),
                StoredMetadata::new(BankVersion::initial(), None, Some(ts_new)),
            ])
            .expect("Save should succeed");

            // WHEN: Checking staleness with newer file timestamps
            let checks = vec![
                (fixtures::TEST_SCHEMA_ID_A, None, Some(ts_new)),
                (fixtures::TEST_SCHEMA_ID_B, None, Some(ts_new)),
            ];
            let result = qry.batch_is_stale(&checks, BankVersion::initial());

            // THEN: First schema is stale (file modified), second is fresh
            assert!(result.is_ok(), "Batch staleness check should succeed");
            let staleness = result.expect("Should unwrap to HashMap");
            assert_eq!(staleness.len(), 2);
            assert_eq!(
                staleness.get(&fixtures::TEST_SCHEMA_ID_A),
                Some(&true),
                "Schema with older timestamp should be stale"
            );
            assert_eq!(
                staleness.get(&fixtures::TEST_SCHEMA_ID_B),
                Some(&false),
                "Schema with matching timestamp should be fresh"
            );
        }

        #[test]
        fn batch_is_stale_reports_missing_as_stale() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let qry = RedbSchemaQuery::new(QueryAdapter::new(&db));

            // WHEN: Checking staleness for schemas that don't exist
            let schemas = vec![
                (fixtures::TEST_SCHEMA_ID_A, None, None),
                (fixtures::TEST_SCHEMA_ID_B, None, None),
            ];
            let result = qry.batch_is_stale(&schemas, BankVersion::initial());

            // THEN: All missing schemas are reported as stale
            assert!(result.is_ok(), "Batch staleness check should succeed");
            let staleness = result.expect("Should unwrap to HashMap");
            assert_eq!(staleness.len(), 2);
            assert_eq!(
                staleness.get(&fixtures::TEST_SCHEMA_ID_A),
                Some(&true),
                "Missing schema should be stale"
            );
            assert_eq!(
                staleness.get(&fixtures::TEST_SCHEMA_ID_B),
                Some(&true),
                "Missing schema should be stale"
            );
        }
    }
}
