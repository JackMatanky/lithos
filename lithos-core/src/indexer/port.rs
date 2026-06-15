//! Scanner port — filesystem traversal contract for the indexer context.
//!
//! Defines the `ScannerPort` trait, the interface through which the indexer
//! requests filesystem traversal, along with its associated result type.
//! Implementations (adapters) live in the scanner submodule.

use crate::{
    fs::{DirNode, FileNode},
    indexer::{error::ScannerError, report::SkippedEntry, scan::IndexScope},
};

/// Interface for filesystem traversal.
pub(crate) trait ScannerPort {
    /// Scan the filesystem within the given scope.
    ///
    /// # Errors
    /// Returns a `ScannerError` if scanning fails.
    fn scan(&self, scope: &IndexScope) -> Result<ScanResult, ScannerError>;
}

/// The result of a scan operation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ScanResult {
    /// Discovered files.
    pub(crate) files: Vec<FileNode>,
    /// Discovered directories.
    pub(crate) dirs: Vec<DirNode>,
    /// Entries that were skipped during scanning.
    pub(crate) skipped: Vec<SkippedEntry>,
}

impl ScanResult {
    /// Create a new `ScanResult`.
    #[must_use]
    #[inline]
    pub(crate) fn new(
        files: Vec<FileNode>,
        dirs: Vec<DirNode>,
        skipped: Vec<SkippedEntry>,
    ) -> Self {
        Self {
            files,
            dirs,
            skipped,
        }
    }
}
