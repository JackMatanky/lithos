//! Unified repository for vault metadata storage.
//!
//! Stores vault file and folder entries as rkyv-serialized domain types.

use redb::ReadableTable as _;

use crate::{
    db::{ArchivedEntity, BatchReader, BatchWriter, Database, DbError},
    fs::{BaseName, FileFormat, NormalizedPath},
    vault::{
        DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
        FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
        error::VaultRepositoryError,
        model::{DirId, DirView, FileId, FileView, FsEntryView},
    },
};

#[inline]
fn storage_err<E>(error: E) -> VaultRepositoryError
where
    E: Into<DbError>,
{
    VaultRepositoryError::Storage(error.into())
}

/// Unified repository trait for vault storage and queries.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Repository methods are grouped by read/write semantics"
)]
pub trait Repository: Send + Sync {
    /// Batch reader type for grouped read operations.
    type BatchReader<'reader>;

    /// Batch writer type for grouped write operations.
    type BatchWriter<'writer>;

    /// Storage error type for repository operations.
    type Error: From<VaultRepositoryError> + std::error::Error;

    /// Finds a stored file view by normalized path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_file_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FileView>, Self::Error>;

    /// Finds a stored directory view by normalized path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_dir_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<DirView>, Self::Error>;

    /// Returns a stored file view by ID.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn get_file_view(
        &self,
        id: FileId,
    ) -> Result<Option<FileView>, Self::Error>;

    /// Returns a stored directory view by ID.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn get_dir_view(&self, id: DirId) -> Result<Option<DirView>, Self::Error>;

    /// Returns either file or directory entry by normalized path.
    ///
    /// When both a file and a directory are indexed at the same path,
    /// file entries take precedence.
    ///
    /// # Errors
    ///
    /// Returns a repository error if either lookup fails.
    fn get_entry(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FsEntryView>, Self::Error>;

    /// Finds files by basename index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_file_views_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Finds files by parent directory ID index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_file_views_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Lists files by format index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn list_file_views_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Lists markdown files by index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn list_markdown_file_views(&self) -> Result<Vec<FileView>, Self::Error>;

    /// Lists all file views from primary table.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_all_file_views(&self) -> Result<Vec<FileView>, Self::Error>;

    /// Lists all normalized file paths from the path index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_file_paths(&self) -> Result<Vec<NormalizedPath>, Self::Error>;

    /// Lists all directory views from primary table.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_all_dir_views(&self) -> Result<Vec<DirView>, Self::Error>;

    /// Lists all normalized directory paths from the path index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_dir_paths(&self) -> Result<Vec<NormalizedPath>, Self::Error>;

    /// Saves a file view and all related indexes atomically.
    ///
    /// # Errors
    ///
    /// Returns a repository error if serialization or transaction operations
    /// fail.
    fn save_file_view(
        &self,
        path: &NormalizedPath,
        file: &FileView,
    ) -> Result<(), Self::Error>;

    /// Saves a directory view and path index atomically.
    ///
    /// # Errors
    ///
    /// Returns a repository error if serialization or transaction operations
    /// fail.
    fn save_dir_view(
        &self,
        path: &NormalizedPath,
        dir: &DirView,
    ) -> Result<(), Self::Error>;

    /// Deletes a file view and all related indexes by ID.
    ///
    /// # Errors
    ///
    /// Returns a repository error if transaction operations fail.
    fn delete_file_view(&self, id: FileId) -> Result<(), Self::Error>;

    /// Deletes a directory view and related path index by ID.
    ///
    /// # Errors
    ///
    /// Returns a repository error if transaction operations fail.
    fn delete_dir_view(&self, id: DirId) -> Result<(), Self::Error>;

    /// Executes many read operations within a single transaction.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the transaction fails.
    fn with_batch_read<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: for<'reader> FnOnce(
            Self::BatchReader<'reader>,
        ) -> Result<R, Self::Error>;

    /// Executes many write operations within a single transaction.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the transaction fails.
    fn with_batch_write<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: for<'writer> FnOnce(
            &mut Self::BatchWriter<'writer>,
        ) -> Result<(), Self::Error>;
}

