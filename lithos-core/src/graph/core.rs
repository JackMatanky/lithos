//! Core graph data structures for directed graphs.
//!
//! Provides `Graph` and `GraphBuilder` for building and querying a directed
//! graph that may contain cycles. DAG validation is handled separately by
//! `DagGraph`.
//!
//! # Examples
//!
//! ```
//! use lithos_core::graph::{GraphBuilder, Node};
//!
//! let mut builder = GraphBuilder::new();
//! builder.add_node(1u8, Node::new(Box::<str>::from("A")));
//! builder.add_node(2u8, Node::new(Box::<str>::from("B")));
//! builder.add_parent(2, 1);
//!
//! let graph = builder.build();
//! assert_eq!(graph.parents_of(2), &[1]);
//! ```

use std::{collections::HashMap, hash::Hash};

use crate::graph::node::{GraphNode, GraphNodeMut};

/// Type alias for the components returned by `into_parts()`.
type GraphParts<Id, N> =
    (HashMap<Id, N>, HashMap<Id, Vec<Id>>, HashMap<Id, Vec<Id>>);

/// Directed graph infrastructure (raw, may contain cycles).
///
/// **Pure infrastructure** - NO serialization constraint. Domain wrappers
/// (like `schema::InheritanceGraph`) add Archive bounds when needed.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{GraphBuilder, Node};
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Node::new(Box::<str>::from("A")));
/// builder.add_node(2u8, Node::new(Box::<str>::from("B")));
/// builder.add_parent(2, 1);
///
/// let graph = builder.build();
/// assert_eq!(graph.parents_of(2), &[1]);
/// ```
#[derive(Debug, Clone)]
pub struct Graph<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode,
{
    /// Nodes indexed by Id (ID is key, not in value).
    nodes: HashMap<Id, N>,

    /// Adjacency: child -> parents (for topological sort).
    parents: HashMap<Id, Vec<Id>>,

    /// Adjacency: parent -> children (for depth computation & traversal).
    children: HashMap<Id, Vec<Id>>,
}

impl<Id, N> Graph<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode,
{
    /// Returns node by ID.
    #[inline]
    #[must_use]
    pub fn get(&self, id: Id) -> Option<&N> {
        self.nodes.get(&id)
    }

    /// Returns mutable node by ID.
    #[inline]
    pub fn get_mut(&mut self, id: Id) -> Option<&mut N>
    where
        N: GraphNodeMut,
    {
        self.nodes.get_mut(&id)
    }

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

    /// Iterates over all node IDs.
    #[inline]
    pub(crate) fn node_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.nodes.keys().copied()
    }

    /// Iterates over all (id, node) pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Id, &N)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    /// Iterates over all (id, node) pairs with mutable access.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut N)>
    where
        N: GraphNodeMut,
    {
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

    /// Consumes the graph and returns its internal components.
    ///
    /// This is useful for advanced graph transformations that need to
    /// rebuild the graph structure with modified nodes.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> GraphParts<Id, N> {
        (self.nodes, self.parents, self.children)
    }

    /// Constructs a graph from its internal components.
    ///
    /// This is the inverse of `into_parts()`, allowing reconstruction
    /// after transformation.
    ///
    /// **Note**: The caller must ensure that all parent/child references
    /// point to valid node IDs and that the adjacency maps are consistent.
    #[inline]
    #[must_use]
    pub fn from_parts(
        nodes: HashMap<Id, N>,
        parents: HashMap<Id, Vec<Id>>,
        children: HashMap<Id, Vec<Id>>,
    ) -> Self {
        Self {
            nodes,
            parents,
            children,
        }
    }

    /// Maps each node payload into a new graph while preserving structure.
    ///
    /// This is a high-performance, consuming transformation that reuses the
    /// existing adjacency maps.
    ///
    /// # Errors
    ///
    /// Returns an error from the mapping closure.
    #[inline]
    pub fn map_nodes<M, E, F>(self, mut f: F) -> Result<Graph<Id, M>, E>
    where
        M: GraphNode,
        F: FnMut(Id, &N::Payload) -> Result<M::Payload, E>,
    {
        let mut new_nodes = HashMap::with_capacity(self.nodes.len());

        #[expect(
            clippy::iter_over_hash_type,
            reason = "graph transforms do not depend on HashMap order"
        )]
        for (id, node) in self.nodes {
            let new_payload = f(id, node.payload())?;
            new_nodes.insert(id, M::from_payload(new_payload));
        }

        Ok(Graph {
            nodes: new_nodes,
            parents: self.parents,
            children: self.children,
        })
    }
}

