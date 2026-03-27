//! Ingestor adapter for loading raw notes from the filesystem.
//!
//! This module provides the bridge between the physical filesystem and the
//! logical parsing pipeline. It is responsible for reading content,
//! extracting filesystem metadata (timestamps), and delegating to the
//! markdown parser.
//!
//! The [`Ingestor`] is a pure file-to-raw translation layer. It does not
//! interact with the database or assign stable identifiers.

use std::{path::Path, sync::Arc, time::SystemTime};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{
        error::{NoteFileError, NoteIngestError},
        paths::NotePath,
        raw::RawNote,
    },
};

/// Ingestor for loading raw markdown notes from a file source.
///
/// This adapter orchestrates the low-level ingestion of a markdown file:
/// 1. Reads raw bytes via [`FsReader`].
/// 2. Extracts filesystem `created_at` and `modified_at` timestamps.
/// 3. Initiates the [`MarkdownParser`][crate::note::parser::MarkdownParser]
///    pipeline.
///
/// The ingestor holds a reference to the active [`Config`] to ensure
/// ingestion rules (like vault root resolution) are consistently applied.
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
    ///
    /// This is the standard way to initialize an ingestor for a configured
    /// vault.
    #[inline]
    #[must_use]
    pub fn from_config(config: &'config Config) -> Self {
        let root = config.vault_metadata().root().as_path();
        Self::new(FsReader::new(root), config)
    }

    /// Returns a reference to the underlying filesystem reader.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &FsReader {
        &self.source
    }

    /// Load and parse a note from a vault-relative path.
    ///
    /// This method uses the internal `FsReader` to load the file content
    /// and then runs it through the ingestion pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if:
    /// - The file cannot be read from the source.
    /// - The content fails to parse as valid markdown facts.
    #[inline]
    pub fn ingest_path(
        &self,
        path: &NotePath,
    ) -> Result<RawNote<'static>, NoteIngestError> {
        let relative = Path::new(path.as_str());
        let created_at = self.source.created_at(relative);
        let modified_at = self.source.modified_at(relative);
        let task_spec = Arc::new(self.config.to_task_spec());
        self.source
            .read_with(relative, |_path, markdown| {
                let task_spec = Arc::clone(&task_spec);
                super::parser::MarkdownParser::parse(
                    markdown,
                    path.clone(),
                    created_at,
                    modified_at,
                    &task_spec,
                )
                .map(RawNote::into_owned)
            })
            .map_err(|e: NoteIngestError| {
                NoteFileError::ReadFailed {
                    path: path.clone(),
                    message: e.to_string().into(),
                }
                .into()
            })
    }

    /// Load and parse a note from provided markdown content.
    ///
    /// Use this method when the markdown content is already in memory
    /// (e.g., from an LSP buffer) but needs to be processed as if it
    /// came from a specific path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if the markdown cannot be parsed.
    #[inline]
    pub fn ingest_markdown<'markdown>(
        &self,
        path: &NotePath,
        markdown: &'markdown str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<RawNote<'markdown>, NoteIngestError> {
        let task_spec = Arc::new(self.config.to_task_spec());
        super::parser::MarkdownParser::parse(
            markdown,
            path.clone(),
            created_at,
            modified_at,
            &task_spec,
        )
    }

    /// Returns the configuration used by this ingestor.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &'config Config {
        self.config
    }
}
