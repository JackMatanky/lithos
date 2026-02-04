//! `Graph` domain service for inheritance resolution.
//!
//! Provides topological sorting and cycle detection for schema inheritance
//! graphs.

use std::collections::{HashMap, HashSet};

use super::{aggregate::SchemaName, error::SchemaError};

/// Domain Service: Validates acyclic schema inheritance and determines
/// resolution order.
///
/// Uses topological sorting to ensure parent schemas are resolved before child
/// schemas. Detects circular inheritance dependencies.
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::graph::Graph;
/// # use lithos_core::schema::aggregate::SchemaName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let mut graph = Graph::new();
/// graph.add_node("child".try_into()?, Some("parent".try_into()?));
/// graph.add_node("parent".try_into()?, None);
///
/// let order = graph.resolve_order()?;
/// assert_eq!(order.len(), 2);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Graph {
    /// Adjacency list: Schema Name -> Parent Name.
    pub nodes: HashMap<SchemaName, Option<SchemaName>>,
}

impl Default for Graph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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
    /// Returns `SchemaError::CircularInheritance` if a cycle is detected.
    #[inline]
    pub fn resolve_order(&self) -> Result<Vec<SchemaName>, SchemaError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Sort keys for deterministic output
        let mut keys: Vec<_> = self.nodes.keys().cloned().collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0));

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
    mod proptests {
        use std::collections::BTreeSet;

        use proptest::{
            prelude::*,
            test_runner::{TestCaseError, TestRunner},
        };

        use super::super::*;

        /// 3.3-UNIT-018: `schema_graph_detects_arbitrary_cycles`.
        /// Priority: P0.
        #[test]
        #[expect(
            clippy::indexing_slicing,
            clippy::integer_division_remainder_used,
            reason = "Test uses index-based collection access and modulo \
                      arithmetic for circular graph traversal. Index safety \
                      is guaranteed by loop bounds over `unique_names` length."
        )]
        fn schema_graph_detects_arbitrary_cycles() {
            let mut runner = TestRunner::deterministic();
            let strategy = prop::collection::vec("[a-zA-Z0-9]{3,10}", 2..10);

            let run_result = runner.run(&strategy, |names| {
                let parse_name =
                    |raw: String| -> Result<SchemaName, TestCaseError> {
                        match SchemaName::new(raw) {
                            Ok(name) => Ok(name),
                            Err(e) => Err(TestCaseError::fail(format!(
                                "Invalid generated schema name: {e}"
                            ))),
                        }
                    };

                // GIVEN: a set of unique schema names
                let unique_names: Vec<_> = names
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                if unique_names.len() < 2 {
                    return Ok(());
                }

                // WHEN: creating a circular inheritance graph
                let mut graph = Graph::new();
                for i in 0..unique_names.len() {
                    let next = (i + 1) % unique_names.len();
                    let name = parse_name(unique_names[i].clone())?;
                    let next_name = parse_name(unique_names[next].clone())?;
                    graph.add_node(name, Some(next_name));
                }

                // THEN: it must detect the circular inheritance
                let res = graph.resolve_order();
                assert!(
                    matches!(res, Err(SchemaError::CircularInheritance(_))),
                    "Proptest circular dependency should be detected, got: \
                     {res:?}"
                );
                Ok(())
            });
            assert!(
                run_result.is_ok(),
                "Deterministic proptest should not fail: {run_result:?}"
            );
        }

        /// 3.3-UNIT-019: `schema_graph_accepts_arbitrary_lineage`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::indexing_slicing,
            reason = "Test uses index-based collection access for building \
                      linear inheritance graphs. Index safety is guaranteed \
                      by loop bounds over `unique_names` length."
        )]
        fn schema_graph_accepts_arbitrary_lineage() {
            let mut runner = TestRunner::deterministic();
            let strategy = prop::collection::vec("[a-zA-Z0-9]{3,10}", 1..10);

            let run_result = runner.run(&strategy, |names| {
                let parse_name =
                    |raw: String| -> Result<SchemaName, TestCaseError> {
                        match SchemaName::new(raw) {
                            Ok(name) => Ok(name),
                            Err(e) => Err(TestCaseError::fail(format!(
                                "Invalid generated schema name: {e}"
                            ))),
                        }
                    };

                // GIVEN: a set of unique schema names
                let unique_names: Vec<_> = names
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();

                // WHEN: creating a valid linear inheritance graph
                let mut graph = Graph::new();
                for (i, name_str) in unique_names.iter().enumerate() {
                    let name = parse_name(name_str.clone())?;
                    let parent = match i {
                        0 => None,
                        _ => Some(parse_name(unique_names[i - 1].clone())?),
                    };
                    graph.add_node(name, parent);
                }

                // THEN: it must succeed and return the correct order
                let res = graph.resolve_order();
                prop_assert!(
                    res.is_ok(),
                    "Linear graph should resolve successfully, got: {res:?}"
                );
                let Ok(order) = res else {
                    return Ok(());
                };
                assert_eq!(
                    order.len(),
                    unique_names.len(),
                    "Resolution order should contain all schemas"
                );
                Ok(())
            });
            assert!(
                run_result.is_ok(),
                "Deterministic proptest should not fail: {run_result:?}"
            );
        }
    }

    use super::*;

    fn schema_name_literal(lit: &str) -> Option<SchemaName> {
        let result: Result<SchemaName, _> = lit.try_into();
        assert!(result.is_ok(), "Valid schema name expected: {result:?}");
        result.ok()
    }

    /// 3.3-UNIT-021: `detects_circular_inheritance`.
    /// Priority: P0.
    #[test]
    fn detects_circular_inheritance() {
        // GIVEN: a simple circular dependency between two schemas
        let mut graph = Graph::new();
        let Some(a) = schema_name_literal("a") else {
            return;
        };

        let Some(b) = schema_name_literal("b") else {
            return;
        };

        graph.add_node(a.clone(), Some(b.clone()));
        graph.add_node(b, Some(a));

        // WHEN: resolving the order
        let res = graph.resolve_order();

        // THEN: it must return a CircularInheritance error
        assert!(
            matches!(res, Err(SchemaError::CircularInheritance(_))),
            "Circular inheritance between schemas should be detected, got: \
             {res:?}"
        );
    }

    /// 3.3-UNIT-020: `resolves_empty_graph`.
    /// Priority: P2.
    #[test]
    fn resolves_empty_graph() {
        // GIVEN: an empty graph with no schemas
        let graph = Graph::new();

        // WHEN: resolving the order
        let order_result = graph.resolve_order();
        assert!(
            order_result.is_ok(),
            "Empty graph should resolve successfully, got: {order_result:?}"
        );
        let Ok(order) = order_result else {
            return;
        };

        // THEN: it should return an empty order
        assert!(
            order.is_empty(),
            "Empty graph should return empty resolution order"
        );
    }

    /// 3.3-UNIT-022: `determines_topological_resolution_order`.
    /// Priority: P1.
    #[test]
    fn determines_topological_resolution_order() {
        // GIVEN: a linear inheritance: child -> parent
        let mut graph = Graph::new();
        let Some(child) = schema_name_literal("child") else {
            return;
        };

        let Some(parent) = schema_name_literal("parent") else {
            return;
        };

        graph.add_node(child.clone(), Some(parent.clone()));
        graph.add_node(parent.clone(), None);

        // WHEN: resolving the order
        let order_result = graph.resolve_order();
        assert!(
            order_result.is_ok(),
            "Valid linear graph should resolve successfully, got: \
             {order_result:?}"
        );
        let Ok(order) = order_result else {
            return;
        };

        // THEN: it should return parent before child
        assert_eq!(
            order,
            vec![parent, child],
            "Parent schema should be ordered before child schema"
        );
    }
}
