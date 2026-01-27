//! Cache serialization abstraction layer.
//!
//! This module provides the `Codec` trait for abstracting serialization and
//! deserialization of cache keys and values across different storage backends.

use crate::spi::errors::CacheError;

/// Codec trait for cache key and value serialization/deserialization.
///
/// This abstraction allows different cache backends to use different
/// serialization strategies without leaking implementation details into
/// the public API.
pub trait Codec<K, V>: Send + Sync {
    /// The archived representation of the value for zero-copy access.
    type Archived: ?Sized;

    /// Provide zero-copy access to the archived value.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if access or validation fails.
    fn access<'view>(
        &self,
        encoded: &'view [u8],
    ) -> Result<&'view Self::Archived, CacheError>;

    /// Decode a value from bytes retrieved from storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if decoding fails.
    fn decode(&self, encoded: &[u8]) -> Result<V, CacheError>;

    /// Encode a key into bytes for storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    fn encode_key(&self, key: &K) -> Result<Vec<u8>, CacheError>;

    /// Encode a value into bytes for storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    fn encode_value(&self, value: &V) -> Result<Vec<u8>, CacheError>;
}

/// Zero-copy codec using `rkyv` for persistent storage.
///
/// This codec serializes values using `rkyv` and validates them on
/// deserialization, enabling zero-copy access to archived data.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct RkyvCodec;

impl<K, V> Codec<K, V> for RkyvCodec
where
    K: std::fmt::Debug
        + for<'ser> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
    V: rkyv::Archive,
    V: for<'ser> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<V>: rkyv::Deserialize<
            V,
            rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
        >,
    for<'validation> rkyv::Archived<V>: rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'validation, rkyv::rancor::Error>,
        >,
{
    type Archived = rkyv::Archived<V>;

    #[inline]
    fn access<'view>(
        &self,
        encoded: &'view [u8],
    ) -> Result<&'view Self::Archived, CacheError> {
        // Validation with bytecheck
        rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(encoded).map_err(
            |e| CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("Failed to access archived value: {e}").into(),
            },
        )
    }

    #[inline]
    fn decode(&self, encoded: &[u8]) -> Result<V, CacheError> {
        let archived = <Self as Codec<K, V>>::access(self, encoded)?;

        rkyv::deserialize::<V, rkyv::rancor::Error>(archived).map_err(|e| {
            CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("Failed to deserialize value: {e}").into(),
            }
        })
    }

    #[inline]
    fn encode_key(&self, key: &K) -> Result<Vec<u8>, CacheError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(key)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| CacheError::SerializationError {
                type_name: std::any::type_name::<K>(),
                message: format!("Failed to serialize key: {e}").into(),
            })
    }

    #[inline]
    fn encode_value(&self, value: &V) -> Result<Vec<u8>, CacheError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("Failed to serialize value: {e}").into(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod round_trip {
        use rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes};

        use super::*;

        #[derive(
            Archive,
            Serialize,
            Deserialize,
            CheckBytes,
            Debug,
            Clone,
            PartialEq,
            Eq,
        )]
        #[bytecheck(crate = rkyv::bytecheck)]
        struct TestMetadata {
            version: String,
            count: u32,
        }

        #[test]
        fn preserves_metadata_and_value() {
            let codec: RkyvCodec = RkyvCodec;
            let original = TestMetadata {
                version: "1.0".to_owned(),
                count: 42,
            };

            let key = "test_key".to_owned();
            let key_bytes =
                <RkyvCodec as Codec<String, TestMetadata>>::encode_key(
                    &codec, &key,
                )
                .expect("encode_key failed");
            let value_bytes =
                <RkyvCodec as Codec<String, TestMetadata>>::encode_value(
                    &codec, &original,
                )
                .expect("encode_value failed");

            let decoded: TestMetadata =
                <RkyvCodec as Codec<String, TestMetadata>>::decode(
                    &codec,
                    &value_bytes,
                )
                .expect("decode failed");

            assert_eq!(decoded, original);
            assert!(!key_bytes.is_empty());
        }
    }

    mod rkyv_codec {
        use super::*;

        #[test]
        fn provides_zero_copy_access() {
            let codec: RkyvCodec = RkyvCodec;
            let original = "zero-copy data".to_owned();

            let encoded = <RkyvCodec as Codec<String, String>>::encode_value(
                &codec, &original,
            )
            .unwrap();

            let archived =
                <RkyvCodec as Codec<String, String>>::access(&codec, &encoded)
                    .expect("access failed");

            assert_eq!(archived.as_str(), original.as_str());
        }

        #[test]
        fn returns_error_on_corrupted_bytes() {
            let codec: RkyvCodec = RkyvCodec;
            let corrupted = vec![0xFF, 0xFF, 0xFF, 0xFF];

            let result: Result<String, CacheError> =
                <RkyvCodec as Codec<String, String>>::decode(
                    &codec, &corrupted,
                );

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                CacheError::SerializationError { .. }
            ));
        }
    }
}
