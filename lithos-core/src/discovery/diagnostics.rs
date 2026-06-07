//! Structured non-fatal diagnostics for the discovery process.
//!
//! These types are used to report recoverable issues (like malformed
//! configuration ceiling paths) that do not stop discovery but should be
//! surfaced to the user.

use std::path::PathBuf;

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
