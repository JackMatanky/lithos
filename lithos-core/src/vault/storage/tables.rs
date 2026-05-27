//! Table definitions for vault storage.
//!
//! Defines the redb table layout used by [`RedbRepository`]. Each constant is a
//! type-safe wrapper ([`UuidTable`], [`PathTable`], [`UuidMultimap`]) that
//! enforces key/value types at compile time.
//!
//! # Table Inventory
//!
//! | Category   | Name                   | Key Type   | Value Type | Purpose                     |
//! | ---------- | ---------------------- | ---------- | ---------- | --------------------------- |
//! | Primary    | `FILE_VIEWS`           | [`FileId`] | `&[u8]`    | O(1) file lookup by ID      |
//! | Primary    | `DIR_VIEWS`            | [`DirId`]  | `&[u8]`    | O(1) directory lookup by ID |
//! | Path index | `FILE_ID_BY_PATH`      | `String`   | [`FileId`] | Path-to-ID resolution       |
//! | Path index | `DIR_ID_BY_PATH`       | `String`   | [`DirId`]  | Path-to-ID resolution       |
//! | Reverse    | `PATH_BY_FILE_ID`      | [`FileId`] | `String`   | O(1) delete path recovery   |
//! | Reverse    | `PATH_BY_DIR_ID`       | [`DirId`]  | `String`   | O(1) delete path recovery   |
//! | Multimap   | `FILE_IDS_BY_BASENAME` | `&str`     | [`FileId`] | Wikilink-style lookup       |
//! | Multimap   | `FILE_IDS_BY_PARENT`   | [`DirId`]  | [`FileId`] | Child listing queries       |
//! | Multimap   | `FILE_IDS_BY_FORMAT`   | `&str`     | [`FileId`] | Format filter queries       |
//!
//! # Invariant
//!
//! All five file tables and both directory tables are updated atomically within
//! the same [`redb::WriteTransaction`]. This guarantees that indexes never
//! diverge from primary data.
//!
//! [`RedbRepository`]: super::RedbRepository

use redb::MultimapTableDefinition;

use super::super::model::{DirId, FileId};
use crate::db::{PathUuidTable, UuidMultimap, UuidPathTable, UuidTable};

/// File views with bincode serialization.
///
/// Authoritative store for file metadata. All other file tables
/// (`FILE_ID_BY_PATH`, `FILE_IDS_BY_*`) derive from this primary table and are
/// kept in sync within the same write transaction.
///
/// Keys use the raw 16-byte UUID representation (via [`UuidTable`]) to avoid
/// the overhead of string formatting on hot read paths.
///
/// Key: `FileId`
/// Value: bincode-serialized `FileView`
pub(crate) const FILE_VIEWS: UuidTable<FileId, &[u8]> =
    UuidTable::new("file_views");

/// Directory views with bincode serialization.
///
/// Authoritative store for directory metadata. Updated atomically with
/// `DIR_ID_BY_PATH` inside the same write transaction.
///
/// Key: `DirId`
/// Value: bincode-serialized `DirView`
pub(crate) const DIR_VIEWS: UuidTable<DirId, &[u8]> =
    UuidTable::new("dir_views");

/// Path-to-file-id index for path-based file lookup.
///
/// Supports `find_file_view_by_path`. Updated in lockstep with `FILE_VIEWS`
/// and all `FILE_IDS_BY_*` multimaps within the same write transaction.
///
/// Uses the [`PathUuidTable`] wrapper (path keys → UUID values) because paths
/// are discovered at runtime from the filesystem, not known at compile time.
///
/// Key: vault-relative path string
/// Value: `FileId`
pub(crate) const FILE_ID_BY_PATH: PathUuidTable<FileId> =
    PathUuidTable::new("file_id_by_path");

/// Path-to-dir-id index for path-based directory lookup.
///
/// Supports `find_dir_view_by_path`. Updated atomically with `DIR_VIEWS`.
///
/// Key: vault-relative path string
/// Value: `DirId`
pub(crate) const DIR_ID_BY_PATH: PathUuidTable<DirId> =
    PathUuidTable::new("dir_id_by_path");

