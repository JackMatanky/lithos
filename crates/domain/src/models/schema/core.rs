//! Schema aggregate root and resolution services.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Schema prefix is descriptive"
)]

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    sync::OnceLock,
};

use uuid::Uuid;

use crate::{
    errors::DomainError,
    events::{PropertyBankUpdated, SchemaCreated},
    models::schema::{
        property::{Property, PropertyName, RawProperty, RawPropertyRef},
        property_bank::PropertyBank,
    },
};

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9]+(-[a-z0-9]+)*$` (kebab-case)
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::core::SchemaName;
///
/// let name = SchemaName::new("project-note".to_string()).unwrap();
/// assert_eq!(name.as_str(), "project-note");
///
/// let invalid = SchemaName::new("Project_Note".to_string());
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
        Self::validate_non_empty(&name)?;
        Self::validate_length(&name)?;
        Self::validate_format(&name)?;
        Ok(Self(name))
    }

    fn validate_format(name: &str) -> Result<(), DomainError> {
        static SCHEMA_NAME_RE: OnceLock<regex::Regex> = OnceLock::new();
        #[expect(
            clippy::expect_used,
            clippy::disallowed_methods,
            reason = "Standard pattern for hardcoded regexes - Regex is known valid"
        )]
        let re = SCHEMA_NAME_RE.get_or_init(|| {
            regex::Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$")
                .expect("Hardcoded regex is valid")
        });
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
/// Identity is assigned by the adapter layer (e.g. based on file hash or storage key).
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{RawSchema, SchemaName};
/// use std::collections::HashSet;
/// use uuid::Uuid;
///
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("daily-note".into()).unwrap(),
///     Some(SchemaName::new("base-note".into()).unwrap()),
///     HashSet::new(),
///     vec![],
/// );
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: HashSet<PropertyName>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<SchemaName>,
    /// Unique identity for the schema definition.
    pub id: Uuid,
    /// Unique schema name.
    pub name: SchemaName,
    /// List of raw property definitions.
    pub properties: Vec<RawProperty>,
}

impl RawSchema {
    /// Create a new `RawSchema`.
    #[inline]
    #[must_use]
    pub fn new(
        id: Uuid,
        name: SchemaName,
        extends: Option<SchemaName>,
        excludes: HashSet<PropertyName>,
        properties: Vec<RawProperty>,
    ) -> Self {
        Self {
            excludes,
            extends,
            id,
            name,
            properties,
        }
    }
}

/// Schema aggregate defining metadata validation rules (Output).
///
/// Represents a fully resolved schema with no external dependencies.
/// This is the "Truth" used for validation.
///
/// # Invariants
/// - Schema name must be valid kebab-case.
/// - Properties are fully resolved and unique by name.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{Schema, SchemaName};
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
    /// use lithos_domain::models::schema::{Schema, SchemaName};
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
    /// use lithos_domain::models::schema::{Schema, SchemaName};
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

/// Domain Service: Manages schema lineage and resolution order.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{SchemaGraph, SchemaName};
///
/// let mut graph = SchemaGraph::new();
/// let child = SchemaName::new("child".into()).unwrap();
/// let parent = SchemaName::new("parent".into()).unwrap();
///
/// graph.add_node(child.clone(), Some(parent.clone()));
/// graph.add_node(parent.clone(), None);
///
/// let order = graph.resolve_order().unwrap();
/// assert_eq!(order, vec![parent, child]);
/// ```
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
        {
            if self.nodes.contains_key(parent) {
                self.visit(parent, visited, temp_visited, sorted)?;
            } else {
                return Err(DomainError::ParentSchemaNotFound(
                    parent.to_string(),
                ));
            }
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
///
/// Merges parent properties, applies excludes, and resolves `$ref` pointers.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{SchemaResolver, RawSchema, SchemaName};
/// use lithos_domain::models::schema::property_bank::PropertyBank;
/// use std::collections::HashSet;
/// use uuid::Uuid;
///
/// let bank = PropertyBank::new();
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("test".into()).unwrap(),
///     None,
///     HashSet::new(),
///     vec![],
/// );
///
/// let schema = SchemaResolver::resolve(raw, None, &bank).unwrap();
/// assert_eq!(schema.name.as_str(), "test");
/// ```
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

        // Create the Schema entity using the identity of its raw definition
        let (schema, _) = Schema::new(raw.id, raw.name, final_props)?;
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
            crate::models::schema::property::RawProperty::Inline(inline) => {
                let name = PropertyName::new(inline.name)?;
                Ok(Property::new(
                    inline.id,
                    name,
                    inline.required,
                    inline.array,
                    inline.spec,
                )?)
            }
            crate::models::schema::property::RawProperty::Ref(
                RawPropertyRef {
                    ref_path,
                },
            ) => bank
                .get_by_name(&ref_path)
                .cloned()
                .ok_or_else(|| DomainError::PropertyNotFound(ref_path.clone())),
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

    use super::SchemaName;
    use crate::models::schema::property::{
        Property, fixtures::PropertyBuilder,
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
    mod proptests {
        use std::collections::BTreeSet;

        use proptest::prelude::*;

        use super::super::*;

        proptest! {
            /// 3.3-UNIT-018: `schema_graph_detects_arbitrary_cycles`.
            /// Priority: P0.
            #[test]
            #[expect(clippy::indexing_slicing, reason = "Test logic uses indices known to be in bounds")]
            #[expect(clippy::integer_division_remainder_used, reason = "Test logic uses modulo for cycling")]
            #[expect(clippy::arithmetic_side_effects, reason = "Test logic uses safe arithmetic")]
            fn schema_graph_detects_arbitrary_cycles(
                names in prop::collection::vec("[a-z0-9]{3,10}", 2..10)
            ) {
                // GIVEN a set of unique schema names
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
                if unique_names.len() < 2 { return Ok(()); }

                // WHEN creating a circular inheritance graph
                let mut graph = SchemaGraph::new();
                for i in 0..unique_names.len() {
                    let next = (i + 1) % unique_names.len();
                    let name = SchemaName::new(unique_names[i].clone()).unwrap();
                    let next_name = SchemaName::new(unique_names[next].clone()).unwrap();
                    graph.add_node(name, Some(next_name));
                }

                // THEN it must detect the circular inheritance
                let res = graph.resolve_order();
                assert!(matches!(res, Err(DomainError::CircularInheritance(_))));
            }

            /// 3.3-UNIT-019: `schema_graph_accepts_arbitrary_lineage`.
            /// Priority: P1.
            #[test]
            #[expect(clippy::indexing_slicing, reason = "Test logic uses indices known to be in bounds")]
            #[expect(clippy::arithmetic_side_effects, reason = "Test logic uses safe arithmetic")]
            fn schema_graph_accepts_arbitrary_lineage(
                names in prop::collection::vec("[a-z0-9]{3,10}", 1..10)
            ) {
                // GIVEN a set of unique schema names
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();

                // WHEN creating a valid linear inheritance graph
                let mut graph = SchemaGraph::new();
                for i in 0..unique_names.len() {
                    let name = SchemaName::new(unique_names[i].clone()).unwrap();
                    let parent = if i == 0 { None } else { Some(SchemaName::new(unique_names[i-1].clone()).unwrap()) };
                    graph.add_node(name, parent);
                }

                // THEN it must succeed and return the correct order
                let res = graph.resolve_order();
                assert!(res.is_ok());
                if let Ok(order) = res {
                    assert_eq!(order.len(), unique_names.len());
                }
            }
        }
    }

    mod schema_graph {
        use lithos_test_utils::assert_eq_detailed;

        use super::super::*;

        /// 3.3-UNIT-021: `detects_circular_inheritance`.
        /// Priority: P0.
        #[test]
        fn detects_circular_inheritance() {
            // GIVEN a simple circular dependency between two schemas
            let mut graph = SchemaGraph::new();
            graph.add_node(
                "a".try_into().unwrap(),
                Some("b".try_into().unwrap()),
            );
            graph.add_node(
                "b".try_into().unwrap(),
                Some("a".try_into().unwrap()),
            );

            // WHEN resolving the order
            let res = graph.resolve_order();

            // THEN it must return a CircularInheritance error
            assert!(matches!(res, Err(DomainError::CircularInheritance(_))));
        }

        /// 3.3-UNIT-022: `determines_topological_resolution_order`.
        /// Priority: P1.
        #[test]
        fn determines_topological_resolution_order() {
            // GIVEN a linear inheritance: child -> parent
            let mut graph = SchemaGraph::new();
            graph.add_node(
                "child".try_into().unwrap(),
                Some("parent".try_into().unwrap()),
            );
            graph.add_node("parent".try_into().unwrap(), None);

            // WHEN resolving the order
            let order = graph.resolve_order().unwrap();

            // THEN it should return parent before child
            assert_eq_detailed!(
                order,
                vec!["parent".try_into().unwrap(), "child".try_into().unwrap()]
            );
        }
    }
}
