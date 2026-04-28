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

use std::{collections::HashMap, ops::Deref};

use crate::{
    fs::RelativePath,
    schema::identifier::{SchemaId, SchemaName},
};

/// Bidirectional index for schema lookups.
///
/// Provides efficient lookups by name, ID, and path. This type is designed
/// to be built on-demand from repository data rather than persisted separately.
///
/// # Construction
///
/// ```ignore
/// let index = SchemaIndex::from_path_id_pairs(repo.list_schema_path_id_pairs()?)?;
/// ```
///
/// Or for testing:
/// ```ignore
/// let index = SchemaIndex::from_pairs([
///     (SchemaName::try_new("user")?, id1),
///     (SchemaName::try_new("task")?, id2),
/// ]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SchemaIndex {
    name_to_id: HashMap<SchemaName, SchemaId>,
    id_to_name: HashMap<SchemaId, SchemaName>,
    path_to_id: HashMap<RelativePath, SchemaId>,
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
            name_to_id: HashMap::with_capacity(capacity),
            id_to_name: HashMap::with_capacity(capacity),
            path_to_id: HashMap::with_capacity(capacity),
        }
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

        let mut name_to_id = HashMap::with_capacity(capacity);
        let mut id_to_name = HashMap::with_capacity(capacity);

        for (name, id) in iter {
            id_to_name.insert(id, name.clone());
            name_to_id.insert(name, id);
        }

        Self {
            name_to_id,
            id_to_name,
            path_to_id: HashMap::new(),
        }
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

        let mut name_to_id = HashMap::with_capacity(capacity);

        for (name, id) in iter {
            name_to_id.insert(name, id);
        }

        Self {
            name_to_id,
            id_to_name: HashMap::new(),
            path_to_id: HashMap::new(),
        }
    }

    /// Create index from path→ID pairs (e.g., from repository's list method).
    ///
    /// Derives `SchemaName` from path basename (file stem) for name→ID lookups.
    pub fn from_path_id_pairs<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (RelativePath, SchemaId)>,
    {
        let iter = pairs.into_iter();
        let (low, high) = iter.size_hint();
        let capacity = high.unwrap_or(low);

        let mut path_to_id = HashMap::with_capacity(capacity);
        let mut name_to_id = HashMap::with_capacity(capacity);
        let mut id_to_name = HashMap::with_capacity(capacity);

        for (path, id) in iter {
            if let Ok(name) = SchemaName::try_from(path.clone()) {
                id_to_name.insert(id, name.clone());
                name_to_id.insert(name, id);
            }
            path_to_id.insert(path, id);
        }

        Self {
            name_to_id,
            id_to_name,
            path_to_id,
        }
    }

    /// Create index from both name→ID and path→ID pairs.
    pub fn from_pairs<I, J>(name_pairs: I, path_pairs: J) -> Self
    where
        I: IntoIterator<Item = (SchemaName, SchemaId)>,
        J: IntoIterator<Item = (RelativePath, SchemaId)>,
    {
        let name_iter = name_pairs.into_iter();
        let (n_low, n_high) = name_iter.size_hint();
        let name_capacity = n_high.unwrap_or(n_low);

        let mut name_to_id = HashMap::with_capacity(name_capacity);
        let mut id_to_name = HashMap::with_capacity(name_capacity);

        for (name, id) in name_iter {
            id_to_name.insert(id, name.clone());
            name_to_id.insert(name, id);
        }

        let path_iter = path_pairs.into_iter();
        let (p_low, p_high) = path_iter.size_hint();
        let path_capacity = p_high.unwrap_or(p_low);

        let mut path_to_id = HashMap::with_capacity(path_capacity);
        for (path, id) in path_iter {
            path_to_id.insert(path, id);
        }

        Self {
            name_to_id,
            id_to_name,
            path_to_id,
        }
    }

    /// Get schema ID by name.
    #[inline]
    #[must_use]
    pub fn get_id_by_name(&self, name: &SchemaName) -> Option<SchemaId> {
        self.name_to_id.get(name).copied()
    }

    /// Get schema name by ID.
    #[inline]
    #[must_use]
    pub fn get_name_by_id(&self, id: &SchemaId) -> Option<&SchemaName> {
        self.id_to_name.get(id)
    }

    /// Get schema ID by path.
    #[inline]
    #[must_use]
    pub fn get_id_by_path(&self, path: &RelativePath) -> Option<SchemaId> {
        self.path_to_id.get(path).copied()
    }

    /// Iterate over name→ID pairs.
    pub fn iter_name_id(
        &self,
    ) -> impl Iterator<Item = (&SchemaName, &SchemaId)> {
        self.name_to_id.iter()
    }

    /// Iterate over path→ID pairs.
    pub fn iter_path_id(
        &self,
    ) -> impl Iterator<Item = (&RelativePath, &SchemaId)> {
        self.path_to_id.iter()
    }

    /// Get the number of schemas in the index.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.name_to_id.len()
    }

    /// Check if the index is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name_to_id.is_empty()
    }

    /// Consume the index and return the inner maps.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> IndexMaps {
        (self.name_to_id, self.id_to_name, self.path_to_id)
    }

    /// Insert a name→ID mapping.
    pub fn insert_name(&mut self, name: SchemaName, id: SchemaId) {
        self.id_to_name.insert(id, name.clone());
        self.name_to_id.insert(name, id);
    }

    /// Insert a path→ID mapping.
    pub fn insert_path(&mut self, path: RelativePath, id: SchemaId) {
        self.path_to_id.insert(path, id);
    }
}

