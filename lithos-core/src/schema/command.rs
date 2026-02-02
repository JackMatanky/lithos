//! Schema command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Schema write operations,
//! using the Database layer for persistence.

use uuid::Uuid;

use super::{aggregate::Schema, error::SchemaError};
use crate::db::Database;

/// Command implementation for Schema write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct SchemaCommand<'db> {
    db: &'db Database,
}

impl<'db> SchemaCommand<'db> {
    /// Create a new `SchemaCommand` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns `SchemaError` if deletion fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Delete from main table using ``db.delete()``
    /// 2. Clean up name→ID index
    /// 3. Emit `SchemaDeleted` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement schema deletion"
    )]
    pub fn delete(&self, _id: Uuid) -> Result<(), SchemaError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Delete schema and clean up indexes")
    }

    /// Save a schema to persistence.
    ///
    /// # Errors
    /// Returns `SchemaError` if saving fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Validate schema
    /// 2. Persist to database using ``db.put()``
    /// 3. Update name→ID index
    /// 4. Emit `SchemaCreated` or `SchemaUpdated` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement schema save"
    )]
    pub fn save(&self, _schema: Schema) -> Result<(), SchemaError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Save schema and update indexes")
    }
}
