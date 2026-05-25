//! Schema index types for efficient lookups by name, ID, and path.
//!
//! This module provides the `SchemaIndex` struct which consolidates multiple
//! lookup patterns used throughout the schema system:
//! - Name → ID (for inheritance resolution)
//! - ID → Name (for reverse lookups)
//! - Path → ID (for discovery stage)
//!
//! # Design
//!
//! `SchemaIndex` is designed to be **derived, not persisted**. The data already
//! exists in the `SCHEMA_ID_BY_PATH` table. SchemaName is deterministically
//! derived from the path basename (which follows the same validation pattern).

use std::collections::HashMap;

use crate::{
    fs::PathKey,
    schema::{
        error::SchemaError,
        identifier::{SchemaId, SchemaName},
    },
};

/// An entry in the schema index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIndexEntry {
    /// Schema unique identifier.
    id: SchemaId,
    /// Schema validated name.
    name: SchemaName,
    /// Schema vault-relative path (optional if built from name/id only).
    path: Option<PathKey>,
}

impl SchemaIndexEntry {
    /// Creates a new `SchemaIndexEntry` with the given ID and name.
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, name: SchemaName) -> Self {
        Self {
            id,
            name,
            path: None,
        }
    }

    /// Sets the path for the entry.
    #[inline]
    #[must_use]
    pub fn with_path(mut self, path: PathKey) -> Self {
        self.path = Some(path);
        self
    }

    /// Returns the schema ID.
    #[inline]
    #[must_use]
    pub fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns a reference to the schema ID.
    #[inline]
    #[must_use]
    pub fn id_ref(&self) -> &SchemaId {
        &self.id
    }

    /// Returns the schema name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the schema path, if available.
    #[inline]
    #[must_use]
    pub fn path(&self) -> Option<&PathKey> {
        self.path.as_ref()
    }
}

/// Bidirectional index for schema lookups.
///
/// Provides efficient lookups by name, ID, and path. This type is designed
/// to be built on-demand from repository data rather than persisted separately.
#[derive(Debug, Clone, Default)]
pub struct SchemaIndex {
    entries: Vec<SchemaIndexEntry>,
    name_index: HashMap<SchemaName, usize>,
    id_index: HashMap<SchemaId, usize>,
    path_index: HashMap<PathKey, usize>,
}

