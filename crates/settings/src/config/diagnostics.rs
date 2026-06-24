use std::path::PathBuf;

/// Warnings emitted while selecting local config candidates.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LocalDiscoveryWarning {
    /// All found candidates in search order.
    Ambiguity {
        candidates: Vec<PathBuf>,
    },
}

/// Warnings emitted while selecting among config formats.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FormatDiscoveryWarning {
    /// The common base directory where the ambiguity was found.
    Ambiguity {
        base: PathBuf,
        /// All existing format variants found.
        candidates: Vec<PathBuf>,
    },
}

/// Config-owned warning type wrapping discovery diagnostics.
///
/// Aggregates warning types produced during config discovery that are owned
/// by the Config context, separating them from Discovery-owned warnings.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConfigWarning {
    /// Local config candidate diagnostics.
    Local(LocalDiscoveryWarning),
    /// Structured-format candidate diagnostics.
    Format(FormatDiscoveryWarning),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod local_discovery_warning {
        use super::*;

        #[test]
        fn returns_ambiguity_warning() {
            let warning = LocalDiscoveryWarning::Ambiguity {
                candidates: vec![
                    PathBuf::from("/vault/traces.toml"),
                    PathBuf::from("/vault/.traces/config.toml"),
                ],
            };
            assert!(matches!(warning, LocalDiscoveryWarning::Ambiguity { .. }));
        }
    }

    mod format_discovery_warning {
        use super::*;

        #[test]
        fn returns_ambiguity_warning() {
            let warning = FormatDiscoveryWarning::Ambiguity {
                base: PathBuf::from("/vault/.traces"),
                candidates: vec![
                    PathBuf::from("/vault/.traces/config.toml"),
                    PathBuf::from("/vault/.traces/config.json"),
                ],
            };
            assert!(matches!(
                warning,
                FormatDiscoveryWarning::Ambiguity { .. }
            ));
        }
    }

    mod config_warning {
        use super::*;

        #[test]
        fn wraps_local_warning() {
            let warning =
                ConfigWarning::Local(LocalDiscoveryWarning::Ambiguity {
                    candidates: vec![PathBuf::from("/vault/traces.toml")],
                });
            assert!(matches!(warning, ConfigWarning::Local(..)));
        }

        #[test]
        fn wraps_format_warning() {
            let warning =
                ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.traces"),
                    candidates: vec![PathBuf::from(
                        "/vault/.traces/config.toml",
                    )],
                });
            assert!(matches!(warning, ConfigWarning::Format(..)));
        }

        #[test]
        fn returns_true_when_warning_payloads_match() {
            let first =
                ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.traces"),
                    candidates: vec![PathBuf::from(
                        "/vault/.traces/traces.toml",
                    )],
                });
            let second =
                ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base: PathBuf::from("/vault/.traces"),
                    candidates: vec![PathBuf::from(
                        "/vault/.traces/traces.toml",
                    )],
                });
            assert_eq!(first, second);
        }
    }
}
