use std::path::PathBuf;

use super::error::VaultDiscoveryWarning;

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
}
