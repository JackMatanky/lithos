//! Schema aggregate and identifier types.
//!
//! Core aggregate and value objects for the schema domain:
//! - [`Schema`] - Main schema aggregate (previously `StoredSchema`)
//! - [`SchemaId`] - UUID-based schema identifier
//! - [`SchemaName`] - Validated schema name value object
//!
//! ## Schema Aggregate
//!
//! The `Schema` type is the main domain aggregate for the schema system.
//! It represents a fully resolved schema with all properties merged from
//! parent schemas according to inheritance rules.
//!
//! ## Architecture Notes
//!
//! This module follows the unified Repository pattern:
//! - Files are the source of truth
//! - Domain types are used as storage shape (no separate view types unless
//!   profiling shows need)
//! - `recorded_at` field is private (ingestion metadata, not exposed in public
//!   API)

use std::{
    borrow::Borrow, collections::HashMap, fmt::Display, sync::LazyLock,
    time::SystemTime,
};

use regex::Regex;
use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};
use uuid::Uuid;

use super::{
    error::SchemaError,
    property::{Property, PropertyId, PropertyName},
};

// ============================================================================
// Schema Aggregate
// ============================================================================

/// Main schema aggregate.
///
/// Represents a fully resolved schema with all properties merged from parent
/// schemas. This is the primary domain type used throughout the schema system.
///
/// ## Fields
///
/// - `id`: Unique schema identifier
/// - `name`: Validated schema name
/// - `parent_id`: Optional parent schema for inheritance
/// - `children`: Child schema IDs (for fast inheritance traversal)
/// - `properties`: Resolved properties after inheritance merge
/// - `recorded_at`: Ingestion timestamp (**private** - not part of public API)
///
/// ## Storage
///
/// Persisted to the `schema_by_id` table using `rkyv` serialization.
/// The domain type serves as the storage shape.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
///
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("project-note")?;
/// let schema = Schema::new(id, name, None, vec![], HashMap::new());
/// assert_eq!(schema.id(), &id);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// Schema identity.
    id: SchemaId,
    /// Schema name.
    name: SchemaName,
    /// Parent schema ID, for inheritance.
    parent_id: Option<SchemaId>,
    /// Child schema IDs (for fast inheritance traversal).
    ///
    /// Stores IDs only. Full relationship metadata (extends/excludes) is
    /// managed via inheritance views in the repository layer.
    children: Vec<SchemaId>,
    /// Resolved properties (`HashMap` for O(1) lookup by name).
    properties: HashMap<PropertyName, Property>,
    /// Ingestion timestamp (private - not exposed in public API).
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl Schema {
    /// Creates a new `Schema` with current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    ///
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("note")?;
    /// let schema = Schema::new(id, name, None, vec![], HashMap::new());
    /// assert_eq!(schema.id(), &id);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: SchemaId,
        name: SchemaName,
        parent_id: Option<SchemaId>,
        children: Vec<SchemaId>,
        properties: HashMap<PropertyName, Property>,
    ) -> Self {
        Self {
            id,
            name,
            parent_id,
            children,
            properties,
            recorded_at: SystemTime::now(),
        }
    }

    /// Returns the schema ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &SchemaId {
        &self.id
    }

    /// Returns the schema name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the parent schema ID, if any.
    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<&SchemaId> {
        self.parent_id.as_ref()
    }

    /// Returns the child schema IDs.
    ///
    /// This provides fast access to direct children for inheritance traversal.
    /// Full relationship metadata (extends/excludes) is managed separately
    /// via inheritance views in the repository layer.
    #[inline]
    #[must_use]
    pub fn children(&self) -> &[SchemaId] {
        &self.children
    }

    /// Returns the resolved properties.
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &HashMap<PropertyName, Property> {
        &self.properties
    }

    /// Finds a property by name (O(1) lookup).
    #[inline]
    #[must_use]
    pub fn find_property_by_name(
        &self,
        name: &PropertyName,
    ) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Finds a property by ID (O(n) - iterates all properties).
    #[inline]
    #[must_use]
    pub fn find_property(&self, id: &PropertyId) -> Option<&Property> {
        self.properties.values().find(|p| p.id() == *id)
    }
}

// ============================================================================
// Schema Identifier
// ============================================================================

/// Unique identifier for a schema.
///
/// Wraps a UUID v7 (time-ordered) identifier for schemas.
///
/// # Examples
/// ```
/// use lithos_core::schema::aggregate::SchemaId;
///
/// let id = SchemaId::new();
/// let uuid = id.into_uuid();
/// ```
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SchemaId(Uuid);

