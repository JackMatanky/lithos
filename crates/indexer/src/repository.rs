//! Indexer repository port — persistence contract for indexed records.
//!
//! Defines the `ReadRepository` and `WriteRepository` traits that form the
//! persistence boundary of the indexer context. Adapters (e.g.,
//! `RedbRepository`, `InMemoryRepository`) live in the storage submodule.

use traces_fs::{FileFormat, path::PathKey};

use crate::{
    error::IndexerRepositoryError,
    model::{DirRecord, FileRecord, FsParentId, FsRecordId},
};

/// Repository trait for reading indexed filesystem records.
pub trait ReadRepository {
    /// Find a file record by its ID.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn find_file(
        &self,
        id: FsRecordId,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    /// Find a directory record by its ID.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn find_dir(
        &self,
        id: FsRecordId,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    /// Find a file record by its path key.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn find_file_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    /// Find a directory record by its path key.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn find_dir_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    /// List all files within a specific parent directory.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn list_files_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    /// List all subdirectories within a specific parent directory.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn list_dirs_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[DirRecord]>, IndexerRepositoryError>;
    /// List all files matching a specific format.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn list_files_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    /// List all files matching a specific basename.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn list_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    /// Returns all persisted `PathKey`s; used by the application service for
    /// deletion detection.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError>;
}

/// Repository trait for writing indexed filesystem records.
pub trait WriteRepository {
    /// Save a single file record.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn save_file(
        &self,
        record: &FileRecord,
    ) -> Result<(), IndexerRepositoryError>;
    /// Save a single directory record.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn save_dir(
        &self,
        record: &DirRecord,
    ) -> Result<(), IndexerRepositoryError>;
    /// Delete a file record by its ID.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn delete_file(&self, id: FsRecordId)
    -> Result<(), IndexerRepositoryError>;
    /// Delete a directory record by its ID.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn delete_dir(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError>;
    /// Atomically persist files and dirs in one `WriteTransaction`.
    /// All primary records and all secondary indexes are written together or
    /// not at all.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn save_many_records(
        &self,
        files: &[FileRecord],
        dirs: &[DirRecord],
    ) -> Result<(), IndexerRepositoryError>;
    /// Atomically prune file and dir records in one `WriteTransaction`.
    /// All primary records and all secondary indexes are removed together or
    /// not at all.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn delete_many_records(
        &self,
        file_ids: &[FsRecordId],
        dir_ids: &[FsRecordId],
    ) -> Result<(), IndexerRepositoryError>;
    /// Remove all persisted records.
    ///
    /// # Errors
    /// Returns an `IndexerRepositoryError` if the database operation fails.
    fn clear(&self) -> Result<(), IndexerRepositoryError>;
}

/// Unified repository trait combining read and write capabilities.
pub trait Repository: ReadRepository + WriteRepository {}

impl<T> Repository for T where T: ReadRepository + WriteRepository {}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DirRecord, FileRecord, FsParentId, FsRecordId};

    struct MockRepository;

    impl ReadRepository for MockRepository {
        /// Find a file record by its ID.
        fn find_file(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        /// Find a directory record by its ID.
        fn find_dir(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        /// Find a file record by its path key.
        fn find_file_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        /// Find a directory record by its path key.
        fn find_dir_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        /// List all files within a specific parent directory.
        fn list_files_by_parent(
            &self,
            _parent_id: FsParentId,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        /// List all subdirectories within a specific parent directory.
        fn list_dirs_by_parent(
            &self,
            _parent_id: FsParentId,
        ) -> Result<Box<[DirRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        /// List all files matching a specific format.
        fn list_files_by_format(
            &self,
            _format: FileFormat,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        /// List all files matching a specific basename.
        fn list_files_by_basename(
            &self,
            _basename: &str,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }
    }

    impl WriteRepository for MockRepository {
        /// Save a single file record.
        fn save_file(
            &self,
            _record: &FileRecord,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        /// Save a single directory record.
        fn save_dir(
            &self,
            _record: &DirRecord,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn delete_file(
            &self,
            _id: FsRecordId,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn delete_dir(
            &self,
            _id: FsRecordId,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn save_many_records(
            &self,
            _files: &[FileRecord],
            _dirs: &[DirRecord],
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn delete_many_records(
            &self,
            _file_ids: &[FsRecordId],
            _dir_ids: &[FsRecordId],
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

        fn clear(&self) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }
    }

    #[test]
    fn blanket_repository_impl_accepts_read_write_type() {
        let repo = MockRepository;
        let _: &dyn Repository = &repo;
    }
}
