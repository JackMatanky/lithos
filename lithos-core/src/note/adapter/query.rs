//! Concrete implementation of the [`crate::note::ports::Query`] trait.
//!
//! Provides high-performance, zero-copy read operations for notes and tasks,
//! utilizing specialized database indexes.

use uuid::Uuid;

use crate::{
    db::Database,
    note::{
        aggregate::Note,
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV, NOTES,
            PATH_TO_ID, TASKS_BY_COMPLETED_DATE, TASKS_BY_CREATED_DATE,
            TASKS_BY_DUE_DATE, TASKS_BY_PRIORITY, TASKS_BY_PROJECT,
            TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        ports::Query,
    },
};

/// Query implementation for Note read operations.
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    /// Create a new `QueryAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Helper method to find notes by any task index.
    fn find_notes_by_task_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<Note>, crate::db::DbError> {
        let note_refs = self.db.multimap_get(index_table, index_key)?;

        let mut notes = Vec::with_capacity(note_refs.len());
        for note_id_str in note_refs {
            if let Some(note) =
                self.db.get_owned::<Note>(NOTES, &note_id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }
}

impl Query for QueryAdapter<'_> {
    type Error = crate::db::DbError;
    type NoteArchived<'archived> = &'archived rkyv::Archived<Note>;

    #[inline]
    fn find_by_alias(&self, alias: &str) -> Result<Option<Note>, Self::Error> {
        let ids = self.db.multimap_get(ALIAS_TO_ID, alias)?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Note>(NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn find_by_file_class(
        &self,
        class: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        let ids = self.db.multimap_get(FILE_CLASS_TO_ID, class)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self.db.get_owned::<Note>(NOTES, &id_str)? {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn find_by_folder(&self, folder: &str) -> Result<Vec<Note>, Self::Error> {
        let ids = self.db.multimap_get(FOLDER_TO_ID, folder)?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self.db.get_owned::<Note>(NOTES, &id_str)? {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, Self::Error> {
        self.db.get_owned::<Note>(NOTES, &id.to_string())
    }

    #[inline]
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, Self::Error> {
        let ids = self.db.multimap_get(PATH_TO_ID, path)?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Note>(NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(completed_date);
        self.find_notes_by_task_index(TASKS_BY_COMPLETED_DATE, date_str)
    }

    #[inline]
    fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(created_date);
        self.find_notes_by_task_index(TASKS_BY_CREATED_DATE, date_str)
    }

    #[inline]
    fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(due_date);
        self.find_notes_by_task_index(TASKS_BY_DUE_DATE, date_str)
    }

    #[inline]
    fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        let priority_str = buffer.format(priority);
        self.find_notes_by_task_index(TASKS_BY_PRIORITY, priority_str)
    }

    #[inline]
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        self.find_notes_by_task_index(TASKS_BY_PROJECT, project)
    }

    #[inline]
    fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(reminder_date);
        self.find_notes_by_task_index(TASKS_BY_REMINDER_DATE, date_str)
    }

    #[inline]
    fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        self.find_notes_by_task_index(TASKS_BY_STATUS, status)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Note>, Self::Error> {
        self.db.list_owned::<Note>(NOTES)
    }

    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "String length arithmetic is safe and will not overflow"
    )]
    fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        use std::fmt::Write as _;
        let mut combined_key =
            String::with_capacity(key.len() + value.len() + 1);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut combined_key, "{key}:{value}");
        self.find_notes_by_task_index(FRONTMATTER_KV, &combined_key)
    }

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        self.db.get::<Note, _, R>(NOTES, &id.to_string(), f)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::panic_in_result_fn,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::note::error::{NoteError, NoteQueryError};

    mod fixtures {
        use std::collections::HashMap;

        use super::*;
        use crate::note::{
            aggregate::{Note, NoteId},
            frontmatter::Frontmatter,
            value::FieldValue,
        };
        type QuerySetupResult =
            Result<(TempDir, Database, Uuid, Frontmatter), String>;

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

        pub fn note_with_frontmatter() -> QuerySetupResult {
            let (dir, db) = test_db()?;
            let fm = complex_frontmatter()?;
            let mut note = Note::new(NoteId::new(), "notes/a.md")
                .map_err(|e| e.to_string())?;
            note.set_frontmatter(Some(fm.clone()));
            let id = Uuid::from(note.id());
            db.put_by_uuid(NOTES, id, &note).map_err(|e| e.to_string())?;
            db.multimap_insert(PATH_TO_ID, "notes/a.md", &id.to_string())
                .map_err(|e| e.to_string())?;
            Ok((dir, db, id, fm))
        }
    }

    mod query {
        use super::*;

        #[test]
        fn find_by_id_returns_note_with_matching_id()
        -> Result<(), NoteQueryError> {
            let (_dir, db, id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = QueryAdapter::new(&db);

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
        fn find_by_id_returns_none_for_missing_id() -> Result<(), NoteQueryError>
        {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = QueryAdapter::new(&db);
            let miss = qry.find_by_id(fixtures::TEST_MISSING_ID)?;
            assert!(miss.is_none(), "Non-existent ID should return None");
            Ok(())
        }

        #[test]
        fn list_returns_all_notes() -> Result<(), NoteQueryError> {
            let (_dir, db, _id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| NoteQueryError::Domain(NoteError::Storage(e)))?;
            let qry = QueryAdapter::new(&db);
            let notes = qry.list()?;
            assert_eq!(notes.len(), 1);
            Ok(())
        }
    }
}
