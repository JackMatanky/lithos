//! Redb-backed implementation of the [`crate::config::ports::Query`] trait.

use tracing::instrument;

use super::merged_version_key;
use crate::{
    config::{
        aggregate::{Config, Version},
        db_table::{MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS},
        ports::Query,
        vault::VaultId,
    },
    db::{Database, DbError},
};

/// Redb-backed config query adapter.
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a query adapter for a database.
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Query for QueryAdapter<'_> {
    type Archived<'archived> = &'archived rkyv::Archived<Config>;
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "get_active_version", vault_id = %vault_id)
    )]
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error> {
        self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(
            operation = "get_merged_owned",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error> {
        let key = merged_version_key(vault_id, version);
        self.db.get_owned(MERGED_CONFIG_VERSIONS, &key)
    }

    #[inline]
    #[instrument(
        skip(self, f),
        level = "debug",
        fields(
            operation = "with_archived_merged",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn with_archived_merged<F, R>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let key = merged_version_key(vault_id, version);
        self.db.get::<Config, _, R>(MERGED_CONFIG_VERSIONS, &key, f)
    }
}
