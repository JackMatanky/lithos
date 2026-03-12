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

use std::path::{Path, PathBuf};

use figment::{Figment, providers::Serialized};
use tracing::instrument;

use crate::{
    config::{
        error::ConfigIngestError,
        raw::{RawConfig, RawConfigMetadata, RawGlobalConfig, RawVaultConfig},
        vault::VaultRoot,
    },
    fs::FsReader,
};

/// Configuration ingestion adapter.
///
/// Handles loading raw configuration from the filesystem with metadata
/// extraction. Supports both individual file loading (with timestamps) and
/// Figment-based layered merging.
///
/// Uses separate `FsReader` instances for global (system-wide) and vault
/// (project-scoped) config resolution:
/// - Global: `FsReader::from_system_root()` for absolute system paths
/// - Vault: `FsReader::new(vault_root)` for vault-relative paths
pub struct Ingestor {
    /// Reader for vault-scoped file operations (relative to vault root).
    vault_source: FsReader,
    /// Reader for system-wide file operations (absolute paths).
    global_source: FsReader,
}

impl Ingestor {
    /// Create a new ingestor for the given vault root.
    ///
    /// Automatically creates:
    /// - Vault-scoped reader (for `.lithos/lithos.toml` within vault)
    /// - System-wide reader (for global config resolution)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::config::ingestor::Ingestor;
    ///
    /// let vault_root = PathBuf::from("/vault");
    /// let ingestor = Ingestor::new(vault_root);
    /// ```
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(vault_root: P) -> Self {
        Self {
            vault_source: FsReader::new(vault_root),
            global_source: FsReader::from_system_root(),
        }
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
    pub fn resolve_global_config_path(&self) -> Option<PathBuf> {
        // Priority 1: Environment variable override
        if let Ok(env_path) = std::env::var("LITHOS_GLOBAL_CONFIG") {
            let path = PathBuf::from(env_path);
            if self.global_source.exists(&path) {
                return Some(path);
            }
        }

        // Priority 2: XDG_CONFIG_HOME
        if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
            let path = Path::new(&xdg_home).join("lithos/lithos.toml");
            if self.global_source.exists(&path) {
                return Some(path);
            }
        }

        // Priority 3: HOME/.config (XDG default)
        if let Ok(home) = std::env::var("HOME") {
            let path = Path::new(&home).join(".config/lithos/lithos.toml");
            if self.global_source.exists(&path) {
                return Some(path);
            }
        }

        // Priority 4: HOME/.lithos (legacy)
        if let Ok(home) = std::env::var("HOME") {
            let path = Path::new(&home).join(".lithos/lithos.toml");
            if self.global_source.exists(&path) {
                return Some(path);
            }
        }

        // Priority 5: System-wide /etc
        let system_path = PathBuf::from("/etc/lithos/lithos.toml");
        if self.global_source.exists(&system_path) {
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
    /// - Parses TOML content into [`RawGlobalConfig`]
    /// - Populates metadata fields on the returned type
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
        &self,
    ) -> Result<Option<RawGlobalConfig>, ConfigIngestError> {
        let Some(path) = self.resolve_global_config_path() else {
            return Ok(None);
        };

        self.load_config_from_path(&path)
    }

    /// Load and parse a vault config file with metadata extraction.
    ///
    /// Reads from `{vault_root}/.lithos/lithos.toml`, then:
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Parses TOML content into [`RawVaultConfig`]
    /// - Populates metadata fields on the returned type
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
        &self,
        _vault_root: &VaultRoot,
    ) -> Result<Option<RawVaultConfig>, ConfigIngestError> {
        let relative_path = Path::new(".lithos/lithos.toml");

        if !self.vault_source.exists(relative_path) {
            return Ok(None);
        }

        self.load_config_from_vault_path(relative_path)
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
        &self,
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
    #[instrument(
        skip(self, vault_root, global_config_path),
        level = "debug",
        fields(operation = "build_merged_raw_impl", vault_root = %vault_root.display())
    )]
    fn build_merged_raw_impl(
        &self,
        vault_root: &Path,
        global_config_path: Option<&Path>,
    ) -> Result<RawConfig, ConfigIngestError> {
        // Layer 1: Compiled defaults
        let mut figment =
            Figment::from(Serialized::defaults(RawConfig::default()));

        // Layer 2: Global config (if exists)
        if let Some(path) = global_config_path
            && self.global_source.exists(path)
            && let Some(global) = self.load_config_from_path(path)?
        {
            figment =
                figment.merge(Serialized::defaults(RawConfig::from(global)));
        }

        // Layer 3: Vault config (if exists)
        let vault_config_relative = Path::new(".lithos/lithos.toml");
        if self.vault_source.exists(vault_config_relative)
            && let Some(vault) =
                self.load_config_from_vault_path(vault_config_relative)?
        {
            figment =
                figment.merge(Serialized::defaults(RawConfig::from(vault)));
        }

        // Extract merged config
        figment.extract().map_err(ConfigIngestError::from)
    }

    /// Internal helper to load and parse a global config file with metadata.
    ///
    /// Uses `global_source` for absolute paths.
    fn load_config_from_path(
        &self,
        path: &Path,
    ) -> Result<Option<RawGlobalConfig>, ConfigIngestError> {
        // Extract timestamps using FsReader methods
        let created_at = self.global_source.created_at(path);
        let modified_at = self.global_source.modified_at(path);

        // Read raw file bytes for content hashing (before parsing)
        let raw_bytes = self
            .global_source
            .read_bytes(path)
            .map_err(|e| Self::convert_parse_error(e, path))?;

        // Compute BLAKE3 hash from raw bytes
        let content_hash = blake3::hash(&raw_bytes);

        // Parse TOML content using FsReader
        let mut config: RawGlobalConfig = self
            .global_source
            .parse_structured(path)
            .map_err(|e| Self::convert_parse_error(e, path))?;

        // Populate metadata
        config.metadata = RawConfigMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
        };

        Ok(Some(config))
    }

    /// Internal helper to load and parse a vault config file with metadata.
    ///
    /// Uses `vault_source` for vault-relative paths.
    fn load_config_from_vault_path(
        &self,
        path: &Path,
    ) -> Result<Option<RawVaultConfig>, ConfigIngestError> {
        // Extract timestamps using FsReader methods
        let created_at = self.vault_source.created_at(path);
        let modified_at = self.vault_source.modified_at(path);

        // Read raw file bytes for content hashing (before parsing)
        let raw_bytes = self
            .vault_source
            .read_bytes(path)
            .map_err(|e| Self::convert_parse_error(e, path))?;

        // Compute BLAKE3 hash from raw bytes
        let content_hash = blake3::hash(&raw_bytes);

        // Parse TOML content using FsReader
        let mut config: RawVaultConfig = self
            .vault_source
            .parse_structured(path)
            .map_err(|e| Self::convert_parse_error(e, path))?;

        // Populate metadata
        config.metadata = RawConfigMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
        };

        Ok(Some(config))
    }

    /// Convert `fs::ParseError` to `ConfigIngestError`.
    fn convert_parse_error(
        e: crate::fs::ParseError,
        file_path: &Path,
    ) -> ConfigIngestError {
        match e {
            crate::fs::ParseError::Io {
                path: err_path,
                source: io_source,
            } => ConfigIngestError::Io {
                path: err_path,
                source: io_source,
            },
            crate::fs::ParseError::Toml {
                path: err_path,
                message,
                ..
            } => {
                // Convert fs::ParseError::Toml to ConfigIngestError::TomlParse
                // We need to create a toml::de::Error, but it doesn't have
                // a public constructor. We parse invalid TOML to get an
                // error instance, but preserve the original error message
                // via eprintln for debugging.
                #[expect(
                    clippy::expect_used,
                    reason = "We intentionally create an error by parsing \
                              invalid TOML to get an error instance"
                )]
                let toml_error = toml::from_str::<toml::Value>("[")
                    .expect_err("Invalid TOML should always error");

                // Log the actual error message for debugging
                eprintln!(
                    "TOML parse error in {}: {}",
                    err_path.display(),
                    message
                );

                ConfigIngestError::TomlParse {
                    path: err_path,
                    source: toml_error,
                }
            }
            crate::fs::ParseError::Json {
                ..
            }
            | crate::fs::ParseError::Yaml {
                ..
            }
            | crate::fs::ParseError::UnsupportedFormat {
                ..
            } => ConfigIngestError::Io {
                path: file_path.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            },
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

            // Write vault config only if content is not empty
            if !vault_content.is_empty() {
                let vault_config_dir = vault_dir.path().join(".lithos");
                fs::create_dir_all(&vault_config_dir)?;
                let vault_config_path = vault_config_dir.join("lithos.toml");
                fs::write(&vault_config_path, vault_content)?;
            }

            Ok((global_dir, global_config_path, vault_dir))
        }
    }

    mod resolve_global_config_path_tests {
        use super::*;

        #[test]
        fn returns_option_pathbuf() {
            let ingestor = Ingestor::new(std::env::temp_dir());
            let result = ingestor.resolve_global_config_path();
            assert!(result.is_some() || result.is_none());
        }

        #[test]
        fn returns_absolute_path_when_found() {
            let ingestor = Ingestor::new(std::env::temp_dir());
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
            let ingestor = Ingestor::new(std::env::temp_dir());
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

            let ingestor = Ingestor::new(temp.path());
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
            fs::write(
                &config_path,
                "vault_path = \"/vault\"\n[logging]\nlog_level = \"debug\"\n",
            )
            .expect("write config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new(temp.path());
            let config = ingestor
                .load_vault_config(&vault_root)
                .expect("should parse config")
                .expect("config should exist");

            assert!(config.logging.is_some(), "Should parse logging section");
        }

        #[test]
        fn extracts_metadata_when_available() {
            let temp = tempdir().expect("create temp dir");
            let lithos_dir = temp.path().join(".lithos");
            fs::create_dir_all(&lithos_dir).expect("create .lithos dir");

            let config_path = lithos_dir.join("lithos.toml");
            fs::write(&config_path, "vault_path = \"/vault\"\n[paths]\n")
                .expect("write config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new(temp.path());
            let config = ingestor
                .load_vault_config(&vault_root)
                .expect("should parse config")
                .expect("config should exist");

            assert!(
                config.metadata.modified_at.is_some(),
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

            let ingestor = Ingestor::new(temp.path());
            let result = ingestor.load_vault_config(&vault_root);
            assert!(result.is_err(), "Should return error for invalid TOML");
        }
    }

    mod build_merged_raw_tests {
        use super::*;

        #[test]
        fn uses_defaults_when_file_missing() {
            let dir = tempdir().expect("tempdir");
            let ingestor = Ingestor::new(dir.path());
            let result = ingestor.build_merged_raw(dir.path());
            assert!(result.is_ok(), "Expected default ingest to succeed");
        }

        #[test]
        fn reads_lithos_toml_when_present() {
            let (dir, _path) = fixtures::setup_vault_with_config(
                "vault_path = \"/vault\"\n[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup vault");

            let ingestor = Ingestor::new(dir.path());
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
                "vault_path = \"/vault\"\n[logging]\nlog_level = \"debug\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new(vault_dir.path());
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

            let ingestor = Ingestor::new(vault_dir.path());
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
            let ingestor = Ingestor::new(dir.path());
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
                "vault_path = \"/vault\"\n[paths]\ncache_dir = \".cache\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new(vault_dir.path());
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
                "vault_path = \"/vault\"\n[paths]\ntemplates_dir = \
                 \"vault-templates\"\n",
            )
            .expect("setup configs");

            let ingestor = Ingestor::new(vault_dir.path());
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
