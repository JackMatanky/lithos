//! Configuration building orchestration with hybrid staleness detection.
//!
//! This module provides the [`Builder`] struct which orchestrates the complete
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

use std::path::PathBuf;

use tracing::instrument;

use crate::{
    config::{
        aggregate::{Config, Version},
        discovery::{DiscoveryEngine, DiscoveryResult},
        error::{ConfigError, ConfigIngestError},
        frontmatter::Frontmatter,
        logging::Logging,
        merger::{ConfigResolver, ResolutionPlan},
        paths::Paths,
        processor::{
            AnalysisBranch, ComparisonBranch, ConfigFileProcessor,
            GlobalConfig, VaultConfig,
        },
        raw::{RawGlobalConfig, RawPathsConfig, RawVaultConfig},
        repository::Repository,
        task::Task,
        vault::{VaultId, VaultRoot},
        views::{RawFileVersion, RawGlobalConfigView, RawVaultConfigView},
    },
    fs::FileReader,
};

/// Build validated config from layered raw sources.
///
/// Precedence: defaults < global < vault.
///
/// # Errors
/// Returns [`ConfigError`] if validation fails while constructing domain
/// types.
#[inline]
pub fn build_from_layers(
    global: Option<&RawGlobalConfig>,
    vault: Option<&RawVaultConfig>,
    vault_id: VaultId,
    vault_root: VaultRoot,
    version: Version,
) -> Result<Config, ConfigError> {
    let vault_metadata =
        super::vault::Metadata::new(vault_id, vault_root, None, None)?;

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

    let paths = RawPathsConfig::merge(
        global.map_or_else(RawPathsConfig::default, |g| g.paths.clone().into()),
        vault.map_or_else(RawPathsConfig::default, |v| v.paths.clone().into()),
    );

    let logging =
        logging.map(Logging::try_from).transpose()?.unwrap_or_default();
    let paths = Paths::try_from(&paths)?;
    let frontmatter =
        frontmatter.map(Frontmatter::try_from).transpose()?.unwrap_or_default();
    let task = task.map(Task::try_from_raw).transpose()?.unwrap_or_default();

    Ok(Config::new(version, vault_metadata, logging, paths, frontmatter, task))
}

/// Configuration builder with hybrid staleness detection.
///
/// Coordinates the full configuration loading pipeline:
/// - File discovery (filesystem + database)
/// - Staleness detection (timestamps + content hash)
/// - Config merging (global + vault precedence)
/// - Domain validation
/// - Database persistence
///
/// # Architecture
///
/// The builder owns the orchestration pipeline but delegates to:
/// - `DiscoveryEngine`: File discovery and database query
/// - `Repository`: Database persistence and retrieval
/// - `build_from_layers`: Domain validation and construction
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
///
/// use lithos_core::config::{builder::Builder, storage::RedbStorage};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vault_root = PathBuf::from("/vault");
/// // let repo = RedbStorage::new(...);
/// // let builder = Builder::new(vault_root, repo);
/// // let config = builder.load()?;
/// # Ok(())
/// # }
/// ```
pub struct Builder<R> {
    /// Vault root path for config resolution.
    vault_root: PathBuf,
    /// Repository for database persistence.
    repository: R,
}

