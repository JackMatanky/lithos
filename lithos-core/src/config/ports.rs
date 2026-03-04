//! Configuration port definitions for the CQRS pattern.
//!
//! This module defines the [`Command`], [`CommandState`], and [`Query`] trait
//! interfaces, decoupling domain logic from storage implementation details
//! (like Redb).

use super::{
    aggregate::{Config, Timestamp, Version},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
};

/// Selection strategy for activating a config version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActivationTarget {
    /// Activate a specific exact version.
    Exact(Version),
    /// Activate a version by stepping back from the current active version.
    Previous {
        /// Number of steps to go back from the current active version.
        steps: u32,
    },
}

impl ActivationTarget {
    /// Creates an `ActivationTarget` for a specific exact version.
    #[inline]
    #[must_use]
    pub const fn exact(version: Version) -> Self {
        Self::Exact(version)
    }

    /// Creates an `ActivationTarget` for a previous version by steps.
    #[inline]
    #[must_use]
    pub const fn previous(steps: u32) -> Self {
        Self::Previous {
            steps,
        }
    }
}

/// Command port for configuration write operations (task-oriented).
///
/// This trait defines the public write API for configuration commands.
/// Methods use task-oriented verbs (record_*, activate_*) rather than storage
/// verbs.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped by functionality, not strict alphabetization"
)]
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Records the global configuration with metadata.
    ///
    /// The metadata parameters enable staleness detection:
    /// - `created_at`: File birthtime (detects replacement)
    /// - `modified_at`: File mtime (detects edits)
    ///
    /// # Errors
    /// Returns a storage-specific error if the operation fails.
    fn record_global(
        &self,
        config: &Global,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error>;

    /// Records a merged configuration snapshot.
    ///
    /// # Errors
    /// Returns a storage-specific error if the operation fails.
    fn record_merged(
        &self,
        vault_id: VaultId,
        version: Version,
        config: &Config,
    ) -> Result<(), Self::Error>;

    /// Records vault-specific configuration with metadata.
    ///
    /// The metadata parameters enable staleness detection:
    /// - `created_at`: File birthtime (detects replacement)
    /// - `modified_at`: File mtime (detects edits)
    ///
    /// # Errors
    /// Returns a storage-specific error if the operation fails.
    fn record_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error>;

    /// Records the vault ID to root path mapping.
    ///
    /// # Errors
    /// Returns a storage-specific error if the operation fails.
    fn record_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error>;

    /// Activates a configuration version for a vault.
    ///
    /// # Errors
    /// Returns a storage-specific error if the operation fails.
    fn activate_version(
        &self,
        vault_id: VaultId,
        target: ActivationTarget,
    ) -> Result<Version, Self::Error>;
}

/// Internal command-state port for read-for-write operations.
///
/// This port is crate-private and encapsulates atomic read-modify-write
/// operations needed by command handlers. It is not exposed in the public API.
pub(crate) trait CommandState: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Allocates the next version number for a vault atomically.
    ///
    /// This method reads the current active version (if any), computes the next
    /// version, and returns it without persisting. The caller is responsible
    /// for recording the merged config and activating the version.
    ///
    /// # Errors
    /// Returns a storage-specific error if the read fails.
    fn next_version(&self, vault_id: VaultId) -> Result<Version, Self::Error>;
}

/// Query port for configuration read operations.
pub trait Query: Send + Sync {
    /// Storage error type for query operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Find a merged configuration snapshot as owned data (COLD PATH).
    ///
    /// Use this for operations that need to move/store the config, or when
    /// the closure pattern is inconvenient. For hot paths, prefer
    /// [`with_archived`](Self::with_archived).
    ///
    /// Returns `None` if the specific version does not exist.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn find_merged(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error>;

    /// Find a vault ID by its root path.
    ///
    /// This is used during config loading to map a vault path to its
    /// existing ID, enabling proper staleness detection for vault configs.
    ///
    /// Returns `None` if no vault with this path has been recorded.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::{ports::Query, vault::VaultRoot};
    ///
    /// let query: &dyn Query = todo!();
    /// let vault_root = VaultRoot::try_new("/vault".into())?;
    ///
    /// if let Some(vault_id) = query.find_vault_id_by_path(&vault_root)? {
    ///     println!("Found existing vault: {}", vault_id);
    /// }
    /// ```
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, Self::Error>;

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

    /// Fetches the persisted vault configuration, if present.
    ///
    /// # Errors
    /// Returns a storage-specific error if the lookup fails.
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;

    /// Check if the global config is stale.
    ///
    /// Returns `true` if:
    /// - No stored metadata exists (never ingested)
    /// - Stored `created_at` differs from provided (file replaced)
    /// - Stored `modified_at` is older than provided (file changed)
    ///
    /// # Errors
    /// Returns a storage-specific error if the metadata lookup fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::{ports::Query, aggregate::Timestamp};
    ///
    /// let query: &dyn Query = todo!();
    /// let created = Some(Timestamp::from_secs(1000));
    /// let modified = Timestamp::from_secs(2000);
    ///
    /// if query.is_global_stale(created, modified)? {
    ///     println!("Global config needs reloading");
    /// }
    /// ```
    fn is_global_stale(
        &self,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, Self::Error>;

    /// Check if a vault config is stale.
    ///
    /// Returns `true` if:
    /// - No stored metadata exists for this vault (never ingested)
    /// - Stored `created_at` differs from provided (file replaced)
    /// - Stored `modified_at` is older than provided (file changed)
    ///
    /// # Errors
    /// Returns a storage-specific error if the metadata lookup fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::{
    ///     ports::Query,
    ///     aggregate::Timestamp,
    ///     vault::VaultId,
    /// };
    ///
    /// let query: &dyn Query = todo!();
    /// let vault_id = VaultId::new();
    /// let created = Some(Timestamp::from_secs(1000));
    /// let modified = Timestamp::from_secs(2000);
    ///
    /// if query.is_vault_stale(vault_id, created, modified)? {
    ///     println!("Vault config needs reloading");
    /// }
    /// ```
    fn is_vault_stale(
        &self,
        vault_id: VaultId,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, Self::Error>;

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
