#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaLoaderError,
        property_bank_processor::{
            Completed, ContentBranch, DeltaBranch, Discovery, DiscoveryBranch,
            IsFreshContent, PropertyBankProcessor, TimestampBranch,
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
        let pipeline = PropertyBankProcessor::<Discovery>::new();
        let branch = pipeline.has_raw_view(
            filename,
            &self.source,
            &config_path,
            &self.repository,
        )?;

        // 2. Execute the path to completion
        let completed = match branch {
            DiscoveryBranch::New(p) => {
                let content = self
                    .source
                    .read_to_string(&config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                p.parse(&config_path, &content)?
                    .create(filename, &self.repository)?
            }
            DiscoveryBranch::FreshTimestamp(p) => {
                let content = self
                    .source
                    .read_to_string(&config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                match p.is_match(&content) {
                    TimestampBranch::Fetch(p) => p.fetch(&self.repository)?,
                    TimestampBranch::Content(p) => {
                        self.handle_content_branch(p, filename, &config_path)?
                    }
                }
            }
        };

        // 3. Extract the PropertyBank
        Ok(completed.into_bank())
    }

    /// Load and construct all schemas, resolving inheritance and property
    /// references.
    pub fn load_schemas(
        &self,
        _pb: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        Ok(Vec::new()) // Stub
    }

    /// Run the full ingestion pipeline.
    pub fn load_all(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        let pb = self.load_property_bank()?;
        self.load_schemas(&pb)
    }

    fn handle_content_branch(
        &self,
        processor: PropertyBankProcessor<IsFreshContent<'_>>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError> {
        match processor.is_match(config_path)? {
            ContentBranch::Match(p) => {
                p.update(&self.repository)?.fetch(&self.repository)
            }
            ContentBranch::Mismatch(p) => self
                .handle_delta_branch(p.filter_changed_properties(), filename),
        }
    }

    fn handle_delta_branch(
        &self,
        branch: DeltaBranch,
        filename: &str,
    ) -> Result<PropertyBankProcessor<Completed>, SchemaLoaderError> {
        match branch {
            DeltaBranch::ContentOnly(p) => {
                p.update(&self.repository)?.fetch(&self.repository)
            }
            DeltaBranch::PropertiesChanged(p) => {
                p.update(filename, &self.repository)
            }
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
