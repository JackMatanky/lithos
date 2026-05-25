//! `WriteRepository` implementation for Note persistence.

use super::{NOTES, RedbRepository};
use crate::{
    db::ArchivedEntity,
    note::{
        aggregate::{Note, NoteId},
        error::NoteRepositoryError,
        repository::WriteRepository,
    },
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save(&self, note: &Note) -> Result<NoteId, NoteRepositoryError> {
        let id = note.id();
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
}
