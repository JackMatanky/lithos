//! Unified note repository backed by redb.
//!
//! Combines read and write operations for the note context in a single
//! repository interface, following the File → Raw → Domain → Storage pipeline.

use std::borrow::Cow;

use uuid::Uuid;

use crate::{
    db::{BatchReader, BatchWriter, Database},
    note::{
        LIST_VIEWS_BY_NOTE_ID, NOTE_ID_BY_PATH, NOTES_BY_ID,
        aggregate::{Note, NoteId},
        error::NoteRepositoryError,
        paths::NotePath,
        views::ListView,
    },
};

/// Unified repository trait for note storage and queries.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Repository trait groups operations by usage semantics"
)]
#[allow(
    clippy::missing_errors_doc,
    reason = "Legacy code being replaced - not worth documenting errors"
)]
pub trait Repository: Send + Sync {
    /// Batch reader type for grouped read operations.
    type BatchReader<'reader>;

    /// Batch writer type for grouped write operations.
    type BatchWriter<'writer>;

    /// Storage error type for repository operations.
    type Error: From<NoteRepositoryError> + std::error::Error;

    /// Archived note type for zero-copy reads.
    type NoteArchived<'archived>;

    /// Deletes a note projection by id.
    ///
    /// # Errors
    /// Returns a repository error if deletion fails.
    fn delete_note(&self, id: NoteId) -> Result<(), Self::Error>;

    /// Finds a stored note projection by its unique UUID v7 identifier.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, Self::Error>;

    /// Finds a stored note projection by its vault-relative path.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, Self::Error>;

    /// Lists all stored note projections currently managed in the vault.
    ///
    /// # Errors
    /// Returns a repository error if the scan fails.
    fn list(&self) -> Result<Vec<Note>, Self::Error>;

    /// Persists a note.
    ///
    /// # Errors
    /// Returns a repository error if persistence fails.
    fn save(&self, note: &Note) -> Result<NoteId, Self::Error>;

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;

    /// Accesses a note by path as archived data, enabling zero-copy reads.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    fn with_archived_by_path<F, R>(
        &self,
        path: &NotePath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;

    /// Executes many read operations within a single transaction.
    ///
    /// # Errors
    /// Returns a repository error if the transaction fails.
    fn with_batch_read<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: for<'reader> FnOnce(
            Self::BatchReader<'reader>,
        ) -> Result<R, Self::Error>;

    /// Retrieves a cached `ListView` for a note.
    /// Returns error if the note doesn't exist or database access fails.
    fn get_list_view(&self, note_id: NoteId) -> Result<ListView, Self::Error>;

    /// Caches a `ListView` projection in the database.
    ///
    /// # Errors
    /// Returns error if serialization or database write fails.
    fn cache_list_view(&self, view: &ListView) -> Result<(), Self::Error>;

    /// Removes a cached `ListView` from the database.
    ///
    /// # Errors
    /// Returns error if database access fails.
    fn invalidate_list_view(&self, note_id: NoteId) -> Result<(), Self::Error>;

    /// Executes many write operations within a single transaction.
    ///
    /// # Errors
    /// Returns a repository error if the transaction fails.
    fn with_batch_write<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: for<'writer> FnOnce(
            &mut Self::BatchWriter<'writer>,
        ) -> Result<(), Self::Error>;
}

/// Read-only batch adapter for note storage.
pub struct RedbBatchNoteReader<'reader> {
    reader: &'reader BatchReader,
}

impl<'reader> RedbBatchNoteReader<'reader> {
    #[inline]
    const fn new(reader: &'reader BatchReader) -> Self {
        Self {
            reader,
        }
    }

    /// Returns the note ID for a vault-relative path.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn get_note_id_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteId>, NoteRepositoryError> {
        self.reader
            .get_owned::<NoteId>(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)
    }

    /// Accesses an archived note by its ID.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn with_note_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, NoteRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Note>) -> R,
    {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.reader
            .get::<Note, _, R>(NOTES_BY_ID, id_str, f)
            .map_err(NoteRepositoryError::Storage)
    }

    /// Accesses an archived note by its vault-relative path.
    ///
    /// # Errors
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn with_note_by_path<F, R>(
        &self,
        path: &NotePath,
        f: F,
    ) -> Result<Option<R>, NoteRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Note>) -> R,
    {
        let id = self.get_note_id_by_path(path)?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.with_note_by_id(id, f)
    }
}

