//! Keyed hash map for change detection.
//!
//! Provides [`Blake3HashIndex<K>`], a `HashMap<K, Blake3Hash>` wrapper that
//! tracks per-key hash values for efficient staleness and diff computation.
//!
//! # Exports
//!
//! * [`Blake3HashIndex<K>`] — Hash-indexed map (requires `K: Eq + Hash`).
//! * [`Blake3HashIndex::changed_keys`] — Keys added or modified since a prior
//!   snapshot.
//! * [`Blake3HashIndex::removed_keys`] — Keys present in a prior snapshot but
//!   absent now.
//!
//! # Invariants
//!
//! * Key type `K` must satisfy `Eq + Hash` (the standard `HashMap` bounds).
//! * Hash values are [`Blake3Hash`](crate::support::content_hash::Blake3Hash) —
//!   always 32 bytes.
//! * Empty and default indices are equivalent: `Blake3HashIndex::empty() ==
//!   Blake3HashIndex::default()`.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use rkyv::{Archive, Deserialize, Serialize};

use super::content_hash::Blake3Hash;

/// Hash-indexed map for per-key change detection.
///
/// Wraps `HashMap<K, Blake3Hash>` so call sites use a domain-specific
/// type with diff helpers instead of managing raw hash maps.
///
/// The key type `K` must satisfy the standard `HashMap` bounds (`Eq + Hash`).
///
/// # Examples
///
/// ```rust,ignore
/// // crate-private — illustrative only
/// let mut idx = Blake3HashIndex::default();
/// let value: &[u8] = b"value";
/// idx.insert("key".to_owned(), Blake3Hash::compute(value));
/// assert_eq!(idx.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub(crate) struct Blake3HashIndex<K: Eq + Hash>(HashMap<K, Blake3Hash>);

#[expect(dead_code, reason = "Crate-internal hash index API is staged")]
impl<K: Eq + Hash> Blake3HashIndex<K> {
    /// Creates a new hash index from an existing hash map.
    #[inline]
    #[must_use]
    pub(crate) const fn new(map: HashMap<K, Blake3Hash>) -> Self {
        Self(map)
    }

    /// Returns a reference to the inner hash map.
    #[inline]
    #[must_use]
    pub(crate) const fn as_inner(&self) -> &HashMap<K, Blake3Hash> {
        &self.0
    }

    /// Returns a mutable reference to the inner hash map.
    #[inline]
    #[must_use]
    pub(crate) fn as_inner_mut(&mut self) -> &mut HashMap<K, Blake3Hash> {
        &mut self.0
    }

    /// Consumes the index and returns the inner hash map.
    #[inline]
    #[must_use]
    pub(crate) fn into_inner(self) -> HashMap<K, Blake3Hash> {
        self.0
    }

    /// Creates an empty hash index.
    #[inline]
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Builds a full index by hashing every value in `map`.
    ///
    /// Each value is converted into
    /// [`HashInput`](super::content_hash::HashInput) and hashed via
    /// [`Blake3Hash::compute`](super::content_hash::Blake3Hash::compute).
    #[inline]
    #[must_use]
    pub(crate) fn compute<V>(map: HashMap<K, V>) -> Self
    where
        V: Into<super::content_hash::HashInput>,
    {
        Self(
            map.into_iter()
                .map(|(k, v)| (k, Blake3Hash::compute(v.into())))
                .collect(),
        )
    }

    /// Returns the hash for `key`, if present.
    #[inline]
    #[must_use]
    pub(crate) fn get(&self, key: &K) -> Option<&Blake3Hash> {
        self.0.get(key)
    }

    /// Inserts a key-hash pair into the index.
    #[inline]
    pub(crate) fn insert(
        &mut self,
        key: K,
        hash: Blake3Hash,
    ) -> Option<Blake3Hash> {
        self.0.insert(key, hash)
    }

