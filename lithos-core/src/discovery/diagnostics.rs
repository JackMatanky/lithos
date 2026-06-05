//! Structured non-fatal diagnostics for the discovery process.
//!
//! These types are used to report recoverable issues (like path casing
//! mismatches or malformed configuration ceiling paths) that do not stop
//! discovery but should be surfaced to the user.

use std::path::PathBuf;

/// Typed warning channel for non-fatal discovery diagnostics.
///
/// Warnings are structured so CLI/reporting layers can render deterministic,
/// actionable diagnostics without parsing free-form strings.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DiscoveryWarning {
    /// Path casing was corrected during discovery on case-insensitive
    /// filesystems.
    CaseCorrection {
        /// The path as requested or derived from convention.
        requested: PathBuf,
        /// The actual correctly-cased path found on disk.
        resolved: PathBuf,
    },
    /// Vault-discovery-specific warning payload.
    RootResolution(VaultDiscoveryWarning),
    /// Global-discovery-specific warning payload.
    GlobalResolution(GlobalDiscoveryWarning),
}

/// Specific warnings emitted during global config discovery.
#[allow(dead_code, reason = "Phase-2 seam; surfaced once CLI reporting lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GlobalDiscoveryWarning {
    /// A recognized global config filename was found with non-canonical casing.
    CaseCorrection {
        /// The canonical path discovery expected.
        requested: PathBuf,
        /// The actual path found on disk.
        resolved: PathBuf,
    },
}

/// Specific warnings emitted during vault root resolution and boundary parsing.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VaultDiscoveryWarning {
    /// A segment in the ceiling path list was empty (e.g. `::` or
    /// leading/trailing `:`).
    EmptyCeilingSegment,
    /// A ceiling directory path was either missing or not a directory.
    InvalidCeilingSegment {
        /// The raw segment that failed validation.
        segment: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_case_correction_warning() {
        let warning = DiscoveryWarning::CaseCorrection {
            requested: PathBuf::from("/Vault/.Lithos/lithos.toml"),
            resolved: PathBuf::from("/vault/.lithos/lithos.toml"),
        };
        assert!(matches!(warning, DiscoveryWarning::CaseCorrection { .. }));
    }

    #[test]
    fn returns_root_resolution_warning() {
        let warning = DiscoveryWarning::RootResolution(
            VaultDiscoveryWarning::EmptyCeilingSegment,
        );
        assert!(matches!(
            warning,
            DiscoveryWarning::RootResolution(
                VaultDiscoveryWarning::EmptyCeilingSegment
            )
        ));
    }

    #[test]
    fn returns_global_resolution_warning() {
        let warning = DiscoveryWarning::GlobalResolution(
            GlobalDiscoveryWarning::CaseCorrection {
                requested: PathBuf::from("/config/lithos.toml"),
                resolved: PathBuf::from("/config/Lithos.toml"),
            },
        );
        assert!(matches!(
            warning,
            DiscoveryWarning::GlobalResolution(
                GlobalDiscoveryWarning::CaseCorrection { .. }
            )
        ));
    }

    mod vault_discovery_warning {
        use super::*;

        #[test]
        fn empty_segment_is_constructible() {
            let w = VaultDiscoveryWarning::EmptyCeilingSegment;
            assert_eq!(w, VaultDiscoveryWarning::EmptyCeilingSegment);
        }

        #[test]
        fn invalid_segment_holds_path() {
            let w = VaultDiscoveryWarning::InvalidCeilingSegment {
                segment: PathBuf::from("/invalid"),
            };
            assert!(matches!(
                w,
                VaultDiscoveryWarning::InvalidCeilingSegment { .. }
            ));
        }
    }
}
