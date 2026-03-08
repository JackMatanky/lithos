//! Vault service for file discovery and ingestion orchestration.
//!
//! This service owns vault-wide file discovery and delegates markdown parsing
//! to the Note context when it encounters note files.

use std::{collections::HashSet, path::Path};

use crate::{
    config::aggregate::Config,
    db::{Database, DbError},
    fs::FsReader,
    note::{
        db_command::CommandAdapter,
        error::NoteIngestError,
        loader::{LoadError, Loader as NoteLoader},
        paths::NotePath,
    },
};

/// Errors surfaced during vault loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VaultError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] DbError),
}

/// Vault-level service for loading file-based content.
///
/// Currently orchestrates markdown note ingestion.
pub struct VaultService<'db, 'config> {
    db: &'db Database,
    config: &'config Config,
}

impl<'db, 'config> VaultService<'db, 'config> {
    /// Create a new vault service with database and config.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database, config: &'config Config) -> Self {
        Self {
            db,
            config,
        }
    }

    /// Load vault content and persist projections.
    ///
    /// Returns the note IDs that were inserted or updated.
    ///
    /// # Errors
    ///
    /// Returns [`VaultError`] on I/O, parsing, or storage failure.
    #[inline]
    pub fn load(
        &self,
    ) -> Result<Vec<crate::note::identity::NoteId>, VaultError> {
        let fs = FsReader::new(self.config.vault_metadata().root().as_path());
        let paths = Self::scan_note_paths(&fs)?;

        let command = CommandAdapter::new(self.db, self.config);
        let loader = NoteLoader::new(command);

        let mut path_set: HashSet<Box<str>> =
            HashSet::with_capacity(paths.len());
        for note_path in &paths {
            path_set.insert(note_path.as_str().into());
        }

        let stored_notes = loader.command().list_stored_notes()?;
        for stored in stored_notes {
            if !path_set.contains(stored.path().as_str()) {
                loader
                    .record_deleted_note(stored.id())
                    .map_err(map_load_error)?;
            }
        }

        let mut note_ids = Vec::with_capacity(paths.len());
        for note_path in paths {
            let stored = loader.command().stored_note_by_path(&note_path)?;
            let metadata = fs.metadata(Path::new(note_path.as_str())).map_err(
                |error| NoteIngestError::Source(error.to_string().into()),
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
                        NoteIngestError::Source(error.to_string().into())
                    })?;
                let hash =
                    blake3::hash(markdown.as_bytes()).to_hex().to_string();
                if is_same_size && stored.source_hash() == hash {
                    continue;
                }

                let note_id = loader
                    .load_content(
                        &note_path,
                        markdown.into_boxed_str(),
                        created,
                        modified,
                    )
                    .map_err(map_load_error)?;
                note_ids.push(note_id);
            } else {
                let markdown = fs
                    .read_to_string(Path::new(note_path.as_str()))
                    .map_err(|error| {
                        NoteIngestError::Source(error.to_string().into())
                    })?;
                let note_id = loader
                    .load_content(
                        &note_path,
                        markdown.into_boxed_str(),
                        created,
                        modified,
                    )
                    .map_err(map_load_error)?;
                note_ids.push(note_id);
            }
        }

        Ok(note_ids)
    }

    fn scan_note_paths(fs: &FsReader) -> Result<Vec<NotePath>, VaultError> {
        let pattern = "**/*";
        let files = fs.list_files(pattern).map_err(|error| {
            NoteIngestError::Source(error.to_string().into())
        })?;

        let mut notes = Vec::with_capacity(files.len());
        for path in files {
            if !crate::fs::types::Markdown::is_supported(&path) {
                continue;
            }
            if let Err(error) = fs.validate_path(&path) {
                return Err(VaultError::Ingestion(NoteIngestError::Source(
                    error.to_string().into(),
                )));
            }
            let path_str = path.to_str().ok_or_else(|| {
                NoteIngestError::Source("invalid UTF-8 in note path".into())
            })?;
            let note_path =
                NotePath::try_new(path_str).map_err(NoteIngestError::Domain)?;
            notes.push(note_path);
        }

        Ok(notes)
    }
}

fn map_load_error(error: LoadError) -> VaultError {
    match error {
        LoadError::Ingestion(error) => VaultError::Ingestion(error),
        LoadError::Command(error) => VaultError::Command(error),
    }
}
