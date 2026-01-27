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
    /// Decode a value from bytes retrieved from storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if decoding fails.
    fn decode_value(&self, bytes: &[u8]) -> Result<V, CacheError>;

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
    #[inline]
    fn decode_value(&self, bytes: &[u8]) -> Result<V, CacheError> {
        use rkyv::util::AlignedVec;

        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);

        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| CacheError::SerializationError {
                    type_name: std::any::type_name::<V>(),
                    message: format!("Failed to access archived value: {e}")
                        .into(),
                })?;

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

/// No-op codec for in-memory caches.
///
/// This codec is used for caches like Moka where values are stored in memory
/// and don't require serialization.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct IdentityCodec;

impl<K, V> Codec<K, V> for IdentityCodec
where
    K: Clone,
    V: Clone,
{
    #[inline]
    fn decode_value(&self, _bytes: &[u8]) -> Result<V, CacheError> {
        // Identity codec doesn't deserialize - this should never be called
        Err(CacheError::BackendError {
            backend: "identity_codec",
            message: "IdentityCodec does not support value decoding".into(),
        })
    }

    #[inline]
    fn encode_key(&self, _key: &K) -> Result<Vec<u8>, CacheError> {
        // Identity codec doesn't serialize - this should never be called
        Err(CacheError::BackendError {
            backend: "identity_codec",
            message: "IdentityCodec does not support key encoding".into(),
        })
    }

    #[inline]
    fn encode_value(&self, _value: &V) -> Result<Vec<u8>, CacheError> {
        // Identity codec doesn't serialize - this should never be called
        Err(CacheError::BackendError {
            backend: "identity_codec",
            message: "IdentityCodec does not support value encoding".into(),
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
                <RkyvCodec as Codec<String, TestMetadata>>::decode_value(
                    &codec,
                    &value_bytes,
                )
                .expect("decode_value failed");

            assert_eq!(decoded, original);
            assert!(!key_bytes.is_empty());
        }
    }

    mod rkyv_codec {
        use super::*;

        #[test]
        fn returns_error_on_corrupted_bytes() {
            let codec: RkyvCodec = RkyvCodec;
            let corrupted_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];

            let result: Result<String, CacheError> =
                <RkyvCodec as Codec<String, String>>::decode_value(
                    &codec,
                    &corrupted_bytes,
                );

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                CacheError::SerializationError { .. }
            ));
        }
    }
}
