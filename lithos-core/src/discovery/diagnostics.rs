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
    /// Vault-discovery-specific warning payload.
    Vault(VaultDiscoveryWarning),
    /// Global-discovery-specific warning payload.
    Global(GlobalDiscoveryWarning),
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

    mod discovery_warning {
        use super::*;

        mod equality {
            use super::*;

            #[test]
            fn returns_vault_variant_when_wrapping_vault_warning() {
                let warning = DiscoveryWarning::Vault(
                    VaultDiscoveryWarning::EmptyCeilingSegment,
                );
                assert!(matches!(
                    warning,
                    DiscoveryWarning::Vault(
                        VaultDiscoveryWarning::EmptyCeilingSegment
                    )
                ));
            }

            #[test]
            fn returns_global_variant_when_wrapping_global_warning() {
                let warning = DiscoveryWarning::Global(
                    GlobalDiscoveryWarning::CaseCorrection {
                        requested: PathBuf::from("/config/lithos.toml"),
                        resolved: PathBuf::from("/config/Lithos.toml"),
                    },
                );
                assert!(matches!(
                    warning,
                    DiscoveryWarning::Global(
                        GlobalDiscoveryWarning::CaseCorrection { .. }
                    )
                ));
            }
        }
    }

    mod vault_discovery_warning {
        use super::*;

        mod equality {
            use super::*;

            #[test]
            fn returns_equal_when_empty_segment_variants_compared() {
                let w = VaultDiscoveryWarning::EmptyCeilingSegment;
                assert_eq!(w, VaultDiscoveryWarning::EmptyCeilingSegment);
            }

            #[test]
            fn returns_true_when_invalid_segment_matches_pattern() {
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
}
