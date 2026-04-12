//! Core graph data structures for directed graphs.
//!
//! Provides `Graph` and `GraphBuilder` for building and querying a directed
//! graph that may contain cycles. DAG validation is handled separately by
//! `DagGraph`.
//!
//! # Examples
//!
//! ```
//! use lithos_core::graph::GraphBuilder;
//!
//! let mut builder = GraphBuilder::new();
//! builder.add_node(1u8, Box::<str>::from("A"));
//! builder.add_node(2u8, Box::<str>::from("B"));
//! builder.add_parent(2, 1);
//!
//! let graph = builder.build();
//! assert_eq!(graph.parents_of(2), &[1]);
//! ```

use std::{collections::HashMap, hash::Hash};

use rkyv::{Archive, Deserialize, Serialize};

use crate::graph::node::{Node, NodeDepth};

/// Directed graph infrastructure (raw, may contain cycles).
///
/// # Examples
///
/// ```
/// use lithos_core::graph::GraphBuilder;
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Box::<str>::from("A"));
/// builder.add_node(2u8, Box::<str>::from("B"));
/// builder.add_parent(2, 1);
///
/// let graph = builder.build();
/// assert_eq!(graph.parents_of(2), &[1]);
/// ```
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Graph<Id, T>
where
    Id: Copy + Eq + Hash + Ord + Archive,
    T: Archive,
{
    /// Nodes indexed by Id (ID is key, not in value).
    nodes: HashMap<Id, Node<T>>,

    /// Adjacency: child -> parents (for topological sort).
    parents: HashMap<Id, Vec<Id>>,

    /// Adjacency: parent -> children (for depth computation & traversal).
    children: HashMap<Id, Vec<Id>>,
}

impl<Id, T> Graph<Id, T>
where
    Id: Copy + Eq + Hash + Ord + Archive,
    T: Archive,
{
    /// Returns parent IDs for a node (empty slice if none).
    #[inline]
    #[must_use]
    pub fn parents_of(&self, id: Id) -> &[Id] {
        self.parents.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Returns child IDs for a node (empty slice if none).
    #[inline]
    #[must_use]
    pub fn children_of(&self, id: Id) -> &[Id] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Returns node by ID.
    #[inline]
    #[must_use]
    pub fn get(&self, id: Id) -> Option<&Node<T>> {
        self.nodes.get(&id)
    }

    /// Returns mutable node by ID.
    #[inline]
    pub fn get_mut(&mut self, id: Id) -> Option<&mut Node<T>> {
        self.nodes.get_mut(&id)
    }

    /// Iterates over all (id, node) pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Id, &Node<T>)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    /// Iterates over all node IDs.
    #[inline]
    pub(crate) fn node_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.nodes.keys().copied()
    }

    /// Iterates over all (id, node) pairs with mutable access.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut Node<T>)> {
        self.nodes.iter_mut().map(|(id, node)| (*id, node))
    }

    /// Returns the number of nodes.
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the parents adjacency map.
    #[inline]
    pub(crate) fn parents(&self) -> &HashMap<Id, Vec<Id>> {
        &self.parents
    }

    /// Computes depths for all nodes given topological order.
    #[must_use]
    pub fn compute_depths(&self, order: &[Id]) -> HashMap<Id, NodeDepth> {
        let mut depths = HashMap::with_capacity(order.len());

        for &id in order {
            let max_parent_depth = self
                .parents_of(id)
                .iter()
                .filter_map(|pid| depths.get(pid))
                .map(|d| d.as_usize())
                .max()
                .unwrap_or(0);

            let depth = if self.parents_of(id).is_empty() {
                NodeDepth::ROOT
            } else {
                NodeDepth::new(max_parent_depth.saturating_add(1))
            };

            depths.insert(id, depth);
        }

        depths
    }

    /// Updates all node depths in-place.
    pub fn update_depths(&mut self, depths: HashMap<Id, NodeDepth>) {
        for (id, depth) in depths {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.set_depth(depth);
            }
        }
    }
}

/// Builder for constructing graphs.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::GraphBuilder;
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Box::<str>::from("A"));
/// builder.add_node(2u8, Box::<str>::from("B"));
/// builder.add_parent(2, 1);
/// let graph = builder.build();
/// assert_eq!(graph.node_count(), 2);
/// ```
pub struct GraphBuilder<Id, T> {
    nodes: HashMap<Id, T>,
    child_to_parents: HashMap<Id, Vec<Id>>,
}

