//! Schema inheritance graph types and utilities.
//!
//! This module provides schema-specific wrappers around the generic graph
//! infrastructure for managing schema inheritance hierarchies with support
//! for multiple parents.
//!
//! # Architecture
//!
//! The module wraps the generic graph infrastructure with schema-specific
//! types:
//!
//! - **`InheritanceGraph<T>`**: Serializable, persistence-focused DAG shape
//! - **`ProcessingGraph<T>`**: Tuple newtype wrapper around `Graph<SchemaId,
//!   T>` for transient processing
//! - **`SchemaGraphBuilder<T>`**: Tuple newtype wrapper around
//!   `GraphBuilder<SchemaId, T>`
//!
//! # DAG vs Tree
//!
//! This implementation supports **multiple inheritance** (DAG) instead of
//! single inheritance (tree):
//!
//! ```text
//! Tree (single parent):          DAG (multiple parents):
//!       A                               A       B
//!      / \                              |      /
//!     B   C                             C  ───┘
//!     |                                 |
//!     D                                 D
//! ```
//!
//! In the DAG model:
//! - Nodes can have **multiple parents** (edges)
//! - Depth = `max(parent_depths) + 1`
//!
//! # Usage
//!
//! ```rust
//! use lithos_core::schema::{
//!     identifier::SchemaId,
//!     inheritance::{InheritanceGraph, SchemaGraphBuilder},
//! };
//!
//! let root_id = SchemaId::new();
//! let child_id = SchemaId::new();
//!
//! let mut builder = SchemaGraphBuilder::new();
//! builder.add_node(root_id, ());
//! builder.add_node(child_id, ());
//! builder.add_parent(child_id, root_id);
//!
//! let graph = builder.build();
//! let dag = InheritanceGraph::try_from(graph).expect("acyclic graph");
//!
//! assert_eq!(dag.roots(), &[root_id]);
//! assert_eq!(dag.topo_order().first(), Some(&root_id));
//! ```

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use crate::{
    graph::{
        DagGraph, Graph, GraphBuilder, sorting::topological_sort_with_nodes,
    },
    schema::identifier::SchemaId,
};

// ============================================================================
//  INHERITANCE GRAPH (for persistence)
// ============================================================================

/// Schema inheritance graph with serialization support.
///
/// This newtype wrapper enforces Archive on the payload for persistence.
/// Use this for the final validated DAG that gets saved to the database.
///
/// # Example
///
/// ```
/// use lithos_core::schema::{
///     identifier::SchemaId,
///     inheritance::{InheritanceGraph, SchemaGraphBuilder},
/// };
///
/// let root = SchemaId::new();
/// let child = SchemaId::new();
///
/// let mut builder = SchemaGraphBuilder::new();
/// builder.add_node(root, ());
/// builder.add_node(child, ());
/// builder.add_parent(child, root);
///
/// let dag = InheritanceGraph::try_from(builder.build()).unwrap();
/// assert_eq!(dag.roots(), &[root]);
/// ```
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(bytecheck(bounds()))]
pub struct InheritanceGraph<T>
where
    T: rkyv::Archive,
{
    nodes: HashMap<SchemaId, T>,
    parents: HashMap<SchemaId, Vec<SchemaId>>,
    children: HashMap<SchemaId, Vec<SchemaId>>,
    #[rkyv(with = rkyv::with::Skip)]
    topo_order: Vec<SchemaId>,
    #[rkyv(with = rkyv::with::Skip)]
    roots: Vec<SchemaId>,
}

