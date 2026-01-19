//! Schema domain aggregates: Schema and `PropertyBank`.
//!
//! This module contains the primary aggregate roots for the Schema bounded context,
//! providing a pure domain representation of schemas and a centralized property registry.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Schema/PropertyBank prefixes are descriptive"
)]

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use uuid::Uuid;

use super::{
    events::{PropertyBankUpdated, SchemaCreated, SchemaEvents},
    property::Property,
};
use crate::{errors::DomainError, validation};

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-zA-Z0-9_-]+$` (alphanumeric, underscores, dashes)
///
/// # Examples
///
/// ```
/// use lithos_domain::schema::SchemaName;
///
/// let name = SchemaName::new("project-note".to_string()).unwrap();
/// assert_eq!(name.as_str(), "project-note");
///
/// let name2 = SchemaName::new("daily_note".to_string()).unwrap();
/// assert_eq!(name2.as_str(), "daily_note");
///
/// let name3 = SchemaName::new("MySchema".to_string()).unwrap();
/// assert_eq!(name3.as_str(), "MySchema");
///
/// let invalid = SchemaName::new("".to_string());
/// assert!(invalid.is_err());
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct SchemaName(String);

impl SchemaName {
    /// Get string reference.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a new `SchemaName` with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptySchemaName);
        }
        if name.len() > 64 {
            return Err(DomainError::SchemaNameTooLong(name.len()));
        }
        if !validation::is_alphanumeric_name(&name) {
            return Err(DomainError::InvalidSchemaName(name));
        }
        Ok(Self(name))
    }
}

impl Display for SchemaName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for SchemaName {
    type Error = DomainError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for SchemaName {
    type Error = DomainError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<SchemaName> for String {
    #[inline]
    fn from(val: SchemaName) -> Self {
        val.0
    }
}

impl AsRef<str> for SchemaName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Schema aggregate defining metadata validation rules (Output).
///
/// Represents a fully resolved schema with no external dependencies.
/// This is the "Truth" used for validation.
///
/// # Invariants
/// - Schema name must be valid alphanumeric/underscore/dash format.
/// - Properties are fully resolved and unique by name.
///
/// # Examples
///
/// ```
/// use lithos_domain::schema::{Schema, SchemaName};
/// use uuid::Uuid;
///
/// let name = SchemaName::new("project-note".into()).unwrap();
/// let (schema, _) = Schema::new(Uuid::now_v7(), name, vec![]).unwrap();
/// assert!(schema.properties.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// UUID v7 identity for schema.
    pub id: Uuid,
    /// Unique schema name.
    pub name: SchemaName,
    /// Fully resolved properties after inheritance.
    pub properties: Vec<Property>,
}

impl Schema {
    /// Gets a property by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::schema::{Schema, SchemaName};
    /// use uuid::Uuid;
    ///
    /// let name = SchemaName::new("test".into()).unwrap();
    /// let (schema, _) = Schema::new(Uuid::now_v7(), name, vec![]).unwrap();
    /// assert!(schema.get("missing").is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name.as_str() == name)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.properties.iter().any(|p| p.name.as_str() == name)
    }

    /// Create a new resolved Schema.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::schema::{Schema, SchemaName};
    /// use uuid::Uuid;
    ///
    /// let name = SchemaName::new("project-note".to_string()).unwrap();
    /// let (schema, event) = Schema::new(Uuid::now_v7(), name, vec![]).unwrap();
    /// assert_eq!(schema.name.as_str(), "project-note");
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(
        id: Uuid,
        name: SchemaName,
        properties: Vec<Property>,
    ) -> Result<(Self, SchemaEvents), DomainError> {
        let name_str = name.to_string();
        let schema = Self {
            id,
            name,
            properties,
        };

        let event = SchemaEvents::SchemaCreated(SchemaCreated::new(
            id,
            name_str,
            chrono::Utc::now().timestamp(),
        ));

        Ok((schema, event))
    }
}

