//! Schema query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Schema read operations,
//! using the Database layer for zero-copy reads.

use uuid::Uuid;

use super::{aggregate::Schema, error::SchemaError};
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
        self.db.get_owned("schemas", name).map_err(|e: crate::db::DbError| {
            SchemaError::Storage(e.to_string())
        })
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    #[inline]
    fn list(&self) -> Result<Vec<Schema>, SchemaError> {
        self.db
            .list_owned::<Schema>("schemas")
            .map_err(|e| SchemaError::Storage(e.to_string()))
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test code uses unwrap/expect for clarity"
)]
mod tests {
    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::schema::{aggregate::SchemaName, command, ports::Query as _};

    const TEST_SCHEMA_ID_A: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0201);
    const TEST_SCHEMA_ID_B: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0202);

    fn test_db() -> (TempDir, Database) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("schema.redb");
        let db = Database::open(&path).unwrap();
        (dir, db)
    }

    fn schema_fixture(id: Uuid, name: &str) -> Schema {
        Schema::new(id, SchemaName::new(name.to_owned()).unwrap(), vec![])
            .unwrap()
    }

    #[test]
    fn find_by_id_returns_none_for_unindexed_schema() {
        let (_dir, db) = test_db();
        let qry = Query::new(&db);

        let result = qry.find_by_id(TEST_SCHEMA_ID_A).unwrap();
        assert!(result.is_none(), "find_by_id should return None");
    }

    #[test]
    fn find_by_name_returns_saved_schema() {
        let (_dir, db) = test_db();
        let cmd = command::Command::new(&db);
        let qry = Query::new(&db);

        let schema = schema_fixture(TEST_SCHEMA_ID_A, "note");
        cmd.save(&schema).unwrap();

        let stored = qry.find_by_name("note").unwrap();
        let stored_schema = stored.expect("Schema should be found by name");
        assert_eq!(
            stored_schema.name().as_ref(),
            "note",
            "Stored schema name should match"
        );
    }

    #[test]
    fn list_returns_all_saved_schemas() {
        let (_dir, db) = test_db();
        let cmd = command::Command::new(&db);
        let qry = Query::new(&db);

        let schema_a = schema_fixture(TEST_SCHEMA_ID_A, "note");
        let schema_b = schema_fixture(TEST_SCHEMA_ID_B, "project");

        cmd.save(&schema_a).unwrap();
        cmd.save(&schema_b).unwrap();

        let schemas = qry.list().unwrap();
        assert_eq!(schemas.len(), 2, "List should return all saved schemas");
    }
}
