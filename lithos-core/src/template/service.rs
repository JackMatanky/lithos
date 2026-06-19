//! Template ingestion service orchestration.
//!
//! This module coordinates the template processor pipeline with filesystem
//! discovery and repository batch reads. [`TemplateService`] owns the
//! use-case-level flow: discover markdown templates, prefetch cached template
//! views and IDs, drive each discovered file through the typestate processor,
//! and identify cached paths that are absent from the current scan for deferred
//! deletion handling.
//!
//! The service keeps file I/O behind [`crate::fs::reader::FileReader`], keeps
//! storage access behind the template repository traits, and leaves engine
//! compilation/rendering outside the ingestion path.

use std::collections::HashSet;

use crate::{
    config::template::TemplateConfigSpec,
    db::DbError,
    fs::{PathKey, entry::FileNode, scanner::DirScanner},
    template::{
        aggregate::{Template, TemplateId},
        error::{TemplateError, TemplateRepositoryError},
        processor::{
            Comparison, ContentBranch, Discovered, Discovery, DiscoveryBranch,
            MetadataBranch, Present, Suspect, TemplateProcessor,
        },
        repository::{ReadRepository, WriteRepository},
        views::RawTemplateView,
    },
};

/// Cached repository state for one discovered template path.
///
/// The first element is the persisted template identity, if an aggregate exists
/// for the path. The second element is the raw view used for cheap freshness
/// checks, if one exists for the same path.
type CacheExistence = (Option<TemplateId>, Option<RawTemplateView>);

/// Orchestrates the discovery and pipeline ingestion of templates.
pub struct TemplateService;

impl TemplateService {
    /// Creates a new `TemplateService`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Loads and processes all templates within the configured template
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when scanning, reading, validating, or
    /// repository access fails during template ingestion.
    #[inline]
    pub fn load<R: ReadRepository + WriteRepository>(
        &self,
        config: &TemplateConfigSpec,
        repository: &R,
    ) -> Result<Vec<Template>, TemplateError> {
        let scanned = Self::scan_templates(config)?;
        let paths: Vec<PathKey> =
            scanned.iter().map(|(_, p)| p.clone()).collect();

        let batch_info = Self::check_batch_existence(repository, &paths)?;
        let _deleted_paths =
            Self::identify_deleted_template_paths(repository, &paths)?;
        // TODO(issue-07): remove templates and raw views for deleted paths.

        let mut results = Vec::with_capacity(scanned.len());
        let file_reader =
            crate::fs::reader::FileReader::new(config.root().as_path());
        let template_root = config.to_dir_path().map_err(|e| {
            TemplateError::Path(
                crate::template::error::TemplatePathError::from(e),
            )
        })?;

        for ((file, path_key), (id, view)) in
            scanned.into_iter().zip(batch_info)
        {
            let processor =
                TemplateProcessor::<Discovery, Discovered>::new(file, path_key);
            let template = Self::process_branch(
                processor.compare(id, view),
                repository,
                &file_reader,
                template_root.as_path(),
            )?;
            results.push(template);
        }

        Ok(results)
    }

    /// Processes one discovery branch through the rest of the ingestion
    /// pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when file reading, template construction,
    /// metadata refresh, or repository access fails for the branch.
    fn process_branch<R: ReadRepository + WriteRepository>(
        branch: DiscoveryBranch,
        repository: &R,
        file_reader: &crate::fs::reader::FileReader,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        match branch {
            DiscoveryBranch::Missing(parsed) => {
                parsed.parse(file_reader)?.create(repository, template_root)
            }
            DiscoveryBranch::Present(comparison) => Self::process_present(
                comparison,
                repository,
                file_reader,
                template_root,
            ),
        }
    }

