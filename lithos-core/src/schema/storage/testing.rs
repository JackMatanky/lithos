//! Testing and benchmarking utilities for the schema module.
//!
//! This module provides test doubles and benchmark fixtures for schema
//! components. Code in this module is compiled for both `#[cfg(test)]`
//! and benchmarks.
//!
//! # Available Utilities
//!
//! - [`InMemoryRepository`] - HashMap-backed Repository for pure unit tests
//! - Test helpers for building test data
//!
//! # Design Rationale
//!
//! This module exists to enable **pure unit tests** following matklad's
//! test purity hierarchy:
//!
//! - **Pure computation** (fastest, most reliable)
//! - Threads → Filesystem → Network → Processes (slowest, least reliable)
//!
//! By providing an in-memory Repository implementation, we eliminate filesystem
//! IO from unit tests while maintaining test extent (can still test full
//! pipelines end-to-end).
//!
//! # When to Use
//!
//! - **Unit tests** (`#[cfg(test)]` modules): Use `InMemoryRepository`
//! - **Integration tests** (`tests/` directory): Use `RedbRepository`
//! - **Benchmarks**: Use `InMemoryRepository` for micro-benchmarks

// Test-only code: relax pedantic lints for pragmatic test utilities
#![expect(
    clippy::missing_inline_in_public_items,
    clippy::map_err_ignore,
    clippy::significant_drop_tightening,
    clippy::pattern_type_mismatch,
    clippy::exhaustive_enums,
    clippy::iter_over_hash_type,
    clippy::doc_markdown,
    clippy::doc_paragraphs_missing_punctuation,
    reason = "Test utilities prioritize readability over micro-optimizations"
)]

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::DbError,
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        identifier::{SchemaId, SchemaName},
        index::{NameIdPairs, PathIdPairs, SchemaIndex},
        inheritance::InheritanceGraph,
        property::{PropertyMap, PropertyName},
        repository::{
            SchemaReadRepository, SchemaStorageV2Error, SchemaWriteRepository,
        },
        views::{RawPropertyBankView, RawSchemaView, RawView as _},
    },
};

// ============================================================================
// InMemoryRepository - For Pure Unit Tests
// ============================================================================

/// HashMap-backed Repository implementation for pure unit tests.
///
/// Provides an in-memory implementation of the Repository trait that eliminates
/// filesystem IO to achieve test purity. This is NOT a mock - it's a real
/// Repository implementation that uses HashMap for storage.
///
/// # Thread Safety
///
/// All internal state is protected by `RwLock` for thread-safe concurrent
/// access. Multiple readers can read simultaneously; writers get exclusive
/// access.
///
/// # Example
///
/// ```ignore
/// use lithos_core::schema::storage::testing::InMemoryRepository;
/// use lithos_core::schema::ingestor::Ingestor;
///
/// #[test]
/// fn test_schema_loading() {
///     let repo = InMemoryRepository::new();
///     let ingestor = Ingestor::new(/* ... */, Arc::new(repo));
///     // ... test logic
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    /// Schema storage: `SchemaId` → `Schema`
    schemas: Arc<RwLock<HashMap<SchemaId, Schema>>>,

    /// Name-to-ID lookup: `SchemaName` → `SchemaId`
    name_to_id: Arc<RwLock<HashMap<SchemaName, SchemaId>>>,

    /// Property bank singleton
    property_bank: Arc<RwLock<Option<PropertyBank>>>,

    /// Raw schema views for staleness detection: `SchemaId` → `RawSchemaView`
    raw_schema_views: Arc<RwLock<HashMap<SchemaId, RawSchemaView>>>,

    /// Path-to-ID lookup for raw views: file path → `SchemaId`
    path_to_id: Arc<RwLock<HashMap<RelativePath, SchemaId>>>,

    /// Raw property bank views for staleness detection: path →
    /// `RawPropertyBankView`
    raw_bank_views: Arc<RwLock<HashMap<RelativePath, RawPropertyBankView>>>,

    /// Cached topological graph singleton.
    topological_graph: Arc<RwLock<Option<InheritanceGraph<()>>>>,
}

