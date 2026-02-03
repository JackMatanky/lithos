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
    clippy::disallowed_methods,
    reason = "Test code uses unwrap/expect for clarity"
)]
mod tests {
    use tempfile::{TempDir, tempdir};
    use uuid::Uuid;

    use super::*;
    use crate::note::{
        aggregate::{Note, NotePath},
        ports::Command as _,
        tag::Tag,
    };

    const TEST_MISSING_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

    fn test_db() -> (TempDir, Database) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("notes.redb");
        let db = Database::open(&path).unwrap();
        (dir, db)
    }

    #[test]
    fn create_persists_note_and_path_index() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let note = cmd.create("notes/a.md".to_owned()).unwrap();
        let id_str = note.id.to_string();

        let stored = db.get_owned::<Note>("notes", &id_str).unwrap();
        let stored_note = stored.expect("Stored note should exist");
        assert_eq!(
            stored_note.path.as_str(),
            "notes/a.md",
            "Stored note path should match"
        );

        let ids = db.multimap_get("path_to_id", note.path.as_str()).unwrap();
        assert!(
            ids.contains(&id_str),
            "Path index should contain created note id"
        );
    }

    #[test]
    fn update_updates_path_and_tags_indexes() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let mut note = cmd.create("notes/a.md".to_owned()).unwrap();
        let id_str = note.id.to_string();
        let old_path = note.path.as_str().to_owned();

        note.path = NotePath::new("notes/b.md".to_owned()).unwrap();
        let tag = Tag::new("#project").unwrap();
        note.tags = vec![tag];

        cmd.update(note.clone()).unwrap();

        let old_ids = db.multimap_get("path_to_id", old_path.as_str()).unwrap();
        assert!(
            !old_ids.contains(&id_str),
            "Old path index should not contain updated note id"
        );

        let new_ids =
            db.multimap_get("path_to_id", note.path.as_str()).unwrap();
        assert!(
            new_ids.contains(&id_str),
            "New path index should contain updated note id"
        );

        let tag_key = note
            .tags
            .first()
            .expect("Note should have one tag")
            .full_path
            .as_str();
        let tag_ids = db.multimap_get("tags_to_notes", tag_key).unwrap();
        assert!(
            tag_ids.contains(&id_str),
            "Tag index should contain updated note id"
        );

        let stored = db.get_owned::<Note>("notes", &id_str).unwrap();
        let stored_note = stored.expect("Updated note should exist");
        assert_eq!(
            stored_note.path.as_str(),
            "notes/b.md",
            "Stored note path should be updated"
        );
        assert_eq!(
            stored_note.tags.len(),
            1,
            "Stored note should have updated tags"
        );
    }

    #[test]
    fn delete_removes_note_and_indexes() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let mut note = cmd.create("notes/a.md".to_owned()).unwrap();
        let id = note.id;
        let id_str = id.to_string();

        let tag = Tag::new("#project").unwrap();
        note.tags = vec![tag];
        cmd.update(note.clone()).unwrap();

        cmd.delete(id).unwrap();

        let stored = db.get_owned::<Note>("notes", &id_str).unwrap();
        assert!(stored.is_none(), "Deleted note should not exist");

        let path_ids =
            db.multimap_get("path_to_id", note.path.as_str()).unwrap();
        assert!(
            !path_ids.contains(&id_str),
            "Path index should not contain deleted note id"
        );

        let tag_key = note
            .tags
            .first()
            .expect("Note should have one tag")
            .full_path
            .as_str();
        let tag_ids = db.multimap_get("tags_to_notes", tag_key).unwrap();
        assert!(
            !tag_ids.contains(&id_str),
            "Tag index should not contain deleted note id"
        );
    }

    #[test]
    fn delete_missing_note_is_noop() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let _existing = cmd.create("notes/existing.md".to_owned()).unwrap();
        let result = cmd.delete(TEST_MISSING_ID);

        assert!(
            result.is_ok(),
            "Deleting missing note should be a no-op, got: {result:?}"
        );
    }

    #[test]
    fn update_removes_old_tag_indexes_when_tags_change() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        // GIVEN: a note with an initial tag
        let mut note = cmd.create("notes/test.md".to_owned()).unwrap();
        let id_str = note.id.to_string();
        let old_tag = Tag::new("#old-tag").unwrap();
        note.tags = vec![old_tag.clone()];
        cmd.update(note.clone()).unwrap();

        // Verify old tag is indexed
        let old_tag_ids = db
            .multimap_get("tags_to_notes", old_tag.full_path.as_str())
            .unwrap();
        assert!(
            old_tag_ids.contains(&id_str),
            "Old tag index should contain note before update"
        );

        // WHEN: updating the note with a different tag
        let new_tag = Tag::new("#new-tag").unwrap();
        note.tags = vec![new_tag.clone()];
        cmd.update(note.clone()).unwrap();

        // THEN: old tag index should not contain the note
        let old_tag_ids_after = db
            .multimap_get("tags_to_notes", old_tag.full_path.as_str())
            .unwrap();
        assert!(
            !old_tag_ids_after.contains(&id_str),
            "Old tag index should not contain note after update with \
             different tag"
        );

        // AND: new tag index should contain the note
        let new_tag_ids = db
            .multimap_get("tags_to_notes", new_tag.full_path.as_str())
            .unwrap();
        assert!(
            new_tag_ids.contains(&id_str),
            "New tag index should contain note after update"
        );
    }
}
