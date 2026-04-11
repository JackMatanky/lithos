//! Topological sorting for schema inheritance graphs.
//!
//! This module provides a directed-acyclic validation gate and deterministic
//! topological ordering for inheritance graphs. Construction is fallible to
//! enforce the "parse, don't validate" rule: use `try_new` to validate the
//! graph's directionality once, then call `sort` or `sort_scoped` for ordering.
//!
//! # Ordering
//! - `sort` returns a global topological order and the root set.
//! - `sort_scoped` returns a topological order for a subset while preserving
//!   in-scope dependencies.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::schema::{
    aggregate::SchemaId, error::SchemaResolutionError, graph::AdjacencyMap,
};

/// Topological ordering result for a directed inheritance graph.
///
/// The `order` guarantees parents appear before children, and `roots` captures
/// every node with zero in-degree in the same deterministic order used to seed
/// the sort. This structure is intentionally minimal and decoupled from any
/// payload so it can be reused across schema processing stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologicalOrder {
    order: Vec<SchemaId>,
    roots: Vec<SchemaId>,
}

impl TopologicalOrder {
    /// Returns the topological order of schema IDs.
    #[inline]
    #[must_use]
    pub fn order(&self) -> &[SchemaId] {
        &self.order
    }

    /// Returns the root IDs (nodes with zero in-degree).
    #[inline]
    #[must_use]
    pub fn roots(&self) -> &[SchemaId] {
        &self.roots
    }
}

/// Deterministic topological sorter for directed inheritance graphs.
///
/// Construction is fallible and enforces directionality checks up front. The
/// sorter is otherwise pure: it never mutates the underlying adjacency map and
/// always produces the same ordering for the same input node set.
///
/// ```rust
/// use lithos_core::schema::{
///     aggregate::SchemaId,
///     graph::{AdjacencyMap, Graph},
///     topo_sort::TopologicalSorter,
/// };
///
/// let id_a = SchemaId::new();
/// let id_b = SchemaId::new();
///
/// let mut graph: Graph<(), ()> = Graph::new();
/// graph.add_node(id_a, ());
/// graph.add_node(id_b, ());
/// graph.add_edge(id_a, id_b);
///
/// let adjacency = AdjacencyMap::from_graph(&graph);
/// let sorter =
///     TopologicalSorter::try_new(graph.nodes().keys().copied(), &adjacency)
///         .expect("directed graph");
/// let order = sorter.sort().expect("acyclic graph");
///
/// assert_eq!(order.roots(), &[id_a]);
/// assert_eq!(order.order(), &[id_a, id_b]);
/// ```
pub struct TopologicalSorter<'graph> {
    nodes: Vec<SchemaId>,
    adjacency: &'graph AdjacencyMap,
}

impl<'graph> TopologicalSorter<'graph> {
    /// Creates a sorter after validating the graph is directed.
    ///
    /// # Errors
    /// Returns `SchemaResolutionError::NotDirected` when the adjacency is
    /// inconsistent (i.e. the graph is not directed).
    #[inline]
    pub fn try_new<I>(
        nodes: I,
        adjacency: &'graph AdjacencyMap,
    ) -> Result<Self, SchemaResolutionError>
    where
        I: IntoIterator<Item = SchemaId>,
    {
        let mut nodes: Vec<SchemaId> = nodes.into_iter().collect();
        nodes.sort();
        if !is_directed(adjacency, &nodes) {
            return Err(SchemaResolutionError::NotDirected);
        }
        Ok(Self {
            nodes,
            adjacency,
        })
    }

    /// Produces a full topological order for all nodes.
    ///
    /// # Errors
    /// Returns `SchemaResolutionError::CycleDetected` when a cycle is found.
    #[inline]
    pub fn sort(&self) -> Result<TopologicalOrder, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = self
            .nodes
            .iter()
            .map(|id| (*id, self.adjacency.parents_of(*id).len()))
            .collect();

