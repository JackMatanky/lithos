//! Concrete implementation of the [`crate::config::ports::Command`] trait.

use tracing::instrument;

use super::merged_version_key;
use crate::{
    config::{
        aggregate::{Config, Version},
        db_table::{
            CONFIG, MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS,
            VAULT_ID_BY_PATH, VAULT_PATH_BY_ID,
        },
        global::Global,
        ports::Command,
        vault::{Vault, VaultId, VaultRoot},
    },
    db::{Database, DbError},
};

/// Redb-backed config command adapter.
pub struct CommandAdapter<'db> {
    db: &'db Database,
}

impl<'db> CommandAdapter<'db> {
    #[inline]
    #[must_use]
    /// Create a command adapter for a database.
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Command for CommandAdapter<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(skip(self, config), fields(operation = "save_global"))]
    fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
        self.db.put(CONFIG, "global", config)
    }

    #[inline]
    #[instrument(
        skip(self, config),
        fields(
            operation = "save_merged",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn save_merged(
        &self,
        vault_id: VaultId,
        version: Version,
        config: &Config,
    ) -> Result<(), Self::Error> {
        let key = merged_version_key(vault_id, version);
        self.db.put(MERGED_CONFIG_VERSIONS, &key, config)
    }

    #[inline]
    #[instrument(
        skip(self, config),
        fields(operation = "save_vault", vault_id = %vault_id)
    )]
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error> {
        self.db.put(CONFIG, &vault_id.to_string(), config)
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(
            operation = "set_active_version",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn set_active_version(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<(), Self::Error> {
        self.db.put(MERGED_CONFIG_ACTIVE, &vault_id.to_string(), &version)
    }

    #[inline]
    #[instrument(
        skip(self, vault_root),
        fields(operation = "save_vault_path_mapping", vault_id = %vault_id)
    )]
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error> {
        let path_key = vault_root.as_key();
        self.db.put(VAULT_ID_BY_PATH, &path_key, &vault_id)?;
        self.db.put(VAULT_PATH_BY_ID, &vault_id.to_string(), vault_root)
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "get_next_version", vault_id = %vault_id)
    )]
    fn get_next_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Version, Self::Error> {
        let current: Option<Version> =
            self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())?;

        let candidate = match current {
            Some(v) => v.next().unwrap_or_else(|_| Version::initial()),
            None => Version::initial(),
        };

        Ok(candidate)
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "rollback_active_version", vault_id = %vault_id, steps = %steps)
    )]
    fn rollback_active_version(
        &self,
        vault_id: VaultId,
        steps: u32,
    ) -> Result<Version, Self::Error> {
        self.db.read_write_transaction(|tx| {
            let current: Option<Version> =
                tx.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())?;

            let current = current.ok_or_else(|| {
                DbError::Serialization("no active version".into())
            })?;

            let steps = u64::from(steps);
            let current_val = current.value();
            let target = current_val.saturating_sub(steps);

            if target == 0 {
                return Err(DbError::Serialization(
                    "rollback underflow".into(),
                ));
            }

            let target_version = Version::try_from(target).map_err(|_e| {
                DbError::Serialization("invalid version".into())
            })?;

            tx.put(
                MERGED_CONFIG_ACTIVE,
                &vault_id.to_string(),
                &target_version,
            )?;

            Ok(target_version)
        })
    }
}