/// Basename-to-file-id multimap for wikilink-style resolution.
///
/// Maps a file's basename (e.g., `"index"`) to all `FileId`s sharing that name.
/// Used by `find_file_views_by_basename` when resolving `[[wikilink]]`
/// references.
///
/// Uses the raw [`redb::MultimapTableDefinition`] because redb's multimap API
/// requires `Key` rather than `Value` for the value type; the crate's
/// [`UuidMultimap`] wrapper only handles UUID keys.
///
/// Key: basename string
/// Value: `FileId`
pub(crate) const FILE_IDS_BY_BASENAME: MultimapTableDefinition<&str, FileId> =
    MultimapTableDefinition::new("file_ids_by_basename");

/// Parent-to-children multimap for child listing queries.
///
/// Maps a parent `DirId` to its immediate child `FileId`s. Used by
/// `find_file_views_by_parent` for tree-traversal queries.
///
/// Uses the [`UuidMultimap`] wrapper to store the parent `DirId` in raw UUID
/// form, avoiding string-key overhead for this hot tree-query path.
///
/// Key: `DirId`
/// Value: `FileId`
pub(crate) const FILE_IDS_BY_PARENT: UuidMultimap<DirId, FileId> =
    UuidMultimap::new("file_ids_by_parent");

/// Format-to-file-id multimap for format-filtered queries.
///
/// Maps a format discriminant string (e.g., `"markdown"`, `"org"`) to all
/// `FileId`s of that type. Used by `list_file_views_by_format`.
///
/// Uses the raw [`redb::MultimapTableDefinition`] because format strings are
/// `&str`, not UUIDs — the same constraint as `FILE_IDS_BY_BASENAME`.
///
/// Key: format string
/// Value: `FileId`
pub(crate) const FILE_IDS_BY_FORMAT: MultimapTableDefinition<&str, FileId> =
    MultimapTableDefinition::new("file_ids_by_format");

/// Reverse index for O(1) file path recovery during delete operations.
///
/// Enables [`FileDeleteContext::load`] to avoid O(N) scan of `FILE_ID_BY_PATH`.
/// Kept in sync with `FILE_ID_BY_PATH` within the same write transaction.
///
/// Before this optimization, deleting a file required iterating the entire
/// `FILE_ID_BY_PATH` table to find the path for a given `FileId`. With this
/// reverse index, delete operations achieve O(1) path lookup via direct hash
/// table access.
///
/// Uses the [`UuidPathTable`] wrapper (UUID keys → path values) to enforce
/// type-safe path handling without exposing raw String types.
///
/// Key: `FileId`
/// Value: vault-relative path string
///
/// [`FileDeleteContext::load`]: super::write::FileDeleteContext::load
pub(crate) const PATH_BY_FILE_ID: UuidPathTable<FileId> =
    UuidPathTable::new("path_by_file_id");

/// Reverse index for O(1) directory path recovery during delete operations.
///
/// Enables [`DirDeleteContext::load`] to avoid O(1) scan of `DIR_ID_BY_PATH`.
/// Updated atomically with `DIR_ID_BY_PATH` within the same write transaction.
///
/// Uses the [`UuidPathTable`] wrapper (UUID keys → path values) to enforce
/// type-safe path handling without exposing raw String types.
///
/// Key: `DirId`
/// Value: vault-relative path string
///
/// [`DirDeleteContext::load`]: super::write::DirDeleteContext::load
pub(crate) const PATH_BY_DIR_ID: UuidPathTable<DirId> =
    UuidPathTable::new("path_by_dir_id");

#[cfg(test)]
mod path_by_file_id {
    use super::*;
    use crate::db::Store;

    #[test]
    fn can_open_table() {
        let (_tempdir, store) = Store::open_temp().unwrap();

        let result = store.write(|tx| {
            tx.try_open_table(PATH_BY_FILE_ID.definition())?;
            Ok(())
        });

        assert!(result.is_ok(), "PATH_BY_FILE_ID table should be accessible");
    }
}

#[cfg(test)]
mod path_by_dir_id {
    use super::*;
    use crate::db::Store;

    #[test]
    fn can_open_table() {
        let (_tempdir, store) = Store::open_temp().unwrap();

        let result = store.write(|tx| {
            tx.try_open_table(PATH_BY_DIR_ID.definition())?;
            Ok(())
        });

        assert!(result.is_ok(), "PATH_BY_DIR_ID table should be accessible");
    }
}
