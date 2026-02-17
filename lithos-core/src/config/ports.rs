//! Configuration port definitions for the CQRS pattern.
//!
//! This module defines the [`Command`] and [`Query`] trait interfaces,
//! decoupling domain logic from storage implementation details (like Redb).

use super::{
    aggregate::{Config, Version},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
};

/// Command port for configuration write operations.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped by functionality, not strict alphabetization"
)]
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Allocates the next version number for a vault atomically.
    ///
    /// This method reads the current active version (if any), computes the next
    /// version, and returns it without persisting. The caller is responsible
    /// for saving the merged config and setting the active version.
    ///
    /// # Errors
    /// Returns a storage-specific error if the read fails.
    fn get_next_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Version, Self::Error>;

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

    /// Rolls back the active version by the given number of steps atomically.
    ///
    /// This reads the current active version, computes the target version after
    /// rolling back `steps`, and updates the active pointer.
    ///
    /// # Errors
    /// Returns an error if rollback would underflow or storage fails.
    fn activate_previous_version(
        &self,
        vault_id: VaultId,
        steps: u32,
    ) -> Result<Version, Self::Error>;

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

    /// Fetches the active merged configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error>;

    /// Fetches the persisted global configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn get_global(&self) -> Result<Option<Global>, Self::Error>;

    /// Fetches a merged configuration snapshot as owned data (COLD PATH).
    ///
    /// Use this for operations that need to move/store the config, or when
    /// the closure pattern is inconvenient. For hot paths, prefer
    /// [`with_archived`](Self::with_archived).
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error>;

    /// Fetches the persisted vault configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;

    /// Zero-copy access to archived configuration via closure (HOT PATH).
    ///
    /// The closure receives a reference to the archived data within the
    /// transaction scope, ensuring safety without unsafe code. This is the
    /// recommended method for performance-critical reads (e.g., LSP queries).
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup or access fails.
    fn with_archived<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R;
}
