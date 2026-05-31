//! Content hashing primitives for the Lithos core library.
//!
//! Provides the [`Blake3Hash`] newtype, its archived variant, and the
//! [`HashInput`] strategy enum for flexible content hashing.
//!
//! # Exports
//!
//! * [`Blake3Hash`] — 32-byte BLAKE3 content hash (Copy, rkyv-serializable).
//! * [`Blake3Hash::compute`] — Hash from any [`HashInput`] strategy.
//! * [`HashInput`] — Enum selecting Bytes, Text, or Structured hashing.
//! * [`hash_structured`] — Convenience wrapper for serde-serializable values.
//!
//! # Invariants
//!
//! * All hashes use BLAKE3 exclusively — the algorithm is fixed at compile
//!   time.
//! * [`Blake3Hash`] is a 32-byte newtype with value semantics (`Copy`).
//! * Archived variants use element-wise comparison because rkyv archived types
//!   do not guarantee `PartialEq` across archive/owned boundaries.

use rkyv::{Archive, Deserialize, Serialize};

/// Trait for types that carry a content hash.
#[allow(dead_code, reason = "used through tests and future impls")]
pub(crate) trait HasContentHash {
    /// Returns a reference to the content hash.
    fn content_hash(&self) -> &Blake3Hash;

    /// Returns `true` if this type's content hash matches `hash`.
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content_hash().is_match(hash)
    }
}

/// Mutable extension of [`HasContentHash`].
#[allow(dead_code, reason = "used through tests and future impls")]
pub(crate) trait HasContentHashMut: HasContentHash {
    /// Sets the content hash.
    fn set_content_hash(&mut self, hash: Blake3Hash);
}

/// A 32-byte BLAKE3 hash with value semantics (`Copy`).
///
/// Newtype wrapper around `[u8; 32]` to provide type safety and
/// centralised hashing policy across the project.
///
/// This type uses BLAKE3 for its performance and cryptographic strength,
/// serving as the primary content-addressing and staleness detection
/// primitive in Lithos.
///
/// # Examples
///
/// ```rust,ignore
/// // crate-private — illustrative only
/// let data: &[u8] = b"hello world";
/// let hash = Blake3Hash::compute(data);
/// assert_eq!(hash.as_bytes().len(), 32);
/// ```
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
    /// Creates a hash directly from 32 raw bytes (test-only).
    ///
    /// Use [`from_bytes`](Self::from_bytes) or [`compute`](Self::compute)
    /// in production code to obtain a properly-computed hash.
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

    /// Computes a BLAKE3 hash from any [`HashInput`] strategy.
    ///
    /// Accepts any type that implements `Into<HashInput>`, including:
    /// * `&[u8]`, `Vec<u8>`, `&[u8; N]` — hashed as raw bytes
    /// * `&str`, `String` — hashed as UTF-8 text
    /// * [`hash_structured`] return values — serialized then hashed
    ///
    /// # Panics
    ///
    /// Does not panic. The input conversion is infallible.
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

impl HasContentHash for Blake3Hash {
    fn content_hash(&self) -> &Blake3Hash {
        self
    }
}

impl HasContentHashMut for Blake3Hash {
    fn set_content_hash(&mut self, hash: Blake3Hash) {
        self.0 = hash.0;
    }
}

impl ArchivedBlake3Hash {
    /// Check if archived hash matches an owned [`Blake3Hash`].
    ///
    /// Uses element-wise comparison because rkyv archived types do not
    /// derive `PartialEq` across the archive/owned boundary.
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

/// Input strategy for [`Blake3Hash::compute`].
///
/// Selects how raw data is fed into the BLAKE3 hasher.
/// See the `From` impls on this type for the full set of automatic conversions.
///
/// | Variant                               | Source               | Use case                       |
/// | ------------------------------------- | -------------------- | ------------------------------ |
/// | [`Bytes`](HashInput::Bytes)           | Raw byte vectors     | Binary data, serialised output |
/// | [`Text`](HashInput::Text)             | UTF-8 strings        | Frontmatter, note content      |
/// | [`Structured`](HashInput::Structured) | Pre-serialised bytes | [`hash_structured`] output     |
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

/// Wraps a byte slice as [`HashInput::Bytes`] (test-only helper).
///
/// Convenience wrapper that avoids spelling `HashInput::Bytes(bytes.to_vec())`
/// in test assertions.
#[inline]
#[must_use]
#[cfg(test)]
pub(crate) fn hash_bytes(bytes: &[u8]) -> HashInput {
    HashInput::Bytes(bytes.to_vec())
}

/// Creates a [`HashInput::Structured`] from a serializable value.
///
/// Attempts JSON serialization first via [`serde_json::to_vec`].
/// Falls back to [`Debug`] formatting when serialization fails.
///
/// # Type requirements
///
/// * `T: Serialize` — for the primary JSON path.
/// * `T: Debug` — for the serialization fallback.
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
        fn creates_from_raw_bytes() {
            let bytes = [1u8; 32];
            let hash = Blake3Hash::new(bytes);
            assert_eq!(hash.as_bytes(), &bytes, "Hash bytes mismatch");
        }

