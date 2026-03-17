//! Ingestor adapter for loading raw notes from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.

use std::{path::Path, time::SystemTime};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{error::NoteIngestError, paths::NotePath, raw::note::RawNote},
};

/// Ingestor for loading raw markdown notes from a file source.
///
/// This adapter is responsible for:
/// - Reading markdown content from the vault
/// - Computing the content hash
/// - Extracting filesystem timestamps
/// - Delegating to the parser ingestion pipeline
///
/// It does NOT:
/// - Assign IDs
/// - Query the database
/// - Perform domain validation
///
/// The ingestor stores a `&Config` reference to ensure it uses the final
/// merged path values after config loading completes.
pub struct Ingestor<'config> {
    source: FsReader,
    config: &'config Config,
}

impl<'config> Ingestor<'config> {
    /// Create a new ingestor with the given file source and config.
    #[inline]
    #[must_use]
    pub fn new(source: FsReader, config: &'config Config) -> Self {
        Self {
            source,
            config,
        }
    }

    /// Create a new ingestor using the vault root from config.
    #[inline]
    #[must_use]
    pub fn from_config(config: &'config Config) -> Self {
        let root = config.vault_metadata().root().as_path();
        Self::new(FsReader::new(root), config)
    }

    /// Returns the underlying filesystem reader.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &FsReader {
        &self.source
    }

    /// Load and parse a note from a vault-relative path.
    ///
    /// # Errors
    /// Returns [`NoteIngestError`] if the file cannot be read or parsed.
    #[inline]
    pub fn ingest_path(
        &self,
        path: &NotePath,
    ) -> Result<RawNote, NoteIngestError> {
        let relative = Path::new(path.as_str());
        let created_at = self.source.created_at(relative);
        let modified_at = self.source.modified_at(relative);
        self.source.read_with(relative, |_path, markdown| {
            super::parser::extract::ingest_markdown(
                markdown,
                path.clone(),
                created_at,
                modified_at,
            )
        })
    }

    /// Load and parse a note from provided markdown content.
    ///
    /// # Errors
    /// Returns [`NoteIngestError`] if the markdown cannot be parsed.
    #[inline]
    pub fn ingest_markdown(
        &self,
        path: &NotePath,
        markdown: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<RawNote, NoteIngestError> {
        super::parser::extract::ingest_markdown(
            markdown,
            path.clone(),
            created_at,
            modified_at,
        )
    }

    /// Returns the config used by this ingestor.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &'config Config {
        self.config
    }
}
