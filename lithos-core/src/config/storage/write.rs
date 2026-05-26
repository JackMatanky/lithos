//! Write operations for configuration.
//!
//! Implements the [`WriteRepository`] trait against redb storage. Each write
//! method opens a single write transaction and operates on the required tables
//! within that transaction.

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        CONFIG_VERSIONS, GLOBAL_CONFIG, RAW_GLOBAL_CONFIG_VIEW,
        RAW_VAULT_CONFIG_VIEW, VAULT_CONFIG, VAULT_ID_BY_PATH,
        VAULT_PATH_BY_ID,
    },
};
use crate::{
    config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        repository::WriteRepository,
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    db::ArchivedEntity,
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_global(
        &self,
        config: &Global,
    ) -> Result<(), ConfigRepositoryError> {
        let version_key = config.version().value().to_string();
        let bytes = config.to_bytes()?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(GLOBAL_CONFIG.definition())?;
                table.insert(version_key.as_str(), bytes.as_slice())?;
                Ok(())
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn save_vault(
        &self,
        vault_id: VaultId,
        config: &Vault,
    ) -> Result<(), ConfigRepositoryError> {
        let version_key = format!("{}:{}", vault_id, config.version().value());
        let bytes = config.to_bytes()?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(VAULT_CONFIG.definition())?;
                table.insert(version_key.as_str(), bytes.as_slice())?;
                Ok(())
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn save_config(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, ConfigRepositoryError> {
        // Atomically allocate version and save config
        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(CONFIG_VERSIONS.definition())?;

                let prefix = format!("{vault_id}:");

                // Scan for existing versions to allocate next
                let max_version = table
                    .range(prefix.as_str()..)?
                    .filter_map(Result::ok)
                    .filter_map(|(k, _)| {
                        let key = k.value();
                        if key.starts_with(&prefix) {
                            key.strip_prefix(&prefix)
                                .and_then(|v| v.parse::<u64>().ok())
                        } else {
                            None
                        }
                    })
                    .max();

                let next = if let Some(max) = max_version {
                    Version::try_from(max)
                        .map_err(|e| {
                            crate::db::DbError::Deserialization(e.to_string())
                        })?
                        .next()
                        .map_err(|e| {
                            crate::db::DbError::Serialization(e.to_string())
                        })?
                } else {
                    Version::initial()
                };

                let versioned_config = config.clone().with_version(next);
                let bytes = versioned_config.to_bytes()?;
                let key = format!("{vault_id}:{}", next.value());

                table.insert(key.as_str(), bytes.as_slice())?;

                Ok(next)
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn save_vault_path_mapping(
        &self,
        vault_id: VaultId,
        vault_root: &VaultRoot,
    ) -> Result<(), ConfigRepositoryError> {
        let path_key = vault_root.as_key();
        let id_bytes = vault_id.to_bytes()?;
        let root_bytes = vault_root.to_bytes()?;

        self.store
            .write(|tx| {
                let mut id_by_path =
                    tx.inner.open_table(VAULT_ID_BY_PATH.definition())?;
                let mut path_by_id =
                    tx.inner.open_table(VAULT_PATH_BY_ID.definition())?;

                id_by_path.insert(path_key, id_bytes.as_slice())?;
                path_by_id.insert(&vault_id, root_bytes.as_slice())?;
                Ok(())
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn save_raw_global_view(
        &self,
        view: &RawGlobalConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        let bytes = view.to_bytes()?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(RAW_GLOBAL_CONFIG_VIEW.definition())?;
                table.insert("global", bytes.as_slice())?;
                Ok(())
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn save_raw_vault_view(
        &self,
        vault_id: VaultId,
        view: &RawVaultConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        let bytes = view.to_bytes()?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(RAW_VAULT_CONFIG_VIEW.definition())?;
                table.insert(&vault_id, bytes.as_slice())?;
                Ok(())
            })
            .map_err(ConfigRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        config::{
            aggregate::fixtures as config_fixtures, repository::ReadRepository,
        },
        db::Store,
    };

    fn temp_repo() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        (tempdir, repo)
    }

    #[test]
    fn save_global_persists_config() {
        let (_temp, repo) = temp_repo();
        let global = Global::default();

        repo.save_global(&global).unwrap();

        let retrieved = repo.get_global().unwrap().unwrap();
        assert_eq!(retrieved.version().value(), global.version().value());
    }

    #[test]
    fn save_vault_persists_config() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let vault = Vault::default();

        repo.save_vault(vault_id, &vault).unwrap();

        let retrieved = repo.get_vault(vault_id).unwrap().unwrap();
        assert_eq!(retrieved.version().value(), vault.version().value());
    }

    #[test]
    fn save_config_allocates_version() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let config = config_fixtures::test_config();

        let v1 = repo.save_config(vault_id, &config).unwrap();
        assert_eq!(v1.value(), 1);

        let v2 = repo.save_config(vault_id, &config).unwrap();
        assert_eq!(v2.value(), 2);

        let retrieved = repo.get_config(vault_id, v2).unwrap().unwrap();
        assert_eq!(retrieved.version().value(), 2);
    }

    #[test]
    fn save_vault_path_mapping_is_bidirectional() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let root = config_fixtures::vault_root("/test");

        repo.save_vault_path_mapping(vault_id, &root).unwrap();

        let found_id = repo.find_vault_id_by_path(&root).unwrap().unwrap();
        assert_eq!(found_id, vault_id);

        // Check reverse mapping manually
        let found_root: VaultRoot = repo
            .store
            .read(|tx| {
                let table =
                    tx.try_open_table(VAULT_PATH_BY_ID.definition())?.unwrap();
                Ok(table
                    .get(&vault_id)?
                    .map(|g| VaultRoot::from_bytes(g.value()))
                    .transpose())
            })
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(found_root.as_path(), root.as_path());
    }
}
