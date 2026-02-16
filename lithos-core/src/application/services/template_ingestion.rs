//! Template ingestion service.
//!
//! This service orchestrates the file → raw → domain → database pipeline for
//! template definitions.

use std::{io, path::Path};

use tracing::instrument;

use crate::{
    application::error::IngestionError,
    fs::{parsers::parse_file, source::FileSource},
    template::{
        aggregate::{Template, TemplateName},
        command::Command,
        ports as template_ports,
        query::Query,
        raw::RawTemplate,
    },
};

/// Template file ingestion service.
///
/// This service handles reading template definition files, parsing them into
/// domain aggregates, and persisting them to the database.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific service name is intentional"
)]
pub struct TemplateIngestionService<'svc, Q, C> {
    /// Query port for reading existing templates.
    #[expect(
        dead_code,
        reason = "Reserved for future incremental ingestion features (mtime \
                  tracking)"
    )]
    query: &'svc Query<Q>,
    /// Command port for persisting templates.
    command: &'svc Command<C>,
}

impl<'svc, Q, C> TemplateIngestionService<'svc, Q, C>
where
    Q: template_ports::Query,
    C: template_ports::Command,
{
    /// Creates a new template ingestion service.
    #[inline]
    #[must_use]
    pub const fn new(query: &'svc Query<Q>, command: &'svc Command<C>) -> Self {
        Self {
            query,
            command,
        }
    }

    /// Ingests a single template file.
    ///
    /// This method reads the file, parses it into a `RawTemplate`, validates
    /// it into a `Template` aggregate, and persists it to the database.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (I/O error)
    /// - The file format is invalid (parse error)
    /// - The template fails domain validation (validation error)
    /// - Database persistence fails (command error)
    #[inline]
    #[instrument(skip(self, source), fields(path = %path.display()))]
    pub fn ingest_file<S>(
        &self,
        source: &S,
        path: &Path,
    ) -> Result<TemplateName, IngestionError>
    where
        S: FileSource<Error = io::Error>,
    {
        // Step 1: Parse file (FileSource → RawTemplate)
        let raw: RawTemplate = parse_file(source, path)?;

        // Step 3: Validate (RawTemplate → Template)
        let template = Template::try_from(raw)
            .map_err(|e| IngestionError::Validation(e.to_string()))?;

        // Step 4: Persist (Template → Database)
        let name = template.name().clone();
        self.command
            .create(&template)
            .map_err(|e| IngestionError::TemplateCommand(e.to_string()))?;

        tracing::info!(
            template_name = %name,
            "Template ingested successfully"
        );

        Ok(name)
    }

    /// Ingests all template files matching a glob pattern.
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
    ) -> Result<Vec<TemplateName>, IngestionError>
    where
        S: FileSource<Error = io::Error>,
    {
        let paths = source.list_files(pattern).map_err(|e| {
            IngestionError::Parse(crate::fs::ParseError::Io {
                path: Path::new(pattern).into(),
                source: e,
            })
        })?;

        let mut names = Vec::new();

        for path in paths {
            match self.ingest_file(source, &path) {
                Ok(name) => {
                    tracing::info!(
                        template_name = %name,
                        path = %path.display(),
                        "Template ingested from directory scan"
                    );
                    names.push(name);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        path = %path.display(),
                        "Failed to ingest template file (continuing with \
                         others)"
                    );
                }
            }
        }

        Ok(names)
    }

    /// Checks if a template file needs to be re-ingested.
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
        name: TemplateName,
    ) -> Result<bool, IngestionError>
    where
        S: FileSource<Error = io::Error>,
    {
        let _: (&S, TemplateName) = (source, name);
        Ok(true)
    }
}
