#![expect(clippy::missing_errors_doc, reason = "Facade methods")]
#![expect(clippy::missing_inline_in_public_items, reason = "Facade methods")]
#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "builder context uses pub(crate) fields for tests"
)]

use std::{collections::HashSet, path::PathBuf};

use tracing::info;

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaLoaderError,
        inheritance::{InheritanceGraph, InheritanceNode},
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
    pub fn load_all(&mut self) -> Result<Vec<Schema>, SchemaLoaderError> {
        use super::schema_processor::{
            Discovery, DiscoveryBranch, NeverSeen, Review, SchemaProcessor,
        };

        let mut bank_branch = BankContextBranch::Missing;
        let files_context = self.discover_files(|context| {
            bank_branch = BankContextBranch::Present(context);
        })?;
        let graph_branch = self.discover_graph()?;

        let property_bank = match bank_branch {
            BankContextBranch::Missing => {
                self.property_bank_delta = None;
                None
            }
            BankContextBranch::Present(context) => {
                Some(self.load_property_bank(&context)?)
            }
        };

        if files_context.files.is_empty() {
            return Ok(Vec::new());
        }

        let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

        let branch = match graph_branch {
            GraphContextBranch::Present {
                graph,
            } => SchemaProcessor::<Discovery, Review>::discover(
                &files_context,
                &graph,
                &self.repository,
                &self.source,
            )?,
            GraphContextBranch::Missing => {
                SchemaProcessor::<Discovery, NeverSeen>::discover(
                    &files_context,
                    &self.source,
                )?
            }
        };

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
                    self.property_bank_delta.as_ref(),
                )?;
                let refreshed = analyzed.refresh_metadata(&self.repository)?;
                let constructed = refreshed
                    .construct_schemas(&self.repository, &property_bank)?;
                let schemas =
                    constructed.complete(&self.repository)?.into_schemas();
                Ok(schemas.into_iter().map(|arc| (*arc).clone()).collect())
            }
        }
    }

    /// Load and construct the `PropertyBank`, automatically handling
    /// incremental staleness.
    pub(crate) fn load_property_bank(
        &mut self,
        context: &PropertyBankContext,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let config_path = context.path.as_path();
        let filename = context.filename.as_ref();

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

    pub(crate) fn discover_files<F>(
        &self,
        mut on_bank_found: F,
    ) -> Result<FilesContext, SchemaLoaderError>
    where
        F: FnMut(PropertyBankContext),
    {
        use crate::schema::error::SchemaIngestionError;

        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        let bank_filename = self.resolve_property_bank_filename()?;
        let schema_dir = self.config.paths().schema.schemas_dir();
        let property_bank_path = self.config.paths().property_bank_path();

        let pattern = format!("{}/**/*", schema_dir.as_path().display());
        let all_files = self.source.list_files(&pattern).map_err(|e| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                crate::schema::error::SchemaFileError::Io {
                    path: schema_dir.as_path().to_path_buf(),
                    source: std::io::Error::other(e),
                },
            ))
        })?;

        let mut files = Vec::new();
        let mut has_property_bank = false;

        for path in all_files {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                continue;
            };

            if file_name == bank_filename.as_ref() {
                if has_property_bank {
                    return Err(SchemaLoaderError::Ingestion(
                        SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: "duplicate property bank file found"
                                    .into(),
                            },
                        ),
                    ));
                }
                has_property_bank = true;
                on_bank_found(PropertyBankContext {
                    filename: bank_filename.clone(),
                    path: property_bank_path.clone(),
                });
                continue;
            }

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };

            if SCHEMA_EXTENSIONS
                .iter()
                .any(|allowed| ext.eq_ignore_ascii_case(allowed))
            {
                files.push(path);
            }
        }

        let mut context = FilesContext::new(files);
        if has_property_bank {
            context.set_property_bank_existence();
        }
        Ok(context)
    }

    pub(crate) fn discover_graph(
        &self,
    ) -> Result<GraphContextBranch, SchemaLoaderError> {
        let graph = self
            .repository
            .get_topological_graph()
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        Ok(match graph {
            Some(graph) => GraphContextBranch::Present {
                graph,
            },
            None => GraphContextBranch::Missing,
        })
    }

    fn resolve_property_bank_filename(
        &self,
    ) -> Result<Box<str>, SchemaLoaderError> {
        if let Some(name) = self
            .config
            .paths()
            .property_bank_path()
            .file_name()
            .and_then(|name| name.to_str())
        {
            return Ok(name.into());
        }

        self.source
            .filename(self.config.paths().property_bank_path().as_path())
            .map(Into::into)
            .map_err(|e| SchemaLoaderError::Ingestion(e.into()))
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

#[derive(Debug, Clone)]
pub(crate) struct FilesContext {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) has_property_bank: bool,
}

impl FilesContext {
    #[inline]
    fn new(files: Vec<PathBuf>) -> Self {
        if files.is_empty() {
            info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }
        Self {
            files,
            has_property_bank: false,
        }
    }

    #[inline]
    fn set_property_bank_existence(&mut self) {
        self.has_property_bank = true;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PropertyBankContext {
    pub(crate) filename: Box<str>,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) enum BankContextBranch {
    Missing,
    Present(PropertyBankContext),
}

#[derive(Debug, Clone)]
pub(crate) enum GraphContextBranch {
    Missing,
    Present {
        graph: InheritanceGraph<InheritanceNode>,
    },
}

type PropertyBankCompletion = (PropertyBank, Option<HashSet<PropertyName>>);

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
            inheritance::{InheritanceGraph, InheritanceNode},
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

        let graph = InheritanceGraph::new(nodes, vec![id], vec![id]);

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
        let graph_branch = builder.discover_graph().unwrap();

        assert!(
            matches!(graph_branch, GraphContextBranch::Present { .. }),
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
        let mut bank_branch = BankContextBranch::Missing;
        let context = builder
            .discover_files(|bank| {
                bank_branch = BankContextBranch::Present(bank);
            })
            .unwrap();

        assert!(
            context.has_property_bank,
            "Should detect property bank presence"
        );
        assert_eq!(
            context.files.len(),
            1,
            "Should exclude property_bank from schema files"
        );
        assert!(
            matches!(bank_branch, BankContextBranch::Present(_)),
            "Should return property bank context"
        );
    }

    #[test]
    fn builder_discovery_handles_missing_graph() {
        let temp = TempDir::new().unwrap();
        let repo = InMemoryRepository::new(); // Empty DB
        let config = setup_test_config(&temp);
        let source = FsReader::new(temp.path().to_path_buf());

        let builder = Builder::new(repo, source, &config);
        let graph_branch = builder.discover_graph().unwrap();

        assert!(
            matches!(graph_branch, GraphContextBranch::Missing),
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
        let context = builder.discover_files(|_| {}).unwrap();

        assert_eq!(
            context.files.len(),
            4,
            "Should only include schema extensions"
        );
    }
}
