//! Centralised hashing utilities for the Lithos core library.
//!
//! Provides the [`Blake3Hash`] newtype wrapper around BLAKE3 hashes to ensure
//! type-safe hashing policy and efficient zero-copy storage.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
#[expect(
    clippy::module_name_repetitions,
    reason = "Blake3Hash is descriptive and clear"
)]
#[rkyv(derive(Debug))]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Creates a new hash from raw bytes.
    #[inline]
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Computes a hash directly from raw bytes.
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Computes a BLAKE3 hash using the value's hashing strategy.
    #[inline]
    #[must_use]
    pub fn compute<T: Blake3Hashable + ?Sized>(value: &T) -> Self {
        value.compute_hash()
    }

    /// Returns the underlying bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Checks if this hash matches another hash.
    #[inline]
    #[must_use]
    pub fn is_match(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl ArchivedBlake3Hash {
    /// Check if archived hash matches (for zero-copy staleness checks).
    #[inline]
    #[must_use]
    pub fn is_match(&self, hash: &Blake3Hash) -> bool {
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

/// Hash index for keyed values.
///
/// This wraps `HashMap<K, Blake3Hash>` so call sites can use a domain-specific
/// type with helper methods for change detection.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Blake3HashIndex<K: Eq + Hash>(HashMap<K, Blake3Hash>);

impl<K: Eq + Hash> Blake3HashIndex<K> {
    /// Creates a new hash index from an existing hash map.
    #[inline]
    #[must_use]
    pub const fn new(map: HashMap<K, Blake3Hash>) -> Self {
        Self(map)
    }

    /// Returns a reference to the inner hash map.
    #[inline]
    #[must_use]
    pub const fn as_inner(&self) -> &HashMap<K, Blake3Hash> {
        &self.0
    }

    /// Returns a mutable reference to the inner hash map.
    #[inline]
    #[must_use]
    pub fn as_inner_mut(&mut self) -> &mut HashMap<K, Blake3Hash> {
        &mut self.0
    }

    /// Consumes the index and returns the inner hash map.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> HashMap<K, Blake3Hash> {
        self.0
    }

    /// Creates an empty hash index.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Computes a hash index from a keyed map of hashable values.
    ///
    /// Each value is hashed via its [`Blake3Hashable`] implementation.
    #[inline]
    #[must_use]
    pub fn compute<V>(map: HashMap<K, V>) -> Self
    where
        V: Blake3Hashable,
    {
        Self(
            map.into_iter()
                .map(|(key, value)| (key, value.compute_hash()))
                .collect(),
        )
    }

    /// Returns the hash for `key`, if present.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&Blake3Hash> {
        self.0.get(key)
    }

    /// Inserts a key-hash pair into the index.
    #[inline]
    pub fn insert(&mut self, key: K, hash: Blake3Hash) -> Option<Blake3Hash> {
        self.0.insert(key, hash)
    }

    /// Returns `true` if the index contains `key`.
    #[inline]
    #[must_use]
    pub fn has(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    /// Returns `true` if the index contains `key`.
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    /// Returns the number of hashes in the index.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the index contains no hashes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over all keys.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    /// Returns an iterator over all key-hash pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Blake3Hash)> {
        self.0.iter()
    }
}

impl<K: Clone + Eq + Hash> Blake3HashIndex<K> {
    /// Returns keys that are new or whose hashes differ from `old`.
    #[inline]
    #[must_use]
    pub fn changed_keys(&self, old: &Self) -> HashSet<K> {
        let mut changed = HashSet::new();

        for (key, hash) in self.iter() {
            if old.get(key) != Some(hash) {
                changed.insert(key.clone());
            }
        }

        changed
    }

    /// Returns keys that exist in `old` but not in `self`.
    #[inline]
    #[must_use]
    pub fn removed_keys(&self, old: &Self) -> HashSet<K> {
        let mut removed = HashSet::new();

        for key in old.keys() {
            if !self.contains_key(key) {
                removed.insert(key.clone());
            }
        }

        removed
    }
}

impl<K: Eq + Hash> Default for Blake3HashIndex<K> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<K> ArchivedBlake3HashIndex<K>
where
    K: Archive + Eq + Hash,
    <K as Archive>::Archived: Eq + Hash,
{
    /// Returns the number of hashes in the archived index.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the archived index contains no hashes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether this archived index matches an owned index.
    ///
    /// The caller provides key comparison logic because archived key types are
    /// not always directly comparable with their owned source types.
    #[inline]
    #[must_use]
    pub fn is_match_by<F>(
        &self,
        index: &Blake3HashIndex<K>,
        mut keys_match: F,
    ) -> bool
    where
        F: FnMut(&<K as Archive>::Archived, &K) -> bool,
    {
        if self.len() != index.len() {
            return false;
        }

        for (archived_key, archived_hash) in self.0.iter() {
            let Some((_, hash)) =
                index.iter().find(|&(key, _)| keys_match(archived_key, key))
            else {
                return false;
            };

            if !archived_hash.is_match(hash) {
                return false;
            }
        }

        true
    }
}

impl<K, V> From<HashMap<K, V>> for Blake3HashIndex<K>
where
    K: Eq + Hash,
    V: Blake3Hashable,
{
    #[inline]
    fn from(map: HashMap<K, V>) -> Self {
        Self::compute(map)
    }
}

/// Type that can be hashed into a [`Blake3Hash`].
///
/// Implementors define their hashing strategy, such as raw byte hashing or
/// structured serialization.
pub trait Blake3Hashable {
    /// Computes a hash for this value.
    fn compute_hash(&self) -> Blake3Hash;
}

/// Wrapper that hashes values via JSON serialization.
#[derive(Debug, Clone, Copy)]
#[expect(
    clippy::module_name_repetitions,
    reason = "JsonHash is explicit about structured hashing strategy"
)]
pub struct JsonHash<T>(T);

