//! Unified repository for vault metadata storage.
//!
//! Stores vault file and folder entries as rkyv-serialized domain types.

use crate::{
    db::{BatchReader, BatchWriter, Database},
    vault::{
        VAULT_FILES_BY_PATH, VAULT_FOLDERS_BY_PATH,
        error::VaultRepositoryError,
        model::{VaultFile, VaultFolder, VaultPath},
    },
};

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
            .map_err(VaultRepositoryError::Storage)?;
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