impl<T> InheritanceGraph<T>
where
    T: rkyv::Archive,
{
    /// Returns the cached topological order.
    #[inline]
    #[must_use]
    pub fn topo_order(&self) -> &[SchemaId] {
        &self.topo_order
    }

    /// Returns the cached roots (nodes with no parents).
    #[inline]
    #[must_use]
    pub fn roots(&self) -> &[SchemaId] {
        &self.roots
    }

    /// Returns an iterator over node IDs and payloads.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (SchemaId, &T)> {
        self.nodes.iter().map(|(id, payload)| (*id, payload))
    }

    /// Returns parent IDs for a given node.
    #[inline]
    #[must_use]
    pub fn parents_of(&self, id: SchemaId) -> &[SchemaId] {
        self.parents.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Returns child IDs for a given node.
    #[inline]
    #[must_use]
    pub fn children_of(&self, id: SchemaId) -> &[SchemaId] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Converts to a transient `DagGraph` for processing.
    /// Reconstructs a `DagGraph` from the serializable representation.
    ///
    /// # Errors
    ///
    /// Returns an error if graph reconstruction fails (should never happen
    /// as the graph was validated on construction).
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Order not relevant for reconstruction"
    )]
    pub fn to_dag_graph(
        &self,
    ) -> Result<
        DagGraph<SchemaId, T>,
        crate::schema::error::SchemaInheritanceError,
    >
    where
        T: Clone
            + crate::graph::GraphNode<Payload = T>
            + crate::graph::DiGraphNode,
    {
        let mut builder = SchemaGraphBuilder::new();
        for (id, payload) in &self.nodes {
            builder.add_node(*id, payload.clone());
        }
        for (child_id, parent_ids) in &self.parents {
            for &parent_id in parent_ids {
                builder.add_parent(*child_id, parent_id);
            }
        }
        builder.build::<T>().try_into_dag().map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })
    }
}

impl<T> TryFrom<ProcessingGraph<T>> for InheritanceGraph<T>
where
    T: rkyv::Archive
        + Clone
        + crate::graph::GraphNode
        + crate::graph::DiGraphNode,
{
    type Error = crate::schema::error::SchemaInheritanceError;

    #[inline]
    fn try_from(graph: ProcessingGraph<T>) -> Result<Self, Self::Error> {
        let dag = graph.try_into_dag().map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })?;

        // Extract topology before consuming graph
        let topo_order = dag.topo_order().to_vec();
        let roots = dag.roots().to_vec();

        let raw_graph = dag.into_graph();

        // Extract serializable representation by cloning payloads
        let mut nodes = std::collections::HashMap::new();
        let mut parents = std::collections::HashMap::new();
        let mut children = std::collections::HashMap::new();

        for (id, node) in raw_graph.iter() {
            nodes.insert(id, node.clone());
        }

        for (child_id, _) in raw_graph.iter() {
            let parent_ids = raw_graph.parents_of(child_id);
            if !parent_ids.is_empty() {
                parents.insert(child_id, parent_ids.to_vec());
            }

            let child_ids = raw_graph.children_of(child_id);
            if !child_ids.is_empty() {
                children.insert(child_id, child_ids.to_vec());
            }
        }

        Ok(Self {
            nodes,
            parents,
            children,
            topo_order,
            roots,
        })
    }
}

// ============================================================================
//  PROCESSING GRAPH (for pipeline - no serialization)
// ============================================================================

/// Schema graph wrapper for pipeline processing.
///
/// This wrapper allows using ANY payload type (including non-Archive types
/// like Raw* schemas). Use this for intermediate pipeline stages where
/// serialization is not needed.
///
/// The graph stores nodes directly as `T` where `T` implements graph node
/// traits.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProcessingGraph<T>
where
    T: crate::graph::GraphNode + crate::graph::DiGraphNode,
{
    inner: Graph<SchemaId, T>,
    topo_cache: Option<Arc<[SchemaId]>>,
}