impl<T> JsonHash<T> {
    /// Creates a JSON-hashed wrapper.
    #[inline]
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Blake3Hashable for JsonHash<T>
where
    T: serde::Serialize + std::fmt::Debug,
{
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        if let Ok(json) = serde_json::to_vec(&self.0) {
            Blake3Hash::from_bytes(&json)
        } else {
            Blake3Hash::from_bytes(format!("{:?}", &self.0).as_bytes())
        }
    }
}

impl Blake3Hashable for [u8] {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        Blake3Hash::from_bytes(self)
    }
}

impl Blake3Hashable for Vec<u8> {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        Blake3Hash::from_bytes(self.as_slice())
    }
}

impl Blake3Hashable for str {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        Blake3Hash::from_bytes(self.as_bytes())
    }
}

impl Blake3Hashable for String {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        Blake3Hash::from_bytes(self.as_bytes())
    }
}

impl<const N: usize> Blake3Hashable for [u8; N] {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        Blake3Hash::from_bytes(self.as_slice())
    }
}

impl Blake3Hashable for Blake3Hash {
    #[inline]
    fn compute_hash(&self) -> Blake3Hash {
        *self
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
            let hash = Blake3Hash::compute(data);
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
            let hash = Blake3Hash::compute(&JsonHash::new(&value));

            let json = serde_json::to_string(&value).unwrap();
            let expected = Blake3Hash::compute(json.as_bytes());

            assert_eq!(hash, expected, "JSON hash mismatch");
        }
    }

    mod index {
        use super::*;

        #[test]
        fn should_compute_index_from_hashable_values() {
            let mut map = std::collections::HashMap::new();
            map.insert("a".to_owned(), JsonHash::new(vec![1i32, 2i32]));
            map.insert("b".to_owned(), JsonHash::new(vec![3i32, 4i32]));

            let index = Blake3HashIndex::compute(map);

            assert_eq!(index.len(), 2);
            assert!(index.has(&"a".to_owned()));
            assert!(index.has(&"b".to_owned()));
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn is_match_should_return_true_when_identical() {
            let hash = Blake3Hash::compute(b"test");
            assert!(hash.is_match(&hash), "Identical hashes should match");
        }

        #[test]
        fn is_match_should_return_false_when_different() {
            let hash1 = Blake3Hash::compute(b"test1");
            let hash2 = Blake3Hash::compute(b"test2");
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
            let hash = Blake3Hash::compute(b"test");
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
            let hash1 = Blake3Hash::compute(b"test1");
            let hash2 = Blake3Hash::compute(b"test2");
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
            let hash = Blake3Hash::compute(b"test");
            let bytes: &[u8; 32] = hash.as_ref();
            assert_eq!(bytes, hash.as_bytes());
        }
    }
}
