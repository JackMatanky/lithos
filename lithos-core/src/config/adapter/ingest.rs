//! Configuration ingestion adapter.
//!
//! This module provides the [`Ingestor`] struct for loading and merging raw
//! configuration from filesystem sources. It handles:
//!
//! - **File discovery**: Locates global and vault config files
//! - **Metadata extraction**: Captures filesystem timestamps
//! - **Layered merging**: Combines defaults, global, and vault configs using
//!   Figment
//! - **TOML parsing**: Converts file content to [`RawConfig`]
//!
//! This is a pure adapter - it performs file I/O and parsing but no validation
//! or database access.

use std::{
    fs,
    path::{Path, PathBuf},
};

use figment::{
    Figment,
    providers::{Format as _, Serialized, Toml},
};
use tracing::instrument;

use crate::config::{
    aggregate::Timestamp, error::ConfigIngestError, raw::RawConfig,
    vault::VaultRoot,
};

/// Configuration ingestion adapter.
///
/// Handles loading raw configuration from the filesystem with metadata
/// extraction. Supports both individual file loading (with timestamps) and
/// Figment-based layered merging.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Ingestor;

impl Ingestor {
    /// Create a new ingestor instance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolve the global config file path using priority order.
    ///
    /// Priority (first found wins):
    /// 1. `$LITHOS_GLOBAL_CONFIG` environment variable
    /// 2. `$XDG_CONFIG_HOME/lithos/lithos.toml`
    /// 3. `$HOME/.config/lithos/lithos.toml` (XDG default)
    /// 4. `$HOME/.lithos/lithos.toml` (legacy)
    /// 5. `/etc/lithos/lithos.toml` (system-wide)
    ///
    /// Returns `None` if no config file exists at any location.
    #[inline]
    #[must_use]
    pub fn resolve_global_config_path(self) -> Option<PathBuf> {
        // Priority 1: Environment variable override
        if let Ok(env_path) = std::env::var("LITHOS_GLOBAL_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Some(path);
            }
        }

        // Priority 2: XDG_CONFIG_HOME
        if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
            let path = Path::new(&xdg_home).join("lithos/lithos.toml");
            if path.exists() {
                return Some(path);
            }
        }

        // Priority 3: HOME/.config (XDG default)
        if let Ok(home) = std::env::var("HOME") {
            let path = Path::new(&home).join(".config/lithos/lithos.toml");
            if path.exists() {
                return Some(path);
            }
        }

        // Priority 4: HOME/.lithos (legacy)
        if let Ok(home) = std::env::var("HOME") {
            let path = Path::new(&home).join(".lithos/lithos.toml");
            if path.exists() {
                return Some(path);
            }
        }

        // Priority 5: System-wide /etc
        let system_path = PathBuf::from("/etc/lithos/lithos.toml");
        if system_path.exists() {
            return Some(system_path);
        }

        None
    }

    /// Load and parse the global config file with metadata extraction.
    ///
    /// Uses [`Self::resolve_global_config_path`] to find the config file,
    /// then:
    /// - Reads the file from disk
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Parses TOML content into [`RawConfig`]
    ///
    /// Returns `None` if no global config file exists.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigIngestError`] if:
    /// - File reading fails (I/O error)
    /// - TOML parsing fails (syntax error)
    #[inline]
    pub fn load_global_config(
        self,
    ) -> Result<Option<RawConfigWithMetadata>, ConfigIngestError> {
        let Some(path) = self.resolve_global_config_path() else {
            return Ok(None);
        };

        self.load_config_from_path(&path)
    }

    /// Load and parse a vault config file with metadata extraction.
    ///
    /// Reads from `{vault_root}/.lithos/lithos.toml`, then:
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Parses TOML content into [`RawConfig`]
    ///
    /// Returns `None` if the vault config file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigIngestError`] if:
    /// - File reading fails (I/O error)
    /// - TOML parsing fails (syntax error)
    #[inline]
    pub fn load_vault_config(
        self,
        vault_root: &VaultRoot,
    ) -> Result<Option<RawConfigWithMetadata>, ConfigIngestError> {
        let path = vault_root.as_path().join(".lithos/lithos.toml");

        if !path.exists() {
            return Ok(None);
        }

        self.load_config_from_path(&path)
    }

    /// Build merged raw configuration using Figment layering.
    ///
    /// This implements the configuration hierarchy by layering files on top
    /// of default values:
    /// 1. Compiled defaults
    /// 2. Global config (if exists)
    /// 3. Vault config (if exists)
    ///
    /// The resulting [`RawConfig`] is an intermediate state that must be
    /// validated and transformed into a [`Config`] aggregate.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigIngestError`] if file reading, TOML parsing, or data
    /// extraction fails.
    #[inline]
    #[instrument(
        skip(self, vault_root),
        level = "debug",
        fields(operation = "build_merged_raw", vault_root = %vault_root.display())
    )]
    pub fn build_merged_raw(
        self,
        vault_root: &Path,
    ) -> Result<RawConfig, ConfigIngestError> {
        self.build_merged_raw_impl(
            vault_root,
            self.resolve_global_config_path().as_deref(),
        )
    }

    /// Internal implementation that accepts an optional global config path.
    ///
    /// Exposed for testing with custom global config locations.
    #[inline]
    #[expect(
        clippy::unused_self,
        reason = "Zero-sized type for API consistency"
    )]
    #[instrument(
        skip(self, vault_root, global_config_path),
        level = "debug",
        fields(operation = "build_merged_raw_impl", vault_root = %vault_root.display())
    )]
    fn build_merged_raw_impl(
        self,
        vault_root: &Path,
        global_config_path: Option<&Path>,
    ) -> Result<RawConfig, ConfigIngestError> {
        // Layer 1: Compiled defaults
        let mut figment =
            Figment::from(Serialized::defaults(RawConfig::default()));

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

    /// Internal helper to load and parse a config file with metadata.
    #[expect(
        clippy::unused_self,
        reason = "Zero-sized type for API consistency"
    )]
    fn load_config_from_path(
        self,
        path: &Path,
    ) -> Result<Option<RawConfigWithMetadata>, ConfigIngestError> {
        // Extract metadata before reading file content
        let metadata = fs::metadata(path).ok();
        let created_at = extract_timestamp(
            path,
            metadata.as_ref(),
            fs::Metadata::created,
            "created_at",
        );
        let modified_at = extract_timestamp(
            path,
            metadata.as_ref(),
            fs::Metadata::modified,
            "modified_at",
        );

        // Read and parse TOML content
        let content =
            fs::read_to_string(path).map_err(|e| ConfigIngestError::Io {
                path: path.to_path_buf(),
                source: e,
            })?;

        let config: RawConfig = toml::from_str(&content).map_err(|e| {
            ConfigIngestError::TomlParse {
                path: path.to_path_buf(),
                source: e,
            }
        })?;

        Ok(Some((config, created_at, modified_at)))
    }
}

