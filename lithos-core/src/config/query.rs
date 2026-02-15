//! Configuration query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read-only access
//! to the persisted configuration snapshots, supporting both owned and
//! zero-copy access patterns.

use tracing::instrument;

use super::{
    aggregate::Config,
    error::ConfigQueryError,
    ports::{self as config_ports},
    vault::VaultId,
};

/// Query implementation for configuration read operations.
///
/// This struct provides the primary interface for retrieving persisted
/// configuration snapshots. It is generic over a [`config_ports::Query`]
/// to support different storage backends.
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
    /// Returns the active merged configuration for a vault.
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
        fields(operation = "get_config", vault_id = %vault_id)
    )]
    pub fn get(
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
            .get_merged_owned(vault_id, version)
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
#[expect(
    clippy::disallowed_methods,
    clippy::arbitrary_source_item_ordering,
    reason = "Test module requirements"
)]
mod tests {
    use super::Query;
    use crate::{
        config::{
            aggregate::{Config, Version},
            db_table::{CONFIG, MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS},
            global::Global,
            ports::{self as config_ports},
            vault::VaultId,
        },
        db::{Database, DbError},
    };

    mod fixtures {
        use tempfile::{TempDir, tempdir};

        use crate::{
            config::{
                aggregate::Config,
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
            Config::build(&RawConfig::default(), vault_id, test_root)
                .expect("test config must be valid")
        }
    }

    struct DbPort {
        txn: redb::ReadTransaction,
    }

    impl DbPort {
        fn new(db: &Database) -> Self {
            Self {
                txn: db.begin_read().expect("tx"),
            }
        }
    }

    impl config_ports::Query for DbPort {
        type Error = DbError;

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, DbError> {
            let table = match self.txn.open_table(MERGED_CONFIG_ACTIVE) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                Err(e) => return Err(DbError::Transaction(e.to_string())),
            };
            match table.get(vault_id.to_string().as_str())? {
                Some(guard) => {
                    let bytes: &[u8] = guard.value();
                    let archived = rkyv::access::<
                        rkyv::Archived<Version>,
                        rkyv::rancor::Error,
                    >(bytes)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    Ok(Some(
                        rkyv::deserialize::<Version, rkyv::rancor::Error>(
                            archived,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?,
                    ))
                }
                None => Ok(None),
            }
        }

        fn get_merged_owned(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Config>, DbError> {
            let key = format!("{vault_id}:{}", version.value());
            let table = match self.txn.open_table(MERGED_CONFIG_VERSIONS) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                Err(e) => return Err(DbError::Transaction(e.to_string())),
            };
            match table.get(key.as_str())? {
                Some(guard) => {
                    let bytes: &[u8] = guard.value();
                    let archived = rkyv::access::<
                        rkyv::Archived<Config>,
                        rkyv::rancor::Error,
                    >(bytes)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    Ok(Some(
                        rkyv::deserialize::<Config, rkyv::rancor::Error>(
                            archived,
                        )
                        .map_err(|e| DbError::Deserialization(e.to_string()))?,
                    ))
                }
                None => Ok(None),
            }
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
            let key = format!("{vault_id}:{}", version.value());
            let table = match self.txn.open_table(MERGED_CONFIG_VERSIONS) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                Err(e) => return Err(DbError::Transaction(e.to_string())),
            };
            match table.get(key.as_str())? {
                Some(guard) => {
                    let bytes: &[u8] = guard.value();
                    let archived = rkyv::access::<
                        rkyv::Archived<Config>,
                        rkyv::rancor::Error,
                    >(bytes)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    Ok(Some(f(archived)))
                }
                None => Ok(None),
            }
        }
    }

    mod load {
        use super::*;

        #[test]
        fn get_returns_none_when_active_missing() {
            let (_dir, db) = fixtures::test_db();
            db.put(CONFIG, "global", &Global::default())
                .expect("must put global config");

            let qry = Query::new(DbPort::new(&db));

            let result = qry.get(VaultId::new()).expect("query must succeed");

            assert!(
                result.is_none(),
                "Expected None when active version missing"
            );
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn with_archived_returns_data_via_closure() {
            let (_dir, db) = fixtures::test_db();
            let vault_id = VaultId::new();
            let config = fixtures::test_config();

            db.put(MERGED_CONFIG_VERSIONS, &format!("{vault_id}:1"), &config)
                .expect("must put config version");
            db.put(
                MERGED_CONFIG_ACTIVE,
                &vault_id.to_string(),
                &Version::initial(),
            )
            .expect("must set active version");

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
}
