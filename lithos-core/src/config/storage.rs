//! Configuration storage with unified Repository pattern.
//!
//! This module provides the [`Repository`] trait for configuration persistence,
//! following the unified repository pattern (no CQRS split).
//!
//! # Architecture
//!
//! - **Repository trait**: Single trait combining reads and writes
//! - **RedbStorage**: Production implementation using redb
//! - **InMemoryStorage**: Test implementation using HashMap
//! - **FakeStorage**: Test double for controlled test scenarios
//!
//! # Storage Layout
//!
//! Config data is stored across multiple tables:
//! - `GLOBAL_CONFIG` - Versioned global configuration
//! - `VAULT_CONFIG` - Versioned vault-specific configuration
//! - `CONFIG_VERSIONS` - Final merged configuration snapshots
//! - `VAULT_ID_BY_PATH` / `VAULT_PATH_BY_ID` - Bidirectional vault mapping
//! - `RAW_GLOBAL_CONFIG_VIEW` - Raw global config with version history
//! - `RAW_VAULT_CONFIG_VIEW` - Raw vault config with version history

use super::{
    aggregate::{Config, Version},
    global::Global,
    vault::{Vault, VaultId, VaultRoot},
    views::raw::{RawGlobalConfigView, RawVaultConfigView},
};

// ----------------------------------------------------------- //
//                     Repository Trait                        //
// ----------------------------------------------------------- //

/// Unified repository for configuration persistence.
///
/// This trait combines read and write operations in a single interface,
/// following the unified repository pattern (not CQRS split).
///
/// # Implementations
///
/// - [`RedbStorage`] - Production backend using redb
/// - [`InMemoryStorage`] - In-memory backend for tests
/// - [`FakeStorage`] - Test double with controlled behavior
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::{Repository, vault::VaultId};
///
/// fn load_config(repo: &impl Repository, vault_id: VaultId) -> Result<(), Box<dyn std::error::Error>> {
///     if let Some(global) = repo.get_global()? {
///         println!("Global config found");
///     }
///
///     if let Some(vault) = repo.get_vault(vault_id)? {
///         println!("Vault config found");
///     }
///
///     Ok(())
/// }
/// ```
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped by functionality (global, vault, config, \
              mapping, raw views)"
)]
pub trait Repository: Send + Sync {
    /// Storage error type.
    type Error: std::error::Error + Send + Sync + 'static;

    // ----------------------------------------------------------- //
    //                   Global Config Operations                  //
    // ----------------------------------------------------------- //

    /// Fetches the persisted global configuration.
    ///
    /// Returns `None` if no global config has been saved.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn get_global(&self) -> Result<Option<Global>, Self::Error>;

    /// Saves the global configuration.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if serialization or storage fails.
    fn save_global(&self, config: &Global) -> Result<(), Self::Error>;

    // ----------------------------------------------------------- //
    //                   Vault Config Operations                   //
    // ----------------------------------------------------------- //

    /// Fetches the persisted vault configuration for a specific vault.
    ///
    /// Returns `None` if no vault config exists for this vault ID.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error>;

    /// Saves a vault-specific configuration.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if serialization or storage fails.
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error>;

    // ----------------------------------------------------------- //
    //                  Merged Config Operations                   //
    // ----------------------------------------------------------- //

    /// Fetches a specific configuration snapshot (merged global + vault).
    ///
    /// Returns `None` if the specified version does not exist.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup or deserialization fails.
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error>;

    /// Saves a final merged configuration snapshot with atomic version
    /// allocation.
    ///
    /// The version is computed atomically by scanning existing versions and
    /// incrementing. This prevents race conditions when multiple concurrent
    /// rebuilds occur on the same vault.
    ///
    /// Returns the allocated version number.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if version overflow or storage fails.
    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error>;

    /// Fetches the active (latest) configuration version for a vault.
    ///
    /// Scans the `CONFIG_VERSIONS` table for the maximum version number
    /// with the given `vault_id` prefix.
    ///
    /// Returns `None` if no versions exist for this vault.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the scan fails.
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error>;

