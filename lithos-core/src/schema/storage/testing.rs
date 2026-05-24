//! Testing utilities for schema storage components.
//!
//! This module provides test doubles for the schema repository traits,
//! enabling pure unit tests without filesystem dependencies. Code in this
//! module is compiled for both `#[cfg(test)]` and benchmarks.
//!
//! # Exports
//!
//! - [`InMemoryRepository`] - HashMap-backed [`Repository`] implementation
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
//! # Example
//!
//! ```ignore
//! use lithos_core::schema::storage::testing::InMemoryRepository;
//! use lithos_core::schema::repository::Repository;
//!
//! #[test]
//! fn test_schema_resolution() {
//!     let repo = InMemoryRepository::new();
//!     repo.save_schema(&schema).unwrap();
//!
//!     // Pure computation test - no filesystem I/O
//!     let loaded = repo.find_schema_by_id(schema.id()).unwrap();
//!     assert_eq!(loaded, Some(schema));
//! }
//! ```
//!
//! [`Repository`]: crate::schema::repository::Repository

// Test-only code: relax pedantic lints for pragmatic test utilities
#![expect(
    clippy::missing_inline_in_public_items,
    clippy::significant_drop_tightening,
    clippy::pattern_type_mismatch,
    clippy::iter_over_hash_type,
    clippy::doc_paragraphs_missing_punctuation,
    reason = "Test utilities prioritize readability over micro-optimizations"
)]

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::{
        DbError,
        testing::{FailurePoint, InMemoryHarness, read_lock, write_lock},
    },
    fs::RelativePath,
    schema::{
        aggregate::Schema,
        bank::PropertyBank,
        error::SchemaStorageError,
        identifier::{SchemaId, SchemaName},
        index::{NameIdPairs, PathIdPairs, SchemaIndex},
        inheritance::InheritanceGraph,
        property::{PropertyMap, PropertyName},
        repository::{ReadRepository, WriteRepository},
        views::{RawPropertyBankView, RawSchemaView, RawView as _},
    },
};

// ============================================================================
// InMemoryRepository - For Pure Unit Tests
// ============================================================================

