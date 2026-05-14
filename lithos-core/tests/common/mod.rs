//! Shared test utilities for schema integration tests.
//!
//! Provides RAII test database setup, builders for test data, and assertion
//! helpers following Rust integration test best practices.

#![allow(
    dead_code,
    reason = "Test utilities - not all helpers used in every test file"
)]
#![allow(
    clippy::pub_use,
    reason = "Test module re-exports port traits for test convenience"
)]

use std::{error::Error, path::PathBuf, sync::Arc};

use lithos_core::{
    db::Store,
    schema::{
        aggregate::Schema,
        identifier::SchemaName,
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec, StringSpec},
        repository::ReadRepository,
        storage::RedbRepository,
    },
};
use tempfile::TempDir;

/// Extension trait providing convenience methods for Repository operations in
/// tests.
///
/// Provides `find_by_name` as a convenience that combines
/// `find_schema_id_by_name` + `find_schema_by_id`.
pub trait RepositoryExt {
    /// Find a schema by name.
    ///
    /// Convenience wrapper for `find_schema_id_by_name` + `find_schema_by_id`.
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, Box<dyn std::error::Error>>;
}

impl<R> RepositoryExt for R
where
    R: ReadRepository,
{
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, Box<dyn std::error::Error>> {
        let Some(id) = self.find_schema_id_by_name(name)? else {
            return Ok(None);
        };
        self.find_schema_by_id(id).map_err(Into::into)
    }
}

/// Standard test result type for integration tests.
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// Named property tuple used by schema tests.
pub type NamedProperty = (PropertyName, Property);

// ----------------------------------------------------------- //
//                    RAII Test Database                       //
// ----------------------------------------------------------- //

/// Test database with RAII cleanup via `TempDir`.
///
/// Automatically cleans up filesystem resources when dropped.
/// Each test gets an isolated database for parallel execution.
pub struct TestDb {
    /// Temporary directory (cleanup on drop).
    dir: TempDir,
    /// Store instance wrapped in Arc for sharing with Repository.
    store: Arc<Store>,
}

impl TestDb {
    /// Create a new test database with automatic cleanup.
    ///
    /// # Errors
    /// Returns error if temporary directory or database creation fails.
    ///
    /// # Examples
    /// ```no_run
    /// # use tests::common::{TestDb, TestResult};
    /// # fn test() -> TestResult {
    /// let test_db = TestDb::new()?;
    /// let db = test_db.store();
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn new() -> TestResult<Self> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("lithos.redb");
        let store = Arc::new(Store::open(&db_path)?);
        Ok(Self {
            dir,
            store,
        })
    }

    /// Get reference to the Store Arc (for cloning into Repository).
    #[inline]
    #[must_use]
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    /// Get the database path for logging/debugging.
    #[inline]
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.dir.path().join("lithos.redb")
    }

    /// Reopen the database (simulates application restart).
    ///
    /// Closes the current database and opens a fresh instance.
    /// This allows testing that state persists across sessions.
    ///
    /// **IMPORTANT**: All Arc clones from previous `store()` calls must be
    /// dropped before calling this method, otherwise redb will fail with
    /// "Database already open" error.
    ///
    /// # Errors
    /// Returns error if database cannot be reopened (including if old
    /// Arc clones are still held).
    ///
    /// # Panics
    /// Panics if there are outstanding Arc strong references (indicates a
    /// test bug where repositories weren't properly dropped).
    pub fn reopen(&mut self) -> TestResult<Arc<Store>> {
        let path = self.path();

        // Check if we're the only owner (catch test bugs early)
        let strong_count = Arc::strong_count(&self.store);
        assert!(
            strong_count == 1,
            "Cannot reopen database: {strong_count} outstanding Arc \
             references (expected 1). Did you forget to drop a Repository?"
        );

        // CRITICAL: We must drop the old Store BEFORE opening the new one
        // since redb uses OS-level file locks that prevent concurrent access.
        //
        // Strategy:
        // 1. Create dummy store at different path
        // 2. Swap dummy with real, extracting the old Arc
        // 3. Unwrap and drop the old Store (releases lock)
        // 4. Drop the dummy
        // 5. Open real store at original path

        // Step 1: Create dummy store (different path to avoid lock conflict)
        let dummy_path = self.dir.path().join("temp_dummy.redb");
        let dummy_store = Arc::new(Store::open(&dummy_path)?);

        // Step 2: Swap, getting the old Arc
        let old_arc = std::mem::replace(&mut self.store, dummy_store);

        // Step 3: Unwrap and drop (should always succeed since strong_count ==
        // 1)
        #[expect(
            clippy::panic,
            reason = "Test infrastructure - panic for impossible state is \
                      appropriate"
        )]
        let old_store = Arc::try_unwrap(old_arc).unwrap_or_else(|_| {
            panic!(
                "Arc::try_unwrap failed despite strong_count == 1 - this is a \
                 bug"
            )
        });
        drop(old_store); // Releases the lock!

        // Step 4 & 5: Open the real store (dummy is automatically dropped
        // when we reassign self.store)
        self.store = Arc::new(Store::open(&path)?);

        Ok(Arc::clone(&self.store))
    }
}