impl SchemaId {
    /// Creates a new UUID v7-based `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wraps a UUID into a `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::now_v7();
    /// let id = SchemaId::from_uuid(uuid);
    /// assert_eq!(*id.as_uuid(), uuid);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the inner UUID by value.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _uuid = id.into_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for SchemaId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SchemaId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Schema Name
// ============================================================================

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$` (lowercase alphanumeric, underscores,
///   dashes)
///
/// # Examples
///
/// ```
/// use lithos_core::schema::aggregate::SchemaName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::try_new("project-note")?;
/// assert_eq!(name.as_str(), "project-note", "Schema name should match");
///
/// let name2 = SchemaName::try_new("daily_note")?;
/// assert_eq!(name2.as_str(), "daily_note", "Schema name should match");
///
/// let name3 = SchemaName::try_new("myschema")?;
/// assert_eq!(name3.as_str(), "myschema", "Schema name should match");
///
/// let invalid = SchemaName::try_new("");
/// assert!(invalid.is_err(), "Empty name should be rejected");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaName(Box<str>);

impl SchemaName {
    /// Schema name validation pattern: lowercase letters, numbers, underscores,
    /// and hyphens.
    ///
    /// Pattern: `^[a-z0-9_-]+$`.
    ///
    /// # Examples
    /// - Valid: `daily-note`, `project_schema`, `schema123`
    /// - Invalid: `MySchema`, `invalid name`, `name!`
    const PATTERN: &'static str = "^[a-z0-9_-]+$";

    /// Create a new `SchemaName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::try_new("test")?;
    /// assert_eq!(name.as_str(), "test");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(name.into()))
    }

    #[inline]
    fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(SchemaName::PATTERN));

        if name.is_empty() {
            return Err(super::error::SchemaNameError::Empty.into());
        }
        if name.len() > 64 {
            return Err(super::error::SchemaNameError::TooLong {
                len: name.len(),
                max: 64,
            }
            .into());
        }

        let re = RE.as_ref().map_err(|error| {
            super::error::SchemaValidationError::SchemaNameRegex {
                reason: error.to_string().into(),
            }
        })?;

        if !re.is_match(name) {
            return Err(super::error::SchemaNameError::InvalidFormat {
                name: name.into(),
            }
            .into());
        }
        Ok(())
    }

    /// Returns the inner string slice.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::try_new("project")?;
    /// assert_eq!(name.as_str(), "project");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SchemaName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SchemaName {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for SchemaName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SchemaName> for String {
    #[inline]
    fn from(val: SchemaName) -> Self {
        val.0.into()
    }
}

impl TryFrom<&str> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

impl TryFrom<Box<str>> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========== SchemaId Tests ==========

    #[test]
    fn schema_id_new_creates_unique_ids() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();
        assert_ne!(id1, id2, "Each call to new() should create a unique ID");
    }

    #[test]
    fn schema_id_roundtrip() {
        let uuid = Uuid::now_v7();
        let id = SchemaId::from_uuid(uuid);
        assert_eq!(id.into_uuid(), uuid);
    }

    // ========== SchemaName Tests ==========

    #[test]
    fn schema_name_valid() {
        SchemaName::try_new("test").unwrap();
        SchemaName::try_new("test-name").unwrap();
        SchemaName::try_new("test_name").unwrap();
        SchemaName::try_new("test123").unwrap();
        SchemaName::try_new("a").unwrap();
    }

    #[test]
    fn schema_name_empty_rejected() {
        SchemaName::try_new("").unwrap_err();
    }

    #[test]
    fn schema_name_too_long_rejected() {
        let long_name = "a".repeat(65);
        SchemaName::try_new(&long_name).unwrap_err();
    }

    #[test]
    fn schema_name_invalid_chars_rejected() {
        SchemaName::try_new("Test").unwrap_err(); // Uppercase
        SchemaName::try_new("test name").unwrap_err(); // Space
        SchemaName::try_new("test!").unwrap_err(); // Special char
        SchemaName::try_new("test.name").unwrap_err(); // Period
    }

    #[test]
    fn schema_name_as_str() {
        let name = SchemaName::try_new("test").unwrap();
        assert_eq!(name.as_str(), "test");
        assert_eq!(name.as_ref(), "test");
    }

    // ========== Schema Tests ==========

    #[test]
    fn schema_new_creates_with_empty_properties() {
        let id = SchemaId::new();
        let name = SchemaName::try_new("test").unwrap();
        let schema =
            Schema::new(id, name.clone(), None, vec![], HashMap::new());

        assert_eq!(schema.id(), &id);
        assert_eq!(schema.name(), &name);
        assert_eq!(schema.parent_id(), None);
        assert!(schema.properties().is_empty());
    }

    #[test]
    fn schema_accessors_work() {
        let id = SchemaId::new();
        let parent_id = SchemaId::new();
        let name = SchemaName::try_new("child-schema").unwrap();
        let schema = Schema::new(
            id,
            name.clone(),
            Some(parent_id),
            vec![],
            HashMap::new(),
        );

        assert_eq!(schema.id(), &id);
        assert_eq!(schema.name(), &name);
        assert_eq!(schema.parent_id(), Some(&parent_id));
    }
}
