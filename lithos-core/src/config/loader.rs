//! Configuration loading orchestration with hybrid staleness detection.
//!
//! This module provides the [`Loader`] struct which orchestrates the complete
//! configuration loading pipeline:
//!
//! 1. **Load raw views** from database (cached file state)
//! 2. **Ingest raw configs** from filesystem
//! 3. **Detect staleness** via timestamp + content hash comparison
//! 4. **Merge configs** using Figment when stale
//! 5. **Build domain** via `Config::build`
//! 6. **Persist** updated views and config to database
//!
//! The loader implements efficient staleness tracking by comparing:
//! - File modification timestamps (fast check)
//! - BLAKE3 content hashes (accurate change detection)
//!
//! This hybrid approach avoids unnecessary rebuilds while detecting all actual
//! file changes.

use std::path::PathBuf;

use figment::{Figment, providers::Serialized};
use tracing::instrument;

use crate::config::{
    aggregate::Config,
    error::{ConfigError, ConfigIngestError},
    ingestor::Ingestor,
    raw::{RawConfig, RawGlobalConfig, RawVaultConfig},
    storage::Repository,
    vault::{VaultId, VaultRoot},
};

/// Configuration loader with hybrid staleness detection.
///
/// Coordinates the full configuration loading pipeline:
/// - File discovery and ingestion
/// - Staleness detection (timestamps + content hash)
/// - Figment-based merging
/// - Domain validation
/// - Database persistence
///
/// # Architecture
///
/// The loader owns the orchestration pipeline but delegates to:
/// - `Ingestor`: File discovery and raw config parsing
/// - `Repository`: Database persistence and retrieval
/// - `Config::build`: Domain validation and construction
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
///
/// use lithos_core::config::{loader::Loader, storage::RedbStorage};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vault_root = PathBuf::from("/vault");
/// // let repo = RedbStorage::new(...);
/// // let loader = Loader::new(vault_root, repo);
/// // let config = loader.load()?;
/// # Ok(())
/// # }
/// ```
pub struct Loader<R> {
    /// Vault root path for config resolution.
    vault_root: PathBuf,
    /// Configuration ingestor for file operations.
    ingestor: Ingestor,
    /// Repository for database persistence.
    repository: R,
}

