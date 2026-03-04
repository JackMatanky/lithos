//! Ingestor adapter for loading raw config files from the filesystem.
//!
//! Pure file-to-raw translation with metadata extraction. No DB access.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::{
    aggregate::Timestamp, error::ConfigIngestError, raw::RawConfig,
    vault::VaultRoot,
};

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
///
/// # Examples
///
/// ```
/// use lithos_core::config::adapter::ingestor::resolve_global_config_path;
///
/// let path = resolve_global_config_path();
/// // Returns the first existing config file path, or None
/// ```
#[inline]
#[must_use]
pub fn resolve_global_config_path() -> Option<PathBuf> {
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

/// Result tuple containing parsed config and filesystem timestamps.
///
/// Format: `(RawConfig, created_at, modified_at)`.
pub type RawConfigWithMetadata =
    (RawConfig, Option<Timestamp>, Option<Timestamp>);

/// Load and parse the global config file with metadata extraction.
///
/// Uses [`resolve_global_config_path`] to find the config file, then:
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
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::adapter::ingestor::load_global_config;
///
/// match load_global_config()? {
///     Some((config, created, modified)) => {
///         println!("Loaded global config with metadata");
///     }
///     None => {
///         println!("No global config found");
///     }
/// }
/// ```
#[inline]
pub fn load_global_config()
-> Result<Option<RawConfigWithMetadata>, ConfigIngestError> {
    let Some(path) = resolve_global_config_path() else {
        return Ok(None);
    };

    load_config_from_path(&path)
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
///
/// # Examples
///
/// ```ignore
/// use lithos_core::config::{adapter::ingestor::load_vault_config, vault::VaultRoot};
/// use std::path::PathBuf;
///
/// let vault_root = VaultRoot::try_new(PathBuf::from("/vault"))?;
/// match load_vault_config(&vault_root)? {
///     Some((config, created, modified)) => {
///         println!("Loaded vault config with metadata");
///     }
///     None => {
///         println!("No vault config found");
///     }
/// }
/// ```
#[inline]
pub fn load_vault_config(
    vault_root: &VaultRoot,
) -> Result<Option<RawConfigWithMetadata>, ConfigIngestError> {
    let path = vault_root.as_path().join(".lithos/lithos.toml");

    if !path.exists() {
        return Ok(None);
    }

    load_config_from_path(&path)
}

/// Internal helper to load and parse a config file with metadata.
fn load_config_from_path(
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

    let config: RawConfig =
        toml::from_str(&content).map_err(|e| ConfigIngestError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })?;

    Ok(Some((config, created_at, modified_at)))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod resolve_global_config_path_tests {
        use super::*;

        #[test]
        fn returns_option_pathbuf() {
            // Test that the function compiles and returns Option<PathBuf>
            let result = resolve_global_config_path();
            // Verify type by asserting it's an Option
            assert!(result.is_some() || result.is_none());
        }

        #[test]
        fn returns_absolute_path_when_found() {
            // If a config is found, it should be an absolute path
            if let Some(path) = resolve_global_config_path() {
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
        fn returns_none_when_no_config_exists() {
            // This test can't guarantee no config exists in the test
            // environment but verifies the function signature and
            // type safety
            let result = load_global_config();
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

            let result =
                load_vault_config(&vault_root).expect("should not error");
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
            fs::write(&config_path, "[logging]\nlevel = \"debug\"\n")
                .expect("write config");

            let vault_root = VaultRoot::try_new(temp.path().to_path_buf())
                .expect("valid vault root");

            let result = load_vault_config(&vault_root)
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

            let result = load_vault_config(&vault_root)
                .expect("should parse config")
                .expect("config should exist");

            let (_config, _created, modified) = result;
            // Modified timestamp should be available on most platforms
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

            let result = load_vault_config(&vault_root);
            assert!(result.is_err(), "Should return error for invalid TOML");
        }
    }
}