/// Inner maps returned by `into_inner`.
pub type IndexMaps = (
    HashMap<SchemaName, SchemaId>,
    HashMap<SchemaId, SchemaName>,
    HashMap<RelativePath, SchemaId>,
);

/// Collection of name→ID pairs for schema lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameIdPairs(Vec<(SchemaName, SchemaId)>);

impl NameIdPairs {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, pair: (SchemaName, SchemaId)) {
        self.0.push(pair);
    }

    pub fn into_vec(self) -> Vec<(SchemaName, SchemaId)> {
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &(SchemaName, SchemaId)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to a `HashMap` of name → id.
    pub fn to_map(&self) -> HashMap<SchemaName, SchemaId> {
        self.0.iter().map(|pair| (pair.0.clone(), pair.1)).collect()
    }

    /// Reverse the tuple order to get (id, name) pairs for building id → name
    /// maps.
    pub fn reversed(&self) -> Vec<(SchemaId, SchemaName)> {
        self.0.iter().map(|pair| (pair.1, pair.0.clone())).collect()
    }
}

impl Default for NameIdPairs {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<(SchemaName, SchemaId)>> for NameIdPairs {
    fn from(vec: Vec<(SchemaName, SchemaId)>) -> Self {
        Self(vec)
    }
}

impl Deref for NameIdPairs {
    type Target = Vec<(SchemaName, SchemaId)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Collection of path→ID pairs for schema discovery.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathIdPairs(Vec<(RelativePath, SchemaId)>);

impl PathIdPairs {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, pair: (RelativePath, SchemaId)) {
        self.0.push(pair);
    }

    pub fn into_vec(self) -> Vec<(RelativePath, SchemaId)> {
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &(RelativePath, SchemaId)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to a `HashMap` of path → id.
    pub fn to_map(&self) -> HashMap<RelativePath, SchemaId> {
        self.0.iter().map(|pair| (pair.0.clone(), pair.1)).collect()
    }
}

impl Default for PathIdPairs {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<(RelativePath, SchemaId)>> for PathIdPairs {
    fn from(vec: Vec<(RelativePath, SchemaId)>) -> Self {
        Self(vec)
    }
}

impl Deref for PathIdPairs {
    type Target = Vec<(RelativePath, SchemaId)>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        );

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
            (RelativePath::try_from("schemas/user.json").unwrap(), id1),
            (RelativePath::try_from("schemas/task.json").unwrap(), id2),
        ]);

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
        pairs.push((RelativePath::try_from("schemas/user.json").unwrap(), id1));
        pairs.push((RelativePath::try_from("schemas/task.json").unwrap(), id2));

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
            vec![(RelativePath::try_from("schemas/user.json").unwrap(), id1)]
                .into();
        assert_eq!(pairs.len(), 1);
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
