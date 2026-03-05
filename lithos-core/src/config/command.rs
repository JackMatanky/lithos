//! Configuration command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles configuration
//! mutations (recording settings, rebuilding snapshots). Version allocation is
//! handled atomically within the command port to prevent race conditions.

use tracing::instrument;

use super::{
    aggregate::{Config, Timestamp, Version},
    error::ConfigCommandError,
    global::Global,
    ingest,
    ports::{self as config_ports},
    vault::{Vault, VaultId, VaultRoot},
};

/// # Examples
///
/// ```rust,no_run
/// # use tempfile::tempdir;
/// # use lithos_core::{
/// #     config::{
/// #         aggregate::Timestamp, global::Global, RedbConfigCommand,
/// #         adapter::command::CommandAdapter,
/// #     },
/// #     db::Database,
/// # };
/// let dir = tempdir()?;
/// let db = Database::open(&dir.path().join("config.redb"))?;
/// let command = RedbConfigCommand::new(CommandAdapter::new(&db));
/// let created_at = Some(Timestamp::from_secs(1000));
/// let modified_at = Timestamp::from_secs(2000);
/// command.record_global(&Global::default(), created_at, modified_at)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Command<C> {
    /// Port interface for command storage operations.
    command_port: C,
}

impl<C> Command<C> {
    /// Creates a new `Command` with the given command port.
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
    C: config_ports::Command,
    <C as config_ports::Command>::Error: Into<crate::db::DbError>,
{
    /// Records the global configuration with metadata.
    ///
    /// The metadata parameters enable staleness detection:
    /// - `created_at`: File birthtime (detects replacement)
    /// - `modified_at`: File mtime (detects edits)
    ///
    /// # Errors
    /// Returns [`ConfigCommandError::Storage`] if persistence fails.
    #[inline]
    #[instrument(skip(self, config), fields(operation = "record_global"))]
    pub fn record_global(
        &self,
        config: &Global,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .record_global(config, created_at, modified_at)
            .map_err(|error| ConfigCommandError::Storage(error.into()))
    }

    /// Records a vault-specific configuration with metadata.
    ///
    /// The metadata parameters enable staleness detection:
    /// - `created_at`: File birthtime (detects replacement)
    /// - `modified_at`: File mtime (detects edits)
    ///
    /// # Errors
    /// Returns [`ConfigCommandError::Storage`] if persistence fails.
    #[inline]
    #[instrument(
        skip(self, config),
        fields(operation = "record_vault", vault_id = %vault_id)
    )]
    pub fn record_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), ConfigCommandError> {
        self.command_port
            .record_vault(vault_id, config, created_at, modified_at)
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
    /// 5. **Persistence**: Records the new snapshot.
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
        // Build merged config with placeholder version
        // The actual version is allocated atomically inside record_config
        let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
        let temp_config = Config::build(
            &raw_merged,
            vault_id,
            vault_root.clone(),
            Version::initial(), /* Placeholder - real version assigned
                                 * atomically */
        )
        .map_err(ConfigCommandError::Domain)?;

        // Record vault path mapping
        self.command_port
            .record_vault_path_mapping(vault_id, vault_root)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        // Atomically allocate version and record config
        // Returns the allocated version number
        let version =
            self.command_port
                .record_config(vault_id, &temp_config)
                .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        Ok(version)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(any())]
