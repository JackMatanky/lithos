//! Schema domain entities and business logic.
//!
//! This module defines the Schema aggregate root and PropertyBank registry.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
    errors::DomainError,
    events::{PropertyBankUpdated, SchemaCreated},
    models::property::Property,
};

/// Schema aggregate defining metadata validation rules with inheritance support.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// Property names to exclude from parent schema.
    pub excludes: HashSet<String>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<String>,
    /// UUID v7 identity for schema.
    pub id: Uuid,
    /// Unique schema name (e.g., "project-note").
    pub name: String,
    /// Domain events pending emission.
    #[serde(skip)]
    pub pending_events: Vec<DomainEvent>,
    /// Properties directly defined in this schema.
    pub properties: Vec<Property>,
    /// Fully resolved properties after inheritance (computed).
    pub resolved_properties: Vec<Property>,
}

/// Domain events for the Schema context.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// Property bank was updated.
    PropertyBankUpdated(PropertyBankUpdated),
    /// Schema was created.
    SchemaCreated(SchemaCreated),
}

impl Schema {
    /// Resolve property inheritance from parent schema.
    #[inline]
    fn _resolve_properties(
        own_properties: &[Property],
        parent_schema: Option<&Self>,
        excludes: &HashSet<String>,
    ) -> Vec<Property> {
        let mut resolved = Vec::new();

        // 1. Start with parent's resolved properties
        if let Some(parent) = parent_schema {
            for prop in &parent.resolved_properties {
                // 2. Filter out excluded properties
                if !excludes.contains(&prop.name) {
                    resolved.push(prop.clone());
                }
            }
        }

        // 3. Add/Override with own properties
        for prop in own_properties {
            if let Some(pos) = resolved.iter().position(|p| p.name == prop.name)
            {
                if let Some(p) = resolved.get_mut(pos) {
                    *p = prop.clone();
                }
            } else {
                resolved.push(prop.clone());
            }
        }

        resolved
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Create a new schema with inheritance resolution.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails or inheritance resolution fails.
    ///
    /// # Panics
    /// Panics if internal regex fails to compile (should never happen).
    #[inline]
    pub fn new(
        name: String,
        extends: Option<String>,
        excludes: HashSet<String>,
        properties: Vec<Property>,
        parent_schema: Option<&Self>,
    ) -> Result<Self, DomainError> {
        // Validate name
        if name.is_empty() {
            return Err(DomainError::EmptySchemaName);
        }
        if name.len() > 64 {
            return Err(DomainError::SchemaNameTooLong(name.len()));
        }
        let re = regex::Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$")
            .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
        if !re.is_match(&name) {
            return Err(DomainError::InvalidSchemaName(name.clone()));
        }

        // Circular Inheritance Detection
        if let Some(parent) = parent_schema {
            if parent.name == name {
                return Err(DomainError::CircularInheritance(name));
            }
            // Simple DFS check: if parent extends child, it's a cycle
            if parent.extends.as_deref() == Some(&name) {
                return Err(DomainError::CircularInheritance(format!(
                    "{name} -> {}",
                    parent.name
                )));
            }
        }

        // Inheritance resolution
        let resolved_properties =
            Self::_resolve_properties(&properties, parent_schema, &excludes);

        let id = Uuid::now_v7();
        let mut schema = Self {
            excludes,
            extends,
            id,
            name: name.clone(),
            pending_events: Vec::new(),
            properties,
            resolved_properties,
        };

        schema.add_event(DomainEvent::SchemaCreated(SchemaCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(schema)
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

/// Singleton registry of reusable Property definitions.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Domain events pending emission.
    #[serde(skip)]
    pub pending_events: Vec<DomainEvent>,
    /// Map of property ID -> Property.
    pub properties: HashMap<String, Property>,
}

impl PropertyBank {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Get all properties in the bank.
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.values()
    }

    /// Lookup a property by ID.
    #[inline]
    #[must_use]
    pub fn lookup(&self, id: &str) -> Option<&Property> {
        self.properties.get(id)
    }

    /// Lookup a property by name and spec (computes ID internally).
    #[inline]
    #[must_use]
    pub fn lookup_by_definition(
        &self,
        name: &str,
        spec: &crate::models::property::PropertySpec,
    ) -> Option<&Property> {
        let id = Property::compute_id(name, spec);
        self.lookup(&id)
    }

    /// Create a new empty `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Register a property in the bank.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn register(
        &mut self,
        property: Property,
    ) -> Result<&Property, DomainError> {
        property.validate()?;
        let id = property.id.clone();

        // Handle insertion and event emission without borrow conflicts
        self.properties.entry(id.clone()).or_insert(property);

        let count = self.properties.len();
        self.add_event(DomainEvent::PropertyBankUpdated(
            PropertyBankUpdated::new(count, chrono::Utc::now().timestamp()),
        ));

        self.properties.get(&id).ok_or_else(|| {
            DomainError::Unexpected(
                "Failed to retrieve property after registration".to_owned(),
            )
        })
    }

    /// Resolve $ref pointer to Property.
    ///
    /// Format: `#/properties/name`.
    ///
    /// # Errors
    /// Returns `DomainError` if resolution fails.
    #[inline]
    pub fn resolve_ref(
        &self,
        ref_path: &str,
    ) -> Result<&Property, DomainError> {
        let prefix = "#/properties/";
        if !ref_path.starts_with(prefix) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid ref path format: {ref_path}"
            )));
        }
        let name = ref_path.get(prefix.len()..).ok_or_else(|| {
            DomainError::ValidationFailed(format!(
                "Empty property name in ref: {ref_path}"
            ))
        })?;
        self.properties
            .values()
            .find(|p| p.name == name)
            .ok_or_else(|| DomainError::PropertyNotFound(name.to_owned()))
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
    fn validates_schema_name_format() {
        // Valid name
        let res = Schema::new(
            "valid-schema-name".to_owned(),
            None,
            std::collections::HashSet::new(),
            vec![],
            None,
        );
        res.unwrap();

        // Invalid names
        let invalid_names =
            vec!["InvalidName", "invalid_name", "invalid--name", ""];
        for name in invalid_names {
            let res = Schema::new(
                name.to_owned(),
                None,
                std::collections::HashSet::new(),
                vec![],
                None,
            );
            assert!(
                matches!(res, Err(DomainError::InvalidSchemaName(_)))
                    || matches!(res, Err(DomainError::EmptySchemaName))
            );
        }
    }

    #[test]
    fn detects_circular_inheritance() {
        // In a real scenario, this involves a registry, but for the unit test
        // we simulate by passing a parent that already claims to extend the child.
        let parent = Schema {
            excludes: std::collections::HashSet::new(),
            extends: Some("child".to_owned()),
            id: Uuid::now_v7(),
            name: "parent".to_owned(),
            pending_events: Vec::new(),
            properties: vec![],
            resolved_properties: vec![],
        };

        let res = Schema::new(
            "child".to_owned(),
            Some("parent".to_owned()),
            std::collections::HashSet::new(),
            vec![],
            Some(&parent),
        );

        assert!(matches!(res, Err(DomainError::CircularInheritance(_))));
    }

    #[test]
    fn resolves_inheritance_correctly() {
        use crate::models::schema::fixtures::example_property;
        let p1 = example_property(); // "status"
        let parent = Schema {
            excludes: std::collections::HashSet::new(),
            extends: None,
            id: Uuid::now_v7(),
            name: "parent".to_owned(),
            pending_events: Vec::new(),
            properties: vec![p1.clone()],
            resolved_properties: vec![p1.clone()],
        };

        let mut excludes = std::collections::HashSet::new();
        let _: bool = excludes.insert("status".to_owned());

        let child_res = Schema::new(
            "child".to_owned(),
            Some("parent".to_owned()),
            excludes,
            vec![],
            Some(&parent),
        );

        assert!(child_res.is_ok());
        if let Ok(child) = child_res {
            assert!(
                child.resolved_properties.is_empty(),
                "Property 'status' should have been excluded"
            );
        }
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
    fn deduplicates_properties_on_registration() {
        use crate::models::schema::fixtures::example_property;
        let mut bank = PropertyBank::new();
        let prop = example_property();

        bank.register(prop.clone()).unwrap();
        bank.register(prop.clone()).unwrap();

        assert_eq!(bank.all().count(), 1);
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
    fn resolves_refs_correctly() {
        use crate::models::schema::fixtures::example_property;
        let mut bank = PropertyBank::new();
        let prop = example_property(); // name is "status"
        bank.register(prop.clone()).unwrap();

        let res = bank.resolve_ref("#/properties/status");
        assert_eq!(res.unwrap().name, "status");
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Standard in tests")]
    fn emits_events_on_creation() {
        let res = Schema::new(
            "test-schema".to_owned(),
            None,
            std::collections::HashSet::new(),
            vec![],
            None,
        );
        let mut schema = res.unwrap();
        let events = schema.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events.first().unwrap(),
            DomainEvent::SchemaCreated(_)
        ));
    }
}

/// Test fixtures for deterministic schema data.
#[cfg(test)]
pub mod fixtures {
    use uuid::Uuid;

    use crate::models::property::{Property, PropertySpec, StringSpec};

    /// Fixed UUID for deterministic tests.
    pub const TEST_SCHEMA_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0002);

    /// Example property for testing.
    #[inline]
    #[must_use]
    pub fn example_property() -> Property {
        Property {
            array: false,
            id: "test-id".to_owned(),
            name: "status".to_owned(),
            required: true,
            spec: PropertySpec::String(StringSpec::default()),
        }
    }
}
