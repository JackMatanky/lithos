//! Indexer error types.

use std::path::PathBuf;

use trace_db::DbError;
use trace_fs::{PathKey, error::PathError};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IndexerError {
    #[error(transparent)]
    Scanner(#[from] ScannerError),
    #[error(transparent)]
    Repository(#[from] IndexerRepositoryError),
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
                trace_db::DbError::Serialization("failed to serialize".into());
            let repo_err: IndexerRepositoryError = db_err.into();
            let indexer_err: IndexerError = repo_err.into();

            assert!(matches!(
                indexer_err,
                IndexerError::Repository(IndexerRepositoryError::Storage(
                    trace_db::DbError::Serialization(_)
                ))
            ));

            // Verify DbError wraps through IndexerRepositoryError::Storage and
            // Display output remains actionable
            let msg = indexer_err.to_string();
            assert!(msg.contains(
                "storage error: serialization error: failed to serialize"
            ));
        }
    }
}
