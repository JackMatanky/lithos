//! Schema aggregate root and domain events.

use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    errors::DomainError,
    events::{PropertyBankUpdated, SchemaCreated},
    models::schema::property::Property,
};

/// Schema aggregate defining metadata validation rules with inheritance support.
///
/// # Invariants
/// - Schema name must be non-empty, lowercase-with-hyphens, max 64 chars.
/// - Circular inheritance is prohibited.
/// - Property names must be unique after inheritance resolution.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::Schema;
/// use uuid::Uuid;
/// use std::collections::HashSet;
///
/// let schema = Schema::new(
///     Uuid::now_v7(),
///     "project-note".to_string(),
///     None,
///     HashSet::new(),
///     vec![],
///     None,
/// ).expect("Valid schema");
///
/// assert_eq!(schema.name, "project-note");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// Ordered list of all ancestor schema names (inheritance chain).
    pub ancestry: Vec<String>,
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
        id: Uuid,
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

        let mut ancestry = Vec::new();

        // Circular Inheritance Detection using Ancestry Chain
        if let Some(parent) = parent_schema {
            if parent.name == name || parent.ancestry.contains(&name) {
                return Err(DomainError::CircularInheritance(format!(
                    "{name} -> {}",
                    parent.name
                )));
            }
            // Build ancestry chain: [parent, grandparent, ...]
            ancestry.push(parent.name.clone());
            ancestry.extend(parent.ancestry.clone());
        }

        // Inheritance resolution
        let resolved_properties =
            Self::_resolve_properties(&properties, parent_schema, &excludes);

        let mut schema = Self {
            ancestry,
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    mod new {
        use uuid::Uuid;

        use super::super::*;

        /// 3.3-UNIT-001: `returns_error_when_name_format_is_invalid`.
        #[test]
        fn returns_error_when_name_format_is_invalid() {
            // Invalid names
            let invalid_names =
                vec!["InvalidName", "invalid_name", "invalid--name", ""];
            for name in invalid_names {
                let res = Schema::new(
                    Uuid::now_v7(),
                    name.to_owned(),
                    None,
                    std::collections::HashSet::new(),
                    vec![],
                    None,
                );
                assert!(
                    matches!(res, Err(DomainError::InvalidSchemaName(_)))
                        || matches!(res, Err(DomainError::EmptySchemaName)),
                    "Failed to reject invalid name: {name}"
                );
            }
        }

        /// 3.3-UNIT-002: `returns_error_when_circular_inheritance_is_detected`.
        #[test]
        fn returns_error_when_circular_inheritance_is_detected() {
            let parent = Schema {
                ancestry: vec!["child".to_owned()],
                excludes: std::collections::HashSet::new(),
                extends: Some("child".to_owned()),
                id: Uuid::now_v7(),
                name: "parent".to_owned(),
                pending_events: Vec::new(),
                properties: vec![],
                resolved_properties: vec![],
            };

            let res = Schema::new(
                Uuid::now_v7(),
                "child".to_owned(),
                Some("parent".to_owned()),
                std::collections::HashSet::new(),
                vec![],
                Some(&parent),
            );

            assert!(
                matches!(res, Err(DomainError::CircularInheritance(_))),
                "Failed to detect circular inheritance"
            );
        }

        /// 3.3-UNIT-003: `emits_schema_created_event_on_success`.
        #[test]
        fn emits_schema_created_event_on_success() {
            let mut schema = Schema::new(
                Uuid::now_v7(),
                "test-schema".to_owned(),
                None,
                std::collections::HashSet::new(),
                vec![],
                None,
            )
            .unwrap();
            let events = schema.take_events();
            assert_eq!(events.len(), 1, "Expected exactly 1 event");
            assert!(
                matches!(
                    events.first().unwrap(),
                    DomainEvent::SchemaCreated(_)
                ),
                "Event should be SchemaCreated"
            );
        }
    }

    mod resolve_properties {
        use super::super::*;
        use crate::models::schema::fixtures::example_property;

        /// 3.3-UNIT-004: `resolves_inheritance_with_excludes_correctly`.
        #[test]
        fn resolves_inheritance_with_excludes_correctly() {
            let p1 = example_property(); // "status"
            let parent = Schema {
                ancestry: Vec::new(),
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

            let child = Schema::new(
                Uuid::now_v7(),
                "child".to_owned(),
                Some("parent".to_owned()),
                excludes,
                vec![],
                Some(&parent),
            )
            .unwrap();

            assert!(
                child.resolved_properties.is_empty(),
                "Property 'status' should have been excluded from resolved properties"
            );
        }
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

    use crate::models::schema::property::{Property, PropertySpec, StringSpec};

    /// Fixed UUID for deterministic tests.
    pub const TEST_SCHEMA_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0002);

    /// Example property for testing.
    ///
    /// # Panics
    /// Panics if ID computation fails.
    #[inline]
    #[must_use]
    pub fn example_property() -> Property {
        let spec = PropertySpec::String(StringSpec::default());
        let id = Property::compute_id("status", &spec).expect("Valid ID");
        Property {
            array: false,
            id,
            name: "status".to_owned(),
            required: true,
            spec,
        }
    }
}
