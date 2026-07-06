//! Indexer error types and boundaries.
//!
//! This module defines the core error types for the `traces-indexer` bounded
//! context. [`IndexerError`] separates failures into three arms — scanner /
//! traversal errors, local database (repository) errors, and path-resolution
//! errors — so upstream components can categorize and respond to each
//! appropriately. The `Path` arm is the linchpin of the soft-fail model:
//! per-entry path errors are recoverable, while repository errors are fatal.

#![allow(deprecated, reason = "legacy error compatibility tests")]

use std::path::PathBuf;

use traces_db::DbError;
use traces_fs::{PathKey, error::PathError};

/// Unified error type for the indexer context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IndexerError {
    /// An error originated from the scanner adapter during filesystem
    /// traversal.
    #[error(transparent)]
    Scanner(#[from] ScannerError),
    /// An error originated from the local database or storage repository.
    #[error(transparent)]
    Repository(#[from] IndexerRepositoryError),
    /// An error occurred while parsing or resolving a filesystem path.
    #[error(transparent)]
    Path(#[from] PathError),
}

/// Repository-layer errors surfaced through the port boundary.
/// redb and rkyv types never appear here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IndexerRepositoryError {
    /// Transparent wrapper around the shared `DbError` (follows
    /// `VaultRepositoryError` pattern).
    #[error("storage error: {0}")]
    Storage(#[from] DbError),
    /// A `PathKey` write would create a duplicate entry.
    #[error("duplicate path: {0}")]
    DuplicatePath(PathKey),
}

/// Errors that can occur during filesystem scanning.
#[derive(thiserror::Error, Debug)]
pub enum ScannerError {
    /// A walkdir entry or metadata read failed during traversal.
    #[error("traversal failed for {path}: {source}")]
    Traversal {
        /// Path where the error occurred.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    mod conversions {
        use std::path::PathBuf;

        use super::super::*;

        #[test]
        fn converts_scanner_error_to_indexer_error() {
            let io_err =
                std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
            let scanner_err = ScannerError::Traversal {
                path: PathBuf::from("/test/path"),
                source: io_err,
            };

            let indexer_err: IndexerError = scanner_err.into();

            assert!(matches!(
                indexer_err,
                IndexerError::Scanner(ScannerError::Traversal { .. })
            ));
            assert!(indexer_err.to_string().contains("/test/path"));
            assert!(indexer_err.to_string().contains("test error"));
        }

        #[test]
        fn converts_db_error_to_repository_error_to_indexer_error() {
            let db_err =
                traces_db::DbError::Corruption("failed to serialize".into());
            let repo_err: IndexerRepositoryError = db_err.into();
            let indexer_err: IndexerError = repo_err.into();

            assert!(matches!(
                indexer_err,
                IndexerError::Repository(IndexerRepositoryError::Storage(
                    traces_db::DbError::Corruption(_)
                ))
            ));

            // Verify DbError wraps through IndexerRepositoryError::Storage and
            // Display output remains actionable
            let msg = indexer_err.to_string();
            assert!(msg.contains(
                "storage error: data corruption: failed to serialize"
            ));
        }
    }
}
