//! Note query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Note read operations,
//! using the Database layer for zero-copy reads.

#![allow(
    clippy::same_name_method,
    reason = "CQRS pattern: public methods intentionally delegate to trait \
              impls with same names"
)]

use uuid::Uuid;

use super::{aggregate::Note, error::NoteError};
use crate::db::Database;

/// Query implementation for Note read operations.
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

    /// Finds a note by its UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    #[inline]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        let id_str = id.to_string();
        self.db
            .get_owned::<Note>("notes", &id_str)
            .map_err(|e: crate::db::DbError| NoteError::Storage(e.to_string()))
    }

    /// Finds a note by its vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    #[inline]
    pub fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError> {
        let ids = self.db.multimap_get("path_to_id", path).map_err(
            |e: crate::db::DbError| NoteError::Storage(e.to_string()),
        )?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Note>("notes", id_str).map_err(
                |e: crate::db::DbError| NoteError::Storage(e.to_string()),
            )
        } else {
            Ok(None)
        }
    }

    /// Lists all notes in the vault.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Note>, NoteError> {
        self.db
            .list_owned::<Note>("notes")
            .map_err(|e: crate::db::DbError| NoteError::Storage(e.to_string()))
    }
}

impl super::ports::Query for Query<'_> {
    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        self.find_by_id(id)
    }

    #[inline]
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError> {
        self.find_by_path(path)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Note>, NoteError> {
        self.list()
    }
}
