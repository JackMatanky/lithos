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

use std::{collections::HashSet, path::Path};

use crate::{
    db::DbError,
    fs::FsReader,
    note::{
        adapter::{
            command::CommandAdapter, ingestor::Ingestor, reader::NoteReader,
        },
        aggregate::NoteId,
        error::NoteIngestError,
        ports::Command as _,
    },
};

/// Errors that can occur during note service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NoteServiceError {
    /// Ingestion (file listing or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] NoteIngestError),

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
        let paths =
            ingestor.scan_note_paths().map_err(NoteIngestError::Domain)?;

        let mut path_set: HashSet<Box<str>> =
            HashSet::with_capacity(paths.len());
        for note_path in &paths {
            path_set.insert(note_path.as_str().into());
        }

        let stored_notes = self.command.list_stored_notes()?;
        for stored in stored_notes {
            if !path_set.contains(stored.path().as_str()) {
                self.command.record_deleted_note(stored.id())?;
            }
        }

        let mut note_ids = Vec::with_capacity(paths.len());
        for note_path in paths {
            let stored = self.command.stored_note_by_path(&note_path)?;
            let metadata = fs.metadata(Path::new(note_path.as_str())).map_err(
                |error| {
                    NoteServiceError::Ingestion(NoteIngestError::Source(
                        error.to_string().into(),
                    ))
                },
            )?;
            let modified = metadata.modified().ok();
            let created = metadata.created().ok();
            let size = metadata.len();

            if let Some(stored) = stored.as_ref() {
                let is_same_size = stored.source_bytes() == size;
                let is_same_mtime =
                    stored.modified_at().zip(modified).is_some_and(
                        |(stored_time, current)| stored_time == current,
                    );
                if is_same_size && is_same_mtime {
                    continue;
                }

                let markdown = fs
                    .read_to_string(Path::new(note_path.as_str()))
                    .map_err(|error| {
                        NoteServiceError::Ingestion(NoteIngestError::Source(
                            error.to_string().into(),
                        ))
                    })?;
                let hash =
                    blake3::hash(markdown.as_bytes()).to_hex().to_string();
                if is_same_size && stored.source_hash() == hash {
                    continue;
                }
                let parsed = reader.parse_content(
                    markdown.into_boxed_str(),
                    created,
                    modified,
                )?;
                let note_id =
                    self.command.upsert_parsed_note(&note_path, &parsed)?;
                note_ids.push(note_id);
            } else {
                let parsed =
                    reader.parse(&fs, Path::new(note_path.as_str()))?;
                let note_id =
                    self.command.upsert_parsed_note(&note_path, &parsed)?;
                note_ids.push(note_id);
            }
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
