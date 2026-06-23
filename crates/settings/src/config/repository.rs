//! Configuration storage traits.

use crate::config::{
    aggregate::{Config, Version},
    error::ConfigRepositoryError,
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
    views::{RawGlobalConfigView, RawVaultConfigView},
};

/// Read interface for configuration persistence.
pub trait ReadRepository: Send + Sync {
    /// Fetches the persisted global configuration.
    ///
    /// Returns `None` if no global config has been saved.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup or deserialization
    /// fails.
    fn get_global(&self) -> Result<Option<Global>, ConfigRepositoryError>;

    /// Fetches the persisted vault configuration for a specific vault.
    ///
    /// Returns `None` if no vault config exists for this vault ID.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup or deserialization
    /// fails.
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigRepositoryError>;

    /// Fetches a specific configuration snapshot (merged global + vault).
    ///
    /// Returns `None` if the specified version does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup or deserialization
    /// fails.
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, ConfigRepositoryError>;

    /// Fetches the active (latest) configuration version for a vault.
    ///
    /// Returns `None` if no versions exist for this vault.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the scan fails.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, ConfigRepositoryError>;

    /// Zero-copy access to archived configuration via closure.
    ///
    /// Returns `None` if the configuration does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup or access fails.
    fn with_archived_config<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, ConfigRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R;

    /// Finds a vault ID by its root path.
    ///
    /// Returns `None` if no vault with this path has been recorded.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup fails.
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigRepositoryError>;

    /// Fetches the raw global config view with version history.
    ///
    /// Returns `None` if no global config has been ingested yet.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup fails.
    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, ConfigRepositoryError>;

    /// Fetches the raw vault config view with version history.
    ///
    /// Returns `None` if no vault config has been ingested yet for this vault.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the lookup fails.
    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, ConfigRepositoryError>;
}

/// Write interface for configuration persistence.
pub trait WriteRepository: Send + Sync {
    /// Saves the global configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if serialization or storage fails.
    fn save_global(&self, config: &Global)
    -> Result<(), ConfigRepositoryError>;

    /// Saves a vault-specific configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if serialization or storage fails.
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigRepositoryError>;

    /// Saves a final merged configuration snapshot.
    ///
    /// Returns the allocated version number.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if version overflow or storage fails.
    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, ConfigRepositoryError>;

    /// Saves the bidirectional vault ID ↔ path mapping.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if the operation fails.
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), ConfigRepositoryError>;

    /// Saves the raw global config view.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if serialization or storage fails.
    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), ConfigRepositoryError>;

    /// Saves the raw vault config view.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigRepositoryError`] if serialization or storage fails.
    fn save_raw_vault_view(
        &self,
        vault_id: VaultId,
        view: &RawVaultConfigView,
    ) -> Result<(), ConfigRepositoryError>;
}

/// Unified repository for configuration persistence.
pub trait Repository: ReadRepository + WriteRepository {
    /// Storage error type.
    type Error: std::error::Error + Send + Sync + 'static;
}

// Blanket implementation for Repository.
impl<T> Repository for T
where
    T: ReadRepository + WriteRepository,
{
    type Error = ConfigRepositoryError;
}
