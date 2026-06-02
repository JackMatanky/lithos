use std::path::PathBuf;

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
    /// Local config candidate diagnostics.
    Local(LocalDiscoveryWarning),
    /// Structured-format candidate diagnostics.
    Format(FormatDiscoveryWarning),
    /// Path casing was corrected during discovery on case-insensitive
    /// filesystems.
    CaseCorrection {
        /// The path as requested or derived from convention.
        requested: PathBuf,
        /// The actual correctly-cased path found on disk.
        resolved: PathBuf,
    },
    /// Root-resolution-specific warning payload.
    RootResolution(RootResolutionWarning),
}

/// Warnings emitted while resolving vault roots.
#[allow(
    dead_code,
    reason = "Phase-1 warning seam is implemented before orchestration wiring"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RootResolutionWarning {
    EmptyCeilingSegment,
    InvalidCeilingSegment {
        segment: PathBuf,
    },
}

/// Warnings emitted while selecting local config candidates.
#[allow(
    dead_code,
    reason = "Phase-1 warning seam is implemented before orchestration wiring"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LocalDiscoveryWarning {
    Ambiguity {
        /// All found candidates in search order.
        candidates: Vec<PathBuf>,
    },
}

/// Warnings emitted while selecting among config formats.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FormatDiscoveryWarning {
    Ambiguity {
        /// The common base directory where the ambiguity was found.
        base: PathBuf,
        /// All existing format variants found.
        candidates: Vec<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
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
        fn returns_format_ambiguity_warning() {
            let warning =
                DiscoveryWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.lithos"),
                    candidates: vec![
                        PathBuf::from("/vault/.lithos/config.toml"),
                        PathBuf::from("/vault/.lithos/config.json"),
                    ],
                });
            assert!(matches!(
                warning,
                DiscoveryWarning::Format(
                    FormatDiscoveryWarning::Ambiguity { .. }
                )
            ));
        }

        #[test]
        fn returns_local_ambiguity_warning() {
            let warning =
                DiscoveryWarning::Local(LocalDiscoveryWarning::Ambiguity {
                    candidates: vec![
                        PathBuf::from("/vault/lithos.toml"),
                        PathBuf::from("/vault/.lithos/config.toml"),
                    ],
                });
            assert!(matches!(
                warning,
                DiscoveryWarning::Local(
                    LocalDiscoveryWarning::Ambiguity { .. }
                )
            ));
        }

        #[test]
        fn returns_root_resolution_warning() {
            let warning = DiscoveryWarning::RootResolution(
                RootResolutionWarning::EmptyCeilingSegment,
            );
            assert!(matches!(
                warning,
                DiscoveryWarning::RootResolution(
                    RootResolutionWarning::EmptyCeilingSegment
                )
            ));
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn returns_true_when_warning_payloads_match() {
            let first =
                DiscoveryWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.lithos"),
                    candidates: vec![PathBuf::from(
                        "/vault/.lithos/lithos.toml",
                    )],
                });
            let second =
                DiscoveryWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.lithos"),
                    candidates: vec![PathBuf::from(
                        "/vault/.lithos/lithos.toml",
                    )],
                });
            assert_eq!(first, second);
        }
    }
}