impl<T> ProcessingGraph<T>
where
    T: crate::graph::GraphNode + crate::graph::DiGraphNode,
{
    /// Wraps a graph for processing.
    #[inline]
    #[must_use]
    pub fn from_inner(inner: Graph<SchemaId, T>) -> Self {
        Self {
            inner,
            topo_cache: None,
        }
    }

    /// Returns a shared reference to the underlying graph.
    #[inline]
    #[must_use]
    pub fn as_inner(&self) -> &Graph<SchemaId, T> {
        &self.inner
    }

    /// Returns a mutable reference to the underlying graph.
    ///
    /// NOTE: Modifying the graph directly invalidates the topological cache.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Graph<SchemaId, T> {
        self.topo_cache = None;
        &mut self.inner
    }

    /// Consumes self and returns the underlying graph.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Graph<SchemaId, T> {
        self.inner
    }

    /// Computes topological order for the current graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph has cycles or references missing nodes.
    #[inline]
    pub fn topo_order(
        &self,
    ) -> Result<Vec<SchemaId>, crate::schema::error::SchemaInheritanceError>
    {
        topological_sort_with_nodes(self.inner.parents(), self.inner.node_ids())
            .map(|(order, _roots)| order)
            .map_err(|_e| {
                crate::schema::error::SchemaInheritanceError::CycleDetected {
                    nodes: Vec::new(),
                }
            })
    }

    /// Returns a shared reference to the underlying schema graph.
    #[inline]
    #[must_use]
    pub fn graph(&self) -> &Graph<SchemaId, T> {
        &self.inner
    }

    /// Returns a mutable reference to the underlying schema graph.
    ///
    /// NOTE: Modifying the graph directly invalidates the topological cache.
    #[inline]
    pub fn graph_mut(&mut self) -> &mut Graph<SchemaId, T> {
        self.topo_cache = None;
        &mut self.inner
    }

    /// Validates and converts the processing graph into a `DagGraph`.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph has cycles or missing-node references.
    #[inline]
    pub fn try_into_dag(
        self,
    ) -> Result<
        DagGraph<SchemaId, T>,
        crate::schema::error::SchemaInheritanceError,
    > {
        DagGraph::try_from(self.inner).map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })
    }

    /// Returns node IDs sorted deterministically by UUID.
    #[inline]
    #[must_use]
    pub fn node_ids_sorted(&self) -> Vec<SchemaId> {
        let mut ids: Vec<SchemaId> = self.inner.node_ids().collect();
        ids.sort();
        ids
    }

    /// Maps each node into a new graph while preserving structure.
    ///
    /// This is a high-performance, consuming transformation that reuses the
    /// existing adjacency maps and preserves the topological cache.
    ///
    /// The mapper function receives the full node (e.g., `ProcessorNode<P>`)
    /// and must return a new node type.
    ///
    /// # Errors
    ///
    /// Returns an error from the mapping closure.
    pub fn map_payload<U, E, F>(
        self,
        mut mapper: F,
    ) -> Result<ProcessingGraph<U>, E>
    where
        U: crate::graph::GraphNode + crate::graph::DiGraphNode,
        F: FnMut(SchemaId, T) -> Result<U, E>,
    {
        use std::collections::HashMap;

        // Destructure the inner graph to get its parts
        let crate::graph::Graph {
            nodes,
            parents,
            children,
        } = self.inner;

        let mut new_nodes = HashMap::with_capacity(nodes.len());

        #[expect(
            clippy::iter_over_hash_type,
            reason = "graph transforms do not depend on HashMap order"
        )]
        for (id, node) in nodes {
            let new_node = mapper(id, node)?;
            new_nodes.insert(id, new_node);
        }

        Ok(ProcessingGraph {
            inner: crate::graph::Graph {
                nodes: new_nodes,
                parents,
                children,
            },
            topo_cache: self.topo_cache,
        })
    }
}

impl<T> From<Graph<SchemaId, T>> for ProcessingGraph<T>
where
    T: crate::graph::GraphNode + crate::graph::DiGraphNode,
{
    #[inline]
    fn from(value: Graph<SchemaId, T>) -> Self {
        Self::from_inner(value)
    }
}

