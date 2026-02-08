//! Configuration port definitions for CQRS pattern.
//!
//! This module defines command and query trait interfaces for configuration
//! management. Note: Per proposal, async is removed - sync-first approach.

use super::{
    aggregate::{Config, ConfigVersion},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
};

/// Command port for configuration write operations.
pub trait ConfigCommandPort: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Load active merged config version for a vault.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn load_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error>;
    /// Load persisted global config, if present.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn load_global(&self) -> Result<Option<Global>, Self::Error>;
    /// Load persisted vault config, if present.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;
    /// Persist global config.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn save_global(&self, config: &Global) -> Result<(), Self::Error>;
    /// Persist a merged config snapshot.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn save_merged(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        config: &Config,
    ) -> Result<(), Self::Error>;
    /// Persist vault-specific config.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error>;
    /// Persist vault id/root mapping.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error>;
    /// Set active merged config version for a vault.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn set_active_version(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<(), Self::Error>;
}

/// Query port for configuration read operations.
pub trait ConfigQueryPort: Send + Sync {
    /// Archived merged config type for zero-copy reads.
    type Archived<'archived>;
    /// Storage error type for query operations.
    type Error: std::error::Error;

    /// Fetch active merged config version for a vault.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error>;
    /// Fetch merged config snapshot as owned data.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<Option<Config>, Self::Error>;
    /// Fetch merged config snapshot as archived data.
    ///
    /// # Errors
    /// Returns a storage error on failure.
    fn with_archived_merged<F, R>(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R;
}
