//! `SchemaResolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent
//! properties, applying excludes, resolving $ref pointers through the
//! `PropertyBank`, and enforcing inheritance ordering.

use std::collections::{HashMap, HashSet};

use super::{
    aggregate::{PropertyBank, Schema, SchemaId, SchemaName},
    error::SchemaError,
    property::{Cardinality, Multiplicity, Property, PropertyId, PropertyName},
    property_ref::PropertyRef,
    raw::{RawProperty, RawPropertyRef, RawSchema},
};

/// Domain Service: Resolves a raw schema into a final Schema entity.
///
/// Merges parent properties, applies excludes, and resolves `$ref` pointers.
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::raw::RawSchema;
/// # use lithos_core::schema::aggregate::PropertyBank;
/// # use lithos_core::schema::resolver::SchemaResolver;
/// # use std::collections::HashSet;
/// # use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let bank = PropertyBank::new();
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     "test".to_owned(),
///     None,
///     HashSet::new(),
///     Vec::new(),
/// );
///
/// let schema = SchemaResolver::resolve(raw, None, &bank)?;
/// assert_eq!(schema.name().as_str(), "test", "Schema name should match");
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
pub struct SchemaResolver;

impl SchemaResolver {
    /// Resolve a set of raw schemas into fully resolved schemas.
    ///
    /// Builds an inheritance graph, determines deterministic resolution order,
    /// and resolves each schema with parent context.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (e.g. cycles, missing parents,
    /// or invalid properties).
    #[inline]
    pub fn resolve_all(
        raw_schemas: Vec<RawSchema>,
        bank: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaError> {
        let mut graph = InheritanceGraph::new();
        let mut raw_by_name = HashMap::with_capacity(raw_schemas.len());

        for raw in raw_schemas {
            let name = SchemaName::try_from(raw.name.as_str())?;
            let extends =
                raw.extends.as_deref().map(SchemaName::try_from).transpose()?;

            if raw_by_name.contains_key(&name) {
                return Err(SchemaError::AlreadyExists(name.to_string()));
            }
            graph.add_node(name.clone(), extends);
            raw_by_name.insert(name, raw);
        }

        let order = graph.resolve_order()?;
        let mut resolved_by_name: HashMap<SchemaName, Schema> =
            HashMap::with_capacity(order.len());
        let mut resolved = Vec::with_capacity(order.len());

        for name in order {
            let raw = raw_by_name.remove(&name).ok_or_else(|| {
                SchemaError::NotFound(format!(
                    "Schema definition missing for {name}"
                ))
            })?;
            let parent_name =
                raw.extends.as_deref().map(SchemaName::try_from).transpose()?;
            let parent = parent_name
                .as_ref()
                .and_then(|parent_key| resolved_by_name.get(parent_key));
            let schema = Self::resolve(raw, parent, bank)?;
            resolved_by_name.insert(schema.name().clone(), schema.clone());
            resolved.push(schema);
        }

        Ok(resolved)
    }

    fn merge_parent_properties(
        resolved_props: &mut HashMap<String, Property>,
        parent: Option<&Schema>,
        excludes: &HashSet<PropertyName>,
    ) {
        if let Some(p) = parent {
            for prop in p.properties() {
                if !excludes.contains(prop.name()) {
                    resolved_props
                        .insert(prop.name().to_string(), prop.clone());
                }
            }
        }
    }

    fn parse_excludes(
        excludes: &HashSet<String>,
    ) -> Result<HashSet<PropertyName>, SchemaError> {
        excludes
            .iter()
            .map(|name| PropertyName::try_from(name.as_str()))
            .collect()
    }

    /// Resolve a `RawSchema` into a fully resolved Schema.
    ///
    /// Merges properties from parent, applies excludes, and resolves
    /// references.
    ///
    /// # Arguments
    /// * `raw` - The raw schema definition.
    /// * `parent` - The fully resolved parent schema (if any).
    /// * `bank` - The property bank for resolving references.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (e.g. property not found).
    #[inline]
    pub fn resolve(
        raw: RawSchema,
        parent: Option<&Schema>,
        bank: &PropertyBank,
    ) -> Result<Schema, SchemaError> {
        let mut resolved_props = HashMap::new();
        let excludes = Self::parse_excludes(&raw.excludes)?;

        Self::merge_parent_properties(&mut resolved_props, parent, &excludes);
        Self::resolve_own_properties(
            &mut resolved_props,
            raw.properties,
            bank,
        )?;

        let mut final_props: Vec<Property> =
            resolved_props.into_values().collect();
        // Sort for determinism
        final_props.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));