/// Builder for constructing graphs.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{GraphBuilder, Node};
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Node::new(Box::<str>::from("A")));
/// builder.add_node(2u8, Node::new(Box::<str>::from("B")));
/// builder.add_parent(2, 1);
///
/// let graph = builder.build();
/// assert_eq!(graph.parents_of(2), &[1]);
/// ```
pub struct GraphBuilder<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode,
{
    nodes: HashMap<Id, N>,
    child_to_parents: HashMap<Id, Vec<Id>>,
}

impl<Id, N> Default for GraphBuilder<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode,
{
    #[inline]
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            child_to_parents: HashMap::new(),
        }
    }
}

impl<Id, N> GraphBuilder<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode,
{
    /// Creates a new graph builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self
    where
        N: GraphNode,
    {
        Self::default()
    }

    /// Builds the graph with normalized adjacency lists.
    #[inline]
    #[must_use]
    pub fn build(self) -> Graph<Id, N> {
        let mut parents = HashMap::with_capacity(self.child_to_parents.len());
        let mut children: HashMap<Id, Vec<Id>> = HashMap::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap iteration order doesn't affect graph semantics \
                      (edges are sorted)"
        )]
        for (child_id, mut parent_ids) in self.child_to_parents {
            parent_ids.sort();
            parent_ids.dedup();

            for &parent_id in &parent_ids {
                children.entry(parent_id).or_default().push(child_id);
            }

            parents.insert(child_id, parent_ids);
        }

        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap iteration order doesn't affect graph semantics \
                      (edges are sorted)"
        )]
        for child_list in children.values_mut() {
            child_list.sort();
            child_list.dedup();
        }

        Graph {
            nodes: self.nodes,
            parents,
            children,
        }
    }

    /// Pre-allocates capacity for expected node count.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self
    where
        N: GraphNode,
    {
        Self {
            nodes: HashMap::with_capacity(capacity),
            child_to_parents: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a node to the graph.
    #[inline]
    pub fn add_node(&mut self, id: Id, node: N) {
        self.nodes.insert(id, node);
    }

    /// Adds a parent relationship (child extends parent).
    #[inline]
    pub fn add_parent(&mut self, child: Id, parent: Id) {
        self.child_to_parents.entry(child).or_default().push(parent);
    }
}

#[cfg(test)]
#[expect(
    clippy::as_conversions,
    reason = "Test code uses simplified patterns for clarity"
)]
mod tests {
    use super::*;

    mod builder {
        use super::*;
        use crate::graph::node::Node;

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
            builder.add_node(1i32, Node::new(Box::<str>::from("A")));
            builder.add_node(2i32, Node::new(Box::<str>::from("B")));
            builder.add_parent(2i32, 1i32);
            builder.add_parent(2i32, 1i32);

            let graph = builder.build();
            assert_eq!(
                graph.parents_of(2i32),
                &[1i32],
                "expected parent list to be deduped, got {:?}",
                graph.parents_of(2i32)
            );
        }
    }

    mod accessors {
        use super::*;
        use crate::graph::node::Node;

        #[test]
        fn parents_of_returns_empty_slice_when_missing() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Node::new(Box::<str>::from("A")));
            let graph = builder.build();

            assert_eq!(
                graph.parents_of(1i32),
                &[] as &[i32],
                "expected no parents for root node"
            );
        }

        #[test]
        fn children_of_returns_empty_slice_when_missing() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Node::new(Box::<str>::from("A")));
            let graph = builder.build();

            assert_eq!(
                graph.children_of(1i32),
                &[] as &[i32],
                "expected no children for isolated node"
            );
        }

        #[test]
        fn get_returns_none_when_node_unknown() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Node::new(Box::<str>::from("A")));
            let graph = builder.build();

            assert!(
                graph.get(2i32).is_none(),
                "expected missing node to return None"
            );
        }

        #[test]
        fn iter_returns_all_nodes() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Node::new(Box::<str>::from("A")));
            builder.add_node(2i32, Node::new(Box::<str>::from("B")));
            let graph = builder.build();

            let count = graph.iter().count();
            assert_eq!(
                count, 2,
                "expected iterator to yield 2 nodes, got {count}",
            );
        }
    }
}
