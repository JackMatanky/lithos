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

use std::{error::Error, path::PathBuf};

use lithos_core::{
    db::Database,
    schema::{
        aggregate::{SchemaId, SchemaName},
        db_command, db_query, ports,
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec, StringSpec},
        storage::{StoredProperty, StoredSchema},
    },
};
// Re-export port traits - tests using wildcard import need these in scope
pub use ports::{Command as CommandPort, Query as QueryPort};
use tempfile::TempDir;

/// Extension trait providing convenience methods for Query operations in tests.
///
/// Provides `find_by_name` as a convenience that combines `find_id_by_name` +
/// `find_by_id`.
pub trait QueryExt {
    /// Find a schema by name (convenience wrapper for `find_id_by_name` +
    /// `find_by_id`).
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<StoredSchema>, Box<dyn std::error::Error>>;
}

impl QueryExt for db_query::Query<'_> {
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<StoredSchema>, Box<dyn std::error::Error>> {
        use ports::Query as _;
        let Some(id) = self.find_id_by_name(name)? else {
            return Ok(None);
        };
        Ok(self.find_by_id(id)?)
    }
}

/// Extension trait providing convenience methods for Command operations in
/// tests.
///
/// Provides `save` as a convenience that calls `save_many` with a single
/// schema.
pub trait CommandExt {
    /// Save a single schema (convenience wrapper for `save_many`).
    fn save(
        &self,
        schema: &StoredSchema,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl CommandExt for db_command::Command<'_> {
    fn save(
        &self,
        schema: &StoredSchema,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use ports::Command as _;
        Ok(self.save_many(std::slice::from_ref(schema))?)
    }
}

/// Standard test result type for integration tests.
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

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
    /// Database instance.
    db: Database,
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
    /// let db = test_db.db();
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn new() -> TestResult<Self> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("lithos.redb");
        let db = Database::open(&db_path)?;
        Ok(Self {
            dir,
            db,
        })
    }

    /// Get reference to the database.
    #[inline]
    #[must_use]
    pub const fn db(&self) -> &Database {
        &self.db
    }

    /// Get the database path for logging/debugging.
    #[inline]
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.dir.path().join("lithos.redb")
    }
}

// ----------------------------------------------------------- //
//                    CQRS Setup Helpers                       //
// ----------------------------------------------------------- //

/// Setup CQRS command and query adapters for a database.
///
/// Returns (command, query) pair for easy destructuring.
///
/// # Examples
/// ```no_run
/// # use tests::common::{setup_cqrs, TestDb, TestResult};
/// # fn test() -> TestResult {
/// let test_db = TestDb::new()?;
/// let (command, query) = setup_cqrs(test_db.db());
/// # Ok(())
/// # }
/// ```
#[track_caller]
#[must_use]
pub fn setup_cqrs(
    db: &Database,
) -> (db_command::Command<'_>, db_query::Query<'_>) {
    let command = db_command::Command::new(db);
    let query = db_query::Query::new(db);
    (command, query)
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
    pub fn build_bool(self) -> TestResult<Property> {
        self.build_with_spec(PropertySpec::Bool(BoolSpec::default()))
    }

    /// Build a string property with custom spec.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[expect(
        dead_code,
        reason = "Will be used in CQRS tests with custom string specs"
    )]
    #[track_caller]
    pub fn build_string(self, spec: StringSpec) -> TestResult<Property> {
        self.build_with_spec(PropertySpec::String(spec))
    }

    /// Build a string property with default spec.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[track_caller]
    pub fn build_string_default(self) -> TestResult<Property> {
        self.build_with_spec(PropertySpec::String(StringSpec::default()))
    }

    /// Build a property with a custom spec.
    ///
    /// # Errors
    /// Returns error if property name is invalid.
    #[track_caller]
    pub fn build_with_spec(self, spec: PropertySpec) -> TestResult<Property> {
        let name = PropertyName::try_new(&self.name)?;
        let id = self.id.unwrap_or_default();
        Ok(Property::new(id, name, self.optionality, self.multiplicity, spec))
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
/// let status = bool_property("is_active")?;
/// # Ok(())
/// # }
/// ```
#[track_caller]
pub fn bool_property(name: &str) -> TestResult<Property> {
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
/// let title = string_property("title")?;
/// # Ok(())
/// # }
/// ```
#[track_caller]
pub fn string_property(name: &str) -> TestResult<Property> {
    PropertyBuilder::new(name).build_string_default()
}

// ----------------------------------------------------------- //
//                    Schema Builders                          //
// ----------------------------------------------------------- //

/// Builder for creating test `StoredSchema` instances.
///
/// Provides a fluent API for constructing schemas with properties and
/// inheritance.
///
/// # Examples
/// ```no_run
/// # use tests::common::{SchemaBuilder, bool_property, TestResult};
/// # fn test() -> TestResult {
/// let prop = bool_property("status")?;
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

/// Assert that two schemas are equal, with detailed error messages.
///
/// Compares:
/// - Name
/// - Parent ID
/// - Property count
/// - Individual properties
///
/// # Panics
/// Panics if schemas are not equal with detailed diff message.
#[expect(dead_code, reason = "Will be used in upcoming CQRS integration tests")]
#[track_caller]
pub fn assert_schema_eq(
    actual: &StoredSchema,
    expected: &StoredSchema,
    context: &str,
) {
    assert_eq!(
        actual.name.as_ref(),
        expected.name.as_ref(),
        "{context}: Schema names should match"
    );
    assert_eq!(
        actual.parent_id, expected.parent_id,
        "{context}: Parent IDs should match"
    );
    assert_eq!(
        actual.properties.len(),
        expected.properties.len(),
        "{context}: Property counts should match"
    );

    // Compare properties (sorted by name for stable comparison)
    let mut actual_props = actual.properties.clone();
    let mut expected_props = expected.properties.clone();
    actual_props.sort_by(|a, b| a.name.cmp(&b.name));
    expected_props.sort_by(|a, b| a.name.cmp(&b.name));

    for (actual_prop, expected_prop) in
        actual_props.iter().zip(expected_props.iter())
    {
        assert_eq!(
            actual_prop.name.as_ref(),
            expected_prop.name.as_ref(),
            "{context}: Property names should match"
        );
        assert_eq!(
            actual_prop.required,
            expected_prop.required,
            "{context}: Property '{}' optionality should match",
            actual_prop.name.as_ref()
        );
        assert_eq!(
            actual_prop.multi,
            expected_prop.multi,
            "{context}: Property '{}' multiplicity should match",
            actual_prop.name.as_ref()
        );
    }
}

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