impl SchemaIndex {
    /// Creates an empty `SchemaIndex`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `SchemaIndex` with the specified capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            name_index: HashMap::with_capacity(capacity),
            id_index: HashMap::with_capacity(capacity),
            path_index: HashMap::with_capacity(capacity),
        }
    }

    /// Shrinks the capacity of the index as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.name_index.shrink_to_fit();
        self.id_index.shrink_to_fit();
        self.path_index.shrink_to_fit();
    }

    /// Create index from name→ID pairs (e.g., from repository's list method).
    ///
    /// This method builds both name→ID and ID→name mappings.
    pub fn from_name_id_pairs<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (SchemaName, SchemaId)>,
    {
        let iter = pairs.into_iter();
        let (low, high) = iter.size_hint();
        let capacity = high.unwrap_or(low);

        let mut index = Self::with_capacity(capacity);

        for (name, id) in iter {
            index.insert_entry(SchemaIndexEntry::new(id, name));
        }

        index
    }

    /// Create a name-to-ID index only.
    ///
    /// This is a performance optimization for hot paths that only need
    /// forward resolution (e.g., inheritance).
    pub fn from_name_id_pairs_only<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (SchemaName, SchemaId)>,
    {
        let iter = pairs.into_iter();
        let (low, high) = iter.size_hint();
        let capacity = high.unwrap_or(low);

        let mut entries = Vec::with_capacity(capacity);
        let mut name_index = HashMap::with_capacity(capacity);

        for (name, id) in iter {
            let idx = entries.len();
            entries.push(SchemaIndexEntry::new(id, name.clone()));
            name_index.insert(name, idx);
        }

        Self {
            entries,
            name_index,
            id_index: HashMap::new(),
            path_index: HashMap::new(),
        }
    }

    /// Create index from path→ID pairs (e.g., from repository's list method).
    ///
    /// Derives `SchemaName` from path basename (file stem) for name→ID lookups.
    ///
    /// # Errors
    /// Returns `SchemaError` if a path cannot be converted to a valid schema
    /// name.
    pub fn from_path_id_pairs<I>(pairs: I) -> Result<Self, SchemaError>
    where
        I: IntoIterator<Item = (PathKey, SchemaId)>,
    {
        let iter = pairs.into_iter();
        let (low, high) = iter.size_hint();
        let capacity = high.unwrap_or(low);

        let mut index = Self::with_capacity(capacity);

        for (path, id) in iter {
            let name = SchemaName::try_from(&path)?;
            index.insert_entry(SchemaIndexEntry::new(id, name).with_path(path));
        }

        Ok(index)
    }

    /// Create index from both name→ID and path→ID pairs.
    ///
    /// # Errors
    /// Returns `SchemaError` if a path cannot be converted to a valid schema
    /// name.
    pub fn from_pairs<I, J>(
        name_pairs: I,
        path_pairs: J,
    ) -> Result<Self, SchemaError>
    where
        I: IntoIterator<Item = (SchemaName, SchemaId)>,
        J: IntoIterator<Item = (PathKey, SchemaId)>,
    {
        let name_iter = name_pairs.into_iter();
        let path_iter = path_pairs.into_iter();

        let (n_low, n_high) = name_iter.size_hint();
        let (p_low, p_high) = path_iter.size_hint();
        let capacity =
            n_high.unwrap_or(n_low).saturating_add(p_high.unwrap_or(p_low));

        let mut index = Self::with_capacity(capacity);

        // First, add all name-id pairs
        for (name, id) in name_iter {
            index.insert_entry(SchemaIndexEntry::new(id, name));
        }

        // Second, augment with paths
        for (path, id) in path_iter {
            if let Some(&idx) = index.id_index.get(&id) {
                if let Some(entry) = index.entries.get_mut(idx) {
                    index.path_index.insert(path.clone(), idx);
                    entry.path = Some(path);
                }
            } else {
                // New entry with path only (derive name)
                let name = SchemaName::try_from(&path)?;
                index.insert_entry(
                    SchemaIndexEntry::new(id, name).with_path(path),
                );
            }
        }

        Ok(index)
    }

    /// Get an entry by ID.
    #[inline]
    #[must_use]
    pub fn get_entry_by_id(&self, id: &SchemaId) -> Option<&SchemaIndexEntry> {
        let &idx = self.id_index.get(id)?;
        self.entries.get(idx)
    }

    /// Get an entry by name.
    #[inline]
    #[must_use]
    pub fn get_entry_by_name(
        &self,
        name: &SchemaName,
    ) -> Option<&SchemaIndexEntry> {
        let &idx = self.name_index.get(name)?;
        self.entries.get(idx)
    }

    /// Get an entry by path.
    #[inline]
    #[must_use]
    pub fn get_entry_by_path(
        &self,
        path: &PathKey,
    ) -> Option<&SchemaIndexEntry> {
        let &idx = self.path_index.get(path)?;
        self.entries.get(idx)
    }

    /// Get schema ID by name.
    #[inline]
    #[must_use]
    pub fn get_id_by_name(&self, name: &SchemaName) -> Option<SchemaId> {
        self.get_entry_by_name(name).map(SchemaIndexEntry::id)
    }

    /// Get schema name by ID.
    #[inline]
    #[must_use]
    pub fn get_name_by_id(&self, id: &SchemaId) -> Option<&SchemaName> {
        self.get_entry_by_id(id).map(SchemaIndexEntry::name)
    }

    /// Get schema ID by path.
    #[inline]
    #[must_use]
    pub fn get_id_by_path(&self, path: &PathKey) -> Option<SchemaId> {
        self.get_entry_by_path(path).map(SchemaIndexEntry::id)
    }

    /// Iterate over name→ID pairs.
    pub fn iter_name_id(
        &self,
    ) -> impl Iterator<Item = (&SchemaName, &SchemaId)> {
        self.entries.iter().map(|e| (e.name(), e.id_ref()))
    }

    /// Iterate over path→ID pairs.
    pub fn iter_path_id(&self) -> impl Iterator<Item = (&PathKey, &SchemaId)> {
        self.entries.iter().filter_map(|e| e.path().map(|p| (p, e.id_ref())))
    }

    /// Returns an iterator over all entries in the index.
    pub fn entries(&self) -> impl Iterator<Item = &SchemaIndexEntry> {
        self.entries.iter()
    }

    /// Get the number of schemas in the index.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the index is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Consume the index and return the inner entries.
    #[inline]
    #[must_use]
    pub fn into_entries(self) -> Vec<SchemaIndexEntry> {
        self.entries
    }

    /// Insert a name→ID mapping.
    pub fn insert_name(&mut self, name: SchemaName, id: SchemaId) {
        if let Some(&idx) = self.id_index.get(&id) {
            if let Some(entry) = self.entries.get_mut(idx) {
                // Remove old name from name_index
                self.name_index.remove(entry.name());
                entry.name = name.clone();
                self.name_index.insert(name, idx);
            }
        } else {
            self.insert_entry(SchemaIndexEntry::new(id, name));
        }
    }

    /// Insert a path→ID mapping.
    ///
    /// # Errors
    /// Returns `SchemaError` if the name cannot be derived from the path.
    pub fn insert_path(
        &mut self,
        path: PathKey,
        id: SchemaId,
    ) -> Result<(), SchemaError> {
        if let Some(&idx) = self.id_index.get(&id) {
            if let Some(entry) = self.entries.get_mut(idx) {
                if let Some(old_path) = entry.path.take() {
                    self.path_index.remove(&old_path);
                }
                entry.path = Some(path.clone());
                self.path_index.insert(path, idx);
            }
        } else {
            // Try to derive name if we don't know it yet
            let name = SchemaName::try_from(&path)?;
            self.insert_entry(SchemaIndexEntry::new(id, name).with_path(path));
        }
        Ok(())
    }

    /// Helper to insert a full entry and update all indices.
    fn insert_entry(&mut self, entry: SchemaIndexEntry) {
        let idx = self.entries.len();
        self.name_index.insert(entry.name().clone(), idx);
        self.id_index.insert(entry.id(), idx);
        if let Some(path) = entry.path() {
            self.path_index.insert(path.clone(), idx);
        }
        self.entries.push(entry);
    }
}