/// Read-only batch adapter for vault storage.
#[expect(
    dead_code,
    reason = "Batch reader API is kept as trait surface for grouped \
              transactions"
)]
pub struct RedbBatchVaultReader<'reader> {
    reader: &'reader BatchReader,
}

impl<'reader> RedbBatchVaultReader<'reader> {
    #[inline]
    const fn new(reader: &'reader BatchReader) -> Self {
        Self {
            reader,
        }
    }
}

/// Write-capable batch adapter for vault storage.
#[expect(
    dead_code,
    reason = "Batch writer API is kept as trait surface for grouped \
              transactions"
)]
pub struct RedbBatchVaultWriter<'writer> {
    writer: &'writer mut BatchWriter,
}

impl<'writer> RedbBatchVaultWriter<'writer> {
    #[inline]
    fn new(writer: &'writer mut BatchWriter) -> Self {
        Self {
            writer,
        }
    }
}

/// Redb-backed vault repository adapter.
pub struct RedbRepository<'db> {
    db: &'db Database,
}

impl<'db> RedbRepository<'db> {
    /// Create a new repository with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Repository for RedbRepository<'_> {
    type BatchReader<'reader> = RedbBatchVaultReader<'reader>;
    type BatchWriter<'writer> = RedbBatchVaultWriter<'writer>;
    type Error = VaultRepositoryError;