/// Write-capable batch adapter for note storage.
pub struct RedbBatchNoteWriter<'writer> {
    writer: &'writer mut BatchWriter,
}

impl<'writer> RedbBatchNoteWriter<'writer> {
    #[inline]
    fn new(writer: &'writer mut BatchWriter) -> Self {
        Self {
            writer,
        }
    }

    /// Persists a note by ID in the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if persistence fails.
    #[inline]
    pub fn put_note(&mut self, note: &Note) -> Result<(), NoteRepositoryError> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(note.id()).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.writer
            .put(NOTES_BY_ID, id_str, note)
            .map_err(NoteRepositoryError::Storage)
    }

    /// Deletes a note by ID in the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if deletion fails.
    #[inline]
    pub fn delete_note_by_id(
        &mut self,
        id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.writer
            .delete(NOTES_BY_ID, id_str)
            .map_err(NoteRepositoryError::Storage)?;
        Ok(())
    }

    /// Persists a `ListView` in the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if persistence fails.
    #[inline]
    pub fn put_list_view(
        &mut self,
        id_str: &str,
        view: &ListView,
    ) -> Result<(), NoteRepositoryError> {
        self.writer
            .put(LIST_VIEWS_BY_NOTE_ID, id_str, view)
            .map_err(NoteRepositoryError::Storage)
    }

    /// Removes a `ListView` from the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if removal fails.
    #[inline]
    pub fn remove_list_view(
        &mut self,
        id_str: &str,
    ) -> Result<(), NoteRepositoryError> {
        self.writer
            .delete(LIST_VIEWS_BY_NOTE_ID, id_str)
            .map_err(NoteRepositoryError::Storage)?;
        Ok(())
    }

    /// Persists the note ID index for a path in the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if persistence fails.
    #[inline]
    pub fn put_note_id_by_path(
        &mut self,
        path: &NotePath,
        id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        self.writer
            .put(NOTE_ID_BY_PATH, path.as_str(), &id)
            .map_err(NoteRepositoryError::Storage)
    }

    /// Deletes the note ID index for a path in the batch transaction.
    ///
    /// # Errors
    /// Returns a repository error if deletion fails.
    #[inline]
    pub fn delete_note_id_by_path(
        &mut self,
        path: &NotePath,
    ) -> Result<(), NoteRepositoryError> {
        self.writer
            .delete(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)?;
        Ok(())
    }
}

/// Redb-backed note repository adapter.
pub struct RedbRepository<'db> {
    db: &'db Database,
}

impl<'db> RedbRepository<'db> {
    /// Create a new repository with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    fn ensure_unique_path(
        &self,
        path: &NotePath,
        current_id: Option<NoteId>,
    ) -> Result<(), NoteRepositoryError> {
        let existing = self
            .db
            .get_owned::<NoteId>(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)?;
        if existing.is_some_and(|id| Some(id) != current_id) {
            return Err(NoteRepositoryError::DuplicatePath(path.clone()));
        }
        Ok(())
    }

    fn find_note_id_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteId>, NoteRepositoryError> {
        self.db
            .get_owned::<NoteId>(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)
    }

    fn insert_indexes(
        writer: &mut RedbBatchNoteWriter<'_>,
        path: &NotePath,
        note_id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        writer.put_note_id_by_path(path, note_id)?;
        Ok(())
    }

    fn remove_indexes(
        writer: &mut RedbBatchNoteWriter<'_>,
        path: &NotePath,
    ) -> Result<(), NoteRepositoryError> {
        writer.delete_note_id_by_path(path)?;
        Ok(())
    }

    fn update_indexes(
        writer: &mut RedbBatchNoteWriter<'_>,
        old_path: Option<&NotePath>,
        new_path: &NotePath,
        note_id: NoteId,
    ) -> Result<(), NoteRepositoryError> {
        if let Some(old) = old_path {
            if old.as_str() != new_path.as_str() {
                writer.delete_note_id_by_path(old)?;
                writer.put_note_id_by_path(new_path, note_id)?;
            }
            Ok(())
        } else {
            Self::insert_indexes(writer, new_path, note_id)
        }
    }
}

