use crate::{
    fs::{FileFormat, path::PathKey},
    indexer::{
        error::IndexerRepositoryError,
        model::{DirRecord, FileRecord, FsRecordId},
    },
};

/// Repository trait for reading indexed filesystem records.
pub(crate) trait ReadRepository {
    fn find_file(
        &self,
        id: FsRecordId,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    fn find_dir(
        &self,
        id: FsRecordId,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    fn find_file_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError>;
    fn find_dir_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError>;
    fn list_files_by_parent(
        &self,
        parent_id: FsRecordId,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    fn list_dirs_by_parent(
        &self,
        parent_id: FsRecordId,
    ) -> Result<Box<[DirRecord]>, IndexerRepositoryError>;
    fn list_files_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    fn list_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError>;
    /// Returns all persisted `PathKey`s; used by the application service for
    /// deletion detection.
    fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError>;
}

/// Repository trait for writing indexed filesystem records.
pub(crate) trait WriteRepository {
    fn save_file(
        &self,
        record: &FileRecord,
    ) -> Result<(), IndexerRepositoryError>;
    fn save_dir(
        &self,
        record: &DirRecord,
    ) -> Result<(), IndexerRepositoryError>;
    fn delete_file(&self, id: FsRecordId)
    -> Result<(), IndexerRepositoryError>;
    fn delete_dir(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError>;
    /// Atomically persist files and dirs in one `WriteTransaction`.
    /// All primary records and all secondary indexes are written together or
    /// not at all.
    fn save_many_records(
        &self,
        files: &[FileRecord],
        dirs: &[DirRecord],
    ) -> Result<(), IndexerRepositoryError>;
    /// Atomically prune file and dir records in one `WriteTransaction`.
    /// All primary records and all secondary indexes are removed together or
    /// not at all.
    fn delete_many_records(
        &self,
        file_ids: &[FsRecordId],
        dir_ids: &[FsRecordId],
    ) -> Result<(), IndexerRepositoryError>;
}

/// Unified repository trait combining read and write capabilities.
pub(crate) trait Repository: ReadRepository + WriteRepository {}

impl<T> Repository for T where T: ReadRepository + WriteRepository {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::model::{DirRecord, FileRecord, FsRecordId};

    struct MockRepository;

    impl ReadRepository for MockRepository {
        fn find_file(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_dir(
            &self,
            _id: FsRecordId,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_file_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn find_dir_by_path(
            &self,
            _path: &PathKey,
        ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
            Ok(None)
        }

        fn list_files_by_parent(
            &self,
            _parent_id: FsRecordId,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn list_dirs_by_parent(
            &self,
            _parent_id: FsRecordId,
        ) -> Result<Box<[DirRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

        fn list_files_by_format(
            &self,
            _format: FileFormat,
        ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
            Ok(Box::new([]))
        }

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
        fn save_file(
            &self,
            _record: &FileRecord,
        ) -> Result<(), IndexerRepositoryError> {
            Ok(())
        }

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
    }

    #[test]
    fn blanket_repository_impl_accepts_read_write_type() {
        let repo = MockRepository;
        let _: &dyn Repository = &repo;
    }
}
