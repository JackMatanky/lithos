//! [`UuidV7`](traces_utils::UuidV7) `redb` integration.
//!
//! Provides wrapper-first DB key support for
//! [`UuidV7`](traces_utils::UuidV7)-backed ID types.

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

use std::borrow::Borrow;

use redb::{AccessGuard, Key, ReadableMultimapTable, ReadableTable, Value};

use crate::DbError;

/// The return type for batch get operations on standard tables.
pub type BatchGetResult<'a, V> =
    Result<Vec<Option<AccessGuard<'a, V>>>, DbError>;

/// Extension trait for UUID-keyed tables (Read).
pub trait UuidTableReadExt<K: UuidV7DbType, V: Value> {
    /// Batch get multiple UUIDs in the order provided.
    ///
    /// # Errors
    /// Returns [`DbError`] for underlying storage errors.
    fn get_many(&self, keys: &[K]) -> BatchGetResult<'_, V>;
}

impl<K, V, T> UuidTableReadExt<K, V> for T
where
    K: UuidV7DbType + Copy + 'static,
    V: Value + 'static,
    T: ReadableTable<K, V>,
    for<'a> K: Borrow<K::SelfType<'a>>,
{
    #[inline]
    fn get_many(&self, keys: &[K]) -> BatchGetResult<'_, V> {
        keys.iter().map(|k| self.get(*k).map_err(DbError::from)).collect()
    }
}

/// The return type for batch get operations on multimap tables.
pub type BatchGetMultimapResult<'a, V> =
    Result<Vec<Vec<AccessGuard<'a, V>>>, DbError>;

/// Extension trait for UUID-keyed multimap tables (Read).
pub trait UuidMultimapReadExt<K: UuidV7DbType, V: Key> {
    /// Batch get values for multiple UUIDs.
    ///
    /// # Errors
    /// Returns [`DbError`] for underlying storage errors.
    fn get_many_multimap(&self, keys: &[K]) -> BatchGetMultimapResult<'_, V>;
}

impl<K, V, T> UuidMultimapReadExt<K, V> for T
where
    K: UuidV7DbType + Copy + 'static,
    V: Key + 'static,
    T: ReadableMultimapTable<K, V>,
    for<'a> K: Borrow<K::SelfType<'a>>,
{
    #[inline]
    fn get_many_multimap(&self, keys: &[K]) -> BatchGetMultimapResult<'_, V> {
        keys.iter()
            .map(|k| {
                Ok(
                    self.get(*k).map_err(DbError::from)?.collect::<Result<
                        Vec<AccessGuard<'_, V>>,
                        redb::StorageError,
                    >>(
                    )?,
                )
            })
            .collect()
    }
}

