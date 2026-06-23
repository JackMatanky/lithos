//! Architecture tests to enforce design patterns and constraints inside
//! settings.

#[cfg(test)]
mod tests {
    use std::fs;

    use glob::glob;

    #[test]
    fn config_must_not_import_discovery_diagnostics_or_source_policy() {
        let files = glob("../../crates/settings/src/config/**/*.rs")
            .expect("read config glob");

        for entry in files {
            let path = entry.expect("Glob entry error");
            let content = fs::read_to_string(&path).expect("read config file");

            assert!(
                !content.contains("discovery::diagnostics"),
                "Config file {path:?} must not import Discovery diagnostics"
            );
            assert!(
                !content.contains("GlobalSourceType")
                    && !content.contains("VaultSourceType")
                    && !content.contains("GlobalSourceDirectory"),
                "Config file {path:?} must not import or mention Discovery \
                 source identity"
            );
        }
    }

    #[test]
    fn config_must_not_own_filesystem_candidate_discovery_modules() {
        assert!(
            std::path::Path::new("../settings/src/discovery/location.rs")
                .exists(),
            "location.rs must exist in discovery, not config"
        );
        assert!(
            !std::path::Path::new("../settings/src/config/candidates.rs")
                .exists(),
            "Config must not own filesystem candidate discovery"
        );
        assert!(
            !std::path::Path::new("../settings/src/config/location.rs")
                .exists(),
            "Config must not own source/location taxonomy for discovered paths"
        );
    }

    #[test]
    fn builder_must_not_hardcode_system_global_config_path() {
        let content = fs::read_to_string("../settings/src/config/builder.rs")
            .expect("read config builder");

        assert!(
            !content.contains("/etc/lithos"),
            "Builder must not hardcode partial global discovery runtime paths"
        );
    }

    #[test]
    fn builder_must_not_use_known_vault_root_discovery_shortcut() {
        let content = fs::read_to_string("../settings/src/config/builder.rs")
            .expect("read config builder");

        assert!(
            !content.contains("find_known_vault"),
            "Builder must not use a known-root shortcut that bypasses \
             DiscoveryService's structural invariants"
        );
    }

    #[test]
    fn builder_imports_only_discovery_service_result_from_discovery() {
        let content = fs::read_to_string("../settings/src/config/builder.rs")
            .expect("read config builder");

        assert!(
            !content.contains("discovery::engine"),
            "config/builder.rs must not import from discovery::engine"
        );
        assert!(
            !content.contains("discovery::policy"),
            "config/builder.rs must not import from discovery::policy"
        );
        assert!(
            !content.contains("DiscoveryEngine"),
            "config/builder.rs must not use DiscoveryEngine"
        );
        assert!(
            !content.contains("DiscoveryInput"),
            "config/builder.rs must not use DiscoveryInput"
        );
        assert!(
            !content.contains("GlobalDiscoveryInput"),
            "config/builder.rs must not use GlobalDiscoveryInput"
        );
        assert!(
            !content.contains("DiscoveryPolicy"),
            "config/builder.rs must not use DiscoveryPolicy"
        );
        assert!(
            content.contains("discovery::service::DiscoveryResult")
                || content.contains(
                    "discovery::service::{CandidatePath, DiscoveryResult}"
                ),
            "config/builder.rs must import DiscoveryResult from \
             discovery::service only"
        );
    }
}
