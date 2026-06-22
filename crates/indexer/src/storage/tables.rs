//! Database table definitions for the indexer context.
//!
//! Defines primary tables for file and directory records ([`FILES`], [`DIRS`])
//! and secondary index tables for lookups by path, basename, parent, and
//! format. Uses [`UuidTable`], [`PathUuidTable`], and [`UuidMultimap`]
//! wrappers to avoid coupling entity types to the [`redb::Value`] trait.

use redb::MultimapTableDefinition;
use trace_db::{PathUuidTable, UuidMultimap, UuidTable, impl_redb_uuid};

use crate::model::FsRecordId;

impl_redb_uuid!(FsRecordId);

/// Primary table mapping file record IDs to their archived bytes.
pub(crate) const FILES: UuidTable<FsRecordId, &[u8]> = UuidTable::new("files");
/// Primary table mapping directory record IDs to their archived bytes.
pub(crate) const DIRS: UuidTable<FsRecordId, &[u8]> = UuidTable::new("dirs");

/// Secondary index: file record ID by path (unique).
pub(crate) const FILE_ID_BY_PATH: PathUuidTable<FsRecordId> =
    PathUuidTable::new("file_id_by_path");
/// Secondary index: directory record ID by path (unique).
pub(crate) const DIR_ID_BY_PATH: PathUuidTable<FsRecordId> =
    PathUuidTable::new("dir_id_by_path");
/// Secondary index: file record IDs by basename (multimap).
pub(crate) const FILE_IDS_BY_BASENAME: MultimapTableDefinition<
    &str,
    FsRecordId,
> = MultimapTableDefinition::new("file_ids_by_basename");
/// Secondary index: file record IDs by parent directory (multimap).
pub(crate) const FILE_IDS_BY_PARENT: UuidMultimap<FsRecordId, FsRecordId> =
    UuidMultimap::new("file_ids_by_parent");
/// Secondary index: directory record IDs by parent directory (multimap).
pub(crate) const DIR_IDS_BY_PARENT: UuidMultimap<FsRecordId, FsRecordId> =
    UuidMultimap::new("dir_ids_by_parent");
/// Secondary index: file record IDs by format string (multimap).
pub(crate) const FILE_IDS_BY_FORMAT: MultimapTableDefinition<&str, FsRecordId> =
    MultimapTableDefinition::new("file_ids_by_format");

#[cfg(test)]
mod tests {
    use trace_db::Store;

    use super::*;

    #[test]
    fn test_table_definitions() {
        let (_tempdir, store) = Store::open_temp().unwrap();

        store
            .write(|tx| {
                tx.inner.open_table(FILES.definition())?;
                tx.inner.open_table(DIRS.definition())?;
                tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
                tx.inner.open_table(DIR_ID_BY_PATH.definition())?;
                tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
                tx.inner
                    .open_multimap_table(FILE_IDS_BY_PARENT.definition())?;
                tx.inner.open_multimap_table(DIR_IDS_BY_PARENT.definition())?;
                tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
                Ok(())
            })
            .unwrap();
    }
}
