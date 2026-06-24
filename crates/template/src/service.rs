//! Template ingestion and rendering service orchestration.
//!
//! [`TemplateService`] is the use-case orchestrator that wires together the
//! template repository, the filesystem writer, and the rendering engine. It
//! owns lookup, validation workflow, rendering orchestration, target
//! resolution, conflict checks, and commit orchestration.
//!
//! The service composes its ports via uniform generic parameters
//! ([`ReadRepository`] + [`WriteRepository`], [`FileWriter`], and the
//! crate-internal `TemplateEngine`) following the hexagonal architecture
//! pattern: direct injection of trait-bounded fields, no `Box<dyn …>`, no
//! factory closures.

use std::collections::{HashMap, HashSet};

use trace_db::DbError;
use trace_fs::{DirScanner, FileNode, FileWriter, PathKey, WriteTarget};
use trace_settings::template::TemplateConfigSpec;

use crate::{
    aggregate::{Template, TemplateName},
    artifact::TemplateArtifact,
    engine::TemplateEngine,
    error::{TemplateError, TemplateRepositoryError},
    processor::{Discovered, DiscoveredTemplate, Init, TemplateProcessor},
    repository::{ReadRepository, WriteRepository},
};

#[derive(Debug)]
struct ScannedTemplate {
    file: FileNode,
    path_key: PathKey,
}

/// Input for [`TemplateService::create`].
///
/// Carries the requested template name, the vault-relative output path, the
/// render context, and a `dry_run` flag that selects between rendering-only
/// preview and full render-and-commit.
#[derive(Debug, Clone)]
pub struct CreateInput {
    /// Name of the template to render.
    pub name: TemplateName,
    /// Vault-relative path where the rendered artifact should be written.
    pub output_path: String,
    /// Variables substituted into the template during rendering.
    pub context: HashMap<String, String>,
    /// When `true`, the service renders the template and validates the
    /// output target without writing any file.
    pub dry_run: bool,
}

/// Rendered template text returned by a dry-run preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTemplate(String);

impl RenderedTemplate {
    /// Returns the rendered template content as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the owned rendered string.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Outcome of [`TemplateService::create`].
///
/// Distinguishes between a dry-run preview that produced rendered text and
/// validated the output target, and a real commit that wrote the rendered
/// file to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateTemplateOutcome {
    /// Dry-run preview: the template rendered successfully and the output
    /// path passed validation, but no file was written.
    Preview {
        /// Validated vault-relative output path.
        output_path: WriteTarget,
        /// Rendered template content.
        rendered: RenderedTemplate,
    },
    /// The artifact was committed to disk.
    Created {
        /// Validated vault-relative output path the artifact was written to.
        output_path: WriteTarget,
        /// Number of bytes written to disk.
        bytes_written: u64,
    },
}

/// Orchestrates template ingestion (load) and rendering-to-commit (create).
///
/// `TemplateService` is generic over its three ports: the repository
/// (`R: ReadRepository + WriteRepository`), the filesystem writer
/// (`W: FileWriter`), and the rendering engine (`E: TemplateEngine`). The
/// composition root injects concrete implementations; tests inject in-memory
/// doubles or the production engine directly.
pub struct TemplateService<R, W, E> {
    repository: R,
    writer: W,
    engine: E,
    config: TemplateConfigSpec,
}

