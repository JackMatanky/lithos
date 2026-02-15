//! `SchemaResolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent
//! properties, applying excludes, resolving $ref pointers through the
//! `PropertyBank`, and enforcing inheritance ordering.
//!
//! # Design
//!
//! SchemaResolver is a stateful service that encapsulates:
//! - Inheritance graph building and topological sorting
//! - Schema resolution with parent property merging
//! - Internal caching of resolved schemas for parent lookups
//!
//! This unified design prevents invalid usage patterns (can't resolve before
//! ordering) and enables efficient incremental resolution.

use std::collections::{HashMap, HashSet};

use super::{
    aggregate::{
        PropertyBank, ResolutionMetadata, Schema, SchemaHash, SchemaId,
        SchemaName, Timestamp,
    },
    error::SchemaError,
    property::{Cardinality, Multiplicity, Property, PropertyId, PropertyName},
    property_ref::PropertyRef,
    raw::{RawProperty, RawPropertyRef, RawSchema},
};

/// Domain Service: Resolves raw schemas into fully resolved Schema entities.
///
/// `SchemaResolver` is a stateful service that maintains an internal cache of
/// resolved schemas and the inheritance graph. This design prevents invalid
/// usage patterns and enables efficient parent lookups during resolution.
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
/// let mut resolver = SchemaResolver::new(&bank);
///
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     "test".to_owned(),
///     None,
///     HashSet::new(),
///     Vec::new(),
/// );
///
/// let results = resolver.process(vec![raw])?;
/// assert_eq!(results[0].0.name().as_str(), "test");
/// # Ok(())
/// # }
/// ```
pub struct SchemaResolver<'bank> {
    bank: &'bank PropertyBank,
    graph: InheritanceGraph,
    resolved_cache: HashMap<SchemaId, Schema>,
}

/// Result of building the inheritance graph.
///
/// Contains the indexed raw schemas and file modification times
/// (for incremental resolution).
type GraphBuildResult =
    (HashMap<SchemaId, RawSchema>, HashMap<SchemaId, Option<Timestamp>>);

impl<'bank> SchemaResolver<'bank> {
    /// Create a new resolver with a `PropertyBank` reference.
    ///
    /// The resolver maintains internal state (graph and resolved cache) that
    /// persists across resolution calls.
    #[inline]
    #[must_use]
    pub fn new(bank: &'bank PropertyBank) -> Self {
        Self {
            bank,
            graph: InheritanceGraph::new(),
            resolved_cache: HashMap::new(),
        }
    }

    /// Process a set of raw schemas into fully resolved schemas.
    ///
    /// Builds an inheritance graph, determines deterministic resolution order,
    /// and resolves each schema with parent context.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (e.g. cycles, missing parents,
    /// or invalid properties).
    #[inline]
    pub fn process(
        &mut self,
        raw_schemas: Vec<RawSchema>,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError> {
        type NoLoader = fn(&SchemaId) -> Result<Option<Schema>, SchemaError>;

        // Clear previous state
        self.graph = InheritanceGraph::new();
        self.resolved_cache.clear();

        // Build inheritance graph
        let (raw_by_id, file_mtimes) = self.build_graph(raw_schemas, None)?;

        // Compute topological order
        let order = self.compute_resolution_order::<NoLoader>(None)?;

        // Resolve schemas in order
        self.resolve_schemas_in_order::<NoLoader>(
            order,
            raw_by_id,
            &file_mtimes,
            None,
        )
    }

    /// Process only changed schemas (incremental resolution).
    ///
    /// Requires existing metadata for staleness detection and parent hash
    /// lookups. The `parent_loader` function is called to load parent schemas
    /// that are not in the current batch.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails.
    #[inline]
    pub fn process_changed<F>(
        &mut self,
        raw_schemas: Vec<RawSchema>,
        existing_metadata: &[ResolutionMetadata],
        parent_loader: F,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        // Clear previous state
        self.graph = InheritanceGraph::new();
        self.resolved_cache.clear();

        // Build inheritance graph with file modification times
        let (raw_by_id, file_mtimes) =
            self.build_graph(raw_schemas, Some(existing_metadata))?;

        // Compute topological order with external parent loader
        let order = self.compute_resolution_order(Some(&parent_loader))?;

        // Resolve schemas in order with incremental support
        self.resolve_schemas_in_order(
            order,
            raw_by_id,
            &file_mtimes,
            Some(&parent_loader),
        )
    }

    /// Build inheritance graph and index maps from raw schemas.
    ///
    /// Returns (`raw_by_id`, `file_mtimes`) where `file_mtimes` is populated
    /// only if `existing_metadata` is provided (for incremental mode).
    fn build_graph(
        &mut self,
        raw_schemas: Vec<RawSchema>,
        existing_metadata: Option<&[ResolutionMetadata]>,
    ) -> Result<GraphBuildResult, SchemaError> {
        let mut raw_by_id = HashMap::with_capacity(raw_schemas.len());
        let mut name_to_id = HashMap::with_capacity(raw_schemas.len());

        // Extract file modification times from metadata if provided
        let file_mtimes = existing_metadata
            .map(|metadata| {
                metadata
                    .iter()
                    .map(|meta| (meta.schema_id(), meta.file_modified()))
                    .collect()
            })
            .unwrap_or_default();

        // Build graph with SchemaId as primary key
        for raw in raw_schemas {
            let id = SchemaId::from_uuid(raw.id);
            let name = SchemaName::try_from(raw.name.as_str())?;

            // Check for duplicate names
            if name_to_id.contains_key(&name) {
                return Err(SchemaError::AlreadyExists(name.to_string()));
            }

            // Resolve parent ID from parent name (if extends specified)
            let parent_id = match raw.extends.as_ref() {
                Some(parent_name_str) => {
                    let parent_name =
                        SchemaName::try_from(parent_name_str.as_str())?;
                    name_to_id.get(&parent_name).copied()
                }
                None => None,
            };

            self.graph.add_node(id, name.clone(), parent_id);
            name_to_id.insert(name, id);
            raw_by_id.insert(id, raw);
        }

        Ok((raw_by_id, file_mtimes))
    }

    /// Compute topological resolution order from the inheritance graph.
    ///
    /// For incremental mode, provide a `parent_exists_fn` to check if
    /// parent schemas exist externally (not in the current batch).
    fn compute_resolution_order<F>(
        &self,
        parent_exists_fn: Option<&F>,
    ) -> Result<Vec<SchemaId>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        self.graph.resolve_order(|id| {
            parent_exists_fn
                .is_some_and(|check_fn| matches!(check_fn(id), Ok(Some(_))))
        })
    }

