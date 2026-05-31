//! Content hashing primitives for the Lithos core library.

use rkyv::{Archive, Deserialize, Serialize};

/// A 32-byte BLAKE3 hash.
///
/// Newtype wrapper around `[u8; 32]` to provide type safety and
/// centralised hashing policy across the project.
///
/// This type uses BLAKE3 for its performance and cryptographic strength,
/// serving as the primary content-addressing and staleness detection
/// primitive in Lithos.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub(crate) struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Creates a new hash from raw bytes.
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Computes a hash directly from raw bytes.
    #[inline]
    #[must_use]
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Computes a BLAKE3 hash using the provided input strategy.
    #[inline]
    #[must_use]
    pub(crate) fn compute<I>(input: I) -> Self
    where
        I: Into<HashInput>,
    {
        match input.into() {
            HashInput::Bytes(bytes) | HashInput::Structured(bytes) => {
                Self::from_bytes(&bytes)
            }
            HashInput::Text(text) => Self::from_bytes(text.as_bytes()),
        }
    }

    /// Returns the underlying bytes.
    #[inline]
    #[must_use]
    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Checks if this hash matches another hash.
    #[inline]
    #[must_use]
    pub(crate) fn is_match(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl ArchivedBlake3Hash {
    /// Check if archived hash matches (for zero-copy staleness checks).
    #[inline]
    #[must_use]
    pub(crate) fn is_match(&self, hash: &Blake3Hash) -> bool {
        self.0
            .iter()
            .zip(hash.as_bytes().iter())
            .all(|(left, right)| left == right)
    }
}

impl From<[u8; 32]> for Blake3Hash {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 32]> for Blake3Hash {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Hash input strategy for [`Blake3Hash::compute`].
pub(crate) enum HashInput {
    /// Hash raw bytes without transformation.
    Bytes(Vec<u8>),
    /// Hash text as UTF-8 bytes.
    Text(String),
    /// Hash structured data from pre-serialized bytes.
    Structured(Vec<u8>),
}

impl<'data> From<&'data [u8]> for HashInput {
    #[inline]
    fn from(value: &'data [u8]) -> Self {
        Self::Bytes(value.to_vec())
    }
}

impl<'data, const N: usize> From<&'data [u8; N]> for HashInput {
    #[inline]
    fn from(value: &'data [u8; N]) -> Self {
        Self::Bytes(value.to_vec())
    }
}

impl<'data> From<&'data Vec<u8>> for HashInput {
    #[inline]
    fn from(value: &'data Vec<u8>) -> Self {
        Self::Bytes(value.clone())
    }
}

impl<'data> From<&'data str> for HashInput {
    #[inline]
    fn from(value: &'data str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl<'data> From<&'data String> for HashInput {
    #[inline]
    fn from(value: &'data String) -> Self {
        Self::Text(value.clone())
    }
}

impl From<Vec<u8>> for HashInput {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<String> for HashInput {
    #[inline]
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

/// Creates a byte-hash input strategy.
#[inline]
#[must_use]
#[cfg(test)]
pub(crate) fn hash_bytes(bytes: &[u8]) -> HashInput {
    HashInput::Bytes(bytes.to_vec())
}

/// Creates a structured-hash input strategy.
///
/// Uses JSON serialization and falls back to debug formatting.
#[inline]
#[must_use]
pub(crate) fn hash_structured<T>(value: &T) -> HashInput
where
    T: serde::Serialize + std::fmt::Debug,
{
    if let Ok(json) = serde_json::to_vec(value) {
        HashInput::Structured(json)
    } else {
        HashInput::Structured(format!("{value:?}").into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn should_create_from_bytes() {
            let bytes = [1u8; 32];
            let hash = Blake3Hash::new(bytes);
            assert_eq!(hash.as_bytes(), &bytes, "Hash bytes mismatch");
        }
    }

    mod builders {
        use super::*;

        #[test]
        fn should_compute_from_data() {
            let data = b"hello world";
            let hash = Blake3Hash::compute(hash_bytes(data));
            let expected = blake3::hash(data);
            assert_eq!(
                hash.as_bytes(),
                expected.as_bytes(),
                "Computed hash mismatch"
            );
        }

        #[test]
        fn should_compute_from_hashable_value() {
            let value = vec![1i32, 2i32, 3i32];
            let hash = Blake3Hash::compute(hash_structured(&value));

            let json = serde_json::to_string(&value).unwrap();
            let expected = Blake3Hash::compute(hash_bytes(json.as_bytes()));

            assert_eq!(hash, expected, "JSON hash mismatch");
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn is_match_should_return_true_when_identical() {
            let hash = Blake3Hash::compute(hash_bytes(b"test"));
            assert!(hash.is_match(&hash), "Identical hashes should match");
        }

        #[test]
        fn is_match_should_return_false_when_different() {
            let hash1 = Blake3Hash::compute(hash_bytes(b"test1"));
            let hash2 = Blake3Hash::compute(hash_bytes(b"test2"));
            assert!(
                !hash1.is_match(&hash2),
                "Different hashes should not match"
            );
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn archived_is_match_should_return_true_when_identical() {
            let hash = Blake3Hash::compute(hash_bytes(b"test"));
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&hash)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedBlake3Hash, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived hash");

            assert!(
                archived.is_match(&hash),
                "Archived hash should match identical source"
            );
        }

        #[test]
        fn archived_is_match_should_return_false_when_different() {
            let hash1 = Blake3Hash::compute(hash_bytes(b"test1"));
            let hash2 = Blake3Hash::compute(hash_bytes(b"test2"));
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&hash1)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedBlake3Hash, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived hash");

            assert!(
                !archived.is_match(&hash2),
                "Archived hash should not match different source"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn should_convert_from_array() {
            let bytes = [7u8; 32];
            let hash: Blake3Hash = bytes.into();
            assert_eq!(hash.as_bytes(), &bytes);
        }

        #[test]
        fn should_support_as_ref_array() {
            let hash = Blake3Hash::compute(hash_bytes(b"test"));
            let bytes: &[u8; 32] = hash.as_ref();
            assert_eq!(bytes, hash.as_bytes());
        }
    }
}
