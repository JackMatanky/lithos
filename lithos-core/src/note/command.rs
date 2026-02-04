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

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("notes.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    #[test]
    fn create_persists_note_and_path_index() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test DB: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let note_result = cmd.create("notes/a.md".to_owned());
        assert!(note_result.is_ok(), "Create should succeed: {note_result:?}");
        let Ok(note) = note_result else {
            return;
        };
        let id_str = note.id.to_string();

        let stored_result = db.get_owned::<Note>("notes", &id_str);
        assert!(
            stored_result.is_ok(),
            "Read-back should succeed: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_some(), "Stored note should exist");
        let Some(stored_note) = stored else {
            return;
        };
        assert_eq!(
            stored_note.path.as_str(),
            "notes/a.md",
            "Stored note path should match"
        );

        let ids_result = db.multimap_get("path_to_id", note.path.as_str());
        assert!(
            ids_result.is_ok(),
            "Path index should be readable: {ids_result:?}"
        );
        let Ok(ids) = ids_result else {
            return;
        };
        assert!(
            ids.contains(&id_str),
            "Path index should contain created note id"
        );
    }

    #[test]
    fn update_updates_path_and_tags_indexes() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test DB: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let note_result = cmd.create("notes/a.md".to_owned());
        assert!(note_result.is_ok(), "Create should succeed: {note_result:?}");
        let Ok(mut note) = note_result else {
            return;
        };
        let id_str = note.id.to_string();
        let old_path = note.path.as_str().to_owned();

        let path_result = NotePath::new("notes/b.md".to_owned());
        assert!(
            path_result.is_ok(),
            "NotePath should be valid: {path_result:?}"
        );
        let Ok(path) = path_result else {
            return;
        };
        note.path = path;

        let tag_result = Tag::new("#project");
        assert!(tag_result.is_ok(), "Tag should parse: {tag_result:?}");
        let Ok(tag) = tag_result else {
            return;
        };
        note.tags = vec![tag];

        let update_result = cmd.update(note.clone());
        assert!(
            update_result.is_ok(),
            "Update should succeed: {update_result:?}"
        );

        let old_ids_result = db.multimap_get("path_to_id", old_path.as_str());
        assert!(
            old_ids_result.is_ok(),
            "Old path index should be readable: {old_ids_result:?}"
        );
        let Ok(old_ids) = old_ids_result else {
            return;
        };
        assert!(
            !old_ids.contains(&id_str),
            "Old path index should not contain updated note id"
        );

        let new_ids_result = db.multimap_get("path_to_id", note.path.as_str());
        assert!(
            new_ids_result.is_ok(),
            "New path index should be readable: {new_ids_result:?}"
        );
        let Ok(new_ids) = new_ids_result else {
            return;
        };
        assert!(
            new_ids.contains(&id_str),
            "New path index should contain updated note id"
        );

        let tag_key = note.tags.first().map(|t| t.full_path.as_str());
        assert!(tag_key.is_some(), "Note should have one tag");
        let Some(tag_key) = tag_key else {
            return;
        };

        let tag_ids_result = db.multimap_get("tags_to_notes", tag_key);
        assert!(
            tag_ids_result.is_ok(),
            "Tag index should be readable: {tag_ids_result:?}"
        );
        let Ok(tag_ids) = tag_ids_result else {
            return;
        };
        assert!(
            tag_ids.contains(&id_str),
            "Tag index should contain updated note id"
        );

        let stored_result = db.get_owned::<Note>("notes", &id_str);
        assert!(
            stored_result.is_ok(),
            "Read-back should succeed: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_some(), "Updated note should exist");
        let Some(stored_note) = stored else {
            return;
        };
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
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test DB: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let note_result = cmd.create("notes/a.md".to_owned());
        assert!(note_result.is_ok(), "Create should succeed: {note_result:?}");
        let Ok(mut note) = note_result else {
            return;
        };
        let id = note.id;
        let id_str = id.to_string();

        let tag_result = Tag::new("#project");
        assert!(tag_result.is_ok(), "Tag should parse: {tag_result:?}");
        let Ok(tag) = tag_result else {
            return;
        };
        note.tags = vec![tag];
        let update_result = cmd.update(note.clone());
        assert!(
            update_result.is_ok(),
            "Update should succeed: {update_result:?}"
        );

        let delete_result = cmd.delete(id);
        assert!(
            delete_result.is_ok(),
            "Delete should succeed: {delete_result:?}"
        );

        let stored_result = db.get_owned::<Note>("notes", &id_str);
        assert!(
            stored_result.is_ok(),
            "Read-back should succeed: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_none(), "Deleted note should not exist");

        let path_ids_result = db.multimap_get("path_to_id", note.path.as_str());
        assert!(
            path_ids_result.is_ok(),
            "Path index should be readable: {path_ids_result:?}"
        );
        let Ok(path_ids) = path_ids_result else {
            return;
        };
        assert!(
            !path_ids.contains(&id_str),
            "Path index should not contain deleted note id"
        );

        let tag_key = note.tags.first().map(|t| t.full_path.as_str());
        assert!(tag_key.is_some(), "Note should have one tag");
        let Some(tag_key) = tag_key else {
            return;
        };
        let tag_ids_result = db.multimap_get("tags_to_notes", tag_key);
        assert!(
            tag_ids_result.is_ok(),
            "Tag index should be readable: {tag_ids_result:?}"
        );
        let Ok(tag_ids) = tag_ids_result else {
            return;
        };
        assert!(
            !tag_ids.contains(&id_str),
            "Tag index should not contain deleted note id"
        );
    }

    #[test]
    fn delete_missing_note_is_noop() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test DB: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let existing_result = cmd.create("notes/existing.md".to_owned());
        assert!(
            existing_result.is_ok(),
            "Create should succeed: {existing_result:?}"
        );
        let result = cmd.delete(TEST_MISSING_ID);

        assert!(
            result.is_ok(),
            "Deleting missing note should be a no-op, got: {result:?}"
        );
    }

    #[test]
    fn update_removes_old_tag_indexes_when_tags_change() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test DB: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        // GIVEN: a note with an initial tag
        let note_result = cmd.create("notes/test.md".to_owned());
        assert!(note_result.is_ok(), "Create should succeed: {note_result:?}");
        let Ok(mut note) = note_result else {
            return;
        };
        let id_str = note.id.to_string();
        let old_tag_result = Tag::new("#old-tag");
        assert!(old_tag_result.is_ok(), "Tag should parse: {old_tag_result:?}");
        let Ok(old_tag) = old_tag_result else {
            return;
        };
        note.tags = vec![old_tag.clone()];
        let initial_update_result = cmd.update(note.clone());
        assert!(
            initial_update_result.is_ok(),
            "Update should succeed: {initial_update_result:?}"
        );

        // Verify old tag is indexed
        let old_tag_ids = db
            .multimap_get("tags_to_notes", old_tag.full_path.as_str())
            .map_err(|e| e.to_string());
        assert!(
            old_tag_ids.is_ok(),
            "Old tag index lookup should succeed: {old_tag_ids:?}"
        );
        let Ok(old_tag_ids) = old_tag_ids else {
            return;
        };
        assert!(
            old_tag_ids.contains(&id_str),
            "Old tag index should contain note before update"
        );

        // WHEN: updating the note with a different tag
        let new_tag_result = Tag::new("#new-tag");
        assert!(new_tag_result.is_ok(), "Tag should parse: {new_tag_result:?}");
        let Ok(new_tag) = new_tag_result else {
            return;
        };
        note.tags = vec![new_tag.clone()];
        let second_update_result = cmd.update(note.clone());
        assert!(
            second_update_result.is_ok(),
            "Update should succeed: {second_update_result:?}"
        );

        // THEN: old tag index should not contain the note
        let old_tag_ids_after = db
            .multimap_get("tags_to_notes", old_tag.full_path.as_str())
            .map_err(|e| e.to_string());
        assert!(
            old_tag_ids_after.is_ok(),
            "Old tag index lookup should succeed: {old_tag_ids_after:?}"
        );
        let Ok(old_tag_ids_after) = old_tag_ids_after else {
            return;
        };
        assert!(
            !old_tag_ids_after.contains(&id_str),
            "Old tag index should not contain note after update with \
             different tag"
        );

        // AND: new tag index should contain the note
        let new_tag_ids = db
            .multimap_get("tags_to_notes", new_tag.full_path.as_str())
            .map_err(|e| e.to_string());
        assert!(
            new_tag_ids.is_ok(),
            "New tag index lookup should succeed: {new_tag_ids:?}"
        );
        let Ok(new_tag_ids) = new_tag_ids else {
            return;
        };
        assert!(
            new_tag_ids.contains(&id_str),
            "New tag index should contain note after update"
        );
    }
}