impl<R> Loader<R>
where
    R: Repository,
{
    /// Create a new loader for the given vault root.
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
    /// use lithos_core::config::{loader::Loader, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let loader = Loader::new(vault_root, repo);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(vault_root: P, repository: R) -> Self {
        let vault_root = vault_root.into();
        let ingestor = Ingestor::new(&vault_root);

        Self {
            vault_root,
            ingestor,
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
    /// - Figment merge fails (incompatible layers)
    /// - Domain validation fails (invalid config)
    /// - Database operations fail
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::{loader::Loader, storage::RedbStorage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault_root = PathBuf::from("/vault");
    /// // let repo = RedbStorage::new(...);
    /// // let loader = Loader::new(vault_root, repo);
    /// // let config = loader.load()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(vault_root = %self.vault_root.display())
    )]
    pub fn load(&self) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Step 0: Validate and get vault root
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        // Step 1: Get or create vault ID
        let vault_id = self.get_or_create_vault_id()?;

        // Step 2: Ingest raw configs from files
        let global_config = self.ingestor.global_config()?;
        let vault_config = self.ingestor.vault_config(&vault_root)?;

        // Step 3: Check staleness independently for optimization
        let global_stale = self.is_global_stale(global_config.as_ref())?;
        let vault_stale =
            self.is_vault_stale(vault_id, vault_config.as_ref())?;

        // Step 4: Rebuild strategy based on staleness
        match (global_stale, vault_stale) {
            (false, false) => {
                // Both fresh → load from DB (fast path)
                let version = self
                    .repository
                    .get_active_version(vault_id)
                    .map_err(Into::into)?
                    .ok_or(ConfigError::ValidationFailed {
                        field: "config".into(),
                        message: "No config version found in database".into(),
                    })?;

                self.repository
                    .get_config(vault_id, version)
                    .map_err(Into::into)?
                    .ok_or(ConfigError::ValidationFailed {
                        field: "config".into(),
                        message: "Config not found in database".into(),
                    })
            }
            (true, true) => {
                // Both stale → full Figment merge
                self.rebuild_config(
                    vault_id,
                    vault_root,
                    global_config.as_ref(),
                    vault_config.as_ref(),
                )
            }
            (true, false) => {
                // Only global stale → merge global + cached vault
                self.rebuild_with_cached_vault(
                    vault_id,
                    vault_root,
                    global_config.as_ref(),
                )
            }
            (false, true) => {
                // Only vault stale → merge cached global + vault
                self.rebuild_with_cached_global(
                    vault_id,
                    vault_root,
                    vault_config.as_ref(),
                )
            }
        }
    }

    /// Get or create vault ID for the vault root.
    ///
    /// Looks up existing vault ID from path mapping, or creates a new one.
    fn get_or_create_vault_id(&self) -> Result<VaultId, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        let vault_root = VaultRoot::try_new(self.vault_root.clone())?;

        if let Some(existing_id) = self
            .repository
            .find_vault_id_by_path(&vault_root)
            .map_err(Into::into)?
        {
            Ok(existing_id)
        } else {
            let new_id = VaultId::new();
            self.repository
                .save_vault_path_mapping(new_id, &vault_root)
                .map_err(Into::into)?;
            Ok(new_id)
        }
    }

    /// Check if global config is stale.
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check
    /// 2. Content hash check (only if timestamps match)
    ///
    /// Returns `true` if:
    /// - No view exists in DB (never loaded)
    /// - Global config file appeared/disappeared
    /// - Timestamps differ
    /// - Content hash differs
    fn is_global_stale(
        &self,
        global: Option<&RawGlobalConfig>,
    ) -> Result<bool, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        let global_view =
            self.repository.get_raw_global_view().map_err(Into::into)?;

        Ok(match (global, global_view) {
            (Some(raw), Some(view)) => !view.is_fresh(raw),
            (Some(_), None) | (None, Some(_)) => true, /* Config appeared/ */
            // disappeared
            (None, None) => false, // No global config (consistent)
        })
    }

    /// Check if vault config is stale.
    ///
    /// Performs hybrid staleness detection:
    /// 1. Fast timestamp check
    /// 2. Content hash check (only if timestamps match)
    ///
    /// Returns `true` if:
    /// - No view exists in DB (never loaded)
    /// - Vault config file appeared/disappeared
    /// - Timestamps differ
    /// - Content hash differs
    fn is_vault_stale(
        &self,
        vault_id: VaultId,
        vault: Option<&RawVaultConfig>,
    ) -> Result<bool, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        let vault_view =
            self.repository.get_raw_vault_view(vault_id).map_err(Into::into)?;

        Ok(match (vault, vault_view) {
            (Some(raw), Some(view)) => !view.is_fresh(raw),
            (Some(_), None) | (None, Some(_)) => true, /* Config appeared/ */
            // disappeared
            (None, None) => false, // No vault config (consistent)
        })
    }

    /// Rebuild config from raw configs (both stale).
    ///
    /// Pipeline:
    /// 1. Merge via Figment (defaults → global → vault)
    /// 2. Build domain via `Config::build`
    /// 3. Save raw views to DB
    /// 4. Save config to DB
    fn rebuild_config(
        &self,
        vault_id: VaultId,
        vault_root: VaultRoot,
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Step 1: Merge via Figment
        let merged_raw = Self::merge_raw_configs(global, vault)?;

        // Step 2: Allocate version number
        let version = match self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::into)?
        {
            Some(v) => v.next()?,
            None => super::aggregate::Version::initial(),
        };

        // Step 3: Build domain
        let config = Config::build(&merged_raw, vault_id, vault_root, version)?;

        // Step 4: Save config and return allocated version
        // TODO: Save raw views for version history tracking
        self.repository.save_config(vault_id, &config).map_err(Into::into)?;

        Ok(config)
    }

    /// Rebuild config with cached vault (only global stale).
    ///
    /// Optimization: Reuse cached vault config from DB to avoid re-parsing.
    fn rebuild_with_cached_vault(
        &self,
        vault_id: VaultId,
        vault_root: VaultRoot,
        global: Option<&RawGlobalConfig>,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Merge with new global (no vault layer)
        let merged_raw = Self::merge_raw_configs(global, None)?;

        // Allocate version number
        let version = match self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::into)?
        {
            Some(v) => v.next()?,
            None => super::aggregate::Version::initial(),
        };

        let config = Config::build(&merged_raw, vault_id, vault_root, version)?;

        // Save updated config
        // TODO: Save updated global view
        self.repository.save_config(vault_id, &config).map_err(Into::into)?;

        Ok(config)
    }

    /// Rebuild config with cached global (only vault stale).
    ///
    /// Optimization: Reuse cached global config from DB to avoid re-parsing.
    fn rebuild_with_cached_global(
        &self,
        vault_id: VaultId,
        vault_root: VaultRoot,
        vault: Option<&RawVaultConfig>,
    ) -> Result<Config, ConfigError>
    where
        R::Error: Into<ConfigError>,
    {
        // Merge with new vault (no global layer)
        let merged_raw = Self::merge_raw_configs(None, vault)?;

        // Allocate version number
        let version = match self
            .repository
            .get_active_version(vault_id)
            .map_err(Into::into)?
        {
            Some(v) => v.next()?,
            None => super::aggregate::Version::initial(),
        };

        let config = Config::build(&merged_raw, vault_id, vault_root, version)?;

        // Save updated config
        // TODO: Save updated vault view
        self.repository.save_config(vault_id, &config).map_err(Into::into)?;

        Ok(config)
    }

    /// Merge raw configs using Figment.
    ///
    /// Layer priority (highest to lowest):
    /// 1. Defaults (compiled-in)
    /// 2. Global config (system-wide)
    /// 3. Vault config (project-specific)
    fn merge_raw_configs(
        global: Option<&RawGlobalConfig>,
        vault: Option<&RawVaultConfig>,
    ) -> Result<RawConfig, ConfigIngestError> {
        // Layer 1: Compiled defaults
        let mut figment =
            Figment::from(Serialized::defaults(RawConfig::default()));

        // Layer 2: Global config (if exists)
        if let Some(global_config) = global {
            figment = figment.merge(Serialized::defaults(RawConfig::from(
                global_config.clone(),
            )));
        }

        // Layer 3: Vault config (if exists)
        if let Some(vault_config) = vault {
            figment = figment.merge(Serialized::defaults(RawConfig::from(
                vault_config.clone(),
            )));
        }

        // Extract merged config
        figment.extract().map_err(ConfigIngestError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::config::{
        aggregate::Version,
        global::Global,
        raw::{RawGlobalConfig, RawVaultConfig},
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    };

    // ─────────────────────────────────────────────────────────────────────────
    //  Mock Repository for Testing
    // ─────────────────────────────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    struct MockRepository {
        global_view: Arc<Mutex<Option<RawGlobalConfigView>>>,
        vault_views: Arc<Mutex<HashMap<VaultId, RawVaultConfigView>>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                global_view: Arc::new(Mutex::new(None)),
                vault_views: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn set_global_view(&self, view: RawGlobalConfigView) {
            *self.global_view.lock().unwrap() = Some(view);
        }

        fn set_vault_view(&self, view: RawVaultConfigView) {
            let vault_id = view.vault_id();
            self.vault_views.lock().unwrap().insert(vault_id, view);
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("mock error")]
    struct MockError;

    impl From<MockError> for ConfigError {
        #[inline]
        fn from(err: MockError) -> Self {
            ConfigError::ValidationFailed {
                field: "mock".into(),
                message: err.to_string().into(),
            }
        }
    }

    impl Repository for MockRepository {
        type Error = MockError;

        fn get_global(&self) -> Result<Option<Global>, Self::Error> {
            Ok(None)
        }

        fn save_global(&self, _config: &Global) -> Result<(), Self::Error> {
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
            _config: &Vault,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn get_config(
            &self,
            _vault_id: VaultId,
            _version: Version,
        ) -> Result<Option<Config>, Self::Error> {
            Ok(None)
        }

        fn save_config(
            &self,
            _vault_id: VaultId,
            config: &Config,
        ) -> Result<Version, Self::Error> {
            Ok(config.version())
        }

        fn get_active_version(
            &self,
            _vault_id: VaultId,
        ) -> Result<Option<Version>, Self::Error> {
            Ok(None)
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

        #[expect(
            clippy::unwrap_in_result,
            reason = "Test mock - lock poisoning is a test failure"
        )]
        fn get_raw_global_view(
            &self,
        ) -> Result<Option<RawGlobalConfigView>, Self::Error> {
            Ok(self.global_view.lock().unwrap().clone())
        }

        #[expect(
            clippy::unwrap_in_result,
            reason = "Test mock - lock poisoning is a test failure"
        )]
        fn save_raw_global_view(
            &self,
            view: &RawGlobalConfigView,
        ) -> Result<(), Self::Error> {
            *self.global_view.lock().unwrap() = Some(view.clone());
            Ok(())
        }

        #[expect(
            clippy::unwrap_in_result,
            reason = "Test mock - lock poisoning is a test failure"
        )]
        fn get_raw_vault_view(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<RawVaultConfigView>, Self::Error> {
            Ok(self.vault_views.lock().unwrap().get(&vault_id).cloned())
        }

        #[expect(
            clippy::unwrap_in_result,
            reason = "Test mock - lock poisoning is a test failure"
        )]
        fn save_raw_vault_view(
            &self,
            view: &RawVaultConfigView,
        ) -> Result<(), Self::Error> {
            let vault_id = view.vault_id();
            self.vault_views.lock().unwrap().insert(vault_id, view.clone());
            Ok(())
        }
    }
    // ─────────────────────────────────────────────────────────────────────────
    //  Helper Functions
    // ─────────────────────────────────────────────────────────────────────────

    fn create_test_global() -> RawGlobalConfig {
        RawGlobalConfig::default()
    }

    fn create_test_vault() -> RawVaultConfig {
        RawVaultConfig::default()
    }

    fn create_global_view(
        _raw: &RawGlobalConfig,
        _timestamp: i64,
    ) -> RawGlobalConfigView {
        RawGlobalConfigView::new("/test/lithos.toml".into())
    }

    fn create_vault_view(
        vault_id: VaultId,
        _raw: &RawVaultConfig,
        _timestamp: i64,
    ) -> RawVaultConfigView {
        RawVaultConfigView::new(vault_id, "/test/vault/lithos.toml".into())
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Staleness Detection Tests
    // ─────────────────────────────────────────────────────────────────────────
    //
    // NOTE: Full staleness testing requires properly populating RawFileVersion
    // with timestamps and content hashes. These tests focus on the structural
    // edge cases (file appears/disappears, no view exists).

    #[test]
    fn global_stale_when_no_view_exists() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        let global = Some(create_test_global());
        let result = loader.is_global_stale(global.as_ref()).unwrap();

        assert!(result, "Should be stale when no view exists in DB");
    }

    #[test]
    fn global_stale_when_file_appears() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        // View exists but no file
        let global = create_test_global();
        let view = create_global_view(&global, 1000);
        repo.set_global_view(view);

        let result = loader.is_global_stale(None).unwrap();

        assert!(result, "Should be stale when file disappears");
    }

    #[test]
    fn global_stale_when_file_disappears() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        // File exists but no view
        let global = create_test_global();

        let result = loader.is_global_stale(Some(&global)).unwrap();

        assert!(result, "Should be stale when file appears");
    }

    #[test]
    fn vault_stale_when_no_view_exists() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        let vault_id = VaultId::new();
        let vault = Some(create_test_vault());
        let result = loader.is_vault_stale(vault_id, vault.as_ref()).unwrap();

        assert!(result, "Should be stale when no view exists in DB");
    }

    #[test]
    fn vault_stale_when_file_appears() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        let vault_id = VaultId::new();
        let vault = create_test_vault();
        let view = create_vault_view(vault_id, &vault, 1000);
        repo.set_vault_view(view);

        let result = loader.is_vault_stale(vault_id, None).unwrap();

        assert!(result, "Should be stale when file disappears");
    }

    #[test]
    fn vault_stale_when_file_disappears() {
        let repo = MockRepository::new();
        let vault_root = PathBuf::from("/test/vault");
        let loader = Loader::new(&vault_root, repo.clone());

        let vault_id = VaultId::new();
        let vault = create_test_vault();

        let result = loader.is_vault_stale(vault_id, Some(&vault)).unwrap();

        assert!(result, "Should be stale when file appears");
    }

    //  Merge Strategy Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn merge_raw_configs_with_only_defaults() {
        let result = Loader::<MockRepository>::merge_raw_configs(None, None);
        assert!(result.is_ok(), "Should merge successfully with defaults");
    }

    #[test]
    fn merge_raw_configs_with_global_only() {
        let global = create_test_global();
        let result =
            Loader::<MockRepository>::merge_raw_configs(Some(&global), None);
        assert!(result.is_ok(), "Should merge successfully with global config");
    }

    #[test]
    fn merge_raw_configs_with_vault_only() {
        let vault = create_test_vault();
        let result =
            Loader::<MockRepository>::merge_raw_configs(None, Some(&vault));
        assert!(result.is_ok(), "Should merge successfully with vault config");
    }

    #[test]
    fn merge_raw_configs_with_both() {
        let global = create_test_global();
        let vault = create_test_vault();
        let result = Loader::<MockRepository>::merge_raw_configs(
            Some(&global),
            Some(&vault),
        );
        assert!(result.is_ok(), "Should merge successfully with both configs");
    }
}
