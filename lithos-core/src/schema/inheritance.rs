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
//! - **`SchemaGraph<T>`**: Tuple newtype wrapper around `Graph<SchemaId, T>`
//! - **`SchemaGraphBuilder<T>`**: Tuple newtype wrapper around
//!   `GraphBuilder<SchemaId, T>`
//! - **`ProcessingDag<T>`**: Tuple newtype wrapper around `DagGraph<SchemaId,
//!   T>` for transient processing
//! - **`InheritanceGraph<T>`**: Serializable, persistence-focused DAG shape
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
//!     aggregate::SchemaId,
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
    ops::{Deref, DerefMut},
};

use crate::schema::aggregate::SchemaId;

/// Schema-specific graph for tracking inheritance relationships.
///
/// This tuple newtype wraps `Graph<SchemaId, T>` to provide a schema-focused
/// API boundary and allow schema-specific extensions.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SchemaGraph<T>(pub crate::graph::Graph<SchemaId, T>);

impl<T> SchemaGraph<T> {
    /// Returns parent IDs for a node (empty slice if none).
    #[inline]
    #[must_use]
    pub fn parents_of(&self, id: SchemaId) -> &[SchemaId] {
        self.0.parents_of(id)
    }

    /// Returns child IDs for a node (empty slice if none).
    #[inline]
    #[must_use]
    pub fn children_of(&self, id: SchemaId) -> &[SchemaId] {
        self.0.children_of(id)
    }

    /// Returns node by ID.
    #[inline]
    #[must_use]
    pub fn get(&self, id: SchemaId) -> Option<&crate::graph::Node<T>> {
        self.0.get(id)
    }

    /// Returns mutable node by ID.
    #[inline]
    pub fn get_mut(
        &mut self,
        id: SchemaId,
    ) -> Option<&mut crate::graph::Node<T>> {
        self.0.get_mut(id)
    }

    /// Iterates over all (id, node) pairs.
    #[inline]
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SchemaId, &crate::graph::Node<T>)> {
        self.0.iter()
    }

    /// Iterates over all (id, node) pairs with mutable access.
    #[inline]
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (SchemaId, &mut crate::graph::Node<T>)> {
        self.0.iter_mut()
    }

    /// Returns node count.
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.0.node_count()
    }

    /// Computes depths for all nodes using the provided topological order.
    #[inline]
    #[must_use]
    pub fn compute_depths(
        &self,
        order: &[SchemaId],
    ) -> HashMap<SchemaId, crate::graph::NodeDepth> {
        self.0.compute_depths(order)
    }
}

impl<T> Deref for SchemaGraph<T> {
    type Target = crate::graph::Graph<SchemaId, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SchemaGraph<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Builder for constructing a schema inheritance graph.
///
/// # Example
///
/// ```
/// use lithos_core::schema::{
///     aggregate::SchemaId, inheritance::SchemaGraphBuilder,
/// };
///
/// let mut builder = SchemaGraphBuilder::new();
/// builder.add_node(SchemaId::new(), ());
/// let graph = builder.build();
/// ```
#[non_exhaustive]
pub struct SchemaGraphBuilder<T>(pub crate::graph::GraphBuilder<SchemaId, T>);

impl<T> SchemaGraphBuilder<T> {
    /// Creates a new graph builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(crate::graph::GraphBuilder::new())
    }

    /// Pre-allocates capacity for expected node count.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(crate::graph::GraphBuilder::with_capacity(capacity))
    }

    /// Adds a node to the graph.
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
    #[inline]
    #[must_use]
    pub fn build(self) -> SchemaGraph<T> {
        SchemaGraph(self.0.build())
    }
}

impl<T> Default for SchemaGraphBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

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
///     aggregate::SchemaId,
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
        crate::graph::DagGraph<SchemaId, T>,
        crate::schema::error::SchemaInheritanceError,
    >
    where
        T: Clone,
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
        crate::graph::DagGraph::try_from(builder.build().0).map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })
    }

    /// For compatibility - returns a pseudo-graph interface.
    #[inline]
    #[must_use]
    pub fn graph(&self) -> GraphView<'_, T> {
        GraphView {
            inner: self,
        }
    }
}

/// View into the graph structure for compatibility.
#[expect(
    clippy::single_char_lifetime_names,
    reason = "Standard lifetime name for view"
)]
pub struct GraphView<'a, T>
where
    T: rkyv::Archive,
{
    inner: &'a InheritanceGraph<T>,
}

