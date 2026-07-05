//! rkyv serialization helpers for db module.
//!
//! These helpers provide safe serialization/deserialization for persistent
//! storage. They enforce strict validation and handle memory alignment
//! automatically.
//!
//! # Architecture Constraints
//!
//! - **Strict Validation**: All deserialization paths use `rkyv::access` (which
//!   invokes bytecheck). `access_unchecked` is explicitly forbidden to prevent
//!   undefined behavior from corrupt database pages.
//! - **Alignment**: `rkyv` 0.8 requires 16-byte alignment. These helpers
//!   abstract the complexity of `AlignedVec` and pointer arithmetic away from
//!   domain code.

use std::{borrow::Cow, fmt, marker::PhantomData};

use redb::{Key, Value};
use rkyv::{
    Archive, Deserialize, Portable, Serialize, api::high::HighDeserializer,
    util::AlignedVec,
};

use crate::DbError;

mod private {
    pub trait Sealed {}
}

/// Error classification for codec failures.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecErrorKind {
    /// Serialization to bytes failed.
    Encode,
    /// Archived byte validation failed.
    Access,
    /// Deserialization from archived bytes failed.
    Decode,
}

/// Errors from rkyv codec operations.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// Serialization to rkyv bytes failed.
    #[error("failed to serialize {type_name} with rkyv")]
    RkyvSerialize {
        /// Rust type being serialized.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },

    /// Archived byte validation failed.
    #[error("failed to validate archived {type_name}")]
    RkyvAccess {
        /// Rust type being validated.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },

    /// Deserialization from archived bytes failed.
    #[error("failed to deserialize archived {type_name}")]
    RkyvDeserialize {
        /// Rust type being deserialized.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },
}

impl CodecError {
    /// Classify codec error for stable branching.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> CodecErrorKind {
        match self {
            Self::RkyvSerialize {
                ..
            } => CodecErrorKind::Encode,
            Self::RkyvAccess {
                ..
            } => CodecErrorKind::Access,
            Self::RkyvDeserialize {
                ..
            } => CodecErrorKind::Decode,
        }
    }
}

/// Types that can be encoded to rkyv bytes.
pub trait RkyvEncode:
    private::Sealed
    + Archive
    + for<'ser> Serialize<
        rkyv::api::high::HighSerializer<
            AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'ser>,
            rkyv::rancor::Error,
        >,
    >
{
}

impl<T> private::Sealed for T where T: Archive {}

impl<T> RkyvEncode for T where
    T: Archive
        + for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >
{
}

/// Types that can be decoded from validated rkyv bytes.
pub trait RkyvDecode: private::Sealed + Archive + Sized {
    /// Decode bytes into an owned value.
    ///
    /// # Errors
    /// Returns [`CodecError`] if validation or deserialization fails.
    fn decode_from_rkyv_bytes(bytes: &[u8]) -> Result<Self, CodecError>;

    /// Access an archived value without materializing it.
    ///
    /// # Errors
    /// Returns [`CodecError`] if archived byte validation fails.
    fn with_archived_rkyv_bytes<R, F>(
        bytes: &[u8],
        f: F,
    ) -> Result<R, CodecError>
    where
        F: FnOnce(&Self::Archived) -> R;
}

impl<T> RkyvDecode for T
where
    T: Archive,
    T::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
{
    #[inline]
    fn decode_from_rkyv_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        decode_rkyv(bytes)
    }

    #[inline]
    fn with_archived_rkyv_bytes<R, F>(
        bytes: &[u8],
        f: F,
    ) -> Result<R, CodecError>
    where
        F: FnOnce(&Self::Archived) -> R,
    {
        with_archived_rkyv::<T, R, F>(bytes, f)
    }
}

/// Typed rkyv byte storage for borrowed database reads and owned insert values.
pub struct RkyvBytes<'a, T> {
    bytes: Cow<'a, [u8]>,
    _ty: PhantomData<T>,
}

impl<T> fmt::Debug for RkyvBytes<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RkyvBytes")
            .field("type_name", &std::any::type_name::<T>())
            .field("len", &self.bytes.len())
            .finish()
    }
}

