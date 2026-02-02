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

    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    ///
    /// # Note
    /// Schema is stored by name, not ID. For now, returns `None`.
    /// A name→ID index would be needed for full implementation.
    #[inline]
    pub fn find_by_id(&self, _id: Uuid) -> Result<Option<Schema>, SchemaError> {
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
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Schema>, SchemaError> {
        self.db.get_owned("schemas", name).map_err(|e: crate::db::DbError| {
            SchemaError::Storage(e.to_string())
        })
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Schema>, SchemaError> {
        self.db
            .list_owned::<Schema>("schemas")
            .map_err(|e| SchemaError::Storage(e.to_string()))
    }
}
