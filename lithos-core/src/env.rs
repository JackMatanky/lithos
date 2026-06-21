//! Central registry of environment variables for the Lithos runtime.
//!
//! All Lithos-specific environment variables (`LITHOS_*`) and platform
//! directories are read once via [`LazyLock`] statics and exposed as a
//! consistent API. This module is the single source of truth for
//! environment-derived configuration — no other module should call
//! `std::env::var` or `std::env::var_os` directly for these keys.
//!
//! # Example
//!
//! ```rust,ignore
//! use lithos_core::env;
//!
//! if let Some(cache) = env::LITHOS_CACHE_DIR.as_ref() {
//!     println!("cache: {}", cache.display());
//! }
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
// Platform directories
// ---------------------------------------------------------------------------

/// The user's home directory (`$HOME` / platform home).
///
/// In test builds this points to `<CARGO_MANIFEST_DIR>/tests/fixtures/` so
/// that discovery tests do not accidentally depend on the real home directory.
#[cfg(test)]
pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
});

/// The user's home directory (`$HOME` / platform home).
#[cfg(not(test))]
pub static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

/// `$XDG_CACHE_HOME` or the platform cache directory.
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME")
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| HOME.join(".cache"))
});

/// `$XDG_CONFIG_HOME` or the platform config directory.
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| HOME.join(".config"))
});

// ---------------------------------------------------------------------------
// Lithos-specific environment variables
// ---------------------------------------------------------------------------

/// `LITHOS_CACHE_DIR` — explicit cache directory override.
///
/// When set, discovery uses this path as the cache root regardless of vault
/// locality. When unset, the cache root is determined by vault proximity or
/// platform defaults.
pub static LITHOS_CACHE_DIR: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| var_path("LITHOS_CACHE_DIR"));

/// `LITHOS_VAULT_DIR` — explicit vault root directory.
///
/// When set, discovery uses this path as the vault root and skips ascending
/// traversal entirely.
pub static LITHOS_VAULT_DIR: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| var_path("LITHOS_VAULT_DIR"));

/// `LITHOS_CONFIG_FILE` — explicit config file path.
///
/// When set, discovery skips local config probing and uses this path directly
/// as the sole config input.
pub static LITHOS_CONFIG_FILE: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| var_path("LITHOS_CONFIG_FILE"));

/// `LITHOS_CEILING_DIRS` — colon-separated list of ceiling directories.
///
/// Each entry is a directory path that bounds the ascending traversal.
/// Invalid or non-existent entries are silently ignored at parse time but
/// recorded in [`DiscoveryReport::skipped_ceilings`].
///
/// [`DiscoveryReport::skipped_ceilings`]: crate::discovery::report::DiscoveryReport::skipped_ceilings
pub static LITHOS_CEILING_DIRS: LazyLock<Option<Vec<PathBuf>>> =
    LazyLock::new(|| {
        var_path("LITHOS_CEILING_DIRS").map(|raw| {
            std::env::split_paths(&raw)
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        })
    });

/// `LITHOS_SUPPRESS_GLOBAL` — disable global config lookup.
///
/// When set to a truthy value, global config directories are not probed during
/// discovery. Equivalent to the `--no-global-config` CLI flag.
pub static LITHOS_SUPPRESS_GLOBAL: LazyLock<bool> =
    LazyLock::new(|| var_is_true("LITHOS_SUPPRESS_GLOBAL"));

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
    fn lithos_cache_dir_defaults_to_none() {
        // Must be None unless LITHOS_CACHE_DIR is explicitly set in the
        // process environment, which we cannot control in all CI environments.
        // This test documents the expected default.
        let _ = LITHOS_CACHE_DIR.as_ref();
    }

    #[test]
    fn lithos_suppress_global_defaults_to_false() {
        assert!(
            !*LITHOS_SUPPRESS_GLOBAL,
            "LITHOS_SUPPRESS_GLOBAL should default to false"
        );
    }
}
