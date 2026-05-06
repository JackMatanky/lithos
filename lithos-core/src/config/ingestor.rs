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

use crate::{
    config::{
        error::ConfigIngestError,
        raw::{RawConfigMetadata, RawGlobalConfig, RawVaultConfig},
        vault::VaultRoot,
    },
    fs::FsReader,
    support::hash::Blake3Hash,
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

    /// Get the global config file with metadata extraction.
    ///
    /// Discovers global config using priority order defined by
    /// [`GlobalConfigLocation`]. See that type for the full priority list.
    ///
    /// For each location:
    /// - Resolves the path (if environment variables are set)
    /// - Checks if file exists
    /// - Loads the first file found
    ///
    /// Then:
    /// - Reads the file from disk
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Computes BLAKE3 content hash
    /// - Parses TOML content into [`RawGlobalConfig`]
    /// - Populates metadata fields on the returned type
    ///
    /// Returns `None` if no global config file exists at any location.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigIngestError`] if:
    /// - File reading fails (I/O error)
    /// - TOML parsing fails (syntax error)
    #[inline]
    pub fn global_config(
        &self,
    ) -> Result<Option<RawGlobalConfig>, ConfigIngestError> {
        // Step 1: Find first existing global config path
        let path = GlobalConfigLocation::all().iter().find_map(|location| {
            location.resolve().filter(|p| self.global_source.exists(p))
        });

        let Some(path) = path else {
            return Ok(None);
        };

        // Step 2: Load config from discovered path
        // Extract timestamps using FsReader methods
        let created_at = self.global_source.created_at(&path);
        let modified_at = self.global_source.modified_at(&path);

        // Read raw file bytes for content hashing (before parsing)
        let raw_bytes = self.global_source.read_bytes(&path)?;

        // Compute BLAKE3 hash from raw bytes
        let content_hash = Blake3Hash::compute(&raw_bytes);

        // Parse TOML content using FsReader
        let mut config: RawGlobalConfig =
            self.global_source.parse_structured(&path)?;

        // Populate metadata
        config.metadata = RawConfigMetadata {
            created_at,
            modified_at,
            content_hash: Some(content_hash),
        };

        Ok(Some(config))
    }

    /// Get the vault config file with metadata extraction.
    ///
    /// Reads from `{vault_root}/.lithos/lithos.toml`, then:
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Computes BLAKE3 content hash
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
    pub fn vault_config(
        &self,
        _vault_root: &VaultRoot,
    ) -> Result<Option<RawVaultConfig>, ConfigIngestError> {
        let relative_path = Path::new(".lithos/lithos.toml");

        if !self.vault_source.exists(relative_path) {
            return Ok(None);
        }

        // Extract timestamps using FsReader methods
        let created_at = self.vault_source.created_at(relative_path);
        let modified_at = self.vault_source.modified_at(relative_path);

        // Read raw file bytes for content hashing (before parsing)
        let raw_bytes = self.vault_source.read_bytes(relative_path)?;

        // Compute BLAKE3 hash from raw bytes
        let content_hash = Blake3Hash::compute(&raw_bytes);

        // Parse TOML content using FsReader
        let mut config: RawVaultConfig =
            self.vault_source.parse_structured(relative_path)?;

        // Populate metadata
        config.metadata = RawConfigMetadata {
            created_at,
            modified_at,
            content_hash: Some(content_hash),
        };

        Ok(Some(config))
    }
}