impl<T> GraphView<'_, T>
where
    T: rkyv::Archive,
{
    /// Returns an iterator over all (ID, payload) pairs in the graph.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (SchemaId, &T)> {
        self.inner.iter()
    }

    /// Returns the parent IDs for a given schema (empty slice if no parents).
    #[inline]
    #[must_use]
    pub fn parents_of(&self, id: SchemaId) -> &[SchemaId] {
        self.inner.parents_of(id)
    }

    /// Returns the child IDs for a given schema (empty slice if no children).
    #[inline]
    #[must_use]
    pub fn children_of(&self, id: SchemaId) -> &[SchemaId] {
        self.inner.children_of(id)
    }
}

impl<T> TryFrom<SchemaGraph<T>> for InheritanceGraph<T>
where
    T: rkyv::Archive + Clone,
{
    type Error = crate::schema::error::SchemaInheritanceError;

    #[inline]
    fn try_from(graph: SchemaGraph<T>) -> Result<Self, Self::Error> {
        // Validate it's a DAG
        let dag = crate::graph::DagGraph::try_from(graph.0).map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })?;

        // Extract topology before consuming
        let topo_order = dag.topo_order().to_vec();
        let roots = dag.roots().to_vec();

        // Extract serializable representation by cloning payloads
        let mut nodes = std::collections::HashMap::new();
        let mut parents = std::collections::HashMap::new();
        let mut children = std::collections::HashMap::new();

        for (id, node) in dag.graph().iter() {
            nodes.insert(id, node.payload().clone());
        }

        for (child_id, _) in dag.graph().iter() {
            let parent_ids = dag.graph().parents_of(child_id);
            if !parent_ids.is_empty() {
                parents.insert(child_id, parent_ids.to_vec());
            }
            let child_ids = dag.graph().children_of(child_id);
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
/// # Example
///
/// ```
/// use lithos_core::schema::{
///     aggregate::SchemaId,
///     inheritance::{ProcessingDag, SchemaGraphBuilder},
/// };
///
/// let root = SchemaId::new();
/// let child = SchemaId::new();
///
/// let mut builder = SchemaGraphBuilder::new();
/// builder.add_node(root, "payload");
/// builder.add_node(child, "payload");
/// builder.add_parent(child, root);
///
/// let dag = ProcessingDag::try_from(builder.build()).unwrap();
/// assert_eq!(dag.roots(), &[root]);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProcessingDag<T>(pub crate::graph::DagGraph<SchemaId, T>);

impl<T> ProcessingDag<T> {
    /// Returns the cached topological order.
    #[inline]
    #[must_use]
    pub fn topo_order(&self) -> &[SchemaId] {
        self.0.topo_order()
    }

    /// Returns the cached roots (nodes with no parents).
    #[inline]
    #[must_use]
    pub fn roots(&self) -> &[SchemaId] {
        self.0.roots()
    }

    /// Consumes self and returns the underlying `DagGraph`.
    #[inline]
    #[must_use]
    pub fn into_dag(self) -> crate::graph::DagGraph<SchemaId, T> {
        self.0
    }
}

impl<T> TryFrom<SchemaGraph<T>> for ProcessingDag<T> {
    type Error = crate::schema::error::SchemaInheritanceError;

    #[inline]
    fn try_from(graph: SchemaGraph<T>) -> Result<Self, Self::Error> {
        let dag = crate::graph::DagGraph::try_from(graph.0).map_err(|_e| {
            crate::schema::error::SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            }
        })?;
        Ok(Self(dag))
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
///     aggregate::SchemaId,
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
/// let affected = affected_subtree(&graph, &changed);
///
/// assert!(affected.contains(&a));
/// assert!(affected.contains(&b)); // B is affected (child of A)
/// assert!(affected.contains(&c)); // C is affected (grandchild of A)
/// ```
#[inline]
#[must_use]
#[expect(clippy::implicit_hasher, reason = "HashSet is appropriate here")]
pub fn affected_subtree<T>(
    graph: &crate::graph::Graph<SchemaId, T>,
    changed_ids: &HashSet<SchemaId>,
) -> HashSet<SchemaId> {
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

        let graph = builder.build();
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

        let graph = builder.build();
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

        let graph = builder.build();
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

        let graph = builder.build();
        let changed = HashSet::from([b]);
        let affected = affected_subtree(&graph, &changed);

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

        let graph = builder.build();
        let changed = HashSet::from([a, b]); // Two separate roots
        let affected = affected_subtree(&graph, &changed);

        assert!(affected.contains(&a));
        assert!(affected.contains(&b));
        assert!(affected.contains(&c)); // Descendant of A
    }
}
