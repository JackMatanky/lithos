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
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use ``db.get_archived()`` for zero-copy read
    /// 2. Deserialize if needed
    /// 3. Return Option<Schema>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement schema lookup by ID"
    )]
    pub fn find_by_id(&self, _id: Uuid) -> Result<Option<Schema>, SchemaError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Find schema by ID using `db.get()`")
    }

    /// Find a schema by its unique name.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use name→ID index to resolve name to ID
    /// 2. Look up schema by resolved ID
    /// 3. Return Option<Schema>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement schema lookup by name"
    )]
    pub fn find_by_name(
        &self,
        _name: &str,
    ) -> Result<Option<Schema>, SchemaError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Find schema by name using index")
    }

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Iterate over all schemas in table
    /// 2. Use ``db.scan()`` or similar range query
    /// 3. Return Vec<Schema>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement list all schemas"
    )]
    pub fn list(&self) -> Result<Vec<Schema>, SchemaError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: List all schemas using table scan")
    }
}
