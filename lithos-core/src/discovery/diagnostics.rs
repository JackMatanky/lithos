//! Structured non-fatal diagnostics for the discovery process.
//!
//! While [`DiscoveryError`] handles fatal failures, this module provides types
//! for reporting recoverable issues encountered during discovery. These
//! diagnostics should be surfaced to the user to explain why certain paths or
//! sources were skipped or ignored.
//!
//! # Diagnostics
//!
//! - [`VaultDiscoveryWarning`]: Reports issues during vault root resolution,
//!   primarily focusing on malformed or inaccessible discovery ceiling
//!   segments.

use std::path::PathBuf;

/// Specific warnings emitted during vault root resolution and boundary parsing.
///
/// These warnings indicate issues that are not fatal but might result in
/// unexpected discovery behavior (e.g., ignoring a malformed ceiling path).
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VaultDiscoveryWarning {
    /// A segment in the ceiling path list was empty (e.g. `::` or
    /// leading/trailing `:`).
    EmptyCeilingSegment,
    /// A ceiling directory path was either missing, not a directory, or
    /// inaccessible.
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