impl<'a, T> RkyvBytes<'a, T> {
    /// Borrow typed rkyv bytes from database storage.
    #[inline]
    #[must_use]
    pub const fn borrowed(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            _ty: PhantomData,
        }
    }

    /// Own typed rkyv bytes for insertion.
    #[inline]
    #[must_use]
    pub fn owned(bytes: Vec<u8>) -> RkyvBytes<'static, T> {
        RkyvBytes {
            bytes: Cow::Owned(bytes),
            _ty: PhantomData,
        }
    }

    /// Borrow the raw stored bytes.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Decode bytes into an owned value.
    ///
    /// # Errors
    /// Returns [`CodecError`] if validation or deserialization fails.
    #[inline]
    pub fn decode(&self) -> Result<T, CodecError>
    where
        T: RkyvDecode,
    {
        T::decode_from_rkyv_bytes(self.as_bytes())
    }

    /// Access the archived value without materializing it.
    ///
    /// # Errors
    /// Returns [`CodecError`] if archived byte validation fails.
    #[inline]
    pub fn with_archived<R, F>(&self, f: F) -> Result<R, CodecError>
    where
        T: RkyvDecode,
        F: FnOnce(&T::Archived) -> R,
    {
        T::with_archived_rkyv_bytes(self.as_bytes(), f)
    }
}

impl<T> RkyvBytes<'static, T> {
    /// Encode a value to owned rkyv bytes.
    ///
    /// # Errors
    /// Returns [`CodecError`] if serialization fails.
    #[inline]
    pub fn encode(value: &T) -> Result<Self, CodecError>
    where
        T: RkyvEncode,
    {
        encode_rkyv(value)
    }
}

/// redb adapter for typed rkyv bytes.
#[derive(Clone, Copy)]
pub struct DbRkyvType<T>(PhantomData<T>);

impl<T> fmt::Debug for DbRkyvType<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DbRkyvType").field(&std::any::type_name::<T>()).finish()
    }
}

impl<T: 'static> Value for DbRkyvType<T> {
    type AsBytes<'a> = Cow<'a, [u8]>;
    type SelfType<'a> = RkyvBytes<'a, T>;

    #[inline]
    fn fixed_width() -> Option<usize> {
        None
    }

    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        RkyvBytes::borrowed(data)
    }

    #[inline]
    fn as_bytes<'a, 'source: 'a>(
        value: &'a Self::SelfType<'source>,
    ) -> Self::AsBytes<'a>
    where
        Self: 'source,
    {
        Cow::Borrowed(value.as_bytes())
    }

    #[inline]
    fn type_name() -> redb::TypeName {
        redb::TypeName::new(std::any::type_name::<Self>())
    }
}

impl<T> Key for DbRkyvType<T>
where
    T: RkyvDecode + Ord + 'static,
{
    #[inline]
    #[allow(
        clippy::expect_used,
        reason = "redb::Key::compare cannot return Result for invalid stored \
                  bytes"
    )]
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        // ponytail: decode-on-compare; specialize hot keys only after
        // profiling.
        let left = RkyvBytes::<T>::borrowed(data1)
            .decode()
            .expect("invalid rkyv key bytes");
        let right = RkyvBytes::<T>::borrowed(data2)
            .decode()
            .expect("invalid rkyv key bytes");
        left.cmp(&right)
    }
}

fn encode_rkyv<T>(value: &T) -> Result<RkyvBytes<'static, T>, CodecError>
where
    T: RkyvEncode,
{
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map(|bytes| RkyvBytes::owned(bytes.into_vec()))
        .map_err(|source| CodecError::RkyvSerialize {
            type_name: std::any::type_name::<T>(),
            source,
        })
}

fn decode_rkyv<T>(bytes: &[u8]) -> Result<T, CodecError>
where
    T: Archive,
    T::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
{
    with_archived_rkyv::<T, _, _>(bytes, |archived| {
        rkyv::deserialize::<T, rkyv::rancor::Error>(archived).map_err(
            |source| CodecError::RkyvDeserialize {
                type_name: std::any::type_name::<T>(),
                source,
            },
        )
    })?
}

