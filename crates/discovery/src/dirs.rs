//! Resolved platform and application directories for Lithos.
//!
//! [`AppDirs`] merges [`EnvVars`](crate::EnvVars) overrides with XDG
//! platform defaults from [`crate`].
//!
//! # Example
//!
//! ```rust,ignore
//! use trace_discovery::dirs::AppDirs;
//! use trace_discovery::EnvVars;
//!
//! let vars = EnvVars::capture();
//! let app = AppDirs::new(&vars);
//! let cache = app.cache();
//! ```

use std::path::PathBuf;

use crate::env::{XDG_CACHE_HOME, XDG_CONFIG_HOME};

// ---------------------------------------------------------------------------
// AppDirs
// ---------------------------------------------------------------------------

/// Resolved Lithos application directories.
///
/// Merges [`EnvVars`](crate::EnvVars) overrides with XDG platform
/// defaults, applying the `"lithos"` suffix to platform base directories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppDirs {
    cache: PathBuf,
    config: PathBuf,
    system_config: Option<PathBuf>,
}

impl AppDirs {
    /// Resolve app directories from env captures and platform defaults.
    ///
    /// Precedence per directory:
    /// - **cache**: `vars.cache_dir` → `XDG_CACHE_HOME / "lithos"`
    /// - **config**: `XDG_CONFIG_HOME / "lithos"`
    /// - **`system_config`**: `/etc/lithos` (unix), `%PROGRAMDATA%/Lithos`
    ///   (win)
    #[inline]
    #[must_use]
    pub fn new(vars: &crate::EnvVars) -> Self {
        let cache = vars
            .cache_dir()
            .cloned()
            .unwrap_or_else(|| XDG_CACHE_HOME.join("lithos"));
        let config = XDG_CONFIG_HOME.join("lithos");
        let system_config = platform_system_config();
        Self {
            cache,
            config,
            system_config,
        }
    }

    /// Resolved Lithos cache directory.
    #[inline]
    #[must_use]
    pub fn cache(&self) -> &PathBuf {
        &self.cache
    }

    /// Resolved Lithos global config directory.
    #[inline]
    #[must_use]
    pub fn config(&self) -> &PathBuf {
        &self.config
    }

    /// System-wide Lithos config directory.
    #[inline]
    #[must_use]
    pub fn system_config(&self) -> Option<&PathBuf> {
        self.system_config.as_ref()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// System-wide config directory.
#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "signature matches non-unix variant"
)]
fn platform_system_config() -> Option<PathBuf> {
    Some(PathBuf::from("/etc/lithos"))
}

/// System-wide config directory.
#[cfg(windows)]
fn platform_system_config() -> Option<PathBuf> {
    Some(PathBuf::from(r"C:\ProgramData\Lithos"))
}

/// System-wide config directory (other platforms: none).
#[cfg(not(any(unix, windows)))]
fn platform_system_config() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EnvVars;

    mod app_dirs {
        use super::*;

        #[test]
        fn uses_xdg_config_home_plus_lithos_when_no_env_overrides() {
            let vars = EnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            let expected = XDG_CONFIG_HOME.join("lithos");
            assert_eq!(dirs.config(), &expected);
            assert!(!dirs.cache().as_os_str().is_empty());
        }

        #[test]
        fn cache_uses_env_override_when_set() {
            let vars = EnvVars::new(
                None,
                None,
                Some(PathBuf::from("/override/cache")),
                None,
                false,
            );
            let dirs = AppDirs::new(&vars);

            assert_eq!(dirs.cache(), &PathBuf::from("/override/cache"));
        }

        #[test]
        fn cache_falls_back_to_xdg_cache_home_plus_lithos() {
            let vars = EnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            let expected = XDG_CACHE_HOME.join("lithos");
            assert_eq!(dirs.cache(), &expected);
        }

        #[cfg(unix)]
        #[test]
        fn system_config_is_etc_lithos_on_unix() {
            let vars = EnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            assert_eq!(
                dirs.system_config(),
                Some(&PathBuf::from("/etc/lithos"))
            );
        }
    }
}
