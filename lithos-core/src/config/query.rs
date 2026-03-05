//! Configuration query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type for read-only access to persisted
//! configuration (merged snapshots, global settings, vault overrides),
//! supporting both owned and zero-copy access patterns.

use tracing::instrument;

use super::{
    aggregate::{Config, Timestamp},
    error::ConfigQueryError,
    global::Global,
    ports::{self as config_ports},
    vault::{Vault, VaultId, VaultRoot},
};

/// Query implementation for configuration read operations.
///
/// This struct provides the primary interface for retrieving persisted
/// configuration data (merged snapshots plus global/vault settings) without
/// performing any mutations. It is generic over a [`config_ports::Query`] to
/// support different backends.
///
/// # Examples
///
/// ```rust,no_run
/// # use tempfile::tempdir;
/// # use lithos_core::{
/// #     config::{ConfigQuery, vault::VaultId, adapter::query::QueryAdapter},
/// #     db::Database,
/// # };
/// # let dir = tempfile::tempdir()?;
/// # let db = Database::open(&dir.path().join("config.redb"))?;
/// let query = ConfigQuery::new(QueryAdapter::new(&db));
/// let _config = query.find(VaultId::new())?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Query<Q> {
    /// Port interface for storage operations.
    query_port: Q,
}

impl<Q> Query<Q> {
    /// Creates a new `Query` with the given port.
    #[inline]
    #[must_use]
    pub const fn new(query_port: Q) -> Self {
        Self {
            query_port,
        }
    }
}

impl<Q> Query<Q>
where
    Q: config_ports::Query,
    Q::Error: Into<crate::db::DbError>,
{
    /// Finds the active merged configuration for a vault.
    ///
    /// The returned [`Config`] is guaranteed to be in an "Always Valid" state.
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if storage access fails or the data is
    /// corrupted.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "find_config", vault_id = %vault_id)
    )]
    pub fn find(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Config>, ConfigQueryError> {
        let active = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigQueryError::Storage(error.into()))?;

        let Some(version) = active else {
            return Ok(None);
        };

        self.query_port
            .find_config(vault_id, version)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Returns the persisted global configuration, if present.
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if storage access fails.
    #[inline]
    #[instrument(skip(self), level = "debug", fields(operation = "get_global"))]
    pub fn get_global(&self) -> Result<Option<Global>, ConfigQueryError> {
        self.query_port
            .get_global()
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Returns the persisted vault configuration, if present.
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if storage access fails.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "get_vault", vault_id = %vault_id)
    )]
    pub fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigQueryError> {
        self.query_port
            .get_vault(vault_id)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Finds the vault ID associated with a vault root path.
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if storage access fails.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "find_vault_id_by_path")
    )]
    pub fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigQueryError> {
        self.query_port
            .find_vault_id_by_path(vault_root)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Check if the global config is stale.
    ///
    /// Returns `true` if:
    /// - No stored metadata exists (never ingested)
    /// - Stored `created_at` differs from provided (file replaced)
    /// - Stored `modified_at` is older than provided (file changed)
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if metadata lookup fails.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "is_global_stale")
    )]
    pub fn is_global_stale(
        &self,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, ConfigQueryError> {
        self.query_port
            .is_global_stale(created_at, modified_at)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Check if a vault config is stale.
    ///
    /// Returns `true` if:
    /// - No stored metadata exists for this vault (never ingested)
    /// - Stored `created_at` differs from provided (file replaced)
    /// - Stored `modified_at` is older than provided (file changed)
    ///
    /// # Errors
    /// Returns [`ConfigQueryError`] if metadata lookup fails.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "is_vault_stale", vault_id = %vault_id)
    )]
    pub fn is_vault_stale(
        &self,
        vault_id: VaultId,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, ConfigQueryError> {
        self.query_port
            .is_vault_stale(vault_id, created_at, modified_at)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }

    /// Zero-copy access to archived configuration via closure (HOT PATH).
    ///
    /// This is the recommended method for performance-critical operations
    /// (e.g., LSP queries). The closure receives a reference to the
    /// archived data within the transaction scope.
    ///
    /// # Errors
    /// Returns `ConfigQueryError` if the active version lookup or archived
    /// access fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use lithos_core::config::{query::Query, vault::VaultId};
    /// # fn example<Q>(query: &Query<Q>, vault_id: VaultId) -> Result<(), Box<dyn std::error::Error>>
    /// # where Q: lithos_core::config::ports::Query, Q::Error: Into<lithos_core::db::DbError>
    /// # {
    /// // Access archived config data within closure (zero-copy)
    /// let has_config = query.with_archived(vault_id, |_config| {
    ///     true  // Return value extracted from archived data
    /// })?.is_some();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(
        skip(self, f),
        level = "debug",
        fields(operation = "with_archived_config", vault_id = %vault_id)
    )]
    pub fn with_archived<R, F>(
        &self,
        vault_id: VaultId,
        f: F,
    ) -> Result<Option<R>, ConfigQueryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        let active = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigQueryError::Storage(error.into()))?;
        let Some(version) = active else {
            return Ok(None);
        };

        self.query_port
            .with_archived(vault_id, version, f)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }
}

