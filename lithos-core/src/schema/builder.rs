#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

#[path = "schema_pipeline.rs"]
mod schema_pipeline;

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaLoaderError,
        property_bank_processor::{
            Analysis, AnalysisBranch, Comparison, ComparisonBranch, Completed,
            ContentBranch, Discovery, PropertyBankProcessor, Ready, Suspect,
            TimestampBranch, Unknown,
        },
        storage::Repository,
    },
};

/// Schema loader — orchestrates the full schema ingestion pipeline.
pub struct Builder<'config, R> {
    config: &'config Config,
    source: FsReader,
    repository: R,
}

impl<'config, R: Repository> Builder<'config, R>
where
    R::Error: Into<crate::schema::error::SchemaRepositoryError>,
{
    /// Create a new `Builder` with a repository, file source, and config.
    #[inline]
    #[must_use]
    pub fn new(
        repository: R,
        source: FsReader,
        config: &'config Config,
    ) -> Self {
        Self {
            config,
            source,
            repository,
        }
    }

    /// Load and construct the `PropertyBank`, automatically handling
    /// incremental staleness.
    pub fn load_property_bank(
        &self,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let config_path = self.config.paths().property_bank_path();
        let filename = self
            .source
            .filename(&config_path)
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

        // 1. Discovery: Determine which path to take
        let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
        let branch = pipeline.discover(
            filename,
            &self.source,
            &config_path,
            &self.repository,
        )?;

        // 2. Execute the path to completion
        let completed = match branch {
            ComparisonBranch::Missing(p) => {
                let content = self
                    .source
                    .read_to_string(&config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                p.parse(&config_path, &content)?
                    .create(filename, &self.repository)?
            }
            ComparisonBranch::Present(p) => {
                let content = self
                    .source
                    .read_to_string(&config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

                match p.check_timestamps(&content) {
                    TimestampBranch::Match(p) => p.fetch(&self.repository)?,
                    TimestampBranch::Mismatch(p) => {
                        self.handle_content_mismatch(p, filename, &config_path)?
                    }
                }
            }
        };

        // 3. Extract the PropertyBank
        Ok(completed.into_bank())
    }

    /// Load and construct all schemas, resolving inheritance and property
    /// references.
    #[expect(clippy::unused_self, reason = "stubbed schema loading")]
    pub(crate) fn load_schemas(&self, _pb: &PropertyBank) -> Vec<Schema> {
        Vec::new() // Stub
    }

    #[expect(dead_code, reason = "schema pipeline scaffold")]
    #[expect(
        clippy::todo,
        reason = "scaffold code with incomplete implementation"
    )]
    pub(crate) fn load_schemas_v2(
        &self,
        _pb: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        // TODO: Implement full schema pipeline orchestration
        // This is scaffold code - will be implemented in later phases
        todo!("schema pipeline v2 orchestration")
    }

    /// Run the full ingestion pipeline.
    #[expect(dead_code, reason = "reserved for schema loading")]
    pub(crate) fn load_all(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        let pb = self.load_property_bank()?;
        Ok(self.load_schemas(&pb))
    }

    fn handle_content_mismatch(
        &self,
        processor: PropertyBankProcessor<Comparison, Suspect>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankProcessor<Completed, Ready>, SchemaLoaderError>
    {
        match processor.check_content(config_path) {
            ContentBranch::Match(p) => {
                p.sync_metadata(&self.repository)?.fetch(&self.repository)
            }
            ContentBranch::Mismatch(p) => {
                self.handle_analysis(p, filename, config_path)
            }
        }
    }

    fn handle_analysis(
        &self,
        processor: PropertyBankProcessor<Analysis, Suspect>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankProcessor<Completed, Ready>, SchemaLoaderError>
    {
        match processor.analyze(config_path)? {
            AnalysisBranch::Empty(p) => {
                p.sync_metadata(&self.repository)?.fetch(&self.repository)
            }
            AnalysisBranch::Delta(p) => p.update(filename, &self.repository),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::aggregate::Config, fs::FsReader,
        schema::testing::InMemoryRepository,
    };

    #[test]
    fn builder_constructs() {
        let temp = TempDir::new().unwrap();
        let raw = crate::config::raw::RawConfig::default();
        let config = Config::build(
            &raw,
            crate::config::vault::VaultId::new(),
            crate::config::vault::VaultRoot::try_new(temp.path().to_path_buf())
                .unwrap(),
            crate::config::aggregate::Version::initial(),
        )
        .unwrap();
        let source = FsReader::new(temp.path().to_path_buf());
        let repo = InMemoryRepository::new();

        let _builder = Builder::new(repo, source, &config);
    }
}
