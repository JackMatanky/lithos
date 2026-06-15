use redb::{MultimapTableDefinition, TypeName, Value};

use crate::{
    db::{PathUuidTable, UuidMultimap, UuidTable},
    impl_redb_uuid,
    indexer::model::{DirRecord, FileRecord, FsRecordId},
};

impl_redb_uuid!(FsRecordId);

/// Implements [`redb::Value`] for entity types using rkyv serialization.
///
/// ## Design
///
/// This macro produces owned deserialization (`from_bytes` → full struct). The
/// [`crate::db::ArchivedEntity`] trait also provides `with_archived` for
/// zero-copy access to the archived form, but that path requires storing raw
/// bytes in the DB table (e.g., `TableDefinition<K, &[u8]>`) and manually
/// managing alignment — the pattern used by the vault context.
///
/// The indexer always returns owned values via the [`ReadRepository`] trait, so
/// full owned deserialization is the correct tradeoff. The zero-copy path would
/// add complexity without eliminating the final materialization step.
///
/// ## Validation
///
/// Uses `rkyv::from_bytes` with [`rkyv::rancor::Error`], which invokes
/// bytecheck validation on every read. Stale/corrupt pages are caught at
/// deserialization time with a panic (required by the [`redb::Value`] trait
/// signature — no `Result` return type).
///
/// [`ReadRepository`]: crate::indexer::repository::ReadRepository
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

pub(crate) const FILES: UuidTable<FsRecordId, FileRecord> =
    UuidTable::new("files");
pub(crate) const DIRS: UuidTable<FsRecordId, DirRecord> =
    UuidTable::new("dirs");

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