#[cfg(test)]
mod tests {
    mod fixtures {
        use tempfile::{TempDir, tempdir};

        use crate::{
            config::{
                aggregate::{Config, Version},
                raw::RawConfig,
                vault::{VaultId, VaultRoot},
            },
            db::Database,
        };

        pub fn test_db() -> (TempDir, Database) {
            let dir = tempdir().expect("tempdir must succeed");
            let path = dir.path().join("config.redb");
            let db = Database::open(&path).expect("database must open");
            (dir, db)
        }

        pub fn test_config() -> Config {
            let test_root = VaultRoot::try_new("/test-vault".into())
                .expect("test vault root must be valid");
            let vault_id = VaultId::new();
            Config::build(
                &RawConfig::default(),
                vault_id,
                test_root,
                Version::initial(),
            )
            .expect("test config must be valid")
        }
    }

    mod load {
        use super::{super::Query, *};

        #[test]
        fn find_returns_none_when_active_missing() {
            let (_dir, db) = fixtures::test_db();
            let vault_id = VaultId::new();

            // Put global config in new table format
            let global = Global::default();
            let global_key = format!("{}", global.version().value());
            db.put(
                crate::config::db_table::GLOBAL_CONFIG,
                &global_key,
                &global,
            )
            .expect("must put global config");

            // But don't put any CONFIG_VERSIONS, so there's no active version
            let qry = Query::new(DbPort::new(&db));

            let result = qry.find(vault_id).expect("query must succeed");

            assert!(
                result.is_none(),
                "Expected None when no CONFIG_VERSIONS exists"
            );
        }

        #[test]
        fn get_vault_returns_stored_config() {
            let (_dir, db) = fixtures::test_db();
            let vault_id = VaultId::new();
            let vault = Vault::default();

            // Use new versioned table format
            let vault_key = format!("{}:{}", vault_id, vault.version().value());
            db.put(crate::config::db_table::VAULT_CONFIG, &vault_key, &vault)
                .expect("must put vault config");

            let qry = Query::new(DbPort::new(&db));

            let loaded = qry.get_vault(vault_id).expect("query must succeed");
            assert_eq!(loaded, Some(vault));
        }
    }

    mod borrowing {
        use super::{super::Query, *};

        #[test]
        fn with_archived_returns_data_via_closure() {
            let (_dir, db) = fixtures::test_db();
            let vault_id = VaultId::new();
            let config = fixtures::test_config();

            // Use new versioned table format (no separate ACTIVE table)
            db.put(
                crate::config::db_table::CONFIG_VERSIONS,
                &format!("{}:{}", vault_id, config.version().value()),
                &config,
            )
            .expect("must put config version");

            let qry = Query::new(DbPort::new(&db));

            // Verify closure-based zero-copy access returns data
            // Note: Config fields are private, so we test the pattern works
            // by checking the closure is called and returns a value
            let result: Option<bool> = qry
                .with_archived(vault_id, |_archived| {
                    // Config has private fields, but we can verify
                    // the archived type is accessible within the closure
                    true
                })
                .expect("query must succeed");

            assert!(result.expect("archived config must exist"));
        }
    }

    use crate::{
        config::{
            aggregate::{Config, Timestamp, Version},
            global::Global,
            ports::{self as config_ports},
            vault::{Vault, VaultId, VaultRoot},
        },
        db::{Database, DbError},
    };

    struct DbPort<'db> {
        adapter: crate::config::adapter::query::QueryAdapter<'db>,
    }

    impl<'db> DbPort<'db> {
        fn new(db: &'db Database) -> Self {
            Self {
                adapter: crate::config::adapter::query::QueryAdapter::new(db),
            }
        }
    }

    impl config_ports::Query for DbPort<'_> {
        type Error = DbError;

        fn find_config(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Config>, DbError> {
            self.adapter.find_config(vault_id, version)
        }

        fn find_vault_id_by_path(
            &self,
            vault_root: &VaultRoot,
        ) -> Result<Option<VaultId>, DbError> {
            self.adapter.find_vault_id_by_path(vault_root)
        }

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, DbError> {
            self.adapter.get_active_version(vault_id)
        }

        fn get_global(&self) -> Result<Option<Global>, DbError> {
            self.adapter.get_global()
        }

        fn get_vault(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Vault>, DbError> {
            self.adapter.get_vault(vault_id)
        }

        fn is_global_stale(
            &self,
            created_at: Option<Timestamp>,
            modified_at: Timestamp,
        ) -> Result<bool, DbError> {
            self.adapter.is_global_stale(created_at, modified_at)
        }

        fn is_vault_stale(
            &self,
            vault_id: VaultId,
            created_at: Option<Timestamp>,
            modified_at: Timestamp,
        ) -> Result<bool, DbError> {
            self.adapter.is_vault_stale(vault_id, created_at, modified_at)
        }

        fn with_archived<R, F>(
            &self,
            vault_id: VaultId,
            version: Version,
            f: F,
        ) -> Result<Option<R>, DbError>
        where
            F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
        {
            self.adapter.with_archived(vault_id, version, f)
        }
    }
}