impl InMemoryRepository {
    /// Creates a new empty in-memory repository.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let repo = InMemoryRepository::new();
    /// assert_eq!(repo.schema_count(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            property_bank: Arc::new(RwLock::new(None)),
            raw_schema_views: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            raw_bank_views: Arc::new(RwLock::new(HashMap::new())),
            topological_graph: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the number of schemas currently stored (test helper).
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned (another thread panicked while holding
    /// the lock).
    #[must_use]
    pub fn schema_count(&self) -> usize {
        self.schemas.read().expect("Lock poisoned").len()
    }

    /// Clears all stored data (test helper).
    ///
    /// Useful for resetting state between test cases.
    ///
    /// # Panics
    ///
    /// Panics if any lock is poisoned.
    pub fn clear(&self) {
        self.schemas.write().expect("Lock poisoned").clear();
        self.name_to_id.write().expect("Lock poisoned").clear();
        *self.property_bank.write().expect("Lock poisoned") = None;
        self.raw_schema_views.write().expect("Lock poisoned").clear();
        self.path_to_id.write().expect("Lock poisoned").clear();
        self.raw_bank_views.write().expect("Lock poisoned").clear();
        *self.topological_graph.write().expect("Lock poisoned") = None;
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for InMemoryRepository operations.
///
/// Since this is an in-memory implementation, most operations cannot fail.
/// This error type exists to satisfy the Repository trait's associated Error
/// type.
#[derive(Debug, thiserror::Error)]
pub enum InMemoryError {
    /// A generic error occurred (used for trait compatibility).
    #[error("In-memory repository error: {message}")]
    Internal {
        /// Error message describing what went wrong.
        message: Box<str>,
    },

    /// Lock was poisoned (another thread panicked while holding the lock).
    #[error("Lock poisoned: {context}")]
    LockPoisoned {
        /// Context describing which lock was poisoned.
        context: Box<str>,
    },
}

impl InMemoryError {
    /// Creates an internal error with a message.
    fn internal(message: impl Into<Box<str>>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Creates a lock poisoned error with context.
    fn lock_poisoned(context: impl Into<Box<str>>) -> Self {
        Self::LockPoisoned {
            context: context.into(),
        }
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "error conversion consumes formatted message"
)]
#[inline]
fn to_v2_error(err: InMemoryError) -> SchemaStorageV2Error {
    SchemaStorageV2Error::from(DbError::Corruption(err.to_string()))
}

impl SchemaReadRepository for InMemoryRepository {
    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaStorageV2Error> {
        let schemas = self.schemas.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("find_schema_by_id"))
        })?;
        Ok(schemas.get(&id).cloned())
    }

    #[inline]
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaStorageV2Error> {
        let schemas = self.schemas.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("find_many_schemas_by_id"))
        })?;
        Ok(ids.iter().map(|id| schemas.get(id).cloned()).collect())
    }

    #[inline]
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, SchemaStorageV2Error> {
        let schemas = self.schemas.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("find_schemas_by_ids"))
        })?;
        Ok(ids.iter().filter_map(|id| schemas.get(id).cloned()).collect())
    }

    #[inline]
    fn list_schemas(&self) -> Result<Vec<Schema>, SchemaStorageV2Error> {
        let schemas = self.schemas.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("list_schemas"))
        })?;
        Ok(schemas.values().cloned().collect())
    }

    #[inline]
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<
        std::collections::HashMap<SchemaId, Vec<PropertyName>>,
        SchemaStorageV2Error,
    > {
        let schemas = self.schemas.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_schemas_using_properties",
            ))
        })?;
        let mut usage: std::collections::HashMap<SchemaId, Vec<PropertyName>> =
            HashMap::new();
        for (schema_id, schema) in schemas.iter() {
            let matching_props: Vec<PropertyName> = schema
                .properties()
                .iter()
                .filter(|(prop_name, _)| property_names.contains(prop_name))
                .map(|(prop_name, _)| prop_name.clone())
                .collect();
            if !matching_props.is_empty() {
                usage.insert(*schema_id, matching_props);
            }
        }
        Ok(usage)
    }

    #[inline]
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, SchemaStorageV2Error> {
        let views = self.raw_schema_views.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("get_raw_schema_view"))
        })?;
        Ok(views.get(&id).cloned())
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, SchemaStorageV2Error> {
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_raw_schema_view_by_path (path_to_id)",
            ))
        })?;
        let views = self.raw_schema_views.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_raw_schema_view_by_path (views)",
            ))
        })?;
        Ok(path_to_id.get(path).and_then(|id| views.get(id).cloned()))
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaStorageV2Error> {
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_raw_schema_views_by_paths (path_to_id)",
            ))
        })?;
        let views = self.raw_schema_views.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_raw_schema_views_by_paths (views)",
            ))
        })?;
        Ok(paths
            .iter()
            .map(|path| {
                path_to_id.get(path).and_then(|id| views.get(id).cloned())
            })
            .collect())
    }

    #[inline]
    fn get_property_bank(
        &self,
    ) -> Result<Option<PropertyBank>, SchemaStorageV2Error> {
        let bank = self.property_bank.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("get_property_bank"))
        })?;
        Ok(bank.clone())
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, SchemaStorageV2Error> {
        let graph = self.topological_graph.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("get_topological_graph"))
        })?;
        Ok(graph.clone())
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, SchemaStorageV2Error> {
        let views = self.raw_bank_views.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "get_raw_property_bank_view",
            ))
        })?;
        Ok(views.get(path).cloned())
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error> {
        let name_to_id = self.name_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("find_schema_id_by_name"))
        })?;
        Ok(name_to_id.get(name).copied())
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<SchemaId>, SchemaStorageV2Error> {
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("find_schema_id_by_path"))
        })?;
        Ok(path_to_id.get(path).copied())
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<SchemaId>>, SchemaStorageV2Error> {
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "find_schema_ids_by_paths",
            ))
        })?;
        Ok(paths.iter().map(|path| path_to_id.get(path).copied()).collect())
    }

    #[inline]
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<NameIdPairs, SchemaStorageV2Error> {
        let name_to_id = self.name_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "list_schema_name_id_pairs",
            ))
        })?;
        let pairs: Vec<_> =
            name_to_id.iter().map(|(name, id)| (name.clone(), *id)).collect();
        Ok(pairs.into())
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<PathIdPairs, SchemaStorageV2Error> {
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "list_schema_path_id_pairs",
            ))
        })?;
        let pairs: Vec<_> =
            path_to_id.iter().map(|(path, id)| (path.clone(), *id)).collect();
        Ok(pairs.into())
    }

    #[inline]
    fn get_schema_index(&self) -> Result<SchemaIndex, SchemaStorageV2Error> {
        let name_to_id = self.name_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "get_schema_index (name_to_id)",
            ))
        })?;
        let path_to_id = self.path_to_id.read().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "get_schema_index (path_to_id)",
            ))
        })?;
        let name_pairs: Vec<_> =
            name_to_id.iter().map(|(n, id)| (n.clone(), *id)).collect();
        let path_pairs: Vec<_> =
            path_to_id.iter().map(|(p, id)| (p.clone(), *id)).collect();
        SchemaIndex::from_pairs(name_pairs, path_pairs)
            .map_err(|e| to_v2_error(InMemoryError::internal(e.to_string())))
    }
}