/// Collection of name→ID pairs for schema lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NameIdPairs(Vec<(SchemaName, SchemaId)>);

impl NameIdPairs {
    /// Creates a new empty `NameIdPairs`.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new empty `NameIdPairs` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Pushes a new pair into the collection.
    pub fn push(&mut self, pair: (SchemaName, SchemaId)) {
        self.0.push(pair);
    }

    /// Consumes the collection and returns the inner `Vec`.
    pub fn into_vec(self) -> Vec<(SchemaName, SchemaId)> {
        self.0
    }

    /// Returns an iterator over the pairs.
    pub fn iter(&self) -> impl Iterator<Item = &(SchemaName, SchemaId)> {
        self.0.iter()
    }

    /// Returns the number of pairs in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the first pair in the collection.
    pub fn first(&self) -> Option<&(SchemaName, SchemaId)> {
        self.0.first()
    }
}

impl From<Vec<(SchemaName, SchemaId)>> for NameIdPairs {
    fn from(vec: Vec<(SchemaName, SchemaId)>) -> Self {
        Self(vec)
    }
}

impl IntoIterator for NameIdPairs {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = (SchemaName, SchemaId);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(SchemaName, SchemaId)> for NameIdPairs {
    fn from_iter<I: IntoIterator<Item = (SchemaName, SchemaId)>>(
        iter: I,
    ) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Delegate to inner Vec for performance-sensitive defaults"
)]
impl Extend<(SchemaName, SchemaId)> for NameIdPairs {
    fn extend<I: IntoIterator<Item = (SchemaName, SchemaId)>>(
        &mut self,
        iter: I,
    ) {
        self.0.extend(iter);
    }
}

/// Collection of path→ID pairs for schema discovery.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PathIdPairs(Vec<(PathKey, SchemaId)>);

impl PathIdPairs {
    /// Creates a new empty `PathIdPairs`.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new empty `PathIdPairs` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Pushes a new pair into the collection.
    pub fn push(&mut self, pair: (PathKey, SchemaId)) {
        self.0.push(pair);
    }

    /// Consumes the collection and returns the inner `Vec`.
    pub fn into_vec(self) -> Vec<(PathKey, SchemaId)> {
        self.0
    }

    /// Returns an iterator over the pairs.
    pub fn iter(&self) -> impl Iterator<Item = &(PathKey, SchemaId)> {
        self.0.iter()
    }

