//! Vault context for file and folder metadata tracking.
//!
//! This module owns vault-wide file discovery and metadata persistence. It
//! routes markdown files into the note processor while tracking all files and
//! folders in dedicated vault tables.

#![expect(
    clippy::pub_use,
    reason = "Vault module re-exports types for ergonomic access"
)]
#![expect(
    clippy::module_name_repetitions,
    reason = "Vault module names are explicit and scoped"
)]

use redb::MultimapTableDefinition;

use crate::db::{PathTable, UuidMultimap, UuidTable};

/// Vault error types.
pub mod error;
/// Vault domain types.
pub mod model;
/// Vault processor pipeline.
pub mod processor;
/// Vault repository and redb adapter.
pub mod storage;

pub use error::{
    VaultFileError, VaultPathError, VaultProcessError, VaultRepositoryError,
};
pub use model::{
    DirId, DirView, FileId, FileView, FsEntryView, NormalizedPath, VaultFile,
    VaultFolder, VaultPath,
};
pub use processor::{ScanMode, VaultProcessReport, VaultProcessor};
pub use storage::{RedbRepository, Repository};

/// Primary file view table keyed by [`FileId`].
pub(crate) const VAULT_FILE_VIEWS: UuidTable<FileId, &[u8]> =
    UuidTable::new("vault_file_views");

/// Primary directory view table keyed by [`DirId`].
pub(crate) const VAULT_DIR_VIEWS: UuidTable<DirId, &[u8]> =
    UuidTable::new("vault_dir_views");

/// Path-to-file-id index for exact file lookup by normalized path.
pub(crate) const VAULT_FILE_ID_BY_PATH: PathTable<FileId> =
    PathTable::new("vault_file_id_by_path");

/// Path-to-dir-id index for exact directory lookup by normalized path.
pub(crate) const VAULT_DIR_ID_BY_PATH: PathTable<DirId> =
    PathTable::new("vault_dir_id_by_path");

/// Basename-to-file-id multimap for wikilink-style lookups.
pub(crate) const VAULT_FILE_IDS_BY_BASENAME: MultimapTableDefinition<
    &str,
    FileId,
> = MultimapTableDefinition::new("vault_file_ids_by_basename");

/// Parent-dir-id-to-file-id multimap for child listing queries.
pub(crate) const VAULT_FILE_IDS_BY_PARENT: UuidMultimap<DirId, FileId> =
    UuidMultimap::new("vault_file_ids_by_parent");

/// Format-to-file-id multimap for format queries.
pub(crate) const VAULT_FILE_IDS_BY_FORMAT: MultimapTableDefinition<
    &str,
    FileId,
> = MultimapTableDefinition::new("vault_file_ids_by_format");

/// `VAULT_FILES_BY_PATH` stores serialized [`VaultFile`] values keyed by
/// vault-relative paths.
pub(crate) const VAULT_FILES_BY_PATH: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("vault_files_by_path");

/// `VAULT_FOLDERS_BY_PATH` stores serialized [`VaultFolder`] values keyed by
/// vault-relative paths.
pub(crate) const VAULT_FOLDERS_BY_PATH: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("vault_folders_by_path");
