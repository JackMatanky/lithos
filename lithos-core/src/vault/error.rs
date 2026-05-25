//! Error types for vault processing and storage.

use std::path::PathBuf;

use crate::{
    db::DbError, fs::PathValidationError, note::error::NoteProcessError,
};

/// Errors that occur while validating vault paths.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "VaultPathError clarifies the vault-specific boundary"
)]
pub enum VaultPathError {
    /// Path is invalid under vault validation rules.
    #[error("invalid vault path: {0}")]
    InvalidPath(#[from] PathValidationError),

    /// Path is not valid UTF-8.
    #[error("path contains invalid UTF-8: {path:?}")]
    InvalidPathEncoding {
        /// The path that failed validation.
        path: PathBuf,
    },
}

/// Repository errors for vault storage operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "VaultRepositoryError clarifies the storage boundary"
)]
pub enum VaultRepositoryError {
    /// Database storage error.
    #[error("storage error: {0}")]
    Storage(#[from] DbError),

    /// File requested by unique identifier was not found.
    #[error("file not found: {0}")]
    FileNotFound(super::model::FileId),

    /// Directory requested by unique identifier was not found.
    #[error("directory not found: {0}")]
    DirNotFound(super::model::DirId),

    /// Entry requested by vault path was not found.
    #[error("path not found: {0}")]
    PathNotFound(crate::fs::NormalizedPath),

    /// Persistence conflict where an entry already exists at the target path.
    #[error("duplicate path: entry already exists at {0}")]
    DuplicatePath(crate::fs::NormalizedPath),
}

#[cfg(test)]
impl From<crate::db::testing::InMemoryDbError> for VaultRepositoryError {
    #[inline]
    fn from(value: crate::db::testing::InMemoryDbError) -> Self {
        Self::Storage(DbError::Open(value.to_string()))
    }
}

/// Errors during vault file discovery and metadata extraction.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "VaultFileError clarifies the vault scanning boundary"
)]
pub enum VaultFileError {
    /// A vault path failed validation.
    #[error("invalid vault path {path}: {reason}")]
    InvalidPath {
        /// The invalid path.
        path: Box<str>,
        /// The reason the path is invalid.
        reason: Box<str>,
    },

    /// Reading file metadata failed.
    #[error("failed to read metadata for {path}: {message}")]
    MetadataFailed {
        /// The vault-relative path.
        path: Box<str>,
        /// The error message.
        message: Box<str>,
    },

    /// Reading file content failed.
    #[error("failed to read content for {path}: {message}")]
    ReadFailed {
        /// The vault-relative path.
        path: Box<str>,
        /// The error message.
        message: Box<str>,
    },
}

/// Errors surfaced during the vault processing pipeline.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "VaultProcessError clarifies pipeline failures"
)]
pub enum VaultProcessError {
    /// File discovery or metadata access failed.
    #[error("vault file error: {0}")]
    File(#[from] VaultFileError),

    /// Repository operation failed.
    #[error("vault repository error: {0}")]
    Repository(#[from] VaultRepositoryError),

    /// Note processing failed.
    #[error("note processing error: {0}")]
    Note(#[from] NoteProcessError),
}