/// Registry of reusable Property definitions with dual indexing.
///
/// Provides O(1) lookup by ID and Name.
///
/// # Examples
///
/// ```
/// use lithos_domain::schema::PropertyBank;
/// use lithos_domain::schema::{Property, PropertyName};
/// use lithos_domain::schema::{PropertySpec, BoolSpec};
/// use uuid::Uuid;
///
/// let mut bank = PropertyBank::new();
/// let name = PropertyName::new("is_active".to_string()).unwrap();
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let id = Uuid::now_v7();
/// let property = Property::new(id, name, true, false, spec).unwrap();
///
/// bank.register(property).unwrap();
/// assert!(bank.has_name("is_active"));
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Index mapping ID -> index in properties vector.
    pub id_index: HashMap<Uuid, usize>,
    /// Index mapping Name -> index in properties vector.
    pub name_index: HashMap<String, usize>,
    /// Dense storage of properties.
    pub properties: Vec<Property>,
}

impl PropertyBank {
    /// Get all properties in the bank.
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.iter()
    }

    fn create_updated_event(&self) -> SchemaEvents {
        SchemaEvents::PropertyBankUpdated(PropertyBankUpdated::new(
            self.properties.len(),
            chrono::Utc::now().timestamp(),
        ))
    }

    /// Decodes a `$ref` path to a Property.
    ///
    /// This method performs a key lookup for a property. Format-specific parsing
    /// (e.g., handling "#/properties/") must be handled by the adapters.
    ///
    /// # Errors
    /// Returns `PropertyNotFound` if key does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::schema::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let result = bank.decode("missing");
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn decode(&self, key: &str) -> Result<&Property, DomainError> {
        // Try parsing key as UUID first
        if let Ok(id) = Uuid::parse_str(key)
            && let Some(prop) = self.get_by_id(id)
        {
            return Ok(prop);
        }
        // Fall back to name lookup
        self.get_by_name(key)
            .ok_or_else(|| DomainError::PropertyNotFound(key.to_owned()))
    }

    /// Gets a property by name or ID (string).
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::schema::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// assert!(bank.get("any").is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Property> {
        // Try by ID first
        if let Ok(id) = Uuid::parse_str(key)
            && let Some(prop) = self.get_by_id(id)
        {
            return Some(prop);
        }
        // Fall back to name lookup
        self.get_by_name(key)
    }

    /// Lookup property by ID (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: Uuid) -> Option<&Property> {
        let &idx = self.id_index.get(&id)?;
        self.properties.get(idx)
    }

    /// Lookup property by Name (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&Property> {
        let &idx = self.name_index.get(name)?;
        self.properties.get(idx)
    }

    /// Checks if a property exists by ID.
    #[inline]
    #[must_use]
    pub fn has_id(&self, id: Uuid) -> bool {
        self.id_index.contains_key(&id)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has_name(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Create a new empty `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a property in the bank.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::schema::PropertyBank;
    /// use lithos_domain::schema::{Property, PropertyName};
    /// use lithos_domain::schema::{PropertySpec, BoolSpec};
    /// use uuid::Uuid;
    ///
    /// let mut bank = PropertyBank::new();
    /// let name = PropertyName::new("is_active".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Uuid::now_v7();
    /// let property = Property::new(id, name, true, false, spec).unwrap();
    ///
    /// let (count, event) = bank.register(property).unwrap();
    /// assert_eq!(count, 1);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn register(
        &mut self,
        property: Property,
    ) -> Result<(usize, SchemaEvents), DomainError> {
        property.validate()?;

        // Idempotent success if ID already exists
        if self.id_index.contains_key(&property.id) {
            return Ok((self.properties.len(), self.create_updated_event()));
        }

        // Prevent duplicate names
        self.validate_name_unique(property.name.as_str())?;

        let id = property.id;
        let name = property.name.to_string();
        let idx = self.properties.len();

        self.id_index.insert(id, idx);
        self.name_index.insert(name, idx);
        self.properties.push(property);

        Ok((self.properties.len(), self.create_updated_event()))
    }

    fn validate_name_unique(&self, name: &str) -> Result<(), DomainError> {
        if self.name_index.contains_key(name) {
            return Err(DomainError::DuplicatePropertyName(name.to_owned()));
        }
        Ok(())
    }
}

/// Test fixtures for deterministic schema data.
#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test fixtures use expect for deterministic setup"
)]
pub mod fixtures {
    use uuid::Uuid;

    use super::{
        super::property::{Property, fixtures::PropertyBuilder},
        SchemaName,
    };

    /// Fixed UUID for deterministic tests.
    pub const TEST_SCHEMA_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0002);

    /// Example property for testing.
    #[inline]
    #[must_use]
    pub fn example_property() -> Property {
        PropertyBuilder::new().name("status").required(true).build()
    }

    /// Example schema name for testing.
    ///
    /// # Panics
    /// Panics if the default schema name is invalid.
    #[inline]
    #[must_use]
    pub fn example_schema_name() -> SchemaName {
        SchemaName::new("test-schema".to_owned()).expect("Valid default name")
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    use uuid::Uuid;

    use super::{
        super::{
            property::{Property, PropertyName},
            property_spec::{BoolSpec, PropertySpec, StringSpec},
        },
        *,
    };

    /// 3.3-UNIT-023: `is_idempotent_on_identical_registration`.
    /// Priority: P1.
    #[test]
    fn is_idempotent_on_identical_registration() {
        // GIVEN a PropertyBank and an existing property
        let mut bank = PropertyBank::new();
        let spec = PropertySpec::String(StringSpec::default());
        let name = PropertyName::new("test".to_owned()).unwrap();
        let prop =
            Property::new(Uuid::now_v7(), name, false, false, spec).unwrap();

        // WHEN registering the same property twice
        bank.register(prop.clone()).unwrap();
        let (count, _) = bank.register(prop).unwrap();

        // THEN the count remains 1
        assert_eq!(count, 1);
        assert_eq!(bank.all().count(), 1);
    }

    /// 3.3-UNIT-020: `maintains_dual_indices_for_fast_lookup`.
    /// Priority: P1.
    #[test]
    fn maintains_dual_indices_for_fast_lookup() {
        // GIVEN a PropertyBank and a Property definition
        let mut bank = PropertyBank::new();
        let spec = PropertySpec::String(StringSpec::default());
        let name_str = "test".to_owned();
        let name = PropertyName::new(name_str.clone()).unwrap();
        let id = Uuid::now_v7();
        let prop = Property::new(id, name, false, false, spec).unwrap();

        // WHEN registering the property
        bank.register(prop).unwrap();

        // THEN it should be accessible by both ID and name
        assert!(bank.get_by_id(id).is_some());
        assert!(bank.get_by_name("test").is_some());
    }

    /// 3.3-UNIT-024: `rejects_duplicate_names_with_different_definitions`.
    /// Priority: P1.
    #[test]
    fn rejects_duplicate_names_with_different_definitions() {
        // GIVEN a PropertyBank with a registered property
        let mut bank = PropertyBank::new();
        let spec1 = PropertySpec::String(StringSpec::default());
        let name = PropertyName::new("test".to_owned()).unwrap();
        let prop1 =
            Property::new(Uuid::now_v7(), name.clone(), false, false, spec1)
                .unwrap();
        bank.register(prop1).unwrap();

        // WHEN registering a different definition with the same name
        let spec2 = PropertySpec::Bool(BoolSpec::default());
        let prop2 =
            Property::new(Uuid::now_v7(), name, false, false, spec2).unwrap();
        let res = bank.register(prop2);

        // THEN it must return a DuplicatePropertyName error
        assert!(matches!(res, Err(DomainError::DuplicatePropertyName(_))));
    }
}
