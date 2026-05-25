//! `ReadRepository` implementation for Vault persistence.

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
        FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
    },
};
use crate::{
    db::{ArchivedEntity, DbError},
    fs::{FileFormat, NormalizedPath},
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
                let Some(guard) = table.get(&id)? else {
                    return Ok(None);
                };
                let file = FileView::from_bytes(guard.value())?;
                Ok(Some(file))
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
                let Some(guard) = table.get(&id)? else {
                    return Ok(None);
                };
                let dir = DirView::from_bytes(guard.value())?;
                Ok(Some(dir))
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn find_file_view_by_path(
        &self,
        path: &NormalizedPath,
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

                let Some(id_guard) =
                    path_table.get(path.as_str().to_owned())?
                else {
                    return Ok(None);
                };
                let id = id_guard.value();

                let Some(file_guard) = file_table.get(&id)? else {
                    return Ok(None);
                };
                let file = FileView::from_bytes(file_guard.value())?;
                Ok(Some(file))
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn find_dir_view_by_path(
        &self,
        path: &NormalizedPath,
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

                let Some(id_guard) =
                    path_table.get(path.as_str().to_owned())?
                else {
                    return Ok(None);
                };
                let id = id_guard.value();

                let Some(dir_guard) = dir_table.get(&id)? else {
                    return Ok(None);
                };
                let dir = DirView::from_bytes(dir_guard.value())?;
                Ok(Some(dir))
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn get_entry(
        &self,
        path: &NormalizedPath,
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
                    let id = id_result?.value();
                    if let Some(guard) = file_table.get(&id)? {
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
                    let id = id_result?.value();
                    if let Some(guard) = file_table.get(&id)? {
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
                    let id = id_result?.value();
                    if let Some(guard) = file_table.get(&id)? {
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
                    let (_id_guard, file_guard) = result?;
                    files.push(FileView::from_bytes(file_guard.value())?);
                }

                Ok(files)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_file_paths(
        &self,
    ) -> Result<Vec<NormalizedPath>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(FILE_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut paths = Vec::new();
                for result in table.iter()? {
                    let (path_guard, _id_guard) = result?;
                    let path = NormalizedPath::try_new(&path_guard.value())
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    paths.push(path);
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
                    let (_id_guard, dir_guard) = result?;
                    dirs.push(DirView::from_bytes(dir_guard.value())?);
                }

                Ok(dirs)
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn list_dir_paths(
        &self,
    ) -> Result<Vec<NormalizedPath>, VaultRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) =
                    tx.try_open_table(DIR_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut paths = Vec::new();
                for result in table.iter()? {
                    let (path_guard, _id_guard) = result?;
                    let path = NormalizedPath::try_new(&path_guard.value())
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;
                    paths.push(path);
                }

                Ok(paths)
            })
            .map_err(VaultRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::Store,
        fs::{DirMetadata, FileFormat, FileMetadata, FileName, NormalizedPath},
        vault::{
            model::{DirId, DirView, FileId, FileView},
            repository::ReadRepository,
            storage::RedbRepository,
        },
    };

    #[test]
    fn get_file_view_returns_none_for_missing_id() {
        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let result = repo.get_file_view(FileId::new());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn get_file_view_returns_stored_view() {
        use crate::{db::ArchivedEntity, fs::FsTimes};

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        // Create and save a file view
        let file = FileView::new(
            FileId::new(),
            None, // no parent
            FileName::new("test.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 128, false),
            [0u8; 32], // content hash
        );

        // Serialize BEFORE opening transaction (minimize transaction duration)
        let file_bytes = file.to_bytes().unwrap();

        // Save using low-level store access (WriteRepository not implemented
        // yet)
        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::FILE_VIEWS.definition())?;
                table.insert(&file.id(), file_bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        // Retrieve using ReadRepository
        let retrieved = repo.get_file_view(file.id()).unwrap().unwrap();
        assert_eq!(retrieved.id(), file.id());
        assert_eq!(retrieved.name(), file.name());
    }

    #[test]
    fn get_dir_view_returns_none_for_missing_id() {
        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let result = repo.get_dir_view(DirId::new());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn get_dir_view_returns_stored_view() {
        use crate::{
            db::ArchivedEntity,
            fs::{DirName, FsTimes},
        };

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        let dir = DirView::new(
            DirId::new(),
            None,
            DirName::new("test".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        );

        let dir_bytes = dir.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::DIR_VIEWS.definition())?;
                table.insert(&dir.id(), dir_bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.get_dir_view(dir.id()).unwrap().unwrap();
        assert_eq!(retrieved.id(), dir.id());
        assert_eq!(retrieved.name(), dir.name());
    }

    #[test]
    fn find_file_view_by_path_returns_none_for_missing_path() {
        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let path = NormalizedPath::try_new("missing.md").unwrap();
        let result = repo.find_file_view_by_path(&path);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn find_file_view_by_path_performs_cross_table_lookup() {
        use crate::{db::ArchivedEntity, fs::FsTimes};

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        let file = FileView::new(
            FileId::new(),
            None,
            FileName::new("test.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 128, false),
            [1u8; 32],
        );
        let path = NormalizedPath::try_new("notes/test.md").unwrap();

        // Serialize outside transaction
        let file_bytes = file.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut file_table =
                    tx.try_open_table(super::FILE_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(super::FILE_ID_BY_PATH.definition())?;
                file_table.insert(&file.id(), file_bytes.as_ref())?;
                path_table.insert(path.as_str().to_owned(), &file.id())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.find_file_view_by_path(&path).unwrap().unwrap();
        assert_eq!(retrieved.id(), file.id());
    }

    #[test]
    fn find_dir_view_by_path_performs_cross_table_lookup() {
        use crate::{
            db::ArchivedEntity,
            fs::{DirName, FsTimes},
        };

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        let dir = DirView::new(
            DirId::new(),
            None,
            DirName::new("notes".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        );
        let path = NormalizedPath::try_new("notes").unwrap();

        let dir_bytes = dir.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut dir_table =
                    tx.try_open_table(super::DIR_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(super::DIR_ID_BY_PATH.definition())?;
                dir_table.insert(&dir.id(), dir_bytes.as_ref())?;
                path_table.insert(path.as_str().to_owned(), &dir.id())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let retrieved = repo.find_dir_view_by_path(&path).unwrap().unwrap();
        assert_eq!(retrieved.id(), dir.id());
    }

    #[test]
    fn get_entry_prefers_file_when_both_exist() {
        use crate::{
            db::ArchivedEntity,
            fs::{DirName, FsTimes},
        };

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

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
        let path = NormalizedPath::try_new("test").unwrap();

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
                file_path_table.insert(path.as_str().to_owned(), &file.id())?;
                dir_table.insert(&dir.id(), dir_bytes.as_ref())?;
                dir_path_table.insert(path.as_str().to_owned(), &dir.id())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let entry = repo.get_entry(&path).unwrap().unwrap();
        assert!(entry.is_file());
    }

    #[test]
    fn find_file_views_by_basename_returns_all_matches() {
        use crate::{db::ArchivedEntity, fs::FsTimes};

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

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

        let results = repo.find_file_views_by_basename("shared").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_markdown_file_views_filters_by_format() {
        use crate::{db::ArchivedEntity, fs::FsTimes};

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        let md_file = FileView::new(
            FileId::new(),
            None,
            FileName::new("note.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 128, false),
            [5u8; 32],
        );
        let txt_file = FileView::new(
            FileId::new(),
            None,
            FileName::new("note.json".into()),
            FileFormat::Json,
            FileMetadata::new(FsTimes::new(None, None), 64, false),
            [6u8; 32],
        );

        let md_bytes = md_file.to_bytes().unwrap();
        let txt_bytes = txt_file.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut file_table =
                    tx.try_open_table(super::FILE_VIEWS.definition())?;
                let mut format_map =
                    tx.try_open_multimap(super::FILE_IDS_BY_FORMAT)?;

                file_table.insert(&md_file.id(), md_bytes.as_ref())?;
                file_table.insert(&txt_file.id(), txt_bytes.as_ref())?;
                format_map.insert("markdown", &md_file.id())?;
                format_map.insert("json", &txt_file.id())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let results = repo.list_markdown_file_views().unwrap();
        assert_eq!(results.len(), 1);
        let first = results.first().unwrap();
        assert_eq!(first.format(), FileFormat::Markdown);
    }

    #[test]
    fn list_file_views_returns_all_files() {
        use crate::{db::ArchivedEntity, fs::FsTimes};

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

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

        let f1_bytes = file1.to_bytes().unwrap();
        let f2_bytes = file2.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::FILE_VIEWS.definition())?;
                table.insert(&file1.id(), f1_bytes.as_ref())?;
                table.insert(&file2.id(), f2_bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let results = repo.list_file_views().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_file_paths_returns_all_paths() {
        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

        let id1 = FileId::new();
        let id2 = FileId::new();
        let path1 = NormalizedPath::try_new("a.md").unwrap();
        let path2 = NormalizedPath::try_new("b.md").unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::FILE_ID_BY_PATH.definition())?;
                table.insert(path1.as_str().to_owned(), &id1)?;
                table.insert(path2.as_str().to_owned(), &id2)?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let results = repo.list_file_paths().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn list_dir_views_returns_all_dirs() {
        use crate::{
            db::ArchivedEntity,
            fs::{DirName, FsTimes},
        };

        let (_tempdir, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));

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

        let d1_bytes = dir1.to_bytes().unwrap();
        let d2_bytes = dir2.to_bytes().unwrap();

        repo.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(super::DIR_VIEWS.definition())?;
                table.insert(&dir1.id(), d1_bytes.as_ref())?;
                table.insert(&dir2.id(), d2_bytes.as_ref())?;
                Ok::<_, crate::db::DbError>(())
            })
            .unwrap();

        let results = repo.list_dir_views().unwrap();
        assert_eq!(results.len(), 2);
    }
}