fn with_archived_rkyv<T, R, F>(bytes: &[u8], f: F) -> Result<R, CodecError>
where
    T: Archive,
    T::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
    F: FnOnce(&T::Archived) -> R,
{
    #[expect(
        clippy::as_conversions,
        reason = "Pointer to usize conversion required for alignment check"
    )]
    let ptr_usize = bytes.as_ptr() as usize;

    if ptr_usize.is_multiple_of(16) {
        let archived = rkyv::access::<T::Archived, rkyv::rancor::Error>(bytes)
            .map_err(|source| CodecError::RkyvAccess {
                type_name: std::any::type_name::<T>(),
                source,
            })?;
        Ok(f(archived))
    } else {
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);
        let archived =
            rkyv::access::<T::Archived, rkyv::rancor::Error>(&aligned)
                .map_err(|source| CodecError::RkyvAccess {
                    type_name: std::any::type_name::<T>(),
                    source,
                })?;
        Ok(f(archived))
    }
}

/// A trait for types that can be safely serialized and deserialized to/from the
/// database.
///
/// This trait acts as a codec boundary, hiding the complex `rkyv` trait bounds
/// and validation logic from domain storage adapters. It provides both owned
/// (`from_bytes`) and zero-copy (`with_archived`) read paths.
#[deprecated(note = "use RkyvBytes with RkyvEncode/RkyvDecode")]
pub trait ArchivedEntity: Sized {
    /// The zero-copy view of the entity.
    type View<'a>
    where
        Self: 'a;

    /// Serializes the entity to bytes, ensuring correct alignment.
    ///
    /// # Errors
    /// Returns [`DbError::Codec`] if serialization fails.
    fn to_bytes(&self) -> Result<AlignedVec, DbError>;

    /// Deserializes the entity from bytes, validating them first.
    ///
    /// # Errors
    /// Returns [`DbError::Codec`] if validation or deserialization fails.
    fn from_bytes(bytes: &[u8]) -> Result<Self, DbError>;