    /// Processes a discovered path that has both a cached template ID and raw
    /// view.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when stale-content checks, metadata refresh,
    /// aggregate fetch, or repository access fails.
    fn process_present<R: ReadRepository + WriteRepository>(
        processor: TemplateProcessor<Comparison, Present>,
        repository: &R,
        file_reader: &crate::fs::reader::FileReader,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        match processor.check_metadata() {
            MetadataBranch::Match(fresh) => fresh.fetch(repository),
            MetadataBranch::Mismatch(suspect) => Self::process_suspect(
                suspect,
                repository,
                file_reader,
                template_root,
            ),
        }
    }

    /// Processes a present template whose filesystem metadata no longer
    /// matches the cached raw view.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when reading the file, refreshing the cached
    /// view, reconstructing the aggregate, or persisting updates fails.
    fn process_suspect<R: ReadRepository + WriteRepository>(
        processor: TemplateProcessor<Comparison, Suspect>,
        repository: &R,
        file_reader: &crate::fs::reader::FileReader,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        match processor.check_content(file_reader)? {
            ContentBranch::Match(refresh) => {
                refresh.sync_metadata(repository)?.fetch(repository)
            }
            ContentBranch::Mismatch(stale) => {
                stale.parse().update(repository, template_root)
            }
        }
    }

    /// Identifies cached raw template paths that were not discovered on disk.
    ///
    /// Deletion is intentionally deferred to the follow-up deletion issue; this
    /// method only computes the cache/disk diff so the service has the correct
    /// source of truth when deletion processing is wired in.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when raw-view cache path listing fails.
    fn identify_deleted_template_paths<R: ReadRepository>(
        repository: &R,
        discovered_paths: &[PathKey],
    ) -> Result<Vec<PathKey>, TemplateError> {
        let discovered = discovered_paths.iter().collect::<HashSet<_>>();
        let cached_paths = repository
            .list_raw_template_view_paths()
            .map_err(TemplateError::Repository)?;

        Ok(cached_paths
            .into_iter()
            .filter(|path| !discovered.contains(path))
            .collect())
    }

    #[cfg_attr(test, allow(dead_code, reason = "test-only access"))]
    /// Fetches cached template IDs and raw views for discovered paths.
    ///
    /// Raw views are read with the repository batch method. Template IDs still
    /// use per-path lookup until a dedicated batch ID lookup exists.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when repository reads fail or when the
    /// raw-view batch result violates the repository same-length contract.
    fn check_batch_existence<R: ReadRepository>(
        repository: &R,
        paths: &[PathKey],
    ) -> Result<Vec<CacheExistence>, TemplateError> {
        let views = repository
            .find_raw_template_views_by_paths(paths)
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

        let mut results = Vec::with_capacity(paths.len());
        for (path, view) in paths.iter().zip(views) {
            let id = repository
                .find_template_id_by_path(path)
                .map_err(TemplateError::Repository)?;
            results.push((id, view));
        }

        Ok(results)
    }

    /// Scans the configured template directory for markdown template files.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError`] when the configured directory cannot be
    /// resolved, scanning fails, or a discovered path cannot be converted to a
    /// vault-relative storage key.
    fn scan_templates(
        config: &TemplateConfigSpec,
    ) -> Result<Vec<(FileNode, PathKey)>, TemplateError> {
        let scanner = DirScanner::new(config.to_dir_path().map_err(|e| {
            TemplateError::Path(
                crate::template::error::TemplatePathError::from(e),
            )
        })?);

        let input = crate::fs::scanner::DirScanInput::new()
            .with_extensions(&["md"])
            .recursive(true);

        let mut results = Vec::new();
        let entries = scanner.entries(input).map_err(|e| {
            TemplateError::Scan(
                crate::template::error::TemplateDirScanError::from(e),
            )
        })?;

        for node in entries {
            if let crate::fs::entry::FsNode::File(file) = node {
                let path_key =
                    file.path().as_key(config.root()).map_err(|e| {
                        TemplateError::Path(
                            crate::template::error::TemplatePathError::from(e),
                        )
                    })?;
                results.push((file, path_key));
            }
        }

        Ok(results)
    }
}