// ----------------------------------------------------------- //
//                    Repository Setup Helpers                 //
// ----------------------------------------------------------- //

/// Create a Repository implementation for testing.
///
/// Returns a `RedbRepository` sharing the given store.
/// Use `TestDb::store()` to obtain a store for an isolated test database.
///
/// # Examples
/// ```no_run
/// # use tests::common::{setup_repository, TestDb, TestResult};
/// # fn test() -> TestResult {
/// let test_db = TestDb::new()?;
/// let repository = setup_repository(test_db.store());
/// # Ok(())
/// # }
/// ```
#[track_caller]
#[must_use]
pub fn setup_repository(store: &Arc<Store>) -> RedbRepository {
    RedbRepository::new(Arc::clone(store))
}

// ----------------------------------------------------------- //
//                    Property Builders                        //
// ----------------------------------------------------------- //

/// Builder for creating test `Property` instances.
///
/// Provides a fluent API for constructing properties with sensible defaults.
///
/// # Examples
/// ```no_run
/// # use tests::common::{PropertyBuilder, TestResult};
/// # fn test() -> TestResult {
/// let prop = PropertyBuilder::new("status")
///     .optionality(lithos_core::schema::property::Optionality::Required)
///     .build_bool()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PropertyBuilder {
    name: String,
    optionality: Optionality,
    multiplicity: Multiplicity,
    id: Option<PropertyId>,
}

impl PropertyBuilder {
    /// Create a new builder with a property name.
    ///
    /// Defaults:
    /// - Optionality: Optional
    /// - Multiplicity: Single
    /// - ID: Auto-generated UUID v7
    #[must_use]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            optionality: Optionality::default(),
            multiplicity: Multiplicity::default(),
            id: None,
        }
    }

    /// Set the property's optionality.
    #[must_use]
    pub const fn optionality(mut self, optionality: Optionality) -> Self {
        self.optionality = optionality;
        self
    }

    /// Set the property's multiplicity.
    #[must_use]
    pub const fn multiplicity(mut self, multiplicity: Multiplicity) -> Self {
        self.multiplicity = multiplicity;
        self
    }

    /// Set the property's ID (for deterministic testing).
    #[must_use]
    pub const fn id(mut self, id: PropertyId) -> Self {
        self.id = Some(id);
        self
    }

    /// Build a boolean property.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[track_caller]
    pub fn build_bool(self) -> TestResult<NamedProperty> {
        self.build_with_spec(PropertySpec::Bool(BoolSpec::default()))
    }

    /// Build a string property with default spec.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[track_caller]
    pub fn build_string_default(self) -> TestResult<NamedProperty> {
        self.build_with_spec(PropertySpec::String(StringSpec::default()))
    }

    /// Build a property with a custom spec.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[track_caller]
    pub fn build_with_spec(
        self,
        spec: PropertySpec,
    ) -> TestResult<NamedProperty> {
        let name = PropertyName::try_new(&self.name)?;
        let id = self.id.unwrap_or_default();
        let property =
            Property::new(id, self.optionality, self.multiplicity, spec);
        Ok((name, property))
    }
}

/// Create a required, single-value boolean property.
///
/// # Errors
/// Returns error if property name is invalid.
///
/// # Examples
/// ```no_run
/// # use tests::common::{bool_property, TestResult};
/// # fn test() -> TestResult {
/// let (_name, status) = bool_property("is_active")?;
/// # Ok(())
/// # }
/// ```
#[track_caller]
pub fn bool_property(name: &str) -> TestResult<NamedProperty> {
    PropertyBuilder::new(name).build_bool()
}

