//! Architecture tests to enforce design patterns and constraints.

#[cfg(test)]
mod tests {
    use std::fs;

    use glob::glob;

    #[test]
    fn ports_must_not_import_std_fs() {
        // We search from the project root, but the test runs in the crate root.
        // Paths are relative to the crate root (lithos-core).
        let port_files =
            glob("src/**/ports.rs").expect("Failed to read glob pattern");

        for entry in port_files {
            let path = entry.expect("Glob entry error");
            let content =
                fs::read_to_string(&path).expect("Failed to read port file");

            assert!(
                !content.contains("std::fs"),
                "Port file {path:?} must not import std::fs. Domain ports \
                 must remain pure and storage-agnostic."
            );

            assert!(
                !content.contains("use std::path::PathBuf"),
                "Port file {path:?} must not use PathBuf in imports. Use \
                 &Path for arguments if necessary, but prefer database-native \
                 identifiers."
            );
        }
    }

    #[test]
    fn port_traits_must_not_have_file_io_methods() {
        let port_files =
            glob("src/**/ports.rs").expect("Failed to read glob pattern");

        for entry in port_files {
            let path = entry.expect("Glob entry error");
            let content =
                fs::read_to_string(&path).expect("Failed to read port file");

            assert!(
                !content.contains("fn load_from_file"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'load_from_file'. Use Application Services + FileReader \
                 instead."
            );

            assert!(
                !content.contains("fn scan_directory"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'scan_directory'. Use Application Services + FileReader \
                 instead."
            );

            assert!(
                !content.contains("fn write_to_file"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'write_to_file'. Domain ports handle persistence, not \
                 filesystem directly."
            );
        }
    }

    #[test]
    fn contexts_must_not_import_each_other() {
        let contexts = ["config", "note", "schema", "template"];

        for &ctx in &contexts {
            let pattern = format!("src/{ctx}/**/*.rs");
            let files = glob(&pattern).expect("Failed to read glob pattern");

            for entry in files {
                let path = entry.expect("Glob entry error");
                let content =
                    fs::read_to_string(&path).expect("Failed to read file");

                check_imports(ctx, &content, &path, &contexts);
            }
        }
    }

    #[test]
    fn config_must_not_import_discovery_diagnostics_or_source_policy() {
        let files = glob("src/config/**/*.rs").expect("read config glob");

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
            !std::path::Path::new("src/config/candidates.rs").exists(),
            "Config must not own filesystem candidate discovery"
        );
        assert!(
            !std::path::Path::new("src/config/location.rs").exists(),
            "Config must not own source/location taxonomy for discovered paths"
        );
    }

    #[test]
    fn builder_must_not_hardcode_system_global_config_path() {
        let content = fs::read_to_string("src/config/builder.rs")
            .expect("read config builder");

        assert!(
            !content.contains("/etc/lithos"),
            "Builder must not hardcode partial global discovery runtime paths"
        );
    }

    #[test]
    fn builder_must_not_use_known_vault_root_discovery_shortcut() {
        let content = fs::read_to_string("src/config/builder.rs")
            .expect("read config builder");

        assert!(
            !content.contains("find_known_vault"),
            "Builder must not use a known-root shortcut that bypasses \
             DiscoveryService's structural invariants"
        );
    }

    #[test]
    fn builder_imports_only_discovery_service_result_from_discovery() {
        let content = fs::read_to_string("src/config/builder.rs")
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

    fn check_imports(
        ctx: &str,
        content: &str,
        path: &std::path::Path,
        contexts: &[&str],
    ) {
        for &other in contexts {
            if ctx == other || other == "config" {
                continue;
            }

            let import_pattern = format!("crate::{other}");
            assert!(
                !content.contains(&import_pattern),
                "Context '{ctx}' (file {path:?}) must not import context \
                 '{other}'. Contexts must be isolated."
            );
        }
    }
}