    /// Process schemas in topological order, resolving each with parent
    /// context.
    fn resolve_schemas_in_order<F>(
        &mut self,
        order: Vec<SchemaId>,
        mut raw_by_id: HashMap<SchemaId, RawSchema>,
        file_mtimes: &HashMap<SchemaId, Option<Timestamp>>,
        parent_loader: Option<&F>,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let mut resolved = Vec::with_capacity(order.len());

        for id in order {
            let raw = raw_by_id.remove(&id).ok_or_else(|| {
                let name = self
                    .graph
                    .names
                    .get(&id)
                    .map_or("unknown", super::aggregate::SchemaName::as_str);
                SchemaError::NotFound(format!(
                    "Schema definition missing for {name}"
                ))
            })?;

            let parent = self.load_parent(&id, parent_loader)?;
            let schema = self.resolve_single(raw, parent.as_ref())?;
            let metadata =
                self.create_metadata(&schema, parent.as_ref(), file_mtimes);

            self.resolved_cache.insert(schema.id(), schema.clone());
            resolved.push((schema, metadata));
        }

        Ok(resolved)
    }

    /// Load parent schema from cache or external loader.
    ///
    /// Tries the resolved cache first, then falls back to the external
    /// loader if provided (incremental mode).
    fn load_parent<F>(
        &self,
        id: &SchemaId,
        parent_loader: Option<&F>,
    ) -> Result<Option<Schema>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        if let Some(parent_id) = self.graph.edges.get(id).and_then(|p| *p) {
            // Try cache first
            if let Some(p) = self.resolved_cache.get(&parent_id) {
                return Ok(Some(p.clone()));
            }

            // Fall back to external loader if available (incremental mode)
            if let Some(loader) = parent_loader {
                return loader(&parent_id);
            }
        }

