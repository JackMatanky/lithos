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

    /// Discover schema files and load inheritance graph from DB.
    ///
    /// This method performs initial filesystem scanning and DB queries to
    /// prepare the context for schema pipeline processing.
    ///
    /// # Operations
    ///
    /// 1. Scans schema directory for valid schema files (json, toml, yaml, yml)
    /// 2. Excludes property bank file from schema file list
    /// 3. Loads persisted topological graph from DB (if exists)
    /// 4. Returns `DiscoveryContext` containing graph, files, and metadata
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if:
    /// - File scanning fails (I/O error)
    /// - DB access fails (repository error)
    pub(crate) fn discovery(
        &self,
    ) -> Result<super::schema_pipeline::DiscoveryContext, SchemaLoaderError>
    {
        use super::schema_pipeline::DiscoveryContext;
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

        // 2. Filter for schema files (exclude property bank)
        let property_bank_filename = self.property_bank_filename.as_deref();
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
        })
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

        // Phase 1: Discovery - Load graph + scan files
        let context = self.discovery()?;

        // Start pipeline with discovery context
        let pipeline = SchemaTreeProcessor::<Discovery, Unknown>::new();
        let discovered = pipeline.discover_with_context(
            context,
            &self.source,
            &self.repository,
        )?;

        let compared = discovered
            .compare_files(&self.source, self.property_bank_delta.as_ref())?;
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
