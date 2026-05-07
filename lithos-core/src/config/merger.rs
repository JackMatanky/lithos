//! Config merging orchestration for processor outcomes.
//!
//! This module combines the outcomes from parallel config file processors
//! (global and vault) and merges them according to precedence rules:
//! defaults < global < vault (vault has highest priority).
//!
//! The merger handles all 9 outcome combinations and produces the final
//! domain `Config` object with proper view persistence.

use crate::config::{
    aggregate::{Config, Version},
    error::ConfigError,
    processor::{GlobalConfig, ProcessorOutcome, VaultConfig},
    raw::{RawConfig, RawGlobalConfig, RawPathsConfig, RawVaultConfig},
    storage::Repository,
    vault::{VaultId, VaultRoot},
};

/// Config merger for combining processor outcomes.
///
/// Takes the results from both global and vault processors and:
/// 1. Determines the merge strategy based on outcomes
/// 2. Merges raw configs with proper precedence (vault wins)
/// 3. Builds the domain `Config` object
/// 4. Persists views and config to repository
pub struct ConfigMerger<'a, R> {
    vault_id: VaultId,
    vault_root: VaultRoot,
    repository: &'a R,
}

impl<'a, R> ConfigMerger<'a, R>
where
    R: Repository,
{
    /// Create a new merger.
    #[inline]
    #[must_use]
    pub fn new(
        vault_id: VaultId,
        vault_root: VaultRoot,
        repository: &'a R,
    ) -> Self {
        Self {
            vault_id,
            vault_root,
            repository,
        }
    }

    /// Merge processor outcomes into final Config.
    ///
    /// Handles all 9 combinations of (UseCached | UpdateViewOnly | Rebuild) ×
    /// 2.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Merging fails (incompatible configs)
    /// - Domain validation fails
    /// - Database persistence fails
    pub fn merge(
        self,
        global_outcome: ProcessorOutcome<GlobalConfig>,
        vault_outcome: ProcessorOutcome<VaultConfig>,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        use ProcessorOutcome::{Rebuild, UpdateViewOnly, UseCached};

        match (global_outcome, vault_outcome) {
            // Both fresh - load from DB
            (UseCached, UseCached) => self.load_cached_config(),

            // Metadata-only updates - sync views, keep config
            (
                UpdateViewOnly {
                    raw: global,
                },
                UpdateViewOnly {
                    raw: vault,
                },
            ) => self.update_both_views_and_load(global, vault),
            (
                UpdateViewOnly {
                    raw: global,
                },
                UseCached,
            ) => self.update_global_view_and_load(global),
            (
                UseCached,
                UpdateViewOnly {
                    raw: vault,
                },
            ) => self.update_vault_view_and_load(vault),

            // Vault rebuild - always rebuild full config
            (
                UseCached,
                Rebuild {
                    raw: vault,
                    ..
                },
            ) => {
                // Vault changed, global fresh - use defaults for global
                self.rebuild_with_configs(None, Some(&vault))
            }
            (
                UpdateViewOnly {
                    raw: global,
                },
                Rebuild {
                    raw: vault,
                    ..
                },
            ) => {
                // Both configs available (metadata or rebuild)
                self.rebuild_with_configs(Some(&global), Some(&vault))
            }
            (
                Rebuild {
                    raw: global,
                    ..
                },
                Rebuild {
                    raw: vault,
                    ..
                },
            ) => {
                // Both configs rebuilt
                self.rebuild_with_configs(Some(&global), Some(&vault))
            }

            // Global rebuild only, vault cached/metadata-only
            (
                Rebuild {
                    raw: global,
                    ..
                },
                UseCached,
            ) => {
                // Global changed, vault fresh - use defaults for vault
                self.rebuild_with_configs(Some(&global), None)
            }
            (
                Rebuild {
                    raw: global,
                    ..
                },
                UpdateViewOnly {
                    raw: vault,
                },
            ) => {
                // Global rebuilt, vault metadata only
                self.rebuild_with_configs(Some(&global), Some(&vault))
            }
        }
    }

    /// Load cached config from repository.
    fn load_cached_config(&self) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        let version = self
            .repository
            .get_active_version(self.vault_id)
            .map_err(Into::into)?
            .ok_or(ConfigError::ValidationFailed {
                field: "config".into(),
                message: "No active config version found".into(),
            })?;

        self.repository
            .get_config(self.vault_id, version)
            .map_err(Into::into)?
            .ok_or(ConfigError::ValidationFailed {
                field: "config".into(),
                message: "Config not found in database".into(),
            })
    }

    /// Update both views and load cached config.
    fn update_both_views_and_load(
        &self,
        _global: RawGlobalConfig,
        _vault: RawVaultConfig,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // TODO: Implement view updates
        // For now, just load cached config
        self.load_cached_config()
    }

    /// Update global view and load cached config.
    fn update_global_view_and_load(
        &self,
        _global: RawGlobalConfig,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // TODO: Implement view update
        self.load_cached_config()
    }

    /// Update vault view and load cached config.
    fn update_vault_view_and_load(
        &self,
        _vault: RawVaultConfig,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // TODO: Implement view update
        self.load_cached_config()
    }

    /// Rebuild config from raw configs.
    fn rebuild_with_configs(
        &self,
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Merge raw configs with precedence: defaults < global < vault
        let merged = self.merge_raw_configs(global, vault);

        // Determine next version
        let next_version = self
            .repository
            .get_active_version(self.vault_id)
            .map_err(Into::into)?
            .map(|v| v.next())
            .transpose()?
            .unwrap_or_else(Version::initial);

        // Build domain config
        let config = Config::build(
            &merged,
            self.vault_id,
            self.vault_root.clone(),
            next_version,
        )?;

        // Persist config
        self.repository
            .save_config(self.vault_id, &config)
            .map_err(Into::into)?;

        Ok(config)
    }

    /// Merge raw configs with field-level precedence.
    ///
    /// Precedence order (highest wins):
    /// 1. Vault config (highest)
    /// 2. Global config
    /// 3. Defaults (lowest)
    fn merge_raw_configs(
        &self,
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
    ) -> RawConfig {
        // Start with defaults
        let mut merged = RawConfig::default();

        // Layer 1: Apply global overrides
        if let Some(g) = global {
            if g.logging.is_some() {
                merged.logging = g.logging.clone();
            }
            merged.paths = g.paths.clone().into();
            if g.trusted_vaults.is_some() {
                merged.trusted_vaults = g.trusted_vaults.clone();
            }
            if g.frontmatter.is_some() {
                merged.frontmatter = g.frontmatter.clone();
            }
            if g.task.is_some() {
                merged.task = g.task.clone();
            }
        }

        // Layer 2: Apply vault overrides (highest priority)
        if let Some(v) = vault {
            if v.logging.is_some() {
                merged.logging = v.logging.clone();
            }
            // Merge paths properly (vault paths override global paths)
            let global_paths = merged.paths.clone();
            let vault_paths: RawPathsConfig = v.paths.clone().into();
            merged.paths = RawPathsConfig::merge(global_paths, vault_paths);

            if v.frontmatter.is_some() {
                merged.frontmatter = v.frontmatter.clone();
            }
            if v.task.is_some() {
                merged.task = v.task.clone();
            }
        }

        merged
    }
}
