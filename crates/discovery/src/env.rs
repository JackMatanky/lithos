//! Central registry of Lithos environment variables and platform directories.
//!
//! All `LITHOS_*` environment variables are read once through [`EnvVars`] and
//! exposed as a consistent API. Platform directories (`HOME`, `XDG_*`) are
//! resolved as lazy statics with per-platform fallbacks.
//!
//! No other module should call `std::env::var` or `std::env::var_os` directly.
//!
//! # Example
//!
//! ```rust,ignore
//! use trace_discovery::{EnvVars, XDG_CACHE_HOME};
//!
//! let vars = EnvVars::capture();
//! let cache_base = XDG_CACHE_HOME.join("lithos");
//! ```

use std::{path::PathBuf, sync::LazyLock};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn var_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

fn var_is_true(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            matches!(
                v.to_lowercase().as_str(),
                "y" | "yes" | "true" | "1" | "on"
            )
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// HOME
// ---------------------------------------------------------------------------

/// The user's home directory.
///
/// In test builds this points to `<CARGO_MANIFEST_DIR>/tests/fixtures/` so
/// that discovery tests do not accidentally depend on the real home directory.
#[cfg(test)]
pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
});

/// The user's home directory.
#[cfg(not(test))]
pub static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

// ---------------------------------------------------------------------------
// XDG platform directories
// ---------------------------------------------------------------------------

/// `$XDG_CONFIG_HOME` or the platform config directory.
#[cfg(target_os = "macos")]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

/// `$XDG_CONFIG_HOME` or the platform config directory.
#[cfg(windows)]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .or_else(|| var_path("APPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Roaming"))
});

/// `$XDG_CONFIG_HOME` or the platform config directory.
#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME").unwrap_or_else(|| HOME.join(".config"))
});

// ---------------------------------------------------------------------------

/// `$XDG_CACHE_HOME` or the platform cache directory.
#[cfg(target_os = "macos")]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join("Library/Caches"))
});

/// `$XDG_CACHE_HOME` or the platform cache directory.
#[cfg(windows)]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME")
        .or_else(|| var_path("TEMP"))
        .unwrap_or_else(|| HOME.join("AppData/Local/Temp"))
});

/// `$XDG_CACHE_HOME` or the platform cache directory.
#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join(".cache"))
});

// ---------------------------------------------------------------------------

/// `$XDG_DATA_HOME` or the platform data directory.
#[cfg(target_os = "macos")]
#[expect(dead_code, reason = "Reserved for platform data directory discovery")]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

/// `$XDG_DATA_HOME` or the platform data directory.
#[cfg(windows)]
#[expect(dead_code, reason = "Reserved for platform data directory discovery")]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

/// `$XDG_DATA_HOME` or the platform data directory.
#[cfg(not(any(target_os = "macos", windows)))]
#[expect(dead_code, reason = "Reserved for platform data directory discovery")]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("share"))
});

// ---------------------------------------------------------------------------

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(target_os = "macos")]
#[expect(dead_code, reason = "Reserved for platform state directory discovery")]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(windows)]
#[expect(dead_code, reason = "Reserved for platform state directory discovery")]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(not(any(target_os = "macos", windows)))]
#[expect(dead_code, reason = "Reserved for platform state directory discovery")]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("state"))
});

// ---------------------------------------------------------------------------
// EnvVars
// ---------------------------------------------------------------------------

/// Captured Lithos environment variables.
///
/// Pure capture — no fallbacks, no platform logic, no filesystem validation.
/// Construct via [`EnvVars::capture()`] to read from the real environment, or
/// via [`EnvVars::new()`] for deterministic test construction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnvVars {
    vault_dir: Option<PathBuf>,
    config_file: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    ceiling_dirs: Option<Vec<PathBuf>>,
    suppress_global: bool,
}

