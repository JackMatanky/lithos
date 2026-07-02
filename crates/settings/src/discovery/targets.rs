//! Exact marker filenames for configuration discovery targets.

/// Exact vault-local marker filenames.
pub const VAULT_CONFIG_TARGETS: &[&str] = &[
    "traces.toml",
    "traces.json",
    "traces.yaml",
    "traces.yml",
    ".traces/config.toml",
    ".traces/config.json",
    ".traces/config.yaml",
    ".traces/config.yml",
];

/// Exact global config marker filenames.
///
/// Global config lives under a platform config directory (e.g.
/// `$XDG_CONFIG_HOME`), so targets are `traces.*` and `traces/config.*`. The
/// vault-only `.traces/config.*` forms are intentionally excluded here.
pub const GLOBAL_CONFIG_TARGETS: &[&str] = &[
    "traces.toml",
    "traces.json",
    "traces.yaml",
    "traces.yml",
    "traces/config.toml",
    "traces/config.json",
    "traces/config.yaml",
    "traces/config.yml",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_sets_are_non_empty() {
        assert!(!VAULT_CONFIG_TARGETS.is_empty());
        assert!(!GLOBAL_CONFIG_TARGETS.is_empty());
    }

    #[test]
    fn local_markers_include_exact_nested_and_yml_names() {
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
            assert!(VAULT_CONFIG_TARGETS.contains(&marker), "missing {marker}");
        }
    }

    #[test]
    fn global_targets_use_traces_config_without_dot_prefix_and_bare_names() {
        for marker in [
            "traces.toml",
            "traces.json",
            "traces.yaml",
            "traces.yml",
            "traces/config.toml",
            "traces/config.json",
            "traces/config.yaml",
            "traces/config.yml",
        ] {
            assert!(
                GLOBAL_CONFIG_TARGETS.contains(&marker),
                "missing global {marker}"
            );
        }
    }

    #[test]
    fn global_targets_exclude_dot_traces() {
        for marker in [
            ".traces/config.toml",
            ".traces/config.json",
            ".traces/config.yaml",
            ".traces/config.yml",
        ] {
            assert!(
                !GLOBAL_CONFIG_TARGETS.contains(&marker),
                "global targets should not include {marker}"
            );
        }
    }
}
