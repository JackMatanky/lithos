//! Unified repository for vault metadata storage.
//!
//! Stores vault file and folder entries as rkyv-serialized domain types.

use redb::ReadableTable as _;

use crate::{
    db::{BatchReader, BatchWriter, Database, DbError, deserialize, serialize},
    fs::FileFormat,
    vault::{
        VAULT_DIR_ID_BY_PATH, VAULT_DIR_VIEWS, VAULT_FILE_ID_BY_PATH,
        VAULT_FILE_IDS_BY_BASENAME, VAULT_FILE_IDS_BY_FORMAT,
        VAULT_FILE_IDS_BY_PARENT, VAULT_FILE_VIEWS, VAULT_FILES_BY_PATH,
        VAULT_FOLDERS_BY_PATH,
        error::VaultRepositoryError,
        model::{
            DirId, DirView, FileId, FileView, FsEntryView, NormalizedPath,
            VaultFile, VaultFolder, VaultPath,
        },
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

    /// Archived vault file type for zero-copy reads.
    type VaultFileArchived<'archived>;

    /// Archived vault folder type for zero-copy reads.
    type VaultFolderArchived<'archived>;

    /// Returns a stored file entry by its path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn get_file(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFile>, Self::Error>;

    /// Returns a stored folder entry by its path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn get_folder(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFolder>, Self::Error>;

    /// Lists all stored vault files.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_files(&self) -> Result<Vec<VaultFile>, Self::Error>;

    /// Lists all stored vault folders.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_folders(&self) -> Result<Vec<VaultFolder>, Self::Error>;

    /// Finds a stored file view by normalized path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_file_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FileView>, Self::Error>;

    /// Finds a stored directory view by normalized path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_dir_by_path(
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
    fn get_dir(&self, id: DirId) -> Result<Option<DirView>, Self::Error>;

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
    fn find_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Finds files by parent directory ID index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn find_files_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Lists files by format index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn list_files_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, Self::Error>;

    /// Lists markdown files by index.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn list_markdown_files(&self) -> Result<Vec<FileView>, Self::Error>;

    /// Lists all file views from primary table.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_all_files(&self) -> Result<Vec<FileView>, Self::Error>;

    /// Lists all directory views from primary table.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the scan fails.
    fn list_all_dirs(&self) -> Result<Vec<DirView>, Self::Error>;

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

    /// Deletes a vault file entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if deletion fails.
    fn delete_file(&self, path: &VaultPath) -> Result<(), Self::Error>;

    /// Deletes a vault folder entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if deletion fails.
    fn delete_folder(&self, path: &VaultPath) -> Result<(), Self::Error>;

    /// Persists a vault file entry.
    ///
    /// # Errors
    ///
    /// Returns a repository error if persistence fails.
    fn save_file(&self, file: &VaultFile) -> Result<(), Self::Error>;

    /// Persists a vault folder entry.
    ///
    /// # Errors
    ///
    /// Returns a repository error if persistence fails.
    fn save_folder(&self, folder: &VaultFolder) -> Result<(), Self::Error>;

    /// Accesses a vault file as archived data.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn with_archived_file<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::VaultFileArchived<'archived>) -> R;

    /// Accesses a vault folder as archived data.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    fn with_archived_folder<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::VaultFolderArchived<'archived>) -> R;

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

    /// Returns a stored file entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn get_file(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFile>, VaultRepositoryError> {
        self.reader
            .get_owned::<VaultFile>(VAULT_FILES_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
    }

    /// Returns a stored folder entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn get_folder(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFolder>, VaultRepositoryError> {
        self.reader
            .get_owned::<VaultFolder>(VAULT_FOLDERS_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
    }

    /// Accesses an archived vault file by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn with_file<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, VaultRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<VaultFile>) -> R,
    {
        self.reader
            .get::<VaultFile, _, R>(VAULT_FILES_BY_PATH, path.as_str(), f)
            .map_err(VaultRepositoryError::Storage)
    }

    /// Accesses an archived vault folder by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if the lookup fails.
    #[inline]
    pub fn with_folder<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, VaultRepositoryError>
    where
        F: for<'archived> FnOnce(&'archived rkyv::Archived<VaultFolder>) -> R,
    {
        self.reader
            .get::<VaultFolder, _, R>(VAULT_FOLDERS_BY_PATH, path.as_str(), f)
            .map_err(VaultRepositoryError::Storage)
    }
}

