//! Resolved platform and application directories for Lithos.
//!
//! Each static combines environment variable overrides with platform defaults
//! to produce the final directories used by discovery and the rest of the
//! system. Prefer importing from this module over calling `dirs::*` directly
//! or hardcoding platform-specific paths.
//!
//! # Example
//!
//! ```rust,ignore
//! use lithos_core::dirs;
//!
//! let cache = dirs::CACHE.as_path();
//! ```

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::env;

/// The user's home directory.
///
/// Delegates to [`env::HOME`].
pub static HOME: LazyLock<&Path> = LazyLock::new(|| &env::HOME);

/// Resolved Lithos cache directory.
///
/// Precedence:
/// 1. `$LITHOS_CACHE_DIR` when set.
/// 2. `<platform cache dir>/lithos/` (derived from [`env::XDG_CACHE_HOME`]).
///
/// This is the **platform-level** cache directory. When a vault root is
/// available at runtime, the cache may be placed at `<vault>/.lithos/cache/`
/// instead — see [`crate::discovery::processor::resolve_cache_root`] for
/// the full precedence logic.
pub static CACHE: LazyLock<PathBuf> = LazyLock::new(|| {
    env::LITHOS_CACHE_DIR
        .clone()
        .unwrap_or_else(|| env::XDG_CACHE_HOME.join("lithos"))
});

/// Resolved Lithos global config directory.
///
/// Precedence:
/// 1. `$XDG_CONFIG_HOME/lithos/` when `$XDG_CONFIG_HOME` is set.
/// 2. `<platform config dir>/lithos/` (derived from [`env::XDG_CONFIG_HOME`]).
pub static CONFIG: LazyLock<PathBuf> =
    LazyLock::new(|| env::XDG_CONFIG_HOME.join("lithos"));

/// System-level Lithos config directory.
///
/// - Unix: `/etc/lithos/`
#[cfg(unix)]
pub static SYSTEM_CONFIG: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/etc/lithos"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_matches_env_home() {
        assert_eq!(*HOME, &*env::HOME);
    }

    #[test]
    fn cache_defaults_to_xdg_cache_home_plus_lithos() {
        let expected = env::XDG_CACHE_HOME.join("lithos");
        let actual = &*CACHE;
        // When LITHOS_CACHE_DIR is not set, CACHE should match the XDG
        // fallback.
        if env::LITHOS_CACHE_DIR.is_none() {
            assert_eq!(*actual, expected);
        }
    }

    #[test]
    fn config_defaults_to_xdg_config_home_plus_lithos() {
        let expected = env::XDG_CONFIG_HOME.join("lithos");
        assert_eq!(*CONFIG, expected);
    }

    #[cfg(unix)]
    #[test]
    fn system_config_is_etc_lithos() {
        assert_eq!(*SYSTEM_CONFIG, PathBuf::from("/etc/lithos"));
    }
}
