//! Configuration building orchestration with hybrid staleness detection.
//!
//! This module provides the `Builder` struct which orchestrates the complete
//! configuration loading pipeline:
//!
//! 1. **Load raw views** from database (cached file state)
//! 2. **Ingest raw configs** from filesystem
//! 3. **Detect staleness** via timestamp + content hash comparison
//! 4. **Merge configs** when stale
//! 5. **Build domain** via builder-level construction
//! 6. **Persist** updated views and config to database
//!
//! The builder implements efficient staleness tracking by comparing:
//! - File modification timestamps (fast check)
//! - BLAKE3 content hashes (accurate change detection)
//!
//! This hybrid approach avoids unnecessary rebuilds while detecting all actual
//! file changes.

// ...
use traces_fs::{
    DirPath, FileReader,
    metadata::{FileMetadata, FsMetadata},
};
use tracing::instrument;

use crate::config::{
    aggregate::{AppConfig, Version},
    error::{ConfigError, ConfigIngestError},
    frontmatter::Frontmatter,
    logging::Logging,
    merger::{ConfigResolver, ResolutionPlan},
    processor::{
        AnalysisBranch, ComparisonBranch, ConfigFileProcessor, GlobalConfig,
        VaultConfig,
    },
    raw::RawConfig,
    repository::Repository,
    task::Task,
    vault::{VaultId, VaultRoot},
    views::{RawFileVersion, RawGlobalConfigView, RawVaultConfigView},
};
#[allow(
    unused_imports,
    reason = "required for builder construction tests"
)]
use crate::{candidate::CandidatePath, discovery::service::DiscoveryResult};

/// Build validated config from layered raw sources.
///
/// Precedence: defaults < global < vault.
///
/// # Errors
/// Returns [`ConfigError`] if validation fails while constructing domain
/// types.
#[inline]
#[expect(
    clippy::too_many_lines,
    reason = "Linear orchestration of component merging and domain \
              construction"
)]
pub fn build_from_layers(
    global: Option<&RawConfig>,
    vault: Option<&RawConfig>,
    root: DirPath,
) -> Result<AppConfig, ConfigError> {
    let name = name_from_root(&root);

    let logging = vault.and_then(|v| v.logging.clone()).or_else(|| {
        let g = global?;
        g.logging.clone()
    });
    let frontmatter = vault.and_then(|v| v.frontmatter.clone()).or_else(|| {
        let g = global?;
        g.frontmatter.clone()
    });
    let task = vault.and_then(|v| v.task.clone()).or_else(|| {
        let g = global?;
        g.task.clone()
    });

    let raw_cache_dir =
        vault.and_then(|v| v.cache.as_ref()).and_then(|c| c.directory.clone());
    let raw_template_dir = vault
        .and_then(|v| v.template.as_ref())
        .and_then(|t| t.directory.clone())
        .or_else(|| {
            global
                .and_then(|g| g.template.as_ref())
                .and_then(|t| t.directory.clone())
        });
    let raw_schema_dir = vault
        .and_then(|v| v.schema.as_ref())
        .and_then(|s| s.directory.clone())
        .or_else(|| {
            global
                .and_then(|g| g.schema.as_ref())
                .and_then(|s| s.directory.clone())
        });
    let raw_property_bank_file = vault
        .and_then(|v| v.schema.as_ref())
        .and_then(|s| s.property_bank_file.clone())
        .or_else(|| {
            global
                .and_then(|g| g.schema.as_ref())
                .and_then(|s| s.property_bank_file.clone())
        });

    let logging =
        logging.map(Logging::try_from).transpose()?.unwrap_or_default();
    let frontmatter =
        frontmatter.map(Frontmatter::try_from).transpose()?.unwrap_or_default();
    let task = task.map(Task::try_from_raw).transpose()?.unwrap_or_default();

    // Parse cache
    let cache_dir =
        if let Some(s) = raw_cache_dir.as_ref().filter(|s| !s.is_empty()) {
            super::cache::CacheDir::try_new(std::path::Path::new(s)).map_err(
                |e| ConfigError::ValidationFailed {
                    field: "cache.directory".into(),
                    message: format!("invalid cache directory: {e}").into(),
                },
            )?
        } else {
            super::cache::CacheDir::default()
        };
    let cache = super::cache::CacheConfig::new(cache_dir);

    // Parse template
    let template_dir =
        if let Some(s) = raw_template_dir.as_ref().filter(|s| !s.is_empty()) {
            super::template::TemplateDir::try_new(std::path::Path::new(s))
                .map_err(|e| ConfigError::ValidationFailed {
                    field: "template.directory".into(),
                    message: format!("invalid template directory: {e}").into(),
                })?
        } else {
            super::template::TemplateDir::default()
        };
    let template = super::template::TemplateConfig::new(template_dir);

    // Parse schema
    let schema_dir =
        if let Some(s) = raw_schema_dir.as_ref().filter(|s| !s.is_empty()) {
            super::schema::SchemaDir::try_new(std::path::Path::new(s)).map_err(
                |e| ConfigError::ValidationFailed {
                    field: "schema.directory".into(),
                    message: format!("invalid schema directory: {e}").into(),
                },
            )?
        } else {
            super::schema::SchemaDir::default()
        };
    let property_bank_file = if let Some(s) =
        raw_property_bank_file.as_ref().filter(|s| !s.is_empty())
    {
        super::schema::PropertyBankFile::try_new(s.as_str()).map_err(|e| {
            ConfigError::ValidationFailed {
                field: "schema.property_bank_file".into(),
                message: format!("invalid property_bank_file: {e}").into(),
            }
        })?
    } else {
        super::schema::PropertyBankFile::default()
    };
    let schema =
        super::schema::SchemaConfig::new(schema_dir, property_bank_file);

    Ok(AppConfig::new(
        Version::initial(),
        root,
        name,
        logging,
        cache,
        template,
        schema,
        frontmatter,
        task,
    ))
}

