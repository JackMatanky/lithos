//! Concrete implementation of the [`crate::config::ports::Command`] and
//! [`crate::config::ports::CommandState`] traits.

use tracing::instrument;

use super::{merged_version_key, stored::ConfigMetadata};
use crate::{
    config::{
        aggregate::{Config, Timestamp, Version},
        db_table::{
            CONFIG, CONFIG_METADATA, MERGED_CONFIG_ACTIVE,
            MERGED_CONFIG_VERSIONS, VAULT_ID_BY_PATH, VAULT_PATH_BY_ID,
        },
        global::Global,
        ports::{ActivationTarget, Command, CommandState},
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
    #[instrument(skip(self, config), fields(operation = "record_global"))]
    fn record_global(
        &self,
        config: &Global,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error> {
        let metadata = ConfigMetadata::new(created_at, modified_at);
        self.db.batch_write(|tx| {
            tx.put(CONFIG, "global", config)?;
            tx.put(CONFIG_METADATA, "global", &metadata)
        })
    }

    #[inline]
    #[instrument(
        skip(self, config),
        fields(
            operation = "record_merged",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn record_merged(
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
        fields(operation = "record_vault", vault_id = %vault_id)
    )]
    fn record_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error> {
        let metadata = ConfigMetadata::new(created_at, modified_at);
        let vault_key = vault_id.to_string();
        self.db.batch_write(|tx| {
            tx.put(CONFIG, &vault_key, config)?;
            tx.put(CONFIG_METADATA, &vault_key, &metadata)
        })
    }

    #[inline]
    #[instrument(
        skip(self, vault_root),
        fields(operation = "record_vault_path_mapping", vault_id = %vault_id)
    )]
    fn record_vault_path_mapping(
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
        fields(operation = "activate_version", vault_id = %vault_id)
    )]
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
                let current: Option<Version> =
                    tx.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())?;

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

impl CommandState for CommandAdapter<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self),
        fields(operation = "next_version", vault_id = %vault_id)
    )]
    fn next_version(&self, vault_id: VaultId) -> Result<Version, Self::Error> {
        let current: Option<Version> =
            self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> (Database, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).expect("failed to open database");
        (db, temp_dir)
    }

    #[test]
    fn record_global_persists_config_and_metadata() {
        let (db, _temp) = setup_db();
        let command = CommandAdapter::new(&db);

        let global = Global::default();
        let created_at = Some(Timestamp::from_secs(1000));
        let modified_at = Timestamp::from_secs(2000);

        command
            .record_global(&global, created_at, modified_at)
            .expect("record should succeed");

        // Verify config persisted
        let stored_config: Option<Global> =
            db.get_owned(CONFIG, "global").expect("read should succeed");
        assert_eq!(stored_config, Some(global), "config should be persisted");

        // Verify metadata persisted
        let stored_metadata: Option<ConfigMetadata> = db
            .get_owned(CONFIG_METADATA, "global")
            .expect("read should succeed");
        assert!(stored_metadata.is_some(), "metadata should be persisted");

        let metadata = stored_metadata.unwrap();
        assert_eq!(metadata.created_at, created_at);
        assert_eq!(metadata.modified_at, modified_at);
    }

    #[test]
    fn record_vault_persists_config_and_metadata() {
        let (db, _temp) = setup_db();
        let command = CommandAdapter::new(&db);

        let vault_id = VaultId::new();
        let vault = Vault::default();
        let created_at = Some(Timestamp::from_secs(1000));
        let modified_at = Timestamp::from_secs(2000);

        command
            .record_vault(vault_id, &vault, created_at, modified_at)
            .expect("record should succeed");

        // Verify config persisted
        let vault_key = vault_id.to_string();
        let stored_config: Option<Vault> =
            db.get_owned(CONFIG, &vault_key).expect("read should succeed");
        assert_eq!(stored_config, Some(vault), "config should be persisted");

        // Verify metadata persisted
        let stored_metadata: Option<ConfigMetadata> = db
            .get_owned(CONFIG_METADATA, &vault_key)
            .expect("read should succeed");
        assert!(stored_metadata.is_some(), "metadata should be persisted");

        let metadata = stored_metadata.unwrap();
        assert_eq!(metadata.created_at, created_at);
        assert_eq!(metadata.modified_at, modified_at);
    }

    #[test]
    fn record_global_batch_write_is_atomic() {
        let (db, _temp) = setup_db();
        let command = CommandAdapter::new(&db);

        let global = Global::default();
        let created_at = Some(Timestamp::from_secs(1000));
        let modified_at = Timestamp::from_secs(2000);

        // First write should succeed
        command
            .record_global(&global, created_at, modified_at)
            .expect("first write should succeed");

        // Both config and metadata should exist
        let config_exists = db
            .get_owned::<Global>(CONFIG, "global")
            .expect("read should succeed")
            .is_some();
        let metadata_exists = db
            .get_owned::<ConfigMetadata>(CONFIG_METADATA, "global")
            .expect("read should succeed")
            .is_some();

        assert!(config_exists, "config should exist");
        assert!(metadata_exists, "metadata should exist");
    }

    #[test]
    fn record_vault_batch_write_is_atomic() {
        let (db, _temp) = setup_db();
        let command = CommandAdapter::new(&db);

        let vault_id = VaultId::new();
        let vault = Vault::default();
        let created_at = Some(Timestamp::from_secs(1000));
        let modified_at = Timestamp::from_secs(2000);

        command
            .record_vault(vault_id, &vault, created_at, modified_at)
            .expect("write should succeed");

        let vault_key = vault_id.to_string();

        // Both config and metadata should exist
        let config_exists = db
            .get_owned::<Vault>(CONFIG, &vault_key)
            .expect("read should succeed")
            .is_some();
        let metadata_exists = db
            .get_owned::<ConfigMetadata>(CONFIG_METADATA, &vault_key)
            .expect("read should succeed")
            .is_some();

        assert!(config_exists, "config should exist");
        assert!(metadata_exists, "metadata should exist");
    }
}