impl<Id, T> GraphBuilder<Id, T>
where
    Id: Copy + Eq + Hash + Ord,
{
    /// Creates a new graph builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            child_to_parents: HashMap::new(),
        }
    }

    /// Pre-allocates capacity for expected node count.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(capacity),
            child_to_parents: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a node to the graph.
    #[inline]
    pub fn add_node(&mut self, id: Id, payload: T) {
        self.nodes.insert(id, payload);
    }

    /// Adds a parent relationship (child extends parent).
    #[inline]
    pub fn add_parent(&mut self, child: Id, parent: Id) {
        self.child_to_parents.entry(child).or_default().push(parent);
    }

    /// Builds the graph with normalized adjacency lists.
    #[must_use]
    pub fn build(self) -> Graph<Id, T>
    where
        Id: Archive,
        T: Archive,
    {
        let mut parents = HashMap::with_capacity(self.child_to_parents.len());
        let mut children: HashMap<Id, Vec<Id>> = HashMap::new();

        for (child_id, mut parent_ids) in self.child_to_parents {
            parent_ids.sort();
            parent_ids.dedup();

            for &parent_id in &parent_ids {
                children.entry(parent_id).or_default().push(child_id);
            }

            parents.insert(child_id, parent_ids);
        }

        for child_list in children.values_mut() {
            child_list.sort();
            child_list.dedup();
        }

        let nodes = self
            .nodes
            .into_iter()
            .map(|(id, payload)| (id, Node::new(payload)))
            .collect();

        Graph {
            nodes,
            parents,
            children,
        }
    }
}

impl<Id, T> Default for GraphBuilder<Id, T>
where
    Id: Copy + Eq + Hash + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DagGraph;

    fn sample_depths() -> HashMap<u8, NodeDepth> {
        let mut builder = GraphBuilder::new();
        builder.add_node(1, Box::<str>::from("A"));
        builder.add_node(2, Box::<str>::from("B"));
        builder.add_node(3, Box::<str>::from("C"));
        builder.add_node(4, Box::<str>::from("D"));
        builder.add_parent(2, 1);
        builder.add_parent(3, 1);
        builder.add_parent(4, 2);
        builder.add_parent(4, 3);

        let graph = builder.build();
        let dag =
            DagGraph::try_from(graph).expect("expected sample DAG to be valid");
        dag.graph().compute_depths(dag.topo_order())
    }

    mod builder {
        use super::*;

        #[test]
        fn returns_empty_graph_when_no_nodes_added() {
            let builder = GraphBuilder::<u8, ()>::new();
            let graph = builder.build();
            assert_eq!(
                graph.node_count(),
                0,
                "expected empty graph, got {} nodes",
                graph.node_count()
            );
        }

        #[test]
        fn dedups_parent_edges_when_duplicated() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            builder.add_node(2, Box::<str>::from("B"));
            builder.add_parent(2, 1);
            builder.add_parent(2, 1);

            let graph = builder.build();
            assert_eq!(
                graph.parents_of(2),
                &[1],
                "expected parent list to be deduped, got {:?}",
                graph.parents_of(2)
            );
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn parents_of_returns_empty_slice_when_missing() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            let graph = builder.build();

            assert_eq!(
                graph.parents_of(1),
                &[],
                "expected no parents for root node"
            );
        }

        #[test]
        fn children_of_returns_empty_slice_when_missing() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            let graph = builder.build();

            assert_eq!(
                graph.children_of(1),
                &[],
                "expected no children for isolated node"
            );
        }

        #[test]
        fn get_returns_none_when_node_unknown() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            let graph = builder.build();

            assert!(
                graph.get(2).is_none(),
                "expected missing node to return None"
            );
        }

        #[test]
        fn iter_returns_all_nodes() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            builder.add_node(2, Box::<str>::from("B"));
            let graph = builder.build();

            let count = graph.iter().count();
            assert_eq!(
                count, 2,
                "expected iterator to yield 2 nodes, got {}",
                count
            );
        }
    }

    mod compute_depths {
        use super::*;

        #[test]
        fn returns_root_depth_as_zero() {
            let depths = sample_depths();
            assert_eq!(
                depths[&1].as_usize(),
                0,
                "expected root depth 0, got {}",
                depths[&1].as_usize()
            );
        }

        #[test]
        fn returns_single_parent_depth_as_one() {
            let depths = sample_depths();
            assert_eq!(
                depths[&2].as_usize(),
                1,
                "expected depth 1 for single-parent node, got {}",
                depths[&2].as_usize()
            );
        }

        #[test]
        fn returns_sibling_depth_as_one() {
            let depths = sample_depths();
            assert_eq!(
                depths[&3].as_usize(),
                1,
                "expected depth 1 for sibling node, got {}",
                depths[&3].as_usize()
            );
        }

        #[test]
        fn returns_max_parent_depth_plus_one_for_multi_parent() {
            let depths = sample_depths();
            assert_eq!(
                depths[&4].as_usize(),
                2,
                "expected multi-parent depth 2, got {}",
                depths[&4].as_usize()
            );
        }
    }

    mod update_depths {
        use super::*;

        #[test]
        fn updates_existing_node_depth() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            let mut graph = builder.build();

            let depths = HashMap::from([(1u8, NodeDepth::new(2))]);
            graph.update_depths(depths);

            let depth = match graph.get(1) {
                Some(node) => node.depth().as_usize(),
                None => panic!("expected node 1 to exist"),
            };
            assert_eq!(depth, 2, "expected updated depth 2, got {}", depth);
        }

        #[test]
        fn ignores_missing_nodes_when_updating() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            let mut graph = builder.build();

            let depths = HashMap::from([(2u8, NodeDepth::new(3))]);
            graph.update_depths(depths);

            assert!(
                graph.get(2).is_none(),
                "expected missing node to stay absent after update"
            );
        }
    }
}