impl Default for TemplateService {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::path::{DirPath, RelativeDirPath},
        template::storage::testing::InMemoryRepository,
    };

    mod fixtures {
        use std::time::SystemTime;

        use super::*;
        use crate::{
            db::testing::{
                FailureInjector, FailurePoint as FPFake, InMemoryDbError,
            },
            fs::scanner::{DirScanInput, DirScanner},
            support::content_hash::{Blake3Hash, HashInput},
            template::{
                aggregate::{TemplateBody, TemplateId, TemplateName},
                views::RawTemplateView,
            },
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

        pub fn template(
            path_key: PathKey,
            file_name: &str,
            content: &str,
        ) -> Template {
            let template_path =
                std::path::Path::new("templates").join(file_name);
            Template::new(
                TemplateId::new(),
                path_key,
                TemplateName::try_new(
                    template_path.as_path(),
                    std::path::Path::new("templates"),
                )
                .unwrap(),
                TemplateBody::try_new(content.to_owned()).unwrap(),
            )
        }

        pub fn scanned_metadata(
            temp_dir: &TempDir,
            file_name: &str,
        ) -> crate::fs::metadata::FileMetadata {
            let entries = DirScanner::new(temp_dir.path().join("templates"))
                .entries(DirScanInput::new())
                .unwrap();

            entries
                .into_iter()
                .filter_map(|node| match node {
                    crate::fs::entry::FsNode::File(file) => Some(file),
                    crate::fs::entry::FsNode::Dir(_) => None,
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
            metadata: crate::fs::metadata::FileMetadata,
        ) -> RawTemplateView {
            RawTemplateView::new(
                path_key,
                Blake3Hash::compute(HashInput::Text(content.to_owned())),
                metadata,
                SystemTime::now(),
            )
        }

        pub fn stale_metadata(size: u64) -> crate::fs::metadata::FileMetadata {
            crate::fs::metadata::FileMetadata::new(
                crate::fs::metadata::FsTimes::new(None, None),
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

        use super::*;
        use crate::{
            support::content_hash::{Blake3Hash, HashInput},
            template::{
                aggregate::{TemplateBody, TemplateId, TemplateName},
                error::TemplateRepositoryError,
                views::RawTemplateView,
            },
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

            fn list_raw_template_view_paths(
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

        #[test]
        fn returns_none_for_missing_paths() {
            let repo = InMemoryRepository::new();
            let path = PathKey::try_new("templates/missing.md").unwrap();

            let results =
                TemplateService::check_batch_existence(&repo, &[path]);
            assert!(results.is_ok(), "Expected success, got: {results:?}");
            let results = results.expect("batch existence should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert!(result.is_some_and(|r| r.0.is_none()));
            assert!(result.is_some_and(|r| r.1.is_none()));
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
                crate::fs::metadata::FileMetadata::new(
                    crate::fs::metadata::FsTimes::new(None, None),
                    7,
                    false,
                ),
                SystemTime::now(),
            );
            repo.save_raw_template_view(&view).unwrap();

            let results =
                TemplateService::check_batch_existence(&repo, &[path_key]);
            assert!(results.is_ok(), "Expected success, got: {results:?}");
            let results = results.expect("batch existence should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert_eq!(result.map(|r| r.0), Some(Some(*template.id())));
            assert_eq!(
                result.and_then(|r| r.1.as_ref().map(|v| v.path().as_str())),
                Some("templates/test.md")
            );
        }

        #[test]
        fn returns_error_when_batch_view_count_differs_from_paths() {
            let repo = ShortBatchRepository;
            let path = PathKey::try_new("templates/missing.md").unwrap();

            let result = TemplateService::check_batch_existence(&repo, &[path]);

            assert!(matches!(
                result,
                Err(TemplateError::Repository(
                    TemplateRepositoryError::Storage(_)
                ))
            ));
        }
    }

    mod load {
        use pretty_assertions::assert_eq;

        use super::*;
        use crate::support::content_hash::{Blake3Hash, HashInput};

        #[test]
        fn returns_empty_list_when_template_directory_is_empty() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();
            let service = TemplateService::new();

            let result = service.load(&config, &repo);
            assert!(result.is_ok(), "Expected load success, got: {result:?}");
            let result = result.expect("load should succeed");
            assert!(
                result.is_empty(),
                "Expected empty list from empty directory"
            );
        }

        #[test]
        fn processes_only_markdown_files_when_other_extensions_exist() {
            let temp_dir = fixtures::empty_temp_dir();
            let config = fixtures::config_for_dir(&temp_dir);
            let repo = InMemoryRepository::new();
            let service = TemplateService::new();

            fixtures::write_template_file(
                &temp_dir,
                "new_template.md",
                "test content",
            );
            fixtures::write_template_file(&temp_dir, "ignored.txt", "ignored");

            let results = service.load(&config, &repo);
            assert!(results.is_ok(), "Expected load success, got: {results:?}");
            let results = results.expect("load should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert_eq!(result.map(|r| r.name().as_str()), Some("new_template"));
            assert_eq!(result.map(|r| r.body().as_str()), Some("test content"));

            let views = repo.list_templates().unwrap();
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

            let service = TemplateService::new();
            let results = service.load(&config, &repo);
            assert!(results.is_ok(), "Expected load success, got: {results:?}");
            let results = results.expect("load should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert_eq!(result.map(Template::id), Some(template.id()));
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

            let service = TemplateService::new();
            let results = service.load(&config, &repo);
            assert!(results.is_ok(), "Expected load success, got: {results:?}");
            let results = results.expect("load should succeed");
            let result = results.first();

            assert_eq!(results.len(), 1);
            assert_eq!(result.map(Template::id), Some(template.id()));
            assert_eq!(result.map(|r| r.body().as_str()), Some(new_content));

            let updated_template =
                repo.find_template_by_path(&path_key).unwrap().unwrap();
            let updated_view =
                repo.find_raw_template_view(&path_key).unwrap().unwrap();
            let expected_hash =
                Blake3Hash::compute(HashInput::Text(new_content.to_owned()));
            assert_eq!(updated_template.body().as_str(), new_content);
            assert!(updated_view.content_hash().is_match(&expected_hash));
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

            let service = TemplateService::new();
            let results = service.load(&config, &repo);
            assert!(results.is_ok(), "Expected load success, got: {results:?}");
            let results = results.expect("load should succeed");

            let updated_view =
                repo.find_raw_template_view(&path_key).unwrap().unwrap();
            let snapshot = repo.harness().counters().snapshot();
            let result = results.first();
            let expected_size = u64::try_from(content.len()).unwrap();

            assert_eq!(results.len(), 1);
            assert_eq!(result.map(Template::id), Some(template.id()));
            assert_eq!(updated_view.metadata().size(), expected_size);
            assert_eq!(snapshot.writes, 1);
        }

        #[test]
        fn leaves_repository_unchanged_when_cached_template_is_deleted() {
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
            repo.harness().counters().reset();

            let service = TemplateService::new();
            let results = service.load(&config, &repo);
            assert!(results.is_ok(), "Expected load success, got: {results:?}");
            let results = results.expect("load should succeed");

            let remaining_template =
                repo.find_template_by_path(&path_key).unwrap();
            let remaining_view =
                repo.find_raw_template_view(&path_key).unwrap();
            let snapshot = repo.harness().counters().snapshot();

            assert!(results.is_empty());
            assert!(remaining_template.is_some());
            assert!(remaining_view.is_some());
            assert_eq!(snapshot.deletes, 0);
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
                TemplateService::identify_deleted_template_paths(&repo, &[]);
            assert!(result.is_ok(), "Expected success, got: {result:?}");
            let result = result.expect("deleted path detection should succeed");

            assert_eq!(result, vec![path_key]);
        }
    }
}
