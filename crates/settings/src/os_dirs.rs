//! Raw OS-level platform directories.
//!
//! These are the foundational directories (`HOME`, `XDG_*`, `/etc`) resolved
//! from environment variables with platform-specific fallbacks. They carry
//! **no application suffix** — the `traces` subdirectory is appended in
//! [`crate::dirs`].
//!
//! ## Design
//!
//! | Layer | Module | Example |
//! |-------|--------|---------|
//! | OS platform | `os_dirs` | `~/.config` |
//! | Application | [`crate::dirs`] | `~/.config/traces` |

use std::{path::PathBuf, sync::LazyLock};

fn var_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

// -- HOME -------------------------------------------------------------------

/// The user's home directory.
///
/// Resolves via the `dirs` crate, falling back to `/` when even that fails.
///
/// In test builds this is redirected to `<CARGO_MANIFEST_DIR>/tests/fixtures/`
/// so discovery tests don't depend on the real home directory
/// (see [`HOME`](self::HOME#test-redirect) under test).
#[cfg(not(test))]
pub static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

/// The user's home directory — test fixture redirect.
///
/// Points to `<CARGO_MANIFEST_DIR>/tests/fixtures/` so discovery tests
/// do not accidentally depend on the real home directory.
#[cfg(test)]
pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
});

// -- XDG config -------------------------------------------------------------

/// `$XDG_CONFIG_HOME` — user-specific configuration files.
///
/// macOS:    `~/Library/Application Support`
/// Windows:  `$APPDATA` → `~/AppData/Roaming`
/// Other:   `~/.config`
#[cfg(target_os = "macos")]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

#[cfg(windows)]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME")
        .or_else(|| var_path("APPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Roaming"))
});

#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CONFIG_HOME").unwrap_or_else(|| HOME.join(".config"))
});

// -- XDG cache --------------------------------------------------------------

/// `$XDG_CACHE_HOME` — user-specific non-essential data files.
///
/// macOS:    `~/Library/Caches`
/// Windows:  `$TEMP` → `~/AppData/Local/Temp`
/// Other:   `~/.cache`
#[cfg(target_os = "macos")]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join("Library/Caches"))
});

#[cfg(windows)]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME")
        .or_else(|| var_path("TEMP"))
        .unwrap_or_else(|| HOME.join("AppData/Local/Temp"))
});

#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_CACHE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join(".cache"))
});

// -- XDG data ---------------------------------------------------------------

/// `$XDG_DATA_HOME` — user-specific data files.
///
/// macOS:    `~/Library/Application Support`
/// Windows:  `$LOCALAPPDATA` → `~/AppData/Local`
/// Other:   `~/.local/share`
#[cfg(target_os = "macos")]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

#[cfg(windows)]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("share"))
});

// -- XDG state --------------------------------------------------------------

/// `$XDG_STATE_HOME` — user-specific state data.
///
/// macOS:    `~/Library/Application Support`
/// Windows:  `$LOCALAPPDATA` → `~/AppData/Local`
/// Other:   `~/.local/state`
#[cfg(target_os = "macos")]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

#[cfg(windows)]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("state"))
});

// -- System config ----------------------------------------------------------

/// System-wide config directory.
///
/// Only available on Unix (`/etc`). Used as a fallback for global config
/// discovery when no user-level config exists.
#[cfg(unix)]
pub static SYSTEM_CONFIG_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/etc"));

// -- Tests ------------------------------------------------------------------

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

    #[cfg(unix)]
    #[test]
    fn system_config_dir_is_etc() {
        assert_eq!(
            SYSTEM_CONFIG_DIR.as_path(),
            std::path::Path::new("/etc"),
            "unix SYSTEM_CONFIG_DIR should be /etc, got: {}",
            SYSTEM_CONFIG_DIR.display()
        );
    }
}
