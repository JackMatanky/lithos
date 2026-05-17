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

use rkyv::{
    Archive, Deserialize, Portable, Serialize, api::high::HighDeserializer,
    util::AlignedVec,
};

use crate::db::DbError;

/// A trait for types that can be safely serialized and deserialized to/from the
/// database.
///
/// This trait acts as a codec boundary, hiding the complex `rkyv` trait bounds
/// and validation logic from domain storage adapters. It provides both owned
/// (`from_bytes`) and zero-copy (`with_archived`) read paths.
pub trait ArchivedEntity: Sized {
    /// The zero-copy view of the entity.
    type View<'a>
    where
        Self: 'a;

    /// Serializes the entity to bytes, ensuring correct alignment.
    ///
    /// # Errors
    /// Returns [`DbError::Serialization`] if serialization fails.
    fn to_bytes(&self) -> Result<AlignedVec, DbError>;

    /// Deserializes the entity from bytes, validating them first.
    ///
    /// # Errors
    /// Returns [`DbError::Deserialization`] if validation or deserialization
    /// fails.
    fn from_bytes(bytes: &[u8]) -> Result<Self, DbError>;

    /// Accesses the zero-copy view of the entity from bytes.
    ///
    /// # Errors
    /// Returns [`DbError::Deserialization`] if validation fails.
    fn with_archived<R, F>(bytes: &[u8], f: F) -> Result<R, DbError>
    where
        F: FnOnce(Self::View<'_>) -> R;
}

// Blanket implementation for any type that derives the necessary rkyv traits.
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
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map_err(|e| DbError::Serialization(e.to_string()))
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, DbError> {
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);
        let archived =
            rkyv::access::<rkyv::Archived<T>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        rkyv::deserialize::<T, rkyv::rancor::Error>(archived)
            .map_err(|e| DbError::Deserialization(e.to_string()))
    }

    #[inline]
    fn with_archived<R, F>(bytes: &[u8], f: F) -> Result<R, DbError>
    where
        F: FnOnce(Self::View<'_>) -> R,
    {
        #[expect(
            clippy::as_conversions,
            reason = "Pointer to usize conversion required for alignment check"
        )]
        let ptr_usize = bytes.as_ptr() as usize;

        if ptr_usize.is_multiple_of(16) {
            let archived =
                rkyv::access::<rkyv::Archived<T>, rkyv::rancor::Error>(bytes)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
            Ok(f(archived))
        } else {
            let mut aligned = AlignedVec::<16>::new();
            aligned.extend_from_slice(bytes);
            let archived =
                rkyv::access::<rkyv::Archived<T>, rkyv::rancor::Error>(
                    &aligned,
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
            Ok(f(archived))
        }
    }
}

#[cfg(test)]
mod tests {
    use rkyv::{Archive, Serialize};

    use super::*;

    #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
    #[rkyv(derive(Debug, PartialEq))]
    struct TestData {
        id: u32,
        name: String,
    }

    mod to_bytes {
        use super::*;

        #[test]
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
            assert!(matches!(result.unwrap_err(), DbError::Deserialization(_)));
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
            assert!(matches!(result.unwrap_err(), DbError::Deserialization(_)));
        }
    }

    mod with_archived {
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
