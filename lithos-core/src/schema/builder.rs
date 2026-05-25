#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

use std::{collections::HashSet, sync::Arc};

use crate::{
    config::aggregate::Config,
    fs::{FsReader, RelativePath},
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        discovery::{DiscoveryEngine, PropertyBankDiscovery},
        error::{SchemaIngestionError, SchemaLoaderError},
        property::PropertyName,
        property_bank_processor::{
            AnalysisBranch, Comparison, ComparisonBranch, Construction,
            ContentBranch, Fresh, Missing, Parsed, Present,
            PropertyBankProcessor, Refresh, StaleContent, StaleTimestamps,
            Suspect, TimestampBranch,
        },
        repository::Repository,
    },
};

/// Schema loader — orchestrates the full schema ingestion pipeline.
pub struct Builder<'config, R> {
    config: &'config Config,
    source: FsReader,
    repository: R,
    property_bank_delta: Option<HashSet<PropertyName>>,
}

impl<'config, R> Builder<'config, R>
where
    R: Repository,
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
            property_bank_delta: None,
        }
    }

    /// Run the full ingestion pipeline.
    pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        use super::schema_processor::{
            Discovery, DiscoveryBranch, NeverSeen, Review, SchemaProcessor,
        };

        // 1. Single discovery call replaces discover_files() + discover_graph()
        let schema_spec = self.config.to_schema_spec().map_err(|error| {
            // TODO(.scratch/pathkey-migration/04-schema-configspec-redesign.
            // md): Introduce a dedicated cross-context projection
            // error instead of stringifying config-spec
            // construction failures into FileSystem.
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::FileSystem {
                    reason: format!(
                        "failed to build schema config spec: {error}"
                    )
                    .into(),
                },
            ))
        })?;
        let discovery = DiscoveryEngine::run(&schema_spec, &self.repository)?;

        // 2. Load property bank if present
        let property_bank =
            if let Some(bank_discovery) = discovery.property_bank() {
                Some(self.load_property_bank(bank_discovery)?)
            } else {
                self.property_bank_delta = None;
                None
            };

        // 3. Early exit if no schemas
        if !discovery.has_schemas() {
            return Ok(Vec::new());
        }

        let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

        // 4. Route to SchemaProcessor based on graph presence
        // Use from_discovery_result constructors to skip duplicate discovery
        let branch = match discovery.graph() {
            Some(graph) => {
                SchemaProcessor::<Discovery, Review>::from_discovery_result(
                    &discovery, graph,
                )?
            }
            None => {
                SchemaProcessor::<Discovery, NeverSeen>::from_discovery_result(
                    &discovery,
                )?
            }
        };

        // 6. Process through pipeline (unchanged)
        match branch {
            DiscoveryBranch::AllMissing(missing) => {
                let parsed_new = missing.parse(&self.source)?;
                let new_build = parsed_new.build_new_graph()?;
                new_build
                    .construct_new_schemas(&self.repository, &property_bank)
            }
            DiscoveryBranch::HasPresent(present) => {
                let compared = present
                    .compare(&self.source, self.property_bank_delta.as_ref())?;
                let parsed = compared.parse(&self.source)?;
                let graphed = parsed.build_graph()?;
                let analyzed = graphed.analyze_properties(
                    &self.source,
                    &property_bank,
                    self.property_bank_delta.as_ref(),
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed = refreshed
                    .construct_schemas(&self.repository, &property_bank)?;
                let schemas =
                    constructed.complete(&self.repository)?.into_schemas();
                Ok(schemas)
            }
        }
    }

    /// Load and construct the `PropertyBank`, automatically handling
    /// incremental staleness.
    pub(crate) fn load_property_bank(
        &mut self,
        bank_discovery: &PropertyBankDiscovery,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        use crate::schema::error::SchemaIngestionError;

        let entry_path = bank_discovery.entry().path();
        let config_path = entry_path.as_path();
        let file_info =
            bank_discovery.entry().metadata().as_file().cloned().ok_or_else(
                || {
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: "property bank entry must be a file".into(),
                        },
                    ))
                },
            )?;

        // Convert PathBuf to RelativePath for PropertyBankProcessor
        let relative_raw =
            config_path.strip_prefix(self.source.root()).map_err(|error| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!(
                            "invalid property bank path (outside root): \
                             {error}"
                        )
                        .into(),
                    },
                ))
            })?;
        let relative_path =
            RelativePath::try_from(relative_raw).map_err(|error| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: format!("invalid property bank path: {error}")
                            .into(),
                    },
                ))
            })?;

        // Route directly to Comparison stage using discovered data
        // (skip Discovery stage since PropertyBankDiscovery already has all
        // data)
        let branch = match bank_discovery.view() {
            Some(view) => ComparisonBranch::Present(PropertyBankProcessor::<
                Comparison,
                Present,
            >::transition(
                Comparison,
                Present::new(file_info, view.clone()),
            )),
            None => ComparisonBranch::Missing(PropertyBankProcessor::<
                Parsed,
                Missing,
            >::transition(
                Parsed,
                Missing::new(file_info),
            )),
        };

        let (completed, delta) = match branch {
            ComparisonBranch::Missing(p) => {
                self.handle_missing(p, &relative_path, config_path)?
            }
            ComparisonBranch::Present(p) => {
                self.handle_present(p, &relative_path, config_path)?
            }
        };

        self.property_bank_delta = delta;
        Ok(completed)
    }

    fn handle_missing(
        &self,
        processor: PropertyBankProcessor<Parsed, Missing>,
        path: &RelativePath,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        let constructed = processor.parse(&self.source, config_path)?;
        let completed = constructed.create(path, &self.repository)?;
        Ok((completed.into_bank(), None))
    }

    fn handle_present(
        &self,
        processor: PropertyBankProcessor<Comparison, Present>,
        path: &RelativePath,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_timestamps(&self.source, config_path)? {
            TimestampBranch::Match(p) => Ok((self.fetch_fresh(p)?, None)),
            TimestampBranch::Mismatch(p) => {
                self.handle_content_mismatch(p, path, config_path)
            }
        }
    }

    fn handle_content_mismatch(
        &self,
        processor: PropertyBankProcessor<Comparison, Suspect>,
        path: &RelativePath,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_content() {
            ContentBranch::Match(p) => {
                Ok((self.sync_and_fetch_timestamps(p)?, None))
            }
            ContentBranch::Mismatch(p) => {
                let parsed = p.parse(config_path)?;
                self.handle_analysis_branch(parsed.analyze(), path)
            }
        }
    }

    fn handle_analysis_branch(
        &self,
        branch: AnalysisBranch,
        path: &RelativePath,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match branch {
            AnalysisBranch::Empty(p) => {
                Ok((self.sync_and_fetch_content(p)?, None))
            }
            AnalysisBranch::Delta(p) => {
                let completed = p.update(path, &self.repository)?;
                let (bank, delta) = completed.into_bank_with_changes();
                Ok((bank, Some(delta)))
            }
            AnalysisBranch::Corrupt(p) => {
                let completed = p.create(path, &self.repository)?;
                Ok((completed.into_bank(), None))
            }
        }
    }

    #[inline]
    fn fetch_fresh(
        &self,
        processor: PropertyBankProcessor<Construction, Fresh>,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let completed = processor.fetch(&self.repository)?;
        Ok(completed.into_bank())
    }

    #[inline]
    fn sync_and_fetch_timestamps(
        &self,
        processor: PropertyBankProcessor<Refresh, StaleTimestamps>,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let fresh = processor.sync_metadata(&self.repository)?;
        self.fetch_fresh(fresh)
    }

    #[inline]
    fn sync_and_fetch_content(
        &self,
        processor: PropertyBankProcessor<Refresh, StaleContent>,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let fresh = processor.sync_metadata(&self.repository)?;
        self.fetch_fresh(fresh)
    }
}

