//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

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
}

impl super::ports::Command for Command<'_> {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if note creation fails validation or persistence.
    #[inline]
    fn create(&self, path: String) -> Result<Note, NoteError> {
        let note = Note::new(Uuid::now_v7(), path)?;
        let id_str = note.id.to_string();

        self.db.put("notes", &id_str, &note).map_err(
            |e: crate::db::DbError| NoteError::Storage(e.to_string()),
        )?;

        self.db
            .multimap_insert("path_to_id", note.path.as_str(), &id_str)
            .map_err(|e: crate::db::DbError| {
                NoteError::Storage(e.to_string())
            })?;

        Ok(note)
    }

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteError` if note deletion fails.
    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), NoteError> {
        let id_str = id.to_string();

        // 1. Get note first to clean up indexes
        let note = self.db.get_owned::<Note>("notes", &id_str).map_err(
            |e: crate::db::DbError| NoteError::Storage(e.to_string()),
        )?;

        if let Some(n) = note {
            // 2. Remove from path index
            self.db
                .multimap_remove("path_to_id", n.path.as_str(), &id_str)
                .map_err(|e: crate::db::DbError| {
                    NoteError::Storage(e.to_string())
                })?;

            // 3. Remove from tag indexes
            for tag in &n.tags {
                self.db
                    .multimap_remove(
                        "tags_to_notes",
                        tag.full_path.as_str(),
                        &id_str,
                    )
                    .map_err(|e: crate::db::DbError| {
                        NoteError::Storage(e.to_string())
                    })?;
            }

            // 4. Delete note
            self.db.delete("notes", &id_str).map_err(
                |e: crate::db::DbError| NoteError::Storage(e.to_string()),
            )?;
        }

        Ok(())
    }

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `NoteError` if note update fails validation or persistence.
    #[inline]
    fn update(&self, note: Note) -> Result<Note, NoteError> {
        let id_str = note.id.to_string();

        // 1. Get old note to find what changed
        let old_note = self.db.get_owned::<Note>("notes", &id_str).map_err(
            |e: crate::db::DbError| NoteError::Storage(e.to_string()),
        )?;

        if let Some(old) = old_note {
            // 2. Update path index if changed
            if old.path != note.path {
                self.db
                    .multimap_remove("path_to_id", old.path.as_str(), &id_str)
                    .map_err(|e: crate::db::DbError| {
                        NoteError::Storage(e.to_string())
                    })?;
                self.db
                    .multimap_insert("path_to_id", note.path.as_str(), &id_str)
                    .map_err(|e: crate::db::DbError| {
                        NoteError::Storage(e.to_string())
                    })?;
            }

            // 3. Update tag index
            // Remove old tags
            for tag in &old.tags {
                self.db
                    .multimap_remove(
                        "tags_to_notes",
                        tag.full_path.as_str(),
                        &id_str,
                    )
                    .map_err(|e: crate::db::DbError| {
                        NoteError::Storage(e.to_string())
                    })?;
            }
        } else {
            // New note (even though it's update call), add path index
            self.db
                .multimap_insert("path_to_id", note.path.as_str(), &id_str)
                .map_err(|e: crate::db::DbError| {
                    NoteError::Storage(e.to_string())
                })?;
        }

        // Add new tags
        for tag in &note.tags {
            self.db
                .multimap_insert(
                    "tags_to_notes",
                    tag.full_path.as_str(),
                    &id_str,
                )
                .map_err(|e: crate::db::DbError| {
                    NoteError::Storage(e.to_string())
                })?;
        }

        // 4. Save new note
        self.db.put("notes", &id_str, &note).map_err(
            |e: crate::db::DbError| NoteError::Storage(e.to_string()),
        )?;

        Ok(note)
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
        use tempfile::{TempDir, tempdir};
        use uuid::Uuid;

        use super::*;

        pub const TEST_MISSING_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("notes.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn create_note(cmd: &Command, path: &str) -> Result<Note, String> {
            cmd.create(path.to_owned()).map_err(|e| e.to_string())
        }

        pub fn update_note(cmd: &Command, note: Note) -> Result<Note, String> {
            cmd.update(note).map_err(|e| e.to_string())
        }

        pub fn delete_note(cmd: &Command, id: Uuid) -> Result<(), String> {
            cmd.delete(id).map_err(|e| e.to_string())
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::new(path.to_owned()).map_err(|e| e.to_string())
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
    use crate::note::{
        aggregate::{Note, NotePath},
        ports::Command as _,
        tag::Tag,
    };

    mod persistence {
        use super::*;

        #[test]
        fn create_persists_note_path() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();

            let stored_note = fixtures::stored_note(&db, &id_str)
                .expect("Read-back should succeed")
                .expect("Stored note should exist");
            assert_eq!(
                stored_note.path.as_str(),
                "notes/a.md",
                "Stored note path should match"
            );
        }

        #[test]
        fn create_persists_path_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();

            let ids = fixtures::path_index_ids(&db, note.path.as_str())
                .expect("Path index should be readable");
            assert!(
                ids.contains(&id_str),
                "Path index should contain created note id"
            );
        }

        #[test]
        fn update_removes_old_path_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let old_path = note.path.as_str().to_owned();
            let path = fixtures::parse_path("notes/b.md")
                .expect("NotePath should be valid");
            note.path = path;

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let old_ids = fixtures::path_index_ids(&db, old_path.as_str())
                .expect("Old path index should be readable");
            assert!(
                !old_ids.contains(&id_str),
                "Old path index should not contain updated note id"
            );
        }

        #[test]
        fn update_adds_new_path_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let path = fixtures::parse_path("notes/b.md")
                .expect("NotePath should be valid");
            note.path = path;

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let new_ids = fixtures::path_index_ids(&db, "notes/b.md")
                .expect("New path index should be readable");
            assert!(
                new_ids.contains(&id_str),
                "New path index should contain updated note id"
            );
        }

        #[test]
        fn update_adds_tag_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let tag =
                fixtures::parse_tag("#project").expect("Tag should parse");
            let tag_key = tag.full_path.as_str().to_owned();
            note.tags = vec![tag];

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let tag_ids = fixtures::tag_index_ids(&db, tag_key.as_str())
                .expect("Tag index should be readable");
            assert!(
                tag_ids.contains(&id_str),
                "Tag index should contain updated note id"
            );
        }

        #[test]
        fn update_persists_new_path() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let path = fixtures::parse_path("notes/b.md")
                .expect("NotePath should be valid");
            note.path = path;

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .expect("Read-back should succeed")
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.path.as_str(),
                "notes/b.md",
                "Stored note path should be updated"
            );
        }

        #[test]
        fn update_persists_new_tags() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let tag =
                fixtures::parse_tag("#project").expect("Tag should parse");
            note.tags = vec![tag];

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .expect("Read-back should succeed")
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.tags.len(),
                1,
                "Stored note should have updated tags"
            );
        }

        #[test]
        fn delete_removes_note() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id = note.id;
            let id_str = id.to_string();

            let result = fixtures::delete_note(&cmd, id);
            assert!(result.is_ok(), "Delete should succeed: {result:?}");

            let stored = fixtures::stored_note(&db, &id_str)
                .expect("Read-back should succeed");
            assert!(stored.is_none(), "Deleted note should not exist");
        }

        #[test]
        fn delete_removes_path_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id = note.id;
            let id_str = id.to_string();
            let tag =
                fixtures::parse_tag("#project").expect("Tag should parse");
            note.tags = vec![tag];

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

            let path_ids = fixtures::path_index_ids(&db, note.path.as_str())
                .expect("Path index should be readable");
            assert!(
                !path_ids.contains(&id_str),
                "Path index should not contain deleted note id"
            );
        }

        #[test]
        fn delete_removes_tag_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")
                .expect("Create should succeed");
            let id = note.id;
            let id_str = id.to_string();
            let tag =
                fixtures::parse_tag("#project").expect("Tag should parse");
            let tag_key = tag.full_path.as_str().to_owned();
            note.tags = vec![tag];

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

            let tag_ids = fixtures::tag_index_ids(&db, tag_key.as_str())
                .expect("Tag index should be readable");
            assert!(
                !tag_ids.contains(&id_str),
                "Tag index should not contain deleted note id"
            );
        }

        #[test]
        fn delete_missing_note_is_noop_for_existing_note() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/existing.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();

            let delete_result =
                fixtures::delete_note(&cmd, fixtures::TEST_MISSING_ID);
            assert!(
                delete_result.is_ok(),
                "Deleting missing note should be a no-op: {delete_result:?}"
            );

            let stored = fixtures::stored_note(&db, &id_str)
                .expect("Read-back should succeed");
            assert!(stored.is_some(), "Existing note should remain");
        }

        #[test]
        fn update_removes_old_tag_index_when_tags_change() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let old_tag =
                fixtures::parse_tag("#old-tag").expect("Tag should parse");
            let old_key = old_tag.full_path.as_str().to_owned();
            note.tags = vec![old_tag];

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag =
                fixtures::parse_tag("#new-tag").expect("Tag should parse");
            note.tags = vec![new_tag];
            let update_result_after = fixtures::update_note(&cmd, note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let old_tag_ids = fixtures::tag_index_ids(&db, old_key.as_str())
                .expect("Tag index should be readable");
            assert!(
                !old_tag_ids.contains(&id_str),
                "Old tag index should not contain note after update"
            );
        }

        #[test]
        fn update_adds_new_tag_index_when_tags_change() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test DB");
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")
                .expect("Create should succeed");
            let id_str = note.id.to_string();
            let old_tag =
                fixtures::parse_tag("#old-tag").expect("Tag should parse");
            note.tags = vec![old_tag];

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag =
                fixtures::parse_tag("#new-tag").expect("Tag should parse");
            let new_key = new_tag.full_path.as_str().to_owned();
            note.tags = vec![new_tag];
            let update_result_after = fixtures::update_note(&cmd, note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let new_tag_ids = fixtures::tag_index_ids(&db, new_key.as_str())
                .expect("Tag index should be readable");
            assert!(
                new_tag_ids.contains(&id_str),
                "New tag index should contain note after update"
            );
        }
    }
}
