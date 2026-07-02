//! Report-only process metadata emitted by discovery.

use std::path::PathBuf;

/// Non-fatal process metadata for Bootstrapper and CLI diagnostics.
///
/// The default value represents a run with no skipped ceilings, no explicit
/// stop reason beyond reaching the filesystem root, and global resolution not
/// suppressed. Use [`DiscoveryReport::default`] as the zero-value when
/// constructing a fresh processor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveryReport {
    /// Ceiling path segments ignored during traversal setup.
    pub skipped_ceilings: Vec<SkippedCeiling>,
    /// Why local traversal stopped or did not run.
    pub local_traversal_stop_reason: LocalTraversalStopReason,
    /// Why global resolution was skipped, if it was skipped explicitly.
    pub global_resolution_skip_reason: Option<GlobalResolutionSkipReason>,
}

/// A ceiling segment that could not be used for traversal bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedCeiling {
    /// Raw segment after path-list splitting.
    pub segment: PathBuf,
    /// Why the segment was ignored.
    pub reason: SkippedCeilingReason,
}

/// Reasons a ceiling path-list segment is ignored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkippedCeilingReason {
    /// Segment was empty after trimming.
    EmptySegment,
    /// Segment did not resolve to an existing directory.
    InvalidPath,
}

/// Reasons local traversal stopped or was skipped.
///
/// The default value is [`LocalTraversalStopReason::FilesystemRoot`], which
/// represents the normal termination condition when no other stop reason is
/// set.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum LocalTraversalStopReason {
    /// Traversal did not run because an explicit config file was supplied.
    ///
    /// Old-path only: emitted by `processor_old` and consumed by the old
    /// bootstrap/CLI diagnostics. New linear discovery never sets this
    /// variant. Retained until the old discovery service is removed in
    /// issue 07.
    ExplicitConfigFile,
    /// Traversal reached the filesystem root.
    #[default]
    FilesystemRoot,
    /// Traversal stopped at a project boundary marker.
    ProjectBoundaryMarker {
        /// Boundary marker path that stopped traversal.
        marker: PathBuf,
    },
    /// Traversal stopped at a configured ceiling directory.
    CeilingEnforced {
        /// Ceiling directory that stopped traversal.
        ceiling: PathBuf,
    },
}

/// Reasons global resolution was intentionally skipped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalResolutionSkipReason {
    /// Invocation used `--no-global-config`.
    SuppressedByFlag,
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn skipped_ceiling() -> SkippedCeiling {
        SkippedCeiling {
            segment: PathBuf::from("/missing"),
            reason: SkippedCeilingReason::InvalidPath,
        }
    }

    fn report_with_all_fields() -> DiscoveryReport {
        DiscoveryReport {
            skipped_ceilings: vec![skipped_ceiling()],
            local_traversal_stop_reason:
                LocalTraversalStopReason::ExplicitConfigFile,
            global_resolution_skip_reason: Some(
                GlobalResolutionSkipReason::SuppressedByFlag,
            ),
        }
    }

    mod discovery_report {
        use super::*;

        mod accessors {
            use super::*;

            #[test]
            fn returns_skipped_ceilings() {
                let report = report_with_all_fields();
                assert_eq!(
                    report.skipped_ceilings,
                    [skipped_ceiling()],
                    "expected skipped_ceilings to hold the configured ceiling"
                );
            }

            #[test]
            fn returns_local_traversal_stop_reason() {
                let report = report_with_all_fields();
                assert_eq!(
                    report.local_traversal_stop_reason,
                    LocalTraversalStopReason::ExplicitConfigFile,
                    "expected ExplicitConfigFile stop reason"
                );
            }

            #[test]
            fn returns_global_resolution_skip_reason_when_present() {
                let report = report_with_all_fields();
                assert_eq!(
                    report.global_resolution_skip_reason,
                    Some(GlobalResolutionSkipReason::SuppressedByFlag),
                    "expected SuppressedByFlag skip reason"
                );
            }

            #[test]
            fn returns_none_for_global_resolution_skip_reason_when_absent() {
                let report = DiscoveryReport {
                    skipped_ceilings: vec![],
                    local_traversal_stop_reason:
                        LocalTraversalStopReason::FilesystemRoot,
                    global_resolution_skip_reason: None,
                };
                assert!(
                    report.global_resolution_skip_reason.is_none(),
                    "expected no skip reason when global resolution was not \
                     skipped"
                );
            }
        }
    }
}
