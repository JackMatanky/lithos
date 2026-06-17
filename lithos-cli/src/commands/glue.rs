//! Glue implementations wiring `Bootstrapper<D>` to the CLI port traits.
//!
//! This module bridges the core bootstrap types to the local handler traits
//! (`BootstrapRunner`, `DiscoveryRunner`) defined in the command handler
//! modules, without coupling the core crate to the CLI.
//!
//! ## Repository Strategy
//!
//! The `run_bootstrap` implementation uses [`CliRepository`], a minimal
//! in-memory repository scoped to a single CLI invocation.  The `config`
//! and `doctor` commands only read and resolve configuration from disk; they
//! do not need cross-run persistence.  A persistent `RedbRepository` will be
//! wired in a future slice when vault indexing or caching requires cross-run
//! state.
//!
//! ## Generic Implementation
//!
//! The traits are implemented generically for `Bootstrapper<D>` where
//! `D: DiscoveryPort`.  This avoids naming the private `DiscoveryService`
//! type directly while still satisfying the Rust orphan rules (the local
//! `BootstrapRunner` and `DiscoveryRunner` traits are defined in this crate).

use std::{
    collections::HashMap,
    path::Path,
    sync::{PoisonError, RwLock},
};

use lithos_core::{
    app::bootstrap::{BootstrapError, BootstrapResult, Bootstrapper},
    config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        repository::{ReadRepository, WriteRepository},
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    discovery::{
        DiscoveryFlags, port::DiscoveryPort, report::DiscoveryReport,
        service::DiscoveryResult,
    },
};

use crate::commands::{
    config::{BootstrapOutcome, BootstrapRunner},
    config_files::DiscoveryRunner,
};

// ------------------------------------------------------------------ //
//                      In-Memory CLI Repository                      //
// ------------------------------------------------------------------ //

/// Minimal in-memory config repository for a single CLI invocation.
///
/// This repository satisfies the `Repository` bound required by
/// `Bootstrapper::run()`.  All writes are retained in memory for the
/// duration of the process; reads return whatever was written in the same
/// run.  This is sufficient for `config` and `doctor` commands, which
/// build and display the resolved configuration without needing cross-run
/// persistence.
#[allow(clippy::type_complexity, reason = "Internal state uses nested maps")]
struct CliRepository {
    configs: RwLock<HashMap<(VaultId, Version), Config>>,
    active_versions: RwLock<HashMap<VaultId, Version>>,
    globals: RwLock<Option<Global>>,
    vaults: RwLock<HashMap<VaultId, Vault>>,
    global_views: RwLock<Option<RawGlobalConfigView>>,
    vault_views: RwLock<HashMap<VaultId, RawVaultConfigView>>,
    vault_id_mappings: RwLock<HashMap<VaultRoot, VaultId>>,
}

impl CliRepository {
    /// Creates an empty in-memory CLI repository.
    fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            active_versions: RwLock::new(HashMap::new()),
            globals: RwLock::new(None),
            vaults: RwLock::new(HashMap::new()),
            global_views: RwLock::new(None),
            vault_views: RwLock::new(HashMap::new()),
            vault_id_mappings: RwLock::new(HashMap::new()),
        }
    }
}

impl ReadRepository for CliRepository {
    fn get_global(&self) -> Result<Option<Global>, ConfigRepositoryError> {
        Ok(self.globals.read().unwrap_or_else(PoisonError::into_inner).clone())
    }

    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigRepositoryError> {
        Ok(self
            .vaults
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&vault_id)
            .cloned())
    }

    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, ConfigRepositoryError> {
        Ok(self
            .configs
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(vault_id, version))
            .cloned())
    }

    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, ConfigRepositoryError> {
        Ok(self
            .active_versions
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&vault_id)
            .copied())
    }

    fn with_archived_config<R, F>(
        &self,
        _vault_id: VaultId,
        _version: Version,
        _f: F,
    ) -> Result<Option<R>, ConfigRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        // The CLI does not use zero-copy archived reads; return None to
        // fall through to the rebuild path in Builder.
        Ok(None)
    }

    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigRepositoryError> {
        Ok(self
            .vault_id_mappings
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(vault_root)
            .copied())
    }

    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, ConfigRepositoryError> {
        Ok(self
            .global_views
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone())
    }

    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, ConfigRepositoryError> {
        Ok(self
            .vault_views
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&vault_id)
            .cloned())
    }
}

impl WriteRepository for CliRepository {
    fn save_global(
        &self,
        config: &Global,
    ) -> Result<(), ConfigRepositoryError> {
        *self.globals.write().unwrap_or_else(PoisonError::into_inner) =
            Some(config.clone());
        Ok(())
    }

    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigRepositoryError> {
        self.vaults
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(vault_id, config.clone());
        Ok(())
    }

    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, ConfigRepositoryError> {
        let version = *config.version();
        self.configs
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((vault_id, version), config.clone());
        self.active_versions
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(vault_id, version);
        Ok(version)
    }

    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), ConfigRepositoryError> {
        self.vault_id_mappings
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(vault_root.clone(), vault_id);
        Ok(())
    }

    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        *self.global_views.write().unwrap_or_else(PoisonError::into_inner) =
            Some(view.clone());
        Ok(())
    }

    fn save_raw_vault_view(
        &self,
        vault_id: VaultId,
        view: &RawVaultConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        self.vault_views
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(vault_id, view.clone());
        Ok(())
    }
}

// ------------------------------------------------------------------ //
//      Generic BootstrapRunner impl for Bootstrapper<D>              //
// ------------------------------------------------------------------ //

impl<D: DiscoveryPort> BootstrapRunner for Bootstrapper<D> {
    /// Runs the full bootstrap pipeline (discovery → config build).
    ///
    /// Discovery is performed twice: once to capture the resolved config file
    /// paths (for [`BootstrapOutcome`]), and once inside
    /// [`Bootstrapper::run`] to build the [`Config`] aggregate.  The second
    /// traversal is a fast filesystem walk and is acceptable overhead for a
    /// CLI command.
    ///
    /// A [`CliRepository`] is used for in-memory config persistence within the
    /// process lifetime.  This is correct for read-only CLI commands;
    /// cross-run caching will be wired in a later slice.
    fn run_bootstrap(
        &self,
        flags: Option<DiscoveryFlags>,
        anchor: &Path,
    ) -> Result<BootstrapOutcome, BootstrapError> {
        // First pass: discover only, to capture the resolved candidate paths.
        let (discovery, _first_report) =
            self.run_discovery_only(flags.clone(), None, anchor)?;

        let vault_config_path =
            discovery.vault().first().map(|c| c.path().as_path().to_path_buf());
        let global_config_path = discovery
            .global()
            .first()
            .map(|c| c.path().as_path().to_path_buf());

        // Second pass: full bootstrap (discovery + config build).
        let BootstrapResult {
            config,
            report,
        } = self.run(flags, None, anchor, CliRepository::new())?;

        Ok(BootstrapOutcome {
            config,
            report,
            vault_config_path,
            global_config_path,
        })
    }
}

// ------------------------------------------------------------------ //
//      Generic DiscoveryRunner impl for Bootstrapper<D>              //
// ------------------------------------------------------------------ //

impl<D: DiscoveryPort> DiscoveryRunner for Bootstrapper<D> {
    /// Runs discovery only (without config loading).
    fn run_discovery(
        &self,
        flags: Option<DiscoveryFlags>,
        anchor: &Path,
    ) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError> {
        self.run_discovery_only(flags, None, anchor)
    }
}
