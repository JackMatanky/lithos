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

#![allow(
    dead_code,
    clippy::as_conversions,
    reason = "Functions for issue-02 storage adapter use"
)]

use rkyv::{
    Archive, Deserialize, Portable, Serialize, api::high::HighDeserializer,
    util::AlignedVec,
};

use crate::db::DbError;

/// Serialize a value to rkyv-aligned bytes.
///
/// Ensures the resulting buffer uses `AlignedVec` to meet `rkyv`'s 16-byte
/// alignment requirements for zero-copy access later.
///
/// # Errors
///
/// Returns [`DbError::Serialization`] if `rkyv` fails to serialize the value.
pub(crate) fn serialize<V>(value: &V) -> Result<AlignedVec, DbError>
where
    V: Archive
        + for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
{
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map_err(|e| DbError::Serialization(e.to_string()))
}

/// Deserialize rkyv bytes with validation (copies data).
///
/// Converts a raw byte slice into an owned domain type. This enables the
/// **Two-Phase Iteration Pattern** required by `redb` to prevent lifetime
/// conflicts:
/// 1. Collect raw bytes from `redb::AccessGuard` items into memory.
/// 2. Deserialize those bytes into owned instances.
///
/// # Errors
///
/// Returns [`DbError::Deserialization`] if byte validation or deserialization
/// fails.
pub(crate) fn deserialize<V>(bytes: &[u8]) -> Result<V, DbError>
where
    V: Archive,
    V::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + Deserialize<V, HighDeserializer<rkyv::rancor::Error>>,
{
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    let archived =
        rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
    rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
        .map_err(|e| DbError::Deserialization(e.to_string()))
}

/// Access archived data via zero-copy closure.
///
/// Handles alignment automatically without exposing pointer arithmetic to
/// callers:
/// - **Fast path**: Direct, allocation-free access if the input slice happens
///   to fall on a 16-byte boundary (common for `redb` pages).
/// - **Slow path**: Allocates and copies to an `AlignedVec` before access if
///   the slice is unaligned.
///
/// # Errors
///
/// Returns [`DbError::Deserialization`] if byte validation fails.
pub(crate) fn with_archived<V, F, R>(bytes: &[u8], f: F) -> Result<R, DbError>
where
    V: Archive,
    V::Archived: Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        >,
    F: FnOnce(&V::Archived) -> R,
{
    let ptr_usize = bytes.as_ptr() as usize;

    if ptr_usize.is_multiple_of(16) {
        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(bytes)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        Ok(f(archived))
    } else {
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);
        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        Ok(f(archived))
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

    #[test]
    fn serialize_produces_bytes() {
        let data = TestData {
            id: 42,
            name: "test".to_owned(),
        };
        let result = serialize(&data);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let original = TestData {
            id: 123,
            name: "hello".to_owned(),
        };
        let bytes = serialize(&original).unwrap();
        let deserialized: TestData = deserialize(&bytes).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn with_archived_zero_copy() {
        let original = TestData {
            id: 999,
            name: "zero-copy".to_owned(),
        };
        let bytes = serialize(&original).unwrap();
        let result = with_archived::<TestData, _, _>(&bytes, |archived| {
            assert_eq!(archived.id, 999);
            archived.name.as_str().to_owned()
        });
        assert_eq!(result.unwrap(), "zero-copy");
    }

    #[test]
    fn deserialize_returns_deserialization_error_for_invalid_bytes() {
        let invalid_bytes = &[0u8, 1, 2, 3];
        let result: Result<TestData, DbError> = deserialize(invalid_bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::Deserialization(_)));
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        clippy::integer_division,
        clippy::integer_division_remainder_used,
        reason = "Test intentionally truncates bytes to verify error handling"
    )]
    fn deserialize_validates_and_returns_error_for_truncated_bytes() {
        let original = TestData {
            id: 1,
            name: "test".to_owned(),
        };
        let bytes = serialize(&original).unwrap();
        let truncated = &bytes[..bytes.len() / 2];
        let result: Result<TestData, DbError> = deserialize(truncated);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::Deserialization(_)));
    }

    #[test]
    fn with_archived_alignment_fast_path_aligned() {
        let original = TestData {
            id: 1,
            name: "aligned".to_owned(),
        };
        let bytes = serialize(&original).unwrap();
        let ptr = bytes.as_ptr() as usize;
        if ptr.is_multiple_of(16) {
            let result =
                with_archived::<TestData, _, _>(&bytes, |archived| archived.id);
            assert_eq!(result.unwrap(), 1);
        }
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test intentionally uses unaligned slice to verify error \
                  handling"
    )]
    fn with_archived_alignment_slow_path_returns_error_for_invalid_unaligned_data()
     {
        let original = TestData {
            id: 2,
            name: "unaligned".to_owned(),
        };
        let bytes = serialize(&original).unwrap();
        let unaligned: Vec<u8> = bytes.iter().copied().collect();
        let result =
            with_archived::<TestData, _, _>(&unaligned[1..], |archived| {
                archived.id
            });
        result.unwrap_err();
    }
}
