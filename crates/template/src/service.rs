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
    aggregate::TemplateName,
    artifact::TemplateArtifact,
    engine::{RenderedTemplate, TemplateEngine},
    error::{TemplateError, TemplateRepositoryError},
    processor::{
        Discovered, DiscoveredTemplate, Init, ProcessOutcomeKind,
        TemplateProcessor,
    },
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

/// Counts produced by [`TemplateService::process_all`].
///
/// Reports how many templates were created, updated, left unchanged, or
/// deleted during a full filesystem-to-repository reconciliation. Intended for
/// observability and tests; the indexed aggregates themselves live in the
/// repository, which is the source of truth after a successful pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct ProcessSummary {
    /// Templates newly created during this pass.
    pub created: usize,
    /// Existing templates rebuilt from changed (or recovered) source.
    pub updated: usize,
    /// Cached templates that were already in sync with disk.
    pub unchanged: usize,
    /// Orphaned templates removed because their source file is gone.
    pub deleted: usize,
}

impl ProcessSummary {
    /// Increments the count matching a per-template processor outcome.
    fn record(&mut self, outcome: ProcessOutcomeKind) {
        match outcome {
            ProcessOutcomeKind::Created => {
                self.created = self.created.saturating_add(1);
            }
            ProcessOutcomeKind::Updated => {
                self.updated = self.updated.saturating_add(1);
            }
            ProcessOutcomeKind::Unchanged => {
                self.unchanged = self.unchanged.saturating_add(1);
            }
        }
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

/// Orchestrates template ingestion ([`process_all`](Self::process_all)) and
/// rendering-to-commit ([`create`](Self::create)).
///
/// `TemplateService` is generic over its three ports: the repository
/// (`R: ReadRepository + WriteRepository`), the filesystem writer
/// (`W: FileWriter`), and the rendering engine (`E: TemplateEngine`). The
/// composition root injects concrete implementations; tests inject in-memory
/// doubles or the production engine directly.
///
/// `TemplateService` is intentionally not bound `Send + Sync + 'static`. Those
/// bounds are runtime-specific (axum / tokio injection sites), not
/// hexagonal-architecture-intrinsic, and the foundation service has no async
/// surface. When a runtime needs them, the composition root adds them at the
/// injection point; the service is otherwise free of runtime assumptions.
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
    /// The engine instance persists across [`create`](Self::create) calls —
    /// compiled templates accumulate in the engine, and re-compiling an
    /// already-registered name updates the source in place.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use trace_template::TemplateService;
    /// # use trace_settings::template::TemplateConfigSpec;
    /// # fn wire<R, W, E>(
    /// #     repository: R,
    /// #     writer: W,
    /// #     engine: E,
    /// #     config: TemplateConfigSpec,
    /// # ) -> TemplateService<R, W, E>
    /// # where
    /// #     R: trace_template::ReadRepository + trace_template::WriteRepository,
    /// #     W: trace_fs::FileWriter,
    /// #     E: trace_template::TemplateEngine,
    /// # {
    /// TemplateService::new(repository, writer, engine, config)
    /// # }
    /// ```
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
    /// The repository is the source of truth: `create` fetches the requested
    /// [`Template`] from the repository by name rather than accepting a
    /// caller-supplied map, so a render can never bypass the processor
    /// pipeline.
    ///
    /// Steps:
    /// 1. [`verify_path`](Self::verify_path) reprocesses the single requested
    ///    template if its source file is stale, keeping the repository in sync
    ///    with disk without walking the whole tree.
    /// 2. Fetch the requested template from the repository by name; return
    ///    [`TemplateError::NotFound`] if absent.
    /// 3. Compile **only** the requested template into the engine. Re-compiling
    ///    an already-registered name updates the source in place (`MiniJinja`'s
    ///    `add_template_owned` is idempotent on re-registration).
    /// 4. Render the requested template with the supplied context.
    /// 5. Resolve the output target via the artifact pipeline.
    /// 6. When `dry_run` is `false`, commit the artifact to disk through the
    ///    [`FileWriter`]; otherwise return a preview without writing.
    ///
    /// # Dry-run semantics
    ///
    /// When `input.dry_run` is `true`, the service renders the template and
    /// validates the output target path syntactically, but does **not** check
    /// whether the destination file already exists on disk.
    /// Destination-collision errors
    /// ([`WriteError::AlreadyExists`](trace_fs::error::WriteError::AlreadyExists))
    /// surface only at commit time. A successful preview is therefore not a
    /// guarantee that a subsequent non-dry `create` call will succeed.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError::NotFound`] when no template with the requested
    /// name is persisted, [`TemplateError::Engine`] when compilation or
    /// rendering fails, [`TemplateError::Artifact`] when target validation or
    /// the filesystem commit fails, and any ingestion error surfaced while
    /// reconciling the requested path via
    /// [`verify_path`](Self::verify_path).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::collections::HashMap;
    /// # use trace_template::{CreateInput, TemplateName, TemplateService};
    /// # fn run<R, W, E>(
    /// #     mut service: TemplateService<R, W, E>,
    /// #     name: TemplateName,
    /// # ) -> Result<(), trace_template::TemplateError>
    /// # where
    /// #     R: trace_template::ReadRepository + trace_template::WriteRepository,
    /// #     W: trace_fs::FileWriter,
    /// #     E: trace_template::TemplateEngine,
    /// # {
    /// let input = CreateInput {
    ///     name,
    ///     output_path: "notes/out.md".to_owned(),
    ///     context: HashMap::new(),
    ///     dry_run: true,
    /// };
    /// let _outcome = service.create(&input)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn create(
        &mut self,
        input: &CreateInput,
    ) -> Result<CreateTemplateOutcome, TemplateError> {
        self.verify_path(&input.name)?;

        let template = self
            .repository
            .find_template_by_name(&input.name)
            .map_err(TemplateError::Repository)?
            .ok_or_else(|| TemplateError::NotFound {
                name: input.name.clone(),
            })?;

        self.engine.compile(&template).map_err(TemplateError::Engine)?;

        let rendered = self
            .engine
            .render(&template, &input.context)
            .map_err(TemplateError::Engine)?;
        let artifact = TemplateArtifact::rendered(
            template.name().clone(),
            rendered.into_inner(),
        );

        let resolved = artifact
            .try_resolve_target(&input.output_path)
            .map_err(TemplateError::Artifact)?;

        if input.dry_run {
            let output_path = resolved.target_path().clone();
            let preview = RenderedTemplate::new(resolved.into_content());
            return Ok(CreateTemplateOutcome::Preview {
                output_path,
                rendered: preview,
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

    /// Reconciles the repository with every template file on disk.
    ///
    /// Walks the configured template directory, runs the
    /// [`TemplateProcessor`] pipeline for each discovered file, persists the
    /// resulting [`Template`] aggregates and `RawTemplateView`s via the
    /// repository, and removes any cached templates/views whose source files
    /// were deleted from disk since the previous scan.
    ///
    /// This is the indexer that makes the repository the source of truth: after
    /// a successful pass, every persisted template matches a file on disk and
    /// every deleted file's cache entry is gone. Returns a [`ProcessSummary`]
    /// describing the work done (created / updated / unchanged / deleted) for
    /// observability and tests, rather than the aggregates themselves — callers
    /// fetch templates from the repository by name.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when scanning, reading, validation, or any
    /// repository operation fails during ingestion.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use trace_template::TemplateService;
    /// # fn run<R, W, E>(
    /// #     service: TemplateService<R, W, E>,
    /// # ) -> Result<(), trace_template::TemplateError>
    /// # where
    /// #     R: trace_template::ReadRepository + trace_template::WriteRepository,
    /// #     W: trace_fs::FileWriter,
    /// #     E: trace_template::TemplateEngine,
    /// # {
    /// let summary = service.process_all()?;
    /// println!("created {} templates", summary.created);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn process_all(&self) -> Result<ProcessSummary, TemplateError> {
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

        let mut summary = ProcessSummary::default();
        for discovered_template in discovered {
            let completed =
                TemplateProcessor::<Init, Discovered>::new(discovered_template)
                    .run(
                        &self.repository,
                        &file_reader,
                        template_root.as_path(),
                    )?;
            summary.record(completed.outcome());
        }

        if !deleted_paths.is_empty() {
            self.repository
                .delete_many_templates(&deleted_paths)
                .map_err(TemplateError::Repository)?;
            summary.deleted = deleted_paths.len();
        }

        Ok(summary)
    }

    /// Reprocesses a single requested template so its repository entry is fresh
    /// before a render.
    ///
    /// This is the per-template freshness safeguard for
    /// [`create`](Self::create): rather than walking the whole tree on every
    /// render (which would bound render latency to directory size), it scans
    /// only the file backing `name`, runs the processor pipeline for that one
    /// path, and lets the pipeline's cheap metadata/hash comparison skip work
    /// when the cache is already in sync. A stale single entry self-heals; a
    /// fresh one costs only a `stat` and a hash compare.
    ///
    /// When no file on disk derives `name`, the repository is left untouched —
    /// `create` then surfaces [`TemplateError::NotFound`] from its own
    /// repository lookup.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when scanning, reading, validation, or any
    /// repository operation fails while reprocessing the requested path.
    #[inline]
    pub fn verify_path(
        &self,
        name: &TemplateName,
    ) -> Result<(), TemplateError> {
        let template_root = self.config.to_dir_path().map_err(|e| {
            TemplateError::Path(crate::error::TemplatePathError::from(e))
        })?;

        let scanned = Self::scan_templates(&self.config)?;
        let Some(matching) = scanned.into_iter().find(|entry| {
            Self::derives_name(&entry.file, template_root.as_path(), name)
        }) else {
            // No source file derives this name; leave the repository as-is and
            // let create()'s own lookup report NotFound.
            return Ok(());
        };

        let discovered =
            Self::check_batch_existence(&self.repository, vec![matching])?;

        let file_reader =
            trace_fs::reader::FileReader::new(self.config.root().as_path());
        for discovered_template in discovered {
            TemplateProcessor::<Init, Discovered>::new(discovered_template)
                .run(&self.repository, &file_reader, template_root.as_path())?;
        }

        Ok(())
    }

    /// Returns `true` when the scanned file derives the requested template
    /// name under `template_root`.
    fn derives_name(
        file: &FileNode,
        template_root: &std::path::Path,
        name: &TemplateName,
    ) -> bool {
        TemplateName::try_new(file.path().as_ref(), template_root)
            .is_ok_and(|derived| &derived == name)
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
        aggregate::Template, engine::mini_jinja::MiniJinjaEngine,
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

    mod process_all {
        use pretty_assertions::assert_eq;
        use trace_support::{Blake3Hash, HasContentHash, HashInput};

        use super::*;

        #[test]
        fn returns_empty_summary_when_template_directory_is_empty() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();
            let service = service_for(&temp_dir, config, repo);

            let result = service.process_all();
            assert!(
                result.is_ok(),
                "Expected process_all success, got: {result:?}"
            );
            let summary = result.expect("process_all should succeed");
            assert_eq!(
                summary,
                ProcessSummary::default(),
                "Expected zeroed summary from empty directory"
            );
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
            let summary = service.process_all().expect("process_all succeeds");

            assert_eq!(summary.created, 1);
            let name = fixtures::template_name("new_template.md");
            let template = service
                .repository
                .find_template_by_name(&name)
                .unwrap()
                .expect("template persisted by name");
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
            let summary = service.process_all().expect("process_all succeeds");

            let loaded = service
                .repository
                .find_template_by_name(template.name())
                .unwrap()
                .expect("template present in repository");
            assert_eq!(summary.unchanged, 1);
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
            let summary = service.process_all().expect("process_all succeeds");

            assert_eq!(summary.updated, 1);

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
            assert_eq!(updated_template.id(), template.id());
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
            let summary = service.process_all().expect("process_all succeeds");

            let updated_view = service
                .repository
                .find_raw_template_view(&path_key)
                .unwrap()
                .unwrap();
            let snapshot = service.repository.harness().counters().snapshot();
            let expected_size = u64::try_from(content.len()).unwrap();

            assert_eq!(summary.unchanged, 1);
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
            let summary = service.process_all().expect("process_all succeeds");

            let remaining_template =
                service.repository.find_template_by_path(&path_key).unwrap();
            let remaining_view =
                service.repository.find_raw_template_view(&path_key).unwrap();

            assert_eq!(summary.deleted, 1);
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
            let summary = service.process_all().expect("process_all succeeds");

            assert_eq!(
                summary.created, 1,
                "only the active template should be created"
            );
            assert_eq!(summary.deleted, 1, "the orphan should be deleted");
            let active_name = fixtures::template_name("active.md");
            assert!(
                service
                    .repository
                    .find_template_by_name(&active_name)
                    .unwrap()
                    .is_some(),
                "expected active template to be persisted"
            );

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
        fn persists_template_retrievable_by_name() {
            // Sanity: process_all persists templates so create() can look them
            // up by name from the repository (the source of truth).
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            fixtures::write_template_file(
                &temp_dir,
                "greeting.md",
                "Hello {{ name }}",
            );

            let service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            service.process_all().expect("process_all");
            let name = fixtures::template_name("greeting.md");
            assert!(
                service
                    .repository
                    .find_template_by_name(&name)
                    .unwrap()
                    .is_some(),
                "expected template retrievable by name after process_all"
            );
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

        /// Writes a template source file to disk so `create` can reconcile it
        /// into the repository via `verify_path`.
        fn seed_template(temp_dir: &TempDir, name: &str, body: &str) {
            fixtures::write_template_file(
                temp_dir,
                &format!("{name}.md"),
                body,
            );
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
        fn returns_not_found_when_template_name_missing_from_repository() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            let missing = fixtures::template_name("missing.md");

            let result =
                service.create(&create_input("missing", "out/x.md", false));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::NotFound { ref name })
                        if *name == missing
                ),
                "expected NotFound for unindexed name, got: {result:?}"
            );
        }

        #[test]
        fn renders_and_commits_file_to_disk() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());

            seed_template(&temp_dir, "greeting", "Hello {{ name }}");
            let mut context = HashMap::new();
            context.insert("name".to_owned(), "Alice".to_owned());

            let outcome = service
                .create(&CreateInput {
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

            seed_template(&temp_dir, "greeting", "Hello {{ name }}");
            let mut context = HashMap::new();
            context.insert("name".to_owned(), "Alice".to_owned());

            let outcome = service
                .create(&CreateInput {
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
            seed_template(&temp_dir, "broken", "Hello {{ name");

            let result =
                service.create(&create_input("broken", "notes/x.md", false));

            assert!(
                matches!(result, Err(TemplateError::Engine(_))),
                "expected Engine error, got: {result:?}"
            );
        }

        #[test]
        fn rejects_absolute_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            seed_template(&temp_dir, "greeting", "Hello");

            let result =
                service.create(&create_input("greeting", "/abs/x.md", false));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Path(
                        WriteTargetError::Absolute(_)
                    )))
                ),
                "expected Absolute path rejection, got: {result:?}"
            );
        }

