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

#![expect(dead_code, reason = "index methods used by downstream modules")]

use std::{collections::HashMap, path::PathBuf};

use crate::schema::aggregate::{SchemaId, SchemaName};

/// Inner maps returned by `into_inner`.
pub(crate) type IndexMaps = (
    HashMap<SchemaName, SchemaId>,
    HashMap<SchemaId, SchemaName>,
    HashMap<PathBuf, SchemaId>,
);

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
pub(crate) struct SchemaIndex {
    name_to_id: HashMap<SchemaName, SchemaId>,
    id_to_name: HashMap<SchemaId, SchemaName>,
    path_to_id: HashMap<PathBuf, SchemaId>,
}

impl SchemaIndex {
    /// Create an empty index.
    #[inline]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Create index from name→ID pairs (e.g., from repository's list method).
    pub(crate) fn from_name_id_pairs(
        pairs: impl IntoIterator<Item = (SchemaName, SchemaId)>,
    ) -> Self {
        let mut name_to_id = HashMap::new();
        let mut id_to_name = HashMap::new();

        for (name, id) in pairs {
            id_to_name.insert(id, name.clone());
            name_to_id.insert(name, id);
        }

        Self {
            name_to_id,
            id_to_name,
            path_to_id: HashMap::new(),
        }
    }

    /// Create index from path→ID pairs (e.g., from repository's list method).
    pub(crate) fn from_path_id_pairs(
        pairs: impl IntoIterator<Item = (PathBuf, SchemaId)>,
    ) -> Self {
        let mut path_to_id = HashMap::new();

        for (path, id) in pairs {
            path_to_id.insert(path, id);
        }

        Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            path_to_id,
        }
    }

    /// Create index from both name→ID and path→ID pairs.
    pub(crate) fn from_pairs(
        name_pairs: impl IntoIterator<Item = (SchemaName, SchemaId)>,
        path_pairs: impl IntoIterator<Item = (PathBuf, SchemaId)>,
    ) -> Self {
        let mut name_to_id = HashMap::new();
        let mut id_to_name = HashMap::new();

        for (name, id) in name_pairs {
            id_to_name.insert(id, name.clone());
            name_to_id.insert(name, id);
        }

        let mut path_to_id = HashMap::new();
        for (path, id) in path_pairs {
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
    pub(crate) fn get_id_by_name(&self, name: &SchemaName) -> Option<SchemaId> {
        self.name_to_id.get(name).copied()
    }

    /// Get schema name by ID.
    #[inline]
    pub(crate) fn get_name_by_id(&self, id: &SchemaId) -> Option<&SchemaName> {
        self.id_to_name.get(id)
    }

    /// Get schema ID by path.
    #[inline]
    pub(crate) fn get_id_by_path(&self, path: &PathBuf) -> Option<SchemaId> {
        self.path_to_id.get(path).copied()
    }

    /// Iterate over name→ID pairs.
    pub(crate) fn iter_name_id(
        &self,
    ) -> impl Iterator<Item = (&SchemaName, &SchemaId)> {
        self.name_to_id.iter()
    }

    /// Iterate over path→ID pairs.
    pub(crate) fn iter_path_id(
        &self,
    ) -> impl Iterator<Item = (&PathBuf, &SchemaId)> {
        self.path_to_id.iter()
    }

    /// Get the number of schemas in the index.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.name_to_id.len()
    }

    /// Check if the index is empty.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.name_to_id.is_empty()
    }

    /// Consume the index and return the inner maps.
    #[inline]
    pub(crate) fn into_inner(self) -> IndexMaps {
        (self.name_to_id, self.id_to_name, self.path_to_id)
    }

    /// Insert a name→ID mapping.
    pub(crate) fn insert_name(&mut self, name: SchemaName, id: SchemaId) {
        self.id_to_name.insert(id, name.clone());
        self.name_to_id.insert(name, id);
    }

    /// Insert a path→ID mapping.
    pub(crate) fn insert_path(&mut self, path: PathBuf, id: SchemaId) {
        self.path_to_id.insert(path, id);
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
}
