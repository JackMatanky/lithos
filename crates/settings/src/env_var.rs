//! Traces environment variables.

use std::path::PathBuf;

const DEFAULT_VAULT_KEY: &str = "TRACES_DEFAULT_VAULT";
const GLOBAL_CONFIG_KEY: &str = "TRACES_GLOBAL_CONFIG";

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

/// Captured Traces environment variables.
///
/// Pure capture — no fallbacks, no platform logic, no filesystem validation.
/// Construct via [`SettingsEnvVars::capture()`] to read from the real
/// environment, or via [`SettingsEnvVars::new()`] for deterministic test
/// construction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SettingsEnvVars {
    default_vault_dir: Option<PathBuf>,
    global_config: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    ceiling_dirs: Option<Vec<PathBuf>>,
    suppress_global: bool,
}

impl SettingsEnvVars {
    /// Read all TRACES_* env vars from the process environment.
    #[inline]
    #[must_use]
    pub fn capture() -> Self {
        Self {
            default_vault_dir: var_path(DEFAULT_VAULT_KEY),
            global_config: var_path(GLOBAL_CONFIG_KEY),
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
        default_vault_dir: Option<PathBuf>,
        global_config: Option<PathBuf>,
        cache_dir: Option<PathBuf>,
        ceiling_dirs: Option<Vec<PathBuf>>,
        suppress_global: bool,
    ) -> Self {
        Self {
            default_vault_dir,
            global_config,
            cache_dir,
            ceiling_dirs,
            suppress_global,
        }
    }

    /// Default vault root fallback used when local traversal finds nothing.
    #[inline]
    #[must_use]
    pub fn default_vault_dir(&self) -> Option<&PathBuf> {
        self.default_vault_dir.as_ref()
    }

    /// Explicit global config file path.
    #[inline]
    #[must_use]
    pub fn global_config(&self) -> Option<&PathBuf> {
        self.global_config.as_ref()
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

    mod capture {
        use super::*;

        #[test]
        fn uses_new_env_keys() {
            assert_eq!(DEFAULT_VAULT_KEY, "TRACES_DEFAULT_VAULT");
            assert_eq!(GLOBAL_CONFIG_KEY, "TRACES_GLOBAL_CONFIG");
        }

        #[test]
        fn capture_returns_well_formed_struct() {
            let vars = SettingsEnvVars::capture();
            let _ = vars.default_vault_dir();
            let _ = vars.global_config();
            let _ = vars.cache_dir();
            let _ = vars.ceiling_dirs();
            let _ = vars.suppress_global();
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn new_returns_struct_with_all_fields_set() {
            let vault = PathBuf::from("/vault");
            let config = PathBuf::from("/config.toml");
            let cache = PathBuf::from("/cache");
            let ceilings = vec![PathBuf::from("/c1"), PathBuf::from("/c2")];

            let vars = SettingsEnvVars::new(
                Some(vault.clone()),
                Some(config.clone()),
                Some(cache.clone()),
                Some(ceilings.clone()),
                true,
            );

            assert_eq!(vars.default_vault_dir(), Some(&vault));
            assert_eq!(vars.global_config(), Some(&config));
            assert_eq!(vars.cache_dir(), Some(&cache));
            assert_eq!(vars.ceiling_dirs(), Some(ceilings.as_slice()));
            assert!(vars.suppress_global());
        }

        #[test]
        fn new_returns_none_fields_when_omitted() {
            let vars = SettingsEnvVars::new(None, None, None, None, false);

            assert!(vars.default_vault_dir().is_none());
            assert!(vars.global_config().is_none());
            assert!(vars.cache_dir().is_none());
            assert!(vars.ceiling_dirs().is_none());
            assert!(!vars.suppress_global());
        }
    }
}