        #[test]
        fn rejects_traversal_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            seed_template(&temp_dir, "greeting", "Hello");

            let result = service.create(&create_input(
                "greeting",
                "../escape.md",
                false,
            ));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Path(
                        WriteTargetError::Traversal(_)
                    )))
                ),
                "expected Traversal path rejection, got: {result:?}"
            );
        }

        #[test]
        fn rejects_hidden_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            seed_template(&temp_dir, "greeting", "Hello");

            let result = service.create(&create_input(
                "greeting",
                ".hidden/x.md",
                false,
            ));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Path(
                        WriteTargetError::Hidden(_)
                    )))
                ),
                "expected Hidden path rejection, got: {result:?}"
            );
        }

        #[test]
        fn rejects_empty_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            seed_template(&temp_dir, "greeting", "Hello");

            let result = service.create(&create_input("greeting", "", false));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Path(
                        WriteTargetError::Empty
                    )))
                ),
                "expected Empty path rejection, got: {result:?}"
            );
        }

        #[test]
        fn rejects_current_dir_output_path() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let mut service =
                service_for(&temp_dir, config, InMemoryRepository::new());
            seed_template(&temp_dir, "greeting", "Hello");

            let result =
                service.create(&create_input("greeting", "./x.md", false));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Path(
                        WriteTargetError::CurrentDir(_)
                    )))
                ),
                "expected CurrentDir path rejection, got: {result:?}"
            );
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
            seed_template(&temp_dir, "greeting", "Hello");

            let result = service.create(&create_input(
                "greeting",
                "notes/out.md",
                false,
            ));

            assert!(
                matches!(
                    result,
                    Err(TemplateError::Artifact(TemplateArtifactError::Write(
                        WriteError::AlreadyExists { .. }
                    )))
                ),
                "expected AlreadyExists write rejection, got: {result:?}"
            );
            assert_eq!(
                std::fs::read_to_string(temp_dir.path().join("notes/out.md"))
                    .expect("read existing"),
                "existing",
                "existing file must remain untouched"
            );
        }
    }
}
