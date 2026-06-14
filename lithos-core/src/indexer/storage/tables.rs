use redb::{MultimapTableDefinition, TableDefinition, TypeName, Value};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    fs::path::PathKey,
    impl_redb_uuid,
    indexer::model::{DirRecord, FileRecord, FsRecordId},
};

impl_redb_uuid!(FsRecordId);

// Implement redb::Value for FileRecord and DirRecord using rkyv
macro_rules! impl_rkyv_redb_value {
    ($type:ty) => {
        impl Value for $type {
            type AsBytes<'bytes> = Vec<u8>;
            type SelfType<'value> = $type;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'bytes>(data: &'bytes [u8]) -> Self::SelfType<'bytes>
            where
                Self: 'bytes,
            {
                rkyv::from_bytes::<$type, rkyv::rancor::Error>(data)
                    .expect("failed to deserialize")
            }

            fn as_bytes<'value, 'source: 'value>(
                value: &'value Self::SelfType<'source>,
            ) -> Self::AsBytes<'value> {
                rkyv::to_bytes::<rkyv::rancor::Error>(value)
                    .expect("failed to serialize")
                    .to_vec()
            }

            fn type_name() -> TypeName {
                TypeName::new(stringify!($type))
            }
        }
    };
}

impl_rkyv_redb_value!(FileRecord);
impl_rkyv_redb_value!(DirRecord);

pub(crate) const FILES: TableDefinition<FsRecordId, FileRecord> =
    TableDefinition::new("files");
pub(crate) const DIRS: TableDefinition<FsRecordId, DirRecord> =
    TableDefinition::new("dirs");

// TODO: PathKey needs to implement Value
// For now, let's use String as key for path-based lookups.
pub(crate) const FILE_ID_BY_PATH: TableDefinition<&str, FsRecordId> =
    TableDefinition::new("file_id_by_path");
pub(crate) const DIR_ID_BY_PATH: TableDefinition<&str, FsRecordId> =
    TableDefinition::new("dir_id_by_path");
pub(crate) const FILE_IDS_BY_BASENAME: MultimapTableDefinition<
    &str,
    FsRecordId,
> = MultimapTableDefinition::new("file_ids_by_basename");
pub(crate) const FILE_IDS_BY_PARENT: MultimapTableDefinition<
    FsRecordId,
    FsRecordId,
> = MultimapTableDefinition::new("file_ids_by_parent");
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
                tx.inner.open_table(FILES)?;
                tx.inner.open_table(DIRS)?;
                tx.inner.open_table(FILE_ID_BY_PATH)?;
                tx.inner.open_table(DIR_ID_BY_PATH)?;
                tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
                tx.inner.open_multimap_table(FILE_IDS_BY_PARENT)?;
                tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
                Ok(())
            })
            .unwrap();
    }
}
