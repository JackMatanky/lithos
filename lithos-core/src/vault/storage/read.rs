//! Read-only repository operations for vault files and directories.
//!
//! Implements the [`ReadRepository`] trait against redb storage. Each method
//! opens a read transaction and uses
//! [`ReadTransaction::try_open_table`](redb::ReadTransaction::try_open_table)
//! to handle uninitialized tables gracefully, returning `Ok(None)` or an
//! empty vector when tables do not yet exist.
//!
//! The trait defines per-method documentation; this file contains the
//! concrete [`redb`] access patterns for each operation.

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
        FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
    },
};
use crate::{
    db::ArchivedEntity,
    fs::{FileFormat, PathKey},
    vault::{
        error::VaultRepositoryError,
        model::{DirId, DirView, FileId, FileView, FsEntryView},
        repository::ReadRepository,
    },
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn get_file_view(
        &self,
        id: FileId,
    ) -> Result<Option<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(None);
                };
                table
                    .get(&id)?
                    .map(|g| FileView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn get_dir_view(
        &self,
        id: DirId,
    ) -> Result<Option<DirView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(DIR_VIEWS.definition())?
                else {
                    return Ok(None);
                };
                table
                    .get(&id)?
                    .map(|g| DirView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn find_file_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(path_table) =
                    tx.try_open_table(FILE_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };
                let Some(file_table) =
                    tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(None);
                };

                path_table
                    .get(path)?
                    .map(|id| file_table.get(&id.value()))
                    .transpose()?
                    .flatten()
                    .map(|g| FileView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn find_dir_view_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(path_table) =
                    tx.try_open_table(DIR_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };
                let Some(dir_table) =
                    tx.try_open_table(DIR_VIEWS.definition())?
                else {
                    return Ok(None);
                };

                path_table
                    .get(path)?
                    .map(|id| dir_table.get(&id.value()))
                    .transpose()?
                    .flatten()
                    .map(|g| DirView::from_bytes(g.value()))
                    .transpose()
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn get_entry(
        &self,
        path: &PathKey,
    ) -> Result<Option<FsEntryView>, VaultRepositoryError> {
        // Try file first
        if let Some(file) = self.find_file_view_by_path(path)? {
            return Ok(Some(FsEntryView::File(file)));
        }
        // Then try dir
        if let Some(dir) = self.find_dir_view_by_path(path)? {
            return Ok(Some(FsEntryView::Dir(dir)));
        }
        Ok(None)
    }

    #[inline]
    fn find_file_views_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(multimap) =
                    tx.try_open_multimap(FILE_IDS_BY_BASENAME)?
                else {
                    return Ok(Vec::new());
                };
                let Some(file_table) =
                    tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut files = Vec::new();
                for id_result in multimap.get(basename)? {
                    if let Some(guard) = file_table.get(&id_result?.value())? {
                        files.push(FileView::from_bytes(guard.value())?);
                    }
                }

                Ok(files)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn find_file_views_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(multimap) =
                    tx.try_open_multimap(FILE_IDS_BY_PARENT.definition())?
                else {
                    return Ok(Vec::new());
                };
                let Some(file_table) =
                    tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut files = Vec::new();
                for id_result in multimap.get(&parent_id)? {
                    if let Some(guard) = file_table.get(&id_result?.value())? {
                        files.push(FileView::from_bytes(guard.value())?);
                    }
                }

                Ok(files)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_file_views_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(multimap) =
                    tx.try_open_multimap(FILE_IDS_BY_FORMAT)?
                else {
                    return Ok(Vec::new());
                };
                let Some(file_table) =
                    tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let format_key = format.as_str();
                let mut files = Vec::new();
                for id_result in multimap.get(format_key)? {
                    if let Some(guard) = file_table.get(&id_result?.value())? {
                        files.push(FileView::from_bytes(guard.value())?);
                    }
                }

                Ok(files)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_markdown_file_views(
        &self,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.list_file_views_by_format(FileFormat::Markdown)
    }

    #[inline]
    fn list_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(FILE_VIEWS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut files = Vec::new();
                for result in table.iter()? {
                    let (_, guard) = result?;
                    files.push(FileView::from_bytes(guard.value())?);
                }

                Ok(files)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_file_paths(&self) -> Result<Vec<PathKey>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(FILE_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut paths = Vec::new();
                for result in table.iter()? {
                    let (guard, _) = result?;
                    paths.push(guard.value().clone());
                }

                Ok(paths)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_dir_views(&self) -> Result<Vec<DirView>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(DIR_VIEWS.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut dirs = Vec::new();
                for result in table.iter()? {
                    let (_, guard) = result?;
                    dirs.push(DirView::from_bytes(guard.value())?);
                }

                Ok(dirs)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_dir_paths(&self) -> Result<Vec<PathKey>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(DIR_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut paths = Vec::new();
                for result in table.iter()? {
                    let (guard, _) = result?;
                    paths.push(guard.value().clone());
                }

                Ok(paths)
            })
            .map_err(VaultRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    //! Tests for vault read operations.
    //!
    //! Each test opens an isolated temp database, seeds state via low-level
    //! store access, and asserts [`ReadRepository`] behavior. Tests are
    //! organised by capability following the Structure A convention.
    use std::sync::Arc;

    use super::*;
    use crate::{
        db::{ArchivedEntity, Store},
        fs::{
            DirMetadata, DirName, FileFormat, FileMetadata, FileName, FsTimes,
            PathKey,
        },
        vault::{
            model::{DirId, DirView, FileId, FileView},
            repository::ReadRepository,
            storage::RedbRepository,
        },
    };

    /// Creates a temp database with a [`RedbRepository`] ready for testing.
    fn temp_vault() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        (tempdir, repo)
    }

    /// Seeds a [`FileView`] into the `FILE_VIEWS` table.
    fn seed_file(repo: &RedbRepository, file: &FileView) {
        let bytes = file.to_bytes().unwrap();
        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::FILE_VIEWS.definition())?;
                table.insert(&file.id(), bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();
    }

    /// Seeds a [`DirView`] into the `DIR_VIEWS` table.
    fn seed_dir(repo: &RedbRepository, dir: &DirView) {
        let bytes = dir.to_bytes().unwrap();
        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::DIR_VIEWS.definition())?;
                table.insert(&dir.id(), bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();
    }

    /// Direct ID lookups for files and directories.
    mod accessors {
        use super::*;

        #[test]
        fn get_file_view_returns_none_when_id_missing() {
            let (_tempdir, repo) = temp_vault();
            let result = repo.get_file_view(FileId::new());
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn get_file_view_returns_view_when_id_exists() {
            let (_tempdir, repo) = temp_vault();
            let file = FileView::new(
                FileId::new(),
                None,
                FileName::new("test.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [0u8; 32],
            );
            seed_file(&repo, &file);

            let result = repo.get_file_view(file.id());
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let retrieved = result.unwrap();
            assert!(retrieved.is_some(), "Expected Some, got None");
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.id(), file.id());
            assert_eq!(retrieved.name(), file.name());
        }

        #[test]
        fn get_dir_view_returns_none_when_id_missing() {
            let (_tempdir, repo) = temp_vault();
            let result = repo.get_dir_view(DirId::new());
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn get_dir_view_returns_view_when_id_exists() {
            let (_tempdir, repo) = temp_vault();
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("test".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            seed_dir(&repo, &dir);

            let result = repo.get_dir_view(dir.id());
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let retrieved = result.unwrap();
            assert!(retrieved.is_some(), "Expected Some, got None");
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.id(), dir.id());
            assert_eq!(retrieved.name(), dir.name());
        }
    }

    /// Path-based lookups and entry resolution.
    mod lookup {
        use super::*;

        #[test]
        fn find_file_view_by_path_returns_none_when_path_missing() {
            let (_tempdir, repo) = temp_vault();
            let path = PathKey::try_new("missing.md").unwrap();
            let result = repo.find_file_view_by_path(&path);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn find_file_view_by_path_returns_view_when_path_exists() {
            let (_tempdir, repo) = temp_vault();
            let file = FileView::new(
                FileId::new(),
                None,
                FileName::new("test.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [1u8; 32],
            );
            let path = PathKey::try_new("notes/test.md").unwrap();
            let file_bytes = file.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut path_table =
                        tx.try_open_table(super::FILE_ID_BY_PATH.definition())?;
                    file_table.insert(&file.id(), file_bytes.as_ref())?;
                    path_table.insert(&path, &file.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.find_file_view_by_path(&path);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let retrieved = result.unwrap();
            assert!(retrieved.is_some(), "Expected Some, got None");
            assert_eq!(retrieved.unwrap().id(), file.id());
        }

        #[test]
        fn find_dir_view_by_path_returns_view_when_path_exists() {
            let (_tempdir, repo) = temp_vault();
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("notes".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            let path = PathKey::try_new("notes").unwrap();
            let dir_bytes = dir.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut dir_table =
                        tx.try_open_table(super::DIR_VIEWS.definition())?;
                    let mut path_table =
                        tx.try_open_table(super::DIR_ID_BY_PATH.definition())?;
                    dir_table.insert(&dir.id(), dir_bytes.as_ref())?;
                    path_table.insert(&path, &dir.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.find_dir_view_by_path(&path);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let retrieved = result.unwrap();
            assert!(retrieved.is_some(), "Expected Some, got None");
            assert_eq!(retrieved.unwrap().id(), dir.id());
        }

        #[test]
        fn get_entry_prefers_file_when_both_exist() {
            let (_tempdir, repo) = temp_vault();
            let file = FileView::new(
                FileId::new(),
                None,
                FileName::new("test".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [2u8; 32],
            );
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("test".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            let path = PathKey::try_new("test").unwrap();
            let file_bytes = file.to_bytes().unwrap();
            let dir_bytes = dir.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut file_path_table =
                        tx.try_open_table(super::FILE_ID_BY_PATH.definition())?;
                    let mut dir_table =
                        tx.try_open_table(super::DIR_VIEWS.definition())?;
                    let mut dir_path_table =
                        tx.try_open_table(super::DIR_ID_BY_PATH.definition())?;
                    file_table.insert(&file.id(), file_bytes.as_ref())?;
                    file_path_table.insert(&path, &file.id())?;
                    dir_table.insert(&dir.id(), dir_bytes.as_ref())?;
                    dir_path_table.insert(&path, &dir.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.get_entry(&path);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let entry = result.unwrap();
            assert!(entry.is_some(), "Expected Some, got None");
            assert!(entry.unwrap().is_file());
        }

        #[test]
        fn get_entry_returns_none_when_path_missing() {
            let (_tempdir, repo) = temp_vault();
            let path = PathKey::try_new("missing").unwrap();
            let result = repo.get_entry(&path);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_none());
        }
    }

    /// Multimap index queries (basename, parent).
    mod indexes {
        use super::*;

        #[test]
        fn find_file_views_by_basename_returns_matches() {
            let (_tempdir, repo) = temp_vault();
            let file1 = FileView::new(
                FileId::new(),
                None,
                FileName::new("shared.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [3u8; 32],
            );
            let file2 = FileView::new(
                FileId::new(),
                None,
                FileName::new("shared.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 256, false),
                [4u8; 32],
            );
            let f1_bytes = file1.to_bytes().unwrap();
            let f2_bytes = file2.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut basename_map =
                        tx.try_open_multimap(super::FILE_IDS_BY_BASENAME)?;
                    file_table.insert(&file1.id(), f1_bytes.as_ref())?;
                    file_table.insert(&file2.id(), f2_bytes.as_ref())?;
                    basename_map.insert("shared", &file1.id())?;
                    basename_map.insert("shared", &file2.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.find_file_views_by_basename("shared");
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[test]
        fn find_file_views_by_basename_returns_empty_when_no_match() {
            let (_tempdir, repo) = temp_vault();
            let result = repo.find_file_views_by_basename("absent");
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn find_file_views_by_parent_returns_children_when_parent_specified() {
            let (_tempdir, repo) = temp_vault();
            let parent_id = DirId::new();
            let file = FileView::new(
                FileId::new(),
                Some(parent_id),
                FileName::new("child.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [0u8; 32],
            );
            let file_bytes = file.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut parent_map = tx.try_open_multimap(
                        super::FILE_IDS_BY_PARENT.definition(),
                    )?;
                    file_table.insert(&file.id(), file_bytes.as_ref())?;
                    parent_map.insert(&parent_id, &file.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.find_file_views_by_parent(parent_id);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let children = result.unwrap();
            assert_eq!(children.len(), 1);
            assert_eq!(children.first().unwrap().id(), file.id());
        }
    }

    /// Predicate-driven format filtering.
    mod filter {
        use super::*;

        #[test]
        fn list_markdown_file_views_filters_by_format() {
            let (_tempdir, repo) = temp_vault();
            let md_file = FileView::new(
                FileId::new(),
                None,
                FileName::new("note.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [5u8; 32],
            );
            let json_file = FileView::new(
                FileId::new(),
                None,
                FileName::new("note.json".into()),
                FileFormat::Json,
                FileMetadata::new(FsTimes::new(None, None), 64, false),
                [6u8; 32],
            );
            let md_bytes = md_file.to_bytes().unwrap();
            let json_bytes = json_file.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut format_map =
                        tx.try_open_multimap(super::FILE_IDS_BY_FORMAT)?;
                    file_table.insert(&md_file.id(), md_bytes.as_ref())?;
                    file_table.insert(&json_file.id(), json_bytes.as_ref())?;
                    format_map.insert("markdown", &md_file.id())?;
                    format_map.insert("json", &json_file.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.list_markdown_file_views();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let results = result.unwrap();
            assert_eq!(results.len(), 1);
            let first = results.first().unwrap();
            assert_eq!(first.format(), FileFormat::Markdown);
        }

        #[test]
        fn list_file_views_by_format_filters_by_format() {
            let (_tempdir, repo) = temp_vault();
            let md_file = FileView::new(
                FileId::new(),
                None,
                FileName::new("a.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [0u8; 32],
            );
            let json_file = FileView::new(
                FileId::new(),
                None,
                FileName::new("b.json".into()),
                FileFormat::Json,
                FileMetadata::new(FsTimes::new(None, None), 64, false),
                [0u8; 32],
            );
            let md_bytes = md_file.to_bytes().unwrap();
            let json_bytes = json_file.to_bytes().unwrap();
            repo.store
                .write(|tx| {
                    let mut file_table =
                        tx.try_open_table(super::FILE_VIEWS.definition())?;
                    let mut format_map =
                        tx.try_open_multimap(super::FILE_IDS_BY_FORMAT)?;
                    file_table.insert(&md_file.id(), md_bytes.as_ref())?;
                    file_table.insert(&json_file.id(), json_bytes.as_ref())?;
                    format_map.insert("markdown", &md_file.id())?;
                    format_map.insert("json", &json_file.id())?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.list_file_views_by_format(FileFormat::Json);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            let results = result.unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results.first().unwrap().format(), FileFormat::Json);
        }

        #[test]
        fn list_file_views_by_format_returns_empty_when_no_match() {
            let (_tempdir, repo) = temp_vault();
            let result = repo.list_file_views_by_format(FileFormat::Image);
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_empty());
        }
    }

    /// Full table scan operations.
    mod list {
        use super::*;

        #[test]
        fn list_file_views_returns_empty_when_no_files() {
            let (_tempdir, repo) = temp_vault();
            let result = repo.list_file_views();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn list_file_views_returns_all_files() {
            let (_tempdir, repo) = temp_vault();
            let file1 = FileView::new(
                FileId::new(),
                None,
                FileName::new("a.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [7u8; 32],
            );
            let file2 = FileView::new(
                FileId::new(),
                None,
                FileName::new("b.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 256, false),
                [8u8; 32],
            );
            seed_file(&repo, &file1);
            seed_file(&repo, &file2);

            let result = repo.list_file_views();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[test]
        fn list_file_paths_returns_all_paths() {
            let (_tempdir, repo) = temp_vault();
            let id1 = FileId::new();
            let id2 = FileId::new();
            let path1 = PathKey::try_new("a.md").unwrap();
            let path2 = PathKey::try_new("b.md").unwrap();
            repo.store
                .write(|tx| {
                    let mut table =
                        tx.try_open_table(super::FILE_ID_BY_PATH.definition())?;
                    table.insert(&path1, &id1)?;
                    table.insert(&path2, &id2)?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.list_file_paths();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[test]
        fn list_dir_views_returns_all_dirs() {
            let (_tempdir, repo) = temp_vault();
            let dir1 = DirView::new(
                DirId::new(),
                None,
                DirName::new("notes".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            let dir2 = DirView::new(
                DirId::new(),
                None,
                DirName::new("archive".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            seed_dir(&repo, &dir1);
            seed_dir(&repo, &dir2);

            let result = repo.list_dir_views();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[test]
        fn list_dir_paths_returns_all_paths() {
            let (_tempdir, repo) = temp_vault();
            let id1 = DirId::new();
            let id2 = DirId::new();
            let path1 = PathKey::try_new("notes").unwrap();
            let path2 = PathKey::try_new("archive").unwrap();
            repo.store
                .write(|tx| {
                    let mut table =
                        tx.try_open_table(super::DIR_ID_BY_PATH.definition())?;
                    table.insert(&path1, &id1)?;
                    table.insert(&path2, &id2)?;
                    Ok::<_, crate::db::DbError>(())
                })
                .unwrap();

            let result = repo.list_dir_paths();
            assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
            assert_eq!(result.unwrap().len(), 2);
        }
    }
}