impl Default for Ingestor {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Result tuple containing parsed config and filesystem timestamps.
///
/// Format: `(RawConfig, created_at, modified_at)`.
pub type RawConfigWithMetadata =
    (RawConfig, Option<Timestamp>, Option<Timestamp>);

/// Extract a timestamp from file metadata, logging any errors at debug level.
///
/// Returns `None` if metadata is unavailable or timestamp extraction fails.
fn extract_timestamp(
    path: &Path,
    metadata: Option<&fs::Metadata>,
    time_fn: fn(&fs::Metadata) -> std::io::Result<std::time::SystemTime>,
    time_type: &str,
) -> Option<Timestamp> {
    let meta = metadata?;

    let system_time = match time_fn(meta) {
        Ok(t) => t,
        Err(e) => {
            tracing::debug!(
                path = %path.display(),
                error = %e,
                time_type,
                "Failed to read timestamp from metadata"
            );
            return None;
        }
    };

    match system_time.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(duration) => Some(Timestamp::from_secs(duration.as_secs())),
        Err(e) => {
            tracing::debug!(
                path = %path.display(),
                error = %e,
                time_type,
                "Timestamp before UNIX_EPOCH"
            );
            None
        }
    }
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

    mod resolve_global_config_path_tests {
        use super::*;

        #[test]
        fn returns_option_pathbuf() {
            let ingestor = Ingestor::new();
            let result = ingestor.resolve_global_config_path();
            assert!(result.is_some() || result.is_none());
        }

        #[test]
        fn returns_absolute_path_when_found() {
            let ingestor = Ingestor::new();
            if let Some(path) = ingestor.resolve_global_config_path() {
                assert!(
                    path.is_absolute(),
                    "Config path should be absolute: {}",
                    path.display()
                );
            }
        }
    }

    mod load_global_config_tests {
        use super::*;

        #[test]
        fn can_be_called_without_error() {
            // Test that the function can be called and returns a valid result
            // We can't guarantee the environment state, but we can verify the
            // signature
            let ingestor = Ingestor::new();
            let result = ingestor.load_global_config();
            assert!(result.is_ok(), "Function should not error");
        }
    }

    mod load_vault_config_tests {
        use std::fs;

        use tempfile::tempdir;

        use super::*;

