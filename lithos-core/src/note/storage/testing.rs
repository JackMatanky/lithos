use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::{
        DbError,
        testing::{FailurePoint, InMemoryHarness, read_lock, write_lock},
    },
    note::{
        aggregate::{Note, NoteId},
        error::NoteRepositoryError,
        paths::NotePath,
        repository::{ReadRepository, WriteRepository},
        views::ListView,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct InMemoryRepository {
    harness: Arc<InMemoryHarness>,
    notes: Arc<RwLock<HashMap<NoteId, Note>>>,
    path_to_id: Arc<RwLock<HashMap<NotePath, NoteId>>>,
    views: Arc<RwLock<HashMap<NoteId, ListView>>>,
}

impl InMemoryRepository {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            harness: Arc::new(InMemoryHarness::new()),
            notes: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            views: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[must_use]
    pub(crate) fn with_harness(harness: InMemoryHarness) -> Self {
        Self {
            harness: Arc::new(harness),
            notes: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            views: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[must_use]
    pub(crate) fn harness(&self) -> &InMemoryHarness {
        &self.harness
    }
}

impl Default for InMemoryRepository {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ReadRepository for InMemoryRepository {
    #[inline]
    fn find_by_id(
        &self,
        id: NoteId,
    ) -> Result<Option<Note>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let notes = read_lock(&self.notes, "find_by_id")?;
        self.harness.counters().inc_read();

        Ok(notes.get(&id).cloned())
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id =
            read_lock(&self.path_to_id, "find_by_path (path_to_id)")?;
        self.harness.counters().inc_read();

        let Some(id) = path_to_id.get(path).copied() else {
            return Ok(None);
        };

        let notes = read_lock(&self.notes, "find_by_path (notes)")?;
        self.harness.counters().inc_read();

        Ok(notes.get(&id).cloned())
    }

    #[inline]
    fn find_many_by_id(
        &self,
        ids: &[NoteId],
    ) -> Result<Vec<Note>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let notes = read_lock(&self.notes, "find_many_by_id")?;
        self.harness.counters().inc_read();

        Ok(ids.iter().filter_map(|id| notes.get(id).cloned()).collect())
    }

    #[inline]
    fn list(&self) -> Result<Vec<Note>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let notes = read_lock(&self.notes, "list")?;
        self.harness.counters().inc_read();

        Ok(notes.values().cloned().collect())
    }

    #[inline]
    fn find_list_view(
        &self,
        note_id: NoteId,
    ) -> Result<Option<ListView>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let views = read_lock(&self.views, "find_list_view")?;
        self.harness.counters().inc_read();

        Ok(views.get(&note_id).cloned())
    }
}

impl WriteRepository for InMemoryRepository {
    #[inline]
    fn save(&self, note: &Note) -> Result<NoteId, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut path_map = write_lock(&self.path_to_id, "save (path_to_id)")?;
        self.harness.counters().inc_write();

        let mut notes_map = write_lock(&self.notes, "save (notes)")?;
        self.harness.counters().inc_write();

        let id = note.id();
        if let Some(existing_id) = path_map.get(note.path()).copied()
            && existing_id != id
        {
            return Err(NoteRepositoryError::DuplicatePath(
                note.path().clone(),
            ));
        }

        path_map.insert(note.path().clone(), id);
        notes_map.insert(id, note.clone());