/// HashMap-backed [`Repository`] implementation for pure unit tests.
///
/// This is NOT a mock - it's a fully functional [`Repository`] implementation
/// that uses `HashMap` for storage instead of a persistent database. All
/// [`Repository`] trait methods are implemented with identical semantics to
/// [`RedbRepository`], except data is stored in memory only.
///
/// # Thread Safety
///
/// All internal state is protected by `RwLock` for thread-safe concurrent
/// access. Multiple readers can read simultaneously; writers get exclusive
/// access. Lock poisoning (if a thread panics while holding a lock) is
/// reported via `SchemaStorageError::Storage(DbError::Corruption(...))`.
///
/// # Performance Characteristics
///
/// Optimized for **fast, deterministic unit tests**:
///
/// - **O(1) average lookups**: Direct `HashMap` access (no serialization)
/// - **No disk I/O**: All operations execute in memory
/// - **Cheap cloning**: `Arc`-wrapped state means cloning is a reference count
///   increment
/// - **Memory trade-off**: Stores full deserialized objects (~1-2 MB per 1000
///   schemas)
///
/// Faster than [`RedbRepository`] for small test datasets (< 1000 schemas) but
/// unsuitable for large-scale benchmarks or production use (no durability).
///
/// # When to Use
///
/// **Use for:**
/// - Unit tests in `#[cfg(test)]` modules (pure computation testing)
/// - Micro-benchmarks (isolate logic from storage layer overhead)
///
/// **Do NOT use for:**
/// - Integration tests (use [`RedbRepository`] to verify
///   serialization/durability)
/// - Production code (no persistence guarantees)
///
/// # Example
///
/// ```ignore
/// use lithos_core::schema::storage::testing::InMemoryRepository;
/// use lithos_core::schema::repository::Repository;
///
/// #[test]
/// fn test_schema_crud() {
///     let repo = InMemoryRepository::new();
///     repo.save_schema(&schema).unwrap();
///     assert_eq!(repo.schema_count(), 1);
///
///     let loaded = repo.find_schema_by_id(schema.id()).unwrap();
///     assert_eq!(loaded, Some(schema));
/// }
/// ```
///
/// [`Repository`]: crate::schema::repository::Repository
/// [`RedbRepository`]: crate::schema::storage::RedbRepository
#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    /// Test harness for operation instrumentation and failure injection
    harness: Arc<InMemoryHarness>,

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
            harness: Arc::new(InMemoryHarness::new()),
            schemas: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            property_bank: Arc::new(RwLock::new(None)),
            raw_schema_views: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            raw_bank_views: Arc::new(RwLock::new(HashMap::new())),
            topological_graph: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a new repository with the specified test harness.
    ///
    /// Useful for tests that need custom failure injection behavior.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::db::testing::InMemoryHarness;
    /// use lithos_core::schema::storage::testing::InMemoryRepository;
    ///
    /// let harness = InMemoryHarness::with_injector(Box::new(my_injector));
    /// let repo = InMemoryRepository::with_harness(harness);
    /// ```
    #[must_use]
    pub(crate) fn with_harness(harness: InMemoryHarness) -> Self {
        Self {
            harness: Arc::new(harness),
            schemas: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            property_bank: Arc::new(RwLock::new(None)),
            raw_schema_views: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            raw_bank_views: Arc::new(RwLock::new(HashMap::new())),
            topological_graph: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns a reference to the test harness for instrumentation.
    ///
    /// Allows tests to inspect operation counters and configure failure
    /// injection.
    #[must_use]
    pub(crate) fn harness(&self) -> &InMemoryHarness {
        &self.harness
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

impl ReadRepository for InMemoryRepository {
    #[inline]
    fn find_schema_by_id(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let schemas = read_lock(&self.schemas, "find_schema_by_id")?;
        self.harness.counters().inc_read();

        Ok(schemas.get(&id).cloned())
    }

    #[inline]
    fn find_many_schemas_by_id(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Option<Schema>>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let schemas = read_lock(&self.schemas, "find_many_schemas_by_id")?;
        self.harness.counters().inc_read();

        Ok(ids.iter().map(|id| schemas.get(id).cloned()).collect())
    }

    #[inline]
    fn find_schemas_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<Vec<Schema>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let schemas = read_lock(&self.schemas, "find_schemas_by_ids")?;
        self.harness.counters().inc_read();

        Ok(ids.iter().filter_map(|id| schemas.get(id).cloned()).collect())
    }

    #[inline]
    fn list_schemas(&self) -> Result<Vec<Schema>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let schemas = read_lock(&self.schemas, "list_schemas")?;
        self.harness.counters().inc_read();

        Ok(schemas.values().cloned().collect())
    }

    #[inline]
    fn find_schemas_using_properties(
        &self,
        property_names: &[PropertyName],
    ) -> Result<
        std::collections::HashMap<SchemaId, Vec<PropertyName>>,
        SchemaStorageError,
    > {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let schemas =
            read_lock(&self.schemas, "find_schemas_using_properties")?;
        self.harness.counters().inc_read();

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
    ) -> Result<Option<RawSchemaView>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let views = read_lock(&self.raw_schema_views, "get_raw_schema_view")?;
        self.harness.counters().inc_read();

        Ok(views.get(&id).cloned())
    }

    #[inline]
    fn find_raw_schema_view_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawSchemaView>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id = read_lock(
            &self.path_to_id,
            "find_raw_schema_view_by_path (path_to_id)",
        )?;
        self.harness.counters().inc_read();

        let views = read_lock(
            &self.raw_schema_views,
            "find_raw_schema_view_by_path (views)",
        )?;
        self.harness.counters().inc_read();

        Ok(path_to_id.get(path).and_then(|id| views.get(id).cloned()))
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<RawSchemaView>>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id = read_lock(
            &self.path_to_id,
            "find_raw_schema_views_by_paths (path_to_id)",
        )?;
        self.harness.counters().inc_read();

        let views = read_lock(
            &self.raw_schema_views,
            "find_raw_schema_views_by_paths (views)",
        )?;
        self.harness.counters().inc_read();

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
    ) -> Result<Option<PropertyBank>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let bank = read_lock(&self.property_bank, "get_property_bank")?;
        self.harness.counters().inc_read();

        Ok(bank.clone())
    }

    #[inline]
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let graph =
            read_lock(&self.topological_graph, "get_topological_graph")?;
        self.harness.counters().inc_read();

        Ok(graph.clone())
    }

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        path: &RelativePath,
    ) -> Result<Option<RawPropertyBankView>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let views =
            read_lock(&self.raw_bank_views, "get_raw_property_bank_view")?;
        self.harness.counters().inc_read();

        Ok(views.get(path).cloned())
    }

    #[inline]
    fn find_schema_id_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<SchemaId>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let name_to_id = read_lock(&self.name_to_id, "find_schema_id_by_name")?;
        self.harness.counters().inc_read();

        Ok(name_to_id.get(name).copied())
    }

    #[inline]
    fn find_schema_id_by_path(
        &self,
        path: &RelativePath,
    ) -> Result<Option<SchemaId>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id = read_lock(&self.path_to_id, "find_schema_id_by_path")?;
        self.harness.counters().inc_read();

        Ok(path_to_id.get(path).copied())
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        paths: &[RelativePath],
    ) -> Result<Vec<Option<SchemaId>>, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id =
            read_lock(&self.path_to_id, "find_schema_ids_by_paths")?;
        self.harness.counters().inc_read();

        Ok(paths.iter().map(|path| path_to_id.get(path).copied()).collect())
    }

    #[inline]
    fn list_schema_name_id_pairs(
        &self,
    ) -> Result<NameIdPairs, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let name_to_id =
            read_lock(&self.name_to_id, "list_schema_name_id_pairs")?;
        self.harness.counters().inc_read();

        let pairs: Vec<_> =
            name_to_id.iter().map(|(name, id)| (name.clone(), *id)).collect();
        Ok(pairs.into())
    }

    #[inline]
    fn list_schema_path_id_pairs(
        &self,
    ) -> Result<PathIdPairs, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id =
            read_lock(&self.path_to_id, "list_schema_path_id_pairs")?;
        self.harness.counters().inc_read();
        let pairs: Vec<_> =
            path_to_id.iter().map(|(path, id)| (path.clone(), *id)).collect();
        Ok(pairs.into())
    }

    #[inline]
    fn get_schema_index(&self) -> Result<SchemaIndex, SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let name_to_id =
            read_lock(&self.name_to_id, "get_schema_index (name_to_id)")?;
        self.harness.counters().inc_read();

        let path_to_id =
            read_lock(&self.path_to_id, "get_schema_index (path_to_id)")?;
        self.harness.counters().inc_read();

        let name_pairs: Vec<_> =
            name_to_id.iter().map(|(n, id)| (n.clone(), *id)).collect();
        let path_pairs: Vec<_> =
            path_to_id.iter().map(|(p, id)| (p.clone(), *id)).collect();
        SchemaIndex::from_pairs(name_pairs, path_pairs).map_err(|e| {
            SchemaStorageError::from(DbError::Corruption(e.to_string()))
        })
    }
}

