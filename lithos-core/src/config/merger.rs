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
pub struct ConfigMerger<'repo, R> {
    vault_id: VaultId,
    vault_root: VaultRoot,
    repository: &'repo R,
}

impl<'repo, R> ConfigMerger<'repo, R>
where
    R: Repository,
{
    /// Create a new merger.
    #[inline]
    #[must_use]
    pub fn new(
        vault_id: VaultId,
        vault_root: VaultRoot,
        repository: &'repo R,
    ) -> Self {
        Self {
            vault_id,
            vault_root,
            repository,
        }
    }

    /// Merge processor outcomes into final Config.
    ///
    /// Handles all 9 combinations of (`UseCached` | `UpdateViewOnly` | Rebuild)
    /// × 2.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Merging fails (incompatible configs)
    /// - Domain validation fails
    /// - Database persistence fails
    #[inline]
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
        let result = self.merge_raw_configs(global, vault);

        // Determine next version
        let next_version = self
            .repository
            .get_active_version(self.vault_id)
            .map_err(Into::into)?
            .map(super::aggregate::Version::next)
            .transpose()?
            .unwrap_or_else(Version::initial);

        // Build domain config
        let config = Config::build(
            &result,
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
    #[expect(
        clippy::unused_self,
        reason = "Method keeps self for API consistency with other \
                  ConfigMerger methods"
    )]
    fn merge_raw_configs(
        &self,
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
    ) -> RawConfig {
        // Start with defaults
        let mut result = RawConfig::default();

        // Layer 1: Apply global overrides
        if let Some(g) = global {
            if g.logging.is_some() {
                result.logging.clone_from(&g.logging);
            }
            result.paths = g.paths.clone().into();
            if g.trusted_vaults.is_some() {
                result.trusted_vaults.clone_from(&g.trusted_vaults);
            }
            if g.frontmatter.is_some() {
                result.frontmatter.clone_from(&g.frontmatter);
            }
            if g.task.is_some() {
                result.task.clone_from(&g.task);
            }
        }

        // Layer 2: Apply vault overrides (highest priority)
        if let Some(v) = vault {
            if v.logging.is_some() {
                result.logging.clone_from(&v.logging);
            }
            // Merge paths properly (vault paths override global paths)
            let global_paths = result.paths.clone();
            let vault_paths: RawPathsConfig = v.paths.clone().into();
            result.paths = RawPathsConfig::merge(global_paths, vault_paths);

            if v.frontmatter.is_some() {
                result.frontmatter.clone_from(&v.frontmatter);
            }
            if v.task.is_some() {
                result.task.clone_from(&v.task);
            }
        }

        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::config::{
        aggregate::{Config, Version},
        processor::ProcessorOutcome,
        raw::{
            RawConfigMetadata, RawGlobalConfig, RawGlobalPaths, RawVaultConfig,
            RawVaultPaths,
        },
        testing::InMemoryRepository,
        vault::VaultRoot,
    };

    fn create_test_vault_root() -> VaultRoot {
        VaultRoot::try_new("/tmp/test-vault".into())
            .expect("vault root creation failed")
    }

    fn create_test_global_config() -> RawGlobalConfig {
        RawGlobalConfig {
            logging: None,
            paths: RawGlobalPaths {
                templates_dir: Some("global-templates".into()),
                schemas_dir: Some("global-schemas".into()),
                property_bank_file: None,
            },
            trusted_vaults: None,
            frontmatter: None,
            task: None,
            metadata: RawConfigMetadata::default(),
        }
    }

    fn create_test_vault_config() -> RawVaultConfig {
        RawVaultConfig {
            vault_path: "/tmp/test-vault".into(),
            name: None,
            version: None,
            logging: None,
            paths: RawVaultPaths {
                templates_dir: Some("vault-templates".into()),
                schemas_dir: None, // Will inherit from global
                cache_dir: Some(".cache".into()),
                property_bank_file: None,
            },
            frontmatter: None,
            task: None,
            metadata: RawConfigMetadata::default(),
        }
    }

    #[test]
    fn merge_raw_configs_with_no_inputs_returns_default_struct() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        // No global or vault - should return RawConfig::default()
        let result = merger.merge_raw_configs(None, None);

        // RawConfig::default() has default RawPathsConfig
        // Actual path defaults are applied in Config::build()
        assert!(result.logging.is_none());
        assert!(result.frontmatter.is_none());
        assert!(result.task.is_none());
    }

    #[test]
    fn merge_raw_configs_global_overrides_defaults() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let result = merger.merge_raw_configs(Some(&global), None);

        // Global paths should override defaults
        assert_eq!(
            result.paths.templates_dir.as_ref().unwrap(),
            "global-templates"
        );
        assert_eq!(
            result.paths.schemas_dir.as_ref().unwrap(),
            "global-schemas"
        );
    }

    #[test]
    fn merge_raw_configs_vault_overrides_global() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let vault = create_test_vault_config();
        let result = merger.merge_raw_configs(Some(&global), Some(&vault));

        // Vault templates_dir should win
        assert_eq!(
            result.paths.templates_dir.as_ref().unwrap(),
            "vault-templates"
        );
        // Global schemas_dir should remain (vault didn't override)
        assert_eq!(
            result.paths.schemas_dir.as_ref().unwrap(),
            "global-schemas"
        );
        // Vault cache_dir should be present
        assert_eq!(result.paths.cache_dir.as_ref().unwrap(), ".cache");
    }

    #[test]
    fn merge_both_use_cached_loads_from_db() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        // Pre-save a config
        let config = create_test_global_config();
        storage
            .save_config(
                vault_id,
                &Config::build(
                    &RawConfig::from(config),
                    vault_id,
                    vault_root.clone(),
                    Version::initial(),
                )
                .unwrap(),
            )
            .expect("Failed to save config");

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global_outcome = ProcessorOutcome::UseCached;
        let vault_outcome = ProcessorOutcome::UseCached;

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.version().value(), 1);
    }

    #[test]
    fn merge_both_use_cached_no_config_returns_error() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global_outcome = ProcessorOutcome::UseCached;
        let vault_outcome = ProcessorOutcome::UseCached;

        let result = merger.merge(global_outcome, vault_outcome);
        result.unwrap_err();
    }

    #[test]
    fn merge_both_rebuild_merges_with_vault_precedence() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root.clone(), &storage);

        let global = create_test_global_config();
        let vault = create_test_vault_config();

        let global_outcome = ProcessorOutcome::Rebuild {
            raw: global,
            changed_fields: HashSet::new(),
        };
        let vault_outcome = ProcessorOutcome::Rebuild {
            raw: vault,
            changed_fields: HashSet::new(),
        };

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());

        let config = result.unwrap();
        // Vault should win for templates_dir
        assert_eq!(
            config
                .paths()
                .template
                .templates_dir
                .as_ref()
                .display()
                .to_string(),
            "vault-templates"
        );

        let global_outcome2 = ProcessorOutcome::UpdateViewOnly {
            raw: create_test_global_config(),
        };
        let vault_outcome2 = ProcessorOutcome::UpdateViewOnly {
            raw: create_test_vault_config(),
        };

        // Create new merger since merge() consumes self
        let merger2 = ConfigMerger::new(vault_id, vault_root, &storage);
        let result2 = merger2.merge(global_outcome2, vault_outcome2);
        assert!(result2.is_ok());
        let loaded = result2.unwrap();
        assert_eq!(loaded.version().value(), 1);
    }

    #[test]
    fn merge_version_increments_on_rebuild() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = InMemoryRepository::new();

        let merger = ConfigMerger::new(vault_id, vault_root.clone(), &storage);

        // Save initial config
        let config = create_test_global_config();
        storage
            .save_config(
                vault_id,
                &Config::build(
                    &RawConfig::from(config.clone()),
                    vault_id,
                    vault_root.clone(),
                    Version::initial(),
                )
                .unwrap(),
            )
            .expect("Failed to save initial config");

        // Rebuild with both configs
        let vault = create_test_vault_config();
        let global_outcome = ProcessorOutcome::Rebuild {
            raw: config,
            changed_fields: HashSet::new(),
        };
        let vault_outcome = ProcessorOutcome::Rebuild {
            raw: vault,
            changed_fields: HashSet::new(),
        };

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version().value(), 2);
    }
}
