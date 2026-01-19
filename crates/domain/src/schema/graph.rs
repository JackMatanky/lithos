//! `Graph` domain service for inheritance resolution.
//!
//! Provides topological sorting and cycle detection for schema inheritance graphs.

use std::collections::{HashMap, HashSet};

use super::aggregate::SchemaName;
use crate::errors::DomainError;

/// Domain Service: Validates acyclic schema inheritance and determines resolution order.
///
/// Uses topological sorting to ensure parent schemas are resolved before child schemas.
/// Detects circular inheritance dependencies.
///
/// # Examples
///
/// ```
/// use lithos_domain::schema::{SchemaGraph, SchemaName};
///
/// let mut graph = SchemaGraph::new();
/// graph.add_node("child".try_into().unwrap(), Some("parent".try_into().unwrap()));
/// graph.add_node("parent".try_into().unwrap(), None);
///
/// let order = graph.resolve_order().unwrap();
/// assert_eq!(order.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Graph {
    /// Adjacency list: Schema Name -> Parent Name.
    pub nodes: HashMap<SchemaName, Option<SchemaName>>,
}

impl Graph {
    /// Add a schema node to the graph.
    #[inline]
    pub fn add_node(&mut self, name: SchemaName, extends: Option<SchemaName>) {
        self.nodes.insert(name, extends);
    }

    /// Create a new `Graph`.
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

impl Default for Graph {
    #[inline]
    fn default() -> Self {
        Self::new()
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
                names in prop::collection::vec("[a-zA-Z0-9]{3,10}", 2..10)
            ) {
                // GIVEN a set of unique schema names
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
                if unique_names.len() < 2 { return Ok(()); }

                // WHEN creating a circular inheritance graph
                let mut graph = Graph::new();
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
                names in prop::collection::vec("[a-zA-Z0-9]{3,10}", 1..10)
            ) {
                // GIVEN a set of unique schema names
                let unique_names: Vec<_> = names.into_iter().collect::<BTreeSet<_>>().into_iter().collect();

                // WHEN creating a valid linear inheritance graph
                let mut graph = Graph::new();
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

    use lithos_test_utils::assert_eq_detailed;

    use super::*;

    /// 3.3-UNIT-021: `detects_circular_inheritance`.
    /// Priority: P0.
    #[test]
    fn detects_circular_inheritance() {
        // GIVEN a simple circular dependency between two schemas
        let mut graph = Graph::new();
        graph.add_node("a".try_into().unwrap(), Some("b".try_into().unwrap()));
        graph.add_node("b".try_into().unwrap(), Some("a".try_into().unwrap()));

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
        let mut graph = Graph::new();
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
