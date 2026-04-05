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
            ContentBranch, Discovery, Missing, Parsed, Present,
            PropertyBankProcessor, Ready, Suspect, TimestampBranch, Unknown,
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

    /// Load and construct all schemas, resolving inheritance and property
    /// references.
    pub(crate) fn load_schemas(
        &self,
        pb: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        use super::schema_processor::{
            ContentBranch, Discovery, DiscoveryBranch, DiscoveryState,
            SchemaProcessor, TimestampBranch,
        };

        // Phase 1: Discovery - Load graph + scan files
        let context = self.discovery()?;

        // Start pipeline with discovery context
        let processor =
            SchemaProcessor::<Discovery, DiscoveryState>::from_context(context);

        // Run discovery stage
        let branch = processor.discover(&self.repository, &self.source)?;

        match branch {
            DiscoveryBranch::AllMissing(missing) => {
                // All files are new - parse them
                let parsed = missing.parse_new_schemas(&self.source)?;
                let graphed = parsed.build_graph(None)?;
                let analyzed = SchemaProcessor::from_graphed_batch(graphed)
                    .analyze_properties(None)?;
                let refreshed = SchemaProcessor::from_analyzed_batch(analyzed)
                    .refresh_metadata(&self.repository)?;
                let state =
                    refreshed.construct_schemas(&self.repository, pb)?;
                let processor = SchemaProcessor::from_construction_state(state);
                let schemas = processor.complete(&self.repository)?;
                Ok(schemas.into_iter().map(|arc| (*arc).clone()).collect())
            }
            DiscoveryBranch::SomePresent {
                missing,
                present,
            } => {
                // Phase 2: TimeComparison - check timestamps
                let ts_branch = present.compare_timestamps(&self.source)?;

                match ts_branch {
                    TimestampBranch::AllFresh(fresh) => {
                        // All timestamps match - skip to construction
                        let graphed = fresh.into_graphed_batch();
                        let analyzed =
                            SchemaProcessor::from_graphed_batch(graphed)
                                .analyze_properties(
                                    self.property_bank_delta.as_ref(),
                                )?;
                        let refreshed =
                            SchemaProcessor::from_analyzed_batch(analyzed)
                                .refresh_metadata(&self.repository)?;
                        let state = refreshed
                            .construct_schemas(&self.repository, pb)?;
                        let processor =
                            SchemaProcessor::from_construction_state(state);
                        let schemas = processor.complete(&self.repository)?;
                        Ok(schemas
                            .into_iter()
                            .map(|arc| (*arc).clone())
                            .collect())
                    }
                    TimestampBranch::SomeSuspect {
                        fresh,
                        suspect,
                    } => {
                        // Phase 3: ContentComparison - check content hashes
                        let content_branch =
                            suspect.compare_content(&self.source)?;

                        match content_branch {
                            ContentBranch::AllStaleTimestamps(timestamps) => {
                                // Only timestamps changed - refresh and
                                // construct
                                let graphed = timestamps.into_graphed_batch();
                                let analyzed =
                                    SchemaProcessor::from_graphed_batch(
                                        graphed,
                                    )
                                    .analyze_properties(
                                        self.property_bank_delta.as_ref(),
                                    )?;
                                let refreshed =
                                    SchemaProcessor::from_analyzed_batch(
                                        analyzed,
                                    )
                                    .refresh_metadata(&self.repository)?;
                                let state = refreshed
                                    .construct_schemas(&self.repository, pb)?;
                                let processor =
                                    SchemaProcessor::from_construction_state(
                                        state,
                                    );
                                let schemas =
                                    processor.complete(&self.repository)?;
                                Ok(schemas
                                    .into_iter()
                                    .map(|arc| (*arc).clone())
                                    .collect())
                            }
                            ContentBranch::SomeStaleContent {
                                timestamps,
                                content,
                            } => {
                                // Phase 5: FileParsed - parse changed schemas
                                let parsed_content = content.pass_through();

                                // Parse missing schemas if any
                                let parsed_new = if let Some(m) = missing {
                                    Some(m.parse_new_schemas(&self.source)?)
                                } else {
                                    None
                                };

                                // Merge parsed batches
                                let mut merged = parsed_content;
                                if let Some(new) = parsed_new {
                                    merged.new_schemas.extend(new.new_schemas);
                                }

                                // Get existing graph from fresh/timestamps
                                let existing_graph = fresh
                                    .map(|f| f.into_inheritance_graph())
                                    .or_else(|| {
                                        timestamps
                                            .map(|t| t.into_inheritance_graph())
                                    });

                                // Phase 6: InheritanceGraphed
                                let graphed =
                                    merged.build_graph(existing_graph)?;

                                // Phase 7: PropertyAnalysis
                                let analyzed =
                                    SchemaProcessor::from_graphed_batch(
                                        graphed,
                                    )
                                    .analyze_properties(
                                        self.property_bank_delta.as_ref(),
                                    )?;

                                // Phase 8: Refresh
                                let refreshed =
                                    SchemaProcessor::from_analyzed_batch(
                                        analyzed,
                                    )
                                    .refresh_metadata(&self.repository)?;

                                // Phase 9: Construction
                                let state = refreshed
                                    .construct_schemas(&self.repository, pb)?;

                                // Phase 10: Completion
                                let processor =
                                    SchemaProcessor::from_construction_state(
                                        state,
                                    );
                                let schemas =
                                    processor.complete(&self.repository)?;
                                Ok(schemas
                                    .into_iter()
                                    .map(|arc| (*arc).clone())
                                    .collect())
                            }
                        }
                    }
                }
            }
        }
    }

    /// Run the full ingestion pipeline.
    #[expect(dead_code, reason = "reserved for schema loading")]
    pub(crate) fn load_all(
        &mut self,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        let pb = self.load_property_bank()?;
        self.load_schemas(&pb)
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

        let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
        let branch = pipeline.discover(
            filename,
            &self.source,
            config_path,
            &self.repository,
        )?;

        let (completed, delta) = match branch {
            ComparisonBranch::Missing(p) => {
                self.handle_missing(p, filename, config_path)?
            }
            ComparisonBranch::Present(p) => {
                self.handle_present(p, filename, config_path)?
            }
        };

        self.property_bank_delta = delta;
        Ok(completed.into_bank())
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

    /// Discover schema files and load inheritance graph from DB.
    ///
    /// This method performs initial filesystem scanning and DB queries to
    /// prepare the context for schema pipeline processing.
    ///
    /// # Operations
    ///
    /// 1. Scans schema directory for valid schema files (json, toml, yaml, yml)
    /// 2. Detects if property bank file exists
    /// 3. Excludes property bank file from schema file list
    /// 4. Loads persisted topological graph from DB (if exists)
    /// 5. Returns `DiscoveryContext` containing graph, files, and
    ///    has_property_bank flag
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if:
    /// - File scanning fails (I/O error)
    /// - DB access fails (repository error)
    pub(crate) fn discovery(
        &self,
    ) -> Result<super::schema_processor::DiscoveryContext, SchemaLoaderError>
    {
        use super::schema_processor::DiscoveryContext;
        use crate::schema::error::SchemaIngestionError;

        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        // 1. Scan schema directory
        let pattern = format!("{}/**/*", self.schema_dir.display());
        let all_files = self.source.list_files(&pattern).map_err(|e| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::Io {
                    path: self.schema_dir.clone(),
                    source: std::io::Error::other(e),
                },
            ))
        })?;

        // 2. Check if property bank exists on disk
        let property_bank_filename = self.property_bank_filename.as_deref();
        let has_property_bank = all_files.iter().any(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .zip(property_bank_filename)
                .is_some_and(|(name, pb_name)| name == pb_name)
        });

        // 3. Filter for schema files (exclude property bank)
        let files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|path| {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                else {
                    return false;
                };

                // Exclude property bank file
                if let Some(pb_name) = property_bank_filename
                    && file_name == pb_name
                {
                    return false;
                }

                // Only include valid schema extensions
                let Some(ext) = path.extension().and_then(|e| e.to_str())
                else {
                    return false;
                };

                SCHEMA_EXTENSIONS
                    .iter()
                    .any(|allowed| ext.eq_ignore_ascii_case(allowed))
            })
            .collect();

        // 4. Load graph from DB (if exists)
        let graph = self
            .repository
            .get_topological_graph()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(DiscoveryContext {
            graph,
            files,
            has_property_bank,
        })
    }


    fn handle_missing(
        &self,
        processor: PropertyBankProcessor<Parsed, Missing>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        let constructed = processor.parse(&self.source, config_path)?;
        let delta = constructed.changed_property_names();
        let completed = constructed.create(filename, &self.repository)?;
        Ok((completed, Some(delta)))
    }

    fn handle_present(
        &self,
        processor: PropertyBankProcessor<Comparison, Present>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_timestamps(&self.source, config_path)? {
            TimestampBranch::Match(p) => Ok((p.fetch(&self.repository)?, None)),
            TimestampBranch::Mismatch(p) => {
                self.handle_content_mismatch(p, filename, config_path)
            }
        }
    }

    fn handle_content_mismatch(
        &self,
        processor: PropertyBankProcessor<Comparison, Suspect>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_content() {
            ContentBranch::Match(p) => {
                let completed = p
                    .sync_metadata(&self.repository)?
                    .fetch(&self.repository)?;
                Ok((completed, None))
            }
            ContentBranch::Mismatch(p) => {
                let parsed = p.parse(&self.source, config_path)?;
                self.handle_analysis_branch(parsed.analyze(), filename)
            }
        }
    }

    fn handle_analysis_branch(
        &self,
        branch: AnalysisBranch,
        filename: &str,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match branch {
            AnalysisBranch::Empty(p) => {
                let completed = p
                    .sync_metadata(&self.repository)?
                    .fetch(&self.repository)?;
                Ok((completed, None))
            }
            AnalysisBranch::Delta(p) => {
                let delta = p.changed_property_names();
                let completed = p.update(filename, &self.repository)?;
                Ok((completed, Some(delta)))
            }
            AnalysisBranch::Corrupt(p) => {
                let delta = p.changed_property_names();
                let completed = p.create(filename, &self.repository)?;
                Ok((completed, Some(delta)))
            }
        }
    }

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::aggregate::Config,
        fs::FsReader,
        schema::{
            aggregate::SchemaId,
            graph::{InheritanceNode, TopologicalGraph},
            testing::InMemoryRepository,
        },
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
                r#"