    #[inline]
    fn find_file_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let path_table = match read_tx.open_table(FILE_ID_BY_PATH.definition())
        {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(error) => return Err(storage_err(error)),
        };
        let Some(id_guard) =
            path_table.get(path.as_str().to_owned()).map_err(storage_err)?
        else {
            return Ok(None);
        };
        let id = id_guard.value();
        let views =
            read_tx.open_table(FILE_VIEWS.definition()).map_err(storage_err)?;
        let Some(bytes) = views.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        FileView::from_bytes(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn find_dir_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let path_table = match read_tx.open_table(DIR_ID_BY_PATH.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(error) => return Err(storage_err(error)),
        };
        let Some(id_guard) =
            path_table.get(path.as_str().to_owned()).map_err(storage_err)?
        else {
            return Ok(None);
        };
        let id = id_guard.value();
        let views =
            read_tx.open_table(DIR_VIEWS.definition()).map_err(storage_err)?;
        let Some(bytes) = views.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        DirView::from_bytes(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_file_view(
        &self,
        id: FileId,
    ) -> Result<Option<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table =
            read_tx.open_table(FILE_VIEWS.definition()).map_err(storage_err)?;
        let Some(bytes) = table.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        FileView::from_bytes(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_dir_view(&self, id: DirId) -> Result<Option<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table =
            read_tx.open_table(DIR_VIEWS.definition()).map_err(storage_err)?;
        let Some(bytes) = table.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        DirView::from_bytes(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_entry(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FsEntryView>, Self::Error> {
        if let Some(file) = self.find_file_view_by_path(path)? {
            return Ok(Some(FsEntryView::File(file)));
        }
        self.find_dir_view_by_path(path).map(|opt| opt.map(FsEntryView::Dir))
    }

    #[inline]
    fn find_file_views_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx.open_multimap_table(FILE_IDS_BY_BASENAME) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(FILE_VIEWS.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for id in idx.get(basename).map_err(storage_err)? {
            let id = id.map_err(storage_err)?.value();
            if let Some(bytes) = views.get(id).map_err(storage_err)? {
                out.push(
                    FileView::from_bytes(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn find_file_views_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx
            .open_multimap_table(FILE_IDS_BY_PARENT.definition())
        {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(FILE_VIEWS.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for id in idx.get(parent_id).map_err(storage_err)? {
            let id = id.map_err(storage_err)?.value();
            if let Some(bytes) = views.get(id).map_err(storage_err)? {
                out.push(
                    FileView::from_bytes(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn list_file_views_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx.open_multimap_table(FILE_IDS_BY_FORMAT) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(FILE_VIEWS.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for id in idx.get(file_format_key(format)).map_err(storage_err)? {
            let id = id.map_err(storage_err)?.value();
            if let Some(bytes) = views.get(id).map_err(storage_err)? {
                out.push(
                    FileView::from_bytes(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn list_markdown_file_views(&self) -> Result<Vec<FileView>, Self::Error> {
        self.list_file_views_by_format(FileFormat::Markdown)
    }

    #[inline]
    fn list_all_file_views(&self) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(FILE_VIEWS.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (_, bytes) = row.map_err(storage_err)?;
            out.push(
                FileView::from_bytes(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            );
        }
        Ok(out)
    }

    #[inline]
    fn list_file_paths(&self) -> Result<Vec<NormalizedPath>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(FILE_ID_BY_PATH.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (path_key, _) = row.map_err(storage_err)?;
            out.push(NormalizedPath::try_new(&path_key.value()).map_err(
                |error| VaultRepositoryError::ConstraintViolation {
                    message: error.to_string().into(),
                },
            )?);
        }
        Ok(out)
    }

    #[inline]
    fn list_all_dir_views(&self) -> Result<Vec<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(DIR_VIEWS.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (_, bytes) = row.map_err(storage_err)?;
            out.push(
                DirView::from_bytes(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            );
        }
        Ok(out)
    }

    #[inline]
    fn list_dir_paths(&self) -> Result<Vec<NormalizedPath>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(DIR_ID_BY_PATH.definition()) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let mut out = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (path_key, _) = row.map_err(storage_err)?;
            out.push(NormalizedPath::try_new(&path_key.value()).map_err(
                |error| VaultRepositoryError::ConstraintViolation {
                    message: error.to_string().into(),
                },
            )?);
        }
        Ok(out)
    }

    #[inline]
    fn save_file_view(
        &self,
        path: &NormalizedPath,
        file: &FileView,
    ) -> Result<(), Self::Error> {
        let tx =
            self.db.begin_write().map_err(VaultRepositoryError::Storage)?;
        remove_stale_file_indexes(&tx, file.id())?;
        let bytes = file.to_bytes().map_err(VaultRepositoryError::Storage)?;

        {
            let mut table =
                tx.open_table(FILE_VIEWS.definition()).map_err(storage_err)?;
            table.insert(file.id(), bytes.as_slice()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_table(FILE_ID_BY_PATH.definition())
                .map_err(storage_err)?;
            table
                .insert(path.as_str().to_owned(), file.id())
                .map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_BASENAME)
                .map_err(storage_err)?;
            let basename =
                BaseName::try_from(file.name().clone()).map_err(|error| {
                    VaultRepositoryError::ConstraintViolation {
                        message: error.to_string().into(),
                    }
                })?;
            table.insert(basename.as_str(), file.id()).map_err(storage_err)?;
        }
        if let Some(parent_id) = file.parent_id() {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_PARENT.definition())
                .map_err(storage_err)?;
            table.insert(parent_id, file.id()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_FORMAT)
                .map_err(storage_err)?;
            table
                .insert(file_format_key(file.format()), file.id())
                .map_err(storage_err)?;
        }
        tx.commit().map_err(storage_err)?;
        Ok(())
    }

    #[inline]
    fn save_dir_view(
        &self,
        path: &NormalizedPath,
        dir: &DirView,
    ) -> Result<(), Self::Error> {
        let tx =
            self.db.begin_write().map_err(VaultRepositoryError::Storage)?;
        remove_stale_dir_path_indexes(&tx, dir.id())?;
        let bytes = dir.to_bytes().map_err(VaultRepositoryError::Storage)?;
        {
            let mut table =
                tx.open_table(DIR_VIEWS.definition()).map_err(storage_err)?;
            table.insert(dir.id(), bytes.as_slice()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_table(DIR_ID_BY_PATH.definition())
                .map_err(storage_err)?;
            table
                .insert(path.as_str().to_owned(), dir.id())
                .map_err(storage_err)?;
        }
        tx.commit().map_err(storage_err)?;
        Ok(())
    }

    #[inline]
    fn delete_file_view(&self, id: FileId) -> Result<(), Self::Error> {
        let tx =
            self.db.begin_write().map_err(VaultRepositoryError::Storage)?;
        remove_stale_file_indexes(&tx, id)?;
        {
            let mut table =
                tx.open_table(FILE_VIEWS.definition()).map_err(storage_err)?;
            table.remove(id).map_err(storage_err)?;
        }
        tx.commit().map_err(storage_err)?;
        Ok(())
    }

    #[inline]
    fn delete_dir_view(&self, id: DirId) -> Result<(), Self::Error> {
        let tx =
            self.db.begin_write().map_err(VaultRepositoryError::Storage)?;
        remove_stale_dir_path_indexes(&tx, id)?;
        {
            let mut table =
                tx.open_table(DIR_VIEWS.definition()).map_err(storage_err)?;
            table.remove(id).map_err(storage_err)?;
        }
        tx.commit().map_err(storage_err)?;
        Ok(())
    }

    #[inline]
    fn with_batch_read<F, R>(&self, f: F) -> Result<R, Self::Error>
    where
        F: for<'reader> FnOnce(
            Self::BatchReader<'reader>,
        ) -> Result<R, Self::Error>,
    {
        self.db
            .batch_read(|reader| {
                let batch = RedbBatchVaultReader::new(reader);
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    clippy::unreachable,
                    reason = "Batch closures only perform db operations; \
                              non-Storage variants indicate programming error"
                )]
                f(batch).map_err(|err| match err {
                    VaultRepositoryError::Storage(db_err) => db_err,
                    other => unreachable!(
                        "batch operation returned non-storage error: {other}"
                    ),
                })
            })
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn with_batch_write<F>(&self, f: F) -> Result<(), Self::Error>
    where
        F: for<'writer> FnOnce(
            &mut Self::BatchWriter<'writer>,
        ) -> Result<(), Self::Error>,
    {
        self.db
            .batch_write(|writer| {
                let mut batch = RedbBatchVaultWriter::new(writer);
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    clippy::unreachable,
                    reason = "Batch closures only perform db operations; \
                              non-Storage variants indicate programming error"
                )]
                f(&mut batch).map_err(|err| match err {
                    VaultRepositoryError::Storage(db_err) => db_err,
                    other => unreachable!(
                        "batch operation returned non-storage error: {other}"
                    ),
                })
            })
            .map_err(VaultRepositoryError::Storage)
    }
}

fn remove_stale_file_indexes(
    tx: &redb::WriteTransaction,
    file_id: FileId,
) -> Result<(), VaultRepositoryError> {
    let prior_view = {
        let table =
            tx.open_table(FILE_VIEWS.definition()).map_err(storage_err)?;
        match table.get(file_id).map_err(storage_err)? {
            Some(bytes) => Some(
                FileView::from_bytes(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            ),
            None => None,
        }
    };

    if let Some(prior) = prior_view {
        {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_BASENAME)
                .map_err(storage_err)?;
            let basename =
                BaseName::try_from(prior.name().clone()).map_err(|error| {
                    VaultRepositoryError::ConstraintViolation {
                        message: error.to_string().into(),
                    }
                })?;
            table.remove(basename.as_str(), file_id).map_err(storage_err)?;
        }
        if let Some(parent_id) = prior.parent_id() {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_PARENT.definition())
                .map_err(storage_err)?;
            table.remove(parent_id, file_id).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(FILE_IDS_BY_FORMAT)
                .map_err(storage_err)?;
            table
                .remove(file_format_key(prior.format()), file_id)
                .map_err(storage_err)?;
        }
    }

    let stale_paths = {
        let table =
            tx.open_table(FILE_ID_BY_PATH.definition()).map_err(storage_err)?;
        let mut paths = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (path_key, id_value) = row.map_err(storage_err)?;
            if id_value.value() == file_id {
                paths.push(path_key.value().clone());
            }
        }
        paths
    };

    if !stale_paths.is_empty() {
        let mut table =
            tx.open_table(FILE_ID_BY_PATH.definition()).map_err(storage_err)?;
        for stale_path in stale_paths {
            table.remove(stale_path).map_err(storage_err)?;
        }
    }

    Ok(())
}

fn remove_stale_dir_path_indexes(
    tx: &redb::WriteTransaction,
    dir_id: DirId,
) -> Result<(), VaultRepositoryError> {
    let stale_paths = {
        let table =
            tx.open_table(DIR_ID_BY_PATH.definition()).map_err(storage_err)?;
        let mut paths = Vec::new();
        for row in table.iter().map_err(storage_err)? {
            let (path_key, id_value) = row.map_err(storage_err)?;
            if id_value.value() == dir_id {
                paths.push(path_key.value().clone());
            }
        }
        paths
    };

    if stale_paths.is_empty() {
        return Ok(());
    }

    let mut table =
        tx.open_table(DIR_ID_BY_PATH.definition()).map_err(storage_err)?;
    for stale_path in stale_paths {
        table.remove(stale_path).map_err(storage_err)?;
    }

    Ok(())
}

#[inline]
const fn file_format_key(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Json => "json",
        FileFormat::Toml => "toml",
        FileFormat::Yaml => "yaml",
        FileFormat::Markdown => "markdown",
        FileFormat::Image => "image",
        FileFormat::Pdf => "pdf",
        FileFormat::Document => "document",
        FileFormat::Archive => "archive",
        FileFormat::Binary => "binary",
        FileFormat::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{RedbRepository, Repository};
    use crate::{
        db::Database,
        fs::{
            DirMetadata, DirName, FileFormat, FileMetadata, FileName, FsTimes,
            NormalizedPath,
        },
        vault::{
            FILE_VIEWS,
            model::{DirId, DirView, FileId, FileView, FsEntryView},
        },
    };

    fn open_test_db() -> (tempfile::TempDir, Database) {
        let temp = tempfile::tempdir().expect("create temp dir");
        let db_path = temp.path().join("vault-storage.db");
        let db = Database::open(&db_path).expect("open database");
        (temp, db)
    }

    fn sample_file_view(parent_id: Option<DirId>) -> FileView {
        FileView::new(
            FileId::new(),
            parent_id,
            FileName::new("note.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 128, false),
            [9u8; 32],
        )
    }

    fn sample_dir_view(parent_id: Option<DirId>) -> DirView {
        DirView::new(
            DirId::new(),
            parent_id,
            DirName::new("notes".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        )
    }

    mod path_lookup_tests {
        use super::*;

        #[test]
        fn save_and_find_file_view_by_path_roundtrips() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path = NormalizedPath::try_new("notes/note.md").expect("path");
            let file = sample_file_view(None);
            repo.save_file_view(&path, &file).expect("save");

            let found = repo.find_file_view_by_path(&path).expect("lookup");
            assert!(found.is_some());
            assert_eq!(found.expect("value").id(), file.id());
        }

        #[test]
        fn save_and_find_dir_view_by_path_roundtrips() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path = NormalizedPath::try_new("notes").expect("path");
            let dir = sample_dir_view(None);
            repo.save_dir_view(&path, &dir).expect("save");

            let found = repo.find_dir_view_by_path(&path).expect("lookup");
            assert!(found.is_some());
            assert_eq!(found.expect("value").id(), dir.id());
        }
    }

    mod index_query_tests {
        use super::*;

        #[test]
        fn list_markdown_files_returns_saved_markdown_entries() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let parent = DirId::new();
            let path = NormalizedPath::try_new("notes/note.md").expect("path");
            let file = sample_file_view(Some(parent));
            repo.save_file_view(&path, &file).expect("save");

            let listed =
                repo.list_markdown_file_views().expect("list markdown");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed.first().expect("markdown file").id(), file.id());

            let by_parent =
                repo.find_file_views_by_parent(parent).expect("by parent");
            assert_eq!(by_parent.len(), 1);
            assert_eq!(
                by_parent.first().expect("file by parent").id(),
                file.id()
            );
        }

        #[test]
        fn find_files_by_basename_returns_all_matching_file_ids() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path_a =
                NormalizedPath::try_new("notes/shared.md").expect("path a");
            let path_b =
                NormalizedPath::try_new("archive/shared.md").expect("path b");

            let file_a = FileView::new(
                FileId::new(),
                None,
                FileName::new("shared.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 41, false),
                [11u8; 32],
            );
            let file_b = FileView::new(
                FileId::new(),
                None,
                FileName::new("shared.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 42, false),
                [12u8; 32],
            );

            repo.save_file_view(&path_a, &file_a).expect("save a");
            repo.save_file_view(&path_b, &file_b).expect("save b");

            let listed =
                repo.find_file_views_by_basename("shared").expect("query");
            assert_eq!(listed.len(), 2);
            assert!(listed.iter().any(|file| file.id() == file_a.id()));
            assert!(listed.iter().any(|file| file.id() == file_b.id()));
        }

        #[test]
        fn indexed_file_queries_return_empty_on_fresh_database() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            assert!(
                repo.find_file_views_by_basename("missing")
                    .expect("basename")
                    .is_empty()
            );
            assert!(
                repo.find_file_views_by_parent(DirId::new())
                    .expect("parent")
                    .is_empty()
            );
            assert!(
                repo.list_file_views_by_format(FileFormat::Markdown)
                    .expect("format")
                    .is_empty()
            );
            assert!(
                repo.list_markdown_file_views().expect("markdown").is_empty()
            );
        }

        #[test]
        fn indexed_queries_skip_stale_file_ids() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let parent = DirId::new();
            let path = NormalizedPath::try_new("notes/stale.md").expect("path");
            let file = FileView::new(
                FileId::new(),
                Some(parent),
                FileName::new("stale.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 61, false),
                [15u8; 32],
            );
            repo.save_file_view(&path, &file).expect("save");

            {
                let tx = db.begin_write().expect("begin write");
                {
                    let mut views = tx
                        .open_table(FILE_VIEWS.definition())
                        .expect("open file views");
                    views.remove(file.id()).expect("remove primary");
                }
                tx.commit().expect("commit");
            }

            assert!(
                repo.find_file_views_by_basename("stale")
                    .expect("basename")
                    .is_empty()
            );
            assert!(
                repo.find_file_views_by_parent(parent)
                    .expect("parent")
                    .is_empty()
            );
            assert!(
                repo.list_markdown_file_views().expect("markdown").is_empty()
            );
            assert!(
                repo.list_file_views_by_format(FileFormat::Markdown)
                    .expect("format")
                    .is_empty()
            );
        }

        #[test]
        fn save_file_view_format_change_updates_format_index() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let file_id = FileId::new();
            let path =
                NormalizedPath::try_new("notes/format-swap.md").expect("path");
            let markdown = FileView::new(
                file_id,
                None,
                FileName::new("format-swap.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 51, false),
                [13u8; 32],
            );
            repo.save_file_view(&path, &markdown).expect("save markdown");

            let swapped = FileView::new(
                file_id,
                None,
                FileName::new("format-swap.md".into()),
                FileFormat::Document,
                FileMetadata::new(FsTimes::new(None, None), 52, false),
                [14u8; 32],
            );
            repo.save_file_view(&path, &swapped).expect("save swapped");

            assert!(
                repo.list_markdown_file_views().expect("markdown").is_empty()
            );
            let docs = repo
                .list_file_views_by_format(FileFormat::Document)
                .expect("document");
            assert_eq!(docs.len(), 1);
            assert_eq!(docs.first().expect("document file").id(), file_id);
        }
    }

    mod write_path_consistency_tests {
        use super::*;

        #[test]
        fn save_file_view_overwrite_cleans_stale_indexes() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let file_id = FileId::new();
            let old_parent = DirId::new();
            let new_parent = DirId::new();

            let old_path =
                NormalizedPath::try_new("notes/old.md").expect("path");
            let old = FileView::new(
                file_id,
                Some(old_parent),
                FileName::new("old.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 10, false),
                [1u8; 32],
            );
            repo.save_file_view(&old_path, &old).expect("save old");

            let new_path =
                NormalizedPath::try_new("notes/new.txt").expect("path");
            let new = FileView::new(
                file_id,
                Some(new_parent),
                FileName::new("new.txt".into()),
                FileFormat::Document,
                FileMetadata::new(FsTimes::new(None, None), 11, false),
                [2u8; 32],
            );
            repo.save_file_view(&new_path, &new).expect("save new");

            assert!(
                repo.find_file_view_by_path(&old_path)
                    .expect("old path")
                    .is_none()
            );
            assert!(
                repo.find_file_view_by_path(&new_path)
                    .expect("new path")
                    .is_some()
            );

            assert!(
                repo.find_file_views_by_basename("old")
                    .expect("old basename")
                    .is_empty()
            );
            assert_eq!(
                repo.find_file_views_by_basename("new")
                    .expect("new basename")
                    .len(),
                1
            );

            assert!(
                repo.find_file_views_by_parent(old_parent)
                    .expect("old parent")
                    .is_empty()
            );
            assert_eq!(
                repo.find_file_views_by_parent(new_parent)
                    .expect("new parent")
                    .len(),
                1
            );

            assert!(
                repo.list_markdown_file_views()
                    .expect("markdown list")
                    .is_empty()
            );
            assert_eq!(
                repo.list_file_views_by_format(FileFormat::Document)
                    .expect("document list")
                    .len(),
                1
            );
        }

        #[test]
        fn save_dir_view_overwrite_cleans_stale_path_index() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let dir_id = DirId::new();

            let old_path = NormalizedPath::try_new("notes").expect("path");
            let old = DirView::new(
                dir_id,
                None,
                DirName::new("notes".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            repo.save_dir_view(&old_path, &old).expect("save old");

            let new_path = NormalizedPath::try_new("archive").expect("path");
            let new = DirView::new(
                dir_id,
                None,
                DirName::new("archive".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            repo.save_dir_view(&new_path, &new).expect("save new");

            assert!(
                repo.find_dir_view_by_path(&old_path)
                    .expect("old path")
                    .is_none()
            );
            assert!(
                repo.find_dir_view_by_path(&new_path)
                    .expect("new path")
                    .is_some()
            );
        }

        #[test]
        fn delete_file_view_removes_primary_and_all_indexes() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let parent = DirId::new();
            let path =
                NormalizedPath::try_new("notes/delete-me.md").expect("path");
            let file = FileView::new(
                FileId::new(),
                Some(parent),
                FileName::new("delete-me.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 12, false),
                [3u8; 32],
            );

            repo.save_file_view(&path, &file).expect("save");
            repo.delete_file_view(file.id()).expect("delete");

            assert!(
                repo.find_file_view_by_path(&path).expect("by path").is_none()
            );
            assert!(repo.get_file_view(file.id()).expect("by id").is_none());
            assert!(
                repo.find_file_views_by_basename("delete-me")
                    .expect("basename")
                    .is_empty()
            );
            assert!(
                repo.find_file_views_by_parent(parent)
                    .expect("parent")
                    .is_empty()
            );
            assert!(
                repo.list_markdown_file_views().expect("markdown").is_empty()
            );
        }

        #[test]
        fn delete_dir_view_removes_primary_and_path_index() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path =
                NormalizedPath::try_new("notes/delete-dir").expect("path");
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("delete-dir".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );

            repo.save_dir_view(&path, &dir).expect("save");
            repo.delete_dir_view(dir.id()).expect("delete");

            assert!(
                repo.find_dir_view_by_path(&path).expect("by path").is_none()
            );
            assert!(repo.get_dir_view(dir.id()).expect("by id").is_none());
        }
    }

    mod entry_resolution_tests {
        use super::*;

        #[test]
        fn get_entry_prefers_file_when_file_and_dir_share_path() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path = NormalizedPath::try_new("notes/shared").expect("path");
            let file = FileView::new(
                FileId::new(),
                None,
                FileName::new("shared.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 21, false),
                [5u8; 32],
            );
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("shared".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );

            repo.save_dir_view(&path, &dir).expect("save dir");
            repo.save_file_view(&path, &file).expect("save file");

            let found = repo.get_entry(&path).expect("lookup");
            assert!(matches!(found, Some(FsEntryView::File(_))));
        }

        #[test]
        fn get_entry_returns_dir_when_only_dir_exists() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let path = NormalizedPath::try_new("notes/dir-only").expect("path");
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("dir-only".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );

            repo.save_dir_view(&path, &dir).expect("save dir");

            let found = repo.get_entry(&path).expect("lookup");
            assert!(matches!(found, Some(FsEntryView::Dir(_))));
        }
    }

    mod full_scan_listing_tests {
        use super::*;

        #[test]
        fn list_all_views_returns_empty_on_fresh_database() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            assert!(repo.list_all_file_views().expect("files").is_empty());
            assert!(repo.list_all_dir_views().expect("dirs").is_empty());
        }

        #[test]
        fn list_all_files_reflects_overwrite_and_delete_by_id() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let file_id = FileId::new();
            let first_path =
                NormalizedPath::try_new("notes/original.md").expect("path");
            let first = FileView::new(
                file_id,
                None,
                FileName::new("original.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 30, false),
                [7u8; 32],
            );
            repo.save_file_view(&first_path, &first).expect("save first");

            let second_path =
                NormalizedPath::try_new("notes/renamed.txt").expect("path");
            let second = FileView::new(
                file_id,
                None,
                FileName::new("renamed.txt".into()),
                FileFormat::Document,
                FileMetadata::new(FsTimes::new(None, None), 31, false),
                [8u8; 32],
            );
            repo.save_file_view(&second_path, &second).expect("save second");

            let listed =
                repo.list_all_file_views().expect("list after overwrite");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed.first().expect("listed file").id(), file_id);
            assert_eq!(
                listed.first().expect("listed file").name().as_str(),
                "renamed.txt"
            );

            repo.delete_file_view(file_id).expect("delete");
            assert!(
                repo.list_all_file_views()
                    .expect("list after delete")
                    .is_empty()
            );
        }

        #[test]
        fn list_all_dirs_reflects_overwrite_and_delete_by_id() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);

            let dir_id = DirId::new();
            let first_path =
                NormalizedPath::try_new("notes/original-dir").expect("path");
            let first = DirView::new(
                dir_id,
                None,
                DirName::new("original-dir".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            repo.save_dir_view(&first_path, &first).expect("save first");

            let second_path =
                NormalizedPath::try_new("archive/renamed-dir").expect("path");
            let second = DirView::new(
                dir_id,
                None,
                DirName::new("renamed-dir".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            repo.save_dir_view(&second_path, &second).expect("save second");

            let listed =
                repo.list_all_dir_views().expect("list after overwrite");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed.first().expect("listed dir").id(), dir_id);
            assert_eq!(
                listed.first().expect("listed dir").name().as_str(),
                "renamed-dir"
            );

            repo.delete_dir_view(dir_id).expect("delete");
            assert!(
                repo.list_all_dir_views()
                    .expect("list after delete")
                    .is_empty()
            );
        }
    }

    mod migration_cutover_tests {
        use super::*;

        #[test]
        fn repository_path_indexes_are_queryable_for_views_only() {
            let (_temp, db) = open_test_db();
            let repo = RedbRepository::new(&db);
            let path =
                NormalizedPath::try_new("notes/migration.md").expect("path");
            let file = FileView::new(
                FileId::new(),
                None,
                FileName::new("migration.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, None), 1, false),
                [1u8; 32],
            );
            repo.save_file_view(&path, &file).expect("save");
            let paths = repo.list_file_paths().expect("list file paths");
            assert!(paths.iter().any(|p| p.as_str() == path.as_str()));
        }
    }
}
