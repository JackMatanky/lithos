//! Note query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Note read operations,
//! using the Database layer for zero-copy reads.

use uuid::Uuid;

use super::{aggregate::Note, error::NoteQueryError};
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

    /// Helper method to find notes by any task index.
    ///
    /// This consolidates the logic for task index lookups.
    fn find_notes_by_task_index(
        &self,
        index_name: &str,
        index_key: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        let note_refs = self
            .db
            .multimap_get(index_name, index_key)
            .map_err(NoteQueryError::Storage)?;

        let mut notes = Vec::with_capacity(note_refs.len());
        for note_id_str in note_refs {
            if let Some(note) = self
                .db
                .get_owned::<Note>("notes", &note_id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }
}

impl super::ports::Query for Query<'_> {
    type NoteArchived<'archived> = &'archived rkyv::Archived<Note>;

    /// Finds notes by alias using the alias index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<Note>, NoteQueryError> {
        let ids = self
            .db
            .multimap_get("alias_to_id", alias)
            .map_err(NoteQueryError::Storage)?;

        if let Some(id_str) = ids.first() {
            self.db
                .get_owned::<Note>("notes", id_str)
                .map_err(NoteQueryError::Storage)
        } else {
            Ok(None)
        }
    }

    /// Finds notes by file class using the `file_class` index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_file_class(
        &self,
        class: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        let ids = self
            .db
            .multimap_get("file_class_to_id", class)
            .map_err(NoteQueryError::Storage)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self
                .db
                .get_owned::<Note>("notes", &id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Finds notes by folder path using the folder index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_folder(
        &self,
        folder: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        let ids = self
            .db
            .multimap_get("folder_to_id", folder)
            .map_err(NoteQueryError::Storage)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self
                .db
                .get_owned::<Note>("notes", &id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Finds a note by its UUID v7 identifier (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteQueryError> {
        let id_str = id.to_string();
        self.db
            .get_owned::<Note>("notes", &id_str)
            .map_err(NoteQueryError::Storage)
    }

    /// Finds a note by its vault-relative path (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteQueryError> {
        let ids = self
            .db
            .multimap_get("path_to_id", path)
            .map_err(NoteQueryError::Storage)?;

        if let Some(id_str) = ids.first() {
            self.db
                .get_owned::<Note>("notes", id_str)
                .map_err(NoteQueryError::Storage)
        } else {
            Ok(None)
        }
    }

    /// Finds notes by task completed date using the `tasks_by_completed_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified completed date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_completed_date",
            &completed_date.to_string(),
        )
    }

    /// Finds notes by task created date using the `tasks_by_created_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified created date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_created_date",
            &created_date.to_string(),
        )
    }

    /// Finds notes by task due date using the `tasks_by_due_date` index.
    ///
    /// Returns notes containing tasks with the specified due date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_due_date",
            &due_date.to_string(),
        )
    }

    /// Finds notes by task priority using the `tasks_by_priority` index.
    ///
    /// Returns notes containing tasks with the specified priority.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_priority",
            &priority.to_string(),
        )
    }

    /// Finds notes by task project using the `tasks_by_project` index.
    ///
    /// Returns notes containing tasks with the specified project.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index("tasks_by_project", project)
    }

    /// Finds notes by task reminder date using the `tasks_by_reminder_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified reminder date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_reminder_date",
            &reminder_date.to_string(),
        )
    }

    /// Finds notes by task status using the `tasks_by_status` index.
    ///
    /// Returns notes containing tasks with the specified status.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index("tasks_by_status", status)
    }

    /// Lists all notes in the vault (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn list(&self) -> Result<Vec<Note>, NoteQueryError> {
        self.db.list_owned::<Note>("notes").map_err(NoteQueryError::Storage)
    }

    /// Queries notes by frontmatter key-value pair using the generic
    /// frontmatter index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        let combined_key = format!("{key}:{value}");
        let ids = self
            .db
            .multimap_get("frontmatter_kv_to_id", &combined_key)
            .map_err(NoteQueryError::Storage)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self
                .db
                .get_owned::<Note>("notes", &id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Access a note as archived data (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        let id_str = id.to_string();
        self.db
            .get::<Note, _, R>("notes", &id_str, f)
            .map_err(NoteQueryError::Storage)
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
        use crate::note::{
            aggregate::{Note, NoteId},
            frontmatter::Frontmatter,
            value::FieldValue,
        };

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
                "root".into(),
                FieldValue::Object(HashMap::from([(
                    "nested".into(),
                    FieldValue::Array(vec![
                        FieldValue::String("x".into()),
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

            // Create a simple note directly since command is not implemented
            let fm = complex_frontmatter()?;
            let mut note = Note::new(NoteId::new(), "notes/a.md".to_owned())
                .map_err(|e| e.to_string())?;

            // Set frontmatter and content
            note.set_frontmatter(Some(fm.clone()));

            let id = Uuid::from(note.id());

            // Store the note in the database
            db.put("notes", &id.to_string(), &note)
                .map_err(|e| e.to_string())?;

            // Index by path
            db.multimap_insert("path_to_id", "notes/a.md", &id.to_string())
                .map_err(|e| e.to_string())?;

            Ok((dir, db, id, fm))
        }
    }

    use super::*;
    use crate::note::{error::NoteError, ports::Query as _};

    mod query {
        use super::*;
        use crate::note::aggregate::NoteId;

        #[test]
        fn find_by_id_returns_note_with_matching_id()
        -> Result<(), NoteQueryError> {
            let (_dir, db, id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = Query::new(&db);

            let observed = qry
                .find_by_id(id)?
                .expect("Query by id should return Some(note)");
            assert_eq!(
                Uuid::from(observed.id()),
                id,
                "Observed id should match"
            );
            Ok(())
        }

        #[test]
        fn find_by_id_preserves_frontmatter() -> Result<(), NoteQueryError> {
            let (_dir, db, id, fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = Query::new(&db);

            let observed = qry
                .find_by_id(id)?
                .expect("Query by id should return Some(note)");
            assert_eq!(
                observed.frontmatter(),
                Some(&fm),
                "Frontmatter should roundtrip"
            );
            Ok(())
        }

        #[test]
        fn find_by_id_returns_none_for_missing_id() -> Result<(), NoteQueryError>
        {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;

            // Ensure the data table exists by inserting and deleting a note
            let temp_note = Note::new(NoteId::new(), "temp.md".to_owned())
                .map_err(|e| {
                    NoteQueryError::Domain(NoteError::Storage(e.to_string()))
                })?;
            let temp_id = Uuid::from(temp_note.id());
            db.put("notes", &temp_id.to_string(), &temp_note)
                .map_err(NoteQueryError::Storage)?;
            db.delete("notes", &temp_id.to_string())
                .map_err(NoteQueryError::Storage)?;

            let qry = Query::new(&db);

            let miss = qry.find_by_id(fixtures::TEST_MISSING_ID)?;
            assert!(miss.is_none(), "Non-existent ID should return None");
            Ok(())
        }

        #[test]
        fn find_by_path_returns_note_with_matching_path()
        -> Result<(), NoteQueryError> {
            let (_dir, db, _id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = Query::new(&db);

            let observed = qry
                .find_by_path("notes/a.md")?
                .expect("Query by path should return Some(note)");
            assert_eq!(
                observed.path().as_str(),
                "notes/a.md",
                "Observed path should match"
            );
            Ok(())
        }

        #[test]
        fn find_by_path_returns_none_for_missing_path()
        -> Result<(), NoteQueryError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;

            // Ensure the multimap table exists by inserting and deleting a path
            // mapping
            let temp_note = Note::new(NoteId::new(), "temp.md".to_owned())
                .map_err(|e| {
                    NoteQueryError::Domain(NoteError::Storage(e.to_string()))
                })?;
            let temp_id = Uuid::from(temp_note.id());
            db.multimap_insert("path_to_id", "temp.md", &temp_id.to_string())
                .map_err(NoteQueryError::Storage)?;
            db.multimap_remove("path_to_id", "temp.md", &temp_id.to_string())
                .map_err(NoteQueryError::Storage)?;

            let qry = Query::new(&db);

            let miss = qry.find_by_path("nonexistent.md")?;
            assert!(miss.is_none(), "Non-existent path should return None");
            Ok(())
        }

        #[test]
        fn list_returns_all_notes() -> Result<(), NoteQueryError> {
            let (_dir, db, _id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = Query::new(&db);

            let notes = qry.list()?;
            assert_eq!(notes.len(), 1, "Should return exactly one note");
            assert_eq!(
                notes
                    .first()
                    .expect("Should have at least one note")
                    .path()
                    .as_str(),
                "notes/a.md",
                "Returned note should have correct path"
            );
            Ok(())
        }

        #[test]
        fn with_archived_by_id_provides_zero_copy_access()
        -> Result<(), NoteQueryError> {
            let (_dir, db, id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = Query::new(&db);

            let result = qry.with_archived_by_id(id, |archived| {
                // Access the archived path field to verify zero-copy access
                // works The archived ID field conversion might
                // need special handling
                archived.path().as_str().to_owned()
            })?;

            assert_eq!(
                result,
                Some("notes/a.md".to_owned()),
                "Zero-copy access should return correct path"
            );
            Ok(())
        }
    }
}
