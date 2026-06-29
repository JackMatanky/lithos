//! Platform directories (`HOME`, `XDG_*`).
//!
//! Platform directories are resolved as lazy statics with per-platform
//! fallbacks.

use std::{path::PathBuf, sync::LazyLock};

fn var_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
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
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

/// `$XDG_DATA_HOME` or the platform data directory.
#[cfg(windows)]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

/// `$XDG_DATA_HOME` or the platform data directory.
#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_DATA_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("share"))
});

// ---------------------------------------------------------------------------

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(target_os = "macos")]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join("Library/Application Support"))
});

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(windows)]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .or_else(|| var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});

/// `$XDG_STATE_HOME` or the platform state directory.
#[cfg(not(any(target_os = "macos", windows)))]
pub static XDG_STATE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    var_path("XDG_STATE_HOME")
        .unwrap_or_else(|| HOME.join(".local").join("state"))
});

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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
}
