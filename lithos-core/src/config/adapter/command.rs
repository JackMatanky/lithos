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
    #[instrument(
        skip(self),
        fields(operation = "load_active_version", vault_id = %vault_id)
    )]
    fn load_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, Self::Error> {
        self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
    }

    #[inline]
    #[instrument(skip(self), fields(operation = "load_global"))]
    fn load_global(&self) -> Result<Option<Global>, Self::Error> {
        self.db.get_owned(CONFIG, "global")
    }

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "load_vault", vault_id = %vault_id)
    )]
    fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error> {
        self.db.get_owned(CONFIG, &vault_id.to_string())
    }

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
}
