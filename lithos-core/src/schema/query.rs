//! Schema query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Schema read operations,
//! using the Database layer for zero-copy reads.

use uuid::Uuid;

use super::{SCHEMAS_TABLE, aggregate::Schema, error::SchemaError};
use crate::db::Database;

/// Query implementation for Schema read operations.
///
/// Implements the Query port trait using the Database layer.
pub struct Query<'db> {
    db: &'db Database,
}

impl<'db> Query<'db> {
    /// Create a new `Query` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Query for Query<'_> {
    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    ///
    /// # Note
    /// Schema is stored by name, not ID. For now, returns `None`.
    /// A name→ID index would be needed for full implementation.
    #[inline]
    fn find_by_id(&self, _id: Uuid) -> Result<Option<Schema>, SchemaError> {
        // Schema is stored by name, not ID
        // For now, return None - would need name→id index for full
        // implementation
        Ok(None)
    }

    /// Find a schema by its unique name.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    #[inline]
    fn find_by_name(&self, name: &str) -> Result<Option<Schema>, SchemaError> {
        self.db.get_owned(SCHEMAS_TABLE, name).map_err(
            |e: crate::db::DbError| SchemaError::Storage(e.to_string()),
        )
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    #[inline]
    fn list(&self) -> Result<Vec<Schema>, SchemaError> {
        self.db
            .list_owned::<Schema>(SCHEMAS_TABLE)
            .map_err(|e| SchemaError::Storage(e.to_string()))
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use super::*;

        pub const TEST_SCHEMA_ID_A: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0201);
        pub const TEST_SCHEMA_ID_B: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0202);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("schema.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn schema_fixture(id: Uuid, name: &str) -> Result<Schema, String> {
            let name = SchemaName::new(name).map_err(|e| e.to_string())?;
            Schema::new(id, name, vec![]).map_err(|e| e.to_string())
        }
    }

    use std::collections::HashSet;

    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::schema::{aggregate::SchemaName, command, ports::Query as _};

    mod queries {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn find_by_id_returns_none_for_unindexed_schema() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let qry = Query::new(&db);

            let result = qry
                .find_by_id(fixtures::TEST_SCHEMA_ID_A)
                .expect("Query should succeed");
            assert!(result.is_none(), "find_by_id should return None");
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
            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            cmd.save(&schema).expect("Save should succeed");

            let stored =
                qry.find_by_name("note").expect("Query should succeed");
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
            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_A, "note")
                    .expect("Failed to create schema fixture");
            let schema_b =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_B, "project")
                    .expect("Failed to create schema fixture");

            cmd.save(&schema_a).expect("Save should succeed");
            cmd.save(&schema_b).expect("Save should succeed");

            let schemas = qry.list().expect("List should succeed");
            let names: HashSet<&str> =
                schemas.iter().map(|schema| schema.name().as_ref()).collect();
            assert_eq!(
                names,
                HashSet::from(["note", "project"]),
                "List should return all saved schemas"
            );
        }
    }
}
