//! Configuration location constants.

/// Exact marker filenames.
pub const MARKERS: &[&str] = &[
    "traces.toml",
    "traces.json",
    "traces.yaml",
    "traces.yml",
    ".traces/config.toml",
    ".traces/config.json",
    ".traces/config.yaml",
    ".traces/config.yml",
];

/// Directory names that stop local ancestor discovery.
pub const BOUNDARY_MARKERS: &[&str] = &[".git", ".workspace"];

/// Exact global config filenames.
pub const GLOBAL_CONFIG_TARGETS: &[&str] = MARKERS;

/// Relative subdirectory for tracked config symlinks.
pub const TRACKED_CONFIGS: &str = "TRACKED_CONFIGS";

/// Relative subdirectory for trusted config symlinks.
pub const TRUSTED_CONFIGS: &str = "TRUSTED_CONFIGS";

/// Relative subdirectory for ignored config symlinks.
pub const IGNORED_CONFIGS: &str = "IGNORED_CONFIGS";

/// Vault-relative cache directory.
pub const CACHE_SUBDIR: &str = ".traces/cache";

#[cfg(test)]
mod tests {
    use super::*;

    mod constants {
        use super::*;

        #[test]
        fn marker_sets_are_non_empty() {
            assert!(!MARKERS.is_empty());
            assert!(!BOUNDARY_MARKERS.is_empty());
            assert!(!GLOBAL_CONFIG_TARGETS.is_empty());
        }

        #[test]
        fn marker_sets_include_exact_nested_and_yml_names() {
            for marker in [
                "traces.toml",
                "traces.json",
                "traces.yaml",
                "traces.yml",
                ".traces/config.toml",
                ".traces/config.json",
                ".traces/config.yaml",
                ".traces/config.yml",
            ] {
                assert!(MARKERS.contains(&marker), "missing {marker}");
                assert!(
                    GLOBAL_CONFIG_TARGETS.contains(&marker),
                    "missing global {marker}"
                );
            }
        }

        #[test]
        fn path_constants_are_relative() {
            for path in [
                TRACKED_CONFIGS,
                TRUSTED_CONFIGS,
                IGNORED_CONFIGS,
                CACHE_SUBDIR,
            ] {
                assert!(!path.is_empty());
                assert!(!path.starts_with('/'));
            }
        }
    }
}
