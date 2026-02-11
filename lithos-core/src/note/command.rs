//! Concrete implementation of the [`ports::Command`] trait.
//!
//! Provides write operations for the Note aggregate, including creation,
//! updates, and deletion, ensuring atomic persistence and index consistency.

//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

use uuid::Uuid;

use super::{
    aggregate::{Note, NoteId},
    error::NoteCommandError,
};
use crate::db::Database;

/// Index data extracted from a note for cleanup operations.
/// Contains (path, tags) tuple needed to remove old index entries.
type IndexData = (String, Vec<String>);

/// Command implementation for Note write operations.
///
/// Implements the [`crate::note::ports::Command`] trait using the [`Database`]
/// layer. This struct handles the atomic creation, update, and deletion of
/// notes, ensuring that all secondary indexes remain consistent with the
/// primary note data.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::command::Command;
/// # use lithos_core::db::Database;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let path = std::env::temp_dir().join("test_command.db");
/// # let db = Database::open(&path)?;
/// let command = Command::new(&db);
/// // Use command to create a note...
/// # Ok(())
/// # }
/// ```
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

    /// Helper: Extract path and tags from archived note for index cleanup.
    ///
    /// This is used by both `update()` and `delete()` to get the old index data
    /// before modifying the note. The allocations here are necessary because
    /// the data must outlive the read transaction.
    ///
    /// # Allocation Constraint
    ///
    /// This method necessarily allocates owned strings for path and tags
    /// because:
    /// 1. Read transaction creates archived data with closure-scoped lifetime
    /// 2. Index updates require a separate write transaction
    /// 3. Cannot borrow archived data across transaction boundary
    /// 4. Must extract owned data before read transaction ends
    ///
    /// This is a **fundamental constraint** of the redb transaction model.
    /// Attempted alternatives (reading within write transaction) violate Rust
    /// borrowing rules: cannot call `batch.multimap_remove()` while inside
    /// `batch.get()` closure.
    ///
    /// **Cost**: ~250-450 bytes per note mutation (1 path + N tags × ~20
    /// bytes). **Frequency**: Write operations only (cold path relative to
    /// reads). **Decision**: Accept as necessary architectural cost.
    ///
    /// See `TODO_ALLOCATIONS.md` Issue #6 for detailed analysis.
    ///
    /// # Errors
    /// Returns `NoteCommandError::Storage` if the database read fails.
    fn get_note_index_data(
        &self,
        id_str: &str,
    ) -> Result<Option<IndexData>, NoteCommandError> {
        self.db
            .get::<Note, _, (String, Vec<String>)>(
                "notes",
                id_str,
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

impl super::ports::Command for Command<'_> {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note creation fails.
    #[inline]
    fn create(&self, path: &str) -> Result<Note, NoteCommandError> {
        let note = Note::new(NoteId::new(), path)?;
        let id_str = Uuid::from(note.id()).to_string();

        self.db.batch_write(|batch| {
            batch.put("notes", &id_str, &note)?;
            batch.multimap_insert(
                "path_to_id",
                note.path().as_str(),
                &id_str,
            )?;
            Ok(())
        })?;

        Ok(note)
    }

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note deletion fails.
    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), NoteCommandError> {
        let id_str = id.to_string();

        // 1. Get old data for index cleanup using zero-copy read
        let old_data = self.get_note_index_data(&id_str)?;

        if let Some((path, tags)) = old_data {
            self.db.batch_write(|batch| {
                // 2. Remove from path index
                batch.multimap_remove("path_to_id", &path, &id_str)?;

                // 3. Remove from tag indexes
                for tag in tags {
                    batch.multimap_remove("tags_to_notes", &tag, &id_str)?;
                }

                // 4. Delete note
                batch.delete("notes", &id_str)?;

                Ok(())
            })?;
        }

        Ok(())
    }

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note update fails.
    #[inline]
    fn update(&self, note: Note) -> Result<Note, NoteCommandError> {
        let id_str = Uuid::from(note.id()).to_string();

        // 1. Get old data for index cleanup using zero-copy read
        let old_data = self.get_note_index_data(&id_str)?;

        self.db.batch_write(|batch| {
            if let Some((old_path, old_tags)) = old_data {
                // 2. Update path index if changed
                if old_path != note.path().as_str() {
                    batch.multimap_remove("path_to_id", &old_path, &id_str)?;
                    batch.multimap_insert(
                        "path_to_id",
                        note.path().as_str(),
                        &id_str,
                    )?;
                }

                // 3. Update tag index
                // Remove old tags
                for tag in old_tags {
                    batch.multimap_remove("tags_to_notes", &tag, &id_str)?;
                }
            } else {
                // New note (even though it's update call), add path index
                batch.multimap_insert(
                    "path_to_id",
                    note.path().as_str(),
                    &id_str,
                )?;
            }

            // Add new tags
            for tag in note.tags() {
                batch.multimap_insert(
                    "tags_to_notes",
                    tag.full_path(),
                    &id_str,
                )?;
            }

            // 4. Save new note
            batch.put("notes", &id_str, &note)?;

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
    mod fixtures {
        use tempfile::{TempDir, tempdir};
        use uuid::Uuid;

        use super::*;
        use crate::note::{aggregate::NotePath, tag::Tag};

        pub const TEST_MISSING_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("notes.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn create_note(
            cmd: &Command,
            path: &str,
        ) -> Result<Note, NoteCommandError> {
            crate::note::ports::Command::create(cmd, path)
        }

        pub fn update_note(
            cmd: &Command,
            note: Note,
        ) -> Result<Note, NoteCommandError> {
            crate::note::ports::Command::update(cmd, note)
        }

        pub fn delete_note(
            cmd: &Command,
            id: Uuid,
        ) -> Result<(), NoteCommandError> {
            crate::note::ports::Command::delete(cmd, id)
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::new(path).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::new(tag).map_err(|e| e.to_string())
        }

        pub fn stored_note(
            db: &Database,
            id: &str,
        ) -> Result<Option<Note>, String> {
            db.get_owned::<Note>("notes", id).map_err(|e| e.to_string())
        }

        pub fn path_index_ids(
            db: &Database,
            path: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get("path_to_id", path).map_err(|e| e.to_string())
        }

        pub fn tag_index_ids(
            db: &Database,
            tag: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get("tags_to_notes", tag).map_err(|e| e.to_string())
        }
    }

    use super::*;
    use crate::note::error::NoteError;

    mod persistence {
        use super::*;

        #[test]
        fn create_persists_note_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let stored_note = fixtures::stored_note(&db, &id_str)
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
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                ids.contains(&id_str),
                "Path index should contain created note id"
            );
            Ok(())
        }

        #[test]
        fn update_removes_old_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_path = note.path().as_str().to_owned();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let old_ids = fixtures::path_index_ids(&db, &old_path)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !old_ids.contains(&id_str),
                "Old path index should not contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let new_ids = fixtures::path_index_ids(&db, "notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                new_ids.contains(&id_str),
                "New path index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let tag_ids = fixtures::tag_index_ids(&db, &tag_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                tag_ids.contains(&id_str),
                "Tag index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_persists_new_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.path().as_str(),
                "notes/b.md",
                "Stored note path should be updated"
            );
            Ok(())
        }

        #[test]
        fn update_persists_new_tags() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.tags().count(),
                1,
                "Stored note should have updated tags"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_note() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();

            let result = fixtures::delete_note(&cmd, id);
            assert!(result.is_ok(), "Delete should succeed: {result:?}");

            let stored = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(stored.is_none(), "Deleted note should not exist");
            Ok(())
        }

        #[test]
        fn delete_removes_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );
            let delete_result = fixtures::delete_note(&cmd, id);
            assert!(
                delete_result.is_ok(),
                "Delete should succeed: {delete_result:?}"
            );

            let path_ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !path_ids.contains(&id_str),
                "Path index should not contain deleted note id"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let update_result_after = fixtures::update_note(&cmd, note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );
            let delete_result = fixtures::delete_note(&cmd, id);
            assert!(
                delete_result.is_ok(),
                "Delete should succeed: {delete_result:?}"
            );

            let tag_ids = fixtures::tag_index_ids(&db, &tag_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !tag_ids.contains(&id_str),
                "Tag index should not contain deleted note id"
            );
            Ok(())
        }

        #[test]
        fn delete_missing_note_is_noop_for_existing_note()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/existing.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let delete_result =
                fixtures::delete_note(&cmd, fixtures::TEST_MISSING_ID);
            assert!(
                delete_result.is_ok(),
                "Deleting missing note should be a no-op: {delete_result:?}"
            );

            let stored = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(stored.is_some(), "Existing note should remain");
            Ok(())
        }

        #[test]
        fn update_removes_old_tag_index_when_tags_change()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_tag = fixtures::parse_tag("#old-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let old_key = old_tag.full_path().to_owned();
            note.add_tag(old_tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag = fixtures::parse_tag("#new-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;

            let mut updated_note = Note::new(note.id(), note.path().as_str())?;
            updated_note.add_tag(new_tag);

            let update_result_after = fixtures::update_note(&cmd, updated_note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let old_tag_ids = fixtures::tag_index_ids(&db, &old_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !old_tag_ids.contains(&id_str),
                "Old tag index should not contain note after update"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_tag_index_when_tags_change()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_tag = fixtures::parse_tag("#old-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(old_tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag = fixtures::parse_tag("#new-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let new_key = new_tag.full_path().to_owned();

            let mut updated_note = Note::new(note.id(), note.path().as_str())?;
            updated_note.add_tag(new_tag);

            let update_result_after = fixtures::update_note(&cmd, updated_note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let new_tag_ids = fixtures::tag_index_ids(&db, &new_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                new_tag_ids.contains(&id_str),
                "New tag index should contain note after update"
            );
            Ok(())
        }
    }
}
