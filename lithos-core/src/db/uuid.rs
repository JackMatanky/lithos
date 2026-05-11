//! [`UuidV7`](crate::utils::UuidV7) `redb` integration.
//!
//! Provides wrapper-first DB key support for
//! [`UuidV7`](crate::utils::UuidV7)-backed ID types.

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

use redb::{Key, Value};

/// Marker trait for domain ID wrappers that are valid
/// [`UuidV7`](crate::utils::UuidV7) DB key types.
pub trait UuidV7DbType: sealed::Sealed + Value + Key {}

/// Derive macro to implement `redb::Value` and `redb::Key` for
/// [`UuidV7`](crate::utils::UuidV7) wrappers.
///
/// Usage:
/// ```ignore
/// use lithos_core::db::uuid::impl_redb_uuid;
///
/// impl_redb_uuid!(crate::schema::identifier::SchemaId);
/// impl_redb_uuid!(crate::note::identifier::NoteId);
/// ```
///
/// # Requirements
///
/// The wrapper must be a tuple struct with [`UuidV7`](crate::utils::UuidV7)
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
                let Ok(uuid) = $crate::utils::UuidV7::try_from(data) else {
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
                redb::TypeName::new(concat!("lithos::", stringify!($wrapper)))
            }
        }

        impl redb::Key for $wrapper {
            #[inline]
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                data1.cmp(data2)
            }
        }

        impl $crate::db::sealed::Sealed for $wrapper {}
        impl $crate::db::UuidV7DbType for $wrapper {}
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestId(crate::utils::UuidV7);

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
}
