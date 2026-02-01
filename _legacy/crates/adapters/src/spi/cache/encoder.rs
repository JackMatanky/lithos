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
///
/// # Examples
///
/// ```rust
/// use lithos_adapters::spi::cache::encoder::{Codec, RkyvCodec};
///
/// let codec = RkyvCodec::default();
/// let key = "my_key".to_string();
///
/// // Encode key
/// let encoded_key =
///     <RkyvCodec as Codec<String, String>>::encode_key(&codec, &key).unwrap();
/// assert!(!encoded_key.is_empty());
///
/// // Decode key
/// let decoded_key: String =
///     <RkyvCodec as Codec<String, String>>::decode_key(&codec, &encoded_key)
///         .unwrap();
/// assert_eq!(key, decoded_key);
/// ```
pub trait Codec<K, V>: Send + Sync {
    /// The archived representation of the value for zero-copy access.
    type Archived: ?Sized;

    /// Provide zero-copy access to the archived value.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if access or validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lithos_adapters::spi::cache::encoder::{Codec, RkyvCodec};
    ///
    /// let codec = RkyvCodec::default();
    /// let value = "zero-copy".to_string();
    /// let encoded =
    ///     <RkyvCodec as Codec<String, String>>::encode_value(&codec, &value)
    ///         .unwrap();
    ///
    /// // Access archived representation
    /// let archived =
    ///     <RkyvCodec as Codec<String, String>>::access(&codec, &encoded).unwrap();
    /// assert_eq!(archived.as_str(), "zero-copy");
    /// ```
    fn access<'view>(
        &self,
        encoded: &'view [u8],
    ) -> Result<&'view Self::Archived, CacheError>;

    /// Decode a key from bytes retrieved from storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if decoding fails.
    fn decode_key(&self, encoded: &[u8]) -> Result<K, CacheError>;

    /// Decode a value from bytes retrieved from storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if decoding fails.
    fn decode_value(&self, encoded: &[u8]) -> Result<V, CacheError>;

    /// Encode a key into bytes for storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    fn encode_key(&self, key: &K) -> Result<Vec<u8>, CacheError>;

    /// Encode a key into a reusable buffer for storage.
    ///
    /// This method allows callers to reuse buffers across multiple encode
    /// operations, eliminating per-call allocations in hot paths.
    ///
    /// The buffer is cleared before encoding.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    #[inline]
    fn encode_key_into(
        &self,
        key: &K,
        buf: &mut Vec<u8>,
    ) -> Result<(), CacheError> {
        buf.clear();
        let encoded = self.encode_key(key)?;
        buf.extend_from_slice(&encoded);
        Ok(())
    }

    /// Encode a value into bytes for storage.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    fn encode_value(&self, value: &V) -> Result<Vec<u8>, CacheError>;

    /// Encode a value into a reusable buffer for storage.
    ///
    /// This method allows callers to reuse buffers across multiple encode
    /// operations, eliminating per-call allocations in hot paths.
    ///
    /// The buffer is cleared before encoding.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if encoding fails.
    #[inline]
    fn encode_value_into(
        &self,
        value: &V,
        buf: &mut Vec<u8>,
    ) -> Result<(), CacheError> {
        buf.clear();
        let encoded = self.encode_value(value)?;
        buf.extend_from_slice(&encoded);
        Ok(())
    }
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
        + rkyv::Archive
        + for<'ser> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<K>: rkyv::Deserialize<
            K,
            rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
        >,
    for<'validation> rkyv::Archived<K>: rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'validation, rkyv::rancor::Error>,
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
        // Validation with bytecheck. Zero-copy is only safe for aligned data.
        let alignment = std::mem::align_of::<rkyv::Archived<V>>();
        if encoded.as_ptr().align_offset(alignment) != 0 {
            return Err(CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: "Archived value is not properly aligned".into(),
            });
        }

        rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(encoded).map_err(
            |e| CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("Failed to access archived value: {e}").into(),
            },
        )
    }

    #[inline]
    fn decode_key(&self, encoded: &[u8]) -> Result<K, CacheError> {
        use rkyv::util::AlignedVec;

        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(encoded);

        let archived =
            rkyv::access::<rkyv::Archived<K>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to access archived key: {e}")
                        .into(),
                })?;

        rkyv::deserialize::<K, rkyv::rancor::Error>(archived).map_err(|e| {
            CacheError::SerializationError {
                type_name: std::any::type_name::<K>(),
                message: format!("Failed to deserialize key: {e}").into(),
            }
        })
    }

    #[inline]
    fn decode_value(&self, encoded: &[u8]) -> Result<V, CacheError> {
        use rkyv::util::AlignedVec;

        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(encoded);
        let archived = <Self as Codec<K, V>>::access(self, &aligned)?;

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
    fn encode_key_into(
        &self,
        key: &K,
        buf: &mut Vec<u8>,
    ) -> Result<(), CacheError> {
        buf.clear();
        let aligned_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(key)
            .map_err(|e| CacheError::SerializationError {
                type_name: std::any::type_name::<K>(),
                message: format!("Failed to serialize key: {e}").into(),
            })?;
        buf.extend_from_slice(&aligned_bytes);
        Ok(())
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

    #[inline]
    fn encode_value_into(
        &self,
        value: &V,
        buf: &mut Vec<u8>,
    ) -> Result<(), CacheError> {
        buf.clear();
        let aligned_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map_err(|e| CacheError::SerializationError {
                type_name: std::any::type_name::<V>(),
                message: format!("Failed to serialize value: {e}").into(),
            })?;
        buf.extend_from_slice(&aligned_bytes);
        Ok(())
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
            // GIVEN: an RkyvCodec and a complex metadata structure
            let codec: RkyvCodec = RkyvCodec;
            let original = TestMetadata {
                version: "1.0".to_owned(),
                count: 42,
            };
            let key = "test_key".to_owned();

            // WHEN: the key and value are encoded and then decoded
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

            let decoded_key: String =
                <RkyvCodec as Codec<String, TestMetadata>>::decode_key(
                    &codec, &key_bytes,
                )
                .expect("decode_key failed");

            // THEN: the decoded objects match the originals
            assert_eq!(decoded, original);
            assert_eq!(decoded_key, key);
            assert!(!key_bytes.is_empty());
        }
    }

    mod rkyv_codec {
        use super::*;

        #[test]
        fn provides_zero_copy_access() {
            // GIVEN: an encoded string value
            let codec: RkyvCodec = RkyvCodec;
            let original = "zero-copy data".to_owned();

            let encoded = <RkyvCodec as Codec<String, String>>::encode_value(
                &codec, &original,
            )
            .unwrap();

            // WHEN: the archived representation is accessed
            let archived =
                <RkyvCodec as Codec<String, String>>::access(&codec, &encoded)
                    .expect("access failed");

            // THEN: the archived data matches the original without full
            // deserialization
            assert_eq!(archived.as_str(), original.as_str());
        }

        #[test]
        fn returns_error_on_corrupted_bytes() {
            // GIVEN: a set of corrupted bytes that are not valid rkyv data
            let codec: RkyvCodec = RkyvCodec;
            let corrupted = vec![0xFF, 0xFF, 0xFF, 0xFF];

            // WHEN: attempting to decode the corrupted bytes
            let result: Result<String, CacheError> =
                <RkyvCodec as Codec<String, String>>::decode_value(
                    &codec, &corrupted,
                );

            // THEN: a SerializationError is returned
            assert!(result.is_err());

            assert!(matches!(
                result.unwrap_err(),
                CacheError::SerializationError { .. }
            ));
        }

        #[test]
        fn encode_into_reuses_buffer() {
            // GIVEN: an RkyvCodec and a reusable buffer
            let codec: RkyvCodec = RkyvCodec;
            let key = "first_key".to_owned();
            let value = "first_value".to_owned();
            let mut key_buf = Vec::new();
            let mut value_buf = Vec::new();

            // WHEN: encoding multiple times into the same buffers
            <RkyvCodec as Codec<String, String>>::encode_key_into(
                &codec,
                &key,
                &mut key_buf,
            )
            .unwrap();
            <RkyvCodec as Codec<String, String>>::encode_value_into(
                &codec,
                &value,
                &mut value_buf,
            )
            .unwrap();

            let first_key_bytes = key_buf.clone();
            let first_value_bytes = value_buf.clone();

            let key2 = "second_key".to_owned();
            let value2 = "second_value".to_owned();

            <RkyvCodec as Codec<String, String>>::encode_key_into(
                &codec,
                &key2,
                &mut key_buf,
            )
            .unwrap();
            <RkyvCodec as Codec<String, String>>::encode_value_into(
                &codec,
                &value2,
                &mut value_buf,
            )
            .unwrap();

            // THEN: both encodings produce valid results
            let decoded_key1: String =
                <RkyvCodec as Codec<String, String>>::decode_key(
                    &codec,
                    &first_key_bytes,
                )
                .unwrap();
            let decoded_value1: String =
                <RkyvCodec as Codec<String, String>>::decode_value(
                    &codec,
                    &first_value_bytes,
                )
                .unwrap();
            let decoded_key2: String =
                <RkyvCodec as Codec<String, String>>::decode_key(
                    &codec, &key_buf,
                )
                .unwrap();
            let decoded_value2: String =
                <RkyvCodec as Codec<String, String>>::decode_value(
                    &codec, &value_buf,
                )
                .unwrap();

            assert_eq!(decoded_key1, key);
            assert_eq!(decoded_value1, value);
            assert_eq!(decoded_key2, key2);
            assert_eq!(decoded_value2, value2);
        }
    }
}
