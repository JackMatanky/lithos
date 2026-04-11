//! Raw graph primitives for schema inheritance.
//!
//! This module provides raw graph primitives for inheritance graphs, along with
//! adjacency views used by validation and sorting.
//!
//! # Helper Types
//!
//! - **`AdjacencyMap`**: Parent/child lookups derived from edges
//! - **`Graph::from_child_parents_map`**: Build a raw graph from child →
//!   parents lists
//!
//! # Usage
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use lithos_core::schema::{
//!     aggregate::SchemaId,
//!     graph::{AdjacencyMap, Graph, NodeAccessor, NodeDepth},
//!     topo_sort::TopologicalSorter,
//! };
//!
//! struct MockNode {
//!     id: SchemaId,
//!     depth: NodeDepth,
//! }
//!
//! impl NodeAccessor for MockNode {
//!     fn id(&self) -> SchemaId {
//!         self.id
//!     }
//!
//!     fn depth(&self) -> NodeDepth {
//!         self.depth
//!     }
//! }
//!
//! let root_id = SchemaId::new();
//! let child_id = SchemaId::new();
//! let root = MockNode {
//!     id: root_id,
//!     depth: NodeDepth::ROOT,
//! };
//! let child = MockNode {
//!     id: child_id,
//!     depth: NodeDepth::ROOT,
//! };
//!
//! let mut graph: Graph<MockNode, ()> = Graph::new();
//! graph.add_node(root_id, root);
//! graph.add_node(child_id, child);
//! graph.add_edge(root_id, child_id);
//!
//! let adjacency = AdjacencyMap::from_graph(&graph);
//! let sorter =
//!     TopologicalSorter::try_new(graph.nodes().keys().copied(), &adjacency)
//!         .expect("directed graph");
//! let order = sorter.sort().expect("acyclic graph");
//! assert_eq!(order.roots(), &[root_id]);
//! assert_eq!(order.order(), &[root_id, child_id]);
//! ```

#![expect(
    clippy::iter_over_hash_type,
    reason = "graph algorithms do not rely on HashMap iteration order"
)]

use std::collections::{HashMap, HashSet};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::aggregate::SchemaId;

/// Deconstructed graph components.
pub type GraphParts<T, R> = (HashMap<SchemaId, Node<T>>, Vec<Edge<R>>);

// ============================================================================
//  CORE GRAPH TYPES
// ============================================================================

/// Raw graph primitive for schema inheritance.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Graph<T, R> {
    nodes: HashMap<SchemaId, Node<T>>,
    edges: Vec<Edge<R>>,
}

