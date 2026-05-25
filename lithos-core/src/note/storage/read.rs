//! `ReadRepository` implementation for Note persistence.

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

                let Some(id_guard) =
                    path_table.get(path.as_str().to_owned())?
                else {
                    return Ok(None);
                };
                let id = NoteId::from_bytes(id_guard.value())?;

                let Some(note_guard) = note_table.get(&id)? else {
                    return Ok(None);
                };
                let note = Note::from_bytes(note_guard.value())?;
                Ok(Some(note))
            })
            .map_err(NoteRepositoryError::from)
    }

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

    /// Creates a minimal test note for round-trip testing.
    fn create_test_note() -> Note {
        Note::new(NoteId::new(), NotePath::try_new("test.md").unwrap())
    }

    fn create_test_note_with_path(path: &str) -> Note {
        Note::new(NoteId::new(), NotePath::try_new(path).unwrap())
    }

    #[test]
    fn find_by_id_returns_none_when_note_missing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let missing_id = NoteId::new();
        let found = repo.find_by_id(missing_id).unwrap();

        assert!(found.is_none());
    }

    #[test]
    fn save_and_find_by_id_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note = create_test_note();
        let id = repo.save(&note).unwrap();

        let found = repo.find_by_id(id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), id);
    }

    #[test]
    fn find_by_path_returns_some_when_path_exists() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note = create_test_note();
        let path = note.path().clone();
        let _id = repo.save(&note).unwrap();

        let found = repo.find_by_path(&path).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().path(), &path);
    }

    #[test]
    fn find_many_by_id_skips_missing_and_preserves_input_order() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note1 = create_test_note_with_path("one.md");
        let note2 = create_test_note_with_path("two.md");
        let id1 = repo.save(&note1).unwrap();
        let id2 = repo.save(&note2).unwrap();
        let missing = NoteId::new();

        let found = repo.find_many_by_id(&[id2, missing, id1]).unwrap();

        assert_eq!(found.len(), 2);
        let first = found.first().expect("first note must exist");
        let second = found.get(1).expect("second note must exist");
        assert_eq!(first.id(), id2);
        assert_eq!(second.id(), id1);
    }

    #[test]
    fn list_returns_all_persisted_notes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note1 = create_test_note_with_path("alpha.md");
        let note2 = create_test_note_with_path("beta.md");
        let id1 = repo.save(&note1).unwrap();
        let id2 = repo.save(&note2).unwrap();

        let listed_notes = repo.list().unwrap();
        let ids: std::collections::HashSet<NoteId> =
            listed_notes.into_iter().map(|n| n.id()).collect();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn find_list_view_returns_none_when_not_cached() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let found = repo.find_list_view(NoteId::new()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn cache_and_find_list_view_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).unwrap());
        let repo = RedbRepository::new(store);

        let note = create_test_note_with_path("list-view.md");
        let id = repo.save(&note).unwrap();
        let view = ListView::from_note_items(id, note.list_items());

        repo.save_list_view(&view).unwrap();

        let found = repo.find_list_view(id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.expect("cached view must exist").note_id(), id);
    }
}