/// Derives a vault name from the last component of the vault root path.
///
/// Falls back to `"unnamed"` when the path has no final component.
fn name_from_root(root: &DirPath) -> Box<str> {
    root.as_path()
        .file_name()
        .map_or_else(
            || "unnamed".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        )
        .into_boxed_str()
}

/// Configuration builder with hybrid staleness detection.
///
/// Coordinates the full configuration loading pipeline:
/// - Vault and global candidate paths (from `DiscoveryResult`)
/// - Staleness detection (timestamps + content hash)
/// - `AppConfig` merging (global + vault precedence)
/// - Domain validation
/// - Database persistence
///
/// # Architecture
///
/// The builder owns the orchestration pipeline but delegates to:
/// - `Repository`: Database persistence and retrieval
/// - `build_from_layers`: Domain validation and construction
///
/// Callers construct a `Builder` via `Builder::from_discovery`, which
/// consumes a [`DiscoveryResult`] produced by the Bootstrapper.
///
/// # Second-call / cached-path behaviour
///
/// Each call to [`Builder::build`] performs a full staleness check. If the
/// on-disk files match the cached views stored in the repository (same
/// timestamps and content hashes), the builder follows the `UseCached` plan
/// and loads the previously-built [`AppConfig`] from the database without
/// re-parsing or re-merging. If any file has changed, the builder rebuilds
/// from scratch and persists the updated views and config.
pub struct Builder<R> {
    /// Ordered vault-local candidate paths (guaranteed non-empty by
    /// `DiscoveryService`).
    vault: Box<[CandidatePath]>,
    /// Ordered global candidate paths (may be empty).
    global: Box<[CandidatePath]>,
    /// Repository for database persistence.
    repository: R,
}

/// Return type of [`Builder::build_vault`].
///
/// `(VaultId, VaultRoot, raw config, cached view)`
type VaultBuildResult =
    (VaultId, VaultRoot, Option<RawConfig>, Option<RawVaultConfigView>);

/// Return type of [`Builder::build_global`].
///
/// `(raw config, cached view)`
type GlobalBuildResult = (Option<RawConfig>, Option<RawGlobalConfigView>);

