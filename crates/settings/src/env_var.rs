//! Traces environment variables.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

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
pub(crate) struct SettingsEnvVars {
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
    pub(crate) fn capture() -> Self {
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
            // Raw split only — empty segments are kept so discovery can
            // report them as skipped ceilings. Normalization and validation
            // are policy, and live in the discovery input layer.
            ceiling_dirs: var_os("TRACES_CEILING_DIRS")
                .map(|raw| std::env::split_paths(&raw).collect()),
            suppress_global: var("TRACES_SUPPRESS_GLOBAL")
                .as_deref()
                .is_some_and(is_true),
        }
    }

    /// Pure constructor for tests and custom setups.
    #[cfg(any(test, feature = "testing"))]
    #[allow(
        dead_code,
        reason = "feature-gated constructor is used by internal test helpers"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
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
    pub(crate) fn default_vault_dir(&self) -> Option<&Path> {
        self.default_vault_dir.as_deref()
    }

    /// Explicit global config file path.
    #[inline]
    #[must_use]
    pub(crate) fn global_config(&self) -> Option<&Path> {
        self.global_config.as_deref()
    }

    /// Explicit cache directory override.
    #[allow(
        dead_code,
        reason = "cache env override is exposed through AppDirs public helpers"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn cache_dir(&self) -> Option<&Path> {
        self.cache_dir.as_deref()
    }

    /// Colon-separated ceiling directory paths (raw from env).
    #[inline]
    #[must_use]
    pub(crate) fn ceiling_dirs(&self) -> Option<&[PathBuf]> {
        self.ceiling_dirs.as_deref()
    }

    /// Whether global config lookup is suppressed.
    #[inline]
    #[must_use]
    pub(crate) fn suppress_global(&self) -> bool {
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
        use pretty_assertions::assert_eq;

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

            assert_eq!(vars.default_vault_dir(), Some(Path::new("/vault")));
            assert_eq!(vars.global_config(), Some(Path::new("/global.toml")));
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
        use pretty_assertions::assert_eq;

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

            assert_eq!(vars.default_vault_dir(), Some(vault.as_path()));
            assert_eq!(vars.global_config(), Some(config.as_path()));
            assert_eq!(vars.cache_dir(), Some(cache.as_path()));
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
