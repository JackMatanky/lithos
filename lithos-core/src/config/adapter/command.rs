//! Concrete implementation of the [`crate::config::ports::Command`] trait.

use tracing::instrument;

use super::stored::ConfigMetadata;
use crate::{
    config::{
        aggregate::{Config, Timestamp, Version},
        db_table::{
            CONFIG_METADATA, CONFIG_VERSIONS, GLOBAL_CONFIG, VAULT_CONFIG,
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
        skip(self, config),
        fields(operation = "record_global", version = %config.version())
    )]
    fn record_global(
        &self,
        config: &Global,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error> {
        let version_key = config.version().value().to_string();
        let metadata_key = format!("global:{}", config.version().value());
        let metadata = ConfigMetadata::new(created_at, modified_at);

        self.db.batch_write(|tx| {
            tx.put(GLOBAL_CONFIG, &version_key, config)?;
            tx.put(CONFIG_METADATA, &metadata_key, &metadata)
        })
    }

    #[inline]
    #[instrument(
        skip(self, config),
        fields(operation = "record_config", vault_id = %vault_id)
    )]
    fn record_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error> {
        // Atomically allocate version and record config
        self.db.read_write_unit_of_work(|tx| {
            // Scan CONFIG_VERSIONS for max version with this vault_id prefix
            let prefix = format!("{vault_id}:");

            #[expect(
                clippy::return_and_then,
                reason = "filter_map chain is more readable than nested match"
            )]
            let max_version = tx
                .scan_range::<Config>(CONFIG_VERSIONS, &prefix)?
                .into_iter()
                .filter_map(|(key, _)| {
                    // Extract version from key format "{vault_id}:{version}"
                    key.strip_prefix(&prefix)
                        .and_then(|v| v.parse::<u64>().ok())
                        .and_then(|v| Version::try_from(v).ok())
                })
                .max();

            // Compute next version
            let next = match max_version {
                Some(v) => v.next().map_err(|_err| {
                    DbError::Serialization(
                        "config version overflow - vault has exceeded maximum \
                         rebuilds"
                            .into(),
                    )
                })?,
                None => Version::initial(),
            };

            // Update config with allocated version and write
            let versioned_config = config.clone().with_version(next);
            let key = format!("{}:{}", vault_id, next.value());
            tx.put(CONFIG_VERSIONS, &key, &versioned_config)?;

            Ok(next)
        })
    }

    #[inline]
    #[instrument(
        skip(self, config),
        fields(
            operation = "record_vault",
            vault_id = %vault_id,
            version = %config.version()
        )
    )]
    fn record_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<(), Self::Error> {
        let version_key = format!("{}:{}", vault_id, config.version().value());
        let metadata_key = format!("{}:{}", vault_id, config.version().value());
        let metadata = ConfigMetadata::new(created_at, modified_at);

        self.db.batch_write(|tx| {
            tx.put(VAULT_CONFIG, &version_key, config)?;
            tx.put(CONFIG_METADATA, &metadata_key, &metadata)
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
        let version_key = global.version().value().to_string();
        let metadata_key = format!("global:{}", global.version().value());
        let stored_config: Option<Global> = db
            .get_owned(GLOBAL_CONFIG, &version_key)
            .expect("read should succeed");
        assert_eq!(stored_config, Some(global), "config should be persisted");

        // Verify metadata persisted
        let stored_metadata: Option<ConfigMetadata> = db
            .get_owned(CONFIG_METADATA, &metadata_key)
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
        let vault_key = format!("{}:{}", vault_id, vault.version().value());
        let metadata_key = format!("{}:{}", vault_id, vault.version().value());
        let stored_config: Option<Vault> = db
            .get_owned(VAULT_CONFIG, &vault_key)
            .expect("read should succeed");
        assert_eq!(stored_config, Some(vault), "config should be persisted");

        // Verify metadata persisted
        let stored_metadata: Option<ConfigMetadata> = db
            .get_owned(CONFIG_METADATA, &metadata_key)
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
        let version_key = global.version().value().to_string();
        let metadata_key = format!("global:{}", global.version().value());
        let config_exists = db
            .get_owned::<Global>(GLOBAL_CONFIG, &version_key)
            .expect("read should succeed")
            .is_some();
        let metadata_exists = db
            .get_owned::<ConfigMetadata>(CONFIG_METADATA, &metadata_key)
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

        let vault_key = format!("{}:{}", vault_id, vault.version().value());

        // Both config and metadata should exist
        let config_exists = db
            .get_owned::<Vault>(VAULT_CONFIG, &vault_key)
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
