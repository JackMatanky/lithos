//! Schema command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Schema write operations,
//! using the Database layer for persistence.

use super::{aggregate::Schema, error::SchemaError};
use crate::db::Database;

/// Command implementation for Schema write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct Command<'db> {
    db: &'db Database,
}

impl<'db> Command<'db> {
    /// Create a new `Command` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Delete a schema by name.
    ///
    /// # Errors
    /// Returns `SchemaError` if deletion fails.
    #[inline]
    pub fn delete(&self, name: &str) -> Result<(), SchemaError> {
        self.db
            .delete("schemas", name)
            .map_err(|e| SchemaError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Save a schema to persistence.
    ///
    /// # Errors
    /// Returns `SchemaError` if saving fails.
    #[inline]
    pub fn save(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Get schema name as key
        let name = schema.name().as_ref();

        // Save to database
        self.db.put("schemas", name, schema).map_err(|e: crate::db::DbError| {
            SchemaError::Storage(e.to_string())
        })
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
    use crate::schema::aggregate::SchemaName;

    const TEST_SCHEMA_ID_NOTE: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0101);
    const TEST_SCHEMA_ID_PROJECT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0102);

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
    fn save_persists_schema_by_name() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let schema = schema_fixture(TEST_SCHEMA_ID_NOTE, "note");

        cmd.save(&schema).unwrap();

        let stored = db.get_owned::<Schema>("schemas", "note").unwrap();
        let stored_schema = stored.expect("Stored schema should exist");
        assert_eq!(
            stored_schema.name().as_ref(),
            "note",
            "Stored schema name should match"
        );
    }

    #[test]
    fn delete_removes_schema_by_name() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let schema = schema_fixture(TEST_SCHEMA_ID_PROJECT, "project");
        cmd.save(&schema).unwrap();

        cmd.delete("project").unwrap();

        let stored = db.get_owned::<Schema>("schemas", "project").unwrap();
        assert!(stored.is_none(), "Deleted schema should not exist");
    }
}