impl<R, W, E> TemplateService<R, W, E>
where
    R: ReadRepository + WriteRepository,
    W: FileWriter,
    E: TemplateEngine,
{
    /// Creates a new `TemplateService` from the four dependencies it requires.
    ///
    /// The engine instance persists across `create()` calls — compiled
    /// templates accumulate in the engine, and re-compiling an already-
    /// registered name updates the source in place.
    #[inline]
    #[must_use]
    pub fn new(
        repository: R,
        writer: W,
        engine: E,
        config: TemplateConfigSpec,
    ) -> Self {
        Self {
            repository,
            writer,
            engine,
            config,
        }
    }

    /// Renders the requested template and either writes it to the vault or
    /// returns a preview when [`CreateInput::dry_run`] is set.
    ///
    /// Steps:
    /// 1. Look up the requested template from `templates` by name; return
    ///    [`TemplateError::NotFound`] if absent.
    /// 2. Compile every template in `templates` into the engine. Re-compiling
    ///    an already-registered name updates the source in place.
    /// 3. Render the requested template with the supplied context.
    /// 4. Resolve the output target via the artifact pipeline.
    /// 5. When `dry_run` is `false`, commit the artifact to disk through the
    ///    [`FileWriter`]; otherwise return a preview without writing.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError::NotFound`] when the template name is not in
    /// `templates`, [`TemplateError::Engine`] when compilation or rendering
    /// fails, and [`TemplateError::Artifact`] when target validation or the
    /// filesystem commit fails.
    #[inline]
    pub fn create(
        &mut self,
        templates: &HashMap<TemplateName, Template>,
        input: &CreateInput,
    ) -> Result<CreateTemplateOutcome, TemplateError> {
        let template = templates.get(&input.name).ok_or_else(|| {
            TemplateError::NotFound {
                name: input.name.as_str().to_owned(),
            }
        })?;

        for entry in templates.values() {
            self.engine.compile(entry).map_err(TemplateError::Engine)?;
        }

        let rendered_text = self
            .engine
            .render(template, &input.context)
            .map_err(TemplateError::Engine)?;
        let artifact =
            TemplateArtifact::rendered(template.name().clone(), rendered_text);

        let resolved = artifact
            .try_resolve_target(&input.output_path)
            .map_err(TemplateError::Artifact)?;

        if input.dry_run {
            let output_path = resolved.target_path().clone();
            let rendered = RenderedTemplate(resolved.into_content());
            return Ok(CreateTemplateOutcome::Preview {
                output_path,
                rendered,
            });
        }

        let content_len = resolved.content_len();
        let committed =
            resolved.commit(&self.writer).map_err(TemplateError::Artifact)?;
        let output_path = committed.committed_path().clone();

        Ok(CreateTemplateOutcome::Created {
            output_path,
            bytes_written: content_len,
        })
    }

    /// Loads all templates from the configured template directory.
    ///
    /// Runs the [`TemplateProcessor`] pipeline for each discovered file,
    /// persists the resulting [`Template`] aggregates and `RawTemplateView`s
    /// via the repository, and removes any cached templates/views whose
    /// source files were deleted from disk since the previous scan.
    ///
    /// Returns a `HashMap` keyed by [`TemplateName`] so downstream callers
    /// (notably [`create`](Self::create)) can look up and compile templates
    /// without re-querying the repository.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when scanning, reading, validation, or any
    /// repository operation fails during ingestion.
    #[inline]
    pub fn load(
        &self,
    ) -> Result<HashMap<TemplateName, Template>, TemplateError> {
        let scanned = Self::scan_templates(&self.config)?;
        let paths: Vec<PathKey> =
            scanned.iter().map(|entry| entry.path_key.clone()).collect();

        let discovered =
            Self::check_batch_existence(&self.repository, scanned)?;
        let deleted_paths =
            Self::identify_deleted_template_paths(&self.repository, &paths)?;

        let file_reader =
            trace_fs::reader::FileReader::new(self.config.root().as_path());
        let template_root = self.config.to_dir_path().map_err(|e| {
            TemplateError::Path(crate::error::TemplatePathError::from(e))
        })?;

        let mut results: HashMap<TemplateName, Template> =
            HashMap::with_capacity(discovered.len());
        for discovered_template in discovered {
            let template =
                TemplateProcessor::<Init, Discovered>::new(discovered_template)
                    .run(
                        &self.repository,
                        &file_reader,
                        template_root.as_path(),
                    )?
                    .into_template();
            results.insert(template.name().clone(), template);
        }

        if !deleted_paths.is_empty() {
            self.repository
                .delete_many_templates(&deleted_paths)
                .map_err(TemplateError::Repository)?;
        }

        Ok(results)
    }

    /// Identifies cached raw template paths that were not discovered on disk.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when raw-view cache path listing fails.
    fn identify_deleted_template_paths(
        repository: &R,
        discovered_paths: &[PathKey],
    ) -> Result<Vec<PathKey>, TemplateError> {
        let discovered = discovered_paths.iter().collect::<HashSet<_>>();
        let cached_paths = repository
            .list_template_path_keys()
            .map_err(TemplateError::Repository)?;

        Ok(cached_paths
            .into_iter()
            .filter(|path| !discovered.contains(path))
            .collect())
    }

    /// Fetches cached template IDs and raw views for discovered paths.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when repository reads fail or when the
    /// raw-view batch result violates the repository same-length contract.
    fn check_batch_existence(
        repository: &R,
        scanned: Vec<ScannedTemplate>,
    ) -> Result<Vec<DiscoveredTemplate>, TemplateError> {
        let paths: Vec<PathKey> =
            scanned.iter().map(|entry| entry.path_key.clone()).collect();
        let views = repository
            .find_raw_template_views_by_paths(&paths)
            .map_err(TemplateError::Repository)?;

        if views.len() != paths.len() {
            return Err(TemplateError::Repository(
                TemplateRepositoryError::Storage(DbError::Corruption(format!(
                    "raw template view batch length mismatch: expected {}, \
                     got {}",
                    paths.len(),
                    views.len()
                ))),
            ));
        }

        let mut results = Vec::with_capacity(scanned.len());
        for (entry, view) in scanned.into_iter().zip(views) {
            let id = repository
                .find_template_id_by_path(&entry.path_key)
                .map_err(TemplateError::Repository)?;
            let discovered = match (id, view) {
                (None, None) => {
                    DiscoveredTemplate::new_missing(entry.file, entry.path_key)
                }
                (Some(id), Some(view)) => DiscoveredTemplate::new_present(
                    entry.file,
                    entry.path_key,
                    id,
                    view,
                ),
                (Some(id), None) => DiscoveredTemplate::new_corrupt(
                    entry.file,
                    entry.path_key,
                    id,
                    None,
                ),
                (None, Some(_)) => {
                    return Err(TemplateError::Repository(
                        TemplateRepositoryError::Storage(DbError::Corruption(
                            format!(
                                "raw template view exists without template id \
                                 for path {}",
                                entry.path_key.as_str()
                            ),
                        )),
                    ));
                }
            };
            results.push(discovered);
        }

        Ok(results)
    }

    /// Scans the configured template directory for markdown template files.
    fn scan_templates(
        config: &TemplateConfigSpec,
    ) -> Result<Vec<ScannedTemplate>, TemplateError> {
        let scanner = DirScanner::new(config.to_dir_path().map_err(|e| {
            TemplateError::Path(crate::error::TemplatePathError::from(e))
        })?);

        let input = trace_fs::scanner::DirScanInput::new()
            .with_extensions(&["md"])
            .include_dirs(false)
            .recursive(true);

        let mut results = Vec::new();
        let entries = scanner.entries(input).map_err(|e| {
            TemplateError::Scan(crate::error::TemplateDirScanError::from(e))
        })?;

        for node in entries {
            if let trace_fs::entry::FsNode::File(file) = node {
                let path_key =
                    file.path().as_key(config.root()).map_err(|e| {
                        TemplateError::Path(
                            crate::error::TemplatePathError::from(e),
                        )
                    })?;
                results.push(ScannedTemplate {
                    file,
                    path_key,
                });
            }
        }

        Ok(results)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;
    use trace_fs::path::{DirPath, RelativeDirPath};

    use super::*;
    use crate::{
        engine::mini_jinja::MiniJinjaEngine,
        storage::testing::InMemoryRepository,
    };

    /// Builds a `TemplateService` wired with the production engine and writer.
    ///
    /// Tests that don't exercise engine compile/render still benefit from the
    /// real engine, since the load path does not touch the engine at all.
    fn service_for(
        temp_dir: &TempDir,
        config: TemplateConfigSpec,
        repo: InMemoryRepository,
    ) -> TemplateService<InMemoryRepository, trace_fs::FsWriter, MiniJinjaEngine>
    {
        let writer = trace_fs::FsWriter::new(temp_dir.path());
        TemplateService::new(
            repo,
            writer,
            MiniJinjaEngine::configured(),
            config,
        )
    }

    mod fixtures {
        use std::time::SystemTime;

        use trace_db::testing::{
            FailureInjector, FailurePoint as FPFake, InMemoryDbError,
        };
        use trace_fs::scanner::{DirScanInput, DirScanner};
        use trace_support::{Blake3Hash, HashInput};

        use super::*;
        use crate::{
            aggregate::{TemplateBody, TemplateId, TemplateName},
            views::RawTemplateView,
        };

        pub fn empty_temp_dir() -> TempDir {
            TempDir::new().unwrap()
        }

        pub fn config_for_dir(temp_dir: &TempDir) -> TemplateConfigSpec {
            let templates_path = temp_dir.path().join("templates");
            std::fs::create_dir_all(&templates_path).unwrap();

            let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();
            let relative = RelativeDirPath::try_new("templates").unwrap();
            TemplateConfigSpec::new(root, relative)
        }

        pub fn write_template_file(
            temp_dir: &TempDir,
            name: &str,
            content: &str,
        ) {
            std::fs::write(
                temp_dir.path().join("templates").join(name),
                content,
            )
            .unwrap();
        }

        pub fn path_key(name: &str) -> PathKey {
            PathKey::try_new(&format!("templates/{name}")).unwrap()
        }

        pub fn template_name(file_name: &str) -> TemplateName {
            let template_path =
                std::path::Path::new("templates").join(file_name);
            TemplateName::try_new(
                template_path.as_path(),
                std::path::Path::new("templates"),
            )
            .unwrap()
        }

        pub fn template(
            path_key: PathKey,
            file_name: &str,
            content: &str,
        ) -> Template {
            Template::new(
                TemplateId::new(),
                path_key,
                template_name(file_name),
                TemplateBody::try_new(content.to_owned()).unwrap(),
            )
        }

        pub fn scanned_metadata(
            temp_dir: &TempDir,
            file_name: &str,
        ) -> trace_fs::metadata::FileMetadata {
            let entries = DirScanner::new(temp_dir.path().join("templates"))
                .entries(DirScanInput::new())
                .unwrap();

            entries
                .into_iter()
                .filter_map(|node| match node {
                    trace_fs::entry::FsNode::File(file) => Some(file),
                    trace_fs::entry::FsNode::Dir(_) | _ => None,
                })
                .find(|file| {
                    file.path()
                        .as_ref()
                        .file_name()
                        .is_some_and(|name| name == file_name)
                })
                .map(|file| file.metadata().clone())
                .unwrap()
        }

        pub fn raw_view(
            path_key: PathKey,
            content: &str,
            metadata: trace_fs::metadata::FileMetadata,
        ) -> RawTemplateView {
            RawTemplateView::new(
                path_key,
                Blake3Hash::compute(HashInput::Text(content.to_owned())),
                metadata,
                SystemTime::now(),
            )
        }

        pub fn scanned_template_for_path(path_key: PathKey) -> ScannedTemplate {
            let temp_dir = TempDir::new().unwrap();
            let file_name = path_key
                .as_str()
                .rsplit('/')
                .next()
                .expect("path has file name");
            let file_path = temp_dir.path().join(file_name);
            std::fs::write(&file_path, "content").unwrap();
            let file_path = trace_fs::FilePath::try_new(file_path).unwrap();
            let metadata = trace_fs::metadata::FileMetadata::new(
                trace_fs::metadata::FsTimes::new(None, None),
                7,
                false,
            );
            let file = trace_fs::entry::FileNode::new(file_path, metadata);

            ScannedTemplate {
                file,
                path_key,
            }
        }

        pub fn stale_metadata(size: u64) -> trace_fs::metadata::FileMetadata {
            trace_fs::metadata::FileMetadata::new(
                trace_fs::metadata::FsTimes::new(None, None),
                size,
                false,
            )
        }

        pub struct FailOnWrite;

        impl FailureInjector for FailOnWrite {
            fn fail_at(&self, point: FPFake) -> Result<(), InMemoryDbError> {
                if point != FPFake::BeforeWrite {
                    return Ok(());
                }

                Err(InMemoryDbError::InjectedFailure {
                    point,
                    reason: "write injection".into(),
                })
            }
        }
    }

    mod check_batch_existence {
        use std::time::SystemTime;

        use pretty_assertions::assert_eq;
        use trace_support::{Blake3Hash, HashInput};

        use super::*;
        use crate::{
            aggregate::{TemplateBody, TemplateId, TemplateName},
            error::TemplateRepositoryError,
            processor::{
                Corrupted, DiscoveredCacheState, DiscoveredTemplate, Missing,
                Present,
            },
            views::RawTemplateView,
        };

        struct ShortBatchRepository;

        impl ReadRepository for ShortBatchRepository {
            fn find_template_by_id(
                &self,
                _id: TemplateId,
            ) -> Result<Option<Template>, TemplateRepositoryError> {
                Ok(None)
            }

            fn find_template_by_name(
                &self,
                _name: &TemplateName,
            ) -> Result<Option<Template>, TemplateRepositoryError> {
                Ok(None)
            }

            fn find_template_id_by_path(
                &self,
                _path: &PathKey,
            ) -> Result<Option<TemplateId>, TemplateRepositoryError>
            {
                Ok(None)
            }

            fn find_template_ids_by_paths(
                &self,
                paths: &[PathKey],
            ) -> Result<Vec<Option<TemplateId>>, TemplateRepositoryError>
            {
                Ok(vec![None; paths.len()])
            }

            fn find_template_by_path(
                &self,
                _path: &PathKey,
            ) -> Result<Option<Template>, TemplateRepositoryError> {
                Ok(None)
            }

            fn list_templates(
                &self,
            ) -> Result<Vec<Template>, TemplateRepositoryError> {
                Ok(Vec::new())
            }

            fn find_raw_template_view(
                &self,
                _path: &PathKey,
            ) -> Result<Option<RawTemplateView>, TemplateRepositoryError>
            {
                Ok(None)
            }

            fn list_template_path_keys(
                &self,
            ) -> Result<Vec<PathKey>, TemplateRepositoryError> {
                Ok(Vec::new())
            }

            fn find_raw_template_views_by_paths(
                &self,
                _paths: &[PathKey],
            ) -> Result<Vec<Option<RawTemplateView>>, TemplateRepositoryError>
            {
                Ok(Vec::new())
            }
        }

        impl WriteRepository for ShortBatchRepository {
            fn save_template(
                &self,
                _template: &Template,
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }

            fn delete_template(
                &self,
                _id: TemplateId,
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }

            fn save_raw_template_view(
                &self,
                _view: &RawTemplateView,
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }

            fn delete_raw_template_view(
                &self,
                _path: &PathKey,
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }

            fn save_many_raw_template_views(
                &self,
                _views: &[RawTemplateView],
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }

            fn delete_many_templates(
                &self,
                _paths: &[PathKey],
            ) -> Result<(), TemplateRepositoryError> {
                Ok(())
            }
        }

        #[test]
        fn returns_none_for_missing_paths() {
            let repo = InMemoryRepository::new();
            let path = PathKey::try_new("templates/missing.md").unwrap();
            let scanned = fixtures::scanned_template_for_path(path.clone());

            let results =
                TemplateService::<
                    InMemoryRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::check_batch_existence(&repo, vec![scanned]);
            assert!(results.is_ok(), "Expected success, got: {results:?}");
            let results = results.expect("batch existence should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert!(matches!(
                result.map(DiscoveredTemplate::cache),
                Some(DiscoveredCacheState::New(Missing))
            ));
        }

        #[test]
        fn returns_views_and_ids_for_existing_paths() {
            let repo = InMemoryRepository::new();
            let path_key = PathKey::try_new("templates/test.md").unwrap();

            let template = Template::new(
                TemplateId::new(),
                path_key.clone(),
                TemplateName::try_new(
                    std::path::Path::new("templates/test.md"),
                    std::path::Path::new("templates"),
                )
                .unwrap(),
                TemplateBody::try_new("content".to_owned()).unwrap(),
            );
            repo.save_template(&template).unwrap();

            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                trace_fs::metadata::FileMetadata::new(
                    trace_fs::metadata::FsTimes::new(None, None),
                    7,
                    false,
                ),
                SystemTime::now(),
            );
            repo.save_raw_template_view(&view).unwrap();

            let scanned = fixtures::scanned_template_for_path(path_key.clone());
            let results =
                TemplateService::<
                    InMemoryRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::check_batch_existence(&repo, vec![scanned]);
            assert!(results.is_ok(), "Expected success, got: {results:?}");
            let results = results.expect("batch existence should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert!(matches!(
                result.map(DiscoveredTemplate::cache),
                Some(DiscoveredCacheState::Exists(Present { id, view: cached_view }))
                    if *id == *template.id() && cached_view.path().as_str() == "templates/test.md"
            ));
        }

        #[test]
        fn marks_id_without_view_as_recoverable_corrupt_cache() {
            let repo = InMemoryRepository::new();
            let path_key = PathKey::try_new("templates/corrupt.md").unwrap();
            let template = Template::new(
                TemplateId::new(),
                path_key.clone(),
                TemplateName::try_new(
                    std::path::Path::new("templates/corrupt.md"),
                    std::path::Path::new("templates"),
                )
                .unwrap(),
                TemplateBody::try_new("content".to_owned()).unwrap(),
            );
            repo.save_template(&template).unwrap();

            let scanned = fixtures::scanned_template_for_path(path_key);
            let results =
                TemplateService::<
                    InMemoryRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::check_batch_existence(&repo, vec![scanned]);
            assert!(results.is_ok(), "Expected success, got: {results:?}");
            let results = results.expect("batch existence should succeed");
            let result = results.first();

            assert!(matches!(
                result.map(DiscoveredTemplate::cache),
                Some(DiscoveredCacheState::Corrupt(Corrupted { id, view: None }))
                    if *id == *template.id()
            ));
        }

        #[test]
        fn returns_corruption_when_view_exists_without_template_id() {
            let repo = InMemoryRepository::new();
            let path_key =
                PathKey::try_new("templates/orphan-view.md").unwrap();
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                trace_fs::metadata::FileMetadata::new(
                    trace_fs::metadata::FsTimes::new(None, None),
                    7,
                    false,
                ),
                SystemTime::now(),
            );
            repo.save_raw_template_view(&view).unwrap();

            let scanned = fixtures::scanned_template_for_path(path_key);
            let result =
                TemplateService::<
                    InMemoryRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::check_batch_existence(&repo, vec![scanned]);

            assert!(matches!(
                result,
                Err(TemplateError::Repository(
                    TemplateRepositoryError::Storage(_)
                ))
            ));
        }

        #[test]
        fn returns_error_when_batch_view_count_differs_from_paths() {
            let repo = ShortBatchRepository;
            let path = PathKey::try_new("templates/missing.md").unwrap();
            let scanned = fixtures::scanned_template_for_path(path);

            let result =
                TemplateService::<
                    ShortBatchRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::check_batch_existence(&repo, vec![scanned]);

            assert!(matches!(
                result,
                Err(TemplateError::Repository(
                    TemplateRepositoryError::Storage(_)
                ))
            ));
        }
    }

    mod construction {
        use super::*;

        #[test]
        fn new_constructs_with_all_fields() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();
            let _service = service_for(&temp_dir, config, repo);
        }
    }

    mod load {
        use pretty_assertions::assert_eq;
        use trace_support::{Blake3Hash, HasContentHash, HashInput};

        use super::*;

        #[test]
        fn returns_empty_map_when_template_directory_is_empty() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();
            let service = service_for(&temp_dir, config, repo);

            let result = service.load();
            assert!(result.is_ok(), "Expected load success, got: {result:?}");
            let map = result.expect("load should succeed");
            assert!(map.is_empty(), "Expected empty map from empty directory");
        }

        #[test]
        fn processes_only_markdown_files_when_other_extensions_exist() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();

            fixtures::write_template_file(
                &temp_dir,
                "new_template.md",
                "test content",
            );
            fixtures::write_template_file(&temp_dir, "ignored.txt", "ignored");

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            assert_eq!(map.len(), 1);
            let name = fixtures::template_name("new_template.md");
            let template = map.get(&name).expect("template by name");
            assert_eq!(template.name().as_str(), "new_template");
            assert_eq!(template.body().as_str(), "test content");

            let views = service.repository.list_templates().unwrap();
            assert_eq!(views.len(), 1);
        }

        #[test]
        fn fetches_existing_templates_without_repository_writes_when_fresh() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            let content = "fresh content";
            fixtures::write_template_file(&temp_dir, "fresh.md", content);

            let repo = InMemoryRepository::new();
            let path_key = fixtures::path_key("fresh.md");

            let template =
                fixtures::template(path_key.clone(), "fresh.md", content);
            repo.save_template(&template).unwrap();

            let view = fixtures::raw_view(
                path_key.clone(),
                content,
                fixtures::scanned_metadata(&temp_dir, "fresh.md"),
            );
            repo.save_raw_template_view(&view).unwrap();

            let repo =
                repo.with_failure_injector(Box::new(fixtures::FailOnWrite));

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            let loaded =
                map.get(template.name()).expect("template present in map");
            assert_eq!(map.len(), 1);
            assert_eq!(loaded.id(), template.id());
        }

        #[test]
        fn reconstructs_template_when_content_hash_changes() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            let new_content = "updated content";
            fixtures::write_template_file(&temp_dir, "stale.md", new_content);

            let repo = InMemoryRepository::new();
            let path_key = fixtures::path_key("stale.md");

            let template =
                fixtures::template(path_key.clone(), "stale.md", "old content");
            repo.save_template(&template).unwrap();

            let view = fixtures::raw_view(
                path_key.clone(),
                "old content",
                fixtures::stale_metadata(11),
            );
            repo.save_raw_template_view(&view).unwrap();

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            let loaded =
                map.get(template.name()).expect("template present in map");
            assert_eq!(map.len(), 1);
            assert_eq!(loaded.id(), template.id());
            assert_eq!(loaded.body().as_str(), new_content);

            let updated_template = service
                .repository
                .find_template_by_path(&path_key)
                .unwrap()
                .unwrap();
            let updated_view = service
                .repository
                .find_raw_template_view(&path_key)
                .unwrap()
                .unwrap();
            let expected_hash =
                Blake3Hash::compute(HashInput::Text(new_content.to_owned()));
            assert_eq!(updated_template.body().as_str(), new_content);
            assert!(updated_view.is_content_match(&expected_hash));
        }

        #[test]
        fn syncs_view_metadata_when_content_hash_matches() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            let content = "same content";
            fixtures::write_template_file(&temp_dir, "metadata.md", content);

            let repo = InMemoryRepository::new();
            let path_key = fixtures::path_key("metadata.md");
            let template =
                fixtures::template(path_key.clone(), "metadata.md", content);
            repo.save_template(&template).unwrap();

            let view = fixtures::raw_view(
                path_key.clone(),
                content,
                fixtures::stale_metadata(0),
            );
            repo.save_raw_template_view(&view).unwrap();
            repo.harness().counters().reset();

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            let updated_view = service
                .repository
                .find_raw_template_view(&path_key)
                .unwrap()
                .unwrap();
            let snapshot = service.repository.harness().counters().snapshot();
            let loaded =
                map.get(template.name()).expect("template present in map");
            let expected_size = u64::try_from(content.len()).unwrap();

            assert_eq!(map.len(), 1);
            assert_eq!(loaded.id(), template.id());
            assert_eq!(updated_view.metadata().size(), expected_size);
            assert_eq!(snapshot.writes, 1);
        }

        #[test]
        fn deletes_repository_state_when_cached_template_is_missing_from_disk()
        {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            let repo = InMemoryRepository::new();
            let path_key = fixtures::path_key("deleted.md");
            let template =
                fixtures::template(path_key.clone(), "deleted.md", "content");
            repo.save_template(&template).unwrap();
            repo.save_raw_template_view(&fixtures::raw_view(
                path_key.clone(),
                "content",
                fixtures::stale_metadata(7),
            ))
            .unwrap();

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            let remaining_template =
                service.repository.find_template_by_path(&path_key).unwrap();
            let remaining_view =
                service.repository.find_raw_template_view(&path_key).unwrap();

            assert!(map.is_empty());
            assert!(
                remaining_template.is_none(),
                "expected orphaned template to be deleted"
            );
            assert!(
                remaining_view.is_none(),
                "expected orphaned raw view to be deleted"
            );
        }

        #[test]
        fn deletes_only_orphaned_paths_when_one_template_remains() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            // Active file on disk.
            fixtures::write_template_file(&temp_dir, "active.md", "active");

            let repo = InMemoryRepository::new();
            // Pre-existing cache entry for a file that no longer exists.
            let orphan_path = fixtures::path_key("orphan.md");
            let orphan_template = fixtures::template(
                orphan_path.clone(),
                "orphan.md",
                "orphan content",
            );
            repo.save_template(&orphan_template).unwrap();
            repo.save_raw_template_view(&fixtures::raw_view(
                orphan_path.clone(),
                "orphan content",
                fixtures::stale_metadata(14),
            ))
            .unwrap();

            let service = service_for(&temp_dir, config, repo);
            let map = service.load().expect("load should succeed");

            assert_eq!(map.len(), 1, "only the active template should remain");
            let active_name = fixtures::template_name("active.md");
            assert!(map.contains_key(&active_name));

            assert!(
                service
                    .repository
                    .find_template_by_path(&orphan_path)
                    .unwrap()
                    .is_none(),
                "expected orphan template aggregate to be deleted"
            );
            assert!(
                service
                    .repository
                    .find_raw_template_view(&orphan_path)
                    .unwrap()
                    .is_none(),
                "expected orphan raw view to be deleted"
            );
        }

        #[test]
        fn returns_loaded_map_with_template_name_keys() {
            // Sanity: the map returned by load() is keyed by TemplateName so
            // create() can look up templates by name without re-querying the
            // repository.
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            fixtures::write_template_file(
                &temp_dir,
                "greeting.md",
                "Hello {{ name }}",
            );

            let service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let map = service.load().expect("load");
            let name = fixtures::template_name("greeting.md");
            assert!(map.contains_key(&name));
        }

        #[test]
        fn identifies_deleted_paths_from_cached_raw_views() {
            let repo = InMemoryRepository::new();
            let path_key = fixtures::path_key("orphan-view.md");
            repo.save_raw_template_view(&fixtures::raw_view(
                path_key.clone(),
                "content",
                fixtures::stale_metadata(7),
            ))
            .unwrap();

            let result =
                TemplateService::<
                    InMemoryRepository,
                    trace_fs::FsWriter,
                    MiniJinjaEngine,
                >::identify_deleted_template_paths(&repo, &[]);
            assert!(result.is_ok(), "Expected success, got: {result:?}");
            let result = result.expect("deleted path detection should succeed");

            assert_eq!(result, vec![path_key]);
        }
    }

    mod create {
        use pretty_assertions::assert_eq;
        use trace_fs::error::{WriteError, WriteTargetError};

        use super::*;
        use crate::error::TemplateArtifactError;

        /// Helper that builds a `templates` map with one template.
        fn template_map(
            name: &str,
            body: &str,
        ) -> HashMap<TemplateName, Template> {
            let path_key = fixtures::path_key(&format!("{name}.md"));
            let template =
                fixtures::template(path_key, &format!("{name}.md"), body);
            let mut map = HashMap::new();
            map.insert(template.name().clone(), template);
            map
        }

        fn create_input(
            name: &str,
            output_path: &str,
            dry_run: bool,
        ) -> CreateInput {
            CreateInput {
                name: fixtures::template_name(&format!("{name}.md")),
                output_path: output_path.to_owned(),
                context: HashMap::new(),
                dry_run,
            }
        }

        #[test]
        fn returns_not_found_when_template_name_missing_from_map() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates: HashMap<TemplateName, Template> = HashMap::new();

            let result = service.create(
                &templates,
                &create_input("missing", "out/x.md", false),
            );

            assert!(matches!(
                result,
                Err(TemplateError::NotFound { ref name })
                    if name == "missing"
            ));
        }

        #[test]
        fn renders_and_commits_file_to_disk() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());

            let templates = template_map("greeting", "Hello {{ name }}");
            let mut context = HashMap::new();
            context.insert("name".to_owned(), "Alice".to_owned());

            let outcome = service
                .create(&templates, &CreateInput {
                    name: fixtures::template_name("greeting.md"),
                    output_path: "notes/out.md".to_owned(),
                    context,
                    dry_run: false,
                })
                .expect("create should succeed");

            assert!(
                matches!(
                    &outcome,
                    CreateTemplateOutcome::Created {
                        output_path,
                        bytes_written,
                    }
                        if output_path.as_path().to_str() == Some("notes/out.md")
                            && *bytes_written == u64::try_from("Hello Alice".len()).unwrap()
                ),
                "expected Created outcome with correct path and length, got: \
                 {outcome:?}"
            );

            let written = temp_dir.path().join("notes/out.md");
            assert!(written.exists(), "expected file written to disk");
            assert_eq!(
                std::fs::read_to_string(&written).expect("read committed"),
                "Hello Alice"
            );
        }

        #[test]
        fn dry_run_returns_preview_without_writing_file() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());

            let templates = template_map("greeting", "Hello {{ name }}");
            let mut context = HashMap::new();
            context.insert("name".to_owned(), "Alice".to_owned());

            let outcome = service
                .create(&templates, &CreateInput {
                    name: fixtures::template_name("greeting.md"),
                    output_path: "notes/preview.md".to_owned(),
                    context,
                    dry_run: true,
                })
                .expect("dry run should succeed");

            assert!(
                matches!(
                    &outcome,
                    CreateTemplateOutcome::Preview {
                        output_path,
                        rendered,
                    }
                        if output_path.as_path().to_str() == Some("notes/preview.md")
                            && rendered.as_str() == "Hello Alice"
                ),
                "expected Preview outcome with correct path and content, got: \
                 {outcome:?}"
            );

            let written = temp_dir.path().join("notes/preview.md");
            assert!(
                !written.exists(),
                "dry run must not write the file to disk"
            );
        }

        #[test]
        fn propagates_engine_error_when_template_source_is_invalid() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());

            // Broken Jinja syntax: unterminated `{{ name`.
            let templates = template_map("broken", "Hello {{ name");

            let result = service.create(
                &templates,
                &create_input("broken", "notes/x.md", false),
            );

            assert!(matches!(result, Err(TemplateError::Engine(_))));
        }

        #[test]
        fn rejects_absolute_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service.create(
                &templates,
                &create_input("greeting", "/abs/x.md", false),
            );

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Path(
                    WriteTargetError::Absolute(_)
                )))
            ));
        }

        #[test]
        fn rejects_traversal_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service.create(
                &templates,
                &create_input("greeting", "../escape.md", false),
            );

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Path(
                    WriteTargetError::Traversal(_)
                )))
            ));
        }

        #[test]
        fn rejects_hidden_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service.create(
                &templates,
                &create_input("greeting", ".hidden/x.md", false),
            );

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Path(
                    WriteTargetError::Hidden(_)
                )))
            ));
        }

        #[test]
        fn rejects_empty_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service
                .create(&templates, &create_input("greeting", "", false));

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Path(
                    WriteTargetError::Empty
                )))
            ));
        }

        #[test]
        fn rejects_current_dir_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service
                .create(&templates, &create_input("greeting", "./x.md", false));

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Path(
                    WriteTargetError::CurrentDir(_)
                )))
            ));
        }

        #[test]
        fn rejects_existing_destination_file() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);

            // Pre-create the destination so create_new fails atomically.
            std::fs::create_dir_all(temp_dir.path().join("notes"))
                .expect("create parent");
            std::fs::write(temp_dir.path().join("notes/out.md"), b"existing")
                .expect("seed destination file");

            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let templates = template_map("greeting", "Hello");

            let result = service.create(
                &templates,
                &create_input("greeting", "notes/out.md", false),
            );

            assert!(matches!(
                result,
                Err(TemplateError::Artifact(TemplateArtifactError::Write(
                    WriteError::AlreadyExists { .. }
                )))
            ));
            assert_eq!(
                std::fs::read_to_string(temp_dir.path().join("notes/out.md"))
                    .expect("read existing"),
                "existing",
                "existing file must remain untouched"
            );
        }
    }
}