        Ok(id)
    }

    #[inline]
    fn save_many(
        &self,
        notes: &[Note],
    ) -> Result<Vec<NoteId>, NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut path_map =
            write_lock(&self.path_to_id, "save_many (path_to_id)")?;
        self.harness.counters().inc_write();

        let mut notes_map = write_lock(&self.notes, "save_many (notes)")?;
        self.harness.counters().inc_write();

        for note in notes {
            let id = note.id();
            if let Some(existing_id) = path_map.get(note.path()).copied()
                && existing_id != id
            {
                return Err(NoteRepositoryError::DuplicatePath(
                    note.path().clone(),
                ));
            }

            path_map.insert(note.path().clone(), id);
            notes_map.insert(id, note.clone());
        }

        Ok(notes.iter().map(Note::id).collect())
    }

    #[inline]
    fn delete(&self, id: NoteId) -> Result<(), NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut notes_map = write_lock(&self.notes, "delete (notes)")?;
        self.harness.counters().inc_write();

        if let Some(note) = notes_map.remove(&id) {
            let mut path_map =
                write_lock(&self.path_to_id, "delete (path_to_id)")?;
            self.harness.counters().inc_write();

            path_map.remove(note.path());
        }

        let mut views_map = write_lock(&self.views, "delete (views)")?;
        self.harness.counters().inc_write();

        views_map.remove(&id);

        Ok(())
    }

    #[inline]
    fn delete_many(&self, ids: &[NoteId]) -> Result<(), NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut notes_map = write_lock(&self.notes, "delete_many (notes)")?;
        self.harness.counters().inc_write();

        let mut path_map =
            write_lock(&self.path_to_id, "delete_many (path_to_id)")?;
        self.harness.counters().inc_write();

        let mut views_map = write_lock(&self.views, "delete_many (views)")?;
        self.harness.counters().inc_write();

        for id in ids {
            if let Some(note) = notes_map.remove(id) {
                path_map.remove(note.path());
            }
            views_map.remove(id);
        }

        Ok(())
    }

    #[inline]
    fn save_list_view(
        &self,
        view: &ListView,
    ) -> Result<(), NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views = write_lock(&self.views, "save_list_view")?;
        self.harness.counters().inc_write();

        views.insert(view.note_id(), view.clone());

        Ok(())
    }

    #[inline]
    fn delete_list_view(
        &self,
        note_id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views = write_lock(&self.views, "delete_list_view")?;
        self.harness.counters().inc_write();

        views.remove(&note_id);

        Ok(())
    }
}

#[cfg(test)]
impl From<crate::db::testing::InMemoryDbError> for NoteRepositoryError {
    #[inline]
    fn from(err: crate::db::testing::InMemoryDbError) -> Self {
        use crate::db::testing::InMemoryDbError as DbTestError;

        let db_error = match err {
            DbTestError::LockPoisoned {
                context,
            } => DbError::Corruption(format!("Lock poisoned: {context}")),
            DbTestError::InjectedFailure {
                reason,
                ..
            } => DbError::Corruption(format!("Injected failure: {reason}")),
            DbTestError::InvariantViolation {
                message,
            } => DbError::Corruption(message.into()),
        };

        NoteRepositoryError::Storage(db_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::{FailureInjector, FailurePoint, InMemoryDbError};

    mod fixtures {
        use super::*;

        pub(super) struct FailOnWrite;
        pub(super) struct FailOnRead;

        impl FailureInjector for FailOnWrite {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeWrite {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "write injection".into(),
                    });
                }

                Ok(())
            }
        }

