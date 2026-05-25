//! `WriteRepository` implementation for Vault persistence.

use redb::ReadableTable as _;

use super::{
    RedbRepository,
    tables::{
        DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
        FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
    },
};
use crate::{
    db::{ArchivedEntity, DbError, WriteTx},
    fs::{BaseName, NormalizedPath},
    vault::{
        error::VaultRepositoryError,
        model::{DirId, DirView, FileId, FileView},
        repository::WriteRepository,
    },
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_file_view(
        &self,
        path: &NormalizedPath,
        file: &FileView,
    ) -> Result<(), VaultRepositoryError> {
        let file_bytes = file.to_bytes()?;
        let basename =
            BaseName::try_from(file.name().clone()).map_err(|e| {
                VaultRepositoryError::Storage(DbError::Deserialization(
                    e.to_string(),
                ))
            })?;

        self.store
            .write(|tx| {
                Self::remove_file_graph(tx, file.id())?;

                let mut file_table =
                    tx.try_open_table(FILE_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(FILE_ID_BY_PATH.definition())?;
                let mut by_basename =
                    tx.try_open_multimap(FILE_IDS_BY_BASENAME)?;
                let mut by_parent =
                    tx.try_open_multimap(FILE_IDS_BY_PARENT.definition())?;
                let mut by_format = tx.try_open_multimap(FILE_IDS_BY_FORMAT)?;

                file_table.insert(&file.id(), file_bytes.as_ref())?;
                path_table.insert(path.as_str().to_owned(), &file.id())?;
                by_basename.insert(basename.as_str(), &file.id())?;
                if let Some(parent_id) = file.parent_id() {
                    by_parent.insert(&parent_id, &file.id())?;
                }
                by_format.insert(file.format().as_str(), &file.id())?;
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn save_dir_view(
        &self,
        path: &NormalizedPath,
        dir: &DirView,
    ) -> Result<(), VaultRepositoryError> {
        let dir_bytes = dir.to_bytes()?;

        self.store
            .write(|tx| {
                Self::remove_dir_graph(tx, dir.id())?;

                let mut dir_table =
                    tx.try_open_table(DIR_VIEWS.definition())?;
                let mut path_table =
                    tx.try_open_table(DIR_ID_BY_PATH.definition())?;
                dir_table.insert(&dir.id(), dir_bytes.as_ref())?;
                path_table.insert(path.as_str().to_owned(), &dir.id())?;
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn delete_file_view(&self, id: FileId) -> Result<(), VaultRepositoryError> {
        self.store
            .write(|tx| Self::remove_file_graph(tx, id))
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn delete_dir_view(&self, id: DirId) -> Result<(), VaultRepositoryError> {
        self.store
            .write(|tx| Self::remove_dir_graph(tx, id))
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn save_many_file_views(
        &self,
        entries: &[(NormalizedPath, FileView)],
    ) -> Result<(), VaultRepositoryError> {
        let prepared: Result<Vec<_>, VaultRepositoryError> = entries
            .iter()
            .map(|(path, file)| {
                let basename = BaseName::try_from(file.name().clone())
                    .map_err(|e| {
                        VaultRepositoryError::Storage(DbError::Deserialization(
                            e.to_string(),
                        ))
                    })?;
                Ok((path.clone(), file.clone(), file.to_bytes()?, basename))
            })
            .collect();
        let prepared = prepared?;

        self.store
            .write(|tx| {
                for (path, file, bytes, basename) in &prepared {
                    Self::remove_file_graph(tx, file.id())?;

                    let mut file_table =
                        tx.try_open_table(FILE_VIEWS.definition())?;
                    let mut path_table =
                        tx.try_open_table(FILE_ID_BY_PATH.definition())?;
                    let mut by_basename =
                        tx.try_open_multimap(FILE_IDS_BY_BASENAME)?;
                    let mut by_parent =
                        tx.try_open_multimap(FILE_IDS_BY_PARENT.definition())?;
                    let mut by_format =
                        tx.try_open_multimap(FILE_IDS_BY_FORMAT)?;

                    file_table.insert(&file.id(), bytes.as_ref())?;
                    path_table.insert(path.as_str().to_owned(), &file.id())?;
                    by_basename.insert(basename.as_str(), &file.id())?;
                    if let Some(parent_id) = file.parent_id() {
                        by_parent.insert(&parent_id, &file.id())?;
                    }
                    by_format.insert(file.format().as_str(), &file.id())?;
                }
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn save_many_dir_views(
        &self,
        entries: &[(NormalizedPath, DirView)],
    ) -> Result<(), VaultRepositoryError> {
        let prepared: Result<Vec<_>, VaultRepositoryError> = entries
            .iter()
            .map(|(path, dir)| Ok((path.clone(), dir.clone(), dir.to_bytes()?)))
            .collect();
        let prepared = prepared?;

        self.store
            .write(|tx| {
                for (path, dir, bytes) in &prepared {
                    Self::remove_dir_graph(tx, dir.id())?;

                    let mut dir_table =
                        tx.try_open_table(DIR_VIEWS.definition())?;
                    let mut path_table =
                        tx.try_open_table(DIR_ID_BY_PATH.definition())?;
                    dir_table.insert(&dir.id(), bytes.as_ref())?;
                    path_table.insert(path.as_str().to_owned(), &dir.id())?;
                }
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn delete_many_file_views(
        &self,
        ids: &[FileId],
    ) -> Result<(), VaultRepositoryError> {
        self.store
            .write(|tx| {
                for id in ids {
                    Self::remove_file_graph(tx, *id)?;
                }
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }

    #[inline]
    fn delete_many_dir_views(
        &self,
        ids: &[DirId],
    ) -> Result<(), VaultRepositoryError> {
        self.store
            .write(|tx| {
                for id in ids {
                    Self::remove_dir_graph(tx, *id)?;
                }
                Ok(())
            })
            .map_err(VaultRepositoryError::from)
    }
}

impl RedbRepository {
    fn remove_file_path_index(
        tx: &WriteTx,
        path: Option<&str>,
    ) -> Result<(), DbError> {
        if let Some(path) = path {
            let mut table = tx.try_open_table(FILE_ID_BY_PATH.definition())?;
            table.remove(path.to_owned())?;
        }
        Ok(())
    }

    fn remove_file_basename_index(
        tx: &WriteTx,
        basename: Option<&BaseName>,
        file_id: FileId,
    ) -> Result<(), DbError> {
        if let Some(basename) = basename {
            let mut table = tx.try_open_multimap(FILE_IDS_BY_BASENAME)?;
            table.remove(basename.as_str(), &file_id)?;
        }
        Ok(())
    }

    fn remove_file_parent_index(
        tx: &WriteTx,
        parent_id: Option<DirId>,
        file_id: FileId,
    ) -> Result<(), DbError> {
        if let Some(parent_id) = parent_id {
            let mut table =
                tx.try_open_multimap(FILE_IDS_BY_PARENT.definition())?;
            table.remove(&parent_id, &file_id)?;
        }
        Ok(())
    }

    fn remove_file_format_index(
        tx: &WriteTx,
        format: Option<crate::fs::FileFormat>,
        file_id: FileId,
    ) -> Result<(), DbError> {
        if let Some(format) = format {
            let mut table = tx.try_open_multimap(FILE_IDS_BY_FORMAT)?;
            table.remove(format.as_str(), &file_id)?;
        }
        Ok(())
    }

    fn remove_file_primary(
        tx: &WriteTx,
        file_id: FileId,
    ) -> Result<(), DbError> {
        let mut table = tx.try_open_table(FILE_VIEWS.definition())?;
        table.remove(&file_id)?;
        Ok(())
    }

    fn remove_dir_path_index(
        tx: &WriteTx,
        path: Option<&str>,
    ) -> Result<(), DbError> {
        if let Some(path) = path {
            let mut table = tx.try_open_table(DIR_ID_BY_PATH.definition())?;
            table.remove(path.to_owned())?;
        }
        Ok(())
    }

    fn remove_dir_primary(tx: &WriteTx, dir_id: DirId) -> Result<(), DbError> {
        let mut table = tx.try_open_table(DIR_VIEWS.definition())?;
        table.remove(&dir_id)?;
        Ok(())
    }

    fn remove_file_graph(tx: &WriteTx, file_id: FileId) -> Result<(), DbError> {
        let ctx = FileDeleteContext::load(tx, file_id)?;
        Self::remove_file_path_index(tx, ctx.path.as_deref())?;
        Self::remove_file_basename_index(tx, ctx.basename.as_ref(), file_id)?;
        Self::remove_file_parent_index(tx, ctx.parent_id, file_id)?;
        Self::remove_file_format_index(tx, ctx.format, file_id)?;
        Self::remove_file_primary(tx, file_id)
    }

    fn remove_dir_graph(tx: &WriteTx, dir_id: DirId) -> Result<(), DbError> {
        let ctx = DirDeleteContext::load(tx, dir_id)?;
        Self::remove_dir_path_index(tx, ctx.path.as_deref())?;
        Self::remove_dir_primary(tx, dir_id)
    }
}

#[derive(Debug, Default)]
struct FileDeleteContext {
    path: Option<String>,
    basename: Option<BaseName>,
    parent_id: Option<DirId>,
    format: Option<crate::fs::FileFormat>,
}

impl FileDeleteContext {
    fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> {
        let file_table = tx.try_open_table(FILE_VIEWS.definition())?;
        let path_table = tx.try_open_table(FILE_ID_BY_PATH.definition())?;

        let (basename, parent_id, format) = if let Some(file) = file_table
            .get(&file_id)?
            .map(|g| FileView::from_bytes(g.value()))
            .transpose()?
        {
            (
                Some(
                    BaseName::try_from(file.name().clone())
                        .map_err(|e| DbError::Deserialization(e.to_string()))?,
                ),
                file.parent_id(),
                Some(file.format()),
            )
        } else {
            (None, None, None)
        };

        let path = path_table
            .iter()?
            .find(|res| {
                res.as_ref()
                    .map(|(_, id)| id.value() == file_id)
                    .unwrap_or(false)
            })
            .transpose()?
            .map(|(path, _)| path.value());

        Ok(Self {
            path,
            basename,
            parent_id,
            format,
        })
    }
}

#[derive(Debug, Default)]
struct DirDeleteContext {
    path: Option<String>,
}

impl DirDeleteContext {
    fn load(tx: &WriteTx, dir_id: DirId) -> Result<Self, DbError> {
        let path_table = tx.try_open_table(DIR_ID_BY_PATH.definition())?;

        let path = path_table
            .iter()?
            .find(|res| {
                res.as_ref()
                    .map(|(_, id)| id.value() == dir_id)
                    .unwrap_or(false)
            })
            .transpose()?
            .map(|(path, _)| path.value());

        Ok(Self {
            path,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::Store,
        fs::{
            DirMetadata, DirName, FileFormat, FileMetadata, FileName, FsTimes,
            NormalizedPath,
        },
        vault::{
            model::{DirId, DirView, FileId, FileView},
            repository::{ReadRepository, WriteRepository},
            storage::RedbRepository,
        },
    };

    fn temp_vault() -> (tempfile::TempDir, RedbRepository) {
        let (tempdir, store) = Store::open_temp().unwrap();
        (tempdir, RedbRepository::new(Arc::new(store)))
    }

    fn sample_file(
        parent: Option<DirId>,
        name: &str,
        fmt: FileFormat,
    ) -> FileView {
        FileView::new(
            FileId::new(),
            parent,
            FileName::new(name.into()),
            fmt,
            FileMetadata::new(FsTimes::new(None, None), 128, false),
            [9u8; 32],
        )
    }

    fn sample_dir(name: &str) -> DirView {
        DirView::new(
            DirId::new(),
            None,
            DirName::new(name.into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        )
    }

    mod upsert {
        use super::*;

        #[test]
        fn file_persists_primary_and_indexes() {
            let (_temp, repo) = temp_vault();
            let file = sample_file(None, "test.md", FileFormat::Markdown);
            let path = NormalizedPath::try_new("notes/test.md").unwrap();

            let result = repo.save_file_view(&path, &file);
            assert!(result.is_ok(), "Save failed: {:?}", result.err());

            assert_eq!(
                repo.get_file_view(file.id()).unwrap().unwrap().id(),
                file.id()
            );
            assert_eq!(
                repo.find_file_view_by_path(&path).unwrap().unwrap().id(),
                file.id()
            );
            assert_eq!(
                repo.find_file_views_by_basename("test").unwrap().len(),
                1
            );
            assert_eq!(repo.list_markdown_file_views().unwrap().len(), 1);
        }

        #[test]
        fn file_cleans_stale_indexes_on_overwrite() {
            let (_temp, repo) = temp_vault();

            let id = FileId::new();
            let first = FileView::new(
                id,
                None,
                FileName::new("old.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 128, false),
                [1u8; 32],
            );
            let second = FileView::new(
                id,
                None,
                FileName::new("new.json".into()),
                FileFormat::Json,
                FileMetadata::new(FsTimes::new(None, None), 256, false),
                [2u8; 32],
            );
            let old_path = NormalizedPath::try_new("notes/old.md").unwrap();
            let new_path = NormalizedPath::try_new("notes/new.json").unwrap();

            repo.save_file_view(&old_path, &first).unwrap();

            let result = repo.save_file_view(&new_path, &second);
            assert!(result.is_ok(), "Overwrite failed: {:?}", result.err());

            assert!(repo.find_file_view_by_path(&old_path).unwrap().is_none());
            assert!(
                repo.find_file_views_by_basename("old").unwrap().is_empty()
            );
            assert!(
                repo.list_file_views_by_format(FileFormat::Markdown)
                    .unwrap()
                    .is_empty()
            );

            let current =
                repo.find_file_view_by_path(&new_path).unwrap().unwrap();
            assert_eq!(current.id(), id);
            assert_eq!(current.format(), FileFormat::Json);
        }

        #[test]
        fn file_batch_persists_all_entries() {
            let (_temp, repo) = temp_vault();
            let a = sample_file(None, "a.md", FileFormat::Markdown);
            let b = sample_file(None, "b.md", FileFormat::Markdown);
            let entries = vec![
                (NormalizedPath::try_new("a.md").unwrap(), a.clone()),
                (NormalizedPath::try_new("b.md").unwrap(), b.clone()),
            ];

            let result = repo.save_many_file_views(&entries);
            assert!(result.is_ok(), "Batch save failed: {:?}", result.err());

            assert!(repo.get_file_view(a.id()).unwrap().is_some());
            assert!(repo.get_file_view(b.id()).unwrap().is_some());
        }

        #[test]
        fn dir_persists_with_path_index() {
            let (_temp, repo) = temp_vault();
            let dir = sample_dir("notes");
            let path = NormalizedPath::try_new("notes").unwrap();

            let result = repo.save_dir_view(&path, &dir);
            assert!(result.is_ok(), "Dir save failed: {:?}", result.err());

            assert_eq!(
                repo.find_dir_view_by_path(&path).unwrap().unwrap().id(),
                dir.id()
            );
        }

        #[test]
        fn dir_cleans_stale_path_index_on_overwrite() {
            let (_temp, repo) = temp_vault();

            let id = DirId::new();
            let dir = DirView::new(
                id,
                None,
                DirName::new("notes".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            let old_path = NormalizedPath::try_new("old-notes").unwrap();
            let new_path = NormalizedPath::try_new("new-notes").unwrap();

            repo.save_dir_view(&old_path, &dir).unwrap();

            let result = repo.save_dir_view(&new_path, &dir);
            assert!(result.is_ok(), "Dir overwrite failed: {:?}", result.err());

            assert!(repo.find_dir_view_by_path(&old_path).unwrap().is_none());
            let current =
                repo.find_dir_view_by_path(&new_path).unwrap().unwrap();
            assert_eq!(current.id(), id);
        }

        #[test]
        fn dir_batch_persists_all_entries() {
            let (_temp, repo) = temp_vault();
            let a = sample_dir("notes");
            let b = sample_dir("archive");
            let entries = vec![
                (NormalizedPath::try_new("notes").unwrap(), a.clone()),
                (NormalizedPath::try_new("archive").unwrap(), b.clone()),
            ];

            let result = repo.save_many_dir_views(&entries);
            assert!(
                result.is_ok(),
                "Dir batch save failed: {:?}",
                result.err()
            );

            assert!(repo.get_dir_view(a.id()).unwrap().is_some());
            assert!(repo.get_dir_view(b.id()).unwrap().is_some());
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn file_removes_primary_and_indexes() {
            let (_temp, repo) = temp_vault();
            let file = sample_file(None, "delete.md", FileFormat::Markdown);
            let path = NormalizedPath::try_new("notes/delete.md").unwrap();
            repo.save_file_view(&path, &file).unwrap();

            let result = repo.delete_file_view(file.id());
            assert!(result.is_ok(), "Delete file failed: {:?}", result.err());

            assert!(repo.get_file_view(file.id()).unwrap().is_none());
            assert!(repo.find_file_view_by_path(&path).unwrap().is_none());
            assert!(
                repo.find_file_views_by_basename("delete").unwrap().is_empty()
            );
        }

        #[test]
        fn file_is_idempotent_when_missing() {
            let (_temp, repo) = temp_vault();

            let result = repo.delete_file_view(FileId::new());
            assert!(
                result.is_ok(),
                "Delete missing file failed: {:?}",
                result.err()
            );
        }

        #[test]
        fn dir_removes_path_index_and_primary() {
            let (_temp, repo) = temp_vault();
            let dir = sample_dir("notes");
            let path = NormalizedPath::try_new("notes").unwrap();
            repo.save_dir_view(&path, &dir).unwrap();

            let result = repo.delete_dir_view(dir.id());
            assert!(result.is_ok(), "Delete dir failed: {:?}", result.err());

            assert!(repo.get_dir_view(dir.id()).unwrap().is_none());
            assert!(repo.find_dir_view_by_path(&path).unwrap().is_none());
        }

        #[test]
        fn dir_is_idempotent_when_missing() {
            let (_temp, repo) = temp_vault();

            let result = repo.delete_dir_view(DirId::new());
            assert!(
                result.is_ok(),
                "Delete missing dir failed: {:?}",
                result.err()
            );
        }

        #[test]
        fn batch_file_is_idempotent() {
            let (_temp, repo) = temp_vault();
            let file = sample_file(None, "many.md", FileFormat::Markdown);
            let path = NormalizedPath::try_new("many.md").unwrap();
            repo.save_file_view(&path, &file).unwrap();

            let result =
                repo.delete_many_file_views(&[file.id(), FileId::new()]);
            assert!(
                result.is_ok(),
                "Batch delete file failed: {:?}",
                result.err()
            );

            assert!(repo.get_file_view(file.id()).unwrap().is_none());
        }

        #[test]
        fn batch_dir_is_idempotent() {
            let (_temp, repo) = temp_vault();
            let dir = sample_dir("many");
            let path = NormalizedPath::try_new("many").unwrap();
            repo.save_dir_view(&path, &dir).unwrap();

            let result = repo.delete_many_dir_views(&[dir.id(), DirId::new()]);
            assert!(
                result.is_ok(),
                "Batch delete dir failed: {:?}",
                result.err()
            );

            assert!(repo.get_dir_view(dir.id()).unwrap().is_none());
            assert!(repo.find_dir_view_by_path(&path).unwrap().is_none());
        }
    }
}
