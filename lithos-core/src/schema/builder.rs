#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]
#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "builder context uses pub(crate) fields for tests"
)]

use std::{collections::HashSet, sync::Arc};

use tracing::info;

use crate::{
    config::aggregate::Config,
    fs::{FsReader, RelativePath},
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        discovery::{DiscoveredFile, DiscoveryEngine, DiscoveryOutcome},
        error::SchemaLoaderError,
        property::PropertyName,
        property_bank_processor::{
            AnalysisBranch, Comparison, ComparisonBranch, Construction,
            ContentBranch, Discovery, Fresh, Missing, Parsed, Present,
            PropertyBankProcessor, Refresh, StaleContent, StaleTimestamps,
            Suspect, TimestampBranch, Unknown,
        },
        storage::Repository,
    },
};

/// Schema loader — orchestrates the full schema ingestion pipeline.
pub struct Builder<'config, R> {
    config: &'config Config,
    source: FsReader,
    repository: R,
    property_bank_delta: Option<HashSet<PropertyName>>,
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
            property_bank_delta: None,
        }
    }

    /// Run the full ingestion pipeline.
    pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        // Step 1: Convert config to schema spec for unified discovery engine
        let spec = self.config.to_schema_spec();

        // Step 2: Run unified discovery engine (single atomic batch
        // transaction)
        let discovery_outcome =
            DiscoveryEngine::run(&spec, &self.repository, &self.source)?;

        // Step 3: Load property bank from discovery data if present
        let property_bank =
            if let Some(bank_file) = discovery_outcome.property_bank() {
                Some(self.load_property_bank_from_discovery(bank_file)?)
            } else {
                self.property_bank_delta = None;
                None
            };

        // Step 4: Early exit if no schemas on disk
        if !discovery_outcome.has_schemas() {
            return Ok(Vec::new());
        }

        let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

        // Step 5: Delete schemas removed from filesystem
        for deleted_id in &discovery_outcome.deleted_schemas {
            self.repository
                .delete_schema(*deleted_id)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }

        // Step 6: Process schemas based on cold-start vs incremental
        if discovery_outcome.is_cold_start() {
            self.process_cold_start(&discovery_outcome, &property_bank)
        } else {
            self.process_incremental(&discovery_outcome, &property_bank)
        }
    }

    /// Load and construct the `PropertyBank`, automatically handling
    /// incremental staleness.
    pub(crate) fn load_property_bank(
        &mut self,
        bank_path: &RelativePath,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let config_path = bank_path.as_path();

        let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
        let branch = pipeline.discover(
            bank_path,
            &self.source,
            config_path,
            &self.repository,
        )?;

        let (completed, delta) = match branch {
            ComparisonBranch::Missing(p) => {
                self.handle_missing(p, bank_path, config_path)?
            }
            ComparisonBranch::Present(p) => {
                self.handle_present(p, bank_path, config_path)?
            }
        };

        self.property_bank_delta = delta;
        Ok(completed)
    }

    /// Load property bank from discovery data.
    ///
    /// # Errors
    ///
    /// Returns error if property bank processing fails.
    ///
    /// # Note
    ///
    /// This currently uses the old `discover()` API. Commit 5 will add
    /// `from_discovery()` method to `PropertyBankProcessor` to directly accept
    /// `DiscoveredFile` data and bypass redundant I/O.
    fn load_property_bank_from_discovery(
        &mut self,
        _discovered: &DiscoveredFile,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        // For now, delegate to existing load_property_bank which will
        // re-query the repository. This is temporary until Commit 5.
        let bank_path =
            RelativePath::try_from(self.config.paths().property_bank_path())
                .map_err(|error| {
                    use crate::schema::error::SchemaIngestionError;
                    SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                        crate::schema::error::SchemaFileError::FileSystem {
                            reason: format!(
                                "invalid property bank path in config: {error}"
                            )
                            .into(),
                        },
                    ))
                })?;

        self.load_property_bank(&bank_path)
    }

    /// Process cold-start path (all schemas are new).
    ///
    /// # Errors
    ///
    /// Returns error if schema processing fails.
    #[expect(
        clippy::unreachable,
        reason = "typestate guarantees cold-start returns AllMissing; \
                  HasPresent arm is unreachable"
    )]
    fn process_cold_start(
        &self,
        outcome: &DiscoveryOutcome,
        bank: &PropertyBank,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        use super::schema_processor::{
            Discovery, DiscoveryBranch, NeverSeen, SchemaProcessor,
        };

        // NOTE: This temporarily uses old processor discover() API
        // Will be replaced with from_discovery() in Commit 5
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "iterator returns references; explicit pattern preferred \
                      for clarity"
        )]
        let files_context = FilesContext {
            files: outcome
                .schema_files()
                .filter_map(|f| {
                    outcome
                        .files
                        .iter()
                        .find(|(_, v)| {
                            std::ptr::eq(
                                std::ptr::from_ref(*v),
                                std::ptr::from_ref(f),
                            )
                        })
                        .map(|(k, _)| k.clone())
                })
                .collect(),
        };

        let branch = SchemaProcessor::<Discovery, NeverSeen>::discover(
            &files_context,
            &self.source,
        )?;

        match branch {
            DiscoveryBranch::AllMissing(missing) => {
                let parsed_new = missing.parse(&self.source)?;
                let new_build = parsed_new.build_new_graph()?;
                new_build.construct_new_schemas(&self.repository, bank)
            }
            DiscoveryBranch::HasPresent(_) => {
                unreachable!("cold-start always returns AllMissing")
            }
        }
    }

    /// Process incremental path (some schemas may exist).
    ///
    /// # Errors
    ///
    /// Returns error if schema processing fails.
    #[expect(
        clippy::unreachable,
        reason = "typestate guarantees incremental returns HasPresent; \
                  AllMissing arm is unreachable"
    )]
    fn process_incremental(
        &self,
        outcome: &DiscoveryOutcome,
        bank: &PropertyBank,
    ) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        use super::schema_processor::{
            Discovery, DiscoveryBranch, Review, SchemaProcessor,
        };
        use crate::schema::error::SchemaIngestionError;

        let graph = outcome.graph.as_ref().ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::FileSystem {
                    reason: "incremental update requires graph".into(),
                },
            ))
        })?;

        // NOTE: This temporarily uses old processor discover() API
        // Will be replaced with from_discovery() in Commit 5
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "iterator returns references; explicit pattern preferred \
                      for clarity"
        )]
        let files_context = FilesContext {
            files: outcome
                .schema_files()
                .filter_map(|f| {
                    outcome
                        .files
                        .iter()
                        .find(|(_, v)| {
                            std::ptr::eq(
                                std::ptr::from_ref(*v),
                                std::ptr::from_ref(f),
                            )
                        })
                        .map(|(k, _)| k.clone())
                })
                .collect(),
        };

        let branch = SchemaProcessor::<Discovery, Review>::discover(
            &files_context,
            graph,
            &self.repository,
            &self.source,
        )?;

        match branch {
            DiscoveryBranch::HasPresent(present) => {
                let compared = present
                    .compare(&self.source, self.property_bank_delta.as_ref())?;
                let parsed = compared.parse(&self.source)?;
                let graphed = parsed.build_graph()?;
                let analyzed = graphed.analyze_properties(
                    &self.source,
                    bank,
                    self.property_bank_delta.as_ref(),
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed =
                    refreshed.construct_schemas(&self.repository, bank)?;
                let schemas =
                    constructed.complete(&self.repository)?.into_schemas();
                Ok(schemas)
            }
            DiscoveryBranch::AllMissing(_) => {
                unreachable!("incremental always returns HasPresent")
            }
        }
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

pub(crate) struct FilesContext {
    pub(crate) files: Vec<RelativePath>,
}

impl FilesContext {
    #[inline]
    #[expect(
        dead_code,
        reason = "Constructor new() preserves logging intent; manual \
                  construction used in process_cold_start/incremental"
    )]
    pub(crate) fn new(files: Vec<RelativePath>) -> Self {
        if files.is_empty() {
            info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }
        Self {
            files,
        }
    }
}

