//! Configuration port definitions for the CQRS pattern.
//!
//! This module defines the [`Command`] and [`Query`] trait interfaces,
//! decoupling domain logic from storage implementation details (like Redb).

use super::{
    aggregate::{Config, ConfigVersion},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
};

/// Command port for configuration write operations.
///
/// This trait defines the interface for persisting configuration state.
/// Implementations are responsible for mapping domain types to the
/// physical storage layer.
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Loads the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn load_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error>;
    /// Loads the persisted global configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn load_global(&self) -> Result<Option<Global>, Self::Error>;
    /// Loads the persisted vault configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;
    /// Persists the global configuration.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn save_global(&self, config: &Global) -> Result<(), Self::Error>;
    /// Persists a merged configuration snapshot.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn save_merged(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        config: &Config,
    ) -> Result<(), Self::Error>;
    /// Persists vault-specific configuration.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error>;
    /// Persists the vault ID to root path mapping.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error>;
    /// Sets the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn set_active_version(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<(), Self::Error>;
}

/// Query port for configuration read operations.
///
/// This trait defines the interface for retrieving configuration state.
/// It supports both owned data retrieval and zero-copy access via
/// archived types.
pub trait Query: Send + Sync {
    /// Archived merged config type for zero-copy reads.
    type Archived<'archived>;
    /// Storage error type for query operations.
    type Error: std::error::Error;

    /// Fetches the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error>;
    /// Fetches a merged configuration snapshot as owned data.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<Option<Config>, Self::Error>;
    /// Fetches a merged configuration snapshot for zero-copy access.
    ///
    /// # Errors
    /// Returns a storage-specific error on failure.
    fn with_archived_merged<F, R>(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R;
}
