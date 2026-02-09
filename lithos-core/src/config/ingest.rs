//! Figment-based configuration ingestion (adapter boundary).
//!
//! This module loads Raw configuration types from external sources and keeps
//! Figment out of the domain modules.

use std::path::{Path, PathBuf};

use figment::{
    Figment,
    providers::{Format as _, Serialized, Toml},
};

use super::{error::ConfigIngestError, raw::RawConfig};

fn global_config_path_from_env() -> Option<PathBuf> {
    std::env::var_os("LITHOS_GLOBAL_CONFIG").map(PathBuf::from)
}

/// Build merged raw config from global and vault sources using Figment.
///
/// This implements the configuration hierarchy:
/// 1. Compiled defaults (lowest priority)
/// 2. Global config file (~/.config/lithos/lithos.toml)
/// 3. Vault config file (<vault>/.lithos/lithos.toml) (highest priority)
///
/// # Errors
/// Returns `ConfigIngestError` if file reading, parsing, or extraction fails.
#[inline]
pub fn build_merged_raw(
    vault_root: &Path,
) -> Result<RawConfig, ConfigIngestError> {
    // Layer 1: Compiled defaults
    let mut figment = Figment::from(Serialized::defaults(RawConfig::default()));

    // Layer 2: Global config (if exists)
    if let Some(path) = global_config_path_from_env()
        && path.exists()
    {
        figment = figment.merge(Toml::file(&path));
    }

    // Layer 3: Vault config (if exists)
    let vault_config_path = vault_root.join(".lithos").join("lithos.toml");
    if vault_config_path.exists() {
        figment = figment.merge(Toml::file(&vault_config_path));
    }

    // Extract merged config
    figment.extract().map_err(ConfigIngestError::from)
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability"
)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    mod fixtures {
        use std::{fs, path::PathBuf};

        use tempfile::TempDir;

        pub fn setup_vault_with_config(
            content: &str,
        ) -> Result<(TempDir, PathBuf), std::io::Error> {
            let dir = tempfile::tempdir()?;
            let config_dir = dir.path().join(".lithos");
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("lithos.toml");
            fs::write(&config_path, content)?;
            Ok((dir, config_path))
        }
    }

    mod load {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn build_merged_raw_uses_defaults_when_file_missing() {
            let dir = tempdir().expect("tempdir");
            let result = build_merged_raw(dir.path());
            assert!(result.is_ok(), "Expected default ingest to succeed");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn build_merged_raw_reads_lithos_toml_when_present() {
            let (dir, _path) = fixtures::setup_vault_with_config(
                "[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup vault");

            let raw = build_merged_raw(dir.path()).expect("build merged raw");
            let logging = raw.logging.expect("logging section missing");
            assert_eq!(logging.log_level.as_deref(), Some("debug"));
        }
    }
}
