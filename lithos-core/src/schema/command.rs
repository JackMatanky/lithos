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
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use super::*;

        pub const TEST_SCHEMA_ID_NOTE: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0101);
        pub const TEST_SCHEMA_ID_PROJECT: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0102);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("schema.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn schema_fixture(id: Uuid, name: &str) -> Result<Schema, String> {
            let schema_name =
                SchemaName::new(name).map_err(|e| e.to_string())?;
            Schema::new(id, schema_name, vec![]).map_err(|e| e.to_string())
        }
    }

    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::schema::aggregate::SchemaName;

    mod persistence {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn save_persists_schema_by_name() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = Command::new(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");

            cmd.save(&schema).expect("Save should succeed");

            let stored = db
                .get_owned::<Schema>("schemas", "note")
                .expect("Read after save should succeed");
            let stored_schema = stored.expect("Stored schema should exist");
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
        fn delete_removes_schema_by_name() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = Command::new(&db);

            let schema = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");
            cmd.save(&schema).expect("Save should succeed");

            cmd.delete("project").expect("Delete should succeed");

            let stored = db
                .get_owned::<Schema>("schemas", "project")
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted schema should not exist");
        }
    }
}