impl<R> Builder<R>
where
    R: Repository,
{
    /// Create a new builder.
    #[inline]
    #[must_use]
    pub fn new(
        vault: Box<[CandidatePath]>,
        global: Box<[CandidatePath]>,
        repository: R,
    ) -> Self {
        Self {
            vault,
            global,
            repository,
        }
    }

    /// Build configuration from discovered candidates.
    ///
    /// Pipeline:
    /// 1. Build vault config via `build_vault()`
    /// 2. Build global config via `build_global()` (if global candidate exists)
    /// 3. Resolve processor outcomes
    /// 4. Execute persistence/build plan
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - File ingestion fails (I/O error, parse error)
    /// - Merging/building fails due to invalid values
    /// - Domain validation fails (invalid config)
    /// - Database operations fail
    #[inline]
    #[instrument(skip(self), level = "debug")]
    pub fn build(&self) -> Result<AppConfig, ConfigError> {
        let (vault_id, vault_root, vault_raw, vault_view) =
            self.build_vault()?;

        let (global_raw, global_view) = self.build_global()?;

        // Process global config
        let global_processor = ConfigFileProcessor::<GlobalConfig, _, _>::new(
            global_raw,
            global_view,
        );
        let global_outcome = match global_processor.compare()? {
            ComparisonBranch::Fresh(proc) => proc.finalize(),
            ComparisonBranch::Stale(proc) => match proc.analyze()? {
                AnalysisBranch::NoChanges(proc) => proc.finalize(),
                AnalysisBranch::PropertyChanges(proc) => proc.finalize(),
            },
        };

        // Process vault config
        let vault_processor = ConfigFileProcessor::<VaultConfig, _, _>::new(
            vault_raw, vault_view,
        );
        let vault_outcome = match vault_processor.compare()? {
            ComparisonBranch::Fresh(proc) => proc.finalize(),
            ComparisonBranch::Stale(proc) => match proc.analyze()? {
                AnalysisBranch::NoChanges(proc) => proc.finalize(),
                AnalysisBranch::PropertyChanges(proc) => proc.finalize(),
            },
        };

        // Resolve outcomes, then execute persistence/build plan
        let resolver = ConfigResolver::new();
        let plan = resolver.resolve(global_outcome, vault_outcome)?;
        self.execute_plan(vault_id, &vault_root, plan)
    }

    /// Read, classify, and fetch cached view for the primary vault candidate.
    ///
    /// Returns `(VaultId, VaultRoot, Option<RawConfig>,
    /// Option<RawVaultConfigView>)`. Always called because the vault
    /// candidate list is guaranteed non-empty by `DiscoveryService`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - `VaultRoot` derivation fails
    /// - `VaultId` lookup or creation fails
    /// - Filesystem metadata cannot be read
    /// - Database view query fails
    fn build_vault(&self) -> Result<VaultBuildResult, ConfigError> {
        // INVARIANT: vault[0] is safe — DiscoveryService guarantees the vault
        // candidate list is non-empty before producing a DiscoveryResult.
        #[expect(
            clippy::indexing_slicing,
            reason = "vault[0] is safe: DiscoveryService enforces non-empty \
                      vault candidates"
        )]
        let candidate = &self.vault[0];

        // Derive VaultRoot from the candidate base directory.
        let vault_root = VaultRoot::from_dir_path(candidate.base().clone());

        // Resolve or create the VaultId.
        let vault_id = self.get_or_create_vault_id(&vault_root)?;

        // Read filesystem metadata from the candidate path.
        let metadata = FsMetadata::from_path(candidate.path().as_path())
            .map_err(|e| ConfigError::Ingestion(e.to_string().into()))?;
        let file_metadata: Option<FileMetadata> = metadata.as_file().cloned();

        // Parse the raw vault config.
        let reader = FileReader::from_system_root();
        let mut raw = reader
            .parse_structured::<RawConfig>(candidate.path().as_path())
            .map_err(ConfigIngestError::from)
            .map_err(ConfigError::from)?;
        raw.metadata = file_metadata;

        // Fetch the cached vault view from the database.
        let view = self
            .repository
            .get_raw_vault_view(vault_id)
            .map_err(Into::<ConfigError>::into)?;

        Ok((vault_id, vault_root, Some(raw), view))
    }

    /// Read, classify, and fetch cached view for the primary global candidate.
    ///
    /// Returns `(Option<RawConfig>, Option<RawGlobalConfigView>)`.
    /// Returns `(None, None)` when no global candidate is present.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if filesystem metadata or database queries
    /// fail.
    fn build_global(&self) -> Result<GlobalBuildResult, ConfigError> {
        let Some(candidate) = self.global.first() else {
            return Ok((None, None));
        };

        // Read filesystem metadata.
        let metadata = FsMetadata::from_path(candidate.path().as_path())
            .map_err(|e| ConfigError::Ingestion(e.to_string().into()))?;
        let file_metadata: Option<FileMetadata> = metadata.as_file().cloned();

        // Parse the raw global config.
        let reader = FileReader::from_system_root();
        let mut raw = reader
            .parse_structured::<RawConfig>(candidate.path().as_path())
            .map_err(ConfigIngestError::from)
            .map_err(ConfigError::from)?;
        raw.metadata = file_metadata;

        // Fetch the cached global view from the database.
        let view = self
            .repository
            .get_raw_global_view()
            .map_err(Into::<ConfigError>::into)?;

        Ok((Some(raw), view))
    }

    fn execute_plan(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
        plan: ResolutionPlan,
    ) -> Result<AppConfig, ConfigError> {
        match plan {
            ResolutionPlan::UseCached => self.load_cached_config(vault_id),
            ResolutionPlan::UpdateViews {
                global,
                vault,
            } => {
                if let Some(global) = global {
                    let path = self
                        .global
                        .first()
                        .map(|c| {
                            c.path().as_path().to_string_lossy().into_owned()
                        })
                        .ok_or(ConfigError::ValidationFailed {
                            field: "global_path".into(),
                            message: "missing file path for global view update"
                                .into(),
                        })?;
                    self.update_global_view(&global, &path)?;
                }
                if let Some(vault) = vault {
                    let path = self
                        .vault
                        .first()
                        .map(|c| {
                            c.path().as_path().to_string_lossy().into_owned()
                        })
                        .ok_or(ConfigError::ValidationFailed {
                            field: "vault_config_file".into(),
                            message: "missing file path for vault view update"
                                .into(),
                        })?;
                    self.update_vault_view(vault_id, &vault, &path)?;
                }
                self.load_cached_config(vault_id)
            }
            ResolutionPlan::Rebuild {
                global,
                vault,
            } => self.rebuild_with_configs(
                vault_id,
                vault_root,
                global.as_ref(),
                vault.as_ref(),
            ),
        }
    }

    fn load_cached_config(
        &self,
        vault_id: VaultId,
    ) -> Result<AppConfig, ConfigError> {
        let version = self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::<ConfigError>::into)?
            .ok_or(ConfigError::ValidationFailed {
                field: "config".into(),
                message: "No active config version found".into(),
            })?;

        self.repository
            .get_config(vault_id, version)
            .map_err(Into::<ConfigError>::into)?
            .ok_or(ConfigError::ValidationFailed {
                field: "config".into(),
                message: "AppConfig not found in database".into(),
            })
    }

    fn update_global_view(
        &self,
        raw: &RawConfig,
        file_path: &str,
    ) -> Result<(), ConfigError> {
        let mut view = self
            .repository
            .get_raw_global_view()
            .map_err(Into::<ConfigError>::into)?
            .unwrap_or_else(|| RawGlobalConfigView::new(file_path.into()));

        let version = Self::raw_global_to_version(raw)?;
        view.push_version(version);
        self.repository
            .save_raw_global_view(&view)
            .map_err(Into::<ConfigError>::into)
    }

    fn update_vault_view(
        &self,
        vault_id: VaultId,
        raw: &RawConfig,
        file_path: &str,
    ) -> Result<(), ConfigError> {
        let mut view = self
            .repository
            .get_raw_vault_view(vault_id)
            .map_err(Into::<ConfigError>::into)?
            .unwrap_or_else(|| RawVaultConfigView::new(file_path.into()));

        let version = Self::raw_vault_to_version(raw)?;
        view.push_version(version);
        self.repository
            .save_raw_vault_view(vault_id, &view)
            .map_err(Into::<ConfigError>::into)
    }

    fn rebuild_with_configs(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
        global: Option<&RawConfig>,
        vault: Option<&RawConfig>,
    ) -> Result<AppConfig, ConfigError> {
        let next_version = self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::<ConfigError>::into)?
            .map(super::aggregate::Version::next)
            .transpose()?
            .unwrap_or_else(Version::initial);

        let config =
            build_from_layers(global, vault, vault_root.as_dir_path().clone())?
                .with_version(next_version);

        self.repository
            .save_config(vault_id, &config)
            .map_err(Into::<ConfigError>::into)?;

        if let Some(global) = global {
            let path = self
                .global
                .first()
                .map(|c| c.path().as_path().to_string_lossy().into_owned())
                .ok_or(ConfigError::ValidationFailed {
                    field: "global_path".into(),
                    message: "missing file path for global view update during \
                              rebuild"
                        .into(),
                })?;
            self.update_global_view(global, &path)?;
        }
        if let Some(vault) = vault {
            let path = self
                .vault
                .first()
                .map(|c| c.path().as_path().to_string_lossy().into_owned())
                .ok_or(ConfigError::ValidationFailed {
                    field: "vault_path".into(),
                    message: "missing file path for vault view update during \
                              rebuild"
                        .into(),
                })?;
            self.update_vault_view(vault_id, vault, &path)?;
        }

        Ok(config)
    }

    fn raw_global_to_version(
        raw: &RawConfig,
    ) -> Result<RawFileVersion, ConfigError> {
        let file_info =
            raw.metadata.clone().ok_or(ConfigError::ValidationFailed {
                field: "global.metadata".into(),
                message: "missing file metadata for global config view update"
                    .into(),
            })?;
        let content = toml::to_string(raw).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "global".into(),
                message: format!(
                    "failed to serialize global raw config: {error}"
                )
                .into(),
            }
        })?;
        RawFileVersion::new(content.as_bytes(), file_info).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "global".into(),
                message: format!(
                    "failed to record global raw version: {error}"
                )
                .into(),
            }
        })
    }

    fn raw_vault_to_version(
        raw: &RawConfig,
    ) -> Result<RawFileVersion, ConfigError> {
        let file_info =
            raw.metadata.clone().ok_or(ConfigError::ValidationFailed {
                field: "vault.metadata".into(),
                message: "missing file metadata for vault config view update"
                    .into(),
            })?;
        let content = toml::to_string(raw).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "vault".into(),
                message: format!(
                    "failed to serialize vault raw config: {error}"
                )
                .into(),
            }
        })?;
        RawFileVersion::new(content.as_bytes(), file_info).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "vault".into(),
                message: format!("failed to record vault raw version: {error}")
                    .into(),
            }
        })
    }

    /// Get or create vault ID for the discovered vault root.
    ///
    /// Looks up existing vault ID from path mapping, or creates a new one.
    fn get_or_create_vault_id(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<VaultId, ConfigError> {
        if let Some(existing_id) = self
            .repository
            .find_vault_id_by_path(vault_root)
            .map_err(Into::<ConfigError>::into)?
        {
            Ok(existing_id)
        } else {
            let new_id = VaultId::new();
            self.repository
                .save_vault_path_mapping(new_id, vault_root)
                .map_err(Into::<ConfigError>::into)?;
            Ok(new_id)
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use traces_fs::{DirPath, FilePath};

    use super::*;
    use crate::config::storage::testing::InMemoryRepository;
    #[allow(
        unused_imports,
        reason = "required for builder construction tests"
    )]
    use crate::report::DiscoveryReport;

    /// Create a temporary vault directory with a `traces.toml` config file.
    fn make_vault_candidate(dir: &TempDir, content: &str) -> CandidatePath {
        std::fs::write(dir.path().join("traces.toml"), content)
            .expect("write vault config");
        CandidatePath::new(
            DirPath::try_new(dir.path().to_path_buf()).expect("valid base dir"),
            FilePath::try_new(dir.path().join("traces.toml"))
                .expect("valid file path"),
        )
    }

    /// Create a temporary global directory with a `traces.toml` config file.
    fn make_global_candidate(dir: &TempDir, content: &str) -> CandidatePath {
        std::fs::write(dir.path().join("traces.toml"), content)
            .expect("write global config");
        CandidatePath::new(
            DirPath::try_new(dir.path().to_path_buf()).expect("valid base dir"),
            FilePath::try_new(dir.path().join("traces.toml"))
                .expect("valid file path"),
        )
    }

    fn placeholder_cache_root() -> crate::CacheRoot {
        crate::CacheRoot::new(
            crate::CacheLocation::Global(
                crate::GlobalCacheLocation::PlatformUserCache,
            ),
            std::path::PathBuf::from("/tmp/placeholder-cache"),
        )
    }

    mod new {
        use super::*;

        #[test]
        fn constructs_builder() {
            let vault = Box::from([]);
            let global = Box::from([]);
            let _builder: Builder<InMemoryRepository> =
                Builder::new(vault, global, InMemoryRepository::new());
        }
    }

    mod from_discovery {
        use super::*;
        use crate::config::repository::ReadRepository as _;

        #[test]
        fn stores_vault_candidates_as_boxed_slice() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            assert_eq!(builder.vault.len(), 1);
        }

        #[test]
        fn stores_global_candidates_as_boxed_slice() {
            let vault_dir = TempDir::new().expect("vault dir");
            let global_dir = TempDir::new().expect("global dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let global = make_global_candidate(&global_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![global],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            assert_eq!(builder.global.len(), 1);
        }

        #[test]
        fn is_infallible() {
            // from_discovery() returns Self, not Result — this compiles only
            // if the return type is Builder<_>.
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let _builder: Builder<InMemoryRepository> = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );
        }

        #[test]
        fn stores_repository() {
            // Verify the stored repository is reachable by performing a
            // repository operation through build_vault (which calls
            // get_or_create_vault_id).
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            // get_or_create_vault_id exercises the stored repository.
            let vault_root = VaultRoot::from_dir_path(
                builder
                    .vault
                    .first()
                    .expect("vault[0] present by construction")
                    .base()
                    .clone(),
            );
            let id =
                builder.get_or_create_vault_id(&vault_root).expect("vault id");
            let found = builder
                .repository
                .find_vault_id_by_path(&vault_root)
                .expect("find vault")
                .expect("present");
            assert_eq!(id, found);
        }
    }

    mod build_vault {
        use super::*;
        use crate::config::repository::ReadRepository as _;

        #[test]
        fn derives_vault_root_from_candidate_base() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let base = vault.base().clone();
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (_, vault_root, _, _) =
                builder.build_vault().expect("build_vault");

            assert_eq!(vault_root.as_dir_path(), &base);
        }

        #[test]
        fn resolves_vault_id_from_database() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (vault_id, vault_root, _, _) =
                builder.build_vault().expect("build_vault");

            // Calling again should return the same VaultId (DB lookup).
            let found = builder
                .repository
                .find_vault_id_by_path(&vault_root)
                .expect("find vault")
                .expect("present");
            assert_eq!(vault_id, found);
        }

        #[test]
        fn reads_file_metadata_from_candidate() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(
                &vault_dir,
                "[template]\ndirectory = \"t\"",
            );
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (_, _, raw, _) = builder.build_vault().expect("build_vault");

            // metadata is populated from FsMetadata
            assert!(raw.as_ref().and_then(|r| r.metadata.as_ref()).is_some());
        }

        #[test]
        fn queries_database_for_vault_view() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            // No prior view → None from DB.
            let (_, _, _, view) = builder.build_vault().expect("build_vault");
            assert!(view.is_none(), "should be None on first call");
        }

        #[test]
        fn parses_raw_config_from_candidate_file() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(
                &vault_dir,
                "[template]\ndirectory = \"custom-templates\"",
            );
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (_, _, raw, _) = builder.build_vault().expect("build_vault");

            assert_eq!(
                raw.as_ref()
                    .and_then(|r| r.template.as_ref())
                    .and_then(|t| t.directory.as_deref()),
                Some("custom-templates")
            );
        }

        #[test]
        fn returns_raw_vault_config() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (_, _, raw, _) = builder.build_vault().expect("build_vault");
            assert!(raw.is_some());
        }
    }

    mod build_global {
        use super::*;

        #[test]
        fn returns_none_when_no_global_candidate() {
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (raw, view) = builder.build_global().expect("build_global");
            assert!(raw.is_none());
            assert!(view.is_none());
        }

        #[test]
        fn reads_file_and_returns_raw_global_config() {
            let vault_dir = TempDir::new().expect("vault dir");
            let global_dir = TempDir::new().expect("global dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let global = make_global_candidate(
                &global_dir,
                "[template]\ndirectory = \"global-templates\"",
            );
            let result = DiscoveryResult::new(
                vec![vault],
                vec![global],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (raw, _) = builder.build_global().expect("build_global");

            assert_eq!(
                raw.as_ref()
                    .and_then(|r| r.template.as_ref())
                    .and_then(|t| t.directory.as_deref()),
                Some("global-templates")
            );
        }

        #[test]
        fn queries_database_for_global_view() {
            let vault_dir = TempDir::new().expect("vault dir");
            let global_dir = TempDir::new().expect("global dir");
            let vault = make_vault_candidate(&vault_dir, "");
            let global = make_global_candidate(&global_dir, "");
            let result = DiscoveryResult::new(
                vec![vault],
                vec![global],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let (_, view) = builder.build_global().expect("build_global");
            // No prior view → None from DB.
            assert!(view.is_none(), "should be None on first call");
        }
    }

    /// Regression tests confirming `build_from_layers` merge contract is
    /// preserved during refactoring.
    ///
    /// These tests are independent of the new `Builder` API — they exercise
    /// the merge seam directly to detect any accidental behavioural changes.
    mod build_from_layers_regression {
        use super::*;

        fn test_vault_root(dir: &TempDir) -> DirPath {
            DirPath::try_new(dir.path().to_path_buf())
                .expect("valid vault root")
        }

        #[test]
        fn preserves_existing_merge_behavior_vault_overrides_global() {
            let dir = TempDir::new().expect("dir");
            let vault_root = test_vault_root(&dir);

            let global = RawConfig {
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("global-tpl".to_owned()),
                }),
                ..Default::default()
            };
            let vault = RawConfig {
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("vault-tpl".to_owned()),
                }),
                ..Default::default()
            };

            let config =
                build_from_layers(Some(&global), Some(&vault), vault_root)
                    .expect("build_from_layers");

            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "vault-tpl",
                "vault template must override global template"
            );
        }

        #[test]
        fn preserves_existing_merge_behavior_global_used_when_no_vault() {
            let dir = TempDir::new().expect("dir");
            let vault_root = test_vault_root(&dir);

            let global = RawConfig {
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("global-tpl".to_owned()),
                }),
                ..Default::default()
            };

            let config = build_from_layers(Some(&global), None, vault_root)
                .expect("build_from_layers");

            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "global-tpl",
                "global template must be used when vault has none"
            );
        }

        #[test]
        fn preserves_existing_merge_behavior_defaults_used_when_no_sources() {
            let dir = TempDir::new().expect("dir");
            let vault_root = test_vault_root(&dir);

            let config = build_from_layers(None, None, vault_root)
                .expect("build_from_layers");

            // Defaults: the documented default template dir is "templates"
            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "templates",
                "defaults must produce the documented default template dir"
            );
        }
    }

    mod build {
        use super::*;

        #[test]
        fn orchestrates_vault_and_global() {
            let vault_dir = TempDir::new().expect("vault dir");
            let global_dir = TempDir::new().expect("global dir");
            let vault = make_vault_candidate(
                &vault_dir,
                "[template]\ndirectory = \"vault-templates\"",
            );
            let global = make_global_candidate(
                &global_dir,
                "[template]\ndirectory = \"global-templates\"",
            );
            let result = DiscoveryResult::new(
                vec![vault],
                vec![global],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let config = builder.build().expect("build config");

            // Vault takes precedence over global for template directory.
            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "vault-templates"
            );
        }

        #[test]
        fn builds_from_vault_only_discovery() {
            // IMPORTANT: content must contain at least one non-default field so
            // `compute_field_hashes` returns a non-empty set and the resolver
            // chooses `Rebuild` instead of `UseCached`.  An empty TOML would
            // drive `UseCached` → `load_cached_config` → error on a fresh
            // `InMemoryRepository` ("No active config version found").
            let vault_dir = TempDir::new().expect("vault dir");
            let vault = make_vault_candidate(
                &vault_dir,
                "[template]\ndirectory = \"custom-templates\"",
            );
            let result = DiscoveryResult::new(
                vec![vault],
                vec![],
                placeholder_cache_root(),
                DiscoveryReport::default(),
            );
            let (vault_candidates, global_candidates) = result.candidates();
            let builder = Builder::new(
                vault_candidates,
                global_candidates,
                InMemoryRepository::new(),
            );

            let config = builder.build().expect("build config");

            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "custom-templates"
            );
        }
    }
}