/// Write-capable batch adapter for vault storage.
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

    /// Persists a vault file entry.
    ///
    /// # Errors
    ///
    /// Returns a repository error if persistence fails.
    #[inline]
    pub fn put_file(
        &mut self,
        file: &VaultFile,
    ) -> Result<(), VaultRepositoryError> {
        self.writer
            .put(VAULT_FILES_BY_PATH, file.path().as_str(), file)
            .map_err(VaultRepositoryError::Storage)
    }

    /// Persists a vault folder entry.
    ///
    /// # Errors
    ///
    /// Returns a repository error if persistence fails.
    #[inline]
    pub fn put_folder(
        &mut self,
        folder: &VaultFolder,
    ) -> Result<(), VaultRepositoryError> {
        self.writer
            .put(VAULT_FOLDERS_BY_PATH, folder.path().as_str(), folder)
            .map_err(VaultRepositoryError::Storage)
    }

    /// Deletes a vault file entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if deletion fails.
    #[inline]
    pub fn delete_file(
        &mut self,
        path: &VaultPath,
    ) -> Result<(), VaultRepositoryError> {
        self.writer
            .delete(VAULT_FILES_BY_PATH, path.as_str())
            .map_err(storage_err)?;
        Ok(())
    }

    /// Deletes a vault folder entry by path.
    ///
    /// # Errors
    ///
    /// Returns a repository error if deletion fails.
    #[inline]
    pub fn delete_folder(
        &mut self,
        path: &VaultPath,
    ) -> Result<(), VaultRepositoryError> {
        self.writer
            .delete(VAULT_FOLDERS_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)?;
        Ok(())
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
    type VaultFileArchived<'archived> = &'archived rkyv::Archived<VaultFile>;
    type VaultFolderArchived<'archived> =
        &'archived rkyv::Archived<VaultFolder>;

    #[inline]
    fn get_file(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFile>, Self::Error> {
        self.db
            .get_owned::<VaultFile>(VAULT_FILES_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_folder(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultFolder>, Self::Error> {
        self.db
            .get_owned::<VaultFolder>(VAULT_FOLDERS_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn list_files(&self) -> Result<Vec<VaultFile>, Self::Error> {
        self.db
            .list_owned::<VaultFile>(VAULT_FILES_BY_PATH)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn list_folders(&self) -> Result<Vec<VaultFolder>, Self::Error> {
        self.db
            .list_owned::<VaultFolder>(VAULT_FOLDERS_BY_PATH)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn find_file_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let path_table =
            match read_tx.open_table(VAULT_FILE_ID_BY_PATH.definition()) {
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
        let views = read_tx
            .open_table(VAULT_FILE_VIEWS.definition())
            .map_err(storage_err)?;
        let Some(bytes) = views.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        deserialize(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn find_dir_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let path_table =
            match read_tx.open_table(VAULT_DIR_ID_BY_PATH.definition()) {
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
        let views = read_tx
            .open_table(VAULT_DIR_VIEWS.definition())
            .map_err(storage_err)?;
        let Some(bytes) = views.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        deserialize(bytes.value())
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
        let table = read_tx
            .open_table(VAULT_FILE_VIEWS.definition())
            .map_err(storage_err)?;
        let Some(bytes) = table.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        deserialize(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_dir(&self, id: DirId) -> Result<Option<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = read_tx
            .open_table(VAULT_DIR_VIEWS.definition())
            .map_err(storage_err)?;
        let Some(bytes) = table.get(id).map_err(storage_err)? else {
            return Ok(None);
        };
        deserialize(bytes.value())
            .map(Some)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn get_entry(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FsEntryView>, Self::Error> {
        if let Some(file) = self.find_file_by_path(path)? {
            return Ok(Some(FsEntryView::File(file)));
        }
        self.find_dir_by_path(path).map(|opt| opt.map(FsEntryView::Dir))
    }

    #[inline]
    fn find_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx.open_multimap_table(VAULT_FILE_IDS_BY_BASENAME)
        {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(VAULT_FILE_VIEWS.definition()) {
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
                    deserialize(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn find_files_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx
            .open_multimap_table(VAULT_FILE_IDS_BY_PARENT.definition())
        {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(VAULT_FILE_VIEWS.definition()) {
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
                    deserialize(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn list_files_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let idx = match read_tx.open_multimap_table(VAULT_FILE_IDS_BY_FORMAT) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(storage_err(error)),
        };
        let views = match read_tx.open_table(VAULT_FILE_VIEWS.definition()) {
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
                    deserialize(bytes.value())
                        .map_err(VaultRepositoryError::Storage)?,
                );
            }
        }
        Ok(out)
    }

    #[inline]
    fn list_markdown_files(&self) -> Result<Vec<FileView>, Self::Error> {
        self.list_files_by_format(FileFormat::Markdown)
    }

    #[inline]
    fn list_all_files(&self) -> Result<Vec<FileView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(VAULT_FILE_VIEWS.definition()) {
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
                deserialize(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            );
        }
        Ok(out)
    }

    #[inline]
    fn list_all_dirs(&self) -> Result<Vec<DirView>, Self::Error> {
        let read_tx =
            self.db.begin_read().map_err(VaultRepositoryError::Storage)?;
        let table = match read_tx.open_table(VAULT_DIR_VIEWS.definition()) {
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
                deserialize(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            );
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
        let bytes = serialize(file).map_err(VaultRepositoryError::Storage)?;

        {
            let mut table = tx
                .open_table(VAULT_FILE_VIEWS.definition())
                .map_err(storage_err)?;
            table.insert(file.id(), bytes.as_slice()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_table(VAULT_FILE_ID_BY_PATH.definition())
                .map_err(storage_err)?;
            table
                .insert(path.as_str().to_owned(), file.id())
                .map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_BASENAME)
                .map_err(storage_err)?;
            table
                .insert(file.name().basename_str(), file.id())
                .map_err(storage_err)?;
        }
        if let Some(parent_id) = file.parent_id() {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_PARENT.definition())
                .map_err(storage_err)?;
            table.insert(parent_id, file.id()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_FORMAT)
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
        let bytes = serialize(dir).map_err(VaultRepositoryError::Storage)?;
        {
            let mut table = tx
                .open_table(VAULT_DIR_VIEWS.definition())
                .map_err(storage_err)?;
            table.insert(dir.id(), bytes.as_slice()).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_table(VAULT_DIR_ID_BY_PATH.definition())
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
            let mut table = tx
                .open_table(VAULT_FILE_VIEWS.definition())
                .map_err(storage_err)?;
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
            let mut table = tx
                .open_table(VAULT_DIR_VIEWS.definition())
                .map_err(storage_err)?;
            table.remove(id).map_err(storage_err)?;
        }
        tx.commit().map_err(storage_err)?;
        Ok(())
    }

    #[inline]
    fn save_file(&self, file: &VaultFile) -> Result<(), Self::Error> {
        self.db
            .put(VAULT_FILES_BY_PATH, file.path().as_str(), file)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn save_folder(&self, folder: &VaultFolder) -> Result<(), Self::Error> {
        self.db
            .put(VAULT_FOLDERS_BY_PATH, folder.path().as_str(), folder)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn delete_file(&self, path: &VaultPath) -> Result<(), Self::Error> {
        self.db
            .delete(VAULT_FILES_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
            .map(|_| ())
    }

    #[inline]
    fn delete_folder(&self, path: &VaultPath) -> Result<(), Self::Error> {
        self.db
            .delete(VAULT_FOLDERS_BY_PATH, path.as_str())
            .map_err(VaultRepositoryError::Storage)
            .map(|_| ())
    }

    #[inline]
    fn with_archived_file<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::VaultFileArchived<'archived>) -> R,
    {
        self.db
            .get::<VaultFile, _, R>(VAULT_FILES_BY_PATH, path.as_str(), f)
            .map_err(VaultRepositoryError::Storage)
    }

    #[inline]
    fn with_archived_folder<F, R>(
        &self,
        path: &VaultPath,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::VaultFolderArchived<'archived>) -> R,
    {
        self.db
            .get::<VaultFolder, _, R>(VAULT_FOLDERS_BY_PATH, path.as_str(), f)
            .map_err(VaultRepositoryError::Storage)
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
        let table = tx
            .open_table(VAULT_FILE_VIEWS.definition())
            .map_err(storage_err)?;
        match table.get(file_id).map_err(storage_err)? {
            Some(bytes) => Some(
                deserialize::<FileView>(bytes.value())
                    .map_err(VaultRepositoryError::Storage)?,
            ),
            None => None,
        }
    };

    if let Some(prior) = prior_view {
        {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_BASENAME)
                .map_err(storage_err)?;
            table
                .remove(prior.name().basename_str(), file_id)
                .map_err(storage_err)?;
        }
        if let Some(parent_id) = prior.parent_id() {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_PARENT.definition())
                .map_err(storage_err)?;
            table.remove(parent_id, file_id).map_err(storage_err)?;
        }
        {
            let mut table = tx
                .open_multimap_table(VAULT_FILE_IDS_BY_FORMAT)
                .map_err(storage_err)?;
            table
                .remove(file_format_key(prior.format()), file_id)
                .map_err(storage_err)?;
        }
    }

    let stale_paths = {
        let table = tx
            .open_table(VAULT_FILE_ID_BY_PATH.definition())
            .map_err(storage_err)?;
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
        let mut table = tx
            .open_table(VAULT_FILE_ID_BY_PATH.definition())
            .map_err(storage_err)?;
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
        let table = tx
            .open_table(VAULT_DIR_ID_BY_PATH.definition())
            .map_err(storage_err)?;
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

    let mut table = tx
        .open_table(VAULT_DIR_ID_BY_PATH.definition())
        .map_err(storage_err)?;
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
        },
        vault::{
            VAULT_FILE_VIEWS,
            model::{
                DirId, DirView, FileId, FileView, FsEntryView, NormalizedPath,
                VaultFile, VaultFolder, VaultPath,
            },
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

    #[test]
    fn save_and_find_file_view_by_path_roundtrips() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let path = NormalizedPath::try_new("notes/note.md").expect("path");
        let file = sample_file_view(None);
        repo.save_file_view(&path, &file).expect("save");

        let found = repo.find_file_by_path(&path).expect("lookup");
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

        let found = repo.find_dir_by_path(&path).expect("lookup");
        assert!(found.is_some());
        assert_eq!(found.expect("value").id(), dir.id());
    }

    #[test]
    fn list_markdown_files_returns_saved_markdown_entries() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let parent = DirId::new();
        let path = NormalizedPath::try_new("notes/note.md").expect("path");
        let file = sample_file_view(Some(parent));
        repo.save_file_view(&path, &file).expect("save");

        let listed = repo.list_markdown_files().expect("list markdown");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed.first().expect("markdown file").id(), file.id());

        let by_parent = repo.find_files_by_parent(parent).expect("by parent");
        assert_eq!(by_parent.len(), 1);
        assert_eq!(by_parent.first().expect("file by parent").id(), file.id());
    }

    #[test]
    fn save_file_view_overwrite_cleans_stale_indexes() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let file_id = FileId::new();
        let old_parent = DirId::new();
        let new_parent = DirId::new();

        let old_path = NormalizedPath::try_new("notes/old.md").expect("path");
        let old = FileView::new(
            file_id,
            Some(old_parent),
            FileName::new("old.md".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 10, false),
            [1u8; 32],
        );
        repo.save_file_view(&old_path, &old).expect("save old");

        let new_path = NormalizedPath::try_new("notes/new.txt").expect("path");
        let new = FileView::new(
            file_id,
            Some(new_parent),
            FileName::new("new.txt".into()),
            FileFormat::Document,
            FileMetadata::new(FsTimes::new(None, None), 11, false),
            [2u8; 32],
        );
        repo.save_file_view(&new_path, &new).expect("save new");

        assert!(repo.find_file_by_path(&old_path).expect("old path").is_none());
        assert!(repo.find_file_by_path(&new_path).expect("new path").is_some());

        assert!(
            repo.find_files_by_basename("old")
                .expect("old basename")
                .is_empty()
        );
        assert_eq!(
            repo.find_files_by_basename("new").expect("new basename").len(),
            1
        );

        assert!(
            repo.find_files_by_parent(old_parent)
                .expect("old parent")
                .is_empty()
        );
        assert_eq!(
            repo.find_files_by_parent(new_parent).expect("new parent").len(),
            1
        );

        assert!(repo.list_markdown_files().expect("markdown list").is_empty());
        assert_eq!(
            repo.list_files_by_format(FileFormat::Document)
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

        assert!(repo.find_dir_by_path(&old_path).expect("old path").is_none());
        assert!(repo.find_dir_by_path(&new_path).expect("new path").is_some());
    }

    #[test]
    fn delete_file_view_removes_primary_and_all_indexes() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let parent = DirId::new();
        let path = NormalizedPath::try_new("notes/delete-me.md").expect("path");
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

        assert!(repo.find_file_by_path(&path).expect("by path").is_none());
        assert!(repo.get_file_view(file.id()).expect("by id").is_none());
        assert!(
            repo.find_files_by_basename("delete-me")
                .expect("basename")
                .is_empty()
        );
        assert!(repo.find_files_by_parent(parent).expect("parent").is_empty());
        assert!(repo.list_markdown_files().expect("markdown").is_empty());
    }

    #[test]
    fn delete_dir_view_removes_primary_and_path_index() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let path = NormalizedPath::try_new("notes/delete-dir").expect("path");
        let dir = DirView::new(
            DirId::new(),
            None,
            DirName::new("delete-dir".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
        );

        repo.save_dir_view(&path, &dir).expect("save");
        repo.delete_dir_view(dir.id()).expect("delete");

        assert!(repo.find_dir_by_path(&path).expect("by path").is_none());
        assert!(repo.get_dir(dir.id()).expect("by id").is_none());
    }

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

    #[test]
    fn list_all_views_returns_empty_on_fresh_database() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        assert!(repo.list_all_files().expect("files").is_empty());
        assert!(repo.list_all_dirs().expect("dirs").is_empty());
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

        let listed = repo.list_all_files().expect("list after overwrite");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed.first().expect("listed file").id(), file_id);
        assert_eq!(
            listed.first().expect("listed file").name().as_str(),
            "renamed.txt"
        );

        repo.delete_file_view(file_id).expect("delete");
        assert!(repo.list_all_files().expect("list after delete").is_empty());
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

        let listed = repo.find_files_by_basename("shared").expect("query");
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().any(|file| file.id() == file_a.id()));
        assert!(listed.iter().any(|file| file.id() == file_b.id()));
    }

    #[test]
    fn indexed_file_queries_return_empty_on_fresh_database() {
        let (_temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        assert!(
            repo.find_files_by_basename("missing")
                .expect("basename")
                .is_empty()
        );
        assert!(
            repo.find_files_by_parent(DirId::new()).expect("parent").is_empty()
        );
        assert!(
            repo.list_files_by_format(FileFormat::Markdown)
                .expect("format")
                .is_empty()
        );
        assert!(repo.list_markdown_files().expect("markdown").is_empty());
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

        assert!(repo.list_markdown_files().expect("markdown").is_empty());
        let docs =
            repo.list_files_by_format(FileFormat::Document).expect("document");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs.first().expect("document file").id(), file_id);
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

        let listed = repo.list_all_dirs().expect("list after overwrite");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed.first().expect("listed dir").id(), dir_id);
        assert_eq!(
            listed.first().expect("listed dir").name().as_str(),
            "renamed-dir"
        );

        repo.delete_dir_view(dir_id).expect("delete");
        assert!(repo.list_all_dirs().expect("list after delete").is_empty());
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
                    .open_table(VAULT_FILE_VIEWS.definition())
                    .expect("open file views");
                views.remove(file.id()).expect("remove primary");
            }
            tx.commit().expect("commit");
        }

        assert!(
            repo.find_files_by_basename("stale").expect("basename").is_empty()
        );
        assert!(repo.find_files_by_parent(parent).expect("parent").is_empty());
        assert!(repo.list_markdown_files().expect("markdown").is_empty());
        assert!(
            repo.list_files_by_format(FileFormat::Markdown)
                .expect("format")
                .is_empty()
        );
    }

    #[test]
    fn legacy_file_methods_roundtrip_for_processor_compatibility() {
        let (temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let path = VaultPath::try_new("notes/legacy.md").expect("path");
        let file_path = temp.path().join("legacy.md");
        std::fs::write(&file_path, b"legacy file").expect("write fixture file");
        let metadata = std::fs::metadata(&file_path).expect("file metadata");
        let file =
            VaultFile::try_new(path.clone(), &metadata).expect("vault file");

        repo.save_file(&file).expect("save");
        let found = repo.get_file(&path).expect("get");
        assert!(found.is_some());
        assert_eq!(found.expect("value").path(), file.path());

        let listed = repo.list_files().expect("list");
        assert_eq!(listed.len(), 1);

        repo.delete_file(&path).expect("delete");
        assert!(repo.get_file(&path).expect("get after delete").is_none());
    }

    #[test]
    fn legacy_folder_methods_roundtrip_for_processor_compatibility() {
        let (temp, db) = open_test_db();
        let repo = RedbRepository::new(&db);

        let path = VaultPath::try_new("notes/legacy-folder").expect("path");
        let folder_path = temp.path().join("legacy-folder");
        std::fs::create_dir(&folder_path).expect("create fixture dir");
        let metadata = std::fs::metadata(&folder_path).expect("dir metadata");
        let folder = VaultFolder::try_new(path.clone(), &metadata)
            .expect("vault folder");

        repo.save_folder(&folder).expect("save");
        let found = repo.get_folder(&path).expect("get");
        assert!(found.is_some());
        assert_eq!(found.expect("value").path(), folder.path());

        let listed = repo.list_folders().expect("list");
        assert_eq!(listed.len(), 1);

        repo.delete_folder(&path).expect("delete");
        assert!(repo.get_folder(&path).expect("get after delete").is_none());
    }
}
