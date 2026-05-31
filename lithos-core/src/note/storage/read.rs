//! Read-only repository operations for Note persistence.
//!
//! This module implements the [`ReadRepository`] trait for the Note context,
//! providing all read operations over the redb-backed storage layer.
//!
//! # Architecture
//!
//! - **Transaction Boundaries**: Each method manages its own read transaction
//!   via the shared [`Store`](crate::db::Store).
//! - **Zero-Copy Deserialization**: Uses rkyv for efficient deserialization
//!   directly from database bytes without intermediate allocations.
//! - **Graceful Degradation**: Returns `Ok(None)` or `Ok(Vec::new())` when
//!   tables don't exist yet (fresh database state).
//!
//! # Indexes
//!
//! Read operations leverage the following indexes:
//! - Primary table: [`NOTES`](super::NOTES) — note ID → full Note aggregate
//! - Path index: [`NOTE_ID_BY_PATH`](super::NOTE_ID_BY_PATH) — path → note ID
//! - View cache: [`LIST_VIEWS`](super::LIST_VIEWS) — note ID → `ListView`
//!
//! # Examples
//!
//! ```rust,ignore
//! use lithos_core::note::{
//!     repository::ReadRepository,
//!     storage::RedbRepository,
//! };
//!
//! let repo = RedbRepository::new(store);
//!
//! // Find by ID
//! let note = repo.find_by_id(note_id)?;
//!
//! // Find by path (cross-table lookup)
//! let note = repo.find_by_path(&NotePath::try_new("daily/2024-05-25.md")?)?;
//!
//! // List all notes
//! let all_notes = repo.list()?;
//! ```

use redb::ReadableTable;

use super::{NOTES, RedbRepository};
use crate::{
    db::ArchivedEntity,
    note::{
        aggregate::{Note, NoteId},
        error::NoteRepositoryError,
        paths::NotePath,
        repository::ReadRepository,
        views::ListView,
    },
};

