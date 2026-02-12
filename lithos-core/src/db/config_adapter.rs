//! Config port adapters for the database.

use tracing::instrument;

use crate::{
    config::{
        aggregate::{Config, Version},
        db_table::{
            CONFIG, MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS,
            VAULT_ID_BY_PATH, VAULT_PATH_BY_ID,
        },
        global::Global,
        ports::{Command, Query},
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
        self.db.get::<Config, _, _>(MERGED_CONFIG_VERSIONS, &key, f)
    }
}

#[inline]
fn merged_version_key(vault_id: VaultId, version: Version) -> String {
    format!("{}:{}", vault_id, version.value())
}