        Ok(None)
    }

    /// Create resolution metadata for a schema.
    ///
    /// Includes parent hash, bank version, and file modification time
    /// (if available from incremental mode).
    fn create_metadata(
        &self,
        schema: &Schema,
        parent: Option<&Schema>,
        file_mtimes: &HashMap<SchemaId, Option<Timestamp>>,
    ) -> ResolutionMetadata {
        let parent_hash = parent.map(SchemaHash::compute);
        let file_modified = file_mtimes.get(&schema.id()).copied().flatten();

        ResolutionMetadata::new(
            schema.id(),
            Timestamp::now(),
            parent_hash,
            self.bank.version(),
            file_modified,
        )
    }

    fn merge_parent_properties(
        resolved_props: &mut HashMap<PropertyName, Property>,
        parent: Option<&Schema>,
        excludes: &HashSet<PropertyName>,
    ) {
        if let Some(p) = parent {
            for prop in p.properties() {
                if !excludes.contains(prop.name()) {
                    resolved_props.insert(prop.name().clone(), prop.clone());
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

    /// Resolve a single `RawSchema` into a fully resolved Schema.
    ///
    /// Merges properties from parent, applies excludes, and resolves
    /// references through the `PropertyBank`.
    ///
    /// # Arguments
    /// * `raw` - The raw schema definition.
    /// * `parent` - The fully resolved parent schema (if any).
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (e.g. property not found).
    fn resolve_single(
        &self,
        raw: RawSchema,
        parent: Option<&Schema>,
    ) -> Result<Schema, SchemaError> {
        let mut resolved_props: HashMap<PropertyName, Property> =
            HashMap::new();
        let excludes = Self::parse_excludes(&raw.excludes)?;

        Self::merge_parent_properties(&mut resolved_props, parent, &excludes);
        self.resolve_own_properties(&mut resolved_props, raw.properties)?;

        let mut final_props: Vec<Property> =
            resolved_props.into_values().collect();
        // Sort for determinism
        final_props.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));

        let name = SchemaName::try_from(raw.name.as_str())?;

        // Create the Schema entity using the identity of its raw definition
        Schema::new(SchemaId::from_uuid(raw.id), name, final_props)
    }

    fn resolve_own_properties(
        &self,
        resolved_props: &mut HashMap<PropertyName, Property>,
        raw_properties: Vec<RawProperty>,
    ) -> Result<(), SchemaError> {
        for raw_prop in raw_properties {
            let prop = self.resolve_single_property(raw_prop)?;
            resolved_props.insert(prop.name().clone(), prop);
        }
        Ok(())
    }

    fn resolve_single_property(
        &self,
        raw_prop: RawProperty,
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
                        self.bank.get_by_id(id).cloned().ok_or_else(|| {
                            SchemaError::PropertyRefNotFound(ref_path.clone())
                        })
                    }
                    PropertyRef::ByName(name) => {
                        self.bank.get_by_name(&name).cloned().ok_or_else(|| {
                            SchemaError::PropertyRefNotFound(ref_path.clone())
                        })
                    }
                }
            }
        }
    }
}

/// Internal inheritance graph structure.
///
/// Stores child → parent relationships using `SchemaId` as the primary key,
/// with a separate lookup map for names (used in error messages).
#[derive(Debug, Clone, PartialEq)]
struct InheritanceGraph {
    edges: HashMap<SchemaId, Option<SchemaId>>,
    names: HashMap<SchemaId, SchemaName>,
}

impl Default for InheritanceGraph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl InheritanceGraph {
    #[inline]
    fn add_node(
        &mut self,
        id: SchemaId,
        name: SchemaName,
        extends: Option<SchemaId>,
    ) {
        self.edges.insert(id, extends);
        self.names.insert(id, name);
    }