impl<T, R> Graph<T, R> {
    /// Creates a new empty graph.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Returns the node map.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &HashMap<SchemaId, Node<T>> {
        &self.nodes
    }

    /// Returns a mutable reference to the node map.
    #[inline]
    #[expect(dead_code, reason = "reserved for graph mutations")]
    pub(crate) fn nodes_mut(&mut self) -> &mut HashMap<SchemaId, Node<T>> {
        &mut self.nodes
    }

    /// Consumes the graph and returns its constituent nodes and edges.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> GraphParts<T, R> {
        (self.nodes, self.edges)
    }

    /// Adds a node with the given ID and payload.
    #[inline]
    pub fn add_node(&mut self, id: SchemaId, payload: T) {
        self.nodes.insert(id, Node::new(payload));
    }

    /// Removes a node and all its incident edges.
    #[inline]
    #[expect(dead_code, reason = "reserved for graph mutations")]
    pub(crate) fn remove_node(&mut self, id: SchemaId) -> Option<Node<T>> {
        self.edges.retain(|edge| edge.from != id && edge.to != id);
        self.nodes.remove(&id)
    }

    /// Returns a reference to the node with the given ID.
    #[inline]
    #[must_use]
    #[expect(dead_code, reason = "reserved for graph inspection")]
    pub(crate) fn node(&self, id: SchemaId) -> Option<&Node<T>> {
        self.nodes.get(&id)
    }

    /// Returns a mutable reference to the node with the given ID.
    #[inline]
    #[expect(dead_code, reason = "reserved for graph mutations")]
    pub(crate) fn node_mut(&mut self, id: SchemaId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(&id)
    }

    /// Returns the depth of the node with the given ID.
    #[inline]
    #[must_use]
    #[expect(dead_code, reason = "reserved for graph inspection")]
    pub(crate) fn node_depth(&self, id: SchemaId) -> Option<NodeDepth> {
        self.nodes.get(&id).map(Node::depth)
    }

    /// Adds a directed edge between two existing nodes.
    #[inline]
    pub fn add_edge(&mut self, from: SchemaId, to: SchemaId)
    where
        R: Default,
    {
        self.edges.push(Edge::new(from, to, R::default()));
    }

    /// Adds a directed edge with custom relation metadata.
    #[inline]
    pub fn add_edge_with(&mut self, from: SchemaId, to: SchemaId, relation: R) {
        self.edges.push(Edge::new(from, to, relation));
    }

    /// Removes an edge between two nodes.
    #[inline]
    pub fn remove_edge(&mut self, from: SchemaId, to: SchemaId) {
        self.edges.retain(|edge| !(edge.from == from && edge.to == to));
    }

    /// Reconstructs a graph from nodes and an edge accessor.
    #[inline]
    pub fn from_nodes_and_edges<TNode, E, F>(
        nodes: &HashMap<SchemaId, TNode>,
        edges: &E,
        mut clone_payload: F,
    ) -> Graph<T, R>
    where
        F: FnMut(&TNode) -> T,
        E: EdgeAccessor,
        R: Default,
    {
        let mut graph = Graph::new();
        for (id, node) in nodes {
            graph.add_node(*id, clone_payload(node));
        }
        for id in nodes.keys().copied() {
            for parent_id in edges.parents_of(id) {
                if nodes.contains_key(parent_id) {
                    graph.add_edge(*parent_id, id);
                }
            }
        }
        graph
    }

    /// Resets depths to ROOT for a set of nodes.
    #[inline]
    pub fn reset_depths(&mut self, ids: &HashSet<SchemaId>)
    where
        T: NodeDepthMut,
    {
        for id in ids {
            if let Some(node) = self.nodes.get_mut(id) {
                node.set_depth(NodeDepth::ROOT);
                node.payload_mut().set_depth(NodeDepth::ROOT);
            }
        }
    }

    /// Recomputes depths for the provided node IDs based on the current graph
    /// structure.
    ///
    /// The algorithm handles both global recomputation and scoped updates by
    /// looking up depths from `base_nodes` for parents that are outside the
    /// `affected` set.
    #[inline]
    #[must_use]
    pub fn compute_depths<TNode: NodeAccessor>(
        base_nodes: &HashMap<SchemaId, TNode>,
        affected: &HashSet<SchemaId>,
        order: &[SchemaId],
        adjacency: &AdjacencyMap,
    ) -> HashMap<SchemaId, NodeDepth> {
        let mut depth_by_id = HashMap::with_capacity(order.len());
        for id in order {
            let mut max_parent_depth = 0;
            for parent_id in adjacency.parents_of(*id) {
                let d =
                    parent_depth(base_nodes, &depth_by_id, affected, parent_id);
                max_parent_depth = max_parent_depth.max(d);
            }

            let new_depth = if adjacency.parents_of(*id).is_empty() {
                NodeDepth::ROOT
            } else {
                NodeDepth::new(max_parent_depth.saturating_add(1))
            };
            depth_by_id.insert(*id, new_depth);
        }
        depth_by_id
    }

    /// Fast graph constructor from a map of child → parents.
    #[inline]
    #[must_use]
    pub fn from_child_parents_map<F>(
        mut map: ChildParentsMap,
        mut create_payload: F,
    ) -> Self
    where
        F: FnMut(SchemaId, &[SchemaId]) -> T,
        R: Default,
    {
        map.normalize_all();
        let mut graph = Self::new();
        for (id, parents) in map.into_inner() {
            graph.add_node(id, create_payload(id, &parents));
            for parent_id in parents {
                graph.add_edge(parent_id, id);
            }
        }
        graph
    }
}

fn parent_depth<TNode: NodeAccessor>(
    base_nodes: &HashMap<SchemaId, TNode>,
    depth_by_id: &HashMap<SchemaId, NodeDepth>,
    affected: &HashSet<SchemaId>,
    parent_id: &SchemaId,
) -> usize {
    if affected.contains(parent_id) {
        depth_by_id.get(parent_id).copied().map_or(0, NodeDepth::as_usize)
    } else {
        base_nodes.get(parent_id).map_or(0, |node| node.depth().as_usize())
    }
}

/// A node in the raw graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    depth: NodeDepth,
    payload: T,
}

