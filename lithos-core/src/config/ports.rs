//! Configuration port definitions for the CQRS pattern.
//!
//! This module defines the [`Command`] and [`Query`] trait interfaces,
//! decoupling domain logic from storage implementation details (like Redb).

use std::ops::Deref;

use super::{
    aggregate::{Config, Version},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
};

/// Command port for configuration write operations.
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Loads the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn load_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error>;

    /// Loads the persisted global configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn load_global(&self) -> Result<Option<Global>, Self::Error>;

    /// Loads the persisted vault configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;

    /// Persists the global configuration.
    ///
    /// # Errors
    /// Returns a storage-specific error if the save operation fails.
    fn save_global(&self, config: &Global) -> Result<(), Self::Error>;

    /// Persists a merged configuration snapshot.
    ///
    /// # Errors
    /// Returns a storage-specific error if the save operation fails.
    fn save_merged(
        &self,
        vault_id: VaultId,
        version: Version,
        config: &Config,
    ) -> Result<(), Self::Error>;

    /// Persists vault-specific configuration.
    ///
    /// # Errors
    /// Returns a storage-specific error if the save operation fails.
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error>;

    /// Persists the vault ID to root path mapping.
    ///
    /// # Errors
    /// Returns a storage-specific error if the save operation fails.
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error>;

    /// Sets the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error if the update fails.
    fn set_active_version(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<(), Self::Error>;
}

/// Query port for configuration read operations.
pub trait Query: Send + Sync {
    /// Storage error type for query operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// RAII Guard that holds the archived data and dereferences to it.
    type Guard<'archived>: Deref<Target = rkyv::Archived<Config>> + 'archived
    where
        Self: 'archived;

    /// Fetches the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error>;

    /// Fetches a merged configuration snapshot for zero-copy access via a GAT
    /// Guard.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup or access fails.
    fn get_archived(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Self::Guard<'_>>, Self::Error>;

    /// Fetches a merged configuration snapshot as owned data.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error>;
}
