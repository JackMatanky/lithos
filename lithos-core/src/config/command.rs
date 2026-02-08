//! Config command implementations (CQRS write operations).
//!
//! Generic over a command port for storage access.

use super::{
    aggregate::{Config, ConfigVersion},
    error::{ConfigCommandError, ConfigError},
    global::Global,
    ingest,
    ports::ConfigCommandPort,
    vault::{Vault, VaultId, VaultRoot},
};

/// Command implementation for Config write operations.
pub struct Command<C> {
    command_port: C,
}

impl<C> Command<C> {
    /// Create a new `Command` with the given port.
    #[inline]
    #[must_use]
    pub const fn new(command_port: C) -> Self {
        Self {
            command_port,
        }
    }
}

impl<C> Command<C>
where
    C: ConfigCommandPort,
    C::Error: Into<crate::db::DbError>,
{
    /// Save global configuration.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if persistence fails.
    #[inline]
    pub fn save_global(
        &self,
        config: &Global,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .save_global(config)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if persistence fails.
    #[inline]
    pub fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .save_vault(vault_id, config)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if storage fails.
    #[inline]
    pub fn load_global(&self) -> Result<Option<Global>, ConfigCommandError> {
        self.command_port
            .load_global()
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if storage fails.
    #[inline]
    pub fn load_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigCommandError> {
        self.command_port
            .load_vault(vault_id)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Rebuild the merged config read model for a vault.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if ingestion, validation, or persistence
    /// fails.
    #[inline]
    pub fn rebuild_merged(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<ConfigVersion, ConfigCommandError> {
        let raw_global = ingest::ingest_global()?;
        let raw_vault = ingest::ingest_vault(vault_root.as_path())?;

        let global =
            Global::try_from(raw_global).map_err(ConfigCommandError::Domain)?;
        let vault =
            Vault::try_from(raw_vault).map_err(ConfigCommandError::Domain)?;

        self.save_global(&global)?;
        self.save_vault(vault_id, &vault)?;

        self.command_port
            .save_vault_path_mapping(vault_id, vault_root)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        let merged =
            Config::build(Some(&global), vault_id, vault_root.clone(), &vault)
                .map_err(ConfigCommandError::Domain)?;

        let version = self.next_version(vault_id)?;
        self.command_port
            .save_merged(vault_id, version, &merged)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;
        self.command_port
            .set_active_version(vault_id, version)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        Ok(version)
    }

    /// Activate a specific merged config version for a vault.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if the version does not exist or the
    /// storage operation fails.
    #[inline]
    pub fn activate_version(
        &self,
        vault_id: VaultId,
        version: ConfigVersion,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .set_active_version(vault_id, version)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;
        Ok(())
    }

    /// Roll back the active merged config version by `steps`.
    ///
    /// # Errors
    /// Returns `ConfigCommandError` if rollback would underflow or storage
    /// access fails.
    #[inline]
    pub fn rollback(
        &self,
        vault_id: VaultId,
        steps: u32,
    ) -> Result<ConfigVersion, ConfigCommandError> {
        let active = self
            .command_port
            .load_active_version(vault_id)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?
            .ok_or_else(|| {
                ConfigCommandError::Domain(ConfigError::ValidationFailed {
                    field: "config_version".to_owned().into(),
                    message: "no active version".to_owned().into(),
                })
            })?;

        let steps = u64::from(steps);
        let current = active.value();
        let target = current.saturating_sub(steps);
        if target == 0 {
            return Err(ConfigCommandError::Domain(
                ConfigError::ValidationFailed {
                    field: "config_version".to_owned().into(),
                    message: "rollback underflow".to_owned().into(),
                },
            ));
        }

        let target = ConfigVersion::try_from(target)?;
        self.activate_version(vault_id, target)?;
        Ok(target)
    }

    fn next_version(
        &self,
        vault_id: VaultId,
    ) -> Result<ConfigVersion, ConfigCommandError> {
        let candidate = self
            .command_port
            .load_active_version(vault_id)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?
            .map(ConfigVersion::next)
            .transpose()
            .map_err(ConfigCommandError::Domain)?
            .unwrap_or_else(ConfigVersion::initial);

        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{
        config::ports::ConfigCommandPort,
        db::{Database, DbError},
    };

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

    impl ConfigCommandPort for DbPort<'_> {
        type Error = DbError;

        fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
            self.db.put("config", "global", config)
        }

        fn save_vault(
            &self,
            vault_id: VaultId,
            config: &Vault,
        ) -> Result<(), Self::Error> {
            self.db.put("config", &vault_id.to_string(), config)
        }

        fn load_global(&self) -> Result<Option<Global>, Self::Error> {
            self.db.get_owned("config", "global")
        }

        fn load_vault(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Vault>, Self::Error> {
            self.db.get_owned("config", &vault_id.to_string())
        }

        fn save_merged(
            &self,
            vault_id: VaultId,
            version: ConfigVersion,
            config: &Config,
        ) -> Result<(), Self::Error> {
            let key = format!("{}:{}", vault_id, version.value());
            self.db.put("merged_config_versions", &key, config)
        }

        fn load_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<ConfigVersion>, Self::Error> {
            self.db.get_owned("merged_config_active", &vault_id.to_string())
        }

        fn set_active_version(
            &self,
            vault_id: VaultId,
            version: ConfigVersion,
        ) -> Result<(), Self::Error> {
            self.db.put("merged_config_active", &vault_id.to_string(), &version)
        }

        fn save_vault_path_mapping(
            &self,
            vault_id: VaultId,
            vault_root: &VaultRoot,
        ) -> Result<(), Self::Error> {
            self.db.put(
                "vault_id_by_path",
                vault_root.as_path().to_string_lossy().as_ref(),
                &vault_id,
            )?;
            self.db.put("vault_path_by_id", &vault_id.to_string(), vault_root)
        }
    }

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("config.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    #[test]
    fn save_global_persists_configuration() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));

        let global = Global::default();
        cmd.save_global(&global).map_err(|e| e.to_string())?;

        let stored = db
            .get_owned::<Global>("config", "global")
            .map_err(|e| e.to_string())?;
        let stored_global = stored
            .ok_or_else(|| "Stored global config should exist".to_owned())?;
        if stored_global != global {
            return Err("Stored global config should match input".to_owned());
        }
        Ok(())
    }

    #[test]
    fn save_vault_persists_configuration() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));

        let vault = Vault::default();
        let vault_id = VaultId::new();
        cmd.save_vault(vault_id, &vault).map_err(|e| e.to_string())?;

        let stored = db
            .get_owned::<Vault>("config", &vault_id.to_string())
            .map_err(|e| e.to_string())?;
        let stored_vault = stored
            .ok_or_else(|| "Stored vault config should exist".to_owned())?;
        if stored_vault != vault {
            return Err("Stored vault config should match input".to_owned());
        }
        Ok(())
    }
}
