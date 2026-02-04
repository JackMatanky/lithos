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
mod tests {
    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::schema::aggregate::SchemaName;

    const TEST_SCHEMA_ID_NOTE: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0101);
    const TEST_SCHEMA_ID_PROJECT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0102);

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("schema.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    fn schema_fixture(id: Uuid, name: &str) -> Result<Schema, String> {
        let schema_name =
            SchemaName::new(name.to_owned()).map_err(|e| e.to_string())?;
        Schema::new(id, schema_name, vec![]).map_err(|e| e.to_string())
    }

    #[test]
    fn save_persists_schema_by_name() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test db: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let schema_result = schema_fixture(TEST_SCHEMA_ID_NOTE, "note");
        assert!(
            schema_result.is_ok(),
            "Failed to create schema fixture: {schema_result:?}"
        );
        let Ok(schema) = schema_result else {
            return;
        };

        let save_result = cmd.save(&schema).map_err(|e| e.to_string());
        assert!(save_result.is_ok(), "Save should succeed: {save_result:?}");

        let stored_result = db
            .get_owned::<Schema>("schemas", "note")
            .map_err(|e| e.to_string());
        assert!(
            stored_result.is_ok(),
            "Read after save should succeed: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_some(), "Stored schema should exist");
        let Some(stored_schema) = stored else {
            return;
        };
        assert_eq!(
            stored_schema.name().as_ref(),
            "note",
            "Stored schema name should match"
        );
    }

    #[test]
    fn delete_removes_schema_by_name() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test db: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let schema_result = schema_fixture(TEST_SCHEMA_ID_PROJECT, "project");
        assert!(
            schema_result.is_ok(),
            "Failed to create schema fixture: {schema_result:?}"
        );
        let Ok(schema) = schema_result else {
            return;
        };
        let save_result = cmd.save(&schema).map_err(|e| e.to_string());
        assert!(save_result.is_ok(), "Save should succeed: {save_result:?}");

        let delete_result = cmd.delete("project").map_err(|e| e.to_string());
        assert!(
            delete_result.is_ok(),
            "Delete should succeed: {delete_result:?}"
        );

        let stored_result = db
            .get_owned::<Schema>("schemas", "project")
            .map_err(|e| e.to_string());
        assert!(
            stored_result.is_ok(),
            "Read after delete should succeed: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_none(), "Deleted schema should not exist");
    }
}
