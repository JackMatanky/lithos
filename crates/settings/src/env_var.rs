//! Traces environment variables.

use std::{ffi::OsString, path::PathBuf};

const DEFAULT_VAULT_KEY: &str = "TRACES_DEFAULT_VAULT";
const GLOBAL_CONFIG_KEY: &str = "TRACES_GLOBAL_CONFIG";

fn is_true(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "y" | "yes" | "true" | "1" | "on")
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
        Self::capture_from(
            |key| std::env::var_os(key),
            |key| std::env::var(key).ok(),
        )
    }

    fn capture_from(
        var_os: impl Fn(&str) -> Option<OsString>,
        var: impl Fn(&str) -> Option<String>,
    ) -> Self {
        Self {
            default_vault_dir: var_os(DEFAULT_VAULT_KEY).map(PathBuf::from),
            global_config: var_os(GLOBAL_CONFIG_KEY).map(PathBuf::from),
            cache_dir: var_os("TRACES_CACHE_DIR").map(PathBuf::from),
            ceiling_dirs: var_os("TRACES_CEILING_DIRS").map(|raw| {
                std::env::split_paths(&raw)
                    .filter(|p| !p.as_os_str().is_empty())
                    .collect()
            }),
            suppress_global: var("TRACES_SUPPRESS_GLOBAL")
                .as_deref()
                .is_some_and(is_true),
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
        fn reads_new_env_keys() {
            let vars = SettingsEnvVars::capture_from(
                |key| match key {
                    "TRACES_DEFAULT_VAULT" => Some("/vault".into()),
                    "TRACES_GLOBAL_CONFIG" => Some("/global.toml".into()),
                    _ => None,
                },
                |_| None,
            );

            assert_eq!(
                vars.default_vault_dir(),
                Some(&PathBuf::from("/vault"))
            );
            assert_eq!(
                vars.global_config(),
                Some(&PathBuf::from("/global.toml"))
            );
        }

        #[test]
        fn ignores_old_env_keys() {
            let vars = SettingsEnvVars::capture_from(
                |key| match key {
                    "TRACES_VAULT_DIR" => Some("/old-vault".into()),
                    "TRACES_CONFIG_FILE" => Some("/old-global.toml".into()),
                    _ => None,
                },
                |_| None,
            );

            assert!(vars.default_vault_dir().is_none());
            assert!(vars.global_config().is_none());
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
