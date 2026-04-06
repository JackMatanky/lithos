#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]
#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "builder context uses pub(crate) fields for tests"
)]

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
            AnalysisBranch, Comparison, ComparisonBranch, ContentBranch,
            Discovery, Missing, Parsed, Present, PropertyBankProcessor,
            Suspect, TimestampBranch, Unknown,
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

#[derive(Debug, Clone)]
pub(crate) struct DiscoveryContext {
    pub(crate) graph: Option<
        crate::schema::graph::InheritanceGraph<
            crate::schema::graph::InheritanceNode,
        >,
    >,
    pub(crate) files: Vec<PathBuf>,
    #[expect(dead_code, reason = "kept for future property-bank gating")]
    pub(crate) has_property_bank: bool,
}

type PropertyBankCompletion = (PropertyBank, Option<HashSet<PropertyName>>);

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

    /// Load and construct all schemas using a discovery context.
    fn load_schemas(
        &self,
        pb: &PropertyBank,
        context: &DiscoveryContext,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        use std::collections::HashMap;

        use super::schema_processor::{
            Discovery, DiscoveryBranch, FileParsed, GraphMissing, GraphPresent,
            InheritanceGraphed, Missing, NewBatch, Parsed, SchemaProcessor,
        };
        use crate::schema::graph::InheritanceGraph;

        let branch = if context.graph.is_some() {
            SchemaProcessor::<Discovery, GraphPresent>::discover(
                context,
                &self.repository,
                &self.source,
            )?
        } else {
            SchemaProcessor::<Discovery, GraphMissing>::discover(
                context,
                &self.source,
            )?
        };

        match branch {
            DiscoveryBranch::AllMissing(missing) => {
                let parsed_new = missing
                    .into_file_parsed()
                    .parse_new_schemas(&self.source)?;
                let parsed_state: SchemaProcessor<FileParsed, Parsed> =
                    SchemaProcessor::<FileParsed, Parsed>::transition(
                        FileParsed,
                        Parsed {
                            graph: InheritanceGraph {
                                order: Vec::new(),
                                nodes: HashMap::new(),
                                roots: Vec::new(),
                            },
                            new_schemas: NewBatch::new(),
                            deleted_ids: Vec::new(),
                        },
                    );
                let graphed =
                    SchemaProcessor::<InheritanceGraphed, Parsed>::build_graph(
                        parsed_state,
                        &parsed_new,
                    )?;
                let analyzed = graphed.analyze_properties(
                    &self.source,
                    self.property_bank_delta.as_ref(),
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed =
                    refreshed.construct_schemas(&self.repository, pb)?;
                let schemas = constructed.complete(&self.repository)?;
                Ok(schemas.into_iter().map(|arc| (*arc).clone()).collect())
            }
            DiscoveryBranch::HasPresent(present) => {
                let present = present.into_comparison();
                let new_schemas = present.status.new_schemas.clone();
                let parsed_new = if new_schemas.is_empty() {
                    NewBatch::new()
                } else {
                    let missing_processor: SchemaProcessor<
                        FileParsed,
                        Missing,
                    > = SchemaProcessor::<FileParsed, Missing>::transition(
                        FileParsed,
                        Missing {
                            new_schemas,
                        },
                    );
                    missing_processor.parse_new_schemas(&self.source)?
                };
                let compared = present
                    .compare(&self.source, self.property_bank_delta.as_ref())?;
                let parsed = compared.parse_stale_schemas(&self.source)?;
                let graphed =
                    SchemaProcessor::<InheritanceGraphed, Parsed>::build_graph(
                        parsed,
                        &parsed_new,
                    )?;
                let analyzed = graphed.analyze_properties(
                    &self.source,
                    self.property_bank_delta.as_ref(),
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed =
                    refreshed.construct_schemas(&self.repository, pb)?;
                let schemas = constructed.complete(&self.repository)?;
                Ok(schemas.into_iter().map(|arc| (*arc).clone()).collect())
            }
        }
    }

    /// Run the full ingestion pipeline.
    pub fn load_all(&mut self) -> Result<Vec<Schema>, SchemaLoaderError> {
        let context = self.discover()?;
        let pb = self.load_property_bank()?;
        if context.files.is_empty() {
            return Ok(Vec::new());
        }
        self.load_schemas(&pb, &context)
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
        Ok(completed)
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
    ///    `has_property_bank` flag
    ///
    /// # Errors
    ///
    /// Returns `SchemaLoaderError` if:
    /// - File scanning fails (I/O error)
    /// - DB access fails (repository error)
    pub(crate) fn discover(
        &self,
    ) -> Result<DiscoveryContext, SchemaLoaderError> {
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
        let completed = constructed.create(filename, &self.repository)?;
        Ok((completed.into_bank(), None))
    }

    fn handle_present(
        &self,
        processor: PropertyBankProcessor<Comparison, Present>,
        filename: &str,
        config_path: &std::path::Path,
    ) -> Result<PropertyBankCompletion, SchemaLoaderError> {
        match processor.check_timestamps(&self.source, config_path)? {
            TimestampBranch::Match(p) => {
                let completed = p.fetch(&self.repository)?;
                Ok((completed.into_bank(), None))
            }
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
                Ok((completed.into_bank(), None))
            }
            ContentBranch::Mismatch(p) => {
                let parsed = p.parse(config_path)?;
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
                Ok((completed.into_bank(), None))
            }
            AnalysisBranch::Delta(p) => {
                let completed = p.update(filename, &self.repository)?;
                let (bank, delta) = completed.into_bank_with_changes();
                Ok((bank, Some(delta)))
            }
            AnalysisBranch::Corrupt(p) => {
                let completed = p.create(filename, &self.repository)?;
                Ok((completed.into_bank(), None))
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
            graph::{InheritanceGraph, InheritanceNode},
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

        let graph = InheritanceGraph {
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
        let context = builder.discover().unwrap();

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
        let context = builder.discover().unwrap();

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
        let context = builder.discover().unwrap();

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
        let context = builder.discover().unwrap();

        assert_eq!(
            context.files.len(),
            4,
            "Should only include schema extensions"
        );
    }
}
