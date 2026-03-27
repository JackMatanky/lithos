//! Note loader — orchestrates parsing and persistence of note facts.
//!
//! This module provides the [`Loader`] service, which coordinates the full
//! pipeline from raw content to persisted domain facts. It acts as a
//! higher-level facade over the parsing and repository layers.

use crate::{
    config::aggregate::Config,
    note::{
        aggregate::{Note, NoteId},
        error::{NoteLoadError, NoteRepositoryError},
        parser,
        paths::NotePath,
        raw::RawNote,
        storage::Repository,
    },
};

/// Thin orchestration service for note parsing and persistence.
///
/// The `Loader` handles:
/// 1. Parsing markdown source into raw artifacts via
///    [`MarkdownParser`][crate::note::parser::MarkdownParser].
/// 2. Resolving existing note IDs by path from the [`Repository`].
/// 3. Normalizing raw artifacts into domain facts ([`Note`]).
/// 4. Saving the finalized facts back to storage.
pub struct Loader<'repo, 'config, R>
where
    R: Repository<Error = NoteRepositoryError>,
{
    repository: &'repo R,
    config: &'config Config,
}

impl<'repo, 'config, R> Loader<'repo, 'config, R>
where
    R: Repository<Error = NoteRepositoryError>,
{
    /// Create a new `Loader` using the provided repository and configuration.
    #[inline]
    #[must_use]
    pub const fn new(repository: &'repo R, config: &'config Config) -> Self {
        Self {
            repository,
            config,
        }
    }

    /// Parses markdown content and persists the resulting projections.
    ///
    /// This method performs the complete end-to-end transformation from
    /// a string buffer to stored note facts.
    ///
    /// Returns the stable [`NoteId`] assigned or found for the path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteLoadError`] if:
    /// - Markdown parsing fails.
    /// - Domain normalization fails.
    /// - Database persistence fails.
    #[inline]
    pub fn load_content(
        &self,
        path: &NotePath,
        markdown: &str,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> Result<NoteId, NoteLoadError> {
        let task_spec = std::sync::Arc::new(self.config.to_task_spec());
        let raw_note = parser::MarkdownParser::parse(
            markdown,
            path.clone(),
            created_at,
            modified_at,
            &task_spec,
        )?;
        self.load_raw(raw_note)
    }

    /// Persists note projections starting from a pre-parsed raw note.
    ///
    /// This method is useful when the ingestion phase has already
    /// completed (e.g., during bulk ingestion).
    ///
    /// # Errors
    ///
    /// Returns [`NoteLoadError`] if storage or domain conversion fails.
    #[inline]
    pub fn load_raw(
        &self,
        raw_note: RawNote<'_>,
    ) -> Result<NoteId, NoteLoadError> {
        let note_id = self
            .repository
            .find_by_path(&raw_note.path)?
            .map_or_else(NoteId::new, |note| note.id());
        let frontmatter_spec = self.config.to_frontmatter_spec();
        let task_spec = self.config.to_task_spec();
        let facts =
            Note::try_from((raw_note, note_id, &frontmatter_spec, &task_spec))?;
        let saved_id = self.repository.save(&facts)?;
        Ok(saved_id)
    }

    /// Returns a reference to the repository used by this loader.
    #[inline]
    #[must_use]
    pub const fn repository(&self) -> &'repo R {
        self.repository
    }

    /// Records the deletion of a note from storage.
    ///
    /// This removes all structural facts associated with the given ID.
    ///
    /// # Errors
    ///
    /// Returns [`NoteLoadError`] if the database operation fails.
    #[inline]
    pub fn record_deleted_note(&self, id: NoteId) -> Result<(), NoteLoadError> {
        self.repository.delete_note(id)?;
        Ok(())
    }
}