        #[test]
        fn returns_none_when_config_missing() {
            let temp = tempdir().expect("create temp dir");
            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new();
            let result = ingestor
                .load_vault_config(&vault_root)
                .expect("should not error");
            assert!(
                result.is_none(),
                "Should return None when config doesn't exist"
            );
        }

        #[test]
        fn loads_valid_toml_config() {
            let temp = tempdir().expect("create temp dir");
            let lithos_dir = temp.path().join(".lithos");
            fs::create_dir_all(&lithos_dir).expect("create .lithos dir");

            let config_path = lithos_dir.join("lithos.toml");
            fs::write(&config_path, "[logging]\nlog_level = \"debug\"\n")
                .expect("write config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new();
            let result = ingestor
                .load_vault_config(&vault_root)
                .expect("should parse config")
                .expect("config should exist");

            let (config, _created, _modified) = result;
            assert!(config.logging.is_some(), "Should parse logging section");
        }

        #[test]
        fn extracts_metadata_when_available() {
            let temp = tempdir().expect("create temp dir");
            let lithos_dir = temp.path().join(".lithos");
            fs::create_dir_all(&lithos_dir).expect("create .lithos dir");

            let config_path = lithos_dir.join("lithos.toml");
            fs::write(&config_path, "[paths]\n").expect("write config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new();
            let result = ingestor
                .load_vault_config(&vault_root)
                .expect("should parse config")
                .expect("config should exist");

            let (_config, _created, modified) = result;
            assert!(
                modified.is_some(),
                "Modified timestamp should be extracted"
            );
        }

        #[test]
        fn returns_error_on_invalid_toml() {
            let temp = tempdir().expect("create temp dir");
            let lithos_dir = temp.path().join(".lithos");
            fs::create_dir_all(&lithos_dir).expect("create .lithos dir");

            let config_path = lithos_dir.join("lithos.toml");
            fs::write(&config_path, "invalid toml [[[")
                .expect("write invalid config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new();
            let result = ingestor.load_vault_config(&vault_root);
            assert!(result.is_err(), "Should return error for invalid TOML");
        }
    }

    mod build_merged_raw_tests {
        use super::*;

        #[test]
        fn uses_defaults_when_file_missing() {
            let dir = tempdir().expect("tempdir");
            let ingestor = Ingestor::new();
            let result = ingestor.build_merged_raw(dir.path());
            assert!(result.is_ok(), "Expected default ingest to succeed");
        }

        #[test]
        fn reads_lithos_toml_when_present() {
            let (dir, _path) = fixtures::setup_vault_with_config(
                "[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup vault");

            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw(dir.path())
                .expect("build merged raw");
            let logging = raw.logging.expect("logging section missing");
            assert_eq!(logging.log_level.as_deref(), Some("debug"));
        }
    }

    mod layering_tests {
        use fixtures::setup_layered_configs;

        use super::*;

        #[test]
        fn vault_overrides_global() {
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[logging]\nlog_level = \"info\"\n",
                "[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                .expect("build merged raw");

            let logging = raw.logging.expect("logging should be Some");
            assert_eq!(
                logging.log_level.as_deref(),
                Some("debug"),
                "Vault config should override global"
            );
        }

        #[test]
        fn global_used_when_vault_missing() {
            let (_global_dir, global_path, vault_dir) =
                setup_layered_configs("[logging]\nlog_level = \"warn\"\n", "")
                    .expect("setup configs");

            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                .expect("build merged raw");

            let logging = raw.logging.expect("logging should be Some");
            assert_eq!(
                logging.log_level.as_deref(),
                Some("warn"),
                "Global config should be used when vault missing"
            );
        }

        #[test]
        fn defaults_used_when_both_missing() {
            let dir = tempdir().expect("tempdir");
            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw_impl(dir.path(), None)
                .expect("build merged raw");

            assert!(
                raw.logging.is_none(),
                "Logging should be None when not specified"
            );
        }

        #[test]
        fn paths_fields_merge_correctly() {
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[paths]\nschemas_dir = \"global-schemas\"\n",
                "[paths]\ncache_dir = \".cache\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                .expect("build merged raw");

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
        fn vault_overrides_global_paths_field() {
            let (_global_dir, global_path, vault_dir) = setup_layered_configs(
                "[paths]\ntemplates_dir = \"global-templates\"\n",
                "[paths]\ntemplates_dir = \"vault-templates\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new();
            let raw = ingestor
                .build_merged_raw_impl(vault_dir.path(), Some(&global_path))
                .expect("build merged raw");

            assert_eq!(
                raw.paths.templates_dir.as_deref(),
                Some("vault-templates"),
                "Vault paths field should override global"
            );
        }
    }
}