    /// Zero-copy access to archived configuration via closure (HOT PATH).
    ///
    /// The closure receives a reference to the archived data within the
    /// transaction scope, ensuring safety without unsafe code. This is the
    /// recommended method for performance-critical reads (e.g., LSP queries).
    ///
    /// Returns `None` if the configuration does not exist.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup or access fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::config::{Repository, vault::VaultId, aggregate::Version};
    ///
    /// fn query_config_zero_copy(
    ///     repo: &impl Repository,
    ///     vault_id: VaultId,
    ///     version: Version,
    /// ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    ///     repo.with_archived_config(vault_id, version, |archived_config| {
    ///         // Work with archived data without deserialization
    ///         archived_config.task().enabled
    ///     }).map_err(Into::into)
    /// }
    /// ```
    fn with_archived_config<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R;

    // ----------------------------------------------------------- //
    //                  Vault Path Mapping                         //
    // ----------------------------------------------------------- //

    /// Finds a vault ID by its root path.
    ///
    /// This is used during config loading to map a vault path to its
    /// existing ID, enabling proper staleness detection for vault configs.
    ///
    /// Returns `None` if no vault with this path has been recorded.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup fails.
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, Self::Error>;

    /// Saves the bidirectional vault ID ↔ path mapping.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the operation fails.
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error>;

    // ----------------------------------------------------------- //
    //                   Raw Config Views                          //
    // ----------------------------------------------------------- //

    /// Fetches the raw global config view with version history.
    ///
    /// Returns `None` if no global config has been ingested yet.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup fails.
    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, Self::Error>;

    /// Saves the raw global config view.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if serialization or storage fails.
    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), Self::Error>;

    /// Fetches the raw vault config view with version history.
    ///
    /// Returns `None` if no vault config has been ingested yet for this vault.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if the lookup fails.
    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, Self::Error>;

    /// Saves the raw vault config view.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if serialization or storage fails.
    fn save_raw_vault_view(
        &self,
        view: &RawVaultConfigView,
    ) -> Result<(), Self::Error>;
}

// ----------------------------------------------------------- //
//                  Repository Implementations                 //
// ----------------------------------------------------------- //

/// Redb-backed repository implementation.
///
/// This is the production storage backend using the `redb` embedded database.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::{db::Database, config::storage::RedbStorage};
///
/// let db = Database::open("lithos.db")?;
/// let repo = RedbStorage::new(&db);
/// ```
pub struct RedbStorage<'db> {
    db: &'db crate::db::Database,
}

impl<'db> RedbStorage<'db> {
    /// Creates a new redb storage backend.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db crate::db::Database) -> Self {
        Self {
            db,
        }
    }
}

