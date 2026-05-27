//! Write operations for Note persistence.
//!
//! Implements the [`WriteRepository`] trait against redb storage. Each write
//! method opens a single write transaction and operates on the `NOTES` and
//! `NOTE_ID_BY_PATH` tables within that transaction.
//!
//! ## Path uniqueness
//!
//! The module enforces a unique-path constraint: no two notes may share the
//! same path. Both [`save`](WriteRepository::save) and
//! [`save_many`](WriteRepository::save_many) call
//! [`assert_path_available`] before writing to check for conflicts. The
//! check runs in a separate read transaction, creating a TOCTOU window — a
//! concurrent writer could insert the same path between the check and the
//! write. Under redb's single-writer serialization this is safe in practice,
//! but callers should be aware of the theoretical contention window.
//!
//! ## Delete semantics
//!
//! Both [`delete`](WriteRepository::delete) and
//! [`delete_many`](WriteRepository::delete_many) use `redb`'s
//! `remove().map()` pattern to recover the note from the primary table
//! **as part of the removal**, obtaining the path without a separate
//! lookup. This avoids the O(N) scan found in the vault module's delete
//! contexts.
//!
//! ## List view cache
//!
//! [`save_list_view`] and [`delete_list_view`] manage pre-computed
//! [`ListView`] records used for efficient listing.
//!
//! The trait defines per-method documentation; this file contains the
//! concrete [`redb`] access patterns for each operation.

use super::{NOTES, RedbRepository};
use crate::{
    db::ArchivedEntity,
    note::{
        aggregate::{Note, NoteId},
        error::NoteRepositoryError,
        repository::WriteRepository,
        views::ListView,
    },
};

