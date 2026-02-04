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
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for deterministic fixtures."
)]
mod tests {
    mod fixtures {
        use std::collections::HashMap;

        use tempfile::{TempDir, tempdir};
        use uuid::Uuid;

        use super::*;
        use crate::note::frontmatter::{FieldValue, Frontmatter};

        pub const TEST_MISSING_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0901);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("test.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn complex_frontmatter() -> Result<Frontmatter, String> {
            Frontmatter::new(HashMap::from([(
                "root".to_owned(),
                FieldValue::Object(HashMap::from([(
                    "nested".to_owned(),
                    FieldValue::Array(vec![
                        FieldValue::String("x".to_owned()),
                        FieldValue::Boolean(true),
                    ]),
                )])),
            )]))
            .map_err(|e| e.to_string())
        }
    }

    use super::*;
    use crate::note::{
        command,
        ports::{Command as _, Query as _},
    };

    mod query {
        use super::*;

        #[test]
        fn find_by_id_returns_note_with_matching_id() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");

            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            let mut note = cmd
                .create("notes/a.md".to_owned())
                .expect("Create should succeed");
            let fm = fixtures::complex_frontmatter()
                .expect("Frontmatter construction should succeed");
            note.set_frontmatter(Some(fm));

            let id = note.id;
            let update_result = cmd.update(note);
            assert!(
                update_result.is_ok(),
                "Update should succeed, got: {update_result:?}"
            );

            let observed = qry
                .find_by_id(id)
                .expect("Query by id should succeed")
                .expect("Query by id should return Some(note)");
            assert_eq!(observed.id, id, "Observed id should match");
        }

        #[test]
        fn find_by_id_preserves_frontmatter() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");

            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            let mut note = cmd
                .create("notes/a.md".to_owned())
                .expect("Create should succeed");
            let fm = fixtures::complex_frontmatter()
                .expect("Frontmatter construction should succeed");
            note.set_frontmatter(Some(fm.clone()));

            let id = note.id;
            let update_result = cmd.update(note);
            assert!(
                update_result.is_ok(),
                "Update should succeed, got: {update_result:?}"
            );

            let observed = qry
                .find_by_id(id)
                .expect("Query by id should succeed")
                .expect("Query by id should return Some(note)");
            assert_eq!(
                observed.frontmatter,
                Some(fm),
                "Frontmatter should roundtrip"
            );
        }

        #[test]
        fn find_by_id_returns_none_for_missing_id() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            cmd.create("notes/a.md".to_owned()).expect("Create should succeed");
            let miss = qry
                .find_by_id(fixtures::TEST_MISSING_ID)
                .expect("Query by id should succeed");
            assert!(miss.is_none(), "Non-existent ID should return None");
        }
    }
}
