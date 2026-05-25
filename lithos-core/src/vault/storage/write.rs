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

#[derive(Debug, Default)]
struct FileDeleteContext {
    path: Option<String>,
    basename: Option<BaseName>,
    parent_id: Option<DirId>,
    format: Option<crate::fs::FileFormat>,
}

#[derive(Debug, Default)]
struct DirDeleteContext {
    path: Option<String>,
}

impl FileDeleteContext {
    fn load(tx: &WriteTx, file_id: FileId) -> Result<Self, DbError> {
        let file_table = tx.try_open_table(FILE_VIEWS.definition())?;
        let path_table = tx.try_open_table(FILE_ID_BY_PATH.definition())?;

        let mut ctx = Self::default();

        if let Some(guard) = file_table.get(&file_id)? {
            let file = FileView::from_bytes(guard.value())?;
            ctx.basename = Some(
                BaseName::try_from(file.name().clone())
                    .map_err(|e| DbError::Deserialization(e.to_string()))?,
            );
            ctx.parent_id = file.parent_id();
            ctx.format = Some(file.format());
        }

        for row in path_table.iter()? {
            let (path, id) = row?;
            if id.value() == file_id {
                ctx.path = Some(path.value());
                break;
            }
        }

        Ok(ctx)
    }
}

impl DirDeleteContext {
    fn load(tx: &WriteTx, dir_id: DirId) -> Result<Self, DbError> {
        let path_table = tx.try_open_table(DIR_ID_BY_PATH.definition())?;
        let mut ctx = Self::default();

        for row in path_table.iter()? {
            let (path, id) = row?;
            if id.value() == dir_id {
                ctx.path = Some(path.value());
                break;
            }
        }

        Ok(ctx)
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

    #[test]
    fn save_file_view_persists_view_and_indexes() {
        let (_temp, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let file = sample_file(None, "test.md", FileFormat::Markdown);
        let path = NormalizedPath::try_new("notes/test.md").unwrap();

        repo.save_file_view(&path, &file).unwrap();

        assert_eq!(
            repo.get_file_view(file.id()).unwrap().unwrap().id(),
            file.id()
        );
        assert_eq!(
            repo.find_file_view_by_path(&path).unwrap().unwrap().id(),
            file.id()
        );
        assert_eq!(repo.find_file_views_by_basename("test").unwrap().len(), 1);
        assert_eq!(repo.list_markdown_file_views().unwrap().len(), 1);
    }

    #[test]
    fn delete_file_view_removes_primary_and_indexes() {
        let (_temp, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let file = sample_file(None, "delete.md", FileFormat::Markdown);
        let path = NormalizedPath::try_new("notes/delete.md").unwrap();
        repo.save_file_view(&path, &file).unwrap();

        repo.delete_file_view(file.id()).unwrap();

        assert!(repo.get_file_view(file.id()).unwrap().is_none());
        assert!(repo.find_file_view_by_path(&path).unwrap().is_none());
        assert!(repo.find_file_views_by_basename("delete").unwrap().is_empty());
    }

    #[test]
    fn save_dir_view_and_delete_dir_view_round_trip() {
        let (_temp, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let dir = sample_dir("notes");
        let path = NormalizedPath::try_new("notes").unwrap();

        repo.save_dir_view(&path, &dir).unwrap();
        assert_eq!(
            repo.find_dir_view_by_path(&path).unwrap().unwrap().id(),
            dir.id()
        );

        repo.delete_dir_view(dir.id()).unwrap();
        assert!(repo.get_dir_view(dir.id()).unwrap().is_none());
        assert!(repo.find_dir_view_by_path(&path).unwrap().is_none());
    }

    #[test]
    fn save_many_file_views_persists_all_entries() {
        let (_temp, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let a = sample_file(None, "a.md", FileFormat::Markdown);
        let b = sample_file(None, "b.md", FileFormat::Markdown);
        let entries = vec![
            (NormalizedPath::try_new("a.md").unwrap(), a.clone()),
            (NormalizedPath::try_new("b.md").unwrap(), b.clone()),
        ];

        repo.save_many_file_views(&entries).unwrap();

        assert!(repo.get_file_view(a.id()).unwrap().is_some());
        assert!(repo.get_file_view(b.id()).unwrap().is_some());
    }

    #[test]
    fn delete_many_file_views_is_idempotent() {
        let (_temp, store) = Store::open_temp().unwrap();
        let repo = RedbRepository::new(Arc::new(store));
        let file = sample_file(None, "many.md", FileFormat::Markdown);
        let path = NormalizedPath::try_new("many.md").unwrap();
        repo.save_file_view(&path, &file).unwrap();

        repo.delete_many_file_views(&[file.id(), FileId::new()]).unwrap();

        assert!(repo.get_file_view(file.id()).unwrap().is_none());
    }
}