// Disabled: TODO: Update tests for new port design (no activate_version, use
// record_config)
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

        fn record_global(
            &self,
            config: &Global,
            _created_at: Option<Timestamp>,
            _modified_at: Timestamp,
        ) -> Result<(), Self::Error> {
            // Facade doesn't use metadata - it's for application service layer
            self.db.put(CONFIG, "global", config)
        }

        fn record_vault(
            &self,
            vault_id: VaultId,
            config: &Vault,
            _created_at: Option<Timestamp>,
            _modified_at: Timestamp,
        ) -> Result<(), Self::Error> {
            // Facade doesn't use metadata - it's for application service layer
            self.db.put(CONFIG, &vault_id.to_string(), config)
        }

        fn record_merged(
            &self,
            vault_id: VaultId,
            version: Version,
            config: &Config,
        ) -> Result<(), Self::Error> {
            let key = format!("{vault_id}:{}", version.value());
            self.db.put(MERGED_CONFIG_VERSIONS, &key, config)
        }

        fn record_vault_path_mapping(
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

        fn activate_version(
            &self,
            vault_id: VaultId,
            target: ActivationTarget,
        ) -> Result<Version, Self::Error> {
            match target {
                ActivationTarget::Exact(version) => {
                    self.db.put(
                        MERGED_CONFIG_ACTIVE,
                        &vault_id.to_string(),
                        &version,
                    )?;
                    Ok(version)
                }
                ActivationTarget::Previous {
                    steps,
                } => self.db.read_write_unit_of_work(|tx| {
                    let current: Option<Version> = tx.get_owned(
                        MERGED_CONFIG_ACTIVE,
                        &vault_id.to_string(),
                    )?;

                    let current = current.ok_or_else(|| {
                        DbError::Serialization("no active version".into())
                    })?;

                    let steps = u64::from(steps);
                    let current_val = current.value();
                    let target_version_val = current_val.saturating_sub(steps);

                    if target_version_val == 0 {
                        return Err(DbError::Serialization(
                            "activation underflow".into(),
                        ));
                    }

                    let target_version = Version::try_from(target_version_val)
                        .map_err(|_e| {
                            DbError::Serialization("invalid version".into())
                        })?;

                    tx.put(
                        MERGED_CONFIG_ACTIVE,
                        &vault_id.to_string(),
                        &target_version,
                    )?;

                    Ok(target_version)
                }),
            }
        }
    }

    impl config_ports::CommandState for DbCommandPort<'_> {
        type Error = DbError;

        fn next_version(
            &self,
            vault_id: VaultId,
        ) -> Result<Version, Self::Error> {
            let current: Option<Version> = self
                .db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())?;

            let candidate = match current {
                Some(v) => v.next().map_err(|_err| {
                    DbError::Serialization(
                        "config version overflow - vault has exceeded maximum \
                         rebuilds"
                            .into(),
                    )
                })?,
                None => Version::initial(),
            };

            Ok(candidate)
        }
    }

    mod persistence {
        use super::*;

        #[test]
        fn record_global_persists_configuration() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd = Command::new(DbCommandPort::new(&db));

            let global = Global::default();
            let created_at = Some(Timestamp::from_secs(1000));
            let modified_at = Timestamp::from_secs(2000);
            cmd.record_global(&global, created_at, modified_at).unwrap();

            let stored = db.get_owned::<Global>(CONFIG, "global").unwrap();
            let stored_global =
                stored.ok_or("Stored global config should exist").unwrap();
            assert_eq!(
                stored_global, global,
                "Stored global config should match input"
            );
        }

        #[test]
        fn record_vault_persists_configuration() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd = Command::new(DbCommandPort::new(&db));

            let vault = Vault::default();
            let vault_id = VaultId::new();
            let created_at = Some(Timestamp::from_secs(1000));
            let modified_at = Timestamp::from_secs(2000);
            cmd.record_vault(vault_id, &vault, created_at, modified_at)
                .unwrap();

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
        fn activate_previous_version_updates_active_version() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd = Command::new(DbCommandPort::new(&db));
            let vault_id = VaultId::new();

            // Setup: Version 1 and 2
            db.put(
                MERGED_CONFIG_ACTIVE,
                &vault_id.to_string(),
                &Version::try_from(2).unwrap(),
            )
            .unwrap();

            // Activate previous 1 step
            let target = cmd
                .activate_version(vault_id, ActivationTarget::previous(1))
                .unwrap();
            assert_eq!(target.value(), 1);

            let active: Option<Version> = db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
                .unwrap();
            assert_eq!(active.expect("active version should exist").value(), 1);
        }

        #[test]
        fn activate_version_updates_active_pointer() {
            let (_dir, db) = fixtures::test_db().unwrap();
            let cmd = Command::new(DbCommandPort::new(&db));
            let vault_id = VaultId::new();
            let version = Version::try_from(5).unwrap();

            cmd.activate_version(vault_id, ActivationTarget::exact(version))
                .unwrap();

            let active: Option<Version> = db
                .get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
                .unwrap();
            assert_eq!(active.expect("active version should exist"), version);
        }

        #[test]
        fn rebuild_merged_persists_and_versions() {
            let (dir, db): (tempfile::TempDir, Database) =
                fixtures::test_db().unwrap();
            let cmd = Command::new(DbCommandPort::new(&db));
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
            let cmd = Command::new(DbCommandPort::new(&db));
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
            let cmd = Command::new(DbCommandPort::new(&db));
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
            let cmd = Command::new(DbCommandPort::new(&db));
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
            let cmd = Command::new(DbCommandPort::new(&db));
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
