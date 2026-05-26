//! Configuration storage persistence implementation.

pub mod tables;

use std::sync::Arc;

use crate::{
    config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        repository::{ReadRepository, Repository, WriteRepository},
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    db::Store,
};

/// Repository implementation for `redb`-backed configuration storage.
#[derive(Debug, Clone)]
pub struct RedbRepository {
    pub(crate) store: Arc<Store>,
}

impl RedbRepository {
    /// Creates a new repository adapter from a database store.
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

impl ReadRepository for RedbRepository {
    fn get_global(&self) -> Result<Option<Global>, ConfigRepositoryError> {
        Ok(None)
    }

    fn get_vault(
        &self,
        _vault_id: VaultId,
    ) -> Result<Option<Vault>, ConfigRepositoryError> {
        Ok(None)
    }

    fn get_config(
        &self,
        _vault_id: VaultId,
        _version: Version,
    ) -> Result<Option<Config>, ConfigRepositoryError> {
        Ok(None)
    }

    fn get_active_version(
        &self,
        _vault_id: VaultId,
    ) -> Result<Option<Version>, ConfigRepositoryError> {
        Ok(None)
    }

    fn with_archived_config<R, F>(
        &self,
        _vault_id: VaultId,
        _version: Version,
        _f: F,
    ) -> Result<Option<R>, ConfigRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<Config>) -> R,
    {
        Ok(None)
    }

    fn find_vault_id_by_path(
        &self,
        _vault_root: &VaultRoot,
    ) -> Result<Option<VaultId>, ConfigRepositoryError> {
        Ok(None)
    }

    fn get_raw_global_view(
        &self,
    ) -> Result<Option<RawGlobalConfigView>, ConfigRepositoryError> {
        Ok(None)
    }

    fn get_raw_vault_view(
        &self,
        _vault_id: VaultId,
    ) -> Result<Option<RawVaultConfigView>, ConfigRepositoryError> {
        Ok(None)
    }
}

impl WriteRepository for RedbRepository {
    fn save_global(
        &self,
        _config: &Global,
    ) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }

    fn save_vault(
        &self,
        _vault_id: VaultId,
        _config: &Vault,
    ) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }

    fn save_config(
        &self,
        _vault_id: VaultId,
        _config: &Config,
    ) -> Result<Version, ConfigRepositoryError> {
        Ok(Version::initial())
    }

    fn save_vault_path_mapping(
        &self,
        _vault_id: VaultId,
        _vault_root: &VaultRoot,
    ) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }

    fn save_raw_global_view(
        &self,
        _view: &RawGlobalConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }

    fn save_raw_vault_view(
        &self,
        _vault_id: VaultId,
        _view: &RawVaultConfigView,
    ) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }
}

mod internal {
    use super::tables;
    use crate::config::{
        aggregate::{Config, Version},
        error::ConfigRepositoryError,
        global::Global,
        vault::{Vault, VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    };

    #[expect(dead_code, reason = "Placeholder for Phase 2")]
    fn silence_errors(
        db: &crate::db::Database,
        tx: &mut crate::db::Writer<'_>,
    ) -> Result<(), ConfigRepositoryError> {
        let _ = db.scan_range::<Global>(tables::GLOBAL_CONFIG.definition(), "");
        let _ = db.get_owned(tables::GLOBAL_CONFIG.definition(), "");
        let _ =
            tx.put(tables::GLOBAL_CONFIG.definition(), "", &Global::default());

        let prefix = "";
        let _ =
            db.scan_range::<Vault>(tables::VAULT_CONFIG.definition(), prefix);
        let _ = db.get_owned(tables::VAULT_CONFIG.definition(), "");
        let _ =
            tx.put(tables::VAULT_CONFIG.definition(), "", &Vault::default());

        let _ = db.get_owned(tables::CONFIG_VERSIONS.definition(), "");
        let _ = db
            .scan_range::<Config>(tables::CONFIG_VERSIONS.definition(), prefix);

        let _ =
            db.get_owned::<Vec<u8>>(tables::VAULT_ID_BY_PATH.definition(), "");
        let _ = tx.put(
            tables::VAULT_ID_BY_PATH.definition(),
            &"".to_string(),
            &[0u8; 16],
        );

        let _ = db.get_owned::<RawGlobalConfigView>(
            tables::RAW_GLOBAL_CONFIG_VIEW.definition(),
            "",
        );

        let _ = db.get_owned::<RawVaultConfigView>(
            tables::RAW_VAULT_CONFIG_VIEW.definition(),
            VaultId::new(),
        );

        let res: Result<Version, crate::config::error::ConfigError> =
            Version::initial().next();
        let _ = res.map_err(ConfigRepositoryError::Domain);

        Ok(())
    }
}
