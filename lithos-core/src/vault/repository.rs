//! Repository traits for Vault persistence.
//!
//! Defines segregated read and write interfaces following ADR 016.

use super::{
    error::VaultRepositoryError,
    model::{DirId, DirView, FileId, FileView, FsEntryView},
};
use crate::fs::{FileFormat, PathKey};

/// Read-only repository operations for Vault file and directory views.
pub trait ReadRepository {
    /// Find a file view by its unique identifier.
    ///
    /// Returns `Ok(None)` if no file with the given ID exists.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn get_file_view(
        &self,
        id: FileId,
    ) -> Result<Option<FileView>, VaultRepositoryError>;

    /// Find a directory view by its unique identifier.
    ///
    /// Returns `Ok(None)` if no directory with the given ID exists.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn get_dir_view(
        &self,
        id: DirId,
    ) -> Result<Option<DirView>, VaultRepositoryError>;

    /// Find a file view by its normalized vault path.
    ///
    /// Performs a cross-table lookup: path → file ID → file view.
    /// Returns `Ok(None)` if no file exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn find_file_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileView>, VaultRepositoryError>;

    /// Find a directory view by its normalized vault path.
    ///
    /// Performs a cross-table lookup: path → dir ID → dir view.
    /// Returns `Ok(None)` if no directory exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn find_dir_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirView>, VaultRepositoryError>;

    /// Get any filesystem entry (file or directory) at the given path.
    ///
    /// Tries file lookup first, then directory lookup.
    /// Returns `Ok(None)` if neither exists.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn get_entry(
        &self,
        path: &PathKey,
    ) -> Result<Option<FsEntryView>, VaultRepositoryError>;

    /// Find all file views with the given basename.
    ///
    /// Uses the basename multimap index for efficient lookup.
    /// Returns an empty vector if no files match.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn find_file_views_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, VaultRepositoryError>;

    /// Find all file views that are children of the given directory.
    ///
    /// Uses the parent multimap index for efficient lookup.
    /// Returns an empty vector if the directory has no files.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn find_file_views_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, VaultRepositoryError>;

    /// List all file views with the specified format.
    ///
    /// Uses the format multimap index for efficient lookup.
    /// Returns an empty vector if no files match the format.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_file_views_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, VaultRepositoryError>;

    /// List all markdown file views.
    ///
    /// Convenience method equivalent to
    /// `list_file_views_by_format(FileFormat::Markdown)`.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_markdown_file_views(
        &self,
    ) -> Result<Vec<FileView>, VaultRepositoryError>;

    /// List all file views in the vault.
    ///
    /// Performs a full table scan. Returns an empty vector if no files exist.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError>;

    /// List all file paths in the vault.
    ///
    /// Scans the path index. Returns an empty vector if no files exist.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_file_paths(&self) -> Result<Vec<PathKey>, VaultRepositoryError>;

    /// List all directory views in the vault.
    ///
    /// Performs a full table scan. Returns an empty vector if no directories
    /// exist.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_dir_views(&self) -> Result<Vec<DirView>, VaultRepositoryError>;

    /// List all directory paths in the vault.
    ///
    /// Scans the path index. Returns an empty vector if no directories exist.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn list_dir_paths(&self) -> Result<Vec<PathKey>, VaultRepositoryError>;
}

/// Write operations for Vault persistence.
pub trait WriteRepository {
    /// Save a file view, creating or updating the entry.
    ///
    /// Atomically writes to:
    /// - Primary file view table
    /// - Path index
    /// - Basename multimap index
    /// - Parent multimap index
    /// - Format multimap index
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn save_file_view(
        &self,
        path: &PathKey,
        file: &FileView,
    ) -> Result<(), VaultRepositoryError>;

    /// Save a directory view, creating or updating the entry.
    ///
    /// Atomically writes to:
    /// - Primary directory view table
    /// - Path index
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn save_dir_view(
        &self,
        path: &PathKey,
        dir: &DirView,
    ) -> Result<(), VaultRepositoryError>;

    /// Delete a file view by its identifier.
    ///
    /// Atomically removes from all tables and indexes.
    /// Idempotent (no error if the file doesn't exist).
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn delete_file_view(&self, id: FileId) -> Result<(), VaultRepositoryError>;

    /// Delete a directory view by its identifier.
    ///
    /// Atomically removes from the primary table and path index.
    /// Idempotent (no error if the directory doesn't exist).
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn delete_dir_view(&self, id: DirId) -> Result<(), VaultRepositoryError>;

    /// Save multiple file views in a single transaction.
    ///
    /// All views are saved atomically with their indexes.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn save_many_file_views(
        &self,
        entries: &[(PathKey, FileView)],
    ) -> Result<(), VaultRepositoryError>;

    /// Save multiple directory views in a single transaction.
    ///
    /// All views are saved atomically with their indexes.
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn save_many_dir_views(
        &self,
        entries: &[(PathKey, DirView)],
    ) -> Result<(), VaultRepositoryError>;

    /// Delete multiple file views in a single transaction.
    ///
    /// Idempotent for each ID (no error if any are missing).
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn delete_many_file_views(
        &self,
        ids: &[FileId],
    ) -> Result<(), VaultRepositoryError>;

    /// Delete multiple directory views in a single transaction.
    ///
    /// Idempotent for each ID (no error if any are missing).
    ///
    /// # Errors
    ///
    /// Returns `VaultRepositoryError::Storage` if the database operation fails.
    fn delete_many_dir_views(
        &self,
        ids: &[DirId],
    ) -> Result<(), VaultRepositoryError>;
}

/// Unified repository combining read and write capabilities.
///
/// This is a marker trait automatically implemented for any type that
/// implements both `ReadRepository` and `WriteRepository`.
pub trait Repository: ReadRepository + WriteRepository {}

// Blanket implementation: any type with both read and write gets Repository for
// free
impl<T> Repository for T where T: ReadRepository + WriteRepository {}