impl<T> From<ProcessingGraph<T>> for Graph<SchemaId, T>
where
    T: crate::graph::GraphNode + crate::graph::DiGraphNode,
{
    #[inline]
    fn from(value: ProcessingGraph<T>) -> Self {
        value.into_inner()
    }
}

// ============================================================================
//  SCHEMA GRAPH BUILDER
// ============================================================================

/// Builder for constructing a schema inheritance graph.
///
/// Generic over payload type `T`. Call `build::<N>()` to specify the node
/// wrapper type.
///
/// # Example
///
/// ```
/// use lithos_core::schema::{
///     identifier::SchemaId, inheritance::SchemaGraphBuilder,
/// };
///
/// let mut builder = SchemaGraphBuilder::<()>::new();
/// builder.add_node(SchemaId::new(), ());
/// let graph = builder.build::<()>();
/// ```
#[non_exhaustive]
pub struct SchemaGraphBuilder<T>(GraphBuilder<SchemaId, T>);

impl<T> SchemaGraphBuilder<T> {
    /// Creates a new graph builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(GraphBuilder::new())
    }

    /// Pre-allocates capacity for expected node count.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(GraphBuilder::with_capacity(capacity))
    }

    /// Adds a node to the graph with its payload.
    #[inline]
    pub fn add_node(&mut self, id: SchemaId, payload: T) {
        self.0.add_node(id, payload);
    }

    /// Adds a parent relationship (child extends parent).
    #[inline]
    pub fn add_parent(&mut self, child: SchemaId, parent: SchemaId) {
        self.0.add_parent(child, parent);
    }

    /// Builds the graph with normalized adjacency lists.
    ///
    /// The type parameter `N` specifies the node wrapper type (e.g., `Node<T>`
    /// for `()` payload, or `ProcessorNode<T>` for schema processing).
    #[inline]
    #[must_use]
    pub fn build<N>(self) -> ProcessingGraph<N>
    where
        N: crate::graph::GraphNode<Payload = T> + crate::graph::DiGraphNode,
    {
        ProcessingGraph::from(self.0.build::<N>())
    }

    /// Consumes self and returns the inner builder.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> GraphBuilder<SchemaId, T> {
        self.0
    }
}

impl<T> Default for SchemaGraphBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the set of all schemas affected by changes to the given schemas.
///
/// Performs BFS traversal from changed nodes to find all descendants (children,
/// grandchildren, etc.). This is useful for incremental processing where only
/// affected schemas need to be reprocessed.
///
/// # Example
///
/// ```
/// use std::collections::HashSet;
///
/// use lithos_core::schema::{
///     identifier::SchemaId,
///     inheritance::{SchemaGraphBuilder, affected_subtree},
/// };
///
/// let a = SchemaId::new();
/// let b = SchemaId::new();
/// let c = SchemaId::new();
///
/// let mut builder = SchemaGraphBuilder::new();
/// builder.add_node(a, ());
/// builder.add_node(b, ());
/// builder.add_node(c, ());
/// builder.add_parent(b, a); // B extends A
/// builder.add_parent(c, b); // C extends B
///
/// let graph = builder.build();
/// let changed = HashSet::from([a]); // A changed
/// let affected = affected_subtree(graph.as_inner(), &changed);
///
/// assert!(affected.contains(&a));
/// assert!(affected.contains(&b)); // B is affected (child of A)
/// assert!(affected.contains(&c)); // C is affected (grandchild of A)
/// ```
#[inline]
#[must_use]
#[expect(clippy::implicit_hasher, reason = "HashSet is appropriate here")]
pub fn affected_subtree<T>(
    graph: &Graph<SchemaId, T>,
    changed_ids: &HashSet<SchemaId>,
) -> HashSet<SchemaId>
where
    T: crate::graph::GraphNode + crate::graph::DiGraphNode,
{
    let mut affected = HashSet::with_capacity(changed_ids.len());
    let mut queue = VecDeque::new();

    #[expect(
        clippy::iter_over_hash_type,
        reason = "BFS traversal does not rely on iteration order"
    )]
    for &id in changed_ids {
        if affected.insert(id) {
            queue.push_back(id);
        }
    }

    while let Some(id) = queue.pop_front() {
        for &child_id in graph.children_of(id) {
            if affected.insert(child_id) {
                queue.push_back(child_id);
            }
        }
    }

    affected
}

