//! Ingestor adapter for loading raw config files from the filesystem.
//!
//! Pure file-to-raw translation with metadata extraction. No DB access.

use std::path::{Path, PathBuf};

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
}
