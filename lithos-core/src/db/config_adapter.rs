//! Config port adapters for the database.

use crate::{
    config::{
        aggregate::{Config, ConfigVersion},
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
    fn load_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error> {
        self.db.get_owned("merged_config_active", &vault_id.to_string())
    }

    #[inline]
    fn load_global(&self) -> Result<Option<Global>, Self::Error> {
        self.db.get_owned("config", "global")
    }

    #[inline]
    fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error> {
        self.db.get_owned("config", &vault_id.to_string())
    }

    #[inline]
    fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
        self.db.put("config", "global", config)
    }

    #[inline]
    fn save_merged(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        config: &Config,
    ) -> Result<(), Self::Error> {
        let key = merged_version_key(vault_id, version);
        self.db.put("merged_config_versions", &key, config)
    }

    #[inline]
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), Self::Error> {
        self.db.put("config", &vault_id.to_string(), config)
    }

    #[inline]
    fn set_active_version(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<(), Self::Error> {
        self.db.put("merged_config_active", &vault_id.to_string(), &version)
    }

    #[inline]
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), Self::Error> {
        let path_key = vault_path_key(vault_root);
        self.db.put("vault_id_by_path", &path_key, &vault_id)?;
        self.db.put("vault_path_by_id", &vault_id.to_string(), vault_root)
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
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<ConfigVersion>, Self::Error> {
        self.db.get_owned("merged_config_active", &vault_id.to_string())
    }

    #[inline]
    fn get_merged_owned(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<Option<Config>, Self::Error> {
        let key = merged_version_key(vault_id, version);
        self.db.get_owned("merged_config_versions", &key)
    }

    #[inline]
    fn with_archived_merged<F, R>(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let key = merged_version_key(vault_id, version);
        self.db.get::<Config, _, _>("merged_config_versions", &key, f)
    }
}

fn merged_version_key(vault_id: VaultId, version: ConfigVersion) -> String {
    format!("{}:{}", vault_id, version.value())
}

fn vault_path_key(root: &VaultRoot) -> String {
    root.as_path().to_string_lossy().into_owned()
}
