//! Redb-backed implementation of the [`crate::config::ports::Query`] trait.

use tracing::instrument;

use super::{merged_version_key, stored::ConfigMetadata};
use crate::{
    config::{
        aggregate::{Config, Timestamp, Version},
        db_table::{
            CONFIG, CONFIG_METADATA, MERGED_CONFIG_ACTIVE,
            MERGED_CONFIG_VERSIONS, VAULT_ID_BY_PATH,
        },
        global::Global,
        ports::Query,
        vault::{Vault, VaultId, VaultRoot},
    },
    db::{Database, DbError},
};

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
    type Error = DbError;

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(
            operation = "find_merged",
            vault_id = %vault_id,
            version = %version
        )
    )]
    fn find_merged(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, Self::Error> {
        let key = merged_version_key(vault_id, version);
        self.db.get_owned(MERGED_CONFIG_VERSIONS, &key)
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
        self.db.get_owned(MERGED_CONFIG_ACTIVE, &vault_id.to_string())
    }

    #[inline]
    #[instrument(skip(self), level = "debug", fields(operation = "get_global"))]
    fn get_global(&self) -> Result<Option<Global>, Self::Error> {
        self.db.get_owned(CONFIG, "global")
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
        self.db.get_owned(CONFIG, &vault_id.to_string())
    }

    #[inline]
    #[instrument(
        skip(self),
        level = "debug",
        fields(operation = "is_global_stale")
    )]
    fn is_global_stale(
        &self,
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, Self::Error> {
        let Some(stored) =
            self.db.get_owned::<ConfigMetadata>(CONFIG_METADATA, "global")?
        else {
            // No metadata stored → config is stale
            return Ok(true);
        };

        // Check if file was replaced (created_at differs)
        if let (Some(file_created), Some(stored_created)) =
            (created_at, stored.created_at)
            && file_created.as_secs() != stored_created.as_secs()
        {
            return Ok(true); // Stale: created_at mismatch
        }

        // Check if file was modified (modified_at is newer)
        if stored.modified_at.as_secs() < modified_at.as_secs() {
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
        created_at: Option<Timestamp>,
        modified_at: Timestamp,
    ) -> Result<bool, Self::Error> {
        let key = vault_id.to_string();
        let Some(stored) =
            self.db.get_owned::<ConfigMetadata>(CONFIG_METADATA, &key)?
        else {
            // No metadata stored → config is stale
            return Ok(true);
        };

        // Check if file was replaced (created_at differs)
        if let (Some(file_created), Some(stored_created)) =
            (created_at, stored.created_at)
            && file_created.as_secs() != stored_created.as_secs()
        {
            return Ok(true); // Stale: created_at mismatch
        }

        // Check if file was modified (modified_at is newer)
        if stored.modified_at.as_secs() < modified_at.as_secs() {
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
        let key = merged_version_key(vault_id, version);
        self.db.get::<Config, _, _>(MERGED_CONFIG_VERSIONS, &key, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        adapter::{command::CommandAdapter, stored::ConfigMetadata},
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
        let query = QueryAdapter::new(&db);

        let created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);

        let result = query
            .is_global_stale(created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when no metadata exists");
    }

    #[test]
    fn is_global_stale_returns_false_for_fresh_config() {
        let (db, _temp) = setup_db();

        // Store metadata
        let created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(created, modified);

        db.put(CONFIG_METADATA, "global", &metadata)
            .expect("metadata write should succeed");

        // Check staleness with same timestamps
        let query = QueryAdapter::new(&db);
        let result = query
            .is_global_stale(created, modified)
            .expect("staleness check should succeed");

        assert!(!result, "should be fresh when timestamps match");
    }

    #[test]
    fn is_global_stale_returns_true_for_created_at_mismatch() {
        let (db, _temp) = setup_db();

        // Store metadata
        let stored_created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(stored_created, modified);

        db.put(CONFIG_METADATA, "global", &metadata)
            .expect("metadata write should succeed");

        // Check staleness with different created_at
        let file_created = Some(Timestamp::from_secs(999));
        let query = QueryAdapter::new(&db);
        let result = query
            .is_global_stale(file_created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when created_at differs");
    }

    #[test]
    fn is_global_stale_returns_true_for_newer_modified_at() {
        let (db, _temp) = setup_db();

        // Store metadata
        let created = Some(Timestamp::from_secs(1000));
        let stored_modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(created, stored_modified);

        db.put(CONFIG_METADATA, "global", &metadata)
            .expect("metadata write should succeed");

        // Check staleness with newer modified_at
        let file_modified = Timestamp::from_secs(2001);
        let query = QueryAdapter::new(&db);
        let result = query
            .is_global_stale(created, file_modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when file has newer modified_at");
    }

    #[test]
    fn is_vault_stale_returns_true_when_metadata_missing() {
        let (db, _temp) = setup_db();
        let query = QueryAdapter::new(&db);

        let vault_id = VaultId::new();
        let created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);

        let result = query
            .is_vault_stale(vault_id, created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when no metadata exists");
    }

    #[test]
    fn is_vault_stale_returns_false_for_fresh_config() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(created, modified);

        db.put(CONFIG_METADATA, &vault_id.to_string(), &metadata)
            .expect("metadata write should succeed");

        let query = QueryAdapter::new(&db);
        let result = query
            .is_vault_stale(vault_id, created, modified)
            .expect("staleness check should succeed");

        assert!(!result, "should be fresh when timestamps match");
    }

    #[test]
    fn is_vault_stale_returns_true_for_created_at_mismatch() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let stored_created = Some(Timestamp::from_secs(1000));
        let modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(stored_created, modified);

        db.put(CONFIG_METADATA, &vault_id.to_string(), &metadata)
            .expect("metadata write should succeed");

        let file_created = Some(Timestamp::from_secs(999));
        let query = QueryAdapter::new(&db);
        let result = query
            .is_vault_stale(vault_id, file_created, modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when created_at differs");
    }

    #[test]
    fn is_vault_stale_returns_true_for_newer_modified_at() {
        let (db, _temp) = setup_db();

        let vault_id = VaultId::new();
        let created = Some(Timestamp::from_secs(1000));
        let stored_modified = Timestamp::from_secs(2000);
        let metadata = ConfigMetadata::new(created, stored_modified);

        db.put(CONFIG_METADATA, &vault_id.to_string(), &metadata)
            .expect("metadata write should succeed");

        let file_modified = Timestamp::from_secs(2001);
        let query = QueryAdapter::new(&db);
        let result = query
            .is_vault_stale(vault_id, created, file_modified)
            .expect("staleness check should succeed");

        assert!(result, "should be stale when file has newer modified_at");
    }

    #[test]
    fn find_vault_id_by_path_returns_none_when_not_found() {
        let (db, _temp) = setup_db();
        let query = QueryAdapter::new(&db);

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
        let query = QueryAdapter::new(&db);
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
