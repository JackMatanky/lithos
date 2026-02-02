//! Note query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Note read operations,
//! using the Database layer for zero-copy reads.

use uuid::Uuid;

use super::{aggregate::Note, error::NoteError};
use crate::db::Database;

/// Query implementation for Note read operations.
///
/// Implements the Query port trait using the Database layer.
pub struct NoteQuery<'db> {
    db: &'db Database,
}

impl<'db> NoteQuery<'db> {
    /// Create a new `NoteQuery` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Finds a note by its UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use ``db.get_archived()`` for zero-copy read (hot path)
    /// 2. Deserialize if needed for mutation
    /// 3. Return Option<Note>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement note lookup by ID"
    )]
    pub fn find_by_id(&self, _id: Uuid) -> Result<Option<Note>, NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Find note by ID using `db.get()`")
    }

    /// Finds a note by its vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use path→ID index (multimap or secondary index)
    /// 2. Look up note by resolved ID
    /// 3. Return Option<Note>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement note lookup by path"
    )]
    pub fn find_by_path(&self, _path: &str) -> Result<Option<Note>, NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Find note by path using index")
    }

    /// Lists all notes in the vault.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Iterate over all notes in table
    /// 2. Use ``db.scan()`` or similar range query
    /// 3. Return Vec<Note>
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement list all notes"
    )]
    pub fn list(&self) -> Result<Vec<Note>, NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: List all notes using table scan")
    }
}