impl Repository for RedbStorage<'_> {
    type Error = crate::db::DbError;

    #[inline]
    fn get_global(&self) -> Result<Option<Global>, Self::Error> {
        // Global config is stored with key = version number
        // We need to find the latest version by scanning all keys
        let prefix = "";

        let max_version_key = self
            .db
            .scan_range::<Global>(super::db_table::GLOBAL_CONFIG, prefix)?
            .into_iter()
            .filter_map(|(k, _)| k.parse::<u64>().ok())
            .max()
            .map(|v| v.to_string());

        match max_version_key {
            Some(key) => {
                self.db.get_owned(super::db_table::GLOBAL_CONFIG, &key)
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
        let version_key = config.version().value().to_string();
        self.db.batch_write(|tx| {
            tx.put(super::db_table::GLOBAL_CONFIG, &version_key, config)
        })
    }

    #[inline]
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error> {
        // Vault config is stored with key = "{vault_id}:{version}"
        // Find the latest version for this vault
        let prefix = format!("{vault_id}:");

        let max_version_key = self
            .db
            .scan_range::<Vault>(super::db_table::VAULT_CONFIG, &prefix)?
            .into_iter()
            .filter_map(|(k, _)| {
                k.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|v| format!("{vault_id}:{v}"))
            })
            .max();

        match max_version_key {
            Some(key) => self.db.get_owned(super::db_table::VAULT_CONFIG, &key),
            None => Ok(None),
        }
    }

    #[inline]
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error> {
        let version_key = format!("{}:{}", vault_id, config.version().value());
        self.db.batch_write(|tx| {
            tx.put(super::db_table::VAULT_CONFIG, &version_key, config)
        })
    }

    #[inline]
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error> {
        let key = format!("{}:{}", vault_id, version.value());
        self.db.get_owned(super::db_table::CONFIG_VERSIONS, &key)
    }

    #[inline]
    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error> {
        // Atomically allocate version and save config
        self.db.read_write_unit_of_work(|tx| {
            let prefix = format!("{vault_id}:");

            #[expect(
                clippy::return_and_then,
                reason = "filter_map chain is more readable than nested match"
            )]
            let max_version = tx
                .scan_range::<Config>(
                    super::db_table::CONFIG_VERSIONS,
                    &prefix,
                )?
                .into_iter()
                .filter_map(|(key, _)| {
                    key.strip_prefix(&prefix)
                        .and_then(|v| v.parse::<u64>().ok())
                        .and_then(|v| Version::try_from(v).ok())
                })
                .max();

            // Compute next version
            let next = match max_version {
                Some(v) => v.next().map_err(|_err| {
                    crate::db::DbError::Serialization(
                        "config version overflow - vault has exceeded maximum \
                         rebuilds"
                            .into(),
                    )
                })?,
                None => Version::initial(),
            };

            // Update config with allocated version and write
            let versioned_config = config.clone().with_version(next);
            let key = format!("{}:{}", vault_id, next.value());
            tx.put(super::db_table::CONFIG_VERSIONS, &key, &versioned_config)?;

            Ok(next)
        })
    }

    #[inline]
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error> {
        let prefix = format!("{vault_id}:");

        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Config>(super::db_table::CONFIG_VERSIONS, &prefix)?
            .into_iter()
            .filter_map(|(key, _)| {
                key.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .and_then(|v| Version::try_from(v).ok())
            })
            .max();

        Ok(max_version)
    }

    #[inline]
    fn with_archived_config<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        let key = format!("{}:{}", vault_id, version.value());
        self.db.get::<Config, _, _>(super::db_table::CONFIG_VERSIONS, &key, f)
    }

    #[inline]
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, Self::Error> {
        let path_key = vault_root.as_key();
        self.db.get_owned(super::db_table::VAULT_ID_BY_PATH, &path_key)
    }

    #[inline]
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error> {
        let path_key = vault_root.as_key();
        self.db.batch_write(|tx| {
            tx.put(super::db_table::VAULT_ID_BY_PATH, &path_key, &vault_id)?;
            tx.put(
                super::db_table::VAULT_PATH_BY_ID,
                &vault_id.to_string(),
                vault_root,
            )
        })
    }

    #[inline]
    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, Self::Error> {
        self.db.get_owned(super::db_table::RAW_GLOBAL_CONFIG_VIEW, "global")
    }

    #[inline]
    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), Self::Error> {
        self.db.batch_write(|tx| {
            tx.put(super::db_table::RAW_GLOBAL_CONFIG_VIEW, "global", view)
        })
    }

    #[inline]
    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, Self::Error> {
        let key = vault_id.to_string();
        self.db.get_owned(super::db_table::RAW_VAULT_CONFIG_VIEW, &key)
    }

    #[inline]
    fn save_raw_vault_view(
        &self,
        view: &RawVaultConfigView,
    ) -> Result<(), Self::Error> {
        let key = view.vault_id().to_string();
        self.db.batch_write(|tx| {
            tx.put(super::db_table::RAW_VAULT_CONFIG_VIEW, &key, view)
        })
    }
}