    /// Returns `true` if the index contains `key`.
    #[inline]
    #[must_use]
    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    /// Returns the number of hashes in the index.
    #[inline]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the index contains no hashes.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over all keys.
    #[inline]
    pub(crate) fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    /// Returns an iterator over all key-hash pairs.
    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &Blake3Hash)> {
        self.0.iter()
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Diff helpers are staged for upcoming wiring")
)]
impl<K: Clone + Eq + Hash> Blake3HashIndex<K> {
    /// Returns keys that are new or whose hashes differ from `old`.
    #[inline]
    #[must_use]
    pub(crate) fn changed_keys(&self, old: &Self) -> HashSet<K> {
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
    pub(crate) fn removed_keys(&self, old: &Self) -> HashSet<K> {
        let mut removed = HashSet::new();

        for key in old.keys() {
            if !self.contains_key(key) {
                removed.insert(key.clone());
            }
        }

        removed
    }
}

// ---------------------------------------------------------------------------
// HasHashIndex / HasHashIndexMut traits
// ---------------------------------------------------------------------------

/// Trait for types that carry a [`Blake3HashIndex`].
#[allow(dead_code, reason = "used through tests and future impls")]
pub(crate) trait HasHashIndex {
    /// The key type of the hash index.
    type Key: Eq + Hash;

    /// Returns a reference to the underlying hash index.
    fn hash_index(&self) -> &Blake3HashIndex<Self::Key>;
}

/// Mutable extension of [`HasHashIndex`].
#[allow(dead_code, reason = "used through tests and future impls")]
pub(crate) trait HasHashIndexMut: HasHashIndex {
    /// Returns a mutable reference to the underlying hash index.
    fn hash_index_mut(&mut self) -> &mut Blake3HashIndex<Self::Key>;
}

// ---------------------------------------------------------------------------
// Trait impls for Blake3HashIndex<K>
// ---------------------------------------------------------------------------

#[allow(dead_code, reason = "used through tests and future impls")]
impl<K: Eq + Hash> HasHashIndex for Blake3HashIndex<K> {
    type Key = K;

