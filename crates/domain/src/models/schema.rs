//! Schema aggregate root, resolution services, and PropertyBank.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Schema prefix is descriptive"
)]

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

use uuid::Uuid;

use crate::{
    errors::DomainError,
    events::{PropertyBankUpdated, SchemaCreated},
    models::property::{Property, PropertyName, RawProperty, RawPropertyRef},
};

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9]+(-[a-z0-9]+)*$` (kebab-case)
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
        Self::validate_non_empty(&name)?;
        Self::validate_length(&name)?;
        Self::validate_format(&name)?;
        Ok(Self(name))
    }

    fn validate_format(name: &str) -> Result<(), DomainError> {
        let re = regex::Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$")
            .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
        if !re.is_match(name) {
            return Err(DomainError::InvalidSchemaName(name.to_owned()));
        }
        Ok(())
    }

    fn validate_length(name: &str) -> Result<(), DomainError> {
        if name.len() > 64 {
            return Err(DomainError::SchemaNameTooLong(name.len()));
        }
        Ok(())
    }

    fn validate_non_empty(name: &str) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptySchemaName);
        }
        Ok(())
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

/// Domain events for the Schema context.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// Property bank was updated.
    PropertyBankUpdated(PropertyBankUpdated),
    /// Schema was created.
    SchemaCreated(SchemaCreated),
}

/// Raw schema definition (Input).
///
/// Represents the unresolved schema loaded from a file.
/// Contains inheritance pointers and excluded properties that need resolution.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: HashSet<PropertyName>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<SchemaName>,
    /// Unique schema name.
    pub name: SchemaName,
    /// List of raw property definitions.
    pub properties: Vec<RawProperty>,
}

/// Schema aggregate defining metadata validation rules (Output).
///
/// Represents a fully resolved schema with no external dependencies.
/// This is the "Truth" used for validation.
///
/// # Invariants
/// - Schema name must be valid kebab-case.
/// - Properties are fully resolved and unique by name.
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
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(
        id: Uuid,
        name: SchemaName,
        properties: Vec<Property>,
    ) -> Result<(Self, DomainEvent), DomainError> {
        let name_str = name.to_string();
        let schema = Self {
            id,
            name,
            properties,
        };

        let event = DomainEvent::SchemaCreated(SchemaCreated::new(
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
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Index mapping ID -> index in properties vector.
    pub id_index: HashMap<String, usize>,
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

    fn create_updated_event(&self) -> DomainEvent {
        DomainEvent::PropertyBankUpdated(PropertyBankUpdated::new(
            self.properties.len(),
            chrono::Utc::now().timestamp(),
        ))
    }

    /// Decodes a `$ref` path to a Property.
    ///
    /// Format-agnostic: the key is extracted by the adapter.
    ///
    /// # Errors
    /// Returns `PropertyNotFound` if key does not exist.
    #[inline]
    pub fn decode(&self, ref_path: &str) -> Result<&Property, DomainError> {
        self.get_by_name(ref_path)
            .ok_or_else(|| DomainError::PropertyNotFound(ref_path.to_owned()))
    }

    /// Gets a property by name or ID.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Property> {
        // Try by ID first (HashMap lookup is O(1))
        if let Some(prop) = self.get_by_id(key) {
            return Some(prop);
        }
        // Fall back to name lookup (O(1))
        self.get_by_name(key)
    }

    /// Lookup property by ID (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: &str) -> Option<&Property> {
        let &idx = self.id_index.get(id)?;
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
    pub fn has_id(&self, id: &str) -> bool {
        self.id_index.contains_key(id)
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
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn register(
        &mut self,
        property: Property,
    ) -> Result<(usize, DomainEvent), DomainError> {
        property.validate()?;

        // Idempotent success if ID already exists
        if self.id_index.contains_key(&property.id) {
            return Ok((self.properties.len(), self.create_updated_event()));
        }

        // Prevent duplicate names
        self.validate_name_unique(property.name.as_str())?;

        let id = property.id.clone();
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

/// Domain Service: Manages schema lineage and resolution order.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SchemaGraph {
    /// Adjacency list: Schema Name -> Parent Name.
    pub nodes: HashMap<SchemaName, Option<SchemaName>>,
}

impl SchemaGraph {
    /// Add a schema node to the graph.
    #[inline]
    pub fn add_node(&mut self, name: SchemaName, extends: Option<SchemaName>) {
        self.nodes.insert(name, extends);
    }

    /// Create a new `SchemaGraph`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Validate acyclic lineage and return topological resolution order.
    ///
    /// # Returns
    /// A vector of schema names in order (parents before children).
    ///
    /// # Errors
    /// Returns `DomainError::CircularInheritance` if a cycle is detected.
    #[inline]
    pub fn resolve_order(&self) -> Result<Vec<SchemaName>, DomainError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Sort keys for deterministic output
        let mut keys: Vec<_> = self.nodes.keys().cloned().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for name in keys {
            if !visited.contains(&name) {
                self.visit(
                    &name,
                    &mut visited,
                    &mut temp_visited,
                    &mut sorted,
                )?;
            }
        }

        Ok(sorted)
    }

    fn validate_not_temporarily_visited(
        name: &SchemaName,
        temp_visited: &HashSet<SchemaName>,
    ) -> Result<(), DomainError> {
        if temp_visited.contains(name) {
            return Err(DomainError::CircularInheritance(name.to_string()));
        }
        Ok(())
    }

    fn visit(
        &self,
        name: &SchemaName,
        visited: &mut HashSet<SchemaName>,
        temp_visited: &mut HashSet<SchemaName>,
        sorted: &mut Vec<SchemaName>,
    ) -> Result<(), DomainError> {
        Self::validate_not_temporarily_visited(name, temp_visited)?;

        if visited.contains(name) {
            return Ok(());
        }

        temp_visited.insert(name.clone());

        self.visit_parent(name, visited, temp_visited, sorted)?;

        temp_visited.remove(name);
        visited.insert(name.clone());
        sorted.push(name.clone());

        Ok(())
    }

    fn visit_parent(
        &self,
        name: &SchemaName,
        visited: &mut HashSet<SchemaName>,
        temp_visited: &mut HashSet<SchemaName>,
        sorted: &mut Vec<SchemaName>,
    ) -> Result<(), DomainError> {
        if let Some(parent_opt) = self.nodes.get(name)
            && let Some(parent) = parent_opt.as_ref()
            && self.nodes.contains_key(parent)
        {
            self.visit(parent, visited, temp_visited, sorted)?;
        }
        Ok(())
    }
}

impl Default for SchemaGraph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Domain Service: Resolves a raw schema into a final Schema entity.
#[non_exhaustive]
pub struct SchemaResolver;

impl SchemaResolver {
    fn merge_parent_properties(
        resolved_props: &mut HashMap<String, Property>,
        parent: Option<&Schema>,
        excludes: &HashSet<PropertyName>,
    ) {
        if let Some(p) = parent {
            for prop in &p.properties {
                if !excludes.contains(&prop.name) {
                    resolved_props.insert(prop.name.to_string(), prop.clone());
                }
            }
        }
    }

    /// Resolve a `RawSchema` into a fully resolved Schema.
    ///
    /// Merges properties from parent, applies excludes, and resolves references.
    ///
    /// # Arguments
    /// * `raw` - The raw schema definition.
    /// * `parent` - The fully resolved parent schema (if any).
    /// * `bank` - The property bank for resolving references.
    ///
    /// # Errors
    /// Returns `DomainError` if resolution fails (e.g. property not found).
    #[inline]
    pub fn resolve(
        raw: RawSchema,
        parent: Option<&Schema>,
        bank: &PropertyBank,
    ) -> Result<Schema, DomainError> {
        let mut resolved_props = HashMap::new();

        Self::merge_parent_properties(
            &mut resolved_props,
            parent,
            &raw.excludes,
        );
        Self::resolve_own_properties(
            &mut resolved_props,
            raw.properties,
            bank,
        )?;

        let mut final_props: Vec<Property> =
            resolved_props.into_values().collect();
        // Sort for determinism
        final_props.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));

        // Create the Schema entity
        let (schema, _) = Schema::new(Uuid::now_v7(), raw.name, final_props)?;
        Ok(schema)
    }

    fn resolve_own_properties(
        resolved_props: &mut HashMap<String, Property>,
        raw_properties: Vec<RawProperty>,
        bank: &PropertyBank,
    ) -> Result<(), DomainError> {
        for raw_prop in raw_properties {
            let prop = Self::resolve_single_property(raw_prop, bank)?;
            resolved_props.insert(prop.name.to_string(), prop);
        }
        Ok(())
    }

    fn resolve_single_property(
        raw_prop: RawProperty,
        bank: &PropertyBank,
    ) -> Result<Property, DomainError> {
        match raw_prop {
            crate::models::property::RawProperty::Inline(inline) => {
                let name = PropertyName::new(inline.name)?;
                let id = Property::compute_id(name.as_str(), &inline.spec)?;
                Ok(Property::new(
                    id,
                    name,
                    inline.required,
                    inline.array,
                    inline.spec,
                )?)
            }
            crate::models::property::RawProperty::Ref(RawPropertyRef {
                ref_path,
            }) => bank
                .get_by_name(&ref_path)
                .cloned()
                .ok_or_else(|| DomainError::PropertyNotFound(ref_path.clone())),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    mod proptests {
        use std::collections::BTreeSet;

        use proptest::prelude::*;

        use super::super::*;

        proptest! {
            #[test]
            #[expect(clippy::indexing_slicing, reason = "Test logic uses indices known to be in bounds")]
            #[expect(clippy::integer_division_remainder_used, reason = "Test logic uses modulo for cycling")]
            #[expect(clippy::arithmetic_side_effects, reason = "Test logic uses safe arithmetic")]
            fn schema_graph_detects_arbitrary_cycles(
                names in prop::collection::vec("[a-z0-9]{3,10}", 2..10)
            ) {
                // Ensure names are unique to avoid accidental self-cycles or early cycles
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
                if unique_names.len() < 2 { return Ok(()); }

                let mut graph = SchemaGraph::new();
                for i in 0..unique_names.len() {
                    let next = (i + 1) % unique_names.len();
                    // All names match regex because of the proptest strategy
                    let name = SchemaName::new(unique_names[i].clone()).unwrap();
                    let next_name = SchemaName::new(unique_names[next].clone()).unwrap();
                    graph.add_node(name, Some(next_name));
                }

                let res = graph.resolve_order();
                assert!(matches!(res, Err(DomainError::CircularInheritance(_))));
            }

            #[test]
            #[expect(clippy::indexing_slicing, reason = "Test logic uses indices known to be in bounds")]
            #[expect(clippy::arithmetic_side_effects, reason = "Test logic uses safe arithmetic")]
            fn schema_graph_accepts_arbitrary_lineage(
                names in prop::collection::vec("[a-z0-9]{3,10}", 1..10)
            ) {
                // Ensure names are unique to avoid cycles
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();

                let mut graph = SchemaGraph::new();
                for i in 0..unique_names.len() {
                    let name = SchemaName::new(unique_names[i].clone()).unwrap();
                    let parent = if i == 0 { None } else { Some(SchemaName::new(unique_names[i-1].clone()).unwrap()) };
                    graph.add_node(name, parent);
                }

                let res = graph.resolve_order();
                assert!(res.is_ok());
                if let Ok(order) = res {
                    assert_eq!(order.len(), unique_names.len());
                }
            }
        }
    }

    use super::*;
    use crate::models::property::{PropertySpec, StringSpec};

    #[test]
    fn property_bank_indexing() {
        let mut bank = PropertyBank::new();
        let spec = PropertySpec::String(StringSpec::default());
        let name_str = "test".to_owned();
        let name = PropertyName::new(name_str.clone()).unwrap();
        let id = Property::compute_id(&name_str, &spec).unwrap();
        let prop = Property::new(id.clone(), name, false, false, spec).unwrap();

        bank.register(prop).unwrap();

        assert!(bank.get_by_id(&id).is_some());
        assert!(bank.get_by_name("test").is_some());
    }

    #[test]
    fn schema_graph_detects_cycles() {
        let mut graph = SchemaGraph::new();
        graph.add_node("a".try_into().unwrap(), Some("b".try_into().unwrap()));
        graph.add_node("b".try_into().unwrap(), Some("a".try_into().unwrap()));

        let res = graph.resolve_order();
        assert!(matches!(res, Err(DomainError::CircularInheritance(_))));
    }

    #[test]
    fn schema_graph_topological_sort() {
        let mut graph = SchemaGraph::new();
        graph.add_node(
            "child".try_into().unwrap(),
            Some("parent".try_into().unwrap()),
        );
        graph.add_node("parent".try_into().unwrap(), None);

        let order = graph.resolve_order().unwrap();
        assert_eq!(
            order,
            vec!["parent".try_into().unwrap(), "child".try_into().unwrap()]
        );
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

    use super::SchemaName;
    use crate::models::property::{
        Property, PropertyName, PropertySpec, StringSpec,
    };

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
        let name = PropertyName::new("status".to_owned()).expect("Valid Name");
        let id = Property::compute_id(name.as_str(), &spec).expect("Valid ID");
        Property::new(id, name, true, false, spec).expect("Valid property")
    }

    /// Example schema name for testing.
    ///
    /// # Panics
    /// Panics if name is invalid.
    #[inline]
    #[must_use]
    pub fn example_schema_name() -> SchemaName {
        SchemaName::new("test-schema".to_owned()).unwrap()
    }
}