impl WriteRepository for InMemoryRepository {
    #[inline]
    fn save_schema(&self, schema: &Schema) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut schemas_map =
            write_lock(&self.schemas, "save_schema (schemas)")?;
        self.harness.counters().inc_write();

        let mut name_to_id_map =
            write_lock(&self.name_to_id, "save_schema (name_to_id)")?;
        self.harness.counters().inc_write();

        schemas_map.insert(*schema.id(), schema.clone());
        name_to_id_map.insert(schema.name().clone(), *schema.id());
        Ok(())
    }

    #[inline]
    fn save_many_schemas(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut schemas_map =
            write_lock(&self.schemas, "save_many_schemas (schemas)")?;
        self.harness.counters().inc_write();

        let mut name_to_id_map =
            write_lock(&self.name_to_id, "save_many_schemas (name_to_id)")?;
        self.harness.counters().inc_write();

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
    ) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut storage =
            write_lock(&self.property_bank, "save_property_bank")?;
        self.harness.counters().inc_write();

        *storage = Some(bank.clone());
        Ok(())
    }

    #[inline]
    fn save_raw_property_bank_view(
        &self,
        path: &RelativePath,
        view: &RawPropertyBankView,
    ) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views =
            write_lock(&self.raw_bank_views, "save_raw_property_bank_view")?;
        self.harness.counters().inc_write();

        views.insert(path.clone(), view.clone());
        Ok(())
    }

    #[inline]
    fn save_raw_schema_view(
        &self,
        id: SchemaId,
        view: &RawSchemaView,
    ) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views =
            write_lock(&self.raw_schema_views, "save_raw_schema_view (views)")?;
        self.harness.counters().inc_write();

        let mut path_to_id =
            write_lock(&self.path_to_id, "save_raw_schema_view (path_to_id)")?;
        self.harness.counters().inc_write();

        path_to_id.insert(view.file_path().clone(), id);
        views.insert(id, view.clone());
        Ok(())
    }

    #[inline]
    fn save_topological_graph(
        &self,
        graph: &InheritanceGraph<()>,
    ) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut storage =
            write_lock(&self.topological_graph, "save_topological_graph")?;
        self.harness.counters().inc_write();

        *storage = Some(graph.clone());
        Ok(())
    }

    #[inline]
    fn delete_schema(&self, id: SchemaId) -> Result<(), SchemaStorageError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut schemas = write_lock(&self.schemas, "delete_schema (schemas)")?;
        self.harness.counters().inc_write();

        let mut name_to_id =
            write_lock(&self.name_to_id, "delete_schema (name_to_id)")?;
        self.harness.counters().inc_write();

        let mut raw_views =
            write_lock(&self.raw_schema_views, "delete_schema (raw_views)")?;
        self.harness.counters().inc_write();

        if let Some(schema) = schemas.remove(&id) {
            name_to_id.remove(schema.name());
        }
        raw_views.remove(&id);
        Ok(())
    }
}