    #[inline]
    fn hash_index(&self) -> &Blake3HashIndex<K> {
        self
    }
}

#[allow(dead_code, reason = "used through tests and future impls")]
impl<K: Eq + Hash> HasHashIndexMut for Blake3HashIndex<K> {
    #[inline]
    fn hash_index_mut(&mut self) -> &mut Blake3HashIndex<K> {
        self
    }
}

impl<K: Eq + Hash> Default for Blake3HashIndex<K> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

#[expect(dead_code, reason = "Archived index comparison is used selectively")]
impl<K> ArchivedBlake3HashIndex<K>
where
    K: Archive + Eq + Hash,
    <K as Archive>::Archived: Eq + Hash,
{
    /// Returns the number of hashes in the archived index.
    #[inline]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the archived index contains no hashes.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether this archived index matches an owned [`Blake3HashIndex`].
    ///
    /// The caller provides key comparison logic because rkyv archived key types
    /// are not always directly comparable with their owned source types
    /// (e.g. `ArchivedString` vs `String`).
    #[inline]
    #[must_use]
    pub(crate) fn is_match_by<F>(
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

impl<K: Eq + Hash> From<HashMap<K, Blake3Hash>> for Blake3HashIndex<K> {
    #[inline]
    fn from(map: HashMap<K, Blake3Hash>) -> Self {
        Self::new(map)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::content_hash::hash_structured, *};

    fn make_hash(k: u8) -> Blake3Hash {
        Blake3Hash::compute(vec![k])
    }

    fn populated_index() -> Blake3HashIndex<String> {
        let mut map = HashMap::new();
        map.insert("a".to_owned(), make_hash(1));
        map.insert("b".to_owned(), make_hash(2));
        Blake3HashIndex::new(map)
    }

    mod constructor {
        use super::*;

        #[test]
        fn empty_creates_empty_index() {
            let index: Blake3HashIndex<String> = Blake3HashIndex::empty();
            assert!(index.is_empty());
            assert_eq!(index.len(), 0);
        }

        #[test]
        fn new_creates_index_from_map() {
            let index = populated_index();
            assert_eq!(index.len(), 2);
        }

        #[test]
        fn default_creates_empty_index() {
            let index: Blake3HashIndex<String> = Blake3HashIndex::default();
            assert!(index.is_empty());
        }

        #[test]
        fn from_hashmap_creates_index() {
            let mut map = HashMap::new();
            map.insert("x".to_owned(), make_hash(9));
            let index = Blake3HashIndex::from(map);
            assert_eq!(index.len(), 1);
            assert!(index.contains_key(&"x".to_owned()));
        }

        #[test]
        fn compute_returns_populated_index() {
            let mut map = HashMap::new();
            map.insert("a".to_owned(), hash_structured(&vec![1i32, 2i32]));
            map.insert("b".to_owned(), hash_structured(&vec![3i32, 4i32]));

            let index = Blake3HashIndex::compute(map);

            assert_eq!(index.len(), 2);
            assert!(index.contains_key(&"a".to_owned()));
            assert!(index.contains_key(&"b".to_owned()));
        }
    }

    mod lookup {
        use super::*;

        #[test]
        fn get_returns_hash_when_key_exists() {
            let index = populated_index();
            assert_eq!(index.get(&"a".to_owned()), Some(&make_hash(1)));
        }

        #[test]
        fn get_returns_none_when_key_missing() {
            let index = populated_index();
            assert_eq!(index.get(&"missing".to_owned()), None);
        }

        #[test]
        fn contains_key_returns_true_when_key_exists() {
            let index = populated_index();
            assert!(index.contains_key(&"a".to_owned()));
        }

        #[test]
        fn contains_key_returns_false_when_key_missing() {
            let index = populated_index();
            assert!(!index.contains_key(&"missing".to_owned()));
        }

        #[test]
        fn len_returns_correct_count() {
            let index = populated_index();
            assert_eq!(index.len(), 2);
        }

        #[test]
        fn len_returns_zero_for_empty_index() {
            let index: Blake3HashIndex<String> = Blake3HashIndex::empty();
            assert_eq!(index.len(), 0);
        }

        #[test]
        fn is_empty_returns_true_when_empty() {
            let index: Blake3HashIndex<String> = Blake3HashIndex::empty();
            assert!(index.is_empty());
        }

        #[test]
        fn is_empty_returns_false_when_populated() {
            let index = populated_index();
            assert!(!index.is_empty());
        }
    }

    mod update {
        use super::*;

        #[test]
        fn insert_adds_key_and_returns_none_for_new_key() {
            let mut index = Blake3HashIndex::empty();
            let result = index.insert("k".to_owned(), make_hash(7));
            assert_eq!(result, None);
            assert_eq!(index.get(&"k".to_owned()), Some(&make_hash(7)));
        }

        #[test]
        fn insert_replaces_existing_hash() {
            let mut index = populated_index();
            let new_hash = make_hash(99);
            let old = index.insert("a".to_owned(), new_hash);
            assert_eq!(old, Some(make_hash(1)));
            assert_eq!(index.get(&"a".to_owned()), Some(&new_hash));
        }
    }

    mod diff {
        use super::*;

        #[test]
        fn changed_keys_returns_empty_when_identical() {
            let index = populated_index();
            let changed = index.changed_keys(&index);
            assert!(
                changed.is_empty(),
                "Identical indices should have no changes"
            );
        }

        #[test]
        fn changed_keys_detects_new_keys() {
            let old = populated_index();
            let mut new = populated_index();
            new.insert("c".to_owned(), make_hash(3));

            let changed = new.changed_keys(&old);
            assert!(changed.contains("c"), "New key should be in changed set");
        }

        #[test]
        fn changed_keys_detects_modified_values() {
            let old = populated_index();
            let mut new = populated_index();
            new.insert("a".to_owned(), make_hash(99));

            let changed = new.changed_keys(&old);
            assert!(
                changed.contains("a"),
                "Modified key should be in changed set"
            );
            assert_eq!(changed.len(), 1, "Only 'a' should be changed");
        }

        #[test]
        fn changed_keys_does_not_include_keys_removed_from_self() {
            let old = populated_index();
            let mut new = Blake3HashIndex::empty();
            new.insert("c".to_owned(), make_hash(3));

            let changed = new.changed_keys(&old);
            assert!(
                !changed.contains("a"),
                "Key removed from self should not appear in changed_keys"
            );
            assert!(!changed.contains("b"));
        }

        #[test]
        fn changed_keys_returns_empty_when_both_empty() {
            let a: Blake3HashIndex<String> = Blake3HashIndex::empty();
            let b: Blake3HashIndex<String> = Blake3HashIndex::empty();
            assert!(a.changed_keys(&b).is_empty());
        }

        #[test]
        fn removed_keys_returns_keys_in_old_not_in_self() {
            let old = populated_index();
            let mut new = populated_index();
            new.insert("c".to_owned(), make_hash(3));
            // Remove "a" by rebuilding without it
            let mut map = HashMap::new();
            map.insert("b".to_owned(), make_hash(2));
            map.insert("c".to_owned(), make_hash(3));
            new = Blake3HashIndex::new(map);

            let removed = new.removed_keys(&old);
            assert!(
                removed.contains("a"),
                "Key in old but not in new should be removed"
            );
            assert_eq!(removed.len(), 1);
        }

        #[test]
        fn removed_keys_returns_empty_when_superset() {
            let old = populated_index();
            let mut new = populated_index();
            new.insert("c".to_owned(), make_hash(3));

            let removed = new.removed_keys(&old);
            assert!(removed.is_empty(), "Superset should have no removed keys");
        }

        #[test]
        fn removed_keys_returns_empty_when_both_empty() {
            let a: Blake3HashIndex<String> = Blake3HashIndex::empty();
            let b: Blake3HashIndex<String> = Blake3HashIndex::empty();
            assert!(a.removed_keys(&b).is_empty());
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn archived_is_match_by_returns_true_when_identical() {
            let index = populated_index();
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&index)
                .expect("Failed to serialize");
            let archived: &ArchivedBlake3HashIndex<String> =
                rkyv::access::<
                    ArchivedBlake3HashIndex<String>,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("Failed to access archived index");

            let result = archived
                .is_match_by(&index, |archived_key, key| archived_key == key);
            assert!(result, "Archived index should match identical source");
        }

        #[test]
        fn archived_is_match_by_returns_false_when_different_value() {
            let index = populated_index();
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&index)
                .expect("Failed to serialize");
            let archived: &ArchivedBlake3HashIndex<String> =
                rkyv::access::<
                    ArchivedBlake3HashIndex<String>,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("Failed to access archived index");

            let mut modified = populated_index();
            modified.insert("a".to_owned(), make_hash(99));

            let result = archived
                .is_match_by(&modified, |archived_key, key| {
                    archived_key == key
                });
            assert!(!result, "Archived index should not match modified source");
        }

        #[test]
        fn archived_is_match_by_returns_false_when_different_length() {
            let index = populated_index();
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&index)
                .expect("Failed to serialize");
            let archived: &ArchivedBlake3HashIndex<String> =
                rkyv::access::<
                    ArchivedBlake3HashIndex<String>,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("Failed to access archived index");

            let bigger = {
                let mut map = HashMap::new();
                map.insert("a".to_owned(), make_hash(1));
                map.insert("b".to_owned(), make_hash(2));
                map.insert("c".to_owned(), make_hash(3));
                Blake3HashIndex::new(map)
            };

            let result = archived
                .is_match_by(&bigger, |archived_key, key| archived_key == key);
            assert!(
                !result,
                "Archived index should not match when lengths differ"
            );
        }
    }

    mod has_hash_index {
        use super::*;

        #[test]
        fn returns_self_for_blake3_hash_index() {
            let index = populated_index();
            let hash_ref: &Blake3HashIndex<String> = index.hash_index();
            assert_eq!(
                std::ptr::from_ref(hash_ref),
                std::ptr::from_ref(&index),
                "hash_index should return &self"
            );
        }

        #[test]
        fn provides_read_access_via_hash_index() {
            let index = populated_index();
            let hash_ref = index.hash_index();
            assert_eq!(hash_ref.len(), 2);
            assert!(hash_ref.contains_key(&"a".to_owned()));
        }

        #[test]
        fn returns_empty_index_when_empty() {
            let index: Blake3HashIndex<String> = Blake3HashIndex::empty();
            let hash_ref = index.hash_index();
            assert!(hash_ref.is_empty());
        }
    }

    mod has_hash_index_mut {
        use super::*;

        #[test]
        fn returns_mut_self_for_blake3_hash_index() {
            let mut index = Blake3HashIndex::<String>::empty();
            let hash_ref: &mut Blake3HashIndex<String> = index.hash_index_mut();
            assert_eq!(
                std::ptr::from_mut(hash_ref),
                std::ptr::from_mut(&mut index),
                "hash_index_mut should return &mut self"
            );
        }

        #[test]
        fn provides_write_access_via_hash_index_mut() {
            let mut index = populated_index();
            {
                let hash_ref = index.hash_index_mut();
                hash_ref.insert("new".to_owned(), make_hash(99));
            }
            assert_eq!(index.len(), 3);
            assert!(index.contains_key(&"new".to_owned()));
        }

        #[test]
        fn returns_consistent_behavior_after_mut_write() {
            let mut index = populated_index();
            let hash = make_hash(99);
            index.hash_index_mut().insert("new".to_owned(), hash);

            let read = index.hash_index();
            assert_eq!(read.get(&"new".to_owned()), Some(&make_hash(99)));
        }
    }
}
