//! Filesystem scanner port and walkdir implementation.

use std::path::PathBuf;

use crate::{
    fs::{DirNode, FileNode},
    indexer::scan::IndexScope,
};

pub(crate) mod walkdir;

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

/// A record of a node that could not be indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkippedEntry {
    /// Path to the skipped entry.
    pub(crate) path: PathBuf,
    /// The reason the entry was skipped.
    pub(crate) reason: SkipReason,
}

/// The reason a node was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkipReason {
    /// Access was denied.
    PermissionDenied,
    /// The entry type (e.g., socket, pipe) is not supported.
    UnsupportedEntryType,
    /// An unknown error occurred.
    Unknown(String),
}

/// Errors that can occur during filesystem scanning.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub(crate) enum ScannerError {
    /// A walkdir entry or metadata read failed during traversal.
    #[error("traversal failed for {path}: {source}")]
    Traversal {
        /// Path where the error occurred.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// An unknown error occurred.
    #[error("Scanner error: {0}")]
    Unknown(String),
}