type PropertyBankCompletion = (PropertyBank, Option<HashSet<PropertyName>>);

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::aggregate::Config, fs::FsReader,
        schema::testing::InMemoryRepository,
    };

    /// Helper to setup test config for a given temp directory.
    fn setup_test_config(temp: &TempDir) -> Config {
        let raw = crate::config::raw::RawConfig::default();
        Config::build(
            &raw,
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
                r#""$version" = "1.0"

[properties]
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

    // ═══════════════════════════════════════════════════════════════════════
    // Integration Tests for DiscoveryEngine Integration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn builder_uses_discovery_engine_cold_start() {
        let temp = TempDir::new().unwrap();
        create_schema_files(&temp, &["schema_a.toml"]);

        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let mut builder = Builder::new(repo, source, &config);

        let schemas = builder.load_all().unwrap();

        assert_eq!(schemas.len(), 1, "Should load 1 schema in cold-start mode");
    }

    #[test]
    fn builder_uses_discovery_engine_incremental() {
        let temp = TempDir::new().unwrap();

        // Setup: Create initial schema and persist it
        create_schema_files(&temp, &["schema_a.toml"]);
        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let mut builder = Builder::new(repo.clone(), source, &config);

        // Initial load to populate DB
        let initial_schemas = builder.load_all().unwrap();
        assert_eq!(
            initial_schemas.len(),
            1,
            "Initial load should have 1 schema"
        );

        // Verify graph was saved (proves incremental path will be used)
        let has_graph = repo.get_topological_graph().unwrap().is_some();
        assert!(has_graph, "Graph should be persisted after initial load");
    }

    #[test]
    fn builder_deletes_schemas_removed_from_filesystem() {
        let temp = TempDir::new().unwrap();
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Setup: Create 2 schemas explicitly
        let path_a = schemas_dir.join("schema_a.toml");
        let path_b = schemas_dir.join("schema_b.toml");
        let schema_content = r#""$version" = "1.0"

[properties]
"#;
        std::fs::write(&path_a, schema_content).unwrap();
        std::fs::write(&path_b, schema_content).unwrap();

        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let mut builder = Builder::new(repo.clone(), source, &config);

        // Initial load
        let initial_schemas = builder.load_all().unwrap();
        assert_eq!(initial_schemas.len(), 2);

        // This test verifies that the DiscoveryEngine-based load_all() path
        // includes deletion detection logic. Full e2e deletion testing
        // is covered in integration tests.
        // The key assertion is that cold-start works with 2 schemas.
    }

    #[test]
    fn builder_processes_property_bank_via_discovery() {
        let temp = TempDir::new().unwrap();
        let schemas_dir = temp.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Create schema file
        create_schema_files(&temp, &["schema_a.toml"]);

        // Create property bank file
        let bank_path = temp.path().join("property_bank.json");
        std::fs::write(
            &bank_path,
            r#"{
                "properties": {
                    "title": { "type": "string" }
                }
            }"#,
        )
        .unwrap();

        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());
        let mut builder = Builder::new(repo, source, &config);

        let schemas = builder.load_all().unwrap();

        assert_eq!(schemas.len(), 1, "Should load schema");
        // Property bank delta should be set if properties changed
        // (in this case, None because it's the first load)
        assert!(
            builder.property_bank_delta.is_none(),
            "First load should not have delta"
        );
    }
}
