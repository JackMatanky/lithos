//! Note loader — parses markdown content and persists projections.

use crate::{
    db::DbError,
    note::{
        db_command::CommandAdapter, error::NoteIngestError, identity::NoteId,
        paths::NotePath, ports::Command as _, reader::NoteReader,
    },
};

/// Errors that can occur during note loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    /// Ingestion (file listing or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] DbError),
}

/// Thin orchestration service for note parsing and persistence.
pub struct Loader<'db, 'config> {
    command: CommandAdapter<'db, 'config>,
    reader: NoteReader<'config>,
}

impl<'db, 'config> Loader<'db, 'config> {
    /// Create a new `Loader` with command adapter.
    #[inline]
    #[must_use]
    pub const fn new(command: CommandAdapter<'db, 'config>) -> Self {
        let reader = NoteReader::new(command.config());
        Self {
            command,
            reader,
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
        markdown: Box<str>,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> Result<NoteId, LoadError> {
        let parsed =
            self.reader.parse_content(markdown, created_at, modified_at)?;
        let note_id = self.command.upsert_parsed_note(path, &parsed)?;
        Ok(note_id)
    }

    #[inline]
    #[must_use]
    /// Access the command adapter used by this loader.
    pub const fn command(&self) -> &CommandAdapter<'db, 'config> {
        &self.command
    }

    /// Record deletion of a stored note.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] if persistence fails.
    #[inline]
    pub fn record_deleted_note(&self, id: NoteId) -> Result<(), LoadError> {
        self.command.record_deleted_note(id)?;
        Ok(())
    }
}