impl ReadRepository for RedbRepository {
    /// Finds a note by its unique identifier.
    ///
    /// Returns `Ok(None)` if the note table doesn't exist or if no note with
    /// the given ID exists in the database.
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::Storage`] if the database operation fails
    /// (e.g., I/O error, transaction conflict).
    ///
    /// Returns [`NoteRepositoryError::Deserialization`] if the stored bytes
    /// cannot be deserialized into a valid [`Note`].
    #[inline]
    fn find_by_id(
        &self,
        id: NoteId,
    ) -> Result<Option<Note>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(NOTES.definition())? else {
                    return Ok(None);
                };
                let Some(guard) = table.get(&id)? else {
                    return Ok(None);
                };
                let note = Note::from_bytes(guard.value())?;
                Ok(Some(note))
            })
            .map_err(NoteRepositoryError::from)
    }

    /// Finds a note by its vault-relative path.
    ///
    /// Performs a cross-table lookup:
    /// 1. Path index lookup: `path` → `note_id`
    /// 2. Primary table lookup: `note_id` → `Note`
    ///
    /// Returns `Ok(None)` if the path index or note table doesn't exist, or if
    /// no note exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::Storage`] if either database operation
    /// fails.
    ///
    /// Returns [`NoteRepositoryError::Deserialization`] if the stored bytes
    /// (note ID or note) cannot be deserialized.
    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let Some(path_table) =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };
                let Some(note_table) = tx.try_open_table(NOTES.definition())?
                else {
                    return Ok(None);
                };

                let path_key = crate::fs::PathKey::try_new(path.as_str())
                    .map_err(|e| {
                        crate::db::DbError::Deserialization(e.to_string())
                    })?;
                let Some(id_guard) = path_table.get(&path_key)? else {
                    return Ok(None);
                };
                let id = id_guard.value();

                let Some(note_guard) = note_table.get(&id)? else {
                    return Ok(None);
                };
                let note = Note::from_bytes(note_guard.value())?;
                Ok(Some(note))
            })
            .map_err(NoteRepositoryError::from)
    }

    /// Finds multiple notes by their IDs in a single transaction.
    ///
    /// Missing notes are skipped silently. The returned vector preserves the
    /// order of the input IDs (only found notes appear in the result).
    ///
    /// Returns an empty vector if the note table doesn't exist.
    ///
    /// # Performance
    ///
    /// Uses a single read transaction for all lookups. Prefer this over
    /// multiple `find_by_id` calls when retrieving multiple notes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::Storage`] if any database operation
    /// fails.
    ///
    /// Returns [`NoteRepositoryError::Deserialization`] if any stored bytes
    /// cannot be deserialized into a valid [`Note`].
    #[inline]
    fn find_many_by_id(
        &self,
        ids: &[NoteId],
    ) -> Result<Vec<Note>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(NOTES.definition())? else {
                    return Ok(Vec::new());
                };

                let mut notes = Vec::with_capacity(ids.len());
                for id in ids {
                    if let Some(guard) = table.get(id)? {
                        notes.push(Note::from_bytes(guard.value())?);
                    }
                }

                Ok(notes)
            })
            .map_err(NoteRepositoryError::from)
    }

    /// Lists all notes in the database.
    ///
    /// Returns all persisted notes in an unordered collection. Returns an empty
    /// vector if the note table doesn't exist yet (fresh database state).
    ///
    /// # Performance
    ///
    /// Performs a full table scan. For large vaults (1000+ notes), consider
    /// using more specific queries if you need a subset of notes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::Storage`] if the database operation
    /// fails.
    ///
    /// Returns [`NoteRepositoryError::Deserialization`] if any stored bytes
    /// cannot be deserialized into a valid [`Note`].
    #[inline]
    fn list(&self) -> Result<Vec<Note>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(NOTES.definition())? else {
                    return Ok(Vec::new());
                };

                let mut notes = Vec::new();
                for result in table.iter()? {
                    let (_id_guard, note_guard) = result?;
                    notes.push(Note::from_bytes(note_guard.value())?);
                }

                Ok(notes)
            })
            .map_err(NoteRepositoryError::from)
    }

    /// Finds a cached list view projection for a note.
    ///
    /// List views are materialized projections of notes containing list items,
    /// stored separately for query optimization. This is a rebuildable cache.
    ///
    /// Returns `Ok(None)` if the list view table doesn't exist or if no cached
    /// view exists for the given note ID.
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::Storage`] if the database operation
    /// fails.
    ///
    /// Returns [`NoteRepositoryError::Deserialization`] if the stored bytes
    /// cannot be deserialized into a valid [`ListView`].
    #[inline]
    fn find_list_view(
        &self,
        note_id: NoteId,
    ) -> Result<Option<ListView>, NoteRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(super::LIST_VIEWS.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(&note_id)? else {
                    return Ok(None);
                };

                let view = ListView::from_bytes(guard.value())?;
                Ok(Some(view))
            })
            .map_err(NoteRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::Store,
        note::{
            aggregate::{Note, NoteId},
            paths::NotePath,
            repository::{ReadRepository, WriteRepository},
            storage::RedbRepository,
            views::ListView,
        },
    };

    mod fixtures {
        use super::*;

        // Creates an isolated temp-db repo for a single test.
        pub(super) fn repo() -> (tempfile::TempDir, RedbRepository) {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            (temp_dir, RedbRepository::new(store))
        }

        pub(super) fn note() -> Note {
            Note::new_test(NoteId::new(), NotePath::try_new("test.md").unwrap())
        }

        pub(super) fn note_with_path(path: &str) -> Note {
            Note::new_test(NoteId::new(), NotePath::try_new(path).unwrap())
        }
    }

    mod lookup {
        use super::*;

        #[test]
        fn returns_none_when_note_missing() {
            let (_temp, repo) = fixtures::repo();
            let found = repo.find_by_id(NoteId::new()).unwrap();
            assert!(found.is_none(), "Expected None for missing note id");
        }

        #[test]
        fn returns_saved_note_when_id_exists() {
            let (_temp, repo) = fixtures::repo();
            let note = fixtures::note();
            let id = repo.save(&note).unwrap();
            let found = repo.find_by_id(id).unwrap();
            let found = found.expect("Saved note should be found by id");
            assert_eq!(found.id(), id, "Returned note id should match");
        }

        #[test]
        fn returns_note_when_path_exists() {
            let (_temp, repo) = fixtures::repo();
            let note = fixtures::note();
            let path = note.path().clone();
            repo.save(&note).unwrap();
            let found = repo.find_by_path(&path).unwrap();
            let found = found.expect("Note should be found by saved path");
            assert_eq!(found.path(), &path, "Returned note path should match");
        }

        #[test]
        fn returns_none_when_path_does_not_exist() {
            let (_temp, repo) = fixtures::repo();
            let path = NotePath::try_new("nonexistent.md").unwrap();
            let found = repo.find_by_path(&path).unwrap();
            assert!(found.is_none(), "Expected None for unsaved path");
        }

        #[test]
        fn skips_missing_ids_silently() {
            let (_temp, repo) = fixtures::repo();
            let saved = fixtures::note();
            let saved_id = repo.save(&saved).unwrap();
            let missing = NoteId::new();
            let found = repo.find_many_by_id(&[saved_id, missing]).unwrap();
            assert_eq!(
                found.len(),
                1,
                "Should skip missing id and return only the saved note"
            );
            assert_eq!(
                found.first().expect("First note must exist").id(),
                saved_id,
                "Returned note should match saved id"
            );
        }

        #[test]
        fn preserves_input_order_for_found_notes() {
            let (_temp, repo) = fixtures::repo();
            let note1 = fixtures::note_with_path("a.md");
            let note2 = fixtures::note_with_path("b.md");
            let id1 = repo.save(&note1).unwrap();
            let id2 = repo.save(&note2).unwrap();
            let found = repo.find_many_by_id(&[id2, id1]).unwrap();
            assert_eq!(found.len(), 2, "Both saved notes should be returned");
            assert_eq!(
                found.first().expect("First result must exist").id(),
                id2,
                "First result should match first input id"
            );
            assert_eq!(
                found.get(1).expect("Second result must exist").id(),
                id1,
                "Second result should match second input id"
            );
        }

        #[test]
        fn returns_empty_when_no_notes_table() {
            let (_temp, repo) = fixtures::repo();
            let found = repo.find_many_by_id(&[NoteId::new()]).unwrap();
            assert!(
                found.is_empty(),
                "Expected empty vec when no NOTES table exists"
            );
        }
    }

    mod list {
        use super::*;

        #[test]
        fn returns_all_saved_notes() {
            let (_temp, repo) = fixtures::repo();
            let note1 = fixtures::note_with_path("alpha.md");
            let note2 = fixtures::note_with_path("beta.md");
            let id1 = repo.save(&note1).unwrap();
            let id2 = repo.save(&note2).unwrap();
            let listed = repo.list().unwrap();
            let ids: std::collections::HashSet<NoteId> =
                listed.into_iter().map(|n| n.id()).collect();
            assert!(ids.contains(&id1), "List should contain note alpha.md");
            assert!(ids.contains(&id2), "List should contain note beta.md");
        }

        #[test]
        fn returns_empty_when_no_notes_table() {
            let (_temp, repo) = fixtures::repo();
            let listed = repo.list().unwrap();
            assert!(
                listed.is_empty(),
                "Expected empty list for fresh database with no NOTES table"
            );
        }
    }

    mod caching {
        use super::*;

        #[test]
        fn returns_none_when_not_cached() {
            let (_temp, repo) = fixtures::repo();
            let found = repo.find_list_view(NoteId::new()).unwrap();
            assert!(
                found.is_none(),
                "Expected None for note with no cached view"
            );
        }

        #[test]
        fn returns_cached_view_when_id_has_view() {
            let (_temp, repo) = fixtures::repo();
            let note = fixtures::note_with_path("list-view.md");
            let id = repo.save(&note).unwrap();
            let view = ListView::from_note_items(id, note.list_items());
            repo.save_list_view(&view).unwrap();
            let found = repo.find_list_view(id).unwrap();
            let found = found.expect("Cached list view should be found");
            assert_eq!(
                found.note_id(),
                id,
                "Cached view should reference the correct note id"
            );
        }
    }
}
