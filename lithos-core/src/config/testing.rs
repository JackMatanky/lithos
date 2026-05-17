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
#![expect(
    clippy::missing_inline_in_public_items,
    clippy::map_err_ignore,
    clippy::significant_drop_tightening,
    reason = "Test utilities prioritize readability over micro-optimizations"
)]

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{
    aggregate::{Config, Version},
    global::Global,
    storage::Repository,
    vault::{Vault, VaultId, VaultRoot},
};
use crate::config::error::ConfigError;

// ─────────────────────────────────────────────────────────────────────────────
// InMemoryRepository
// ─────────────────────────────────────────────────────────────────────────────

/// HashMap-backed Repository for pure unit tests.
///
/// This enables fast, deterministic, side-effect-free tests that exercise
/// the full config pipeline without touching the filesystem.
///
/// # Example
///
/// ```rust,ignore
/// use crate::config::testing::InMemoryRepository;
///
/// let repo = InMemoryRepository::new();
/// // Use repo in config processor/merger tests
/// ```
pub struct InMemoryRepository {
    globals: Arc<RwLock<HashMap<VaultId, Global>>>,
    vaults: Arc<RwLock<HashMap<VaultId, Vault>>>,
    configs: Arc<RwLock<HashMap<VaultId, Config>>>,
    active_versions: Arc<RwLock<HashMap<VaultId, Version>>>,
    global_views: Arc<RwLock<Option<super::views::RawGlobalConfigView>>>,
    vault_views:
        Arc<RwLock<HashMap<VaultId, super::views::RawVaultConfigView>>>,
    vault_id_mappings: Arc<RwLock<HashMap<VaultRoot, VaultId>>>,
}

impl InMemoryRepository {
    /// Creates an empty in-memory repository.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            globals: Arc::new(RwLock::new(HashMap::new())),
            vaults: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            active_versions: Arc::new(RwLock::new(HashMap::new())),
            global_views: Arc::new(RwLock::new(None)),
            vault_views: Arc::new(RwLock::new(HashMap::new())),
            vault_id_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Repository for InMemoryRepository {
    type Error = ConfigError;

    fn get_global(&self) -> Result<Option<Global>, Self::Error> {
        let globals =
            self.globals.read().map_err(|_| ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            })?;
        Ok(globals.get(&VaultId::new()).cloned())
    }

    fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
        let mut globals = self.globals.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        globals.insert(VaultId::new(), config.clone());
        Ok(())
    }

    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error> {
        let vaults =
            self.vaults.read().map_err(|_| ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            })?;
        Ok(vaults.get(&vault_id).cloned())
    }

    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error> {
        let mut vaults =
            self.vaults.write().map_err(|_| ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            })?;
        vaults.insert(vault_id, config.clone());
        Ok(())
    }

    fn get_config(
        &self,
        vault_id: VaultId,
        _version: Version,
    ) -> Result<Option<Config>, Self::Error> {
        let configs =
            self.configs.read().map_err(|_| ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            })?;
        Ok(configs.get(&vault_id).cloned())
    }

    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error> {
        let mut configs = self.configs.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        let mut versions = self.active_versions.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        let version = *config.version();
        configs.insert(vault_id, config.clone());
        versions.insert(vault_id, version);
        Ok(version)
    }

    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error> {
        let versions = self.active_versions.read().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        Ok(versions.get(&vault_id).copied())
    }

    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, Self::Error> {
        let mappings = self.vault_id_mappings.read().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        Ok(mappings.get(vault_root).copied())
    }

    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error> {
        let mut mappings = self.vault_id_mappings.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        mappings.insert(vault_root.clone(), vault_id);
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
    ) -> Result<Option<super::views::RawGlobalConfigView>, Self::Error> {
        let views = self.global_views.read().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        Ok(views.clone())
    }

    fn save_raw_global_view(
        &self,
        view: &super::views::RawGlobalConfigView,
    ) -> Result<(), Self::Error> {
        let mut views = self.global_views.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        *views = Some(view.clone());
        Ok(())
    }

    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<super::views::RawVaultConfigView>, Self::Error> {
        let views = self.vault_views.read().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
        Ok(views.get(&vault_id).cloned())
    }

    fn save_raw_vault_view(
        &self,
        vault_id: VaultId,
        view: &super::views::RawVaultConfigView,
    ) -> Result<(), Self::Error> {
        let mut views = self.vault_views.write().map_err(|_| {
            ConfigError::ValidationFailed {
                field: "storage".into(),
                message: "Lock poisoned".into(),
            }
        })?;
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