impl<T> Node<T> {
    /// Creates a new node with a root depth.
    #[inline]
    #[must_use]
    pub fn new(payload: T) -> Self {
        Self {
            depth: NodeDepth::ROOT,
            payload,
        }
    }

    /// Returns the current depth.
    #[inline]
    #[must_use]
    pub fn depth(&self) -> NodeDepth {
        self.depth
    }

    /// Sets the depth.
    #[inline]
    pub fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }

    /// Returns a reference to the payload.
    #[inline]
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Returns a mutable reference to the payload.
    #[inline]
    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Consumes the node and returns its payload.
    #[inline]
    #[must_use]
    pub fn into_payload(self) -> T {
        self.payload
    }
}

/// An edge in the raw graph representing a relationship between two nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge<R> {
    from: SchemaId,
    to: SchemaId,
    relation: R,
}

impl<R> Edge<R> {
    /// Creates a new edge.
    #[inline]
    #[must_use]
    pub fn new(from: SchemaId, to: SchemaId, relation: R) -> Self {
        Self {
            from,
            to,
            relation,
        }
    }

    /// Returns the source node ID.
    #[inline]
    #[must_use]
    pub fn from(&self) -> SchemaId {
        self.from
    }

    /// Returns the target node ID.
    #[inline]
    #[must_use]
    pub fn to(&self) -> SchemaId {
        self.to
    }

    /// Returns a reference to the relation metadata.
    #[inline]
    #[must_use]
    pub fn relation(&self) -> &R {
        &self.relation
    }

    /// Returns a mutable reference to the relation metadata.
    #[inline]
    pub fn relation_mut(&mut self) -> &mut R {
        &mut self.relation
    }
}

// ============================================================================
//  ADJACENCY MAP
// ============================================================================

/// Fast adjacency-based view of a graph for traversal and sorting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjacencyMap {
    /// For each node: incoming neighbors (predecessors).
    /// Key: child node id.
    /// Value: list of parent ids (incoming edges).
    in_neighbors: HashMap<SchemaId, Vec<SchemaId>>,
    /// For each node: outgoing neighbors (successors).
    /// Key: parent node id.
    /// Value: list of child ids (outgoing edges).
    out_neighbors: HashMap<SchemaId, Vec<SchemaId>>,
}

impl AdjacencyMap {
    const EMPTY_NEIGHBORS: &'static [SchemaId] = &[];

    /// Builds an adjacency map from a raw graph.
    #[inline]
    #[must_use]
    pub fn from_graph<T, R>(graph: &Graph<T, R>) -> Self {
        let mut in_neighbors: HashMap<SchemaId, Vec<SchemaId>> = HashMap::new();
        let mut out_neighbors: HashMap<SchemaId, Vec<SchemaId>> =
            HashMap::new();

        for id in graph.nodes.keys() {
            in_neighbors.entry(*id).or_default();
            out_neighbors.entry(*id).or_default();
        }

        for edge in &graph.edges {
            in_neighbors.entry(edge.to).or_default().push(edge.from);
            out_neighbors.entry(edge.from).or_default().push(edge.to);
        }

        for neighbors in in_neighbors.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }
        for neighbors in out_neighbors.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }

        Self {
            in_neighbors,
            out_neighbors,
        }
    }

    /// Builds an adjacency map from raw node IDs and (from, to) edge pairs.
    #[inline]
    #[must_use]
    pub fn from_nodes_and_edges<N, E>(nodes: N, edges: E) -> Self
    where
        N: IntoIterator<Item = SchemaId>,
        E: IntoIterator<Item = (SchemaId, SchemaId)>,
    {
        let mut in_neighbors: HashMap<SchemaId, Vec<SchemaId>> = HashMap::new();
        let mut out_neighbors: HashMap<SchemaId, Vec<SchemaId>> =
            HashMap::new();

        for id in nodes {
            in_neighbors.entry(id).or_default();
            out_neighbors.entry(id).or_default();
        }

        for (from, to) in edges {
            in_neighbors.entry(to).or_default().push(from);
            out_neighbors.entry(from).or_default().push(to);
        }

        for neighbors in in_neighbors.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }
        for neighbors in out_neighbors.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }

        Self {
            in_neighbors,
            out_neighbors,
        }
    }

    /// Returns parent IDs for the given node.
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "mirrors EdgeAccessor for adjacency queries"
    )]
    pub fn parents_of(&self, id: SchemaId) -> &[SchemaId] {
        self.in_neighbors.get(&id).map_or(Self::EMPTY_NEIGHBORS, Vec::as_slice)
    }

    /// Returns child IDs for the given node.
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "mirrors EdgeAccessor for adjacency queries"
    )]
    pub fn children_of(&self, id: SchemaId) -> &[SchemaId] {
        self.out_neighbors.get(&id).map_or(Self::EMPTY_NEIGHBORS, Vec::as_slice)
    }

    /// Returns the total number of nodes in the map.
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.in_neighbors.len()
    }
}

