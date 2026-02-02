//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

#![allow(
    clippy::same_name_method,
    clippy::missing_inline_in_public_items,
    reason = "CQRS pattern: trait impls don't need inline"
)]

use uuid::Uuid;

use super::{aggregate::Note, error::NoteError};
use crate::db::Database;

/// Command implementation for Note write operations.
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

    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if note creation fails validation or persistence.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Validate path
    /// 2. Create Note aggregate
    /// 3. Persist to database using `db.put()`
    /// 4. Update indexes (tags, backlinks)
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement note creation"
    )]
    pub fn create(&self, _path: String) -> Result<Note, NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Create note and persist to database")
    }

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteError` if note deletion fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Delete from main table using `db.delete()`
    /// 2. Clean up indexes (tags, backlinks)
    /// 3. Emit `NoteDeleted` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement note deletion"
    )]
    pub fn delete(&self, _id: Uuid) -> Result<(), NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Delete note and clean up indexes")
    }

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `NoteError` if note update fails validation or persistence.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Validate note
    /// 2. Persist to database using `db.put()`
    /// 3. Update indexes (tags, backlinks) - delta calculation
    /// 4. Emit `NoteUpdated` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement note update"
    )]
    pub fn update(&self, _note: Note) -> Result<Note, NoteError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Update note and refresh indexes")
    }
}

impl super::ports::Command for Command<'_> {
    #[inline]
    fn create(&self, path: String) -> Result<Note, NoteError> {
        self.create(path)
    }

    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), NoteError> {
        self.delete(id)
    }

    #[inline]
    fn update(&self, note: Note) -> Result<Note, NoteError> {
        self.update(note)
    }
}
