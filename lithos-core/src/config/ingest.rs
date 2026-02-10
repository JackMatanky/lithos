//! Figment-based configuration ingestion.
//!
//! This module handles the loading and merging of raw configuration data
//! from external files and environment variables into [`RawConfig`].

use std::path::{Path, PathBuf};

use figment::{
    Figment,
    providers::{Format as _, Serialized, Toml},
};

use super::{error::ConfigIngestError, raw::RawConfig};

/// Builds a merged raw configuration from global and vault sources using
/// Figment.
///
/// This implements the configuration hierarchy by layering files on top of
/// default values. The resulting [`RawConfig`] is an intermediate state
/// that must be validated and transformed into a [`Config`] aggregate.
///
/// # Errors
/// Returns [`ConfigIngestError`] if file reading, TOML parsing, or data
/// extraction fails.
#[inline]
pub fn build_merged_raw(
    vault_root: &Path,
) -> Result<RawConfig, ConfigIngestError> {
    build_merged_raw_impl(vault_root, global_config_path_from_env().as_deref())
}

/// Internal implementation that accepts an optional global config path.
/// Exposed for testing.
#[inline]
fn build_merged_raw_impl(
    vault_root: &Path,
    global_config_path: Option<&Path>,
) -> Result<RawConfig, ConfigIngestError> {
    // Layer 1: Compiled defaults
    let mut figment = Figment::from(Serialized::defaults(RawConfig::default()));

    // Layer 2: Global config (if exists)
    if let Some(path) = global_config_path
        && path.exists()
    {
        figment = figment.merge(Toml::file(path));
    }

    // Layer 3: Vault config (if exists)
    let vault_config_path = vault_root.join(".lithos").join("lithos.toml");
    if vault_config_path.exists() {
        figment = figment.merge(Toml::file(&vault_config_path));
    }

    // Extract merged config
    figment.extract().map_err(ConfigIngestError::from)
}

fn global_config_path_from_env() -> Option<PathBuf> {
    // Placeholder for environment variable support.
    // TODO: Implement via `figment::providers::Env` with LITHOS_ prefix.
    // Reserved for future use.
    // Example implementation:
    // std::env::var_os("LITHOS_GLOBAL_CONFIG").map(PathBuf::from)
    None
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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

        /// Create a temporary directory with global and vault configs.
        /// Returns (`global_dir`, `global_config_path`, `vault_dir`).
        pub fn setup_layered_configs(
            global_content: &str,
            vault_content: &str,
        ) -> Result<(TempDir, PathBuf, TempDir), std::io::Error> {
            let vault_dir = tempfile::tempdir()?;
            let global_dir = tempfile::tempdir()?;

            // Write global config
            let global_config_path = global_dir.path().join("lithos.toml");
            fs::write(&global_config_path, global_content)?;

            // Write vault config
            let vault_config_dir = vault_dir.path().join(".lithos");
            fs::create_dir_all(&vault_config_dir)?;
            let vault_config_path = vault_config_dir.join("lithos.toml");
            fs::write(&vault_config_path, vault_content)?;

            Ok((global_dir, global_config_path, vault_dir))
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

    mod layering {
        use fixtures::setup_layered_configs;

        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn vault_overrides_global() {
            // GIVEN: global config with logging = info, vault config with debug
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[logging]\nlog_level = \"info\"\n",
                "[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup configs");

            // WHEN: building merged config
            let raw =
                build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                    .expect("build merged raw");

            // THEN: vault value wins
            let logging = raw.logging.expect("logging should be Some");
            assert_eq!(
                logging.log_level.as_deref(),
                Some("debug"),
                "Vault config should override global"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn global_used_when_vault_missing() {
            // GIVEN: only global config
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[logging]\nlog_level = \"warn\"\n",
                "", // empty vault config
            )
            .expect("setup configs");

            // WHEN: building merged config
            let raw =
                build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                    .expect("build merged raw");

            // THEN: global value is used
            let logging = raw.logging.expect("logging should be Some");
            assert_eq!(
                logging.log_level.as_deref(),
                Some("warn"),
                "Global config should be used when vault missing"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn defaults_used_when_both_missing() {
            // GIVEN: no configs at all
            let dir = tempdir().expect("tempdir");
            let vault_path = dir.path();

            // WHEN: building merged config with no global
            let raw = build_merged_raw_impl(vault_path, None)
                .expect("build merged raw");

            // THEN: defaults apply
            assert!(
                raw.logging.is_none(),
                "Logging should be None when not specified"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn paths_fields_merge_correctly() {
            // GIVEN: global with schemas_dir, vault with cache_dir
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[paths]\nschemas_dir = \"global-schemas\"\n",
                "[paths]\ncache_dir = \".cache\"\n",
            )
            .expect("setup configs");

            // WHEN: building merged config
            let raw =
                build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                    .expect("build merged raw");

            // THEN: both fields present (deep merge)
            let fs = &raw.paths;
            assert_eq!(
                fs.schemas_dir.as_deref(),
                Some("global-schemas"),
                "Global schemas_dir should be preserved"
            );
            assert_eq!(
                fs.cache_dir.as_deref(),
                Some(".cache"),
                "Vault cache_dir should be added"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        fn vault_overrides_global_paths_field() {
            // GIVEN: global with templates_dir = global-templates, vault with
            // vault-templates
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[paths]\ntemplates_dir = \"global-templates\"\n",
                "[paths]\ntemplates_dir = \"vault-templates\"\n",
            )
            .expect("setup configs");

            // WHEN: building merged config
            let raw =
                build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                    .expect("build merged raw");

            // THEN: vault value wins
            assert_eq!(
                raw.paths.templates_dir.as_deref(),
                Some("vault-templates"),
                "Vault paths field should override global"
            );
        }
    }
}
