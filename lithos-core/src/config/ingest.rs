//! Figment-based configuration ingestion (adapter boundary).
//!
//! This module loads Raw configuration types from external sources and keeps
//! Figment out of the domain modules.

use std::path::{Path, PathBuf};

use figment::{
    Figment,
    providers::{Format as _, Serialized, Toml},
};

use super::{
    error::ConfigIngestError,
    raw::{RawGlobal, RawVault},
};

fn global_config_path_from_env() -> Option<PathBuf> {
    std::env::var_os("LITHOS_GLOBAL_CONFIG").map(PathBuf::from)
}

/// Ingest global configuration using Figment providers.
///
/// If `LITHOS_GLOBAL_CONFIG` is set and the file exists, it is merged on top
/// of defaults.
///
/// # Errors
/// Returns `ConfigIngestError` if Figment extraction fails.
#[inline]
pub fn ingest_global() -> Result<RawGlobal, ConfigIngestError> {
    ingest_global_with_path(global_config_path_from_env())
}

fn ingest_global_with_path(
    path: Option<PathBuf>,
) -> Result<RawGlobal, ConfigIngestError> {
    let mut figment = Figment::from(Serialized::defaults(RawGlobal::default()));

    if let Some(path) = path
        && path.exists()
    {
        figment = figment.merge(Toml::file(path));
    }

    figment.extract().map_err(ConfigIngestError::from)
}

/// Ingest vault configuration using Figment providers.
///
/// Looks for `.lithos/lithos.toml` under the provided vault root.
///
/// # Errors
/// Returns `ConfigIngestError` if Figment extraction fails.
#[inline]
pub fn ingest_vault(vault_root: &Path) -> Result<RawVault, ConfigIngestError> {
    let mut figment = Figment::from(Serialized::defaults(RawVault::default()));
    let vault_config_path = vault_root.join(".lithos").join("lithos.toml");

    if vault_config_path.exists() {
        figment = figment.merge(Toml::file(vault_config_path));
    }

    figment.extract().map_err(ConfigIngestError::from)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn ingest_global_returns_defaults_when_env_unset() {
        let result = ingest_global_with_path(None);
        assert!(result.is_ok(), "Expected default ingest to succeed");
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert! which can panic."
    )]
    fn ingest_vault_uses_defaults_when_file_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let result = ingest_vault(dir.path());
        assert!(result.is_ok(), "Expected default ingest to succeed");
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert! which can panic."
    )]
    fn ingest_vault_reads_lithos_toml_when_present()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let config_dir = dir.path().join(".lithos");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("lithos.toml");

        fs::write(&config_path, "[logging]\nlog_level = \"debug\"\n")?;

        let raw = ingest_vault(dir.path())?;
        let logging = raw.logging.ok_or("logging section missing")?;
        assert_eq!(logging.log_level.as_deref(), Some("debug"));
        Ok(())
    }
}