        #[test]
        fn creates_from_arbitrary_data() {
            let data = b"hello world";
            let hash = Blake3Hash::from_bytes(data);
            let expected = blake3::hash(data);
            assert_eq!(
                hash.as_bytes(),
                expected.as_bytes(),
                "from_bytes hash mismatch"
            );
        }
    }

    mod compute {
        use super::*;

        #[test]
        fn returns_correct_hash_from_bytes() {
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
        fn returns_correct_hash_from_structured() {
            let value = vec![1i32, 2i32, 3i32];
            let hash = Blake3Hash::compute(hash_structured(&value));

            let json = serde_json::to_string(&value).unwrap();
            let expected = Blake3Hash::compute(hash_bytes(json.as_bytes()));

            assert_eq!(hash, expected, "JSON hash mismatch");
        }

        #[test]
        fn returns_correct_hash_from_text() {
            let text = "hello world";
            let hash = Blake3Hash::compute(text);
            let expected = blake3::hash(text.as_bytes());
            assert_eq!(
                hash.as_bytes(),
                expected.as_bytes(),
                "Text hash mismatch"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn returns_true_when_hashes_match() {
            let hash = Blake3Hash::compute(hash_bytes(b"test"));
            assert!(hash.is_match(&hash), "Identical hashes should match");
        }

        #[test]
        fn returns_false_when_hashes_differ() {
            let hash1 = Blake3Hash::compute(hash_bytes(b"test1"));
            let hash2 = Blake3Hash::compute(hash_bytes(b"test2"));
            assert!(
                !hash1.is_match(&hash2),
                "Different hashes should not match"
            );
        }
    }

    mod has_content_hash {
        use super::*;

        #[test]
        fn returns_hash_from_self() {
            let hash = Blake3Hash::compute(hash_bytes(b"test data"));
            assert_eq!(hash.content_hash(), &hash);
        }

        #[test]
        fn is_content_match_returns_true_when_identical() {
            let hash = Blake3Hash::compute(hash_bytes(b"test data"));
            assert!(hash.is_content_match(&hash));
        }

        #[test]
        fn is_content_match_returns_false_when_different() {
            let hash1 = Blake3Hash::compute(hash_bytes(b"test data"));
            let hash2 = Blake3Hash::compute(hash_bytes(b"other data"));
            assert!(!hash1.is_content_match(&hash2));
        }
    }

    mod has_content_hash_mut {
        use super::*;

        #[test]
        fn set_content_hash_updates_hash() {
            let mut hash = Blake3Hash::compute(hash_bytes(b"original"));
            let new = Blake3Hash::compute(hash_bytes(b"updated"));
            hash.set_content_hash(new);
            assert_eq!(hash.content_hash(), &new);
        }

        #[test]
        fn set_content_hash_changes_match_behavior() {
            let mut hash = Blake3Hash::compute(hash_bytes(b"original"));
            let other = Blake3Hash::compute(hash_bytes(b"other"));
            let original_clone = hash;
            hash.set_content_hash(other);
            assert!(!hash.is_content_match(&original_clone));
            assert!(hash.is_content_match(&other));
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn returns_true_when_archived_matches_source() {
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
        fn returns_false_when_archived_matches_different_source() {
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
        fn converts_from_byte_array() {
            let bytes = [7u8; 32];
            let hash: Blake3Hash = bytes.into();
            assert_eq!(hash.as_bytes(), &bytes);
        }

        #[test]
        fn supports_as_ref_to_byte_array() {
            let hash = Blake3Hash::compute(hash_bytes(b"test"));
            let bytes: &[u8; 32] = hash.as_ref();
            assert_eq!(bytes, hash.as_bytes());
        }

        #[test]
        fn hash_input_from_byte_slice() {
            let data: &[u8] = &[1, 2, 3];
            let input: HashInput = data.into();
            assert!(matches!(input, HashInput::Bytes(v) if v == vec![1, 2, 3]));
        }

        #[test]
        fn hash_input_from_byte_array_ref() {
            let data: &[u8; 3] = &[4, 5, 6];
            let input: HashInput = data.into();
            assert!(matches!(input, HashInput::Bytes(v) if v == vec![4, 5, 6]));
        }

        #[test]
        fn hash_input_from_vec_ref() {
            let data = vec![7u8, 8, 9];
            let input: HashInput = (&data).into();
            assert!(matches!(input, HashInput::Bytes(v) if v == vec![7, 8, 9]));
        }

        #[test]
        fn hash_input_from_vec() {
            let data = vec![10u8, 11, 12];
            let input: HashInput = data.into();
            assert!(
                matches!(input, HashInput::Bytes(v) if v == vec![10, 11, 12])
            );
        }

        #[test]
        fn hash_input_from_str_ref() {
            let input: HashInput = "hello".into();
            assert!(matches!(input, HashInput::Text(v) if v == "hello"));
        }

        #[test]
        fn hash_input_from_string() {
            let s = "world".to_owned();
            let input: HashInput = s.into();
            assert!(matches!(input, HashInput::Text(v) if v == "world"));
        }

        #[test]
        fn hash_input_from_string_ref() {
            let s = "foo".to_owned();
            let input: HashInput = (&s).into();
            assert!(matches!(input, HashInput::Text(v) if v == "foo"));
        }
    }
}
