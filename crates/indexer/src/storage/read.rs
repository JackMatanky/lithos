//! Redb-backed read repository implementation.
//!
//! Records are stored through typed rkyv redb adapters and decoded at the
//! storage boundary.
//!
//! The [`ReadRepository`] trait returns owned values, so full materialization
//! occurs at the storage boundary. Zero-copy access via [`rkyv::access`] is
//! available locally in any hot path that needs field-level checks before
//! materializing the full record.
//!
//! [`ReadRepository`]: crate::repository::ReadRepository

use redb::ReadableTable;
use traces_db::{DbError, path::DbPathKey};
use traces_fs::path::PathKey;

use crate::{
    error::IndexerRepositoryError,
    model::{DirRecord, FileRecord, FsParentId, FsRecordId},
    repository::ReadRepository,
    storage::{
        RedbRepository,
        tables::{
            DIR_ID_BY_PATH, DIR_IDS_BY_PARENT, DIRS, FILE_ID_BY_PATH,
            FILE_IDS_BY_BASENAME, FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT,
            FILES,
        },
    },
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn find_file(
        &self,
        id: FsRecordId,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let table = tx.inner.open_table(FILES.definition())?;
                let file = table.get(id)?;
                file.map(|v| v.value().decode().map_err(DbError::from))
                    .transpose()
            })
            .map_err(Into::into)
    }

    #[inline]
    fn find_dir(
        &self,
        id: FsRecordId,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let table = tx.inner.open_table(DIRS.definition())?;
                let dir = table.get(id)?;
                dir.map(|v| v.value().decode().map_err(DbError::from))
                    .transpose()
            })
            .map_err(Into::into)
    }

    #[inline]
    fn find_file_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let path_table =
                    tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
                let id = path_table.get(DbPathKey::from(path))?;

                if let Some(id_value) = id {
                    let file_table = tx.inner.open_table(FILES.definition())?;
                    let file = file_table.get(id_value.value())?;
                    Ok(file
                        .map(|v| v.value().decode().map_err(DbError::from))
                        .transpose()?)
                } else {
                    Ok(None)
                }
            })
            .map_err(Into::into)
    }

    #[inline]
    fn find_dir_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let path_table =
                    tx.inner.open_table(DIR_ID_BY_PATH.definition())?;
                let id = path_table.get(DbPathKey::from(path))?;

                if let Some(id_value) = id {
                    let dir_table = tx.inner.open_table(DIRS.definition())?;
                    let dir = dir_table.get(id_value.value())?;
                    Ok(dir
                        .map(|v| v.value().decode().map_err(DbError::from))
                        .transpose()?)
                } else {
                    Ok(None)
                }
            })
            .map_err(Into::into)
    }

    #[inline]
    fn list_files_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let id_table = tx
                    .inner
                    .open_multimap_table(FILE_IDS_BY_PARENT.definition())?;
                let file_table = tx.inner.open_table(FILES.definition())?;

                let mut records = Vec::new();
                let iter = id_table.get(parent_id.to_storage_key())?;
                for id in iter {
                    if let Some(record) = file_table.get(id?.value())? {
                        records.push(
                            record.value().decode().map_err(DbError::from)?,
                        );
                    }
                }

                Ok(records.into_boxed_slice())
            })
            .map_err(Into::into)
    }

    #[inline]
    fn list_dirs_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[DirRecord]>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let id_table = tx
                    .inner
                    .open_multimap_table(DIR_IDS_BY_PARENT.definition())?;
                let dir_table = tx.inner.open_table(DIRS.definition())?;

                let mut records = Vec::new();
                let iter = id_table.get(parent_id.to_storage_key())?;
                for id in iter {
                    if let Some(record) = dir_table.get(id?.value())? {
                        records.push(
                            record.value().decode().map_err(DbError::from)?,
                        );
                    }
                }

                Ok(records.into_boxed_slice())
            })
            .map_err(Into::into)
    }

    #[inline]
    fn list_files_by_format(
        &self,
        format: traces_fs::FileFormat,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let id_table =
                    tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
                let file_table = tx.inner.open_table(FILES.definition())?;

                let mut records = Vec::new();
                let iter = id_table.get(format.as_str())?;
                for id in iter {
                    if let Some(record) = file_table.get(id?.value())? {
                        records.push(
                            record.value().decode().map_err(DbError::from)?,
                        );
                    }
                }

                Ok(records.into_boxed_slice())
            })
            .map_err(Into::into)
    }

    #[inline]
    fn list_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let id_table =
                    tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
                let file_table = tx.inner.open_table(FILES.definition())?;

                let mut records = Vec::new();
                let iter = id_table.get(basename)?;
                for id in iter {
                    if let Some(record) = file_table.get(id?.value())? {
                        records.push(
                            record.value().decode().map_err(DbError::from)?,
                        );
                    }
                }

                Ok(records.into_boxed_slice())
            })
            .map_err(Into::into)
    }

    #[inline]
    fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError> {
        self.store
            .read(|tx| {
                let file_path_table =
                    tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
                let dir_path_table =
                    tx.inner.open_table(DIR_ID_BY_PATH.definition())?;

                let mut paths = Vec::new();
                let mut seen = std::collections::HashSet::new();

                for result in file_path_table.iter()? {
                    let (path, _id) = result?;
                    let pk = path.value().into_inner();
                    if seen.insert(pk.clone()) {
                        paths.push(pk);
                    }
                }

                for result in dir_path_table.iter()? {
                    let (path, _id) = result?;
                    let pk = path.value().into_inner();
                    if seen.insert(pk.clone()) {
                        paths.push(pk);
                    }
                }

                Ok(paths.into_boxed_slice())
            })
            .map_err(Into::into)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::SystemTime};

    #[allow(unused_imports, reason = "added globally for ease")]
    use pretty_assertions::{assert_eq, assert_ne};
    use traces_db::{DbPathKey, RkyvBytes, Store};
    use traces_fs::{
        FileFormat,
        metadata::{FileMetadata, FsTimes},
        name::FileName,
        path::PathKey,
    };

    use super::*;
    use crate::{
        model::{DirRecord, FileRecord, FsParentId, FsRecordId},
        repository::ReadRepository,
        storage::tables::FILES,
    };

    fn file_bytes(record: &FileRecord) -> RkyvBytes<'static, FileRecord> {
        RkyvBytes::encode(record).unwrap()
    }

    fn dir_bytes(record: &DirRecord) -> RkyvBytes<'static, DirRecord> {
        RkyvBytes::encode(record).unwrap()
    }

    fn setup_repo() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        (tempdir, RedbRepository::try_new(Arc::new(store)).unwrap())
    }

    mod lookup {
        #[allow(unused_imports, reason = "added globally for ease")]
        use pretty_assertions::{assert_eq, assert_ne};

        use super::*;
        use crate::storage::tables::FILE_IDS_BY_BASENAME;

        #[test]
        fn find_file_returns_none_when_missing() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            assert!(repo.find_file(id).unwrap().is_none());
        }

        #[test]
        fn find_file_returns_record_when_present() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsParentId::Id(FsRecordId::new());
            let path = PathKey::try_new("test").unwrap();
            let name = FileName::new("test".into());
            let format = FileFormat::Unknown;
            let metadata =
                FileMetadata::new(FsTimes::new(None, None), 0, false);
            let recorded_at = SystemTime::now();

            let record = FileRecord::new(
                id,
                parent_id,
                path,
                name,
                format,
                metadata,
                recorded_at,
            );
            // Need to insert record using write tx
            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut table =
                        tx.inner.open_table(FILES.definition()).unwrap();
                    table.insert(id, &file_bytes(&record)).unwrap();
                    Ok(())
                })
                .unwrap();

            assert_eq!(repo.find_file(id).unwrap().unwrap(), record);
        }

        #[test]
        fn find_dir_returns_none_when_missing() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            assert!(repo.find_dir(id).unwrap().is_none());
        }

        #[test]
        fn find_dir_returns_record_when_present() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let path = PathKey::try_new("test_dir").unwrap();
            let name = traces_fs::name::DirName::new("test_dir".into());
            let metadata =
                traces_fs::DirMetadata::new(FsTimes::new(None, None), false);
            let recorded_at = SystemTime::now();

            let record = DirRecord::new(
                id,
                FsParentId::Root,
                path,
                name,
                metadata,
                recorded_at,
            );
            // Need to insert record using write tx
            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut table =
                        tx.inner.open_table(DIRS.definition()).unwrap();
                    table.insert(id, &dir_bytes(&record)).unwrap();
                    Ok(())
                })
                .unwrap();

            assert_eq!(repo.find_dir(id).unwrap().unwrap(), record);
        }

        #[test]
        fn find_file_by_path_returns_record_when_path_exists() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsParentId::Id(FsRecordId::new());
            let path = PathKey::try_new("test").unwrap();
            let name = FileName::new("test".into());
            let format = FileFormat::Unknown;
            let metadata =
                FileMetadata::new(FsTimes::new(None, None), 0, false);
            let recorded_at = SystemTime::now();

            let record = FileRecord::new(
                id,
                parent_id,
                path.clone(),
                name,
                format,
                metadata,
                recorded_at,
            );
            // Seed both primary and secondary tables
            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_table =
                        tx.inner.open_table(FILES.definition()).unwrap();
                    f_table.insert(id, &file_bytes(&record)).unwrap();
                    let mut p_table = tx
                        .inner
                        .open_table(FILE_ID_BY_PATH.definition())
                        .unwrap();
                    p_table.insert(DbPathKey::from(&path), id).unwrap();
                    Ok(())
                })
                .unwrap();

            assert_eq!(repo.find_file_by_path(&path).unwrap().unwrap(), record);
        }

        #[test]
        fn find_dir_by_path_returns_record_when_path_exists() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let path = PathKey::try_new("test_dir").unwrap();
            let name = traces_fs::name::DirName::new("test_dir".into());
            let metadata =
                traces_fs::DirMetadata::new(FsTimes::new(None, None), false);
            let recorded_at = SystemTime::now();

            let record = DirRecord::new(
                id,
                FsParentId::Root,
                path.clone(),
                name,
                metadata,
                recorded_at,
            );

            // Seed both primary and secondary tables
            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut d_table =
                        tx.inner.open_table(DIRS.definition()).unwrap();
                    d_table.insert(id, &dir_bytes(&record)).unwrap();
                    let mut p_table = tx
                        .inner
                        .open_table(DIR_ID_BY_PATH.definition())
                        .unwrap();
                    p_table.insert(DbPathKey::from(&path), id).unwrap();
                    Ok(())
                })
                .unwrap();

            assert_eq!(repo.find_dir_by_path(&path).unwrap().unwrap(), record);
        }

        #[test]
        fn returns_files_for_basename() {
            let (_tempdir, repo) = setup_repo();
            let basename = "target.txt";

            let file1_id = FsRecordId::new();
            let file1 = FileRecord::new(
                file1_id,
                FsParentId::Id(FsRecordId::new()),
                PathKey::try_new("dir1/target.txt").unwrap(),
                FileName::new("target.txt".into()),
                FileFormat::Unknown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            let file2_id = FsRecordId::new();
            let file2 = FileRecord::new(
                file2_id,
                FsParentId::Id(FsRecordId::new()),
                PathKey::try_new("dir2/target.txt").unwrap(),
                FileName::new("target.txt".into()),
                FileFormat::Unknown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_table =
                        tx.inner.open_table(FILES.definition()).unwrap();
                    f_table.insert(file1_id, &file_bytes(&file1)).unwrap();
                    f_table.insert(file2_id, &file_bytes(&file2)).unwrap();

                    let mut p_table = tx
                        .inner
                        .open_multimap_table(FILE_IDS_BY_BASENAME)
                        .unwrap();
                    p_table.insert(basename, file1_id).unwrap();
                    p_table.insert(basename, file2_id).unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.list_files_by_basename(basename).unwrap();
            assert_eq!(results.len(), 2);
            assert!(results.contains(&file1));
            assert!(results.contains(&file2));
        }
    }

    mod list {
        #[allow(unused_imports, reason = "added globally for ease")]
        use pretty_assertions::{assert_eq, assert_ne};

        use super::*;
        use crate::storage::tables::{DIR_IDS_BY_PARENT, FILE_IDS_BY_PARENT};

        #[test]
        fn returns_files_for_parent() {
            let (_tempdir, repo) = setup_repo();
            let parent_key = FsRecordId::new();
            let parent_id = FsParentId::Id(parent_key);

            let file1_id = FsRecordId::new();
            let file1 = FileRecord::new(
                file1_id,
                parent_id,
                PathKey::try_new("parent/file1").unwrap(),
                FileName::new("file1".into()),
                FileFormat::Unknown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            let file2_id = FsRecordId::new();
            let file2 = FileRecord::new(
                file2_id,
                parent_id,
                PathKey::try_new("parent/file2").unwrap(),
                FileName::new("file2".into()),
                FileFormat::Unknown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            let other_id = FsRecordId::new();
            let other_file = FileRecord::new(
                other_id,
                FsParentId::Id(FsRecordId::new()),
                PathKey::try_new("other/file").unwrap(),
                FileName::new("file".into()),
                FileFormat::Unknown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_table =
                        tx.inner.open_table(FILES.definition()).unwrap();
                    f_table.insert(file1_id, &file_bytes(&file1)).unwrap();
                    f_table.insert(file2_id, &file_bytes(&file2)).unwrap();
                    f_table.insert(other_id, &file_bytes(&other_file)).unwrap();

                    let mut p_table = tx
                        .inner
                        .open_multimap_table(FILE_IDS_BY_PARENT.definition())
                        .unwrap();
                    p_table.insert(parent_key, file1_id).unwrap();
                    p_table.insert(parent_key, file2_id).unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.list_files_by_parent(parent_id).unwrap();
            assert_eq!(results.len(), 2);
            assert!(results.contains(&file1));
            assert!(results.contains(&file2));
        }

        #[test]
        fn returns_dirs_for_parent() {
            let (_tempdir, repo) = setup_repo();
            let parent_key = FsRecordId::new();
            let parent_id = FsParentId::Id(parent_key);

            let dir1_id = FsRecordId::new();
            let dir1 = DirRecord::new(
                dir1_id,
                parent_id,
                PathKey::try_new("parent/dir1").unwrap(),
                traces_fs::name::DirName::new("dir1".into()),
                traces_fs::DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            );

            let dir2_id = FsRecordId::new();
            let dir2 = DirRecord::new(
                dir2_id,
                parent_id,
                PathKey::try_new("parent/dir2").unwrap(),
                traces_fs::name::DirName::new("dir2".into()),
                traces_fs::DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            );

            let other_id = FsRecordId::new();
            let other_dir = DirRecord::new(
                other_id,
                FsParentId::Root,
                PathKey::try_new("root_dir").unwrap(),
                traces_fs::name::DirName::new("root_dir".into()),
                traces_fs::DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            );

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut d_table =
                        tx.inner.open_table(DIRS.definition()).unwrap();
                    d_table.insert(dir1_id, &dir_bytes(&dir1)).unwrap();
                    d_table.insert(dir2_id, &dir_bytes(&dir2)).unwrap();
                    d_table.insert(other_id, &dir_bytes(&other_dir)).unwrap();

                    let mut p_table = tx
                        .inner
                        .open_multimap_table(DIR_IDS_BY_PARENT.definition())
                        .unwrap();
                    p_table.insert(parent_key, dir1_id).unwrap();
                    p_table.insert(parent_key, dir2_id).unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.list_dirs_by_parent(parent_id).unwrap();
            assert_eq!(results.len(), 2);
            assert!(results.contains(&dir1));
            assert!(results.contains(&dir2));
        }

        #[test]
        fn returns_all_paths() {
            let (_tempdir, repo) = setup_repo();

            let file_path = PathKey::try_new("dir/file.txt").unwrap();
            let dir_path = PathKey::try_new("dir").unwrap();

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_p_table = tx
                        .inner
                        .open_table(FILE_ID_BY_PATH.definition())
                        .unwrap();
                    f_p_table
                        .insert(DbPathKey::from(&file_path), FsRecordId::new())
                        .unwrap();

                    let mut d_p_table = tx
                        .inner
                        .open_table(DIR_ID_BY_PATH.definition())
                        .unwrap();
                    d_p_table
                        .insert(DbPathKey::from(&dir_path), FsRecordId::new())
                        .unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.all_paths().unwrap();
            assert_eq!(results.len(), 2);
            assert!(results.contains(&file_path));
            assert!(results.contains(&dir_path));
        }

        #[test]
        fn list_files_by_parent_root() {
            use crate::repository::WriteRepository;

            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let record = FileRecord::new(
                id,
                FsParentId::Root,
                PathKey::try_new("root_file.md").unwrap(),
                FileName::new("root_file.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );
            repo.save_file(&record).unwrap();

            let results = repo.list_files_by_parent(FsParentId::Root).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results.first(), Some(&record));
        }

        #[test]
        fn all_paths_deduplicates_across_file_and_dir_tables() {
            let (_tempdir, repo) = setup_repo();
            let shared = PathKey::try_new("shared").unwrap();

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_p_table = tx
                        .inner
                        .open_table(FILE_ID_BY_PATH.definition())
                        .unwrap();
                    f_p_table
                        .insert(DbPathKey::from(&shared), FsRecordId::new())
                        .unwrap();

                    let mut d_p_table = tx
                        .inner
                        .open_table(DIR_ID_BY_PATH.definition())
                        .unwrap();
                    d_p_table
                        .insert(DbPathKey::from(&shared), FsRecordId::new())
                        .unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.all_paths().unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results.first(), Some(&shared));
        }
    }

    mod filter {
        #[allow(unused_imports, reason = "added globally for ease")]
        use pretty_assertions::{assert_eq, assert_ne};

        use super::*;
        use crate::storage::tables::FILE_IDS_BY_FORMAT;

        #[test]
        fn returns_files_for_format() {
            let (_tempdir, repo) = setup_repo();
            let format = FileFormat::Markdown;

            let file1_id = FsRecordId::new();
            let file1 = FileRecord::new(
                file1_id,
                FsParentId::Id(FsRecordId::new()),
                PathKey::try_new("file1.md").unwrap(),
                FileName::new("file1.md".into()),
                format,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            let other_format = FileFormat::Unknown;
            let file2_id = FsRecordId::new();
            let file2 = FileRecord::new(
                file2_id,
                FsParentId::Id(FsRecordId::new()),
                PathKey::try_new("file2.txt").unwrap(),
                FileName::new("file2.txt".into()),
                other_format,
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::now(),
            );

            repo.store
                .write(|tx| -> Result<(), DbError> {
                    let mut f_table =
                        tx.inner.open_table(FILES.definition()).unwrap();
                    f_table.insert(file1_id, &file_bytes(&file1)).unwrap();
                    f_table.insert(file2_id, &file_bytes(&file2)).unwrap();

                    let mut p_table = tx
                        .inner
                        .open_multimap_table(FILE_IDS_BY_FORMAT)
                        .unwrap();
                    p_table.insert(format.as_str(), file1_id).unwrap();
                    p_table.insert(other_format.as_str(), file2_id).unwrap();
                    Ok(())
                })
                .unwrap();

            let results = repo.list_files_by_format(format).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results.first(), Some(&file1));
        }
    }
}
