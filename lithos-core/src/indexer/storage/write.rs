//! Redb-backed write repository implementation.
//!
//! Implements [`WriteRepository`] for [`RedbRepository`] with save and delete
//! operations that maintain primary tables and secondary indexes within
//! single-transaction boundaries.
//!
//! Updates follow a load → remove stale indexes → insert new data pattern
//! to keep all indexes consistent.
//!
//! [`WriteRepository`]: crate::indexer::repository::WriteRepository

use redb::{ReadableMultimapTable, ReadableTable, WriteTransaction};

use crate::{
    db::DbError,
    indexer::{
        error::IndexerRepositoryError,
        model::{DirRecord, FileRecord, FsRecordId},
        repository::WriteRepository,
        storage::{
            RedbRepository,
            tables::{
                DIR_ID_BY_PATH, DIR_IDS_BY_PARENT, DIRS, FILE_ID_BY_PATH,
                FILE_IDS_BY_BASENAME, FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT,
                FILES,
            },
        },
    },
};

impl RedbRepository {
    fn load_file_delete_context(
        tx: &crate::db::WriteTx,
        id: FsRecordId,
    ) -> Result<Option<FileRecord>, DbError> {
        let table = tx.inner.open_table(FILES.definition())?;
        table
            .get(id)?
            .map(|guard| {
                rkyv::from_bytes::<FileRecord, rkyv::rancor::Error>(
                    guard.value(),
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .transpose()
    }

    fn load_dir_delete_context(
        tx: &crate::db::WriteTx,
        id: FsRecordId,
    ) -> Result<Option<DirRecord>, DbError> {
        let table = tx.inner.open_table(DIRS.definition())?;
        table
            .get(id)?
            .map(|guard| {
                rkyv::from_bytes::<DirRecord, rkyv::rancor::Error>(
                    guard.value(),
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))
            })
            .transpose()
    }

    fn remove_file_graph(
        tx: &crate::db::WriteTx,
        record: &FileRecord,
    ) -> Result<(), DbError> {
        let mut path_table =
            tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
        path_table.remove(record.path())?;

        let mut basename_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
        basename_table.remove(record.name().as_str(), record.id())?;

        let mut parent_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_PARENT.definition())?;
        parent_table.remove(record.parent_id(), record.id())?;

        let mut format_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
        format_table.remove(record.format().as_str(), record.id())?;

        let mut files_table = tx.inner.open_table(FILES.definition())?;
        files_table.remove(record.id())?;

        Ok(())
    }

    fn remove_dir_graph(
        tx: &crate::db::WriteTx,
        record: &DirRecord,
    ) -> Result<(), DbError> {
        let mut path_table =
            tx.inner.open_table(DIR_ID_BY_PATH.definition())?;
        path_table.remove(record.path())?;

        if let Some(parent_id) = record.parent_id() {
            let mut parent_table =
                tx.inner.open_multimap_table(DIR_IDS_BY_PARENT.definition())?;
            parent_table.remove(parent_id, record.id())?;
        }

        let mut dirs_table = tx.inner.open_table(DIRS.definition())?;
        dirs_table.remove(record.id())?;

        Ok(())
    }

    fn save_file_in_tx(
        tx: &crate::db::WriteTx,
        record: &FileRecord,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        // If this is an update, clean up stale graph entries
        if let Some(old) = Self::load_file_delete_context(tx, record.id())? {
            Self::remove_file_graph(tx, &old)?;
        }

        // Primary table
        let mut files_table = tx.inner.open_table(FILES.definition())?;
        files_table.insert(record.id(), bytes)?;

        // Secondary indexes
        let mut path_table =
            tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
        path_table.insert(record.path(), record.id())?;

        let mut basename_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
        basename_table.insert(record.name().as_str(), record.id())?;

        let mut parent_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_PARENT.definition())?;
        parent_table.insert(record.parent_id(), record.id())?;

        let mut format_table =
            tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
        format_table.insert(record.format().as_str(), record.id())?;

        Ok(())
    }

    fn save_dir_in_tx(
        tx: &crate::db::WriteTx,
        record: &DirRecord,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        // If this is an update, clean up stale graph entries
        if let Some(old) = Self::load_dir_delete_context(tx, record.id())? {
            Self::remove_dir_graph(tx, &old)?;
        }

        // Primary table
        let mut dirs_table = tx.inner.open_table(DIRS.definition())?;
        dirs_table.insert(record.id(), bytes)?;

        // Secondary indexes
        let mut path_table =
            tx.inner.open_table(DIR_ID_BY_PATH.definition())?;
        path_table.insert(record.path(), record.id())?;

        if let Some(parent_id) = record.parent_id() {
            let mut parent_table =
                tx.inner.open_multimap_table(DIR_IDS_BY_PARENT.definition())?;
            parent_table.insert(parent_id, record.id())?;
        }

        Ok(())
    }