    /// Accesses the zero-copy view of the entity from bytes.
    ///
    /// # Errors
    /// Returns [`DbError::Codec`] if validation fails.
    fn with_archived<R, F>(bytes: &[u8], f: F) -> Result<R, DbError>
    where
        F: FnOnce(Self::View<'_>) -> R;
}

// Blanket implementation for any type that derives the necessary rkyv traits.
#[allow(deprecated, reason = "compatibility shim implements deprecated trait")]
impl<T> ArchivedEntity for T
where
    T: 'static
        + Archive
        + for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
    T::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
{
    type View<'a> = &'a T::Archived;

    #[inline]
    fn to_bytes(&self) -> Result<AlignedVec, DbError> {
        let bytes = encode_rkyv(self).map_err(DbError::from)?;
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes.as_bytes());
        Ok(aligned)
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, DbError> {
        decode_rkyv(bytes).map_err(DbError::from)
    }

    #[inline]
    fn with_archived<R, F>(bytes: &[u8], f: F) -> Result<R, DbError>
    where
        F: FnOnce(Self::View<'_>) -> R,
    {
        with_archived_rkyv::<T, R, F>(bytes, f).map_err(DbError::from)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use rkyv::{Archive, Serialize};

    use super::*;

    #[derive(
        Archive, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord,
    )]
    #[rkyv(derive(Debug, PartialEq))]
    struct TestData {
        id: u32,
        name: String,
    }

    #[derive(Archive, Serialize, Deserialize)]
    struct NoDebugData {
        id: u32,
    }

    #[derive(
        Archive, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
    )]
    struct TestKey {
        rank: i32,
    }

    fn test_rancor_error() -> rkyv::rancor::Error {
        use rkyv::rancor::Source;

        rkyv::rancor::Error::new(std::io::Error::other("test codec error"))
    }

    mod codec_error {
        use super::*;

        #[test]
        fn kind_classifies_codec_error_variants() {
            let encode = CodecError::RkyvSerialize {
                type_name: "TestData",
                source: test_rancor_error(),
            };
            let access = CodecError::RkyvAccess {
                type_name: "TestData",
                source: test_rancor_error(),
            };
            let decode = CodecError::RkyvDeserialize {
                type_name: "TestData",
                source: test_rancor_error(),
            };

            assert_eq!(encode.kind(), CodecErrorKind::Encode);
            assert_eq!(access.kind(), CodecErrorKind::Access);
            assert_eq!(decode.kind(), CodecErrorKind::Decode);
        }

        #[test]
        fn db_error_codec_variant_returns_codec_kind() {
            let err = DbError::from(CodecError::RkyvAccess {
                type_name: "TestData",
                source: test_rancor_error(),
            });

            assert_eq!(err.kind(), crate::DbErrorKind::Codec);
        }
    }

    mod to_bytes {
        use super::*;

        fn accepts_decode<T: RkyvDecode>() {}

        #[test]
        fn exposes_rkyv_decode_bound_for_public_decode_api() {
            accepts_decode::<TestData>();
        }

        #[test]
        fn rkyv_bytes_encode_decode_roundtrips_entity() {
            let original = TestData {
                id: 7,
                name: "typed-bytes".to_owned(),
            };

            let bytes = RkyvBytes::<TestData>::encode(&original).unwrap();
            let decoded = bytes.decode().unwrap();

            assert_eq!(decoded, original);
        }

        #[test]
        fn rkyv_bytes_with_archived_reads_field() {
            let original = TestData {
                id: 8,
                name: "archived".to_owned(),
            };
            let bytes = RkyvBytes::<TestData>::encode(&original).unwrap();

            let id = bytes.with_archived(|archived| archived.id).unwrap();

            assert_eq!(id, 8);
        }

        #[test]
        fn rkyv_bytes_decode_returns_access_error_for_invalid_bytes() {
            let err = RkyvBytes::<TestData>::borrowed(&[0, 1, 2, 3])
                .decode()
                .unwrap_err();

            assert!(matches!(err, CodecError::RkyvAccess { .. }));
            assert_eq!(err.kind(), CodecErrorKind::Access);
        }

        #[test]
        fn debug_impls_do_not_require_debug_entity() {
            let bytes = RkyvBytes::<NoDebugData>::borrowed(&[]);
            let db_type = DbRkyvType::<NoDebugData>(PhantomData);

            let _ = format!("{bytes:?} {db_type:?}");
        }

        #[test]
        fn db_rkyv_type_roundtrips_through_redb_table() {
            use redb::ReadableDatabase;

            const TABLE: redb::TableDefinition<&str, DbRkyvType<TestData>> =
                redb::TableDefinition::new("typed_rkyv_value");
            let original = TestData {
                id: 9,
                name: "stored".to_owned(),
            };
            let encoded = RkyvBytes::<TestData>::encode(&original).unwrap();
            let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
            let db = redb::Database::create(tmp.path()).expect("db");

            let write_tx = db.begin_write().expect("tx");
            {
                let mut table = write_tx.open_table(TABLE).expect("open");
                table.insert("item", &encoded).expect("insert");
            }
            write_tx.commit().expect("commit");

            let read_tx = db.begin_read().expect("tx");
            let table = read_tx.open_table(TABLE).expect("open");
            let stored = table.get("item").expect("get").expect("value");

            assert_eq!(stored.value().decode().unwrap(), original);
        }

        #[test]
        fn db_rkyv_type_roundtrips_as_redb_multimap_value() {
            use redb::ReadableDatabase;

            const TABLE: redb::MultimapTableDefinition<
                &str,
                DbRkyvType<TestData>,
            > = redb::MultimapTableDefinition::new("typed_rkyv_key");
            let original = TestData {
                id: 10,
                name: "mapped".to_owned(),
            };
            let encoded = RkyvBytes::<TestData>::encode(&original).unwrap();
            let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
            let db = redb::Database::create(tmp.path()).expect("db");

            let write_tx = db.begin_write().expect("tx");
            {
                let mut table =
                    write_tx.open_multimap_table(TABLE).expect("open");
                table.insert("items", &encoded).expect("insert");
            }
            write_tx.commit().expect("commit");

            let read_tx = db.begin_read().expect("tx");
            let table = read_tx.open_multimap_table(TABLE).expect("open");
            let mut values = table.get("items").expect("get");
            let stored = values.next().expect("next").expect("value");

            assert_eq!(stored.value().decode().unwrap(), original);
        }

        #[test]
        fn db_rkyv_type_compare_matches_entity_ordering() {
            let low = RkyvBytes::<TestKey>::encode(&TestKey {
                rank: -1,
            })
            .unwrap();
            let high = RkyvBytes::<TestKey>::encode(&TestKey {
                rank: 1,
            })
            .unwrap();

            let ordering =
                DbRkyvType::<TestKey>::compare(low.as_bytes(), high.as_bytes());

            assert!(ordering.is_lt());
        }

        #[test]
        #[allow(deprecated, reason = "compatibility shim test")]
        fn produces_aligned_bytes() {
            let data = TestData {
                id: 42,
                name: "test".to_owned(),
            };
            let result = data.to_bytes();
            assert!(result.is_ok());
            let bytes = result.unwrap();
            assert!(!bytes.is_empty());
        }
    }

    mod from_bytes {
        #![allow(deprecated, reason = "compatibility shim tests")]

        use super::*;

        #[test]
        fn roundtrips_valid_entity() {
            let original = TestData {
                id: 123,
                name: "hello".to_owned(),
            };
            let bytes = original.to_bytes().unwrap();
            let deserialized: TestData = TestData::from_bytes(&bytes).unwrap();
            assert_eq!(original, deserialized);
        }

        #[test]
        fn returns_error_for_invalid_bytes() {
            let invalid_bytes = &[0u8, 1, 2, 3];
            let result: Result<TestData, DbError> =
                TestData::from_bytes(invalid_bytes);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, DbError::Codec(_)));
            assert_eq!(err.kind(), crate::DbErrorKind::Codec);
        }

        #[test]
        #[expect(
            clippy::indexing_slicing,
            clippy::integer_division,
            clippy::integer_division_remainder_used,
            reason = "Test intentionally truncates bytes to verify error \
                      handling"
        )]
        fn returns_error_for_truncated_bytes() {
            let original = TestData {
                id: 1,
                name: "test".to_owned(),
            };
            let bytes = original.to_bytes().unwrap();
            let truncated = &bytes[..bytes.len() / 2];
            let result: Result<TestData, DbError> =
                TestData::from_bytes(truncated);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, DbError::Codec(_)));
            assert_eq!(err.kind(), crate::DbErrorKind::Codec);
        }
    }

    mod with_archived {
        #![allow(deprecated, reason = "compatibility shim tests")]

        use super::*;

        #[test]
        fn provides_zero_copy_access() {
            let original = TestData {
                id: 999,
                name: "zero-copy".to_owned(),
            };
            let bytes = original.to_bytes().unwrap();
            let result = TestData::with_archived(&bytes, |archived| {
                assert_eq!(archived.id, 999);
                archived.name.as_str().to_owned()
            });
            assert_eq!(result.unwrap(), "zero-copy");
        }

        #[test]
        fn uses_fast_path_when_aligned() {
            let original = TestData {
                id: 1,
                name: "aligned".to_owned(),
            };
            let bytes = original.to_bytes().unwrap();
            #[expect(
                clippy::as_conversions,
                reason = "Pointer to usize conversion required for alignment \
                          check"
            )]
            let ptr = bytes.as_ptr() as usize;
            if ptr.is_multiple_of(16) {
                let result =
                    TestData::with_archived(&bytes, |archived| archived.id);
                assert_eq!(result.unwrap(), 1);
            }
        }

        #[test]
        #[expect(
            clippy::indexing_slicing,
            reason = "Test intentionally uses unaligned slice to verify error \
                      handling"
        )]
        fn returns_error_for_invalid_unaligned_data() {
            let original = TestData {
                id: 2,
                name: "unaligned".to_owned(),
            };
            let bytes = original.to_bytes().unwrap();
            let unaligned: Vec<u8> = bytes.iter().copied().collect();
            let result = TestData::with_archived(&unaligned[1..], |archived| {
                archived.id
            });
            result.unwrap_err();
        }
    }
}
