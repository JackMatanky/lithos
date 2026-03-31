#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

use std::{collections::HashSet, path::PathBuf};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaLoaderError,
        property::PropertyName,
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
    #[expect(dead_code, reason = "Builder retains config for future stages")]
    config: &'config Config,
    source: FsReader,
    repository: R,
    schema_dir: PathBuf,
    property_bank_path: PathBuf,
    property_bank_filename: Option<Box<str>>,
    property_bank_delta: Option<HashSet<PropertyName>>,
}

type PropertyBankCompletion =
    (PropertyBankProcessor<Completed, Ready>, Option<HashSet<PropertyName>>);

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
        let property_bank_path = config.paths().property_bank_path();
        let property_bank_filename = property_bank_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(Into::into);
        let schema_dir =
            config.paths().schema.schemas_dir().as_path().to_path_buf();

        Self {
            config,
            source,
            repository,
            schema_dir,
            property_bank_path,
            property_bank_filename,
            property_bank_delta: None,
        }
    }

    #[inline]
    #[expect(
        dead_code,
        reason = "property bank delta will be wired into later stages"
    )]
    pub(crate) fn set_property_bank_delta(
        &mut self,
        property_bank_delta: Option<HashSet<PropertyName>>,
    ) {
        self.property_bank_delta = property_bank_delta;
    }

    /// Load and construct the `PropertyBank`, automatically handling
    /// incremental staleness.
    pub fn load_property_bank(
        &mut self,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let config_path = self.property_bank_path.as_path();
        let filename = match self.property_bank_filename.as_deref() {
            Some(name) => name,
            None => self
                .source
                .filename(config_path)
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?,
        };

        // 1. Discovery: Determine which path to take
        let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
        let branch = pipeline.discover(
            filename,
            &self.source,
            config_path,
            &self.repository,
        )?;

        // 2. Execute the path to completion
        let (completed, delta) = match branch {
            ComparisonBranch::Missing(p) => {
                let content = self
                    .source
                    .read_to_string(config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                let parsed = p.parse(config_path, &content)?;
                let delta = parsed.changed_property_names();
                let delta = if delta.is_empty() {
                    None
                } else {
                    Some(delta)
                };
                let completed = parsed.create(filename, &self.repository)?;
                (completed, delta)
            }
            ComparisonBranch::Present(p) => {
                let content = self
                    .source
                    .read_to_string(config_path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;

                match p.check_timestamps(&content) {
                    TimestampBranch::Match(p) => {
                        (p.fetch(&self.repository)?, None)
                    }
                    TimestampBranch::Mismatch(p) => {
                        self.handle_content_mismatch(p, filename, config_path)?
                    }
                }
            }
        };

        self.property_bank_delta = delta;

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
    /// Load schemas using the new typestate pipeline (Phase 5+).
    ///
    /// This method orchestrates the 7-stage schema pipeline:
    /// 1. `Discovery` - Scan filesystem, query DB, branch into pipelines
    /// 2. `Comparison` - Timestamp/content hash checks (per-schema)
    /// 3. `TreeGraphed` - Batch graph building with cycle detection
    /// 4. `PropertyAnalysis` - Batch property/excludes delta computation
    /// 5. `Construction` - Batch level-by-level expand + merge
    /// 6. `Completed` - Batch persistence
    /// 7. `Refresh` - Early exit for metadata-only changes
    ///
    /// # Arguments
    /// * `pb` - Property bank for property resolution
    ///
    /// # Errors
    /// Returns `SchemaLoaderError` if any stage fails.
    pub(crate) fn load_schemas_v2(
        &self,
        pb: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        use super::schema_pipeline::{Discovery, SchemaTreeProcessor, Unknown};

        let property_bank_filename =
            match self.property_bank_filename.as_deref() {
                Some(name) => name,
                None => self
                    .source
                    .filename(self.property_bank_path.as_path())
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?,
            };

        let pipeline = SchemaTreeProcessor::<Discovery, Unknown>::new();
        let discovered = pipeline.discover_tree(
            &self.source,
            &self.repository,
            self.schema_dir.as_path(),
            property_bank_filename,
        )?;

        let compared = discovered.compare_files(&self.source)?;
        let graphed = compared.graph_structure()?;
        let analyzed =
            graphed.analyze_properties(self.property_bank_delta.as_ref())?;
        let refreshed = analyzed.refresh_metadata(&self.repository)?;
        let constructed = refreshed.construct_schemas(&self.repository, pb)?;

        constructed.persist(&self.repository)
    }

    /// Run the full ingestion pipeline.
    #[expect(dead_code, reason = "reserved for schema loading")]
    pub(crate) fn load_all(
        &mut self,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        let pb = self.load_property_bank()?;
        Ok(self.load_schemas(&pb))
    }

    fn handle_content_mismatch(
        &self,
        processor: PropertyBankProcessor<Comparison, Suspect>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_content(config_path) {
            ContentBranch::Match(p) => {
                let completed = p
                    .sync_metadata(&self.repository)?
                    .fetch(&self.repository)?;
                Ok((completed, None))
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
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.analyze(config_path)? {
            AnalysisBranch::Empty(p) => {
                let completed = p
                    .sync_metadata(&self.repository)?
                    .fetch(&self.repository)?;
                Ok((completed, None))
            }
            AnalysisBranch::Delta(p) => {
                let delta = p.changed_property_names();
                let delta = if delta.is_empty() {
                    None
                } else {
                    Some(delta)
                };
                let completed = p.update(filename, &self.repository)?;
                Ok((completed, delta))
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
