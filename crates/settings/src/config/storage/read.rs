//! Read-only repository operations for configuration.
//!
//! Implements the [`ReadRepository`] trait against redb storage. Each method
//! opens a read transaction and uses
//! [`ReadTx::try_open_table`](traces_db::ReadTx::try_open_table)
//! to handle uninitialized tables gracefully.

#![allow(deprecated, reason = "storage adapter migration pending")]

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        CONFIG_VERSIONS, GLOBAL_CONFIG, RAW_GLOBAL_CONFIG_VIEW,
        RAW_VAULT_CONFIG_VIEW, VAULT_CONFIG, VAULT_ID_BY_PATH,
    },
};
use crate::config::{
    aggregate::{AppConfig, Version},
    error::ConfigRepositoryError,
    global::GlobalConfig,
    repository::ReadRepository,
    vault::{LocalConfig, VaultId, VaultRoot},
    views::{RawGlobalConfigView, RawVaultConfigView},
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn get_global(
        &self,
    ) -> Result<Option<GlobalConfig>, ConfigRepositoryError> {
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
                    .map(|g| {
                        g.value().decode().map_err(traces_db::DbError::from)
                    })
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_vault(
        &self,
        vault_id: VaultId,
    ) -> Result<Option<LocalConfig>, ConfigRepositoryError> {
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
                    .map(|g| {
                        g.value().decode().map_err(traces_db::DbError::from)
                    })
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }

    #[inline]
    fn get_config(
        &self,
        vault_id: VaultId,
        version: Version,
    ) -> Result<Option<AppConfig>, ConfigRepositoryError> {
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
                    .map(|g| {
                        g.value().decode().map_err(traces_db::DbError::from)
                    })
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

                Version::try_from(max)
                    .map(Some)
                    .map_err(|e| traces_db::DbError::Corruption(e.to_string()))
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
        F: for<'archived> FnOnce(&'archived rkyv::Archived<AppConfig>) -> R,
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
                    .map(|g| {
                        g.value()
                            .with_archived(f)
                            .map_err(traces_db::DbError::from)
                    })
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
                    .map(|g| Ok(g.value()))
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
                    .map(|g| {
                        g.value().decode().map_err(traces_db::DbError::from)
                    })
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
                    .map(|g| {
                        g.value().decode().map_err(traces_db::DbError::from)
                    })
                    .transpose()
            })
            .map_err(ConfigRepositoryError::from)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use traces_db::Store;

    use super::*;
    use crate::config::aggregate::fixtures as config_fixtures;

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
        let (_guard, global1) =
            crate::config::global::fixtures::global_config();
        let global2 = GlobalConfig::new(
            global1.version().next().unwrap(),
            global1.path().clone(),
            global1.logging().clone(),
            global1.template().cloned(),
            global1.schema().cloned(),
            global1.trusted_vaults().cloned(),
            global1.frontmatter().clone(),
            global1.task().cloned(),
        );

        let bytes1 = traces_db::RkyvBytes::encode(&global1).unwrap();
        let bytes2 = traces_db::RkyvBytes::encode(&global2).unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(GLOBAL_CONFIG.definition())?;
                table.insert("1", &bytes1)?;
                table.insert("2", &bytes2)?;
                Ok::<_, traces_db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_global().unwrap().unwrap();
        assert_eq!(retrieved.version().value(), 2);
    }

    #[test]
    fn get_vault_returns_latest_for_id() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let (_base, _file, vault) =
            crate::config::vault::fixtures::local_config();
        let bytes = traces_db::RkyvBytes::encode(&vault).unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(VAULT_CONFIG.definition())?;
                let key = format!("{vault_id}:1");
                table.insert(key.as_str(), &bytes)?;
                Ok::<_, traces_db::DbError>(())
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
        let bytes = traces_db::RkyvBytes::encode(&config).unwrap();
        let version = config.version();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(CONFIG_VERSIONS.definition())?;
                let key = format!("{vault_id}:{}", version.value());
                table.insert(key.as_str(), &bytes)?;
                Ok::<_, traces_db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_config(vault_id, *version).unwrap().unwrap();
        assert_eq!(retrieved.version().value(), version.value());
    }

    #[test]
    fn find_vault_id_by_path_works() {
        let (_temp, repo) = temp_repo();
        let vault_id = VaultId::new();
        let root =
            VaultRoot::from_dir_path(config_fixtures::vault_root("/test"));
        let id_bytes = vault_id;

        repo.store
            .write(|tx| {
                let mut table =
                    tx.inner.open_table(VAULT_ID_BY_PATH.definition())?;
                table.insert(root.as_key().as_str(), id_bytes)?;
                Ok::<_, traces_db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.find_vault_id_by_path(&root).unwrap().unwrap();
        assert_eq!(retrieved, vault_id);
    }
}
