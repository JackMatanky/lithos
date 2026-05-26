//! Testing and benchmarking utilities for the config module.
//!
//! This module provides test doubles and benchmark fixtures for config
//! components. Code in this module is compiled for both `#[cfg(test)]`
//! and benchmarks.
//!
//! # Available Utilities
//!
//! - [`InMemoryRepository`] - HashMap-backed Repository for pure unit tests
//! - Test helpers for building test data
//!
//! # Design Rationale
//!
//! This module exists to enable **pure unit tests** following matklad's
//! test purity hierarchy:
//!
//! - **Pure computation** (fastest, most reliable)
//! - Threads → Filesystem → Network → Processes (slowest, least reliable)
//!
//! By providing an in-memory Repository implementation, we eliminate filesystem
//! IO from unit tests while maintaining test extent (can still test full
//! pipelines end-to-end).
//!
//! # When to Use
//!
//! - **Unit tests** (`#[cfg(test)]` modules): Use `InMemoryRepository`
//! - **Integration tests** (`tests/` directory): Use `RedbRepository`
//! - **Benchmarks**: Use `InMemoryRepository` for micro-benchmarks

// Test-only code: relax pedantic lints for pragmatic test utilities
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::map_err_ignore,
    clippy::significant_drop_tightening,
    unfulfilled_lint_expectations,
    reason = "Test utilities prioritize readability over micro-optimizations"
)]

use std::{collections::HashMap, sync::RwLock};

use crate::{
    config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        repository::{ReadRepository, WriteRepository},
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    db::testing::{FailurePoint, InMemoryHarness, read_lock, write_lock},
};

// ─────────────────────────────────────────────────────────────────────────────
// InMemoryRepository
// ─────────────────────────────────────────────────────────────────────────────

/// HashMap-backed Repository for pure unit tests.
///
/// This enables fast, deterministic, side-effect-free tests that exercise
/// the full config pipeline without touching the filesystem.
#[allow(clippy::type_complexity, reason = "Internal state uses nested maps")]
pub struct InMemoryRepository {
    globals: RwLock<Option<Global>>,
    vaults: RwLock<HashMap<VaultId, Vault>>,
    configs: RwLock<HashMap<(VaultId, Version), Config>>,
    active_versions: RwLock<HashMap<VaultId, Version>>,
    global_views: RwLock<Option<RawGlobalConfigView>>,
    vault_views: RwLock<HashMap<VaultId, RawVaultConfigView>>,
    vault_id_mappings: RwLock<HashMap<VaultRoot, VaultId>>,
    harness: InMemoryHarness,
}

impl InMemoryRepository {
    /// Creates an empty in-memory repository.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            globals: RwLock::new(None),
            vaults: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
            active_versions: RwLock::new(HashMap::new()),
            global_views: RwLock::new(None),
            vault_views: RwLock::new(HashMap::new()),
            vault_id_mappings: RwLock::new(HashMap::new()),
            harness: InMemoryHarness::new(),
        }
    }

    /// Creates a new repository with the specified failure injector.
    #[inline]
    #[must_use]
    pub fn with_injector(
        injector: Box<dyn crate::db::testing::FailureInjector + Send + Sync>,
    ) -> Self {
        let mut repo = Self::new();
        repo.harness = InMemoryHarness::with_injector(injector);
        repo
    }

    /// Returns a reference to the operation counters.
    #[inline]
    #[must_use]
    pub fn counters(&self) -> &crate::db::testing::OpCounters {
        self.harness.counters()
    }
}

impl ReadRepository for InMemoryRepository {
    #[inline]
    fn get_global(&self) -> Result<Option<Global>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let globals = read_lock::<Option<Global>>(&self.globals, "get_global")?;
        self.harness.counters().inc_read();
        Ok(globals.clone())
    }

    #[inline]
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let vaults =
            read_lock::<HashMap<VaultId, Vault>>(&self.vaults, "get_vault")?;
        self.harness.counters().inc_read();
        Ok(vaults.get(&vault_id).cloned())
    }

    #[inline]
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let configs = read_lock::<HashMap<(VaultId, Version), Config>>(
            &self.configs,
            "get_config",
        )?;
        self.harness.counters().inc_read();
        Ok(configs.get(&(vault_id, version)).cloned())
    }

    #[inline]
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let versions = read_lock::<HashMap<VaultId, Version>>(
            &self.active_versions,
            "get_active_version",
        )?;
        self.harness.counters().inc_read();
        Ok(versions.get(&vault_id).copied())
    }

    #[inline]
    fn with_archived_config<R, F>(
        &self,
        _vault_id: VaultId,
        _version: Version,
        _f: F,
    ) -> Result<Option<R>, ConfigRepositoryError>
    where
        F: for<'archived> FnOnce(
            &'archived rkyv::Archived<crate::config::aggregate::Config>,
        ) -> R,
    {
        Ok(None)
    }

    #[inline]
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let mappings = read_lock::<HashMap<VaultRoot, VaultId>>(
            &self.vault_id_mappings,
            "find_vault_id_by_path",
        )?;
        self.harness.counters().inc_read();
        Ok(mappings.get(vault_root).copied())
    }

    #[inline]
    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let views = read_lock::<Option<RawGlobalConfigView>>(
            &self.global_views,
            "get_raw_global_view",
        )?;
        self.harness.counters().inc_read();
        Ok(views.clone())
    }

    #[inline]
    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        let views = read_lock::<HashMap<VaultId, RawVaultConfigView>>(
            &self.vault_views,
            "get_raw_vault_view",
        )?;
        self.harness.counters().inc_read();
        Ok(views.get(&vault_id).cloned())
    }
}