// ============================================================================
//  ACCESSOR TRAITS
// ============================================================================

/// Read-only access to a node's identity and depth.
pub trait NodeAccessor {
    /// Returns the current depth of the node.
    fn depth(&self) -> NodeDepth;
    /// Returns the node identifier.
    fn id(&self) -> SchemaId;
}

/// Mutable access to update node depth during graph processing.
pub trait NodeDepthMut {
    /// Sets the node depth.
    fn set_depth(&mut self, depth: NodeDepth);
}

/// Provides parent/child adjacency lookups.
pub trait EdgeAccessor {
    /// Returns the children for a given node.
    fn children_of(&self, id: SchemaId) -> &[SchemaId];
    /// Returns the parents for a given node.
    fn parents_of(&self, id: SchemaId) -> &[SchemaId];
}

impl EdgeAccessor for AdjacencyMap {
    #[inline]
    fn children_of(&self, id: SchemaId) -> &[SchemaId] {
        self.children_of(id)
    }

    #[inline]
    fn parents_of(&self, id: SchemaId) -> &[SchemaId] {
        self.parents_of(id)
    }
}

// ============================================================================
//  DEPTH NEWTYPE
// ============================================================================

/// Inheritance depth in the DAG (0-indexed for roots).
///
/// - Root nodes: `depth = 0`
/// - Child nodes: `depth = max(parent_depths) + 1`
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
pub struct NodeDepth(usize);

impl NodeDepth {
    /// Root node depth (no parents).
    pub const ROOT: Self = Self(0);

    #[inline]
    #[must_use]
    /// Creates a depth value from a raw count.
    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }

    #[inline]
    #[must_use]
    /// Returns the raw depth value.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    #[inline]
    #[must_use]
    /// Returns the next depth value, saturating on overflow.
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

// ============================================================================
//  CHILD -> PARENTS MAP
// ============================================================================

/// Map of child schema IDs to their parent IDs.
///
/// In this map:
/// - **Key**: The `SchemaId` of a child node.
/// - **Value**: A `Vec<SchemaId>` containing the IDs of its direct parents.
#[derive(Debug, Clone, Default)]
pub struct ChildParentsMap(HashMap<SchemaId, Vec<SchemaId>>);

impl ChildParentsMap {
    /// Creates a new empty map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates a map with pre-existing entries.
    #[inline]
    #[must_use]
    pub fn with_entries(map: HashMap<SchemaId, Vec<SchemaId>>) -> Self {
        Self(map)
    }

    /// Inserts a node and its parents.
    #[inline]
    pub fn insert(&mut self, id: SchemaId, parents: Vec<SchemaId>) {
        self.0.insert(id, parents);
    }

    /// Returns the underlying map.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> HashMap<SchemaId, Vec<SchemaId>> {
        self.0
    }

    /// Sorts and deduplicates parent IDs.
    #[inline]
    pub fn normalize_parents(parents: &mut Vec<SchemaId>) {
        parents.sort();
        parents.dedup();
    }

    /// Normalizes all parent lists in the map.
    #[inline]
    pub fn normalize_all(&mut self) {
        for parents in self.0.values_mut() {
            Self::normalize_parents(parents);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_depth_root_is_zero() {
        assert_eq!(NodeDepth::ROOT.as_usize(), 0);
    }

    #[test]
    fn graph_from_child_parents_builds_edges() {
        let parent = SchemaId::new();
        let child = SchemaId::new();

        let mut map = ChildParentsMap::new();
        map.insert(child, vec![parent]);
        map.insert(parent, Vec::new());

        let graph: Graph<(), ()> =
            Graph::from_child_parents_map(map, |_id, _parents| ());

        assert_eq!(graph.edges.len(), 1);
        let edge = graph.edges.first().expect("edge");
        assert_eq!(edge.from(), parent);
        assert_eq!(edge.to(), child);
    }
}
