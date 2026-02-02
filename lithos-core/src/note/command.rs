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

        if let Some(fm) = note.frontmatter.as_ref() {
            self.db.put_json("frontmatter", &id_str, fm).map_err(
                |e: crate::db::DbError| NoteError::Storage(e.to_string()),
            )?;
        }

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

            // 5. Delete frontmatter (stored separately)
            let _deleted: bool =
                self.db.delete("frontmatter", &id_str).map_err(
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

        // 5. Save frontmatter separately (recursive, serde JSON)
        if let Some(fm) = note.frontmatter.as_ref() {
            self.db.put_json("frontmatter", &id_str, fm).map_err(
                |e: crate::db::DbError| NoteError::Storage(e.to_string()),
            )?;
        } else {
            let _deleted: bool =
                self.db.delete("frontmatter", &id_str).map_err(
                    |e: crate::db::DbError| NoteError::Storage(e.to_string()),
                )?;
        }

        Ok(note)
    }
}
