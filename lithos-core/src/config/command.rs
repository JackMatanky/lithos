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

    mod fixtures {
        use super::*;

        pub fn test_db()
        -> Result<(TempDir, Database), Box<dyn std::error::Error>> {
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

    #[expect(
        clippy::type_complexity,
        reason = "Test database setup returns complex tuple."
    )]
    fn test_db() -> Result<(TempDir, Database), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("config.redb");
        let db = Database::open(&path)?;
        Ok((dir, db))
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! which can panic."
    )]
    fn save_global_persists_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));

        let global = Global::default();
        cmd.save_global(&global)?;

        let stored = db.get_owned::<Global>("config", "global")?;
        let stored_global =
            stored.ok_or("Stored global config should exist")?;
        assert_eq!(
            stored_global, global,
            "Stored global config should match input"
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! which can panic."
    )]
    fn save_vault_persists_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));

        let vault = Vault::default();
        let vault_id = VaultId::new();
        cmd.save_vault(vault_id, &vault)?;

        let stored = db.get_owned::<Vault>("config", &vault_id.to_string())?;
        let stored_vault = stored.ok_or("Stored vault config should exist")?;
        assert_eq!(
            stored_vault, vault,
            "Stored vault config should match input"
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! and unwrap which can panic."
    )]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test uses unwrap on Option for concise assertion."
    )]
    fn rollback_updates_active_version()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));
        let vault_id = VaultId::new();

        // Setup: Version 1 and 2
        db.put(
            "merged_config_active",
            &vault_id.to_string(),
            &ConfigVersion::try_from(2)?,
        )?;

        // Rollback 1 step
        let target = cmd.rollback(vault_id, 1)?;
        assert_eq!(target.value(), 1);

        let active: Option<ConfigVersion> =
            db.get_owned("merged_config_active", &vault_id.to_string())?;
        assert_eq!(active.unwrap().value(), 1);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! and unwrap which can panic."
    )]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test uses unwrap on Option for concise assertion."
    )]
    fn activate_version_updates_active_pointer()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(DbPort::new(&db));
        let vault_id = VaultId::new();
        let version = ConfigVersion::try_from(5)?;

        cmd.activate_version(vault_id, version)?;

        let active: Option<ConfigVersion> =
            db.get_owned("merged_config_active", &vault_id.to_string())?;
        assert_eq!(active.unwrap(), version);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert_eq! which can panic."
    )]
    fn load_vault_returns_stored_config()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = fixtures::test_db()?;
        let cmd = Command::new(DbPort::new(&db));
        let vault_id = VaultId::new();
        let vault = Vault::default();

        db.put("config", &vault_id.to_string(), &vault)?;

        let loaded = cmd.load_vault(vault_id)?;
        assert_eq!(loaded, Some(vault));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert! which can panic."
    )]
    fn rebuild_merged_persists_and_versions()
    -> Result<(), Box<dyn std::error::Error>> {
        let (dir, db) = fixtures::test_db()?;
        let cmd = Command::new(DbPort::new(&db));
        let vault_id = VaultId::new();
        let vault_root = VaultRoot::try_new(dir.path().join("vault"))?;
        std::fs::create_dir_all(vault_root.as_path())?;

        let version = cmd.rebuild_merged(vault_id, &vault_root)?;
        assert_eq!(version.value(), 1);

        let active: Option<ConfigVersion> =
            db.get_owned("merged_config_active", &vault_id.to_string())?;
        assert_eq!(active.unwrap(), version);

        Ok(())
    }
}