impl SchemaWriteRepository for InMemoryRepository {
    #[inline]
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageV2Error> {
        let mut schemas_map = self.schemas.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("save_schema (schemas)"))
        })?;
        let mut name_to_id_map = self.name_to_id.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_schema (name_to_id)",
            ))
        })?;
        schemas_map.insert(*schema.id(), schema.clone());
        name_to_id_map.insert(schema.name().clone(), *schema.id());
        Ok(())
    }

    #[inline]
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaStorageV2Error> {
        let mut schemas_map = self.schemas.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_many_schemas (schemas)",
            ))
        })?;
        let mut name_to_id_map = self.name_to_id.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_many_schemas (name_to_id)",
            ))
        })?;
        for schema in schemas {
            schemas_map.insert(*schema.id(), schema.clone());
            name_to_id_map.insert(schema.name().clone(), *schema.id());
        }
        Ok(())
    }

    #[inline]
    fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaStorageV2Error> {
        let mut storage = self.property_bank.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("save_property_bank"))
        })?;
        *storage = Some(bank.clone());
        Ok(())
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaStorageV2Error> {
        let mut views = self.raw_bank_views.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_raw_property_bank_view",
            ))
        })?;
        views.insert(path.clone(), view.clone());
        Ok(())
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaStorageV2Error> {
        let mut views = self.raw_schema_views.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_raw_schema_view (views)",
            ))
        })?;
        let mut path_to_id = self.path_to_id.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "save_raw_schema_view (path_to_id)",
            ))
        })?;
        path_to_id.insert(view.file_path().clone(), id);
        views.insert(id, view.clone());
        Ok(())
    }

    #[inline]
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), SchemaStorageV2Error> {
        let mut storage = self.topological_graph.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("save_topological_graph"))
        })?;
        *storage = Some(graph.clone());
        Ok(())
    }

    #[inline]
    fn delete_schema(&self, id: SchemaId) -> Result<(), SchemaStorageV2Error> {
        let mut schemas = self.schemas.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned("delete_schema (schemas)"))
        })?;
        let mut name_to_id = self.name_to_id.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "delete_schema (name_to_id)",
            ))
        })?;
        let mut raw_views = self.raw_schema_views.write().map_err(|_| {
            to_v2_error(InMemoryError::lock_poisoned(
                "delete_schema (raw_views)",
            ))
        })?;
        if let Some(schema) = schemas.remove(&id) {
            name_to_id.remove(schema.name());
        }
        raw_views.remove(&id);
        Ok(())
    }
}

// ============================================================================
// Error Conversions
// ============================================================================

/// Convert `InMemoryError` to `SchemaRepositoryError` for loader compatibility.
impl From<InMemoryError> for crate::schema::error::SchemaRepositoryError {
    fn from(err: InMemoryError) -> Self {
        let db_error = match err {
            InMemoryError::Internal {
                message,
            } => DbError::Corruption(message.into()),
            InMemoryError::LockPoisoned {
                context,
            } => DbError::Corruption(format!("Lock poisoned: {context}")),
        };

        Self::Storage(crate::schema::error::SchemaStorageError::Storage(
            db_error,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_repository_is_empty() {
        let repo = InMemoryRepository::new();
        assert_eq!(repo.schema_count(), 0);
    }

    #[test]
    fn default_repository_is_empty() {
        let repo = InMemoryRepository::default();
        assert_eq!(repo.schema_count(), 0);
    }

    #[test]
    fn clear_resets_all_state() {
        let repo = InMemoryRepository::new();

        // Add some data
        let id = SchemaId::new();
        let name = SchemaName::try_new("test-schema").unwrap();
        let schema =
            Schema::new(id, name, Vec::new(), vec![], PropertyMap::new());

        repo.save_schema(&schema).unwrap();
        assert_eq!(repo.schema_count(), 1);

        // Clear
        repo.clear();
        assert_eq!(repo.schema_count(), 0);
    }
}
