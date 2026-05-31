//! Keyed hash map for change detection.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use rkyv::{Archive, Deserialize, Serialize};

use super::content_hash::Blake3Hash;

/// Hash index for keyed values.
///
/// This wraps `HashMap<K, Blake3Hash>` so call sites can use a domain-specific
/// type with helper methods for change detection.
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

    /// Computes a hash index from keyed values.
    ///
    /// Values are converted into [`HashInput`] and hashed via
    /// [`Blake3Hash::compute`].
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

#[expect(dead_code, reason = "Diff helpers are not yet wired into all paths")]
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

    /// Checks whether this archived index matches an owned index.
    ///
    /// The caller provides key comparison logic because archived key types are
    /// not always directly comparable with their owned source types.
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

    mod index {
        use super::*;

        #[test]
        fn should_compute_index_from_hashable_values() {
            let mut map = std::collections::HashMap::new();
            map.insert("a".to_owned(), hash_structured(&vec![1i32, 2i32]));
            map.insert("b".to_owned(), hash_structured(&vec![3i32, 4i32]));

            let index = Blake3HashIndex::compute(map);

            assert_eq!(index.len(), 2);
            assert!(index.contains_key(&"a".to_owned()));
            assert!(index.contains_key(&"b".to_owned()));
        }
    }
}