impl RedbRepository {
    /// Checks that no other note occupies the given path.
    ///
    /// Performs a read-transaction lookup of the path-to-ID index. If an
    /// entry exists with a different ID, returns
    /// [`NoteRepositoryError::DuplicatePath`].
    ///
    /// # Parameters
    ///
    /// * `path` — The path to check for conflicts.
    /// * `current_id` — The note ID to exclude from the conflict check (so that
    ///   updating an existing note does not self-conflict).
    ///
    /// # Errors
    ///
    /// Returns [`NoteRepositoryError::DuplicatePath`] if `path` is already
    /// assigned to a different note. Returns
    /// [`NoteRepositoryError::Storage`] if the database operation fails.
    fn assert_path_available(
        &self,
        path: &crate::note::paths::NotePath,
        current_id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        let existing = self
            .store
            .read(|tx| {
                let Some(path_table) =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };

                let path_key = crate::fs::PathKey::try_new(path.as_str())
                    .map_err(|e| {
                        crate::db::DbError::Deserialization(e.to_string())
                    })?;
                path_table
                    .get(&path_key)?
                    .map(|g| NoteId::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(NoteRepositoryError::from)?;

        if let Some(existing_id) = existing
            && existing_id != current_id
        {
            return Err(NoteRepositoryError::DuplicatePath(path.clone()));
        }

        Ok(())
    }
}

impl WriteRepository for RedbRepository {
    #[inline]
    fn save(&self, note: &Note) -> Result<NoteId, NoteRepositoryError> {
        let id = note.id();
        self.assert_path_available(note.path(), id)?;
        let note_bytes = note.to_bytes()?;
        let id_bytes = id.to_bytes()?;

        self.store
            .write(|tx| {
                let mut note_table = tx.try_open_table(NOTES.definition())?;
                let mut path_table =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?;
                let path_key =
                    crate::fs::PathKey::try_new(note.path().as_str()).map_err(
                        |e| crate::db::DbError::Deserialization(e.to_string()),
                    )?;
                note_table.insert(&id, note_bytes.as_slice())?;
                path_table.insert(&path_key, id_bytes.as_slice())?;
                Ok(id)
            })
            .map_err(NoteRepositoryError::from)
    }

    #[inline]
    fn save_many(
        &self,
        notes: &[Note],
    ) -> Result<Vec<NoteId>, NoteRepositoryError> {
        for note in notes {
            self.assert_path_available(note.path(), note.id())?;
        }

        self.store
            .write(|tx| {
                let mut note_table = tx.try_open_table(NOTES.definition())?;
                let mut path_table =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?;

                let mut ids = Vec::with_capacity(notes.len());
                for note in notes {
                    let id = note.id();
                    let note_bytes = note.to_bytes()?;
                    let id_bytes = id.to_bytes()?;
                    let path_key =
                        crate::fs::PathKey::try_new(note.path().as_str())
                            .map_err(|e| {
                                crate::db::DbError::Deserialization(
                                    e.to_string(),
                                )
                            })?;
                    note_table.insert(&id, note_bytes.as_slice())?;
                    path_table.insert(&path_key, id_bytes.as_slice())?;
                    ids.push(id);
                }

                Ok(ids)
            })
            .map_err(NoteRepositoryError::from)
    }

    #[inline]
    fn delete(&self, id: NoteId) -> Result<(), NoteRepositoryError> {
        self.store
            .write(|tx| {
                let mut note_table = tx.try_open_table(NOTES.definition())?;
                let mut path_table =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?;

                if let Some(existing) = note_table.remove(&id)? {
                    let note = Note::from_bytes(existing.value())?;
                    let path_key =
                        crate::fs::PathKey::try_new(note.path().as_str())
                            .map_err(|e| {
                                crate::db::DbError::Deserialization(
                                    e.to_string(),
                                )
                            })?;
                    let _ = path_table.remove(&path_key)?;
                }

                Ok(())
            })
            .map_err(NoteRepositoryError::from)
    }

    #[inline]
    fn delete_many(&self, ids: &[NoteId]) -> Result<(), NoteRepositoryError> {
        self.store
            .write(|tx| {
                let mut note_table = tx.try_open_table(NOTES.definition())?;
                let mut path_table =
                    tx.try_open_table(super::NOTE_ID_BY_PATH.definition())?;

                for id in ids {
                    if let Some(existing) = note_table.remove(id)? {
                        let note = Note::from_bytes(existing.value())?;
                        let path_key =
                            crate::fs::PathKey::try_new(note.path().as_str())
                                .map_err(|e| {
                                crate::db::DbError::Deserialization(
                                    e.to_string(),
                                )
                            })?;
                        let _ = path_table.remove(&path_key)?;
                    }
                }

                Ok(())
            })
            .map_err(NoteRepositoryError::from)
    }

    #[inline]
    fn save_list_view(
        &self,
        view: &ListView,
    ) -> Result<(), NoteRepositoryError> {
        let bytes = view.to_bytes()?;
        let note_id = view.note_id();

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::LIST_VIEWS.definition())?;
                table.insert(&note_id, bytes.as_slice())?;
                Ok(())
            })
            .map_err(NoteRepositoryError::from)
    }

    #[inline]
    fn delete_list_view(
        &self,
        note_id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::LIST_VIEWS.definition())?;
                let _ = table.remove(&note_id)?;
                Ok(())
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
            error::NoteRepositoryError,
            paths::NotePath,
            repository::{ReadRepository, WriteRepository},
            storage::RedbRepository,
            views::ListView,
        },
    };

    mod fixtures {
        use super::*;

        pub(super) fn create_note(path: &str) -> Note {
            Note::new(NoteId::new(), NotePath::try_new(path).unwrap())
        }

        pub(super) fn repo() -> (tempfile::TempDir, RedbRepository) {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let store = Arc::new(Store::open(&db_path).unwrap());
            let repo = RedbRepository::new(store);
            (temp_dir, repo)
        }
    }

    mod save {
        use super::*;

        #[test]
        fn persists_note_and_path_index() {
            let (_tmp, repo) = fixtures::repo();
            let note = fixtures::create_note("save-me.md");

            let id = repo.save(&note).unwrap();

            assert_eq!(id, note.id(), "Saved id should match note id");
            let by_id = repo.find_by_id(id).unwrap();
            assert!(by_id.is_some(), "Saved note should be findable by id");
            let by_path = repo.find_by_path(note.path()).unwrap();
            assert!(by_path.is_some(), "Saved note should be findable by path");
        }

        #[test]
        fn persists_batch_of_notes() {
            let (_tmp, repo) = fixtures::repo();
            let note1 = fixtures::create_note("batch-a.md");
            let note2 = fixtures::create_note("batch-b.md");

            let ids = repo.save_many(&[note1.clone(), note2.clone()]).unwrap();

            assert_eq!(ids.len(), 2, "Should return two ids");
            let first = ids.first().copied().expect("First id must exist");
            let second = ids.get(1).copied().expect("Second id must exist");

            let by_id_1 = repo.find_by_id(first).unwrap();
            let by_id_2 = repo.find_by_id(second).unwrap();
            let by_path_1 = repo.find_by_path(note1.path()).unwrap();
            let by_path_2 = repo.find_by_path(note2.path()).unwrap();

            assert!(by_id_1.is_some(), "First note should be findable by id");
            assert!(by_id_2.is_some(), "Second note should be findable by id");
            assert!(
                by_path_1.is_some(),
                "First note should be findable by path"
            );
            assert!(
                by_path_2.is_some(),
                "Second note should be findable by path"
            );
        }

        #[test]
        fn rejects_duplicate_path() {
            let (_tmp, repo) = fixtures::repo();
            let note1 = fixtures::create_note("dup.md");
            let note2 = fixtures::create_note("dup.md");

            let _ = repo.save(&note1).unwrap();
            let result = repo.save(&note2);

            assert!(
                matches!(result, Err(NoteRepositoryError::DuplicatePath(_))),
                "Should reject duplicate path: {result:?}"
            );
        }

        #[test]
        fn updates_existing_note_with_same_path() {
            let (_tmp, repo) = fixtures::repo();
            let original = fixtures::create_note("stable.md");

            let id = repo.save(&original).unwrap();
            let updated = original.clone().with_id(id);
            let id2 = repo.save(&updated).unwrap();

            assert_eq!(id, id2, "Update should return original id");
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn removes_note_completely() {
            let (_tmp, repo) = fixtures::repo();
            let note = fixtures::create_note("delete-one.md");
            let path = note.path().clone();
            let id = repo.save(&note).unwrap();

            repo.delete(id).unwrap();

            let by_id = repo.find_by_id(id).unwrap();
            let by_path = repo.find_by_path(&path).unwrap();
            assert!(by_id.is_none(), "Note should be gone after delete");
            assert!(by_path.is_none(), "Path should be gone after delete");
        }

        #[test]
        fn is_idempotent_for_missing_id() {
            let (_tmp, repo) = fixtures::repo();
            let missing = NoteId::new();

            repo.delete(missing).unwrap();
            repo.delete(missing).unwrap();
        }

        #[test]
        fn batch_removes_all_notes() {
            let (_tmp, repo) = fixtures::repo();
            let note1 = fixtures::create_note("delete-many-a.md");
            let note2 = fixtures::create_note("delete-many-b.md");
            let path1 = note1.path().clone();
            let path2 = note2.path().clone();
            let id1 = repo.save(&note1).unwrap();
            let id2 = repo.save(&note2).unwrap();

            repo.delete_many(&[id1, id2]).unwrap();

            let by_id_1 = repo.find_by_id(id1).unwrap();
            let by_id_2 = repo.find_by_id(id2).unwrap();
            let by_path_1 = repo.find_by_path(&path1).unwrap();
            let by_path_2 = repo.find_by_path(&path2).unwrap();

            assert!(by_id_1.is_none(), "First note should be removed");
            assert!(by_id_2.is_none(), "Second note should be removed");
            assert!(by_path_1.is_none(), "First path should be removed");
            assert!(by_path_2.is_none(), "Second path should be removed");
        }
    }

    mod caching {
        use super::*;

        #[test]
        fn persists_list_view() {
            let (_tmp, repo) = fixtures::repo();
            let note = fixtures::create_note("cache-save.md");
            let id = repo.save(&note).unwrap();
            let view = ListView::from_note_items(id, note.list_items());

            repo.save_list_view(&view).unwrap();

            let found = repo.find_list_view(id).unwrap();
            assert!(found.is_some(), "Saved list view should be findable");
        }

        #[test]
        fn removes_cached_list_view() {
            let (_tmp, repo) = fixtures::repo();
            let note = fixtures::create_note("cache-delete.md");
            let id = repo.save(&note).unwrap();
            let view = ListView::from_note_items(id, note.list_items());
            repo.save_list_view(&view).unwrap();

            repo.delete_list_view(id).unwrap();

            let found = repo.find_list_view(id).unwrap();
            assert!(
                found.is_none(),
                "List view should be removed after delete"
            );
        }
    }
}
