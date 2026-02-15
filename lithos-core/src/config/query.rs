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

    /// Returns a zero-copy RAII Guard for the merged config.
    ///
    /// # Errors
    /// Returns `ConfigQueryError` if the active version lookup or archived
    /// access fails.
    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "get_archived_config", vault_id = %vault_id)
    )]
    pub fn get_archived(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Q::Guard<'_>>, ConfigQueryError> {
        let active = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigQueryError::Storage(error.into()))?;
        let Some(version) = active else {
            return Ok(None);
        };

        self.query_port
            .get_archived(vault_id, version)
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

    use std::ops::Deref;

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

    struct DbPort<'db> {
        db: &'db Database,
    }

    impl<'db> DbPort<'db> {
        fn new(db: &'db Database) -> Self {
            Self {
                db,
            }
        }
    }

    struct TestGuard(rkyv::util::AlignedVec<16>);
    impl Deref for TestGuard {
        type Target = rkyv::Archived<Config>;

        #[inline]
        #[expect(clippy::disallowed_methods, reason = "Validated at creation")]
        fn deref(&self) -> &Self::Target {
            // Safe access using rkyv::access.
            // Since we validated the data during creation, this expect is
            // logically safe.
            rkyv::access::<Self::Target, rkyv::rancor::Error>(self.0.as_slice())
                .expect("valid")
        }
    }

    impl config_ports::Query for DbPort<'_> {
        type Error = DbError;
        type Guard<'archived>
            = TestGuard
        where
            Self: 'archived;

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, DbError> {
            self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
        }

        fn get_merged_owned(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Config>, DbError> {
            let key = format!("{vault_id}:{}", version.value());
            self.db.get_owned(MERGED_CONFIG_VERSIONS, &key)
        }

        fn get_archived(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Self::Guard<'_>>, DbError> {
            let result = self.get_merged_owned(vault_id, version)?;
            match result {
                Some(config) => {
                    let mut vec = rkyv::util::AlignedVec::<16>::new();
                    rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(
                        &config, &mut vec,
                    )
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    Ok(Some(TestGuard(vec)))
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
        fn get_archived_returns_guard_to_data() {
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

            let result =
                qry.get_archived(vault_id).expect("query must succeed");
            assert!(result.is_some());
            let guard = result.unwrap();
            // Verify we can access data via the guard
            assert_eq!(
                guard.paths().cache().cache_dir().as_path(),
                config.paths().cache.cache_dir().as_path()
            );
        }
    }
}
