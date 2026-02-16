//! Configuration command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles configuration
//! mutations (saving settings, rebuilding snapshots) while relying on a
//! query port for version lookups during write workflows.

use tracing::instrument;

use super::{
    aggregate::{Config, Version},
    error::{ConfigCommandError, ConfigError},
    global::Global,
    ingest,
    ports::{self as config_ports},
    vault::{Vault, VaultId, VaultRoot},
};

/// Command implementation for configuration write operations.
///
/// This struct handles configuration mutations (saving, rebuilding, version
/// activation). It composes a query port for read-only version lookups and a
/// command port for persistence.
///
/// # Examples
///
/// ```rust
/// # use lithos_core::config::{
/// #     aggregate::{Config, Version},
/// #     command::Command,
/// #     global::Global,
/// #     vault::{Vault, VaultId, VaultRoot},
/// #     ports,
/// # };
/// # struct MockQueryPort;
/// # struct MockCommandPort;
/// #
/// # impl ports::Query for MockQueryPort {
/// #     type Error = std::io::Error;
/// #
/// #     fn get_active_version(
/// #         &self,
/// #         _: VaultId,
/// #     ) -> Result<Option<Version>, Self::Error> {
/// #         Ok(None)
/// #     }
/// #
/// #     fn get_global(&self) -> Result<Option<Global>, Self::Error> {
/// #         Ok(None)
/// #     }
/// #
/// #     fn get_vault(
/// #         &self,
/// #         _: VaultId,
/// #     ) -> Result<Option<Vault>, Self::Error> {
/// #         Ok(None)
/// #     }
/// #
/// #     fn get_merged_owned(
/// #         &self,
/// #         _: VaultId,
/// #         _: Version,
/// #     ) -> Result<Option<Config>, Self::Error> {
/// #         Ok(None)
/// #     }
/// #
/// #     fn with_archived<R, F>(
/// #         &self,
/// #         _: VaultId,
/// #         _: Version,
/// #         _: F,
/// #     ) -> Result<Option<R>, Self::Error>
/// #     where
/// #         F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
/// #     {
/// #         Ok(None)
/// #     }
/// # }
/// #
/// # impl ports::Command for MockCommandPort {
/// #     type Error = std::io::Error;
/// #
/// #     fn save_global(&self, _: &Global) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// #
/// #     fn save_merged(
/// #         &self,
/// #         _: VaultId,
/// #         _: Version,
/// #         _: &Config,
/// #     ) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// #
/// #     fn save_vault(
/// #         &self,
/// #         _: VaultId,
/// #         _: &Vault,
/// #     ) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// #
/// #     fn save_vault_path_mapping(
/// #         &self,
/// #         _: VaultId,
/// #         _: &VaultRoot,
/// #     ) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// #
/// #     fn set_active_version(
/// #         &self,
/// #         _: VaultId,
/// #         _: Version,
/// #     ) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// # }
/// let cmd = Command::new(MockQueryPort, MockCommandPort);
/// ```
pub struct Command<Q, C> {
    /// Port interface for query storage operations.
    query_port: Q,
    /// Port interface for command storage operations.
    command_port: C,
}

impl<Q, C> Command<Q, C> {
    /// Creates a new `Command` with the given ports.
    #[inline]
    #[must_use]
    pub const fn new(query_port: Q, command_port: C) -> Self {
        Self {
            query_port,
            command_port,
        }
    }
}

