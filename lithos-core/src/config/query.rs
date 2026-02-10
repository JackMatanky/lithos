//! Configuration query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read-only access
//! to the persisted configuration snapshots, supporting both owned and
//! zero-copy access patterns.

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
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules group fixtures and test logic for readability"
)]
mod tests {
    use super::*;
    use crate::{
        config::{
            aggregate::{Config, Version},
            global::Global,
            ports as config_ports,
            vault::VaultId,
        },
        db::{Database, DbError},
    };

    mod fixtures {
        use tempfile::{TempDir, tempdir};

        use crate::db::Database;

        type TestDbResult =
            Result<(TempDir, Database), Box<dyn std::error::Error>>;

        pub fn test_db() -> TestDbResult {
            let dir = tempdir()?;
            let path = dir.path().join("config.redb");
            let db = Database::open(&path)?;
            Ok((dir, db))
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

    impl config_ports::Query for DbPort<'_> {
        type Archived<'archived> = &'archived rkyv::Archived<Config>;
        type Error = DbError;

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, DbError> {
            self.db.get_owned("merged_config_active", &vault_id.to_string())
        }

        fn get_merged_owned(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Config>, DbError> {
            let key = format!("{vault_id}:{}", version.value());
            self.db.get_owned("merged_config_versions", &key)
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
            self.db.get::<Config, _, _>("merged_config_versions", &key, f)
        }
    }

    mod load {
        use super::*;

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert! which can panic."
        )]
        fn get_returns_none_when_active_missing()
        -> Result<(), Box<dyn std::error::Error>> {
            let (_dir, db) = fixtures::test_db()?;
            db.put("config", "global", &Global::default())?;
            let qry = Query::new(DbPort::new(&db));

            let result = qry.get(VaultId::new())?;
            assert!(
                result.is_none(),
                "Expected None when active version missing"
            );
            Ok(())
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn with_archived_executes_closure_on_archived_data()
        -> Result<(), Box<dyn std::error::Error>> {
            let (_dir, db) = fixtures::test_db()?;
            let vault_id = VaultId::new();
            let version = Version::try_from(1)?;
            let config = Config::default();

            // Setup: version 1 active with default config
            db.put(
                "merged_config_versions",
                &format!("{vault_id}:1"),
                &config,
            )?;
            db.put("merged_config_active", &vault_id.to_string(), &version)?;

            let qry = Query::new(DbPort::new(&db));

            let result = qry.with_archived(vault_id, |_archived| true)?;

            assert_eq!(result, Some(true));
            Ok(())
        }
    }
}
