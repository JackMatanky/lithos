#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema, bank::PropertyBank, error::SchemaLoaderError,
        property_bank_pipeline::PropertyBankPipeline, storage::Repository,
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

        // 1. Discovery: Determine which path to take
        let path = PropertyBankPipeline::discover(
            &config_path,
            &self.source,
            &self.repository,
        )?;

        // 2. Execute the path to completion
        let completed = path.into_completed(&self.repository)?;

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