impl<R> Builder<R>
where
    R: Repository,
{
    /// Create a new builder for the given vault root.
    ///
    /// # Parameters
    ///
    /// - `vault_root`: Vault root directory for config resolution
    /// - `repository`: Database repository for persistence
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::{builder::Builder, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let builder = Builder::new(vault_root, repo);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(vault_root: P, repository: R) -> Self {
        Self {
            vault_root: vault_root.into(),
            repository,
        }
    }

    /// Load configuration with hybrid staleness detection.
    ///
    /// Pipeline:
    /// 1. Load raw views from DB (if exist)
    /// 2. Ingest raw configs from files
    /// 3. Detect staleness (timestamps + content hash)
    /// 4. If stale: merge → build → save
    /// 5. If fresh: load cached config from DB
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - File ingestion fails (I/O error, parse error)
    /// - Merging/building fails due to invalid values
    /// - Domain validation fails (invalid config)
    /// - Database operations fail
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::{builder::Builder, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let builder = Builder::new(vault_root, repo);
    /// // let config = builder.load()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(vault_root = %self.vault_root.display())
    )]
    pub fn load(&self) -> Result<Config, ConfigError> {
        // Step 1: Validate and get vault root
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        // Step 2: Get or create vault ID
        let vault_id = self.get_or_create_vault_id()?;

        // Step 3: Run discovery engine (filesystem + database)
        let discovery = DiscoveryEngine::run(&vault_root, &self.repository)?;

        // Step 4: Parse raw configs from discovered files
        let global_raw = if let Some(entry) = discovery.global().entry() {
            let reader = FileReader::from_system_root();
            let entry_path = entry.path();
            let mut raw = reader
                .parse_structured::<RawGlobalConfig>(entry_path.as_path())
                .map_err(ConfigIngestError::from)
                .map_err(ConfigError::from)?;
            raw.metadata = discovery.global().info().cloned();
            Some(raw)
        } else {
            None
        };

        let vault_raw = if let Some(_entry) = discovery.vault().entry() {
            let reader = FileReader::new(vault_root.as_path());
            let relative_path = std::path::Path::new(".lithos/lithos.toml");
            let mut raw = reader
                .parse_structured::<RawVaultConfig>(relative_path)
                .map_err(ConfigIngestError::from)
                .map_err(ConfigError::from)?;
            raw.metadata = discovery.vault().info().cloned();
            Some(raw)
        } else {
            None
        };

        // Extract views from discovery
        let global_view = discovery.global().view().cloned();
        let vault_view = discovery.vault().view().cloned();

        // Step 5: Process global config
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

        // Step 6: Process vault config
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

        // Step 7: Resolve outcomes, then execute persistence/build plan
        let resolver = ConfigResolver::new();
        let plan = resolver.resolve(global_outcome, vault_outcome)?;
        self.execute_plan(vault_id, &vault_root, plan, &discovery)
    }

    #[allow(clippy::too_many_arguments, reason = "Internal execution pipeline")]
    fn execute_plan(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
        plan: ResolutionPlan,
        discovery: &DiscoveryResult,
    ) -> Result<Config, ConfigError> {
        match plan {
            ResolutionPlan::UseCached => self.load_cached_config(vault_id),
            ResolutionPlan::UpdateViews {
                global,
                vault,
            } => {
                if let Some(global) = global {
                    let path = discovery
                        .global()
                        .entry()
                        .map(|e| {
                            e.path().as_path().to_string_lossy().into_owned()
                        })
                        .ok_or(ConfigError::ValidationFailed {
                            field: "global_path".into(),
                            message: "missing file path for global view update"
                                .into(),
                        })?;
                    self.update_global_view(&global, &path)?;
                }
                if let Some(vault) = vault {
                    let path = discovery
                        .vault()
                        .entry()
                        .map(|e| {
                            e.path().as_path().to_string_lossy().into_owned()
                        })
                        .ok_or(ConfigError::ValidationFailed {
                            field: "vault_path".into(),
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
                discovery,
            ),
        }
    }

    fn load_cached_config(
        &self,
        vault_id: VaultId,
    ) -> Result<Config, ConfigError> {
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
                message: "Config not found in database".into(),
            })
    }

    fn update_global_view(
        &self,
        raw: &RawGlobalConfig,
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
        raw: &RawVaultConfig,
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

    #[allow(clippy::too_many_arguments, reason = "Internal execution pipeline")]
    fn rebuild_with_configs(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
        discovery: &DiscoveryResult,
    ) -> Result<Config, ConfigError> {
        let next_version = self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::<ConfigError>::into)?
            .map(super::aggregate::Version::next)
            .transpose()?
            .unwrap_or_else(Version::initial);

        let config = build_from_layers(
            global,
            vault,
            vault_id,
            vault_root.clone(),
            next_version,
        )?;

        self.repository
            .save_config(vault_id, &config)
            .map_err(Into::<ConfigError>::into)?;

        if let Some(global) = global {
            let path = discovery
                .global()
                .entry()
                .map(|e| e.path().as_path().to_string_lossy().into_owned())
                .ok_or(ConfigError::ValidationFailed {
                    field: "global_path".into(),
                    message: "missing file path for global view update during \
                              rebuild"
                        .into(),
                })?;
            self.update_global_view(global, &path)?;
        }
        if let Some(vault) = vault {
            let path = discovery
                .vault()
                .entry()
                .map(|e| e.path().as_path().to_string_lossy().into_owned())
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
        raw: &RawGlobalConfig,
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
        raw: &RawVaultConfig,
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

    /// Get or create vault ID for the vault root.
    ///
    /// Looks up existing vault ID from path mapping, or creates a new one.
    fn get_or_create_vault_id(&self) -> Result<VaultId, ConfigError> {
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        if let Some(existing_id) = self
            .repository
            .find_vault_id_by_path(&vault_root)
            .map_err(Into::<ConfigError>::into)?
        {
            Ok(existing_id)
        } else {
            let new_id = VaultId::new();
            self.repository
                .save_vault_path_mapping(new_id, &vault_root)
                .map_err(Into::<ConfigError>::into)?;
            Ok(new_id)
        }
    }
}
