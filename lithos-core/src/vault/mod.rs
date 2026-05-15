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
pub(crate) const FILE_VIEWS: UuidTable<FileId, &[u8]> =
    UuidTable::new("file_views");

/// Primary directory view table keyed by [`DirId`].
pub(crate) const DIR_VIEWS: UuidTable<DirId, &[u8]> =
    UuidTable::new("dir_views");

/// Path-to-file-id index for exact file lookup by normalized path.
pub(crate) const FILE_ID_BY_PATH: PathTable<FileId> =
    PathTable::new("file_id_by_path");

/// Path-to-dir-id index for exact directory lookup by normalized path.
pub(crate) const DIR_ID_BY_PATH: PathTable<DirId> =
    PathTable::new("dir_id_by_path");

/// Basename-to-file-id multimap for wikilink-style lookups.
pub(crate) const FILE_IDS_BY_BASENAME: MultimapTableDefinition<&str, FileId> =
    MultimapTableDefinition::new("file_ids_by_basename");

/// Parent-dir-id-to-file-id multimap for child listing queries.
pub(crate) const FILE_IDS_BY_PARENT: UuidMultimap<DirId, FileId> =
    UuidMultimap::new("file_ids_by_parent");

/// Format-to-file-id multimap for format queries.
pub(crate) const FILE_IDS_BY_FORMAT: MultimapTableDefinition<&str, FileId> =
    MultimapTableDefinition::new("file_ids_by_format");