// ============================================================================
//  TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::graph::NodeDepth;

    #[test]
    fn inheritance_graph_validates_dag() {
        let parent = SchemaId::new();
        let child = SchemaId::new();

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(parent, ());
        builder.add_node(child, ());
        builder.add_parent(child, parent);

        let graph = builder.build::<()>();
        let dag = InheritanceGraph::try_from(graph).expect("valid DAG");

        assert_eq!(dag.roots(), &[parent]);
        assert_eq!(dag.topo_order().first(), Some(&parent));
        assert!(dag.topo_order().contains(&child));
    }

    #[test]
    fn inheritance_graph_computes_depths() {
        let parent = SchemaId::new();
        let child = SchemaId::new();

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(parent, ());
        builder.add_node(child, ());
        builder.add_parent(child, parent);

        let graph = builder.build::<()>();
        let dag = InheritanceGraph::try_from(graph).expect("valid DAG");

        let underlying_dag = dag.to_dag_graph().expect("valid DAG");
        let depths =
            underlying_dag.graph().compute_depths(underlying_dag.topo_order());

        assert_eq!(depths.get(&parent), Some(&NodeDepth::ROOT));
        assert_eq!(depths.get(&child), Some(&NodeDepth::new(1)));
    }

    #[test]
    fn inheritance_graph_detects_cycles() {
        let a = SchemaId::new();
        let b = SchemaId::new();

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(a, ());
        builder.add_node(b, ());
        builder.add_parent(a, b);
        builder.add_parent(b, a); // Cycle!

        let graph = builder.build::<()>();
        let result = InheritanceGraph::try_from(graph);

        result.unwrap_err();
    }

    #[test]
    #[expect(
        clippy::many_single_char_names,
        reason = "Test setup uses single-letter node names"
    )]
    fn affected_subtree_computes_all_descendants() {
        let a = SchemaId::new();
        let b = SchemaId::new();
        let c = SchemaId::new();
        let d = SchemaId::new();
        let e = SchemaId::new();

        // Graph structure:
        //   A
        //   └─ B
        //      ├─ C
        //      │  └─ E
        //      └─ D
        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(a, ());
        builder.add_node(b, ());
        builder.add_node(c, ());
        builder.add_node(d, ());
        builder.add_node(e, ());

        builder.add_parent(b, a);
        builder.add_parent(c, b);
        builder.add_parent(d, b);
        builder.add_parent(e, c);

        let graph = builder.build::<()>();
        let changed = HashSet::from([b]);
        let affected = affected_subtree(graph.as_inner(), &changed);

        // B changed, so B, C, D, E are affected
        assert!(affected.contains(&b));
        assert!(affected.contains(&c));
        assert!(affected.contains(&d));
        assert!(affected.contains(&e));
        // A is not affected (it's a parent, not a descendant)
        assert!(!affected.contains(&a));
    }

    #[test]
    fn affected_subtree_handles_multiple_roots() {
        let a = SchemaId::new();
        let b = SchemaId::new();
        let c = SchemaId::new();

        let mut builder = SchemaGraphBuilder::new();
        builder.add_node(a, ());
        builder.add_node(b, ());
        builder.add_node(c, ());
        builder.add_parent(c, a);

        let graph = builder.build::<()>();
        let changed = HashSet::from([a, b]); // Two separate roots
        let affected = affected_subtree(graph.as_inner(), &changed);

        assert!(affected.contains(&a));
        assert!(affected.contains(&b));
        assert!(affected.contains(&c)); // Descendant of A
    }
}
