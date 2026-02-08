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
    clippy::panic_in_result_fn,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use std::collections::HashMap;

        use tempfile::{TempDir, tempdir};
        use uuid::Uuid;

        use super::*;
        use crate::note::{frontmatter::Frontmatter, value::FieldValue};

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

        #[expect(
            clippy::type_complexity,
            reason = "Fixture returns a complex tuple for test setup \
                      convenience."
        )]
        pub fn note_with_frontmatter()
        -> Result<(TempDir, Database, Uuid, Frontmatter), String> {
            let (dir, db) = test_db()?;
            let cmd = command::Command::new(&db);
            let mut note = cmd
                .create("notes/a.md".to_owned())
                .map_err(|e| e.to_string())?;
            let fm = complex_frontmatter()?;
            note.set_frontmatter(Some(fm.clone()));
            let id = Uuid::from(note.id());
            cmd.update(note).map_err(|e| e.to_string())?;
            Ok((dir, db, id, fm))
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
        fn find_by_id_returns_note_with_matching_id() -> Result<(), String> {
            let (_dir, db, id, _fm) = fixtures::note_with_frontmatter()?;
            let qry = Query::new(&db);

            let observed = qry
                .find_by_id(id)
                .map_err(|e| e.to_string())?
                .expect("Query by id should return Some(note)");
            assert_eq!(
                Uuid::from(observed.id()),
                id,
                "Observed id should match"
            );
            Ok(())
        }

        #[test]
        fn find_by_id_preserves_frontmatter() -> Result<(), String> {
            let (_dir, db, id, fm) = fixtures::note_with_frontmatter()?;
            let qry = Query::new(&db);

            let observed = qry
                .find_by_id(id)
                .map_err(|e| e.to_string())?
                .expect("Query by id should return Some(note)");
            assert_eq!(
                observed.frontmatter(),
                Some(&fm),
                "Frontmatter should roundtrip"
            );
            Ok(())
        }

        #[test]
        fn find_by_id_returns_none_for_missing_id() -> Result<(), String> {
            let (_dir, db) = fixtures::test_db()?;
            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            cmd.create("notes/a.md".to_owned()).map_err(|e| e.to_string())?;
            let miss = qry
                .find_by_id(fixtures::TEST_MISSING_ID)
                .map_err(|e| e.to_string())?;
            assert!(miss.is_none(), "Non-existent ID should return None");
            Ok(())
        }
    }
}