impl EnvVars {
    /// Read all LITHOS_* env vars from the process environment.
    #[inline]
    #[must_use]
    pub fn capture() -> Self {
        Self {
            vault_dir: var_path("LITHOS_VAULT_DIR"),
            config_file: var_path("LITHOS_CONFIG_FILE"),
            cache_dir: var_path("LITHOS_CACHE_DIR"),
            ceiling_dirs: var_path("LITHOS_CEILING_DIRS").map(|raw| {
                std::env::split_paths(&raw)
                    .filter(|p| !p.as_os_str().is_empty())
                    .collect()
            }),
            suppress_global: var_is_true("LITHOS_SUPPRESS_GLOBAL"),
        }
    }

    /// Pure constructor for tests and custom setups.
    #[inline]
    #[must_use]
    pub fn new(
        vault_dir: Option<PathBuf>,
        config_file: Option<PathBuf>,
        cache_dir: Option<PathBuf>,
        ceiling_dirs: Option<Vec<PathBuf>>,
        suppress_global: bool,
    ) -> Self {
        Self {
            vault_dir,
            config_file,
            cache_dir,
            ceiling_dirs,
            suppress_global,
        }
    }

    /// Explicit vault root directory override.
    #[inline]
    #[must_use]
    pub fn vault_dir(&self) -> Option<&PathBuf> {
        self.vault_dir.as_ref()
    }

    /// Explicit config file path override.
    #[inline]
    #[must_use]
    pub fn config_file(&self) -> Option<&PathBuf> {
        self.config_file.as_ref()
    }

    /// Explicit cache directory override.
    #[inline]
    #[must_use]
    pub fn cache_dir(&self) -> Option<&PathBuf> {
        self.cache_dir.as_ref()
    }

    /// Colon-separated ceiling directory paths (raw from env).
    #[inline]
    #[must_use]
    pub fn ceiling_dirs(&self) -> Option<&[PathBuf]> {
        self.ceiling_dirs.as_deref()
    }

    /// Whether global config lookup is suppressed.
    #[inline]
    #[must_use]
    pub fn suppress_global(&self) -> bool {
        self.suppress_global
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_is_redirected_in_test_mode() {
        let home = HOME.join("test-marker");
        assert!(
            home.to_string_lossy().contains("tests/fixtures"),
            "HOME should point to tests/fixtures in test mode, got: {}",
            home.display()
        );
    }

    #[test]
    fn xdg_config_home_is_absolute() {
        assert!(!XDG_CONFIG_HOME.as_os_str().is_empty());
    }

    #[test]
    fn xdg_cache_home_is_absolute() {
        assert!(!XDG_CACHE_HOME.as_os_str().is_empty());
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    #[test]
    fn xdg_data_home_includes_local_share() {
        assert!(
            XDG_DATA_HOME.to_string_lossy().contains(".local/share"),
            "unix XDG_DATA_HOME should contain .local/share, got: {}",
            XDG_DATA_HOME.display()
        );
    }

    mod env_vars {
        use super::*;

        #[test]
        fn capture_returns_well_formed_struct() {
            let vars = EnvVars::capture();
            let _ = vars.vault_dir();
            let _ = vars.config_file();
            let _ = vars.cache_dir();
            let _ = vars.ceiling_dirs();
            let _ = vars.suppress_global();
        }

        #[test]
        fn new_returns_struct_with_all_fields_set() {
            let vault = PathBuf::from("/vault");
            let config = PathBuf::from("/config.toml");
            let cache = PathBuf::from("/cache");
            let ceilings = vec![PathBuf::from("/c1"), PathBuf::from("/c2")];

            let vars = EnvVars::new(
                Some(vault.clone()),
                Some(config.clone()),
                Some(cache.clone()),
                Some(ceilings.clone()),
                true,
            );

            assert_eq!(vars.vault_dir(), Some(&vault));
            assert_eq!(vars.config_file(), Some(&config));
            assert_eq!(vars.cache_dir(), Some(&cache));
            assert_eq!(vars.ceiling_dirs(), Some(ceilings.as_slice()));
            assert!(vars.suppress_global());
        }

        #[test]
        fn new_returns_none_fields_when_omitted() {
            let vars = EnvVars::new(None, None, None, None, false);

            assert!(vars.vault_dir().is_none());
            assert!(vars.config_file().is_none());
            assert!(vars.cache_dir().is_none());
            assert!(vars.ceiling_dirs().is_none());
            assert!(!vars.suppress_global());
        }
    }
}
