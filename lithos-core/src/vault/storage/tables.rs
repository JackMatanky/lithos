//! Table definitions for vault storage.
//!
//! Defines typed table constants for file and directory views and their
//! indexes.

use redb::MultimapTableDefinition;

use super::super::model::{DirId, FileId};
use crate::db::{PathTable, UuidMultimap, UuidTable};

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
