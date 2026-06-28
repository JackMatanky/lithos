//! Indexer report types — scan outcomes and skipped-entry records.
//!
//! Defines the report types that aggregate the results of a scan run:
//! entries that were skipped (`SkippedEntry`, `SkipReason`) and per-node
//! indexing failures (`IndexNodeFailure`).

use std::path::PathBuf;

/// A summary report containing metrics and failures from an indexer run.
#[derive(Debug, Clone)]
pub struct IndexReport {
    scanned: usize,
    new: usize,
    fresh: usize,
    stale: usize,
    deleted: usize,
    skipped: Box<[SkippedEntry]>,
    failures: Box<[IndexNodeFailure]>,
}

impl IndexReport {
    /// Creates a new index report.
    ///
    /// `pub(crate)`: only the indexer service constructs reports, so consumers
    /// cannot fabricate inconsistent counts. Tests outside the crate use
    /// [`IndexReport::new_for_test`] behind the `testing` feature.
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain report encapsulates all summary counters"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
        scanned: usize,
        new: usize,
        fresh: usize,
        stale: usize,
        deleted: usize,
        skipped: Box<[SkippedEntry]>,
        failures: Box<[IndexNodeFailure]>,
    ) -> Self {
        Self {
            scanned,
            new,
            fresh,
            stale,
            deleted,
            skipped,
            failures,
        }
    }

    /// Test-only constructor for building reports in consumer-crate tests.
    ///
    /// Available only under the `testing` feature so production code cannot
    /// fabricate inconsistent reports through the public API.
    #[cfg(feature = "testing")]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain report encapsulates all summary counters"
    )]
    #[inline]
    #[must_use]
    pub fn new_for_test(
        scanned: usize,
        new: usize,
        fresh: usize,
        stale: usize,
        deleted: usize,
        skipped: Box<[SkippedEntry]>,
        failures: Box<[IndexNodeFailure]>,
    ) -> Self {
        Self::new(scanned, new, fresh, stale, deleted, skipped, failures)
    }

    /// Returns the total number of nodes scanned.
    #[inline]
    #[must_use]
    pub fn scanned(&self) -> usize {
        self.scanned
    }

    /// Returns the count of new nodes.
    #[inline]
    #[must_use]
    pub fn new_count(&self) -> usize {
        self.new
    }

    /// Returns the count of fresh nodes.
    #[inline]
    #[must_use]
    pub fn fresh_count(&self) -> usize {
        self.fresh
    }

    /// Returns the count of stale nodes.
    #[inline]
    #[must_use]
    pub fn stale_count(&self) -> usize {
        self.stale
    }

    /// Returns the count of deleted nodes.
    #[inline]
    #[must_use]
    pub fn deleted_count(&self) -> usize {
        self.deleted
    }

    /// Returns the entries skipped during the scan.
    #[inline]
    #[must_use]
    pub fn skipped(&self) -> &[SkippedEntry] {
        &self.skipped
    }

    /// Returns the failures encountered during the scan.
    #[inline]
    #[must_use]
    pub fn failures(&self) -> &[IndexNodeFailure] {
        &self.failures
    }
}

/// A failure record for a single filesystem node that could not be indexed.
///
/// A pre-classification failure has no assigned record id and no known
/// file-vs-directory kind, so the failure is keyed by the entry's path plus
/// the error message — mirroring [`SkippedEntry`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexNodeFailure {
    path: PathBuf,
    error: Box<str>,
}

impl IndexNodeFailure {
    /// Creates a new failure record.
    ///
    /// `pub(crate)`: only the indexer service records failures. Consumer-crate
    /// tests use [`IndexNodeFailure::new_for_test`] behind the `testing`
    /// feature.
    #[inline]
    #[must_use]
    pub(crate) fn new(path: PathBuf, error: Box<str>) -> Self {
        Self {
            path,
            error,
        }
    }

    /// Test-only constructor for building failures in consumer-crate tests.
    ///
    /// Available only under the `testing` feature.
    #[cfg(feature = "testing")]
    #[inline]
    #[must_use]
    pub fn new_for_test(path: PathBuf, error: Box<str>) -> Self {
        Self::new(path, error)
    }

    /// Returns the path of the entry that failed to index.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the error message for this failure.
    #[inline]
    #[must_use]
    pub fn error(&self) -> &str {
        &self.error
    }
}

/// A record of a node that could not be indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedEntry {
    /// Path to the skipped entry.
    pub(crate) path: PathBuf,
    /// The reason the entry was skipped.
    pub(crate) reason: SkipReason,
}

/// The reason a node was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Access was denied.
    PermissionDenied,
    /// The entry type (e.g., socket, pipe) is not supported.
    UnsupportedEntryType,
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn stores_counts_and_failures() {
        let report =
            IndexReport::new(10, 2, 5, 3, 1, Box::new([]), Box::new([]));
        assert_eq!(report.scanned(), 10);
        assert_eq!(report.new_count(), 2);
        assert_eq!(report.fresh_count(), 5);
        assert_eq!(report.stale_count(), 3);
        assert_eq!(report.deleted_count(), 1);
        assert_eq!(report.skipped().len(), 0);
        assert_eq!(report.failures().len(), 0);
    }

    #[test]
    fn stores_skipped_entries() {
        let skipped = vec![SkippedEntry {
            path: PathBuf::from("restricted"),
            reason: SkipReason::PermissionDenied,
        }];
        let report = IndexReport::new(
            1,
            0,
            0,
            0,
            0,
            skipped.into_boxed_slice(),
            Box::new([]),
        );

        assert_eq!(report.skipped().len(), 1);
        assert_eq!(
            report.skipped().first().unwrap().path,
            PathBuf::from("restricted")
        );
    }

    #[test]
    fn stores_path_and_error() {
        let failure = IndexNodeFailure::new(
            PathBuf::from("notes/bad.md"),
            "permission denied".into(),
        );
        assert_eq!(failure.path(), PathBuf::from("notes/bad.md"));
        assert_eq!(failure.error(), "permission denied");
    }
}
