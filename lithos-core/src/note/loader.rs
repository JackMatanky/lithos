//! Note loader — parses markdown content and persists note facts.

use crate::{
    config::aggregate::Config,
    db::DbError,
    note::{
        aggregate::{NoteFacts, NoteId, RawNoteContext},
        error::{NoteError, NoteIngestError},
        parser,
        paths::NotePath,
        raw::note::RawNote,
        storage::Repository,
    },
};

/// Errors that can occur during note loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    /// Ingestion (file listing or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// Domain conversion failed.
    #[error("domain error: {0}")]
    Domain(#[from] NoteError),

    /// Storage command failed.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),
}

/// Thin orchestration service for note parsing and persistence.
pub struct Loader<'repo, 'config, R>
where
    R: Repository<Error = DbError>,
{
    repository: &'repo R,
    config: &'config Config,
}

impl<'repo, 'config, R> Loader<'repo, 'config, R>
where
    R: Repository<Error = DbError>,
{
    /// Create a new `Loader` with repository and config.
    #[inline]
    #[must_use]
    pub const fn new(repository: &'repo R, config: &'config Config) -> Self {
        Self {
            repository,
            config,
        }
    }

    /// Parse markdown content and persist projections.
    ///
    /// Returns the note ID that was inserted/updated.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] on parsing or storage failure.
    #[inline]
    pub fn load_content(
        &self,
        path: &NotePath,
        markdown: &str,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> Result<NoteId, LoadError> {
        let raw_note = parser::extract::extract_markdown(
            markdown,
            path.clone(),
            created_at,
            modified_at,
        )?;
        self.load_raw(&raw_note)
    }

    /// Persist projections from a raw note.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] on storage or domain conversion failure.
    #[inline]
    pub fn load_raw(&self, raw_note: &RawNote) -> Result<NoteId, LoadError> {
        let note_id = self
            .repository
            .find_by_path(raw_note.path())?
            .map_or_else(NoteId::new, |note| note.id());
        let facts = NoteFacts::try_from(RawNoteContext::new(
            note_id,
            raw_note,
            self.config,
        ))?;
        let saved_id = self.repository.save_note_facts(&facts)?;
        Ok(saved_id)
    }

    #[inline]
    #[must_use]
    /// Access the repository used by this loader.
    pub const fn repository(&self) -> &'repo R {
        self.repository
    }

    /// Record deletion of a stored note.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] if persistence fails.
    #[inline]
    pub fn record_deleted_note(&self, id: NoteId) -> Result<(), LoadError> {
        self.repository.delete_note(id)?;
        Ok(())
    }
}
