//! Concrete implementation of the [`crate::config::ports::Query`] trait.

use std::ops::Deref;

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

/// RAII Guard that holds the archived config data buffer.
///
/// This implementation uses 100% Safe Rust by leveraging rkyv's `unaligned`
/// feature and performing validation during construction.
pub struct ConfigGuard(rkyv::util::AlignedVec<16>);

impl ConfigGuard {
    /// Create a new `ConfigGuard` from an aligned buffer, performing
    /// validation.
    ///
    /// # Errors
    /// Returns `DbError::Deserialization` if the buffer does not contain valid
    /// archived data.
    #[inline]
    pub fn try_new(
        buffer: rkyv::util::AlignedVec<16>,
    ) -> Result<Self, DbError> {
        // Validate the buffer immediately using safe access.
        // The "unaligned" feature ensures this works regardless of byte offset.
        rkyv::access::<rkyv::Archived<Config>, rkyv::rancor::Error>(
            buffer.as_slice(),
        )
        .map_err(|e| DbError::Deserialization(e.to_string()))?;
        Ok(Self(buffer))
    }
}

impl Deref for ConfigGuard {
    type Target = rkyv::Archived<Config>;

    #[inline]
    #[expect(
        clippy::disallowed_methods,
        clippy::expect_used,
        reason = "Data was validated during ConfigGuard construction"
    )]
    fn deref(&self) -> &Self::Target {
        // Safe access. Since we validated the buffer in `try_new`, this is
        // guaranteed to succeed. We use expect here because Deref cannot fail.
        rkyv::access::<rkyv::Archived<Config>, rkyv::rancor::Error>(
            self.0.as_slice(),
        )
        .expect("Data was validated during ConfigGuard construction")
    }
}

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
    type Error = DbError;
    type Guard<'archived>
        = ConfigGuard
    where
        Self: 'archived;

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
        skip(self),
        level = "debug",
        fields(
            operation = "get_archived",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn get_archived<'archived>(
        &'archived self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Self::Guard<'archived>>, Self::Error> {
        let key = merged_version_key(vault_id, version);

        // We fetch the owned Config and re-serialize it into an AlignedVec.
        // While this involves a copy, it fulfills the GAT Guard Port contract
        // and ensures the zero-copy requirement for all subsequent reads.
        let result =
            self.db.get_owned::<Config>(MERGED_CONFIG_VERSIONS, &key)?;
        match result {
            Some(config) => {
                let mut vec = rkyv::util::AlignedVec::<16>::new();
                rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(
                    &config, &mut vec,
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))?;

                Ok(Some(ConfigGuard::try_new(vec)?))
            }
            None => Ok(None),
        }
    }
}