/// Create a required, single-value string property.
///
/// # Errors
/// Returns error if property name is invalid.
///
/// # Examples
/// ```no_run
/// # use tests::common::{string_property, TestResult};
/// # fn test() -> TestResult {
/// let (_name, title) = string_property("title")?;
/// # Ok(())
/// # }
/// ```
#[track_caller]
pub fn string_property(name: &str) -> TestResult<NamedProperty> {
    PropertyBuilder::new(name).build_string_default()
}

// ----------------------------------------------------------- //
//                    Schema Builders                          //
// ----------------------------------------------------------- //

// NOTE: SchemaBuilder and assertion helpers commented out pending migration
// from StoredSchema/StoredProperty to new Schema/Property aggregates.
// See schema_incremental_resolution.rs for modern test patterns using Loader.

/* DISABLED - Pending migration to new Schema aggregate

/// Builder for creating test `StoredSchema` instances.
///
/// Provides a fluent API for constructing schemas with properties and
/// inheritance.
///
/// # Examples
/// ```no_run
/// # use tests::common::{SchemaBuilder, bool_property, TestResult};
/// # fn test() -> TestResult {
/// let (_name, prop) = bool_property("status")?;
/// let schema = SchemaBuilder::new("task").property(prop).build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SchemaBuilder {
    name: String,
    parent_id: Option<SchemaId>,
    properties: Vec<Property>,
    id: Option<SchemaId>,
}

impl SchemaBuilder {
    /// Create a new builder with a schema name.
    ///
    /// Defaults:
    /// - No parent
    /// - No properties
    /// - Auto-generated UUID v7 ID
    #[must_use]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            parent_id: None,
            properties: Vec::new(),
            id: None,
        }
    }

    /// Set the schema's ID (for deterministic testing).
    #[must_use]
    pub const fn id(mut self, id: SchemaId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the parent schema ID for inheritance.
    #[must_use]
    pub const fn parent(mut self, parent_id: SchemaId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add a property to the schema.
    #[must_use]
    pub fn property(mut self, property: Property) -> Self {
        self.properties.push(property);
        self
    }

    /// Add multiple properties to the schema.
    #[must_use]
    pub fn properties(mut self, properties: Vec<Property>) -> Self {
        self.properties.extend(properties);
        self
    }

    /// Build the schema.
    ///
    /// # Errors
    /// Returns error if schema name is invalid or construction fails.
    #[track_caller]
    pub fn build(self) -> TestResult<StoredSchema> {
        let _name_check = SchemaName::try_new(&self.name)?;
        let id = self.id.unwrap_or_default();
        let mut stored_properties: Vec<StoredProperty> = self
            .properties
            .into_iter()
            .map(|p| {
                StoredProperty::new(
                    p.id(),
                    p.name().as_str().into(),
                    p.optionality() == Optionality::Required,
                    p.multiplicity() == Multiplicity::Many,
                    p.spec().clone(),
                )
            })
            .collect();

        // Sort properties by name to match expected storage order
        stored_properties.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(StoredSchema::new(
            id,
            self.name.into(),
            self.parent_id,
            stored_properties,
        ))
    }
}

// ----------------------------------------------------------- //
//                    Assertion Helpers                        //
// ----------------------------------------------------------- //

/// Assert that schema properties are sorted by name.
///
/// # Panics
/// Panics if properties are not sorted.
#[track_caller]
pub fn assert_properties_sorted(schema: &StoredSchema, context: &str) {
    let props = &schema.properties;
    let mut sorted_props = props.clone();
    sorted_props.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(
        props, &sorted_props,
        "{context}: Schema properties should be sorted by name"
    );
}

/// Assert that a schema has a specific property by name.
///
/// # Panics
/// Panics if property is not found.
#[track_caller]
pub fn assert_has_property(
    schema: &StoredSchema,
    property_name: &str,
    context: &str,
) {
    let has_prop =
        schema.properties.iter().any(|p| p.name.as_ref() == property_name);
    assert!(
        has_prop,
        "{context}: Schema should have property '{property_name}'"
    );
}

/// Assert that a schema does NOT have a specific property by name.
///
/// # Panics
/// Panics if property is found.
#[track_caller]
pub fn assert_not_has_property(
    schema: &StoredSchema,
    property_name: &str,
    context: &str,
) {
    let has_prop =
        schema.properties.iter().any(|p| p.name.as_ref() == property_name);
    assert!(
        !has_prop,
        "{context}: Schema should NOT have property '{property_name}'"
    );
}

*/
// End of disabled SchemaBuilder and assertion helpers
