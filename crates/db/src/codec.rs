//! rkyv serialization helpers for the [`Store`](crate::Store) persistence
//! layer.
//!
//! The central type is [`RkyvBytes<T>`] — a zero-copy, alignment-aware wrapper
//! around rkyv-archived byte slices. Domain types derive [`Archive`] and are
//! stored in redb tables via [`DbRkyvType<T>`], which implements
//! [`redb::Value`] and [`redb::Key`].
//!
//! # Quick Start
//!
//! ```rust
//! # use traces_db::{RkyvBytes, CodecError};
//! # use rkyv::{Archive, Serialize, Deserialize};
//! #
//! # #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
//! # #[rkyv(derive(Debug))]
//! # struct Note { id: u32, title: String }
//! #
//! let original = Note {
//!     id: 1,
//!     title: "hello".into(),
//! };
//!
//! let bytes = RkyvBytes::<Note>::encode(&original)?;
//! let decoded: Note = bytes.decode()?;
//! assert_eq!(decoded, original);
//!
//! bytes.with_archived(|a| assert_eq!(&a.title, "hello"))?;
//! # Ok::<(), CodecError>(())
//! ```
//!
//! # Architecture Constraints
//!
//! - **Strict Validation**: All deserialization paths use `rkyv::access` (which
//!   invokes bytecheck). `access_unchecked` is explicitly forbidden to prevent
//!   undefined behavior from corrupt database pages.
//! - **Alignment**: `rkyv` 0.8 requires 16-byte alignment. These helpers
//!   abstract the complexity of [`AlignedVec`] and pointer arithmetic away from
//!   domain code.

use std::{borrow::Cow, fmt, marker::PhantomData};

use redb::{Key, Value};
use rkyv::{
    Archive, Deserialize, Portable, Serialize, api::high::HighDeserializer,
    util::AlignedVec,
};

mod private {
    pub trait Sealed {}
}

use crate::error::CodecError;

/// Marker trait for types that can be serialized to rkyv bytes.
///
/// This trait is **automatically implemented** for any type that derives
/// [`Archive`] and [`Serialize`](rkyv::Serialize). You never need to
/// implement it manually.
///
/// It gates the [`RkyvBytes::encode`] method. See [`RkyvBytes`] for usage
/// examples.
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
///
/// This trait is **automatically implemented** for any type that derives
/// [`Archive`] plus [`Deserialize`](rkyv::Deserialize) and satisfies the
/// [`CheckBytes`](rkyv::bytecheck::CheckBytes) bound. It provides two
/// decoding strategies:
///
/// | Method | Allocation | When to use |
/// |---|---|---|
/// | [`decode_from_rkyv_bytes`](RkyvDecode::decode_from_rkyv_bytes) | Owned value | Short-lived reads, mutation, or returning across a borrow boundary |
/// | [`with_archived_rkyv_bytes`](RkyvDecode::with_archived_rkyv_bytes) | Zero-copy | Read-only field access on a hot path |
///
/// See [`RkyvBytes`] for the type-safe wrapper that is the recommended API.
pub trait RkyvDecode: private::Sealed + Archive + Sized {
    /// Decode bytes into an owned value.
    ///
    /// Allocates a new `Self`. Prefer
    /// [`with_archived_rkyv_bytes`](RkyvDecode::with_archived_rkyv_bytes)
    /// for read-only access.
    ///
    /// # Errors
    /// Returns [`CodecError`] if validation or deserialization fails.
    fn decode_from_rkyv_bytes(bytes: &[u8]) -> Result<Self, CodecError>;

    /// Access an archived value without materializing it.
    ///
    /// The closure receives a validated reference to the archived
    /// representation. No allocation occurs.
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
        with_archived_rkyv::<T, _, _>(bytes, |archived| {
            rkyv::deserialize::<T, rkyv::rancor::Error>(archived).map_err(
                |source| CodecError::RkyvDeserialize {
                    type_name: std::any::type_name::<T>(),
                    source,
                },
            )
        })?
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
///
/// `RkyvBytes` wraps an rkyv-archived byte slice with a static type tag.
/// The type parameter `T` ensures that encode and decode are type-safe:
/// you cannot decode bytes tagged as one type into another.
///
/// The lifetime `'a` distinguishes two modes:
/// * **Borrowed** (`'a` tied to a database guard) — zero-copy reads from a
///   table.
/// * **Owned** (`RkyvBytes<'static, T>`) — values built for insertion or cache
///   storage.
///
/// # Examples
///
/// ```rust
/// # use traces_db::{RkyvBytes, CodecError};
/// # use rkyv::{Archive, Serialize, Deserialize};
/// # #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
/// # #[rkyv(derive(Debug))]
/// # struct Point { x: f32, y: f32 }
/// #
/// let p = Point {
///     x: 1.0,
///     y: 2.0,
/// };
/// let bytes = RkyvBytes::<Point>::encode(&p)?;
///
/// // Borrowed view (simulating a database read).
/// let borrowed = RkyvBytes::<Point>::borrowed(bytes.as_bytes());
/// let decoded: Point = borrowed.decode()?;
/// assert_eq!(decoded, p);
/// # Ok::<(), CodecError>(())
/// ```
pub struct RkyvBytes<'a, T> {
    /// The underlying byte storage — borrowed when read from a table, owned
    /// when constructed for insertion.
    bytes: Cow<'a, [u8]>,
    /// Static type tag; never constructed at runtime.
    _ty: PhantomData<T>,
}

