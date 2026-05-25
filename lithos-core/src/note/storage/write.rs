//! `WriteRepository` implementation for Note persistence.

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

                path_table
                    .get(path.as_str().to_owned())?
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
                note_table.insert(&id, note_bytes.as_slice())?;
                path_table.insert(
                    note.path().as_str().to_owned(),
                    id_bytes.as_slice(),
                )?;
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
                    note_table.insert(&id, note_bytes.as_slice())?;
                    path_table.insert(
                        note.path().as_str().to_owned(),
                        id_bytes.as_slice(),
                    )?;
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
                    let _ =
                        path_table.remove(note.path().as_str().to_owned())?;
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
                        let _ = path_table
                            .remove(note.path().as_str().to_owned())?;
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

    fn create_test_note_with_path(path: &str) -> Note {
        Note::new(NoteId::new(), NotePath::try_new(path).unwrap())
    }

    #[test]
    fn save_many_persists_all_notes_and_path_index() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note1 = create_test_note_with_path("batch-a.md");
        let note2 = create_test_note_with_path("batch-b.md");

        let ids = repo.save_many(&[note1.clone(), note2.clone()]).unwrap();

        assert_eq!(ids.len(), 2);
        let first = ids.first().copied().expect("first id must exist");
        let second = ids.get(1).copied().expect("second id must exist");

        let by_id_1 = repo.find_by_id(first).unwrap();
        let by_id_2 = repo.find_by_id(second).unwrap();
        let by_path_1 = repo.find_by_path(note1.path()).unwrap();
        let by_path_2 = repo.find_by_path(note2.path()).unwrap();

        assert!(by_id_1.is_some());
        assert!(by_id_2.is_some());
        assert!(by_path_1.is_some());
        assert!(by_path_2.is_some());
    }

    #[test]
    fn save_rejects_duplicate_path_for_different_note_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note1 = create_test_note_with_path("dup.md");
        let note2 = create_test_note_with_path("dup.md");

        let _ = repo.save(&note1).unwrap();
        let result = repo.save(&note2);

        assert!(matches!(result, Err(NoteRepositoryError::DuplicatePath(_))));
    }

    #[test]
    fn save_allows_updating_existing_note_same_id_same_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let original = create_test_note_with_path("stable.md");
        let id = repo.save(&original).unwrap();
        let updated = original.clone().with_id(id);

        let id2 = repo.save(&updated).unwrap();

        assert_eq!(id, id2);
        let found = repo.find_by_id(id).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn delete_removes_note_and_path_mapping() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note = create_test_note_with_path("delete-one.md");
        let path = note.path().clone();
        let id = repo.save(&note).unwrap();

        repo.delete(id).unwrap();

        let by_id = repo.find_by_id(id).unwrap();
        let by_path = repo.find_by_path(&path).unwrap();

        assert!(by_id.is_none());
        assert!(by_path.is_none());
    }

    #[test]
    fn delete_is_idempotent_for_missing_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let missing = NoteId::new();

        repo.delete(missing).unwrap();
        repo.delete(missing).unwrap();
    }

    #[test]
    fn delete_many_removes_all_notes_and_path_mappings() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note1 = create_test_note_with_path("delete-many-a.md");
        let note2 = create_test_note_with_path("delete-many-b.md");
        let path1 = note1.path().clone();
        let path2 = note2.path().clone();
        let id1 = repo.save(&note1).unwrap();
        let id2 = repo.save(&note2).unwrap();

        repo.delete_many(&[id1, id2]).unwrap();

        let by_id_1 = repo.find_by_id(id1).unwrap();
        let by_id_2 = repo.find_by_id(id2).unwrap();
        let by_path_1 = repo.find_by_path(&path1).unwrap();
        let by_path_2 = repo.find_by_path(&path2).unwrap();

        assert!(by_id_1.is_none());
        assert!(by_id_2.is_none());
        assert!(by_path_1.is_none());
        assert!(by_path_2.is_none());
    }

    #[test]
    fn delete_list_view_removes_cached_view() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note = create_test_note_with_path("delete-cache.md");
        let id = repo.save(&note).unwrap();
        let view = ListView::from_note_items(id, note.list_items());
        repo.save_list_view(&view).unwrap();

        repo.delete_list_view(id).unwrap();

        let found = repo.find_list_view(id).unwrap();
        assert!(found.is_none());
    }
}
