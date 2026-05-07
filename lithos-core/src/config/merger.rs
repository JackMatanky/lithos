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
    /// Handles all 9 combinations of (`UseCached` | `UpdateViewOnly` | Rebuild) ×
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
            .map(super::aggregate::Version::next)
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

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::config::{
        aggregate::Version,
        global::Global,
        processor::ProcessorOutcome,
        raw::{RawGlobalConfig, RawGlobalPaths, RawVaultConfig, RawVaultPaths},
        storage::Repository,
        vault::{Vault, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    };

    // Test-only in-memory storage
    #[derive(Clone)]
    struct TestStorage {
        configs: Arc<Mutex<HashMap<VaultId, Config>>>,
        active_versions: Arc<Mutex<HashMap<VaultId, Version>>>,
    }

    impl TestStorage {
        fn new() -> Self {
            Self {
                configs: Arc::new(Mutex::new(HashMap::new())),
                active_versions: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl Repository for TestStorage {
        type Error = ConfigError;

        fn get_global(&self) -> Result<Option<Global>, Self::Error> {
            Ok(None)
        }

        fn save_global(&self, _global: &Global) -> Result<(), Self::Error> {
            Ok(())
        }

        fn get_vault(
            &self,
            _vault_id: VaultId,
        ) -> Result<Option<Vault>, Self::Error> {
            Ok(None)
        }

        fn save_vault(
            &self,
            _vault_id: VaultId,
            _vault: &Vault,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn get_config(
            &self,
            vault_id: VaultId,
            _version: Version,
        ) -> Result<Option<Config>, Self::Error> {
            let configs = self.configs.lock().map_err(|_| {
                ConfigError::ValidationFailed {
                    field: "storage".into(),
                    message: "Lock poisoned".into(),
                }
            })?;
            Ok(configs.get(&vault_id).cloned())
        }

        fn save_config(
            &self,
            vault_id: VaultId,
            config: &Config,
        ) -> Result<Version, Self::Error> {
            let mut configs = self.configs.lock().map_err(|_| {
                ConfigError::ValidationFailed {
                    field: "storage".into(),
                    message: "Lock poisoned".into(),
                }
            })?;
            let mut versions = self.active_versions.lock().map_err(|_| {
                ConfigError::ValidationFailed {
                    field: "storage".into(),
                    message: "Lock poisoned".into(),
                }
            })?;
            let version = config.version();
            configs.insert(vault_id, config.clone());
            versions.insert(vault_id, version);
            Ok(version)
        }

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, Self::Error> {
            let versions = self.active_versions.lock().map_err(|_| {
                ConfigError::ValidationFailed {
                    field: "storage".into(),
                    message: "Lock poisoned".into(),
                }
            })?;
            Ok(versions.get(&vault_id).copied())
        }

        fn find_vault_id_by_path(
            &self,
            _vault_root: &VaultRoot,
        ) -> Result<Option<VaultId>, Self::Error> {
            Ok(None)
        }

        fn save_vault_path_mapping(
            &self,
            _vault_id: VaultId,
            _vault_root: &VaultRoot,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn with_archived_config<R, F>(
            &self,
            _vault_id: VaultId,
            _version: Version,
            _f: F,
        ) -> Result<Option<R>, Self::Error>
        where
            F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
        {
            Ok(None)
        }

        fn get_raw_global_view(
            &self,
        ) -> Result<Option<RawGlobalConfigView>, Self::Error> {
            Ok(None)
        }

        fn save_raw_global_view(
            &self,
            _view: &RawGlobalConfigView,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn get_raw_vault_view(
            &self,
            _vault_id: VaultId,
        ) -> Result<Option<RawVaultConfigView>, Self::Error> {
            Ok(None)
        }

        fn save_raw_vault_view(
            &self,
            _view: &RawVaultConfigView,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

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
            metadata: Default::default(),
        }
    }

    fn create_test_vault_config() -> RawVaultConfig {
        RawVaultConfig {
            vault_path: "/tmp/test-vault".into(),
            name: None,
            version: None,
            logging: None,
            paths: RawVaultPaths {
                cache_dir: Some(".cache".into()),
                templates_dir: Some("vault-templates".into()),
                schemas_dir: None,
                property_bank_file: None,
            },
            frontmatter: None,
            task: None,
            metadata: Default::default(),
        }
    }

    #[test]
    fn merge_both_use_cached_loads_from_db() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        // Save a config first
        let raw = RawConfig::default();
        let config = Config::build(
            &raw,
            vault_id,
            vault_root.clone(),
            Version::initial(),
        )
        .expect("config build failed");
        storage.save_config(vault_id, &config).expect("save failed");

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
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global_outcome = ProcessorOutcome::UseCached;
        let vault_outcome = ProcessorOutcome::UseCached;

        let result = merger.merge(global_outcome, vault_outcome);
        result.unwrap_err();
    }

    #[test]
    fn merge_global_rebuild_vault_cached_creates_new_config() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let global_outcome = ProcessorOutcome::Rebuild {
            raw: global,
            changed_fields: HashSet::new(),
        };
        let vault_outcome = ProcessorOutcome::UseCached;

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.version().value(), 1);
        // Verify global paths were applied
        assert_eq!(
            config.paths().template.templates_dir.as_path().to_str().unwrap(),
            "global-templates"
        );
    }

    #[test]
    fn merge_vault_rebuild_global_cached_creates_new_config() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let vault = create_test_vault_config();
        let global_outcome = ProcessorOutcome::UseCached;
        let vault_outcome = ProcessorOutcome::Rebuild {
            raw: vault,
            changed_fields: HashSet::new(),
        };

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.version().value(), 1);
        // Verify vault paths were applied
        assert_eq!(
            config.paths().cache.cache_dir().as_path().to_str().unwrap(),
            ".cache"
        );
    }

    #[test]
    fn merge_both_rebuild_merges_with_vault_precedence() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

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

        // Vault templates_dir should override global
        assert_eq!(
            config.paths().template.templates_dir.as_path().to_str().unwrap(),
            "vault-templates"
        );
        // Global schemas_dir should be used (vault didn't specify)
        assert_eq!(
            config.paths().schema.schemas_dir().as_path().to_str().unwrap(),
            "global-schemas"
        );
        // Vault cache_dir should be present
        assert_eq!(
            config.paths().cache.cache_dir().as_path().to_str().unwrap(),
            ".cache"
        );
    }

    #[test]
    fn merge_version_increments_on_rebuild() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        // Save initial config (version 1)
        let raw = RawConfig::default();
        let config1 = Config::build(
            &raw,
            vault_id,
            vault_root.clone(),
            Version::initial(),
        )
        .expect("config build failed");
        storage.save_config(vault_id, &config1).expect("save failed");

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let global_outcome = ProcessorOutcome::Rebuild {
            raw: global,
            changed_fields: HashSet::new(),
        };
        let vault_outcome = ProcessorOutcome::UseCached;

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        let config2 = result.unwrap();
        // Should increment from 1 to 2
        assert_eq!(config2.version().value(), 2);
    }

    #[test]
    fn merge_update_view_only_loads_cached() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        // Save a config first
        let raw = RawConfig::default();
        let config = Config::build(
            &raw,
            vault_id,
            vault_root.clone(),
            Version::initial(),
        )
        .expect("config build failed");
        storage.save_config(vault_id, &config).expect("save failed");

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let vault = create_test_vault_config();

        let global_outcome = ProcessorOutcome::UpdateViewOnly {
            raw: global,
        };
        let vault_outcome = ProcessorOutcome::UpdateViewOnly {
            raw: vault,
        };

        let result = merger.merge(global_outcome, vault_outcome);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        // Should load existing config without incrementing version
        assert_eq!(loaded.version().value(), 1);
    }

    #[test]
    fn merge_raw_configs_with_no_inputs_returns_default_struct() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        // No global or vault - should return RawConfig::default()
        let merged = merger.merge_raw_configs(None, None);

        // RawConfig::default() has default RawPathsConfig
        // Actual path defaults are applied in Config::build()
        assert!(merged.logging.is_none());
        assert!(merged.frontmatter.is_none());
        assert!(merged.task.is_none());
    }

    #[test]
    fn merge_raw_configs_global_overrides_defaults() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let merged = merger.merge_raw_configs(Some(&global), None);

        // Global paths should override defaults
        assert_eq!(
            merged.paths.templates_dir.as_ref().unwrap(),
            "global-templates"
        );
        assert_eq!(
            merged.paths.schemas_dir.as_ref().unwrap(),
            "global-schemas"
        );
    }

    #[test]
    fn merge_raw_configs_vault_overrides_global() {
        let vault_id = VaultId::new();
        let vault_root = create_test_vault_root();
        let storage = TestStorage::new();

        let merger = ConfigMerger::new(vault_id, vault_root, &storage);

        let global = create_test_global_config();
        let vault = create_test_vault_config();
        let merged = merger.merge_raw_configs(Some(&global), Some(&vault));

        // Vault templates_dir should win
        assert_eq!(
            merged.paths.templates_dir.as_ref().unwrap(),
            "vault-templates"
        );
        // Global schemas_dir should remain (vault didn't override)
        assert_eq!(
            merged.paths.schemas_dir.as_ref().unwrap(),
            "global-schemas"
        );
        // Vault cache_dir should be present
        assert_eq!(merged.paths.cache_dir.as_ref().unwrap(), ".cache");
    }
}