    fn delete_file_in_tx(
        tx: &crate::db::WriteTx,
        id: FsRecordId,
    ) -> Result<(), DbError> {
        // Load the record first to know what to remove from indexes
        // We map to the owned value to drop the AccessGuard immediately
        if let Some(record) = Self::load_file_delete_context(tx, id)? {
            Self::remove_file_graph(tx, &record)?;
        }

        Ok(())
    }

    fn delete_dir_in_tx(
        tx: &crate::db::WriteTx,
        id: FsRecordId,
    ) -> Result<(), DbError> {
        // Load the record first to know what to remove from indexes
        // We map to the owned value to drop the AccessGuard immediately
        if let Some(record) = Self::load_dir_delete_context(tx, id)? {
            Self::remove_dir_graph(tx, &record)?;
        }

        Ok(())
    }
}

impl WriteRepository for RedbRepository {
    fn save_file(
        &self,
        record: &FileRecord,
    ) -> Result<(), IndexerRepositoryError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(record)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        self.store
            .write(|tx| Self::save_file_in_tx(tx, record, bytes.as_slice()))
            .map_err(Into::into)
    }

    fn save_dir(
        &self,
        record: &DirRecord,
    ) -> Result<(), IndexerRepositoryError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(record)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        self.store
            .write(|tx| Self::save_dir_in_tx(tx, record, bytes.as_slice()))
            .map_err(Into::into)
    }

    fn delete_file(
        &self,
        id: FsRecordId,
    ) -> Result<(), IndexerRepositoryError> {
        self.store
            .write(|tx| Self::delete_file_in_tx(tx, id))
            .map_err(Into::into)
    }

    fn delete_dir(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError> {
        self.store
            .write(|tx| Self::delete_dir_in_tx(tx, id))
            .map_err(Into::into)
    }

    fn save_many_records(
        &self,
        files: &[FileRecord],
        dirs: &[DirRecord],
    ) -> Result<(), IndexerRepositoryError> {
        let file_archives: Vec<_> = files
            .iter()
            .map(rkyv::to_bytes::<rkyv::rancor::Error>)
            .collect::<Result<_, _>>()
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let dir_archives: Vec<_> = dirs
            .iter()
            .map(rkyv::to_bytes::<rkyv::rancor::Error>)
            .collect::<Result<_, _>>()
            .map_err(|e| DbError::Serialization(e.to_string()))?;

        self.store
            .write(|tx| {
                for (file, archive) in files.iter().zip(file_archives.iter()) {
                    Self::save_file_in_tx(tx, file, archive.as_slice())?;
                }
                for (dir, archive) in dirs.iter().zip(dir_archives.iter()) {
                    Self::save_dir_in_tx(tx, dir, archive.as_slice())?;
                }
                Ok(())
            })
            .map_err(Into::into)
    }

    fn delete_many_records(
        &self,
        file_ids: &[FsRecordId],
        dir_ids: &[FsRecordId],
    ) -> Result<(), IndexerRepositoryError> {
        self.store
            .write(|tx| {
                for id in file_ids {
                    Self::delete_file_in_tx(tx, *id)?;
                }
                for id in dir_ids {
                    Self::delete_dir_in_tx(tx, *id)?;
                }
                Ok(())
            })
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::SystemTime};

    use crate::{
        db::Store,
        fs::{
            FileFormat, FileMetadata, metadata::FsTimes, name::FileName,
            path::PathKey,
        },
        indexer::{
            model::{DirRecord, FileRecord, FsRecordId},
            repository::{ReadRepository, WriteRepository},
            storage::RedbRepository,
        },
    };

    fn setup_repo() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        (tempdir, RedbRepository::try_new(Arc::new(store)).unwrap())
    }

    mod create {
        use super::*;

        #[test]
        fn save_file_persists_primary_and_indexes() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let path = PathKey::try_new("dir/file.txt").unwrap();
            let name = FileName::new("file.txt".into());
            let format = FileFormat::Markdown;
            let metadata =
                FileMetadata::new(FsTimes::new(None, None), 123, false);
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

            repo.save_file(&record).unwrap();

            // Assert primary
            let found =
                repo.find_file(id).unwrap().expect("primary record missing");
            assert_eq!(found, record);

            // Assert path index
            let found_by_path = repo
                .find_file_by_path(&path)
                .unwrap()
                .expect("path index missing");
            assert_eq!(found_by_path, record);

            // Assert basename index
            let found_by_basename =
                repo.list_files_by_basename("file.txt").unwrap();
            assert!(found_by_basename.contains(&record));

            // Assert parent index
            let found_by_parent = repo.list_files_by_parent(parent_id).unwrap();
            assert!(found_by_parent.contains(&record));

            // Assert format index
            let found_by_format = repo.list_files_by_format(format).unwrap();
            assert!(found_by_format.contains(&record));
        }

        #[test]
        fn save_dir_persists_primary_and_path_index() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let path = PathKey::try_new("dir/subdir").unwrap();
            let name = crate::fs::name::DirName::new("subdir".into());
            let metadata =
                crate::fs::DirMetadata::new(FsTimes::new(None, None), false);
            let recorded_at = SystemTime::now();

            let record = DirRecord::new(
                id,
                Some(parent_id),
                path.clone(),
                name,
                metadata,
                recorded_at,
            );

            repo.save_dir(&record).unwrap();

            // Assert primary
            let found =
                repo.find_dir(id).unwrap().expect("primary record missing");
            assert_eq!(found, record);

            // Assert path index
            let found_by_path = repo
                .find_dir_by_path(&path)
                .unwrap()
                .expect("path index missing");
            assert_eq!(found_by_path, record);

            // Assert parent listing (verifies
            // ReadRepository::list_dirs_by_parent)
            let found_by_parent = repo.list_dirs_by_parent(parent_id).unwrap();
            assert!(found_by_parent.contains(&record));
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn delete_file_removes_primary_and_indexes() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let path = PathKey::try_new("dir/file.txt").unwrap();
            let name = FileName::new("file.txt".into());
            let format = FileFormat::Markdown;
            let metadata =
                FileMetadata::new(FsTimes::new(None, None), 123, false);
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

            repo.save_file(&record).unwrap();

            // Verify it exists
            assert!(repo.find_file(id).unwrap().is_some());

            // Delete
            repo.delete_file(id).unwrap();

            // Assert primary removed
            assert!(repo.find_file(id).unwrap().is_none());

            // Assert indexes removed
            assert!(repo.find_file_by_path(&path).unwrap().is_none());
            assert!(
                repo.list_files_by_basename("file.txt").unwrap().is_empty()
            );
            assert!(repo.list_files_by_parent(parent_id).unwrap().is_empty());
            assert!(repo.list_files_by_format(format).unwrap().is_empty());
        }

        #[test]
        fn delete_file_is_idempotent_when_missing() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();

            // Should not error when deleting non-existent file
            repo.delete_file(id).unwrap();
        }

        #[test]
        fn delete_dir_removes_primary_and_path_index() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let path = PathKey::try_new("dir/subdir").unwrap();
            let name = crate::fs::name::DirName::new("subdir".into());
            let metadata =
                crate::fs::DirMetadata::new(FsTimes::new(None, None), false);
            let recorded_at = SystemTime::now();

            let record = DirRecord::new(
                id,
                Some(parent_id),
                path.clone(),
                name,
                metadata,
                recorded_at,
            );

            repo.save_dir(&record).unwrap();

            // Verify it exists
            assert!(repo.find_dir(id).unwrap().is_some());

            // Delete
            repo.delete_dir(id).unwrap();

            // Assert primary removed
            assert!(repo.find_dir(id).unwrap().is_none());

            // Assert path index removed
            assert!(repo.find_dir_by_path(&path).unwrap().is_none());
        }

        #[test]
        fn delete_dir_is_idempotent_when_missing() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();

            // Should not error when deleting non-existent directory
            repo.delete_dir(id).unwrap();
        }
    }

    mod update {
        use super::*;

        #[test]
        fn save_file_cleans_stale_indexes_when_record_changes() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let old_path = PathKey::try_new("old/file.txt").unwrap();
            let old_name = FileName::new("file.txt".into());
            let old_format = FileFormat::Markdown;
            let metadata =
                FileMetadata::new(FsTimes::new(None, None), 123, false);
            let recorded_at = SystemTime::now();

            let old_record = FileRecord::new(
                id,
                parent_id,
                old_path.clone(),
                old_name,
                old_format,
                metadata.clone(),
                recorded_at,
            );

            repo.save_file(&old_record).unwrap();

            // Update record with new properties
            let new_path = PathKey::try_new("new/file.txt").unwrap();
            let new_format = FileFormat::Json;
            let new_record = FileRecord::new(
                id,
                parent_id,
                new_path.clone(),
                FileName::new("file.txt".into()),
                new_format,
                metadata,
                recorded_at,
            );

            repo.save_file(&new_record).unwrap();

            // Assert old indexes are cleaned up
            assert!(
                repo.find_file_by_path(&old_path).unwrap().is_none(),
                "old path index should be removed"
            );
            let format_results = repo.list_files_by_format(old_format).unwrap();
            assert!(
                !format_results.contains(&new_record),
                "old format index should not contain new record"
            );
            assert!(
                format_results.is_empty(),
                "old format index should be empty"
            );

            // Assert new indexes are present
            assert_eq!(
                repo.find_file_by_path(&new_path).unwrap().unwrap(),
                new_record
            );
            assert!(
                repo.list_files_by_format(new_format)
                    .unwrap()
                    .contains(&new_record)
            );
        }

        #[test]
        fn save_dir_cleans_stale_path_index_when_path_changes() {
            let (_tempdir, repo) = setup_repo();
            let id = FsRecordId::new();
            let parent_id = FsRecordId::new();
            let old_path = PathKey::try_new("old/dir").unwrap();
            let name = crate::fs::name::DirName::new("dir".into());
            let metadata =
                crate::fs::DirMetadata::new(FsTimes::new(None, None), false);
            let recorded_at = SystemTime::now();

            let old_record = DirRecord::new(
                id,
                Some(parent_id),
                old_path.clone(),
                name.clone(),
                metadata.clone(),
                recorded_at,
            );

            repo.save_dir(&old_record).unwrap();

            // Update record with new path
            let new_path = PathKey::try_new("new/dir").unwrap();
            let new_record = DirRecord::new(
                id,
                Some(parent_id),
                new_path.clone(),
                name,
                metadata,
                recorded_at,
            );

            repo.save_dir(&new_record).unwrap();

            // Assert old path index is cleaned up
            assert!(
                repo.find_dir_by_path(&old_path).unwrap().is_none(),
                "old path index should be removed"
            );

            // Assert new path index is present
            assert_eq!(
                repo.find_dir_by_path(&new_path).unwrap().unwrap(),
                new_record
            );
        }
    }

    mod transactions {
        use super::*;

        #[test]
        fn save_many_records_persists_files_and_dirs_together() {
            let (_tempdir, repo) = setup_repo();

            let f_id = FsRecordId::new();
            let f_path = PathKey::try_new("file.txt").unwrap();
            let file = FileRecord::new(
                f_id,
                FsRecordId::new(),
                f_path.clone(),
                FileName::new("file.txt".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 123, false),
                SystemTime::now(),
            );

            let d_id = FsRecordId::new();
            let d_path = PathKey::try_new("subdir").unwrap();
            let dir = DirRecord::new(
                d_id,
                None,
                d_path.clone(),
                crate::fs::name::DirName::new("subdir".into()),
                crate::fs::DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            );

            repo.save_many_records(
                std::slice::from_ref(&file),
                std::slice::from_ref(&dir),
            )
            .unwrap();

            // Verify both are present
            assert_eq!(repo.find_file(f_id).unwrap().unwrap(), file);
            assert_eq!(repo.find_dir(d_id).unwrap().unwrap(), dir);

            // Verify indexes
            assert_eq!(repo.find_file_by_path(&f_path).unwrap().unwrap(), file);
            assert_eq!(repo.find_dir_by_path(&d_path).unwrap().unwrap(), dir);
        }

        #[test]
        fn delete_many_records_removes_files_and_dirs_together() {
            let (_tempdir, repo) = setup_repo();

            // Setup: 1 file and 1 dir
            let f_id = FsRecordId::new();
            let f_path = PathKey::try_new("file.txt").unwrap();
            let file = FileRecord::new(
                f_id,
                FsRecordId::new(),
                f_path.clone(),
                FileName::new("file.txt".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 123, false),
                SystemTime::now(),
            );

            let d_id = FsRecordId::new();
            let d_path = PathKey::try_new("subdir").unwrap();
            let dir = DirRecord::new(
                d_id,
                None,
                d_path.clone(),
                crate::fs::name::DirName::new("subdir".into()),
                crate::fs::DirMetadata::new(FsTimes::new(None, None), false),
                SystemTime::now(),
            );

            repo.save_many_records(
                std::slice::from_ref(&file),
                std::slice::from_ref(&dir),
            )
            .unwrap();

            // Verify they exist
            assert!(repo.find_file(f_id).unwrap().is_some());
            assert!(repo.find_dir(d_id).unwrap().is_some());

            // Delete many
            repo.delete_many_records(&[f_id], &[d_id]).unwrap();

            // Verify both removed
            assert!(repo.find_file(f_id).unwrap().is_none());
            assert!(repo.find_dir(d_id).unwrap().is_none());

            // Verify indexes removed
            assert!(repo.find_file_by_path(&f_path).unwrap().is_none());
            assert!(repo.find_dir_by_path(&d_path).unwrap().is_none());
        }
    }
}