impl WriteRepository for InMemoryRepository {
    #[inline]
    fn save_global(
        &self,
        config: &Global,
    ) -> Result<(), ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut globals =
            write_lock::<Option<Global>>(&self.globals, "save_global")?;
        self.harness.counters().inc_write();
        *globals = Some(config.clone());
        Ok(())
    }

    #[inline]
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut vaults =
            write_lock::<HashMap<VaultId, Vault>>(&self.vaults, "save_vault")?;
        self.harness.counters().inc_write();
        vaults.insert(vault_id, config.clone());
        Ok(())
    }

    #[inline]
    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut configs = write_lock::<HashMap<(VaultId, Version), Config>>(
            &self.configs,
            "save_config",
        )?;
        let mut versions = write_lock::<HashMap<VaultId, Version>>(
            &self.active_versions,
            "save_config_version",
        )?;
        self.harness.counters().inc_write();
        let version = *config.version();
        configs.insert((vault_id, version), config.clone());
        versions.insert(vault_id, version);
        Ok(version)
    }

    #[inline]
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut mappings = write_lock::<HashMap<VaultRoot, VaultId>>(
            &self.vault_id_mappings,
            "save_vault_path_mapping",
        )?;
        self.harness.counters().inc_write();
        mappings.insert(vault_root.clone(), vault_id);
        Ok(())
    }

    #[inline]
    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut views = write_lock::<Option<RawGlobalConfigView>>(
            &self.global_views,
            "save_raw_global_view",
        )?;
        self.harness.counters().inc_write();
        *views = Some(view.clone());
        Ok(())
    }

    #[inline]
    fn save_raw_vault_view(
        &self,
        vault_id: VaultId,
        view: &RawVaultConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        let mut views = write_lock::<HashMap<VaultId, RawVaultConfigView>>(
            &self.vault_views,
            "save_raw_vault_view",
        )?;
        self.harness.counters().inc_write();
        views.insert(vault_id, view.clone());
        Ok(())
    }
}

impl Default for InMemoryRepository {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::{FailureInjector, FailurePoint, InMemoryDbError};

    mod lookup {
        use super::*;

        #[test]
        fn global_roundtrip() {
            let repo = InMemoryRepository::new();
            let global = Global::default();

            repo.save_global(&global).expect("Save global failed");
            let retrieved = repo.get_global().expect("Get global failed");

            assert_eq!(retrieved, Some(global));
        }

        #[test]
        fn vault_roundtrip() {
            let repo = InMemoryRepository::new();
            let vault_id = VaultId::new();
            let vault = Vault::default();

            repo.save_vault(vault_id, &vault).expect("Save vault failed");
            let retrieved = repo.get_vault(vault_id).expect("Get vault failed");

            assert_eq!(retrieved, Some(vault));
        }

        #[test]
        fn config_retrieval() {
            let repo = InMemoryRepository::new();
            let vault_id = VaultId::new();
            let config = crate::config::aggregate::fixtures::test_config();
            let version = *config.version();

            repo.save_config(vault_id, &config).expect("Save config failed");
            let retrieved =
                repo.get_config(vault_id, version).expect("Get config failed");

            assert_eq!(retrieved, Some(config));
        }
    }

    mod update {
        use super::*;

        #[test]
        fn save_config_allocates_version() {
            let repo = InMemoryRepository::new();
            let vault_id = VaultId::new();
            let config = crate::config::aggregate::fixtures::test_config();
            let version = *config.version();

            let saved_version = repo
                .save_config(vault_id, &config)
                .expect("Save config failed");
            assert_eq!(saved_version, version);

            let active = repo
                .get_active_version(vault_id)
                .expect("Get active version failed");
            assert_eq!(active, Some(version));
        }
    }

    mod counters {
        use super::*;

        #[test]
        fn increments_on_ops() {
            let repo = InMemoryRepository::new();
            let vault_id = VaultId::new();

            repo.get_global().unwrap();
            repo.save_global(&Global::default()).unwrap();
            repo.get_vault(vault_id).unwrap();
            repo.save_vault(vault_id, &Vault::default()).unwrap();

            let snapshot = repo.counters().snapshot();
            assert_eq!(snapshot.reads, 2);
            assert_eq!(snapshot.writes, 2);
        }
    }

    mod injection {
        use super::*;

        struct AlwaysFail;
        impl FailureInjector for AlwaysFail {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                Err(InMemoryDbError::InjectedFailure {
                    point,
                    reason: "forced".into(),
                })
            }
        }

        #[test]
        fn returns_error_on_injected_failure() {
            let repo = InMemoryRepository::with_injector(Box::new(AlwaysFail));

            let res = repo.get_global();
            assert!(res.is_err());

            let res_save = repo.save_global(&Global::default());
            assert!(res_save.is_err());
        }
    }
}