    #[inline]
    #[must_use]
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
            names: HashMap::new(),
        }
    }

    #[inline]
    fn resolve_order<F>(
        &self,
        external_parent_exists: F,
    ) -> Result<Vec<SchemaId>, SchemaError>
    where
        F: Fn(&SchemaId) -> bool,
    {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Sort by name for determinism
        let mut keys: Vec<_> = self.edges.keys().copied().collect();
        keys.sort_by(|a, b| {
            let name_a = self
                .names
                .get(a)
                .map_or("", super::aggregate::SchemaName::as_str);
            let name_b = self
                .names
                .get(b)
                .map_or("", super::aggregate::SchemaName::as_str);
            name_a.cmp(name_b)
        });

        for id in keys {
            if !visited.contains(&id) {
                self.visit(
                    id,
                    &mut visited,
                    &mut temp_visited,
                    &mut sorted,
                    &external_parent_exists,
                )?;
            }
        }

        Ok(sorted)
    }

    fn validate_not_temporarily_visited(
        &self,
        id: SchemaId,
        temp_visited: &HashSet<SchemaId>,
    ) -> Result<(), SchemaError> {
        if temp_visited.contains(&id) {
            let name = self
                .names
                .get(&id)
                .map_or("unknown", super::aggregate::SchemaName::as_str);
            return Err(SchemaError::CircularInheritance(name.to_owned()));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Recursive graph traversal requires passing state"
    )]
    fn visit<F>(
        &self,
        id: SchemaId,
        visited: &mut HashSet<SchemaId>,
        temp_visited: &mut HashSet<SchemaId>,
        sorted: &mut Vec<SchemaId>,
        external_parent_exists: &F,
    ) -> Result<(), SchemaError>
    where
        F: Fn(&SchemaId) -> bool,
    {
        self.validate_not_temporarily_visited(id, temp_visited)?;

        if visited.contains(&id) {
            return Ok(());
        }

        temp_visited.insert(id);

        self.visit_parent(
            id,
            visited,
            temp_visited,
            sorted,
            external_parent_exists,
        )?;

        temp_visited.remove(&id);
        visited.insert(id);
        sorted.push(id);

        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Recursive graph traversal requires passing state"
    )]
    fn visit_parent<F>(
        &self,
        id: SchemaId,
        visited: &mut HashSet<SchemaId>,
        temp_visited: &mut HashSet<SchemaId>,
        sorted: &mut Vec<SchemaId>,
        external_parent_exists: &F,
    ) -> Result<(), SchemaError>
    where
        F: Fn(&SchemaId) -> bool,
    {
        if let Some(&Some(parent_id)) = self.edges.get(&id) {
            if self.edges.contains_key(&parent_id) {
                self.visit(
                    parent_id,
                    visited,
                    temp_visited,
                    sorted,
                    external_parent_exists,
                )?;
            } else if !external_parent_exists(&parent_id) {
                let parent_name = self
                    .names
                    .get(&parent_id)
                    .map_or("unknown", super::aggregate::SchemaName::as_str);
                return Err(SchemaError::ParentNotFound(
                    parent_name.to_owned(),
                ));
            } else {
                // If external parent exists, we don't visit it (it's assumed
                // resolved externally)
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
            let resolver = SchemaResolver::new(&bank);
            resolver.resolve_single(raw, Some(&parent_schema))
        }

        pub fn resolved_ref_property() -> Result<Property, SchemaError> {
            let property = status_property()?;
            let bank = property_bank_with(property)?;
            let raw = RawProperty::Ref(RawPropertyRef {
                ref_path: "status".to_owned(),
            });
            let resolver = SchemaResolver::new(&bank);
            resolver.resolve_single_property(raw)
        }

        pub fn resolved_schema_with_excludes() -> Result<Schema, SchemaError> {
            let bank = PropertyBank::new();
            let property = excluded_property()?;
            let parent_schema = parent_schema_with_property(property)?;
            let exclude_name = PropertyName::new("p")?;
            let raw = child_raw_schema_with_excludes(&exclude_name);
            let resolver = SchemaResolver::new(&bank);
            resolver.resolve_single(raw, Some(&parent_schema))
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

            let resolver = SchemaResolver::new(&bank);
            let result = resolver.resolve_single_property(raw);

            assert!(
                matches!(result, Err(SchemaError::PropertyRefNotFound(_))),
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

        fn schema_id_from_name(name: &str) -> SchemaId {
            // Generate deterministic ID from name for testing
            let hash = {
                use std::hash::{Hash as _, Hasher as _};
                let mut hasher =
                    std::collections::hash_map::DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish()
            };
            SchemaId::from_uuid(uuid::Uuid::from_u128(u128::from(hash)))
        }

        fn resolve_order(
            graph: &InheritanceGraph,
        ) -> Result<Vec<SchemaId>, SchemaError> {
            graph.resolve_order(|_| false)
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
                let mut name_to_id = std::collections::HashMap::new();

                // First pass: assign IDs
                for name_raw in &unique_names {
                    let name = parse_name(name_raw)?;
                    let id = schema_id_from_name(name_raw);
                    name_to_id.insert(name, id);
                }

                // Second pass: build graph
                for (name_raw, next_raw) in
                    unique_names.iter().zip(unique_names.iter().cycle().skip(1))
                {
                    let name = parse_name(name_raw)?;
                    let id = *name_to_id
                        .get(&name)
                        .expect("name was inserted in first pass");
                    let next_name = parse_name(next_raw)?;
                    let next_id = *name_to_id
                        .get(&next_name)
                        .expect("next_name was inserted in first pass");
                    graph.add_node(id, name, Some(next_id));
                }

                let res = graph.resolve_order(|_| false);
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
                let mut previous: Option<SchemaId> = None;
                for name_str in &unique_names {
                    let name = parse_name(name_str)?;
                    let id = schema_id_from_name(name_str);
                    graph.add_node(id, name, previous);
                    previous = Some(id);
                }

                let order =
                    graph.resolve_order(|_| false).map_err(|error| {
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
            let id_a = schema_id_from_name("a");
            let id_b = schema_id_from_name("b");

            graph.add_node(id_a, a, Some(id_b));
            graph.add_node(id_b, b, Some(id_a));

            let res = graph.resolve_order(|_| false);

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
            let id_child = schema_id_from_name("child");
            let id_parent = schema_id_from_name("parent");

            graph.add_node(id_child, child, Some(id_parent));
            graph.add_node(id_parent, parent, None);

            let order = resolve_order(&graph)?;

            if order == vec![id_parent, id_child] {
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