/// Global configuration file search locations.
///
/// Represents the priority-ordered locations where Lithos searches for the
/// global configuration file. Priority order (first found wins):
///
/// 1. [`EnvOverride`](Self::EnvOverride) - `$LITHOS_GLOBAL_CONFIG`
/// 2. [`XdgConfigHome`](Self::XdgConfigHome) -
///    `$XDG_CONFIG_HOME/lithos/lithos.toml`
/// 3. [`HomeConfig`](Self::HomeConfig) - `$HOME/.config/lithos/lithos.toml`
/// 4. [`HomeLegacy`](Self::HomeLegacy) - `$HOME/.lithos/lithos.toml`
/// 5. [`SystemWide`](Self::SystemWide) - `/etc/lithos/lithos.toml`
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::ingestor::GlobalConfigLocation;
///
/// // Resolve path with environment variables
/// if let Some(path) = GlobalConfigLocation::EnvOverride.resolve() {
///     println!("Found global config at: {}", path.display());
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GlobalConfigLocation {
    /// Environment variable override: `$LITHOS_GLOBAL_CONFIG`.
    ///
    /// Highest priority. Allows users to specify an arbitrary config location.
    EnvOverride,

    /// XDG Base Directory: `$XDG_CONFIG_HOME/lithos/lithos.toml`.
    ///
    /// Follows XDG Base Directory Specification when `XDG_CONFIG_HOME` is set.
    XdgConfigHome,

    /// XDG default: `$HOME/.config/lithos/lithos.toml`.
    ///
    /// Standard XDG location when `XDG_CONFIG_HOME` is not set.
    HomeConfig,

    /// Legacy location: `$HOME/.lithos/lithos.toml`.
    ///
    /// Historical location for backward compatibility.
    HomeLegacy,

    /// System-wide: `/etc/lithos/lithos.toml`.
    ///
    /// Lowest priority. System administrator default for all users.
    SystemWide,
}

impl GlobalConfigLocation {
    /// Returns all locations in priority order.
    ///
    /// Useful for iterating through search locations.
    #[inline]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::EnvOverride,
            Self::XdgConfigHome,
            Self::HomeConfig,
            Self::HomeLegacy,
            Self::SystemWide,
        ]
    }

    /// Resolves this location to an actual filesystem path.
    ///
    /// Returns `None` if:
    /// - Required environment variables are not set
    /// - Path resolution fails
    ///
    /// Does NOT check if the file exists - use with `FsReader::exists()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lithos_core::config::ingestor::GlobalConfigLocation;
    ///
    /// if let Some(path) = GlobalConfigLocation::HomeConfig.resolve() {
    ///     println!("Home config would be at: {}", path.display());
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve(&self) -> Option<PathBuf> {
        match *self {
            Self::EnvOverride => {
                std::env::var("LITHOS_GLOBAL_CONFIG").ok().map(PathBuf::from)
            }
            Self::XdgConfigHome => std::env::var("XDG_CONFIG_HOME")
                .ok()
                .map(|xdg| Path::new(&xdg).join("lithos/lithos.toml")),
            Self::HomeConfig => std::env::var("HOME").ok().map(|home| {
                Path::new(&home).join(".config/lithos/lithos.toml")
            }),
            Self::HomeLegacy => std::env::var("HOME")
                .ok()
                .map(|home| Path::new(&home).join(".lithos/lithos.toml")),
            Self::SystemWide => Some(PathBuf::from("/etc/lithos/lithos.toml")),
        }
    }

    /// Returns a human-readable description of this location.
    #[inline]
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match *self {
            Self::EnvOverride => {
                "environment variable override ($LITHOS_GLOBAL_CONFIG)"
            }
            Self::XdgConfigHome => {
                "XDG config directory ($XDG_CONFIG_HOME/lithos/lithos.toml)"
            }
            Self::HomeConfig => {
                "home config directory ($HOME/.config/lithos/lithos.toml)"
            }
            Self::HomeLegacy => {
                "legacy home directory ($HOME/.lithos/lithos.toml)"
            }
            Self::SystemWide => "system-wide config (/etc/lithos/lithos.toml)",
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: resolve_global_config_path tests removed - that method was inlined

    mod global_config_tests {
        use super::*;

        #[test]
        fn can_be_called_without_error() {
            // Test that the function can be called and returns a valid result
            // We can't guarantee the environment state, but we can verify the
            // signature
            let ingestor = Ingestor::new(std::env::temp_dir());
            let result = ingestor.global_config();
            assert!(result.is_ok(), "Function should not error");
        }
    }

    mod vault_config_tests {
        use std::fs;

        use tempfile::tempdir;

        use super::*;

        #[test]
        fn returns_none_when_config_missing() {
            let temp = tempdir().expect("create temp dir");
            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let ingestor = Ingestor::new(temp.path());
            let result =
                ingestor.vault_config(&vault_root).expect("should not error");
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
                .vault_config(&vault_root)
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
                .vault_config(&vault_root)
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
            let result = ingestor.vault_config(&vault_root);
            assert!(result.is_err(), "Should return error for invalid TOML");
        }
    }

    // NOTE: build_merged_raw tests removed - that functionality moved to Loader
}
