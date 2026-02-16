//! Note ingestion service.
//!
//! This service orchestrates the file → raw → domain → database pipeline for
//! markdown notes.

use std::{io, path::Path};

use tracing::instrument;

use crate::{
    application::error::IngestionError,
    config::aggregate::Config,
    fs::source::FileSource,
    note::{
        aggregate::{Note, NoteId},
        command::Command,
        parser::NoteParser,
        ports as note_ports,
        query::Query,
    },
};

/// Note file ingestion service.
///
/// This service handles reading markdown note files, parsing them into
/// domain aggregates, and persisting them to the database.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific service name is intentional"
)]
pub struct NoteIngestionService<'svc, Q, C> {
    /// Query port for reading existing notes.
    #[expect(
        dead_code,
        reason = "Reserved for future incremental ingestion features (mtime \
                  tracking)"
    )]
    query: &'svc Query<Q>,
    /// Command port for persisting notes.
    command: &'svc Command<C>,
    /// Configuration for note parsing.
    config: &'svc Config,
}

impl<'svc, Q, C> NoteIngestionService<'svc, Q, C>
where
    Q: note_ports::Query,
    C: note_ports::Command,
{
    /// Creates a new note ingestion service.
    #[inline]
    #[must_use]
    pub const fn new(
        query: &'svc Query<Q>,
        command: &'svc Command<C>,
        config: &'svc Config,
    ) -> Self {
        Self {
            query,
            command,
            config,
        }
    }

    /// Ingests a single note file.
    ///
    /// This method reads the file, parses it using `NoteParser`, builds a
    /// `Note` aggregate, and persists it to the database.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (I/O error)
    /// - The note fails domain validation (validation error)
    /// - Database persistence fails (command error)
    #[inline]
    #[instrument(skip(self, source), fields(path = %path.display()))]
    pub fn ingest_file<S>(
        &self,
        source: &S,
        path: &Path,
    ) -> Result<NoteId, IngestionError>
    where
        S: FileSource<Error = io::Error>,
        C::Error: Into<crate::db::DbError>,
    {
        // Step 1: Read file (FileSource → String)
        let content = source.read_to_string(path).map_err(|e| {
            IngestionError::Parse(crate::fs::ParseError::Io {
                path: path.into(),
                source: e,
            })
        })?;

        // Step 2: Parse (String → Components)
        let parser = NoteParser::new(self.config);
        let path_str = path.to_str().ok_or_else(|| {
            IngestionError::Validation("Path contains invalid UTF-8".to_owned())
        })?;

        let (lists, tasks, headings, links, frontmatter) = parser
            .parse_all(&content)
            .map_err(|e| IngestionError::Validation(e.to_string()))?;

        // Step 3: Build Aggregate
        let id = NoteId::new();
        let mut note = Note::new(id, path_str)
            .map_err(|e| IngestionError::Validation(e.to_string()))?;

        note.set_frontmatter(frontmatter);
        for list in lists {
            note.add_list(list);
        }
        for task in tasks {
            note.add_task(task);
        }
        for heading in headings {
            note.add_heading(heading);
        }
        for link in links {
            note.add_link(link);
        }

        // Step 4: Persist (Note → Database)
        self.command
            .create(note.path().as_str())
            .map_err(|e| IngestionError::NoteCommand(e.to_string()))?;

        tracing::info!(
            note_id = ?id,
            path = %path.display(),
            "Note ingested successfully"
        );

        Ok(id)
    }

    /// Ingests all note files matching a glob pattern.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The glob pattern is invalid
    /// - Directory traversal fails
    #[inline]
    #[instrument(skip(self, source))]
    pub fn ingest_directory<S>(
        &self,
        source: &S,
        pattern: &str,
    ) -> Result<Vec<NoteId>, IngestionError>
    where
        S: FileSource<Error = io::Error>,
        C::Error: Into<crate::db::DbError>,
    {
        let paths = source.list_files(pattern).map_err(|e| {
            IngestionError::Parse(crate::fs::ParseError::Io {
                path: Path::new(pattern).into(),
                source: e,
            })
        })?;

        let mut ids = Vec::new();

        for path in paths {
            match self.ingest_file(source, &path) {
                Ok(id) => {
                    tracing::info!(
                        note_id = ?id,
                        path = %path.display(),
                        "Note ingested from directory scan"
                    );
                    ids.push(id);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        path = %path.display(),
                        "Failed to ingest note file (continuing with others)"
                    );
                }
            }
        }

        Ok(ids)
    }

    /// Checks if a note file needs to be re-ingested.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(true)`. Future implementations may return
    /// an error if filesystem metadata cannot be read.
    #[inline]
    #[instrument(skip(self, source))]
    pub fn needs_update<S>(
        &self,
        source: &S,
        path: &Path,
        id: NoteId,
    ) -> Result<bool, IngestionError>
    where
        S: FileSource<Error = io::Error>,
    {
        let _: (&S, NoteId) = (source, id);
        Ok(true)
    }
}
