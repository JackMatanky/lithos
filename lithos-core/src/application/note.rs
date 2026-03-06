//! Note application service — orchestrates the note ingestion pipeline.
//!
//! Pipeline:
//! 1. Discover note paths via `note::adapter::ingestor::Ingestor`.
//! 2. Parse markdown via `note::adapter::reader::NoteReader`.
//! 3. Persist projections via `note::adapter::command::CommandAdapter`.

#![allow(
    clippy::module_name_repetitions,
    reason = "Namespaced types in application layer"
)]

use std::path::Path;

use crate::{
    db::DbError,
    fs::FsReader,
    note::{
        adapter::{
            command::CommandAdapter, ingestor::Ingestor, reader::NoteReader,
        },
        aggregate::NoteId,
        error::NoteError,
    },
};

/// Errors that can occur during note service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NoteServiceError {
    /// Ingestion (file listing or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] NoteError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] DbError),
}

/// Thin orchestration service for note ingestion.
///
/// Uses concrete redb adapters for production use.
pub struct NoteService<'db, 'config> {
    command: CommandAdapter<'db, 'config>,
}

impl<'db, 'config> NoteService<'db, 'config> {
    /// Create a new `NoteService` with command adapter.
    #[inline]
    #[must_use]
    pub const fn new(command: CommandAdapter<'db, 'config>) -> Self {
        Self {
            command,
        }
    }

    /// Run the note ingestion pipeline.
    ///
    /// Returns the note IDs that were inserted/updated.
    ///
    /// # Errors
    ///
    /// Returns [`NoteServiceError`] on I/O, parsing, or storage failure.
    #[inline]
    pub fn load(
        &self,
        ingestor: &Ingestor<'config>,
    ) -> Result<Vec<NoteId>, NoteServiceError> {
        let config = ingestor.config();
        let reader = NoteReader::new(config);
        let fs = FsReader::new(config.vault_metadata().root().as_path());
        let paths = ingestor.scan_note_paths()?;

        let mut note_ids = Vec::with_capacity(paths.len());
        for note_path in paths {
            let parsed = reader.parse(&fs, Path::new(note_path.as_str()))?;
            let note_id =
                self.command.upsert_parsed_note(&note_path, &parsed)?;
            note_ids.push(note_id);
        }

        Ok(note_ids)
    }

    #[inline]
    #[must_use]
    /// Access the command adapter used by this service.
    pub const fn command(&self) -> &CommandAdapter<'db, 'config> {
        &self.command
    }
}