impl<'a, T> RkyvBytes<'a, T> {
    /// Wrap a borrowed byte slice as typed rkyv bytes.
    ///
    /// The returned value borrows from the input slice — typically a
    /// database page guard. No copying or allocation occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use traces_db::RkyvBytes;
    /// # use rkyv::{Archive, Serialize, Deserialize};
    /// # #[derive(Archive, Serialize, Deserialize)]
    /// # #[rkyv(derive(Debug))]
    /// # struct T;
    /// let raw: &[u8] = &[];
    /// let bytes: RkyvBytes<'_, T> = RkyvBytes::borrowed(raw);
    /// ```
    #[inline]
    #[must_use]
    pub const fn borrowed(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            _ty: PhantomData,
        }
    }

    /// Wrap an owned byte vector as typed rkyv bytes.
    ///
    /// The returned value has a `'static` lifetime, making it suitable for
    /// insertion into a database table or long-lived cache.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use traces_db::RkyvBytes;
    /// let raw = vec![0u8; 16];
    /// let bytes: RkyvBytes<'static, ()> = RkyvBytes::owned(raw);
    /// ```
    #[inline]
    #[must_use]
    pub fn owned(bytes: Vec<u8>) -> RkyvBytes<'static, T> {
        RkyvBytes {
            bytes: Cow::Owned(bytes),
            _ty: PhantomData,
        }
    }

    /// Borrow the raw stored bytes.
    ///
    /// The returned slice is the underlying rkyv archive. It is *not*
    /// validated — call [`decode`](RkyvBytes::decode) or
    /// [`with_archived`](RkyvBytes::with_archived) for typed access.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use traces_db::RkyvBytes;
    /// # use rkyv::{Archive, Serialize, Deserialize};
    /// # #[derive(Archive, Serialize, Deserialize)]
    /// # #[rkyv(derive(Debug))]
    /// # struct T(i32);
    /// let bytes = RkyvBytes::<T>::encode(&T(42)).unwrap();
    /// assert!(!bytes.as_bytes().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Decode bytes into an owned value.
    ///
    /// Validates the archive, then deserializes into an owned `T`. Use this
    /// when you need to return the value or mutate it. For read-only field
    /// access, prefer [`with_archived`](RkyvBytes::with_archived) to avoid
    /// allocation.
    ///
    /// # Errors
    /// Returns [`CodecError`] if the archived bytes are corrupt or
    /// deserialization fails.
    #[inline]
    pub fn decode(&self) -> Result<T, CodecError>
    where
        T: RkyvDecode,
    {
        T::decode_from_rkyv_bytes(self.as_bytes())
    }

    /// Access the archived value without materializing it.
    ///
    /// The closure receives a validated reference to the archived
    /// representation. No `T` is allocated — useful for reading a few fields
    /// on a hot path.
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
    /// Serialize a value to owned rkyv bytes.
    ///
    /// The returned [`RkyvBytes`] has a `'static` lifetime and can be
    /// inserted directly into a database table via [`DbRkyvType`].
    ///
    /// # Errors
    /// Returns [`CodecError`] if serialization fails (e.g. for unrepresentable
    /// enum values).
    #[inline]
    pub fn encode(value: &T) -> Result<Self, CodecError>
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

/// Zero-sized adapter that enables [`RkyvBytes<T>`] to be stored in redb tables
/// as either a [`Value`](redb::Value) or a [`Key`](redb::Key).
///
/// Use this type as the value or key type parameter in redb table definitions:
///
/// ```rust
/// # use redb::TableDefinition;
/// # use traces_db::{RkyvBytes, DbRkyvType};
/// # use rkyv::{Archive, Serialize, Deserialize};
/// # #[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// # #[rkyv(derive(Debug, PartialEq))]
/// # struct Note { id: u32, title: String }
/// const TABLE: TableDefinition<&str, DbRkyvType<Note>> =
///     TableDefinition::new("notes");
/// ```
///
/// The adapter never allocates — it wraps the raw bytes passed by redb into
/// [`RkyvBytes::borrowed`] and unwraps them back on write.
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
    /// Compare two archived keys by decoding them.
    ///
    /// # Panics
    /// Panics if either byte slice contains corrupt or non-archived data,
    /// because the `redb::Key` interface cannot return a `Result`.
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
        use crate::{CodecErrorKind, DbError};

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
        use crate::CodecErrorKind;

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
    }
}
