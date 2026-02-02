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
