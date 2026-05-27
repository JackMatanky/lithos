//! Read-only repository operations for configuration.
//!
//! Implements the [`ReadRepository`] trait against redb storage. Each method
//! opens a read transaction and uses
//! [`ReadTx::try_open_table`](crate::db::ReadTx::try_open_table)
//! to handle uninitialized tables gracefully.

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        CONFIG_VERSIONS, GLOBAL_CONFIG, RAW_GLOBAL_CONFIG_VIEW,
        RAW_VAULT_CONFIG_VIEW, VAULT_CONFIG, VAULT_ID_BY_PATH,
    },
};
use crate::{
    config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        repository::ReadRepository,
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    db::ArchivedEntity,
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn get_global(&self) -> Result<Option<Global>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(GLOBAL_CONFIG.definition())?
                else {
                    return Ok(None);
                };

                // Find max version by scanning
                let max_version = table
                    .iter()?
                    .filter_map(Result::ok)
                    .filter_map(|(k, _)| k.value().parse::<u64>().ok())
                    .max();

                let Some(max) = max_version else {
                    return Ok(None);
                };

                table
                    .get(max.to_string().as_str())?
                    .map(|g| Global::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(VAULT_CONFIG.definition())?
                else {
                    return Ok(None);
                };

                let prefix = format!("{vault_id}:");

                // Scan for keys with vault_id prefix
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

                let Some(max) = max_version else {
                    return Ok(None);
                };

                let key = format!("{prefix}{max}");
                table
                    .get(key.as_str())?
                    .map(|g| Vault::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<Config>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(CONFIG_VERSIONS.definition())?
                else {
                    return Ok(None);
                };

                let key = format!("{}:{}", vault_id, version.value());
                table
                    .get(key.as_str())?
                    .map(|g| Config::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_active_version(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<Version>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(CONFIG_VERSIONS.definition())?
                else {
                    return Ok(None);
                };

                let prefix = format!("{vault_id}:");
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

                let Some(max) = max_version else {
                    return Ok(None);
                };

                Version::try_from(max).map(Some).map_err(|e| {
                    crate::db::DbError::Deserialization(e.to_string())
                })
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn with_archived_config<R, F>(
        &self,
        vault_id: VaultId,
        version: Version,
        f: F,
    ) -> Result<Option<R>, ConfigRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(CONFIG_VERSIONS.definition())?
                else {
                    return Ok(None);
                };

                let key = format!("{}:{}", vault_id, version.value());
                table
                    .get(key.as_str())?
                    .map(|g| Config::with_archived(g.value(), f))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn find_vault_id_by_path(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(VAULT_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };

                table
                    .get(vault_root.as_key().as_str())?
                    .map(|g| VaultId::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(RAW_GLOBAL_CONFIG_VIEW.definition())?
                else {
                    return Ok(None);
                };

                table
                    .get("global")?
                    .map(|g| RawGlobalConfigView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_raw_vault_view(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, ConfigRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(RAW_VAULT_CONFIG_VIEW.definition())?
                else {
                    return Ok(None);
                };

                table
                    .get(&vault_id)?
                    .map(|g| RawVaultConfigView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        config::aggregate::fixtures as config_fixtures,
        db::{ArchivedEntity, Store},
    };

    fn temp_repo() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        (tempdir, repo)
    }

    #[test]
    fn get_global_returns_none_when_empty() {
        let (_temp, repo) = temp_repo();
        let result = repo.get_global().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_global_returns_latest_version() {
        let (_temp, repo) = temp_repo();
        let global1 = Global::default();
        let global2 = Global::new(
            global1.version().next().unwrap(),
            global1.logging().clone(),
            global1.paths().clone(),
            global1.trusted_vaults().cloned(),
            global1.frontmatter().clone(),
            global1.task().cloned(),
        );

        let bytes1 = global1.to_bytes().unwrap();
        let bytes2 = global2.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(GLOBAL_CONFIG.definition())?;
                table.insert("1", bytes1.as_slice())?;
                table.insert("2", bytes2.as_slice())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_global().unwrap().unwrap();
        assert_eq!(retrieved.version().value(), 2);
    }

    #[test]
    fn get_vault_returns_latest_for_id() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let vault = Vault::default();
        let bytes = vault.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(VAULT_CONFIG.definition())?;
                let key = format!("{vault_id}:1");
                table.insert(key.as_str(), bytes.as_slice())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_vault(vault_id).unwrap().unwrap();
        assert_eq!(retrieved.version().value(), 1);
    }

    #[test]
    fn get_config_lookup_works() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let config = config_fixtures::test_config();
        let bytes = config.to_bytes().unwrap();
        let version = config.version();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(CONFIG_VERSIONS.definition())?;
                let key = format!("{vault_id}:{}", version.value());
                table.insert(key.as_str(), bytes.as_slice())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_config(vault_id, *version).unwrap().unwrap();
        assert_eq!(retrieved.version().value(), version.value());
    }

    #[test]
    fn find_vault_id_by_path_works() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let root = config_fixtures::vault_root("/test");
        let id_bytes = vault_id.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(VAULT_ID_BY_PATH.definition())?;
                table.insert(root.as_key().as_str(), id_bytes.as_slice())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.find_vault_id_by_path(&root).unwrap().unwrap();
        assert_eq!(retrieved, vault_id);
    }
}