        let mut queue = build_queue(&in_degree);
        let roots: Vec<SchemaId> = queue.iter().copied().collect();
        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);
            for &child_id in self.adjacency.children_of(node_id) {
                decrement_in_degree(&mut in_degree, &mut queue, child_id);
            }
        }

        if !is_acyclic(order.len(), self.nodes.len()) {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok(TopologicalOrder {
            order,
            roots,
        })
    }

    /// Produces a topological order restricted to the affected subset.
    ///
    /// # Errors
    /// Returns `SchemaResolutionError::CycleDetected` when a cycle is found
    /// in the scoped subgraph.
    #[inline]
    pub fn sort_scoped(
        &self,
        affected: &HashSet<SchemaId>,
    ) -> Result<TopologicalOrder, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = HashMap::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "in-degree construction ignores iteration order"
        )]
        for &id in affected {
            let parents_in_scope = self
                .adjacency
                .parents_of(id)
                .iter()
                .filter(|pid| affected.contains(pid))
                .count();
            in_degree.insert(id, parents_in_scope);
        }

        let mut queue = build_queue(&in_degree);
        let roots: Vec<SchemaId> = queue.iter().copied().collect();
        let mut order = Vec::with_capacity(affected.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);
            for &child_id in self.adjacency.children_of(node_id) {
                if !affected.contains(&child_id) {
                    continue;
                }
                decrement_in_degree(&mut in_degree, &mut queue, child_id);
            }
        }

        if !is_acyclic(order.len(), affected.len()) {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok(TopologicalOrder {
            order,
            roots,
        })
    }
}

/// Builds the initial queue of nodes with zero in-degree.
fn build_queue(in_degree: &HashMap<SchemaId, usize>) -> VecDeque<SchemaId> {
    in_degree.iter().filter(|&(_, deg)| *deg == 0).map(|(&id, _)| id).collect()
}

/// Returns `true` when the produced order covers the expected node count.
/// Decrements in-degree for a child and enqueues it at zero.
fn decrement_in_degree(
    in_degree: &mut HashMap<SchemaId, usize>,
    queue: &mut VecDeque<SchemaId>,
    child_id: SchemaId,
) {
    let Some(deg) = in_degree.get_mut(&child_id) else {
        return;
    };
    *deg = deg.saturating_sub(1);
    if *deg == 0 {
        queue.push_back(child_id);
    }
}

fn is_acyclic(order_len: usize, expected_len: usize) -> bool {
    order_len == expected_len
}

/// Returns `true` when the adjacency lists are internally consistent.
#[inline]
fn is_directed(adjacency: &AdjacencyMap, nodes: &[SchemaId]) -> bool {
    for id in nodes {
        for child in adjacency.children_of(*id) {
            if !adjacency.parents_of(*child).contains(id) {
                return false;
            }
        }
        for parent in adjacency.parents_of(*id) {
            if !adjacency.children_of(*parent).contains(id) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::graph::Graph;

    #[test]
    fn topological_order_respects_all_parents() {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();
        let id_c = SchemaId::new();
        let id_d = SchemaId::new();

        let mut graph: Graph<(), ()> = Graph::new();
        graph.add_node(id_a, ());
        graph.add_node(id_b, ());
        graph.add_node(id_c, ());
        graph.add_node(id_d, ());
        graph.add_edge(id_a, id_b);
        graph.add_edge(id_a, id_c);
        graph.add_edge(id_b, id_d);
        graph.add_edge(id_c, id_d);

        let adjacency = AdjacencyMap::from_graph(&graph);
        let sorter = TopologicalSorter::try_new(
            graph.nodes().keys().copied(),
            &adjacency,
        )
        .expect("directed graph");
        let order = sorter.sort().expect("topo sort");

        let position: HashMap<SchemaId, usize> = order
            .order()
            .iter()
            .copied()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect();

        let pos_a = *position.get(&id_a).expect("position a");
        let pos_b = *position.get(&id_b).expect("position b");
        let pos_c = *position.get(&id_c).expect("position c");
        let pos_d = *position.get(&id_d).expect("position d");

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn detects_cycles_in_multi_parent_graph() {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();
        let id_c = SchemaId::new();

        let mut graph: Graph<(), ()> = Graph::new();
        graph.add_node(id_a, ());
        graph.add_node(id_b, ());
        graph.add_node(id_c, ());
        graph.add_edge(id_c, id_a);
        graph.add_edge(id_a, id_b);
        graph.add_edge(id_b, id_c);

        let adjacency = AdjacencyMap::from_graph(&graph);
        let sorter = TopologicalSorter::try_new(
            graph.nodes().keys().copied(),
            &adjacency,
        )
        .expect("directed graph");

        sorter.sort().unwrap_err();
    }
}
