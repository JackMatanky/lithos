//! Config-side root handoff types.
//!
//! Receives [`crate::discovery::FoundRootMarker`] output from the Discovery
//! context and classifies it into [`DiscoveredConfigFile`] with the appropriate
//! [`crate::config::location::LocalConfigLocation`] variant.
//!
//! [`ConfigDiscoveryResult`] aggregates both local and global classified files
//! and will be consumed by [`crate::config::discovery`] as its input contract.
//! It lives here until the global classification side is built in Phase 2.

use std::path::PathBuf;

use crate::{
    config::location::ConfigLocation, discovery::diagnostics::DiscoveryWarning,
    fs::format::StructuredFileFormat,
};

/// Typed representation of one discovered config file with location metadata.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DiscoveredConfigFile {
    /// Source location classification (e.g. `SystemConfig`,
    /// `HiddenRootConfigFile`).
    pub(crate) location: ConfigLocation,
    /// Base directory used for interpreting relative paths in the config file.
    ///
    /// For local vault configs, this is the vault root.
    pub(crate) base: PathBuf,
    /// Absolute canonicalized path to the discovered config file.
    pub(crate) path: PathBuf,
    /// The detected or assumed structured format of the file.
    pub(crate) format: StructuredFileFormat,
}

/// Combined discovery output for global + local config files.
///
/// `None` for `global` or `local` represents an absent file, not an error.
#[allow(
    dead_code,
    reason = "Phase-1 contract; wired in once pipeline integration lands"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConfigDiscoveryResult {
    /// Selected global environment config candidate, if any.
    pub(crate) global: Option<DiscoveredConfigFile>,
    /// Selected local vault config candidate, if any.
    pub(crate) local: Option<DiscoveredConfigFile>,
    /// Non-fatal diagnostics collected during the discovery process.
    pub(crate) warnings: Vec<DiscoveryWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::location::{GlobalConfigLocation, LocalConfigLocation},
        discovery::diagnostics::LocalDiscoveryWarning,
    };

    mod discovered_config_file {
        use super::*;

        #[test]
        fn returns_true_when_files_have_same_location_path_and_format() {
            let a = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(a, b);
        }

        #[test]
        fn returns_false_when_locations_differ() {
            let a = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::HiddenRootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_ne!(a, b);
        }
    }

    mod config_discovery_result {
        use super::*;

        fn local_file() -> DiscoveredConfigFile {
            DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::ConfigDirectoryFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/config.toml"),
                format: StructuredFileFormat::Toml,
            }
        }

        #[test]
        fn returns_true_when_results_match() {
            let warning =
                DiscoveryWarning::Local(LocalDiscoveryWarning::Ambiguity {
                    candidates: vec![PathBuf::from("/vault/lithos.toml")],
                });
            let a = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: vec![warning],
            };
            let b = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: vec![DiscoveryWarning::Local(
                    LocalDiscoveryWarning::Ambiguity {
                        candidates: vec![PathBuf::from("/vault/lithos.toml")],
                    },
                )],
            };
            assert_eq!(a, b);
        }

        #[test]
        fn returns_false_when_local_files_differ() {
            let with_local = ConfigDiscoveryResult {
                global: None,
                local: Some(local_file()),
                warnings: Vec::new(),
            };
            let without_local = ConfigDiscoveryResult {
                global: None,
                local: None,
                warnings: Vec::new(),
            };
            assert_ne!(with_local, without_local);
        }
    }

    mod config_location {
        use super::*;

        #[test]
        fn returns_false_when_global_and_local_variants_compared() {
            let global =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let local = ConfigLocation::Local(
                LocalConfigLocation::HiddenRootConfigFile,
            );
            assert_ne!(global, local);
        }

        #[test]
        fn returns_true_when_global_locations_match() {
            let first = ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let second =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            assert_eq!(first, second);
        }
    }
}
