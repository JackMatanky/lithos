//! Schema query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read operations
//! through the schema query port.

use super::{
    aggregate::{ResolutionMetadata, Schema, SchemaId, SchemaName},
    error::SchemaQueryError,
    ports as schema_ports,
};

/// Query implementation for schema read operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Query<Q> {
    query_port: Q,
}

impl<Q> Query<Q> {
    /// Create a new `Query` with a storage port.
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
    #[inline]
    pub fn find_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaQueryError> {
        self.query_port
            .find_by_id(id)
            .map_err(|error| SchemaQueryError::Storage(error.into()))
    }

    /// Find a schema by its unique name.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, SchemaQueryError> {
        let id = self
            .query_port
            .lookup_id_by_name(name)
            .map_err(|error| SchemaQueryError::Storage(error.into()))?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.find_by_id(id)
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Schema>, SchemaQueryError> {
        self.query_port
            .list()
            .map_err(|error| SchemaQueryError::Storage(error.into()))
    }

    /// Access a schema by ID as archived data.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn with_archived_by_id<F, R>(
        &self,
        id: SchemaId,
        f: F,
    ) -> Result<Option<R>, SchemaQueryError>
    where
        F: for<'archived> FnOnce(Q::Archived<'archived>) -> R,
    {
        self.query_port
            .with_archived_by_id(id, f)
            .map_err(|error| SchemaQueryError::Storage(error.into()))
    }

    /// Access a schema by name as archived data.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn with_archived_by_name<F, R>(
        &self,
        name: &SchemaName,
        f: F,
    ) -> Result<Option<R>, SchemaQueryError>
    where
        F: for<'archived> FnOnce(Q::Archived<'archived>) -> R,
    {
        self.query_port
            .with_archived_by_name(name, f)
            .map_err(|error| SchemaQueryError::Storage(error.into()))
    }

    /// List all resolution metadata entries.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn list_metadata(
        &self,
    ) -> Result<Vec<ResolutionMetadata>, SchemaQueryError> {
        self.query_port
            .list_metadata()
            .map_err(|error| SchemaQueryError::Storage(error.into()))
    }

    /// Find resolution metadata by schema ID.
    ///
    /// # Errors
    /// Returns `SchemaQueryError` if query fails.
    #[inline]
    pub fn find_metadata_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<ResolutionMetadata>, SchemaQueryError> {
        self.query_port
            .find_metadata_by_id(id)
            .map_err(|error| SchemaQueryError::Storage(error.into()))
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
            Schema::new(id, name, vec![]).map_err(|e| e.to_string())
        }
    }

    use std::collections::HashSet;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::schema::{
        RedbSchemaCommand, RedbSchemaQuery,
        aggregate::{BankVersion, SchemaName, Timestamp},
    };

    mod queries {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn find_by_id_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new_redb(&db);
            let qry = RedbSchemaQuery::new_redb(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

            let result = qry
                .find_by_id(fixtures::TEST_SCHEMA_ID_A)
                .expect("Query should succeed");
            assert!(result.is_some(), "find_by_id should return schema");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn find_by_name_returns_saved_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new_redb(&db);
            let qry = RedbSchemaQuery::new_redb(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

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
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn list_returns_all_saved_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new_redb(&db);
            let qry = RedbSchemaQuery::new_redb(&db);

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let schema_b =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "project")
                    .expect("Failed to create schema fixture");

            let metadata_a = ResolutionMetadata::new(
                schema_a.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            let metadata_b = ResolutionMetadata::new(
                schema_b.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );

            cmd.save_with_metadata(&schema_a, &metadata_a)
                .expect("Save should succeed");
            cmd.save_with_metadata(&schema_b, &metadata_b)
                .expect("Save should succeed");

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
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn list_metadata_returns_saved_entries() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new_redb(&db);
            let qry = RedbSchemaQuery::new_redb(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );

            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

            let items = qry.list_metadata().expect("List should succeed");
            assert_eq!(
                items.len(),
                1,
                "Expected one metadata entry after save"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn find_metadata_by_id_returns_entry() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = RedbSchemaCommand::new_redb(&db);
            let qry = RedbSchemaQuery::new_redb(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );

            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

            let stored = qry
                .find_metadata_by_id(schema.id())
                .expect("Lookup should succeed");
            assert!(stored.is_some(), "Metadata should be found");
        }
    }
}
