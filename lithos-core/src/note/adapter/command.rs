//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

use uuid::Uuid;

use crate::{
    db::Database,
    note::{
        aggregate::{Note, NoteId},
        db_table::{NOTES, PATH_TO_ID, TAGS_TO_NOTES},
        error::NoteCommandError,
        ports::Command,
    },
};

/// Index data extracted from a note for cleanup operations.
/// Contains (path, tags) tuple needed to remove old index entries.
type IndexData = (String, Vec<String>);

/// Command implementation for Note write operations.
pub struct CommandAdapter<'db> {
    db: &'db Database,
}

impl<'db> CommandAdapter<'db> {
    /// Create a new `CommandAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Helper: Extract path and tags from archived note for index cleanup.
    fn get_note_index_data(
        &self,
        id: Uuid,
    ) -> Result<Option<IndexData>, NoteCommandError> {
        self.db
            .get::<Note, _, (String, Vec<String>)>(
                NOTES,
                &id.to_string(),
                |archived| {
                    let path = archived.path().as_str().to_owned();
                    let tags: Vec<String> = archived
                        .tags()
                        .iter()
                        .map(|t| t.full_path().as_str().to_owned())
                        .collect();
                    (path, tags)
                },
            )
            .map_err(NoteCommandError::Storage)
    }
}

impl Command for CommandAdapter<'_> {
    /// Creates a new note with the given vault-relative path.
    #[inline]
    fn create(&self, path: &str) -> Result<Note, NoteCommandError> {
        let note = Note::new(NoteId::new(), path)?;
        let id = Uuid::from(note.id());

        self.db.batch_write(|batch| {
            batch.put(NOTES, &id.to_string(), &note)?;
            batch.multimap_insert(
                PATH_TO_ID,
                note.path().as_str(),
                &id.to_string(),
            )?;
            Ok(())
        })?;

        Ok(note)
    }

    /// Deletes a note by ID.
    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), NoteCommandError> {
        let old_data = self.get_note_index_data(id)?;

        if let Some((path, tags)) = old_data {
            self.db.batch_write(|batch| {
                batch.multimap_remove(PATH_TO_ID, &path, &id.to_string())?;
                for tag in tags {
                    batch.multimap_remove(
                        TAGS_TO_NOTES,
                        &tag,
                        &id.to_string(),
                    )?;
                }
                batch.delete(NOTES, &id.to_string())?;
                Ok(())
            })?;
        }

        Ok(())
    }

    /// Updates an existing note.
    #[inline]
    fn update(&self, note: Note) -> Result<Note, NoteCommandError> {
        let id = Uuid::from(note.id());
        let old_data = self.get_note_index_data(id)?;

        self.db.batch_write(|batch| {
            if let Some((old_path, old_tags)) = old_data {
                if old_path != note.path().as_str() {
                    batch.multimap_remove(
                        PATH_TO_ID,
                        &old_path,
                        &id.to_string(),
                    )?;
                    batch.multimap_insert(
                        PATH_TO_ID,
                        note.path().as_str(),
                        &id.to_string(),
                    )?;
                }
                for tag in old_tags {
                    batch.multimap_remove(
                        TAGS_TO_NOTES,
                        &tag,
                        &id.to_string(),
                    )?;
                }
            } else {
                batch.multimap_insert(
                    PATH_TO_ID,
                    note.path().as_str(),
                    &id.to_string(),
                )?;
            }

            for tag in note.tags() {
                batch.multimap_insert(
                    TAGS_TO_NOTES,
                    tag.full_path(),
                    &id.to_string(),
                )?;
            }

            batch.put(NOTES, &id.to_string(), &note)?;
            Ok(())
        })?;

        Ok(note)
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
    use crate::note::error::NoteError;

    mod fixtures {
        use super::*;
        use crate::note::{aggregate::NotePath, tag::Tag};

        // pub const TEST_MISSING_ID: Uuid =
        //     Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("notes.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn create_note(
            cmd: &CommandAdapter,
            path: &str,
        ) -> Result<Note, NoteCommandError> {
            Command::create(cmd, path)
        }

        pub fn update_note(
            cmd: &CommandAdapter,
            note: Note,
        ) -> Result<Note, NoteCommandError> {
            Command::update(cmd, note)
        }

        pub fn delete_note(
            cmd: &CommandAdapter,
            id: Uuid,
        ) -> Result<(), NoteCommandError> {
            Command::delete(cmd, id)
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::new(path).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::new(tag).map_err(|e| e.to_string())
        }

        pub fn stored_note(
            db: &Database,
            id: Uuid,
        ) -> Result<Option<Note>, String> {
            db.get_owned::<Note>(NOTES, &id.to_string())
                .map_err(|e| e.to_string())
        }

        pub fn path_index_ids(
            db: &Database,
            path: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get(PATH_TO_ID, path).map_err(|e| e.to_string())
        }

        pub fn tag_index_ids(
            db: &Database,
            tag: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get(TAGS_TO_NOTES, tag).map_err(|e| e.to_string())
        }
    }

    mod persistence {
        use super::*;

        #[test]
        fn create_persists_note_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());

            let stored_note = fixtures::stored_note(&db, id)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
                .expect("Stored note should exist");
            assert_eq!(
                stored_note.path().as_str(),
                "notes/a.md",
                "Stored note path should match"
            );
            Ok(())
        }

        #[test]
        fn create_persists_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());

            let ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                ids.contains(&id.to_string()),
                "Path index should contain created note id"
            );
            Ok(())
        }

        #[test]
        fn update_removes_old_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let old_path = note.path().as_str().to_owned();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let old_ids = fixtures::path_index_ids(&db, &old_path)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !old_ids.contains(&id.to_string()),
                "Old path index should not contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let new_ids = fixtures::path_index_ids(&db, "notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                new_ids.contains(&id.to_string()),
                "New path index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let tag_ids = fixtures::tag_index_ids(&db, &tag_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                tag_ids.contains(&id.to_string()),
                "Tag index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_note() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = CommandAdapter::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());

            let result = fixtures::delete_note(&cmd, id);
            assert!(result.is_ok(), "Delete should succeed: {result:?}");

            let stored = fixtures::stored_note(&db, id)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(stored.is_none(), "Deleted note should not exist");
            Ok(())
        }
    }
}