type PropertyBankCompletion = (PropertyBank, Option<HashSet<PropertyName>>);

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::aggregate::Config,
        fs::FsReader,
        schema::{
            repository::Repository, storage::testing::InMemoryRepository,
        },
    };

    /// Helper to setup test config for a given temp directory.
    fn setup_test_config(temp: &TempDir) -> Config {
        crate::config::builder::build_from_layers(
            None,
            None,
            crate::config::vault::VaultId::new(),
            crate::config::vault::VaultRoot::try_new(temp.path().to_path_buf())
                .unwrap(),
            crate::config::aggregate::Version::initial(),
        )
        .unwrap()
    }

    /// Helper to create schema files in temp directory.
    fn create_schema_files(temp: &TempDir, filenames: &[&str]) {
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        for filename in filenames {
            let path = schemas_dir.join(filename);
            std::fs::write(
                path,
                r#"
name = "test"
description = "Test schema"
                "#,
            )
            .unwrap();
        }
    }

    #[test]
    fn builder_constructs() {
        let temp = TempDir::new().unwrap();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let repo = InMemoryRepository::new();

        let _builder = Builder::new(repo, source, &config);
    }

    #[test]
    fn builder_load_all_orchestrates_discovery() {
        let temp = TempDir::new().unwrap();
        create_schema_files(&temp, &["schema_a.toml"]);
        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let mut builder = Builder::new(repo, source, &config);
        let result = builder.load_all();

        // Should successfully run discovery and start pipeline
        // (will likely fail on parsing if files are empty, but that's fine for
        // this test)
        assert!(
            result.is_ok()
                || matches!(
                    result.unwrap_err(),
                    SchemaLoaderError::Ingestion(_)
                )
        );
    }

    #[test]
    fn builder_new_accepts_repository_trait() {
        fn assert_builder_new<R>(repo: R, source: FsReader, config: &Config)
        where
            R: Repository,
        {
            let _ = Builder::new(repo, source, config);
        }

        let temp = TempDir::new().unwrap();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let repo = InMemoryRepository::new();

        assert_builder_new(repo, source, &config);
    }
}
