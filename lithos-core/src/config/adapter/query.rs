//! Redb-backed implementation of the [`crate::config::ports::Query`] trait.

use std::time::SystemTime;

use tracing::instrument;

use super::stored::ConfigMetadata;
use crate::{
    config::{
        aggregate::{Config, Version},
        db_table::{
            CONFIG_METADATA, CONFIG_VERSIONS, GLOBAL_CONFIG, VAULT_CONFIG,
            VAULT_ID_BY_PATH,
        },
        global::{Global, GlobalVersion},
        ports::Query as QueryPort,
        vault::{Vault, VaultId, VaultRoot, VaultVersion},
    },
    db::{Database, DbError},
};

/// Redb-backed config query adapter.
pub struct Query<'db> {
    db: &'db Database,
}

impl<'db> Query<'db> {
    #[inline]
    #[must_use]
    /// Create a query adapter for a database.
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl QueryPort for Query<'_> {
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(
            operation = "find_config",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn find_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error> {
        let key = format!("{}:{}", vault_id, version.value());
        self.db.get_owned(CONFIG_VERSIONS, &key)
    }

    #[inline]
    #[instrument(
        skip(self, vault_root),
        level = "debug",
        fields(operation = "find_vault_id_by_path")
    )]
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, Self::Error> {
        let path_key = vault_root.as_key();
        self.db.get_owned(VAULT_ID_BY_PATH, &path_key)
    }

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
        // Scan CONFIG_VERSIONS for max version with this vault_id prefix
        let prefix = format!("{vault_id}:");

        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Config>(CONFIG_VERSIONS, &prefix)?
            .into_iter()
            .filter_map(|(key, _)| {
                // Extract version from key format "{vault_id}:{version}"
                key.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .and_then(|v| Version::try_from(v).ok())
            })
            .max();

        Ok(max_version)
    }

    #[inline]
    #[instrument(skip(self), level = "debug", fields(operation = "get_global"))]
    fn get_global(&self) -> Result<Option<Global>, Self::Error> {
        // Get latest global config (scan for max version)
        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Global>(GLOBAL_CONFIG, "")?
            .into_iter()
            .filter_map(|(key, _)| {
                key.parse::<u64>()
                    .ok()
                    .and_then(|v| GlobalVersion::try_from(v).ok())
            })
            .max();

        match max_version {
            Some(version) => {
                let key = version.value().to_string();
                self.db.get_owned(GLOBAL_CONFIG, &key)
            }
            None => Ok(None),
        }
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "get_vault", vault_id = %vault_id)
    )]
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, Self::Error> {
        // Get latest vault config (scan for max version)
        let prefix = format!("{vault_id}:");

        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Vault>(VAULT_CONFIG, &prefix)?
            .into_iter()
            .filter_map(|(key, _)| {
                key.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .and_then(|v| VaultVersion::try_from(v).ok())
            })
            .max();

        match max_version {
            Some(version) => {
                let key = format!("{}:{}", vault_id, version.value());
                self.db.get_owned(VAULT_CONFIG, &key)
            }
            None => Ok(None),
        }
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "is_global_stale")
    )]
    fn is_global_stale(
        &self,
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
    ) -> Result<bool, Self::Error> {
        // Get latest global version
        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Global>(GLOBAL_CONFIG, "")?
            .into_iter()
            .filter_map(|(key, _)| {
                key.parse::<u64>()
                    .ok()
                    .and_then(|v| GlobalVersion::try_from(v).ok())
            })
            .max();

        let Some(latest_version) = max_version else {
            // No versions stored → config is stale
            return Ok(true);
        };

        // Check metadata for latest version
        let metadata_key = format!("global:{}", latest_version.value());
        let Some(stored) = self
            .db
            .get_owned::<ConfigMetadata>(CONFIG_METADATA, &metadata_key)?
        else {
            // No metadata stored → config is stale
            return Ok(true);
        };

        // Check if file was replaced (created_at differs)
        if let (Some(file_created), Some(stored_created)) =
            (created_at, stored.created_at)
            && file_created != stored_created
        {
            return Ok(true); // Stale: created_at mismatch
        }

        // Check if file was modified (modified_at is newer)
        if stored.modified_at < modified_at {
            return Ok(true); // Stale: file modified after storage
        }

        Ok(false) // Fresh: all checks passed
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "is_vault_stale", vault_id = %vault_id)
    )]
    fn is_vault_stale(
        &self,
        vault_id: VaultId,
        created_at: Option<SystemTime>,
        modified_at: SystemTime,
    ) -> Result<bool, Self::Error> {
        // Get latest vault version
        let prefix = format!("{vault_id}:");
        #[expect(
            clippy::return_and_then,
            reason = "filter_map chain is more readable than nested match"
        )]
        let max_version = self
            .db
            .scan_range::<Vault>(VAULT_CONFIG, &prefix)?
            .into_iter()
            .filter_map(|(key, _)| {
                key.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .and_then(|v| VaultVersion::try_from(v).ok())
            })
            .max();

        let Some(latest_version) = max_version else {
            // No versions stored → config is stale
            return Ok(true);
        };

        // Check metadata for latest version
        let metadata_key = format!("{}:{}", vault_id, latest_version.value());
        let Some(stored) = self
            .db
            .get_owned::<ConfigMetadata>(CONFIG_METADATA, &metadata_key)?
        else {
            // No metadata stored → config is stale
            return Ok(true);
        };

        // Check if file was replaced (created_at differs)
        if let (Some(file_created), Some(stored_created)) =
            (created_at, stored.created_at)
            && file_created != stored_created
        {
            return Ok(true); // Stale: created_at mismatch
        }

        // Check if file was modified (modified_at is newer)
        if stored.modified_at < modified_at {
            return Ok(true); // Stale: file modified after storage
        }

        Ok(false) // Fresh: all checks passed
    }

    #[inline]
    #[instrument(
        skip(self, f),
        level = "debug",
        fields(
            operation = "with_archived",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn with_archived<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        let key = format!("{}:{}", vault_id, version.value());
        self.db.get::<Config, _, _>(CONFIG_VERSIONS, &key, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        adapter::{command::Command as CommandAdapter, stored::ConfigMetadata},
        ports::Command as _,
    };

    fn setup_db() -> (Database, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).expect("failed to open database");
        (db, temp_dir)
    }

    #[test]
    fn is_global_stale_returns_true_when_metadata_missing() {
        let (db, _temp) = setup_db();
        let query = Query::new(&db);

        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);

        let result = query
            .is_global_stale(created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when no metadata exists");
    }

    #[test]
    fn is_global_stale_returns_false_for_fresh_config() {
        let (db, _temp) = setup_db();

        // Store global config and metadata
        let global = crate::config::global::Global::default();
        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(created, modified);

        let version_key = global.version().value().to_string();
        let metadata_key = format!("global:{}", global.version().value());

        db.put(GLOBAL_CONFIG, &version_key, &global)
            .expect("global write should succeed");
        db.put(CONFIG_METADATA, &metadata_key, &metadata)
            .expect("metadata write should succeed");

        // Check staleness with same timestamps
        let query = Query::new(&db);
        let result = query
            .is_global_stale(created, modified)
            .expect("staleness check should succeed");

        assert!(!result, "should be fresh when timestamps match");
    }

    #[test]
    fn is_global_stale_returns_true_for_created_at_mismatch() {
        let (db, _temp) = setup_db();

        // Store global config and metadata
        let global = crate::config::global::Global::default();
        let stored_created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(stored_created, modified);

        let version_key = global.version().value().to_string();
        let metadata_key = format!("global:{}", global.version().value());

        db.put(GLOBAL_CONFIG, &version_key, &global)
            .expect("global write should succeed");
        db.put(CONFIG_METADATA, &metadata_key, &metadata)
            .expect("metadata write should succeed");

        // Check staleness with different created_at
        let file_created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(999));
        let query = Query::new(&db);
        let result = query
            .is_global_stale(file_created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when created_at differs");
    }

    #[test]
    fn is_global_stale_returns_true_for_newer_modified_at() {
        let (db, _temp) = setup_db();

        // Store global config and metadata
        let global = crate::config::global::Global::default();
        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let stored_modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(created, stored_modified);

        let version_key = global.version().value().to_string();
        let metadata_key = format!("global:{}", global.version().value());

        db.put(GLOBAL_CONFIG, &version_key, &global)
            .expect("global write should succeed");
        db.put(CONFIG_METADATA, &metadata_key, &metadata)
            .expect("metadata write should succeed");

        // Check staleness with newer modified_at
        let file_modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2001);
        let query = Query::new(&db);
        let result = query
            .is_global_stale(created, file_modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when file has newer modified_at");
    }

    #[test]
    fn is_vault_stale_returns_true_when_metadata_missing() {
        let (db, _temp) = setup_db();
        let query = Query::new(&db);

        let vault_id = VaultId::new();
        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);

        let result = query
            .is_vault_stale(vault_id, created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when no metadata exists");
    }

    #[test]
    fn is_vault_stale_returns_false_for_fresh_config() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let vault = crate::config::vault::Vault::default();
        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(created, modified);

        let vault_key = format!("{}:{}", vault_id, vault.version().value());

        db.put(VAULT_CONFIG, &vault_key, &vault)
            .expect("vault write should succeed");
        db.put(CONFIG_METADATA, &vault_key, &metadata)
            .expect("metadata write should succeed");

        let query = Query::new(&db);
        let result = query
            .is_vault_stale(vault_id, created, modified)
            .expect("staleness check should succeed");

        assert!(!result, "should be fresh when timestamps match");
    }

    #[test]
    fn is_vault_stale_returns_true_for_created_at_mismatch() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let vault = crate::config::vault::Vault::default();
        let stored_created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(stored_created, modified);

        let vault_key = format!("{}:{}", vault_id, vault.version().value());

        db.put(VAULT_CONFIG, &vault_key, &vault)
            .expect("vault write should succeed");
        db.put(CONFIG_METADATA, &vault_key, &metadata)
            .expect("metadata write should succeed");

        let file_created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(999));
        let query = Query::new(&db);
        let result = query
            .is_vault_stale(vault_id, file_created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when created_at differs");
    }

    #[test]
    fn is_vault_stale_returns_true_for_newer_modified_at() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let vault = crate::config::vault::Vault::default();
        let created =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000));
        let stored_modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);
        let metadata = ConfigMetadata::new(created, stored_modified);

        let vault_key = format!("{}:{}", vault_id, vault.version().value());

        db.put(VAULT_CONFIG, &vault_key, &vault)
            .expect("vault write should succeed");
        db.put(CONFIG_METADATA, &vault_key, &metadata)
            .expect("metadata write should succeed");

        let file_modified =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2001);
        let query = Query::new(&db);
        let result = query
            .is_vault_stale(vault_id, created, file_modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when file has newer modified_at");
    }

    #[test]
    fn find_vault_id_by_path_returns_none_when_not_found() {
        let (db, _temp) = setup_db();
        let query = Query::new(&db);

        let vault_root =
            VaultRoot::try_new("/test/vault".into()).expect("valid vault root");

        let result = query
            .find_vault_id_by_path(&vault_root)
            .expect("lookup should succeed");

        assert_eq!(result, None, "should return None when path not registered");
    }

    #[test]
    fn find_vault_id_by_path_returns_id_when_found() {
        let (db, _temp) = setup_db();

        let vault_root =
            VaultRoot::try_new("/test/vault".into()).expect("valid vault root");
        let vault_id = VaultId::new();

        // Register the vault path mapping
        let command = CommandAdapter::new(&db);
        command
            .record_vault_path_mapping(vault_id, &vault_root)
            .expect("path mapping should succeed");

        // Look it up
        let query = Query::new(&db);
        let result = query
            .find_vault_id_by_path(&vault_root)
            .expect("lookup should succeed");

        assert_eq!(
            result,
            Some(vault_id),
            "should return the correct vault ID"
        );
    }
}
