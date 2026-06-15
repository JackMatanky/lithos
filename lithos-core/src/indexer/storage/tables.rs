use redb::MultimapTableDefinition;

use crate::{
    db::{PathUuidTable, UuidMultimap, UuidTable},
    impl_redb_uuid,
    indexer::model::FsRecordId,
};

impl_redb_uuid!(FsRecordId);

pub(crate) const FILES: UuidTable<FsRecordId, &[u8]> = UuidTable::new("files");
pub(crate) const DIRS: UuidTable<FsRecordId, &[u8]> = UuidTable::new("dirs");

pub(crate) const FILE_ID_BY_PATH: PathUuidTable<FsRecordId> =
    PathUuidTable::new("file_id_by_path");
pub(crate) const DIR_ID_BY_PATH: PathUuidTable<FsRecordId> =
    PathUuidTable::new("dir_id_by_path");
pub(crate) const FILE_IDS_BY_BASENAME: MultimapTableDefinition<
    &str,
    FsRecordId,
> = MultimapTableDefinition::new("file_ids_by_basename");
pub(crate) const FILE_IDS_BY_PARENT: UuidMultimap<FsRecordId, FsRecordId> =
    UuidMultimap::new("file_ids_by_parent");
pub(crate) const DIR_IDS_BY_PARENT: UuidMultimap<FsRecordId, FsRecordId> =
    UuidMultimap::new("dir_ids_by_parent");
pub(crate) const FILE_IDS_BY_FORMAT: MultimapTableDefinition<&str, FsRecordId> =
    MultimapTableDefinition::new("file_ids_by_format");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Store;

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
