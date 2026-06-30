//! Central registry of old discovery environment variables.
//!
//! All `TRACES_*` environment variables are read once through [`EnvVars`] and
//! exposed as a consistent API.
//!
//! No other module should call `std::env::var` or `std::env::var_os` directly.
//!
//! # Example
//!
//! ```rust,ignore
//! use traces_settings::config::EnvVars;
//!
//! let vars = EnvVars::capture();
//! ```

use std::path::PathBuf;

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
// EnvVars
// ---------------------------------------------------------------------------

/// Captured Traces environment variables.
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
    /// Read all TRACES_* env vars from the process environment.
    #[inline]
    #[must_use]
    pub fn capture() -> Self {
        Self {
            vault_dir: var_path("TRACES_VAULT_DIR"),
            config_file: var_path("TRACES_CONFIG_FILE"),
            cache_dir: var_path("TRACES_CACHE_DIR"),
            ceiling_dirs: var_path("TRACES_CEILING_DIRS").map(|raw| {
                std::env::split_paths(&raw)
                    .filter(|p| !p.as_os_str().is_empty())
                    .collect()
            }),
            suppress_global: var_is_true("TRACES_SUPPRESS_GLOBAL"),
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

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

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
