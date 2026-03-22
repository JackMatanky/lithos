//! Vault service for file discovery and ingestion orchestration.
//!
//! This service owns vault-wide file discovery and delegates markdown parsing
//! to the Note context when it encounters note files.

use std::{collections::HashSet, path::Path};

use super::staleness::StalenessPolicy;
use crate::{
    config::aggregate::Config,
    db::{Database, DbError},
    fs::FsReader,
    note::{
        error::{
            NoteFileError, NoteIngestError, NoteLoadError, NoteRepositoryError,
        },
        ingestor::Ingestor,
        loader::Loader as NoteLoader,
        paths::NotePath,
        storage::{RedbRepository, Repository as _},
    },
};

/// Errors surfaced during vault loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "ServiceError is standard for application services"
)]
pub enum ServiceError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion failed: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// Repository operation failed.
    #[error("repository failure: {0}")]
    Repository(#[from] NoteRepositoryError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] DbError),
}

impl From<NoteFileError> for ServiceError {
    #[inline]
    fn from(err: NoteFileError) -> Self {
        ServiceError::Ingestion(NoteIngestError::File(err))
    }
}

/// Vault-level service for loading file-based content.
///
/// Currently orchestrates markdown note ingestion.
pub struct Service<'db, 'config> {
    db: &'db Database,
    config: &'config Config,
}

impl<'db, 'config> Service<'db, 'config> {
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
    /// Returns [`ServiceError`] on I/O, parsing, or storage failure.
    #[inline]
    pub fn load(
        &self,
    ) -> Result<Vec<crate::note::aggregate::NoteId>, ServiceError> {
        let source =
            FsReader::new(self.config.vault_metadata().root().as_path());
        let ingestor = Ingestor::new(source, self.config);
        let paths = Self::scan_note_paths(ingestor.source())?;

        let repository = RedbRepository::new(self.db, self.config);
        let loader = NoteLoader::new(&repository, self.config);
        let policy = StalenessPolicy::new();

        let mut path_set: HashSet<Box<str>> =
            HashSet::with_capacity(paths.len());
        for note_path in &paths {
            path_set.insert(note_path.as_str().into());
        }

        let stored_notes =
            loader.repository().list().map_err(ServiceError::Repository)?;
        for stored in stored_notes {
            if !path_set.contains(stored.path().as_str()) {
                loader
                    .record_deleted_note(stored.id())
                    .map_err(map_load_error)?;
            }
        }

        let mut note_ids = Vec::with_capacity(paths.len());
        for note_path in paths {
            let stored = loader
                .repository()
                .find_by_path(&note_path)
                .map_err(ServiceError::Repository)?;
            let metadata = ingestor
                .source()
                .metadata(Path::new(note_path.as_str()))
                .map_err(|error| NoteFileError::MetadataFailed {
                    path: note_path.clone(),
                    message: error.to_string().into(),
                })?;
            let modified = metadata.modified().ok();
            let created = metadata.created().ok();
            let size = metadata.len();

            if let Some(stored) = stored.as_ref() {
                if policy.is_metadata_fresh(stored, size, modified) {
                    continue;
                }

                let markdown = ingestor
                    .source()
                    .read_to_string(Path::new(note_path.as_str()))
                    .map_err(|error| NoteFileError::ReadFailed {
                        path: note_path.clone(),
                        message: error.to_string().into(),
                    })?;
                let hash =
                    blake3::hash(markdown.as_bytes()).to_hex().to_string();

                if policy.is_content_fresh(stored, size, &hash) {
                    continue;
                }

                let raw_note = ingestor
                    .ingest_markdown(
                        &note_path,
                        markdown.as_str(),
                        created,
                        modified,
                    )
                    .map_err(ServiceError::from)?;
                let note_id =
                    loader.load_raw(raw_note).map_err(map_load_error)?;
                note_ids.push(note_id);
            } else {
                let raw_note = ingestor
                    .ingest_path(&note_path)
                    .map_err(ServiceError::from)?;
                let note_id =
                    loader.load_raw(raw_note).map_err(map_load_error)?;
                note_ids.push(note_id);
            }
        }

        Ok(note_ids)
    }

    fn scan_note_paths(fs: &FsReader) -> Result<Vec<NotePath>, ServiceError> {
        let pattern = "**/*";
        let files = fs.list_files(pattern).map_err(|error| {
            #[expect(
                clippy::unwrap_used,
                reason = "Static dummy path is valid"
            )]
            let dummy_path = NotePath::try_new("vault.md").unwrap();
            NoteFileError::ReadFailed {
                path: dummy_path,
                message: error.to_string().into(),
            }
        })?;

        let mut notes = Vec::with_capacity(files.len());
        for path in files {
            if !crate::fs::types::Markdown::is_supported(&path) {
                continue;
            }
            if let Err(_error) = fs.validate_path(&path) {
                return Err(ServiceError::Ingestion(
                    NoteFileError::InvalidPath {
                        path: path.to_string_lossy().into(),
                        reason: "invalid path",
                    }
                    .into(),
                ));
            }
            let path_str = path.to_str().ok_or_else(|| {
                NoteIngestError::from(NoteFileError::InvalidPath {
                    path: path.to_string_lossy().into(),
                    reason: "invalid UTF-8 in note path",
                })
            })?;
            let note_path =
                NotePath::try_new(path_str).map_err(NoteIngestError::Domain)?;
            notes.push(note_path);
        }

        Ok(notes)
    }
}

fn map_load_error(error: NoteLoadError) -> ServiceError {
    match error {
        NoteLoadError::Ingestion(error) => ServiceError::Ingestion(error),
        NoteLoadError::Validation(error) => {
            ServiceError::Ingestion(NoteIngestError::Domain(error))
        }
        NoteLoadError::Persistence(error) => ServiceError::Repository(error),
    }
}