name = "test"
description = "Test schema"
                "#,
            )
            .unwrap();
        }
    }

    /// Helper to setup test repository with a persisted graph.
    fn setup_test_repo_with_graph(_temp: &TempDir) -> InMemoryRepository {
        let repo = InMemoryRepository::new();

        // Create minimal graph with one root node
        let id = SchemaId::new();
        let node = InheritanceNode::new_root(id);

        let mut nodes = HashMap::new();
        nodes.insert(id, node);

        let graph = TopologicalGraph {
            nodes,
            order: vec![id],
            roots: vec![id],
        };

        // Save graph to DB
        repo.save_topological_graph(&graph).unwrap();

        repo
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
    fn builder_discovery_loads_graph_from_db() {
        let temp = TempDir::new().unwrap();
        let repo = setup_test_repo_with_graph(&temp);
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let builder = Builder::new(repo, source, &config);
        let context = builder.discovery().unwrap();

        assert!(
            context.graph.is_some(),
            "Should load graph from DB when present"
        );
    }

    #[test]
    fn builder_discovery_excludes_property_bank() {
        let temp = TempDir::new().unwrap();
        create_schema_files(&temp, &["schema_a.toml", "property_bank.json"]);
        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let builder = Builder::new(repo, source, &config);
        let context = builder.discovery().unwrap();

        assert_eq!(
            context.files.len(),
            1,
            "Should exclude property_bank from schema files"
        );
    }

    #[test]
    fn builder_discovery_handles_missing_graph() {
        let temp = TempDir::new().unwrap();
        let repo = InMemoryRepository::new(); // Empty DB
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let builder = Builder::new(repo, source, &config);
        let context = builder.discovery().unwrap();

        assert!(
            context.graph.is_none(),
            "Should handle missing graph gracefully"
        );
    }

    #[test]
    fn builder_discovery_filters_by_extension() {
        let temp = TempDir::new().unwrap();
        create_schema_files(&temp, &[
            "schema_a.toml",
            "schema_b.json",
            "schema_c.yaml",
            "schema_d.yml",
            "readme.md",
            "config.txt",
        ]);
        let repo = InMemoryRepository::new();
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let builder = Builder::new(repo, source, &config);
        let context = builder.discovery().unwrap();

        assert_eq!(
            context.files.len(),
            4,
            "Should only include schema extensions"
        );
    }
}
