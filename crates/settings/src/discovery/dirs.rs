//! Resolved platform and application directories for Traces.
//!
//! [`AppDirs`] merges [`EnvVars`](crate::SettingsEnvVars) overrides with XDG
//! platform defaults from [`crate`].
//!
//! # Example
//!
//! ```rust,ignore
//! use traces_settings::config::dirs::AppDirs;
//! use traces_settings::config::EnvVars;
//!
//! let vars = EnvVars::capture();
//! let app = AppDirs::new(&vars);
//! let cache = app.cache();
//! ```

use std::path::PathBuf;

use crate::os_dirs::{XDG_CACHE_HOME, XDG_CONFIG_HOME};

// ---------------------------------------------------------------------------
// AppDirs
// ---------------------------------------------------------------------------

/// Resolved Traces application directories.
///
/// Merges [`EnvVars`](crate::SettingsEnvVars) overrides with XDG platform
/// defaults, applying the `"traces"` suffix to platform base directories.
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
    /// - **cache**: `vars.cache_dir` → `XDG_CACHE_HOME / "traces"`
    /// - **config**: `XDG_CONFIG_HOME / "traces"`
    /// - **`system_config`**: `/etc/traces` (unix), `%PROGRAMDATA%/Traces`
    ///   (win)
    #[inline]
    #[must_use]
    pub fn new(vars: &crate::SettingsEnvVars) -> Self {
        let cache = vars
            .cache_dir()
            .cloned()
            .unwrap_or_else(|| XDG_CACHE_HOME.join("traces"));
        let config = XDG_CONFIG_HOME.join("traces");
        let system_config = platform_system_config();
        Self {
            cache,
            config,
            system_config,
        }
    }

    /// Resolved Traces cache directory.
    #[inline]
    #[must_use]
    pub fn cache(&self) -> &PathBuf {
        &self.cache
    }

    /// Resolved Traces global config directory.
    #[inline]
    #[must_use]
    pub fn config(&self) -> &PathBuf {
        &self.config
    }

    /// System-wide Traces config directory.
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
    Some(PathBuf::from("/etc/traces"))
}

/// System-wide config directory.
#[cfg(windows)]
fn platform_system_config() -> Option<PathBuf> {
    Some(PathBuf::from(r"C:\ProgramData\Traces"))
}

/// System-wide config directory (other platforms: none).
#[cfg(not(any(unix, windows)))]
fn platform_system_config() -> Option<PathBuf> {
    None
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SettingsEnvVars;

    mod app_dirs {
        use super::*;

        #[test]
        fn uses_xdg_config_home_plus_traces_when_no_env_overrides() {
            let vars = SettingsEnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            let expected = XDG_CONFIG_HOME.join("traces");
            assert_eq!(dirs.config(), &expected);
            assert!(!dirs.cache().as_os_str().is_empty());
        }

        #[test]
        fn cache_uses_env_override_when_set() {
            let vars = SettingsEnvVars::new(
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
        fn cache_falls_back_to_xdg_cache_home_plus_traces() {
            let vars = SettingsEnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            let expected = XDG_CACHE_HOME.join("traces");
            assert_eq!(dirs.cache(), &expected);
        }

        #[cfg(unix)]
        #[test]
        fn system_config_is_etc_traces_on_unix() {
            let vars = SettingsEnvVars::new(None, None, None, None, false);
            let dirs = AppDirs::new(&vars);

            assert_eq!(
                dirs.system_config(),
                Some(&PathBuf::from("/etc/traces"))
            );
        }
    }
}