        impl FailureInjector for FailOnRead {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeRead {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "read injection".into(),
                    });
                }

                Ok(())
            }
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn returns_none_when_repository_is_empty() {
            let repo = InMemoryRepository::new();

            let result = repo.find_by_id(NoteId::new()).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn returns_none_when_path_not_in_index() {
            let repo = InMemoryRepository::new();

            let path = NotePath::try_new("missing.md").unwrap();
            let result = repo.find_by_path(&path).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn returns_none_when_view_not_cached() {
            let repo = InMemoryRepository::new();

            let result = repo.find_list_view(NoteId::new()).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn returns_empty_list_when_no_notes() {
            let repo = InMemoryRepository::new();

            let result = repo.list().unwrap();

            assert!(result.is_empty());
        }

        #[test]
        fn defaults_increment_no_counters() {
            let repo = InMemoryRepository::new();

            let snapshot = repo.harness().counters().snapshot();

            assert_eq!(snapshot.reads, 0);
            assert_eq!(snapshot.writes, 0);
        }
    }

    mod lookup {
        use super::*;
        use crate::note::storage::testing::tests::fixtures::FailOnRead;

        #[test]
        fn returns_note_when_id_exists() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("roundtrip.md").unwrap(),
            );
            let id = repo.save(&note).unwrap();

            let found = repo.find_by_id(id).unwrap();

            assert!(found.is_some());
            assert_eq!(found.unwrap().id(), id);
        }

        #[test]
        fn returns_note_when_path_exists() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("by-path.md").unwrap(),
            );
            let path = note.path().clone();
            let _id = repo.save(&note).unwrap();

            let found = repo.find_by_path(&path).unwrap();

            assert!(found.is_some());
            assert_eq!(found.unwrap().path(), &path);
        }

        #[test]
        fn skips_missing_and_preserves_order() {
            let repo = InMemoryRepository::new();
            let note1 =
                Note::new(NoteId::new(), NotePath::try_new("one.md").unwrap());
            let note2 =
                Note::new(NoteId::new(), NotePath::try_new("two.md").unwrap());
            let id1 = repo.save(&note1).unwrap();
            let id2 = repo.save(&note2).unwrap();
            let missing = NoteId::new();

            let found = repo.find_many_by_id(&[id2, missing, id1]).unwrap();

            assert_eq!(found.len(), 2);
            assert_eq!(found.first().expect("first").id(), id2);
            assert_eq!(found.get(1).expect("second").id(), id1);
        }

        #[test]
        fn increments_read_counter() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("counter.md").unwrap(),
            );
            repo.save(&note).unwrap();

            let _result = repo.find_by_id(note.id()).unwrap();
            let snapshot = repo.harness().counters().snapshot();

            assert!(snapshot.reads >= 1);
        }

        #[test]
        fn returns_storage_error_when_before_read_injected() {
            let harness = InMemoryHarness::with_injector(Box::new(FailOnRead));
            let repo = InMemoryRepository::with_harness(harness);

            let result = repo.find_by_id(NoteId::new());

            assert!(matches!(result, Err(NoteRepositoryError::Storage(_))));
        }
    }

    mod list {
        use super::*;

        #[test]
        fn returns_all_persisted_notes() {
            let repo = InMemoryRepository::new();
            let note1 = Note::new(
                NoteId::new(),
                NotePath::try_new("alpha.md").unwrap(),
            );
            let note2 =
                Note::new(NoteId::new(), NotePath::try_new("beta.md").unwrap());
            let id1 = repo.save(&note1).unwrap();
            let id2 = repo.save(&note2).unwrap();

            let listed = repo.list().unwrap();
            let ids: std::collections::HashSet<NoteId> =
                listed.into_iter().map(|n| n.id()).collect();

            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&id1));
            assert!(ids.contains(&id2));
        }
    }

    mod update {
        use super::*;
        use crate::note::storage::testing::tests::fixtures::FailOnWrite;

        #[test]
        fn persists_note_and_path_index() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("save-me.md").unwrap(),
            );

            let id = repo.save(&note).unwrap();

            let by_id = repo.find_by_id(id).unwrap();
            let by_path = repo.find_by_path(note.path()).unwrap();
            assert!(by_id.is_some());
            assert!(by_path.is_some());
            assert_eq!(by_id.unwrap().path(), note.path());
        }

        #[test]
        fn persists_all_notes_in_batch() {
            let repo = InMemoryRepository::new();
            let note1 = Note::new(
                NoteId::new(),
                NotePath::try_new("batch-a.md").unwrap(),
            );
            let note2 = Note::new(
                NoteId::new(),
                NotePath::try_new("batch-b.md").unwrap(),
            );

            let ids = repo.save_many(&[note1.clone(), note2.clone()]).unwrap();

            assert_eq!(ids.len(), 2);
            for (i, id) in ids.iter().enumerate() {
                let expected = if i == 0 {
                    &note1
                } else {
                    &note2
                };
                let found = repo.find_by_id(*id).unwrap();
                assert!(found.is_some());
                assert_eq!(found.unwrap().path(), expected.path());
            }
        }

        #[test]
        fn rejects_duplicate_path() {
            let repo = InMemoryRepository::new();
            let note1 =
                Note::new(NoteId::new(), NotePath::try_new("dup.md").unwrap());
            let note2 =
                Note::new(NoteId::new(), NotePath::try_new("dup.md").unwrap());

            repo.save(&note1).unwrap();
            let result = repo.save(&note2);

            assert!(matches!(
                result,
                Err(NoteRepositoryError::DuplicatePath(_))
            ));
        }

        #[test]
        fn allows_update_with_same_id_and_path() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("stable.md").unwrap(),
            );
            let id = repo.save(&note).unwrap();
            let updated =
                Note::new(id, NotePath::try_new("stable.md").unwrap());

            let id2 = repo.save(&updated).unwrap();

            assert_eq!(id, id2);
            let found = repo.find_by_id(id).unwrap();
            assert!(found.is_some());
        }

        #[test]
        fn increments_write_counter() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("counter.md").unwrap(),
            );

            repo.save(&note).unwrap();

            let snapshot = repo.harness().counters().snapshot();
            assert!(snapshot.writes >= 1);
        }

        #[test]
        fn returns_storage_error_when_before_write_injected() {
            let harness = InMemoryHarness::with_injector(Box::new(FailOnWrite));
            let repo = InMemoryRepository::with_harness(harness);
            let note =
                Note::new(NoteId::new(), NotePath::try_new("fail.md").unwrap());

            let result = repo.save(&note);

            assert!(matches!(result, Err(NoteRepositoryError::Storage(_))));
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn removes_note_and_path_mapping() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("delete-me.md").unwrap(),
            );
            let path = note.path().clone();
            let id = repo.save(&note).unwrap();

            repo.delete(id).unwrap();

            let by_id = repo.find_by_id(id).unwrap();
            let by_path = repo.find_by_path(&path).unwrap();
            assert!(by_id.is_none());
            assert!(by_path.is_none());
        }

        #[test]
        fn is_idempotent_for_missing_id() {
            let repo = InMemoryRepository::new();
            let missing = NoteId::new();

            repo.delete(missing).unwrap();
            repo.delete(missing).unwrap();
        }

        #[test]
        fn removes_all_notes_in_batch() {
            let repo = InMemoryRepository::new();
            let note1 = Note::new(
                NoteId::new(),
                NotePath::try_new("batch-del-a.md").unwrap(),
            );
            let note2 = Note::new(
                NoteId::new(),
                NotePath::try_new("batch-del-b.md").unwrap(),
            );
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

    mod caching {
        use super::*;

        #[test]
        fn persists_and_retrieves_view() {
            let repo = InMemoryRepository::new();
            let note =
                Note::new(NoteId::new(), NotePath::try_new("view.md").unwrap());
            let id = repo.save(&note).unwrap();
            let view = ListView::from_note_items(id, note.list_items());

            repo.save_list_view(&view).unwrap();

            let found = repo.find_list_view(id).unwrap();
            assert!(found.is_some());
            assert_eq!(found.expect("cached view").note_id(), id);
        }

        #[test]
        fn removes_cached_view() {
            let repo = InMemoryRepository::new();
            let note = Note::new(
                NoteId::new(),
                NotePath::try_new("del-view.md").unwrap(),
            );
            let id = repo.save(&note).unwrap();
            let view = ListView::from_note_items(id, note.list_items());
            repo.save_list_view(&view).unwrap();

            repo.delete_list_view(id).unwrap();

            let found = repo.find_list_view(id).unwrap();
            assert!(found.is_none());
        }
    }
}
