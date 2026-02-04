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
}

impl super::ports::Query for Query<'_> {
    /// Finds a note by its UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
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
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError> {
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
    fn list(&self) -> Result<Vec<Note>, NoteError> {
        self.db
            .list_owned::<Note>("notes")
            .map_err(|e: crate::db::DbError| NoteError::Storage(e.to_string()))
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Expect/unwrap is permitted in Arrange phase of tests."
)]
mod tests {
    use std::collections::HashMap;

    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::note::{
        command,
        frontmatter::{FieldValue, Frontmatter},
        ports::{Command as _, Query as _},
    };

    const TEST_MISSING_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0901);

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("test.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    #[test]
    fn cqrs_roundtrip_preserves_frontmatter_in_note_archive() {
        let (_dir, db) = test_db().expect("Failed to create test DB");

        let cmd = command::Command::new(&db);
        let qry = Query::new(&db);

        let mut note =
            cmd.create("notes/a.md".to_owned()).expect("Create should succeed");

        let fm = Frontmatter::new(HashMap::from([(
            "root".to_owned(),
            FieldValue::Object(HashMap::from([(
                "nested".to_owned(),
                FieldValue::Array(vec![
                    FieldValue::String("x".to_owned()),
                    FieldValue::Boolean(true),
                ]),
            )])),
        )]));
        let fm = fm.expect("Frontmatter construction should succeed");
        note.set_frontmatter(Some(fm.clone()));

        let id = note.id;
        cmd.update(note).expect("Update should succeed");

        let observed = qry.find_by_id(id).expect("Query by id should succeed");
        let observed = observed.expect("Query by id should return Some(note)");
        assert_eq!(observed.id, id);
        assert_eq!(observed.frontmatter, Some(fm));

        // Sanity: changing the id misses the record
        let miss = qry
            .find_by_id(TEST_MISSING_ID)
            .expect("Query should succeed even for non-existent ID");
        assert!(miss.is_none(), "Non-existent ID should return None");
    }
}
