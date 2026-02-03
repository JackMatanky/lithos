//! Schema command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Schema write operations,
//! using the Database layer for persistence.

#![allow(
    clippy::missing_inline_in_public_items,
    clippy::elidable_lifetime_names,
    reason = "CQRS pattern: trait impls don't need inline"
)]

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
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;
    use crate::schema::aggregate::SchemaName;

    #[test]
    fn save_persists_schema_by_name() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("schema.redb");
        let db = Database::open(&path).unwrap();
        let cmd = Command::new(&db);

        let schema = Schema::new(
            Uuid::now_v7(),
            SchemaName::new("note".to_owned()).unwrap(),
            vec![],
        )
        .unwrap();

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
        let dir = tempdir().unwrap();
        let path = dir.path().join("schema.redb");
        let db = Database::open(&path).unwrap();
        let cmd = Command::new(&db);

        let schema = Schema::new(
            Uuid::now_v7(),
            SchemaName::new("project".to_owned()).unwrap(),
            vec![],
        )
        .unwrap();
        cmd.save(&schema).unwrap();

        cmd.delete("project").unwrap();

        let stored = db.get_owned::<Schema>("schemas", "project").unwrap();
        assert!(stored.is_none(), "Deleted schema should not exist");
    }
}