impl<Q, C> Command<Q, C>
where
    Q: config_ports::Query,
    C: config_ports::Command,
    Q::Error: Into<crate::db::DbError>,
    C::Error: Into<crate::db::DbError>,
{
    /// Saves the global configuration.
    ///
    /// # Errors
    /// Returns [`ConfigCommandError::Storage`] if persistence fails.
    #[inline]
    #[instrument(skip(self, config), fields(operation = "save_global"))]
    pub fn save_global(
        &self,
        config: &Global,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .save_global(config)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Saves a vault-specific configuration.
    ///
    /// # Errors
    /// Returns [`ConfigCommandError::Storage`] if persistence fails.
    #[inline]
    #[instrument(
        skip(self, config),
        fields(operation = "save_vault", vault_id = %vault_id)
    )]
    pub fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .save_vault(vault_id, config)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Rebuilds the merged configuration read model for a vault.
    ///
    /// This method performs the full configuration lifecycle:
    /// 1. **Ingestion**: Loads raw configuration from files using Figment.
    /// 2. **Merging**: Layers vault overrides on top of global settings and
    ///    defaults.
    /// 3. **Validation**: Transforms merged raw data into an "Always Valid"
    ///    [`Config`].
    /// 4. **Versioning**: Generates a new [`Version`].
    /// 5. **Persistence**: Saves the new snapshot and updates the active
    ///    pointer.
    ///
    /// # Errors
    /// Returns [`ConfigCommandError`] if ingestion, validation, or persistence
    /// fails.
    #[inline]
    #[instrument(
        skip(self, vault_root),
        fields(operation = "rebuild_merged", vault_id = %vault_id)
    )]
    pub fn rebuild_merged(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<Version, ConfigCommandError> {
        // Build merged config using the simplified API
        let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
        let merged = Config::build(&raw_merged, vault_id, vault_root.clone())
            .map_err(ConfigCommandError::Domain)?;

        // Save vault path mapping
        self.command_port
            .save_vault_path_mapping(vault_id, vault_root)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        // Save merged config and set as active
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
    #[instrument(
        skip(self),
        fields(
            operation = "activate_version",
            vault_id = %vault_id,
            version = %version
        )
    )]
    pub fn activate_version(
        &self,
        vault_id: VaultId,
        version: Version,
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
    #[instrument(
        skip(self),
        fields(operation = "rollback_version", vault_id = %vault_id, steps = %steps)
    )]
    pub fn rollback(
        &self,
        vault_id: VaultId,
        steps: u32,
    ) -> Result<Version, ConfigCommandError> {
        let active = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?
            .ok_or_else(|| {
                ConfigCommandError::Domain(ConfigError::ValidationFailed {
                    field: "config_version".into(),
                    message: "no active version".into(),
                })
            })?;

        let steps = u64::from(steps);
        let current = active.value();
        let target = current.saturating_sub(steps);
        if target == 0 {
            return Err(ConfigCommandError::Domain(
                ConfigError::ValidationFailed {
                    field: "config_version".into(),
                    message: "rollback underflow".into(),
                },
            ));
        }

        let target = Version::try_from(target)?;
        self.activate_version(vault_id, target)?;
        Ok(target)
    }

    fn next_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Version, ConfigCommandError> {
        let candidate = self
            .query_port
            .get_active_version(vault_id)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?
            .map(Version::next)
            .transpose()
            .map_err(ConfigCommandError::Domain)?
            .unwrap_or_else(Version::initial);

        Ok(candidate)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use super::*;
    use crate::{
        config::{
            db_table::{
                CONFIG, MERGED_CONFIG_ACTIVE, MERGED_CONFIG_VERSIONS,
                VAULT_ID_BY_PATH, VAULT_PATH_BY_ID,
            },
            ports as config_ports,
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

    struct DbQueryPort<'db> {
        db: &'db Database,
    }

    impl<'db> DbQueryPort<'db> {
        fn new(db: &'db Database) -> Self {
            Self {
                db,
            }
        }
    }

    impl config_ports::Query for DbQueryPort<'_> {
        type Error = DbError;

        fn get_active_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Version>, Self::Error> {
            self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
        }

        fn get_global(&self) -> Result<Option<Global>, Self::Error> {
            self.db.get_owned(CONFIG, "global")
        }

        fn get_vault(
            &self,
            vault_id: VaultId,
        ) -> Result<Option<Vault>, Self::Error> {
            self.db.get_owned(CONFIG, &vault_id.to_string())
        }

        fn get_merged_owned(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<Option<Config>, Self::Error> {
            let key = format!("{vault_id}:{}", version.value());
            self.db.get_owned(MERGED_CONFIG_VERSIONS, &key)
        }

        fn with_archived<R, F>(
            &self,
            _vault_id: VaultId,
            _version: Version,
            _f: F,
        ) -> Result<Option<R>, Self::Error>
        where
            F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
        {
            Ok(None)
        }
    }

    struct DbCommandPort<'db> {
        db: &'db Database,
    }

    impl<'db> DbCommandPort<'db> {
        fn new(db: &'db Database) -> Self {
            Self {
                db,
            }
        }
    }

    impl config_ports::Command for DbCommandPort<'_> {
        type Error = DbError;

        fn save_global(&self, config: &Global) -> Result<(), Self::Error> {
            self.db.put(CONFIG, "global", config)
        }

        fn save_vault(
            &self,
            vault_id: VaultId,
            config: &Vault,
        ) -> Result<(), Self::Error> {
            self.db.put(CONFIG, &vault_id.to_string(), config)
        }

        fn save_merged(
            &self,
            vault_id: VaultId,
            version: Version,
            config: &Config,
        ) -> Result<(), Self::Error> {
            let key = format!("{vault_id}:{}", version.value());
            self.db.put(MERGED_CONFIG_VERSIONS, &key, config)
        }

        fn set_active_version(
            &self,
            vault_id: VaultId,
            version: Version,
        ) -> Result<(), Self::Error> {
            self.db.put(MERGED_CONFIG_ACTIVE, &vault_id.to_string(), &version)
        }

        fn save_vault_path_mapping(
            &self,
            vault_id: VaultId,
            vault_root: &VaultRoot,
        ) -> Result<(), Self::Error> {
            self.db.put(
                VAULT_ID_BY_PATH,
                vault_root.as_path().to_string_lossy().as_ref(),
                &vault_id,
            )?;
            self.db.put(VAULT_PATH_BY_ID, &vault_id.to_string(), vault_root)
        }
    }

    mod persistence {
        use super::*;

        #[test]
        fn save_global_persists_configuration() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));

            let global = Global::default();
            cmd.save_global(&global).unwrap();

            let stored = db.get_owned::<Global>(CONFIG, "global").unwrap();
            let stored_global =
                stored.ok_or("Stored global config should exist").unwrap();
            assert_eq!(
                stored_global, global,
                "Stored global config should match input"
            );
        }

        #[test]
        fn save_vault_persists_configuration() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));

            let vault = Vault::default();
            let vault_id = VaultId::new();
            cmd.save_vault(vault_id, &vault).unwrap();

            let stored =
                db.get_owned::<Vault>(CONFIG, &vault_id.to_string()).unwrap();
            let stored_vault =
                stored.ok_or("Stored vault config should exist").unwrap();
            assert_eq!(
                stored_vault, vault,
                "Stored vault config should match input"
            );
        }
    }

    mod update {
        use super::*;

        #[test]
        fn rollback_updates_active_version() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();

            // Setup: Version 1 and 2
            db.put(
                MERGED_CONFIG_ACTIVE,
                &vault_id.to_string(),
                &Version::try_from(2).unwrap(),
            )
            .unwrap();

            // Rollback 1 step
            let target = cmd.rollback(vault_id, 1).unwrap();
            assert_eq!(target.value(), 1);

            let active: Option<Version> = db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
                .unwrap();
            assert_eq!(active.expect("active version should exist").value(), 1);
        }

        #[test]
        fn activate_version_updates_active_pointer() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let version = Version::try_from(5).unwrap();

            cmd.activate_version(vault_id, version).unwrap();

            let active: Option<Version> = db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
                .unwrap();
            assert_eq!(active.expect("active version should exist"), version);
        }

        #[test]
        fn rebuild_merged_persists_and_versions() {
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let vault_root =
                VaultRoot::try_new(dir.path().join("vault")).unwrap();
            std::fs::create_dir_all(vault_root.as_path()).unwrap();

            let version = cmd.rebuild_merged(vault_id, &vault_root).unwrap();
            assert_eq!(version.value(), 1);

            let active: Option<Version> = db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
                .unwrap();
            assert_eq!(active.expect("active version should exist"), version);
        }

        #[test]
        fn rebuild_merged_reads_vault_config_file() {
            // GIVEN: vault with config file
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let vault_root =
                VaultRoot::try_new(dir.path().join("vault")).unwrap();

            // Create vault with config
            std::fs::create_dir_all(vault_root.as_path().join(".lithos"))
                .unwrap();
            std::fs::write(
                vault_root.as_path().join(".lithos").join("lithos.toml"),
                "[logging]\nlog_level = \"debug\"\n",
            )
            .unwrap();

            // WHEN: rebuilding merged config
            let version = cmd.rebuild_merged(vault_id, &vault_root).unwrap();

            // THEN: config is read from file and persisted
            let key = format!("{vault_id}:{}", version.value());
            let stored: Option<Config> =
                db.get_owned(MERGED_CONFIG_VERSIONS, &key).unwrap();
            let config = stored.expect("merged config should be persisted");

            assert_eq!(config.logging().level_str(), "debug");
        }

        #[test]
        fn rebuild_merged_applies_defaults_when_no_config_file() {
            // GIVEN: vault without config file
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let vault_root =
                VaultRoot::try_new(dir.path().join("vault")).unwrap();
            std::fs::create_dir_all(vault_root.as_path()).unwrap();

            // WHEN: rebuilding merged config
            let version = cmd.rebuild_merged(vault_id, &vault_root).unwrap();

            // THEN: defaults are applied
            let key = format!("{vault_id}:{}", version.value());
            let stored: Option<Config> =
                db.get_owned(MERGED_CONFIG_VERSIONS, &key).unwrap();
            let config = stored.expect("merged config should be persisted");

            assert_eq!(config.logging().level_str(), "info"); // default
        }

        #[test]
        fn rebuild_merged_saves_vault_path_mapping() {
            // GIVEN: vault directory
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let vault_root =
                VaultRoot::try_new(dir.path().join("vault")).unwrap();
            std::fs::create_dir_all(vault_root.as_path()).unwrap();

            // WHEN: rebuilding merged config
            cmd.rebuild_merged(vault_id, &vault_root).unwrap();

            // THEN: vault path mapping is saved
            let stored_root: Option<VaultRoot> =
                db.get_owned(VAULT_PATH_BY_ID, &vault_id.to_string()).unwrap();
            assert_eq!(
                stored_root.expect("vault root should be mapped"),
                vault_root
            );
        }

        #[test]
        fn rebuild_merged_increments_version_on_subsequent_calls() {
            // GIVEN: vault with existing merged config
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd =
                Command::new(DbQueryPort::new(&db), DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let vault_root =
                VaultRoot::try_new(dir.path().join("vault")).unwrap();
            std::fs::create_dir_all(vault_root.as_path()).unwrap();

            // WHEN: rebuilding multiple times
            let v1 = cmd.rebuild_merged(vault_id, &vault_root).unwrap();
            let v2 = cmd.rebuild_merged(vault_id, &vault_root).unwrap();

            // THEN: versions increment
            assert_eq!(v1.value(), 1);
            assert_eq!(v2.value(), 2);
        }
    }
}
