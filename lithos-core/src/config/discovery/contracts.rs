//! Discovery result contracts shared by config discovery phases.
//!
//! This module defines typed outcomes and warnings transported by discovery.
//! It intentionally excludes traversal, filesystem probing, and precedence
//! execution logic.

use std::path::PathBuf;

use super::location::ConfigLocation;
use crate::fs::format::StructuredFileFormat;

/// Source-annotated vault root resolution outcome.
///
/// This value records which strategy produced the effective vault root, or
/// that no root could be resolved.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VaultRootResolution {
    /// Vault root came from an explicit `--vault` CLI flag.
    ExplicitFlag,
    /// Vault root came from `LITHOS_VAULT_PATH`.
    EnvironmentVariable,
    /// Vault root came from an upward directory walk.
    AscendingDiscovery,
    /// No root source matched.
    NotFound,
}

/// Typed representation of one discovered config file.
///
/// `base` carries the directory context used for relative path interpretation
/// in downstream config processing.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DiscoveredConfigFile {
    /// Source location classification.
    pub(crate) location: ConfigLocation,
    /// Base directory used for diagnostics/resolution context.
    pub(crate) base: PathBuf,
    /// Discovered config file path.
    pub(crate) path: PathBuf,
    /// Parsed structured format.
    pub(crate) format: StructuredFileFormat,
}

/// Typed warning channel for non-fatal discovery diagnostics.
///
/// Warnings are structured so CLI/reporting layers can render deterministic,
/// actionable diagnostics without parsing free-form strings.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DiscoveryWarning {
    /// Multiple local candidates were present.
    LocalAmbiguity {
        candidates: Vec<PathBuf>,
    },
    /// Multiple formats were present for one logical location.
    FormatAmbiguity {
        base: PathBuf,
        candidates: Vec<PathBuf>,
    },
    /// Path casing was corrected during discovery.
    CaseCorrection {
        requested: PathBuf,
        resolved: PathBuf,
    },
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
    /// Selected global config candidate.
    pub(crate) global: Option<DiscoveredConfigFile>,
    /// Selected local (vault) config candidate.
    pub(crate) local: Option<DiscoveredConfigFile>,
    /// Non-fatal diagnostics collected during discovery.
    pub(crate) warnings: Vec<DiscoveryWarning>,
}

#[cfg(test)]
mod tests {
    use super::{
        super::location::{GlobalConfigLocation, LocalConfigLocation},
        *,
    };

    mod validation {
        use super::*;

        #[test]
        fn returns_four_variants_when_enumerating_vault_root_resolution_outcomes()
         {
            let variants = [
                VaultRootResolution::ExplicitFlag,
                VaultRootResolution::EnvironmentVariable,
                VaultRootResolution::AscendingDiscovery,
                VaultRootResolution::NotFound,
            ];
            assert_eq!(variants.len(), 4);
        }

        #[test]
        fn returns_local_location_when_config_location_is_not_global() {
            let global =
                ConfigLocation::Global(GlobalConfigLocation::XdgConfig);
            let local = ConfigLocation::Local(
                LocalConfigLocation::HiddenRootConfigFile,
            );
            assert_ne!(global, local);
        }

        #[test]
        fn returns_case_correction_variant_when_constructing_case_correction_warning()
         {
            let warning = DiscoveryWarning::CaseCorrection {
                requested: PathBuf::from("/Vault/.Lithos/lithos.toml"),
                resolved: PathBuf::from("/vault/.lithos/lithos.toml"),
            };
            assert!(matches!(warning, DiscoveryWarning::CaseCorrection { .. }));
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
                warnings: vec![DiscoveryWarning::LocalAmbiguity {
                    candidates: vec![PathBuf::from("/vault/lithos.toml")],
                }],
            };
            assert_eq!(result.warnings.len(), 1);
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

        #[test]
        fn returns_true_when_comparing_equal_discovery_warning_payloads() {
            let first = DiscoveryWarning::FormatAmbiguity {
                base: PathBuf::from("/vault/.lithos"),
                candidates: vec![PathBuf::from("/vault/.lithos/lithos.toml")],
            };
            let second = DiscoveryWarning::FormatAmbiguity {
                base: PathBuf::from("/vault/.lithos"),
                candidates: vec![PathBuf::from("/vault/.lithos/lithos.toml")],
            };
            assert_eq!(first, second);
        }
    }
}