// ============================================================================
// Error Conversion
// ============================================================================

/// Convert `db::testing::InMemoryDbError` directly to `SchemaStorageError`.
///
/// This avoids an intermediate custom error conversion at call sites that
/// only need to satisfy repository trait error contracts.
#[cfg(test)]
impl From<crate::db::testing::InMemoryDbError> for SchemaStorageError {
    fn from(err: crate::db::testing::InMemoryDbError) -> Self {
        use crate::db::testing::InMemoryDbError as DbTestError;

        let db_error = match err {
            DbTestError::LockPoisoned {
                context,
            } => DbError::Corruption(format!("Lock poisoned: {context}")),
            DbTestError::InjectedFailure {
                reason,
                ..
            } => DbError::Corruption(format!("Injected failure: {reason}")),
            DbTestError::InvariantViolation {
                message,
            } => DbError::Corruption(message.into()),
        };

        SchemaStorageError::Storage(db_error)
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

    mod integration {
        use super::*;
        use crate::db::testing::{
            FailureInjector, FailurePoint, InMemoryDbError,
        };

        struct FailOnWrite;
        struct FailOnRead;

        impl FailureInjector for FailOnWrite {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeWrite {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "write injection".into(),
                    });
                }

                Ok(())
            }
        }

        impl FailureInjector for FailOnRead {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeRead {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "read injection".into(),
                    });
                }

                Ok(())
            }
        }

        #[test]
        fn exposes_harness_for_instrumentation() {
            // Arrange
            let repo = InMemoryRepository::new();

            // Act
            let harness = repo.harness();

            // Assert: counters start at zero
            let snapshot = harness.counters().snapshot();
            assert_eq!(snapshot.reads, 0);
            assert_eq!(snapshot.writes, 0);
        }

        #[test]
        fn instruments_write_operations() {
            // Arrange
            let repo = InMemoryRepository::new();
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, vec![], vec![], PropertyMap::default());

            // Act
            repo.save_schema(&schema).unwrap();

            // Assert: 2 writes (schemas map + name_to_id map)
            let snapshot = repo.harness().counters().snapshot();
            assert_eq!(snapshot.writes, 2);
        }

        #[test]
        fn instruments_read_operations() {
            // Arrange
            let repo = InMemoryRepository::new();
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, vec![], vec![], PropertyMap::default());
            repo.save_schema(&schema).unwrap();

            // Act
            let _result = repo.find_schema_by_id(id).unwrap();

            // Assert: 1 read (schemas map)
            let snapshot = repo.harness().counters().snapshot();
            assert_eq!(snapshot.reads, 1);
        }

        #[test]
        fn respects_write_failure_injection() {
            // Arrange
            let harness = InMemoryHarness::with_injector(Box::new(FailOnWrite));
            let repo = InMemoryRepository::with_harness(harness);
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, vec![], vec![], PropertyMap::default());

            // Act
            let result = repo.save_schema(&schema);

            // Assert
            assert!(matches!(result, Err(SchemaStorageError::Storage(_))));
        }

        #[test]
        fn respects_read_failure_injection() {
            // Arrange
            let setup_repo = InMemoryRepository::new();
            let id = SchemaId::new();
            let name = SchemaName::try_new("test-schema").unwrap();
            let schema =
                Schema::new(id, name, vec![], vec![], PropertyMap::default());
            setup_repo.save_schema(&schema).unwrap();

            let harness = InMemoryHarness::with_injector(Box::new(FailOnRead));
            let repo = InMemoryRepository::with_harness(harness);
            repo.save_schema(&schema).unwrap();

            // Act
            let result = repo.find_schema_by_id(id);

            // Assert
            assert!(matches!(result, Err(SchemaStorageError::Storage(_))));
        }
    }
}