        let name = SchemaName::try_from(raw.name.as_str())?;

        // Create the Schema entity using the identity of its raw definition
        Schema::new(SchemaId::from_uuid(raw.id), name, final_props)
    }

    fn resolve_own_properties(
        resolved_props: &mut HashMap<String, Property>,
        raw_properties: Vec<RawProperty>,
        bank: &PropertyBank,
    ) -> Result<(), SchemaError> {
        for raw_prop in raw_properties {
            let prop = Self::resolve_single_property(raw_prop, bank)?;
            resolved_props.insert(prop.name().to_string(), prop);
        }
        Ok(())
    }

    fn resolve_single_property(
        raw_prop: RawProperty,
        bank: &PropertyBank,
    ) -> Result<Property, SchemaError> {
        match raw_prop {
            RawProperty::Inline(inline) => {
                let name = PropertyName::new(&inline.name)?;
                let spec = inline.spec.try_into_validated()?;
                let cardinality = if inline.required {
                    Cardinality::Required
                } else {
                    Cardinality::Optional
                };
                let multiplicity = if inline.array {
                    Multiplicity::Many
                } else {
                    Multiplicity::Single
                };
                Ok(Property::new(
                    PropertyId::from_uuid(inline.id),
                    name,
                    cardinality,
                    multiplicity,
                    spec,
                )?)
            }
            RawProperty::Ref(RawPropertyRef {
                ref_path,
            }) => {
                let prop_ref = PropertyRef::try_from(ref_path.as_str())?;
                match prop_ref {
                    PropertyRef::ById(id) => {
                        bank.get_by_id(id).cloned().ok_or_else(|| {
                            SchemaError::PropertyNotFound(ref_path.clone())
                        })
                    }
                    PropertyRef::ByName(name) => {
                        bank.get_by_name(&name).cloned().ok_or_else(|| {
                            SchemaError::PropertyNotFound(ref_path.clone())
                        })
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InheritanceGraph {
    nodes: HashMap<SchemaName, Option<SchemaName>>,
}

impl Default for InheritanceGraph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl InheritanceGraph {
    #[inline]
    fn add_node(&mut self, name: SchemaName, extends: Option<SchemaName>) {
        self.nodes.insert(name, extends);
    }

    #[inline]
    #[must_use]
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    #[inline]
    fn resolve_order(&self) -> Result<Vec<SchemaName>, SchemaError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

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
    ) -> Result<(), SchemaError> {
        if temp_visited.contains(name) {
            return Err(SchemaError::CircularInheritance(name.to_string()));
        }
        Ok(())
    }

    fn visit(
        &self,
        name: &SchemaName,
        visited: &mut HashSet<SchemaName>,
        temp_visited: &mut HashSet<SchemaName>,
        sorted: &mut Vec<SchemaName>,
    ) -> Result<(), SchemaError> {
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
    ) -> Result<(), SchemaError> {
        if let Some(parent_opt) = self.nodes.get(name)
            && let Some(parent) = parent_opt.as_ref()
        {
            if self.nodes.contains_key(parent) {
                self.visit(parent, visited, temp_visited, sorted)?;
            } else {
                return Err(SchemaError::ParentSchemaNotFound(
                    parent.to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod fixtures {
        use super::*;

        pub fn parent_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("parent")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_PARENT),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn status_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("status")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_STATUS),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn excluded_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("p")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_EXCLUDE),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn parent_schema_with_property(
            property: Property,
        ) -> Result<Schema, SchemaError> {
            let name = SchemaName::new("parent")?;
            Schema::new(SchemaId::from_uuid(TEST_SCHEMA_ID_PARENT), name, vec![
                property,
            ])
        }

        pub fn child_raw_schema() -> RawSchema {
            RawSchema::new(
                TEST_SCHEMA_ID_CHILD,
                "child".to_owned(),
                None,
                HashSet::new(),
                Vec::new(),
            )
        }

        pub fn child_raw_schema_with_excludes(
            exclude_name: &PropertyName,
        ) -> RawSchema {
            let mut excludes = HashSet::new();
            excludes.insert(exclude_name.as_str().to_owned());
            RawSchema::new(
                TEST_SCHEMA_ID_CHILD,
                "child".to_owned(),
                None,
                excludes,
                Vec::new(),
            )
        }

        pub fn property_bank_with(
            property: Property,
        ) -> Result<PropertyBank, SchemaError> {
            let mut bank = PropertyBank::new();
            bank.register(property)?;
            Ok(bank)
        }

        pub fn resolved_schema_with_parent_property()
        -> Result<Schema, SchemaError> {
            let bank = PropertyBank::new();
            let property = parent_property()?;
            let parent_schema = parent_schema_with_property(property.clone())?;
            let raw = child_raw_schema();
            SchemaResolver::resolve(raw, Some(&parent_schema), &bank)
        }

        pub fn resolved_ref_property() -> Result<Property, SchemaError> {
            let property = status_property()?;
            let bank = property_bank_with(property)?;
            let raw = RawProperty::Ref(RawPropertyRef {
                ref_path: "status".to_owned(),
            });
            SchemaResolver::resolve_single_property(raw, &bank)
        }

        pub fn resolved_schema_with_excludes() -> Result<Schema, SchemaError> {
            let bank = PropertyBank::new();
            let property = excluded_property()?;
            let parent_schema = parent_schema_with_property(property)?;
            let exclude_name = PropertyName::new("p")?;
            let raw = child_raw_schema_with_excludes(&exclude_name);
            SchemaResolver::resolve(raw, Some(&parent_schema), &bank)
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn includes_parent_properties() -> Result<(), SchemaError> {
            let schema = fixtures::resolved_schema_with_parent_property()?;
            let name = PropertyName::new("parent")?;
            if schema.has(&name) {
                Ok(())
            } else {
                Err(SchemaError::ValidationFailed(
                    "Resolved schema should include parent property".to_owned(),
                ))
            }
        }

        #[test]
        fn excludes_properties_listed_in_child() -> Result<(), SchemaError> {
            let schema = fixtures::resolved_schema_with_excludes()?;
            let name = PropertyName::new("p")?;
            if schema.has(&name) {
                Err(SchemaError::ValidationFailed(
                    "Resolved schema should exclude child-listed property"
                        .to_owned(),
                ))
            } else {
                Ok(())
            }
        }
    }

    mod resolve_single_property {
        use super::*;

        #[test]
        fn resolves_ref_property_by_plain_name() -> Result<(), SchemaError> {
            let property = fixtures::resolved_ref_property()?;
            if property.name().as_str() == "status" {
                Ok(())
            } else {
                Err(SchemaError::ValidationFailed(
                    "Resolved property name should match".to_owned(),
                ))
            }
        }

        #[test]
        fn returns_error_for_missing_ref() {
            let bank = PropertyBank::new();
            let raw = RawProperty::Ref(RawPropertyRef {
                ref_path: "missing".to_owned(),
            });

            let result = SchemaResolver::resolve_single_property(raw, &bank);

            assert!(
                matches!(result, Err(SchemaError::PropertyNotFound(_))),
                "Missing property reference should be detected, got: \
                 {result:?}"
            );
        }
    }

    mod inheritance_graph {
        use std::collections::BTreeSet;

        use proptest::{
            prelude::*,
            test_runner::{TestCaseError, TestRunner},
        };

        use super::super::{InheritanceGraph, *};

        fn schema_name_literal(lit: &str) -> Result<SchemaName, SchemaError> {
            SchemaName::new(lit)
        }

        fn resolve_order(
            graph: &InheritanceGraph,
        ) -> Result<Vec<SchemaName>, SchemaError> {
            graph.resolve_order()
        }

        /// 3.3-UNIT-018: `schema_graph_detects_arbitrary_cycles`.
        /// Priority: P0.
        #[test]
        fn schema_graph_detects_arbitrary_cycles() -> Result<(), String> {
            let mut runner = TestRunner::deterministic();
            let strategy = prop::collection::vec("[a-z0-9]{3,10}", 2..10);

            let run_result = runner.run(&strategy, |names| {
                let parse_name =
                    |raw: &str| -> Result<SchemaName, TestCaseError> {
                        match SchemaName::new(raw) {
                            Ok(name) => Ok(name),
                            Err(e) => Err(TestCaseError::fail(format!(
                                "Invalid generated schema name: {e}"
                            ))),
                        }
                    };

                let unique_names: Vec<_> = names
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                prop_assume!(unique_names.len() >= 2);

                let mut graph = InheritanceGraph::new();
                for (name_raw, next_raw) in
                    unique_names.iter().zip(unique_names.iter().cycle().skip(1))
                {
                    let name = parse_name(name_raw)?;
                    let next_name = parse_name(next_raw)?;
                    graph.add_node(name, Some(next_name));
                }

                let res = graph.resolve_order();
                prop_assert!(
                    matches!(res, Err(SchemaError::CircularInheritance(_))),
                    "Proptest circular dependency should be detected, got: \
                     {res:?}"
                );
                Ok(())
            });
            run_result.map_err(|e| {
                format!("Deterministic proptest should not fail: {e:?}")
            })?;

            Ok(())
        }

        /// 3.3-UNIT-019: `schema_graph_accepts_arbitrary_lineage`.
        /// Priority: P1.
        #[test]
        fn schema_graph_accepts_arbitrary_lineage() -> Result<(), String> {
            let mut runner = TestRunner::deterministic();
            let strategy = prop::collection::vec("[a-z0-9]{3,10}", 1..10);

            let run_result = runner.run(&strategy, |names| {
                let parse_name =
                    |raw: &str| -> Result<SchemaName, TestCaseError> {
                        match SchemaName::new(raw) {
                            Ok(name) => Ok(name),
                            Err(e) => Err(TestCaseError::fail(format!(
                                "Invalid generated schema name: {e}"
                            ))),
                        }
                    };

                let unique_names: Vec<_> = names
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();

                let mut graph = InheritanceGraph::new();
                let mut previous: Option<SchemaName> = None;
                for name_str in &unique_names {
                    let name = parse_name(name_str)?;
                    graph.add_node(name.clone(), previous.clone());
                    previous = Some(name);
                }

                let order = graph.resolve_order().map_err(|error| {
                    TestCaseError::fail(format!(
                        "Linear graph should resolve successfully: {error}"
                    ))
                })?;
                prop_assert_eq!(
                    order.len(),
                    unique_names.len(),
                    "Resolution order should contain all schemas"
                );
                Ok(())
            });
            run_result.map_err(|e| {
                format!("Deterministic proptest should not fail: {e:?}")
            })?;

            Ok(())
        }

        /// 3.3-UNIT-021: `detects_circular_inheritance`.
        /// Priority: P0.
        #[test]
        fn detects_circular_inheritance() -> Result<(), SchemaError> {
            let mut graph = InheritanceGraph::new();
            let a = schema_name_literal("a")?;
            let b = schema_name_literal("b")?;

            graph.add_node(a.clone(), Some(b.clone()));
            graph.add_node(b, Some(a));

            let res = graph.resolve_order();

            if matches!(res, Err(SchemaError::CircularInheritance(_))) {
                Ok(())
            } else {
                Err(SchemaError::ValidationFailed(format!(
                    "Circular inheritance between schemas should be detected, \
                     got: {res:?}"
                )))
            }
        }

        /// 3.3-UNIT-020: `resolves_empty_graph`.
        /// Priority: P2.
        #[test]
        fn resolves_empty_graph() -> Result<(), SchemaError> {
            let graph = InheritanceGraph::new();

            let order = resolve_order(&graph)?;

            if order.is_empty() {
                Ok(())
            } else {
                Err(SchemaError::ValidationFailed(
                    "Empty graph should return empty resolution order"
                        .to_owned(),
                ))
            }
        }

        /// 3.3-UNIT-022: `determines_topological_resolution_order`.
        /// Priority: P1.
        #[test]
        fn determines_topological_resolution_order() -> Result<(), SchemaError>
        {
            let mut graph = InheritanceGraph::new();
            let child = schema_name_literal("child")?;
            let parent = schema_name_literal("parent")?;

            graph.add_node(child.clone(), Some(parent.clone()));
            graph.add_node(parent.clone(), None);

            let order = resolve_order(&graph)?;

            if order == vec![parent, child] {
                Ok(())
            } else {
                Err(SchemaError::ValidationFailed(
                    "Parent schema should be ordered before child schema"
                        .to_owned(),
                ))
            }
        }
    }

    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::{SchemaId, SchemaName},
        property_spec::{BoolSpec, PropertySpec},
    };

    const TEST_SCHEMA_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0501);
    const TEST_SCHEMA_ID_CHILD: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0502);
    const TEST_PROPERTY_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0503);
    const TEST_PROPERTY_ID_STATUS: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0504);
    const TEST_PROPERTY_ID_EXCLUDE: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0505);
}
