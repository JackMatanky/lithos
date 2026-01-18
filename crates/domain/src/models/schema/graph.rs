//! `SchemaGraph` domain service for inheritance resolution.
//!
//! Provides topological sorting and cycle detection for schema inheritance graphs.

#![allow(
    clippy::module_name_repetitions,
    reason = "SchemaGraph is the primary service in this module"
)]

use std::collections::{HashMap, HashSet};

use super::core::SchemaName;
use crate::errors::DomainError;

/// Domain Service: Validates acyclic schema inheritance and determines resolution order.
///
/// Uses topological sorting to ensure parent schemas are resolved before child schemas.
/// Detects circular inheritance dependencies.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{SchemaGraph, SchemaName};
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    use lithos_test_utils::assert_eq_detailed;

    use super::*;

    /// 3.3-UNIT-021: `detects_circular_inheritance`.
    /// Priority: P0.
    #[test]
    fn detects_circular_inheritance() {
        // GIVEN a simple circular dependency between two schemas
        let mut graph = SchemaGraph::new();
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
