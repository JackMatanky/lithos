//! Discovery result contracts shared by config discovery phases.
//!
//! This module defines typed outcomes and warnings transported by discovery.
//! It intentionally excludes traversal, filesystem probing, and precedence
//! execution logic.

use std::path::PathBuf;

use super::{diagnostics::DiscoveryWarning, location::ConfigLocation};
use crate::fs::format::StructuredFileFormat;

/// Typed representation of one discovered config file.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
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
    reason = "Phase-1 contracts are defined before full pipeline integration"
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

/// Result of selecting a single candidate from multiple discovered formats.
///
/// Provides a strongly-typed outcome of the deterministic selection logic,
/// carrying both the winner and any ambiguity warning generated.
#[allow(
    dead_code,
    reason = "Phase-2 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConfigSelectionResult {
    /// The final selected config candidate.
    pub(crate) candidate: DiscoveredConfigFile,
    /// A structured warning if multiple formats were available.
    pub(crate) warning: Option<DiscoveryWarning>,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            diagnostics::{DiscoveryWarning, LocalDiscoveryWarning},
            location::{GlobalConfigLocation, LocalConfigLocation},
        },
        *,
    };

    mod validation {
        use super::*;

        #[test]
        fn returns_local_location_when_config_location_is_not_global() {
            let global =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let local = ConfigLocation::Local(
                LocalConfigLocation::HiddenRootConfigFile,
            );
            assert_ne!(global, local);
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_base_path_when_constructing_discovered_config_file() {
            let discovered = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::HiddenRootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(discovered.base, PathBuf::from("/vault"));
        }

        #[test]
        fn returns_warning_list_when_constructing_config_discovery_result() {
            let result = ConfigDiscoveryResult {
                global: None,
                local: None,
                warnings: vec![DiscoveryWarning::Local(
                    LocalDiscoveryWarning::Ambiguity {
                        candidates: vec![PathBuf::from("/vault/lithos.toml")],
                    },
                )],
            };
            assert_eq!(result.warnings.len(), 1);
        }

        #[test]
        fn returns_global_and_local_when_constructing_config_discovery_result_with_discovered_files()
         {
            let global = DiscoveredConfigFile {
                location: ConfigLocation::Global(
                    GlobalConfigLocation::EnvironmentOverride(PathBuf::from(
                        "/env/lithos.toml",
                    )),
                ),
                base: PathBuf::from("/env"),
                path: PathBuf::from("/env/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let local = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::ConfigDirectoryFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/config.toml"),
                format: StructuredFileFormat::Toml,
            };

            let result = ConfigDiscoveryResult {
                global: Some(global),
                local: Some(local),
                warnings: Vec::new(),
            };

            assert!(result.global.is_some());
            assert!(result.local.is_some());
        }

        #[test]
        fn returns_candidate_when_constructing_config_selection_result() {
            let candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::ConfigDirectoryFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos/config.toml"),
                format: StructuredFileFormat::Toml,
            };

            let result = ConfigSelectionResult {
                candidate,
                warning: None,
            };

            assert!(result.warning.is_none());
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn returns_true_when_comparing_equal_global_config_locations() {
            let first = ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let second =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            assert_eq!(first, second);
        }
    }
}
