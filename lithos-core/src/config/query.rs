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
///
/// # Examples
///
/// ```rust
/// // Note: Query is constructed with a storage port implementation
/// // let qry = Query::new(storage_port);
/// // let result = qry.get(vault_id);
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

    /// Execute a zero-copy read against the merged config.
    ///
    /// # Errors
    /// Returns `ConfigQueryError` if the active version lookup or archived
    /// access fails.
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
        F: for<'archived> FnOnce(Q::Archived<'archived>) -> R,
    {
        let active = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigQueryError::Storage(error.into()))?;
        let Some(version) = active else {
            return Ok(None);
        };

        self.query_port
            .with_archived_merged(vault_id, version, f)
            .map_err(|error| ConfigQueryError::Storage(error.into()))
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    clippy::arbitrary_source_item_ordering,
    reason = "Test fixtures use expect for setup; test modules organized for \
              readability"
)]
mod tests {
    use super::*;
    use crate::{
        config::{
            aggregate::Version,
            db_table::{CONFIG, MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS},
            global::Global,
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

        /// Create a Config with test values. Only available in tests.
        pub fn test_config() -> Config {
            let test_root = VaultRoot::try_new("/test-vault".into())
                .expect("test vault root must be valid");
            let vault_id = VaultId::new();

            // Use Config::build with empty raw config
            Config::build(&RawConfig::default(), vault_id, test_root)
                .expect("test config must be valid")
        }
    }

    struct DbPort<'db> {
        db: &'db Database,
    }

    mod load {
        use super::*;

        #[test]
        fn get_returns_none_when_active_missing() {
            // Arrange - unwrap permitted for test setup
            let (_dir, db) = fixtures::test_db();
            db.put(CONFIG, "global", &Global::default())
                .expect("must put global config");
            let qry = Query::new(DbPort::new(&db));

            // Act
            let result = qry.get(VaultId::new()).expect("query must succeed");

            // Assert - explicit assertion
            assert!(
                result.is_none(),
                "Expected None when active version missing"
            );
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn with_archived_executes_closure_on_archived_data() {
            // Arrange - unwrap permitted for test setup
            let (_dir, db) = fixtures::test_db();
            let vault_id = VaultId::new();
            let config = fixtures::test_config();

            // Setup: version 1 active with default config
            db.put(MERGED_CONFIG_VERSIONS, &format!("{vault_id}:1"), &config)
                .expect("must put config version");
            db.put(
                MERGED_CONFIG_ACTIVE,
                &vault_id.to_string(),
                &Version::initial(),
            )
            .expect("must set active version");

            let qry = Query::new(DbPort::new(&db));

            // Act
            let result = qry
                .with_archived(vault_id, |_archived| true)
                .expect("query must succeed");

            // Assert - explicit assertion
            assert_eq!(result, Some(true));
        }
    }

    impl<'db> DbPort<'db> {
        fn new(db: &'db Database) -> Self {
            Self {
                db,
            }
        }
    }

    impl config_ports::Query for DbPort<'_> {
        type Archived<'archived> = &'archived rkyv::Archived<Config>;
        type Error = DbError;

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

        fn with_archived_merged<F, R>(
            &self,
            vault_id: VaultId,
            version: Version,
            f: F,
        ) -> Result<Option<R>, DbError>
        where
            F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
        {
            let key = format!("{vault_id}:{}", version.value());
            self.db.get::<Config, _, _>(MERGED_CONFIG_VERSIONS, &key, f)
        }
    }
}
