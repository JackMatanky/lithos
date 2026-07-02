//! Configuration location constants.

use std::{path::PathBuf, sync::LazyLock};

use crate::os_dirs::STATE;

/// Path to the directory for tracked config symlinks.
pub static TRACKED_CONFIGS: LazyLock<PathBuf> =
    LazyLock::new(|| STATE.join("tracked-configs"));

/// Path to the directory for trusted config symlinks.
pub static TRUSTED_CONFIGS: LazyLock<PathBuf> =
    LazyLock::new(|| STATE.join("trusted-configs"));

/// Path to the directory for ignored config symlinks.
pub static IGNORED_CONFIGS: LazyLock<PathBuf> =
    LazyLock::new(|| STATE.join("ignored-configs"));

/// Vault-relative cache directory.
pub const CACHE_SUBDIR: &str = ".traces/cache";

#[cfg(test)]
mod tests {
    use super::*;

    mod constants {
        use super::*;

        #[test]
        fn path_constants_are_relative() {
            assert!(!CACHE_SUBDIR.is_empty());
            assert!(!CACHE_SUBDIR.starts_with('/'));
        }

        #[test]
        fn lazy_paths_are_derived_from_state() {
            assert!(TRACKED_CONFIGS.ends_with("tracked-configs"));
            assert!(TRUSTED_CONFIGS.ends_with("trusted-configs"));
            assert!(IGNORED_CONFIGS.ends_with("ignored-configs"));
        }
    }
}