    /// Returns the number of pairs in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the first pair in the collection.
    pub fn first(&self) -> Option<&(PathKey, SchemaId)> {
        self.0.first()
    }
}

impl FromIterator<(PathKey, SchemaId)> for PathIdPairs {
    fn from_iter<I: IntoIterator<Item = (PathKey, SchemaId)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Delegate to inner Vec for performance-sensitive defaults"
)]
impl Extend<(PathKey, SchemaId)> for PathIdPairs {
    fn extend<I: IntoIterator<Item = (PathKey, SchemaId)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl From<Vec<(PathKey, SchemaId)>> for PathIdPairs {
    fn from(vec: Vec<(PathKey, SchemaId)>) -> Self {
        Self(vec)
    }
}

impl IntoIterator for PathIdPairs {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = (PathKey, SchemaId);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_index_lookups_work() {
        let name1 = SchemaName::try_new("user").unwrap();
        let name2 = SchemaName::try_new("task").unwrap();
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();

        let index = SchemaIndex::from_pairs(
            [(name1.clone(), id1), (name2.clone(), id2)],
            [],
        )
        .unwrap();

        assert_eq!(index.get_id_by_name(&name1), Some(id1));
        assert_eq!(index.get_id_by_name(&name2), Some(id2));
        assert_eq!(index.get_name_by_id(&id1), Some(&name1));
        assert_eq!(index.get_name_by_id(&id2), Some(&name2));
    }

    #[test]
    fn from_path_id_pairs_derives_names() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();

        let index = SchemaIndex::from_path_id_pairs([
            (PathKey::try_new("schemas/user.json").unwrap(), id1),
            (PathKey::try_new("schemas/task.json").unwrap(), id2),
        ])
        .unwrap();

        let name_user = SchemaName::try_new("user").unwrap();
        let name_task = SchemaName::try_new("task").unwrap();

        assert_eq!(index.get_id_by_name(&name_user), Some(id1));
        assert_eq!(index.get_id_by_name(&name_task), Some(id2));
    }

    #[test]
    fn name_id_pairs_newtype() {
        let name1 = SchemaName::try_new("user").unwrap();
        let name2 = SchemaName::try_new("task").unwrap();
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();

        let mut pairs = NameIdPairs::new();
        pairs.push((name1.clone(), id1));
        pairs.push((name2.clone(), id2));

        assert_eq!(pairs.len(), 2);
        assert!(!pairs.is_empty());

        let vec: Vec<_> = pairs.into_vec();
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn path_id_pairs_newtype() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();

        let mut pairs = PathIdPairs::new();
        pairs.push((PathKey::try_new("schemas/user.json").unwrap(), id1));
        pairs.push((PathKey::try_new("schemas/task.json").unwrap(), id2));

        assert_eq!(pairs.len(), 2);
        assert!(!pairs.is_empty());

        let vec: Vec<_> = pairs.into_vec();
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn name_id_pairs_from_vec() {
        let name1 = SchemaName::try_new("user").unwrap();
        let id1 = SchemaId::new();

        let pairs: NameIdPairs = vec![(name1, id1)].into();
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn path_id_pairs_from_vec() {
        let id1 = SchemaId::new();

        let pairs: PathIdPairs =
            vec![(PathKey::try_new("schemas/user.json").unwrap(), id1)].into();
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn pairs_collect_works() {
        let name1 = SchemaName::try_new("user").unwrap();
        let id1 = SchemaId::new();

        let pairs: NameIdPairs = vec![(name1, id1)].into_iter().collect();
        assert_eq!(pairs.len(), 1);

        let path1 = PathKey::try_new("schemas/user.json").unwrap();
        let id2 = SchemaId::new();
        let p_pairs: PathIdPairs = vec![(path1, id2)].into_iter().collect();
        assert_eq!(p_pairs.len(), 1);
    }

    #[test]
    fn from_name_id_pairs_only_works() {
        let name1 = SchemaName::try_new("user").unwrap();
        let id1 = SchemaId::new();

        let index =
            SchemaIndex::from_name_id_pairs_only([(name1.clone(), id1)]);

        assert_eq!(index.get_id_by_name(&name1), Some(id1));
        assert_eq!(index.get_name_by_id(&id1), None);
    }
}