/// Extension trait for UUID-keyed tables (Write).
pub trait UuidTableWriteExt<K: UuidV7DbType, V: Value> {
    /// Batch save multiple UUID-keyed items.
    ///
    /// # Errors
    /// Returns [`DbError`] for underlying storage errors.
    fn save_many(
        &mut self,
        items: &[(K, V::SelfType<'_>)],
    ) -> Result<(), DbError>;
}

impl<K, V> UuidTableWriteExt<K, V> for redb::Table<'_, K, V>
where
    K: UuidV7DbType + Copy + 'static,
    V: Value + 'static,
    for<'a> K: Borrow<K::SelfType<'a>>,
{
    #[inline]
    fn save_many(
        &mut self,
        items: &[(K, V::SelfType<'_>)],
    ) -> Result<(), DbError> {
        for (k, v) in items {
            self.insert(*k, v)?;
        }
        Ok(())
    }
}

/// Extension trait for UUID-keyed multimap tables (Write).
pub trait UuidMultimapWriteExt<K: UuidV7DbType, V: Key> {
    /// Batch save multiple values for a UUID key.
    ///
    /// # Errors
    /// Returns [`DbError`] for underlying storage errors.
    fn save_many_multimap(
        &mut self,
        key: K,
        values: &[V::SelfType<'_>],
    ) -> Result<(), DbError>;
}

impl<K, V> UuidMultimapWriteExt<K, V> for redb::MultimapTable<'_, K, V>
where
    K: UuidV7DbType + Copy + 'static,
    V: Key + 'static,
    for<'a> K: Borrow<K::SelfType<'a>>,
{
    #[inline]
    fn save_many_multimap(
        &mut self,
        key: K,
        values: &[V::SelfType<'_>],
    ) -> Result<(), DbError> {
        for v in values {
            self.insert(key, v)?;
        }
        Ok(())
    }
}

/// Marker trait for domain ID wrappers that are valid
/// [`UuidV7`](traces_utils::UuidV7) DB key types.
pub trait UuidV7DbType: sealed::Sealed + Value + Key {}

/// Derive macro to implement `redb::Value` and `redb::Key` for
/// [`UuidV7`](traces_utils::UuidV7) wrappers.
///
/// Usage:
/// ```ignore
/// use traces_db::uuid::impl_redb_uuid;
///
/// impl_redb_uuid!(traces_schema::identifier::SchemaId);
/// impl_redb_uuid!(traces_note::identifier::NoteId);
/// ```
///
/// # Requirements
///
/// The wrapper must be a tuple struct with [`UuidV7`](traces_utils::UuidV7)
/// as the first field. The inner field **must be accessible from `db::uuid`**
/// (e.g., `pub(crate) pub struct SchemaId(pub(crate) UuidV7);`).
///
/// This is necessary because the macro needs to:
/// - Construct the type: `Self(uuid)`
/// - Access bytes: `value.0.as_bytes()`
#[macro_export]
macro_rules! impl_redb_uuid {
    ($wrapper:ty) => {
        impl redb::Value for $wrapper {
            type AsBytes<'bytes> = Vec<u8>;
            type SelfType<'value> = $wrapper;

            #[inline]
            fn fixed_width() -> Option<usize> {
                Some(16)
            }

            #[inline]
            fn from_bytes<'bytes>(data: &'bytes [u8]) -> Self::SelfType<'bytes>
            where
                Self: 'bytes,
            {
                let Ok(uuid) = traces_utils::UuidV7::try_from(data) else {
                    panic!("UUID data from database must be valid UUIDv7");
                };

                Self(uuid)
            }

            #[inline]
            fn as_bytes<'value, 'source: 'value>(
                value: &'value Self::SelfType<'source>,
            ) -> Self::AsBytes<'value>
            where
                Self: 'source,
            {
                value.0.as_bytes().to_vec()
            }

            #[inline]
            fn type_name() -> redb::TypeName {
                redb::TypeName::new(concat!("traces::", stringify!($wrapper)))
            }
        }

        impl redb::Key for $wrapper {
            #[inline]
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                data1.cmp(data2)
            }
        }

        impl $crate::sealed::Sealed for $wrapper {}
        impl $crate::UuidV7DbType for $wrapper {}
    };
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestId(traces_utils::UuidV7);

    impl_redb_uuid!(TestId);

    fn accepts_uuid_db_type<T: UuidV7DbType>() {}

    #[test]
    fn wrapper_redb_value_impl_compiles() {
        let _: Option<usize> = TestId::fixed_width();
    }

    #[test]
    fn wrapper_redb_key_impl_compiles() {
        let result = TestId::compare(b"test1", b"test2");
        assert!(result.is_lt());
    }

    #[test]
    fn marker_trait_is_implemented_for_wrapper() {
        accepts_uuid_db_type::<TestId>();
    }

    #[test]
    fn get_many_returns_correct_order() {
        use redb::TableDefinition;
        use tempfile::tempdir;
        use traces_utils::UuidV7;

        use crate::Store;

        const TEST_TABLE: TableDefinition<TestId, &str> =
            TableDefinition::new("test");

        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path).unwrap();

        let id1 = TestId(UuidV7::new());
        let id2 = TestId(UuidV7::new());
        let id3 = TestId(UuidV7::new());

        store
            .write(|tx| -> Result<(), DbError> {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.insert(id1, "val1")?;
                table.insert(id3, "val3")?;
                Ok(())
            })
            .unwrap();

        store
            .read(|tx| -> Result<(), DbError> {
                let table = tx.inner.open_table(TEST_TABLE)?;
                let results = table.get_many(&[id1, id2, id3])?;

                assert_eq!(results.len(), 3);
                assert_eq!(
                    results.first().unwrap().as_ref().unwrap().value(),
                    "val1"
                );
                assert!(results.get(1).unwrap().is_none());
                assert_eq!(
                    results.get(2).unwrap().as_ref().unwrap().value(),
                    "val3"
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn save_many_inserts_atomically() {
        use redb::TableDefinition;
        use tempfile::tempdir;
        use traces_utils::UuidV7;

        use crate::Store;

        const TEST_TABLE: TableDefinition<TestId, &str> =
            TableDefinition::new("test");

        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path).unwrap();

        let id1 = TestId(UuidV7::new());
        let id2 = TestId(UuidV7::new());

        store
            .write(|tx| -> Result<(), DbError> {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.save_many(&[(id1, "val1"), (id2, "val2")])?;
                Ok(())
            })
            .unwrap();

        store
            .read(|tx| -> Result<(), DbError> {
                let table = tx.inner.open_table(TEST_TABLE)?;
                assert_eq!(table.get(id1)?.unwrap().value(), "val1");
                assert_eq!(table.get(id2)?.unwrap().value(), "val2");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn get_many_multimap_returns_all_values() {
        use redb::MultimapTableDefinition;
        use tempfile::tempdir;
        use traces_utils::UuidV7;

        use super::UuidMultimapReadExt;
        use crate::Store;

        const TEST_TABLE: MultimapTableDefinition<TestId, &str> =
            MultimapTableDefinition::new("test_multimap");

        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path).unwrap();

        let id1 = TestId(UuidV7::new());
        let id2 = TestId(UuidV7::new());

        store
            .write(|tx| -> Result<(), DbError> {
                let mut table = tx.inner.open_multimap_table(TEST_TABLE)?;
                table.insert(id1, "val1a")?;
                table.insert(id1, "val1b")?;
                table.insert(id2, "val2")?;
                Ok(())
            })
            .unwrap();

        store
            .read(|tx| -> Result<(), DbError> {
                let table = tx.inner.open_multimap_table(TEST_TABLE)?;
                let results = table.get_many_multimap(&[id1, id2])?;

                assert_eq!(results.len(), 2);
                assert_eq!(results.first().unwrap().len(), 2);
                assert_eq!(
                    results.first().unwrap().first().unwrap().value(),
                    "val1a"
                );
                assert_eq!(
                    results.first().unwrap().get(1).unwrap().value(),
                    "val1b"
                );
                assert_eq!(results.get(1).unwrap().len(), 1);
                assert_eq!(
                    results.get(1).unwrap().first().unwrap().value(),
                    "val2"
                );
                Ok(())
            })
            .unwrap();
    }
}
