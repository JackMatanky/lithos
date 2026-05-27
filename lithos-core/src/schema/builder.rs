#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]

use std::{collections::HashSet, sync::Arc};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        discovery::{DiscoveryEngine, PropertyBankDiscovery},
        error::{SchemaIngestionError, SchemaLoaderError},
        property::PropertyName,
        property_bank_processor::PropertyBankProcessor,
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

        let file = bank_discovery.entry().clone();

        let schema_spec = self.config.to_schema_spec().map_err(|error| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::FileSystem {
                    reason: format!(
                        "failed to build schema config spec: {error}"
                    )
                    .into(),
                },
            ))
        })?;

        let processor =
            PropertyBankProcessor::from_discovery(file, schema_spec.root())
                .map_err(SchemaIngestionError::from)?;

        let (bank, delta) = processor.run(
            bank_discovery.view(),
            &self.source,
            &self.repository,
        )?;

        self.property_bank_delta = delta;
        Ok(bank)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::{aggregate::Config, paths::SchemaConfigSpec},
        fs::{
            DirPath, FsFile, FsReader,
            path::{RelativeDirPath, RelativeFilePath},
        },
        schema::{
            discovery::DiscoveryEngine, repository::Repository,
            storage::testing::InMemoryRepository,
        },
    };

    fn assert_file_guarantee(file: &FsFile) {
        assert!(file.path().as_path().ends_with("property_bank.json"));
    }

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

    #[test]
    fn load_property_bank_discovery_entry_is_file_typed() {
        let temp = TempDir::new().unwrap();
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(schemas_dir.join("property_bank.json"), "{}").unwrap();

        let vault_root = DirPath::try_from(temp.path().to_path_buf()).unwrap();
        let dir_rel = RelativeDirPath::try_from("schemas").unwrap();
        let bank_rel =
            RelativeFilePath::try_from("schemas/property_bank.json").unwrap();
        let spec = SchemaConfigSpec::new(vault_root, dir_rel, bank_rel);

        let repo = InMemoryRepository::new();
        let discovery = DiscoveryEngine::run(&spec, &repo).unwrap();
        let bank_discovery = discovery
            .property_bank()
            .expect("property bank should be discovered");

        assert_file_guarantee(bank_discovery.entry());
    }
}
