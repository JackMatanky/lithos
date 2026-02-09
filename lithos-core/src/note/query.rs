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
}

impl super::ports::Query for Query<'_> {
    type NoteArchived<'archived> = &'archived rkyv::Archived<Note>;

    /// Finds a note by its UUID v7 identifier.
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

    /// Access a note as archived data (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    /// Access a note as archived data by path (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn with_archived_by_path<F, R>(
        &self,
        path: &str,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;

    /// Access a note as archived data by path (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn with_archived_by_path<F, R>(
        &self,
        path: &str,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;

    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn with_archived_by_path<F, R>(
        &self,
        path: &str,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;

    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
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
                .get_owned::<Note>("notes", id_str)
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
                .get_owned::<Note>("notes", id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Queries notes by frontmatter key-value pair using the generic
    /// frontmatter index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        let ids = self
            .db
            .multimap_get("frontmatter_kv_to_id", &(key, value))
            .map_err(NoteQueryError::Storage)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self
                .db
                .get_owned::<Note>("notes", id_str)
                .map_err(NoteQueryError::Storage)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    /// Finds notes by task due date using the tasks_by_due_date index.
    ///
    /// Returns notes containing tasks with the specified due date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_due_date",
            &due_date.to_string(),
        )
    }

    /// Finds notes by task created date using the tasks_by_created_date index.
    ///
    /// Returns notes containing tasks with the specified created date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_created_date",
            &created_date.to_string(),
        )
    }

    /// Finds notes by task reminder date using the tasks_by_reminder_date
    /// index.
    ///
    /// Returns notes containing tasks with the specified reminder date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_reminder_date",
            &reminder_date.to_string(),
        )
    }

    /// Finds notes by task completed date using the tasks_by_completed_date
    /// index.
    ///
    /// Returns notes containing tasks with the specified completed date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_completed_date",
            &completed_date.to_string(),
        )
    }

    /// Finds notes by task priority using the tasks_by_priority index.
    ///
    /// Returns notes containing tasks with the specified priority.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index(
            "tasks_by_priority",
            &priority.to_string(),
        )
    }

    /// Finds notes by task project using the tasks_by_project index.
    ///
    /// Returns notes containing tasks with the specified project.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index("tasks_by_project", project)
    }

    /// Finds notes by task status using the tasks_by_status index.
    ///
    /// Returns notes containing tasks with the specified status.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.find_notes_by_task_index("tasks_by_status", status)
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
        for (note_id_str, task_id_str) in note_refs {
            // Verify the task ID matches to ensure data integrity
            if note_id_str == task_id_str {
                if let Some(note) = self
                    .db
                    .get_owned::<Note>("notes", note_id_str)
                    .map_err(NoteQueryError::Storage)?
                {
                    notes.push(note);
                }
            }
        }
        Ok(notes)
    }

    /// Access a note as archived data by path (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    #[inline]
    fn with_archived_by_path<F, R>(
        &self,
        path: &str,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R
    {
        let ids = self
            .db
            .multimap_get("path_to_id", path)
            .map_err(NoteQueryError::Storage)?;

        if let Some(id_str) = ids.first() {
            self.db
                .get::<Note, _, R>("notes", &id_str, f)
                .map_err(NoteQueryError::Storage)
        } else {
            Ok(None)
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
        error::NoteError,
        ports::{Command as _, Query as _},
    };

    mod query {
        use super::*;

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
            let cmd = command::Command::new(&db);
            let qry = Query::new(&db);

            cmd.create("notes/a.md".to_owned()).map_err(|e| {
                NoteQueryError::Domain(NoteError::Storage(e.to_string()))
            })?;
            let miss = qry.find_by_id(fixtures::TEST_MISSING_ID)?;
            assert!(miss.is_none(), "Non-existent ID should return None");
            Ok(())
        }
    }
}