impl Repository for RedbRepository<'_> {
    type BatchReader<'reader> = RedbBatchNoteReader<'reader>;
    type BatchWriter<'writer> = RedbBatchNoteWriter<'writer>;
    type Error = NoteRepositoryError;
    type NoteArchived<'archived> = &'archived rkyv::Archived<Note>;

    #[inline]
    fn delete_note(&self, id: NoteId) -> Result<(), Self::Error> {
        let stored = self
            .db
            .get_owned_by_uuid::<Note>(NOTES_BY_ID, *id.as_uuid_v7())
            .map_err(NoteRepositoryError::Storage)?;

        if let Some(stored) = stored {
            let stored_path = stored.path().clone();
            self.with_batch_write(|writer| {
                Self::remove_indexes(writer, &stored_path)?;
                writer.delete_note_by_id(id)?;
                Ok(())
            })?;
        }

        Ok(())
    }

    #[inline]
    fn save(&self, note: &Note) -> Result<NoteId, Self::Error> {
        let path = note.path();
        let existing_id = self.find_note_id_by_path(path)?;
        let note_id = existing_id.unwrap_or_else(NoteId::new);
        let id = Uuid::from(note_id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;

        let stored_note: Cow<'_, Note> = if note_id == note.id() {
            Cow::Borrowed(note)
        } else {
            Cow::Owned(note.clone().with_id(note_id))
        };
        let old_index_data = if existing_id.is_some() {
            let stored = self
                .db
                .get_owned::<Note>(NOTES_BY_ID, id_str)
                .map_err(NoteRepositoryError::Storage)?;
            if let Some(stored) = stored {
                if stored.path() != path {
                    self.ensure_unique_path(path, Some(note_id))?;
                }
                Some(stored.path().clone())
            } else {
                None
            }
        } else {
            self.ensure_unique_path(path, None)?;
            None
        };

        self.with_batch_write(|writer| {
            Self::update_indexes(
                writer,
                old_index_data.as_ref(),
                path,
                note_id,
            )?;
            writer.put_note(stored_note.as_ref())?;
            Ok(())
        })?;

        Ok(note_id)
    }

    #[inline]
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, Self::Error> {
        self.db
            .get_owned_by_uuid::<Note>(NOTES_BY_ID, *id.as_uuid_v7())
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, Self::Error> {
        let id = self
            .db
            .get_owned::<NoteId>(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.db
            .get_owned_by_uuid::<Note>(NOTES_BY_ID, *id.as_uuid_v7())
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn with_archived_by_path<F, R>(
        &self,
        path: &NotePath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        let id = self
            .db
            .get_owned::<NoteId>(NOTE_ID_BY_PATH, path.as_str())
            .map_err(NoteRepositoryError::Storage)?;
        let Some(id) = id else {
            return Ok(None);
        };
        self.db
            .get_by_uuid::<Note, _, R>(NOTES_BY_ID, *id.as_uuid_v7(), f)
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn list(&self) -> Result<Vec<Note>, Self::Error> {
        self.db
            .list_owned::<Note>(NOTES_BY_ID)
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        self.db
            .get_by_uuid::<Note, _, R>(NOTES_BY_ID, *id.as_uuid_v7(), f)
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn with_batch_read<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: for<'reader> FnOnce(
            Self::BatchReader<'reader>,
        ) -> Result<R, Self::Error>,
    {
        self.db
            .batch_read(|reader| {
                let batch = RedbBatchNoteReader::new(reader);
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    clippy::unreachable,
                    reason = "Batch closures only perform db operations; \
                              non-Storage variants indicate programming error"
                )]
                f(batch).map_err(|err| match err {
                    NoteRepositoryError::Storage(db_err) => db_err,
                    other => unreachable!(
                        "batch operation returned non-storage error: {other}"
                    ),
                })
            })
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn with_batch_write<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: for<'writer> FnOnce(
            &mut Self::BatchWriter<'writer>,
        ) -> Result<(), Self::Error>,
    {
        self.db
            .batch_write(|writer| {
                let mut batch = RedbBatchNoteWriter::new(writer);
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    clippy::unreachable,
                    reason = "Batch closures only perform db operations; \
                              non-Storage variants indicate programming error"
                )]
                f(&mut batch).map_err(|err| match err {
                    NoteRepositoryError::Storage(db_err) => db_err,
                    other => unreachable!(
                        "batch operation returned non-storage error: {other}"
                    ),
                })
            })
            .map_err(NoteRepositoryError::Storage)
    }

    #[inline]
    fn get_list_view(&self, note_id: NoteId) -> Result<ListView, Self::Error> {
        let id = Uuid::from(note_id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);

        self.db
            .get_owned::<ListView>(LIST_VIEWS_BY_NOTE_ID, id_str)
            .map_err(NoteRepositoryError::Storage)?
            .ok_or(NoteRepositoryError::NotFoundById(note_id))
    }

    #[inline]
    fn cache_list_view(&self, view: &ListView) -> Result<(), Self::Error> {
        let id = Uuid::from(view.note_id());
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);

        self.with_batch_write(|writer| writer.put_list_view(id_str, view))
    }

    #[inline]
    fn invalidate_list_view(&self, note_id: NoteId) -> Result<(), Self::Error> {
        let id = Uuid::from(note_id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);

        self.with_batch_write(|writer| writer.remove_list_view(id_str))
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions and \
              prioritize readability."
)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            vault::{VaultId, VaultRoot},
        },
        db::DbError,
        note::{
            aggregate::{Note, NoteId},
            paths::NotePath,
            raw::RawNote,
        },
    };

    fn test_config() -> Result<Config, String> {
        crate::config::builder::build_from_layers(
            None,
            None,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .map_err(|e| e.to_string())?,
            crate::config::aggregate::Version::initial(),
        )
        .map_err(|e| e.to_string())
    }

    fn raw_note(_path: NotePath) -> RawNote<'static> {
        RawNote::new(
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn save_persists_path() -> Result<(), NoteRepositoryError> {
        let dir = tempdir().map_err(|err| {
            NoteRepositoryError::Storage(DbError::Open(format!(
                "temp dir: {err}"
            )))
        })?;
        let db_path = dir.path().join("notes.redb");
        let db =
            Database::open(&db_path).map_err(NoteRepositoryError::Storage)?;
        let config = test_config().map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(err))
        })?;
        let repo = RedbRepository::new(&db);

        let path = NotePath::try_new("notes/a.md").map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(
                err.to_string(),
            ))
        })?;
        let raw = raw_note(path.clone());
        let frontmatter_spec = config.to_frontmatter_spec();
        let task_spec = config.to_task_spec();
        let facts = Note::try_from((
            raw,
            "", // Add empty source for tests
            &path,
            NoteId::new(),
            &frontmatter_spec,
            &task_spec,
        ))
        .map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(
                err.to_string(),
            ))
        })?;

        let note_id = repo.save(&facts)?;
        let stored =
            repo.find_by_path(&path)?.expect("stored note should exist");
        assert_eq!(stored.id(), note_id);
        assert_eq!(stored.path().as_str(), "notes/a.md");
        Ok(())
    }

    #[test]
    fn delete_note_removes_note() -> Result<(), NoteRepositoryError> {
        let dir = tempdir().map_err(|err| {
            NoteRepositoryError::Storage(DbError::Open(format!(
                "temp dir: {err}"
            )))
        })?;
        let db_path = dir.path().join("notes.redb");
        let db =
            Database::open(&db_path).map_err(NoteRepositoryError::Storage)?;
        let config = test_config().map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(err))
        })?;
        let repo = RedbRepository::new(&db);

        let path = NotePath::try_new("notes/a.md").map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(
                err.to_string(),
            ))
        })?;
        let raw = raw_note(path.clone());
        let frontmatter_spec = config.to_frontmatter_spec();
        let task_spec = config.to_task_spec();
        let facts = Note::try_from((
            raw,
            "", // Add empty source for tests
            &path,
            NoteId::new(),
            &frontmatter_spec,
            &task_spec,
        ))
        .map_err(|err| {
            NoteRepositoryError::Storage(DbError::Serialization(
                err.to_string(),
            ))
        })?;
        let note_id = repo.save(&facts)?;

        repo.delete_note(note_id)?;
        let stored = repo.find_by_id(note_id)?;
        assert!(stored.is_none());
        Ok(())
    }
}
