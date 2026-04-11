//! Directed acyclic graph (DAG) core types for schema inheritance.
//!
//! This module provides the foundational data structures for managing schema
//! inheritance hierarchies with support for multiple parents.
//!
//! # Architecture
//!
//! The module is organized around two core abstractions:
//!
//! - **`InheritanceNode`**: Minimal storage representation (id + depth)
//! - **`InheritanceGraph<T>`**: Container maintaining topological order and
//!   graph invariants
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
//! Construction is handled by converting a raw `Graph<T, R>` using
//! `TryFrom<Graph<T, R>>` or by using `InheritanceEditor` for scoped updates.
//!
//! ```rust
//! use lithos_core::schema::{
//!     aggregate::SchemaId,
//!     graph::{NodeAccessor, NodeDepth},
//!     inheritance::InheritanceNode,
//! };
//!
//! let root_id = SchemaId::new();
//! let child_id = SchemaId::new();
//! let root = InheritanceNode::new_root(root_id);
//! let child = InheritanceNode::new(child_id, NodeDepth::new(1));
//! assert_eq!(root.depth(), NodeDepth::ROOT);
//! assert_eq!(child.depth(), NodeDepth::new(1));
//! ```

#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv derives emit archived structs without non_exhaustive"
)]

use std::collections::{HashMap, HashSet, VecDeque};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{
    aggregate::SchemaId,
    error::{
        SchemaError, SchemaInheritanceError, SchemaLoaderError,
        SchemaResolutionError,
    },
    graph::{
        AdjacencyMap, ChildParentsMap, Graph, NodeAccessor, NodeDepth,
        NodeDepthMut,
    },
    topo_sort::{TopologicalOrder, TopologicalSorter},
};

type GraphParts<T> =
    (HashMap<SchemaId, T>, Vec<InheritanceEdge>, Vec<SchemaId>, Vec<SchemaId>);
type ScopedOrder =
    (TopologicalOrder, Vec<SchemaId>, HashMap<SchemaId, NodeDepth>);

/// Container for a topologically-ordered DAG.
///
/// **Invariants**:
/// - `nodes` contains all nodes indexed by ID
/// - `order` contains all node IDs in topological order (parents before
///   children)
/// - `roots` contains all nodes with no parents
/// - All parent/child references are bidirectional and consistent
///
/// **Generic Parameter**:
/// - `T = InheritanceNode` for storage
///
/// Ordering and roots are typically populated by [`GraphBuilder`] or
/// [`GraphEditor`].
///
/// # Examples
///
/// Construction is typically handled internally by [`GraphBuilder`] and
/// [`GraphEditor`] in `schema::graph`.
///
/// ```rust
/// use lithos_core::schema::inheritance::{InheritanceGraph, InheritanceNode};
///
/// fn takes_graph(_graph: &InheritanceGraph<InheritanceNode>) {}
///
/// # fn example(graph: &InheritanceGraph<InheritanceNode>) {
/// #     takes_graph(graph);
/// # }
/// ```

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InheritanceGraph<T> {
    nodes: HashMap<SchemaId, T>,
    edges: Vec<InheritanceEdge>,
    order: Vec<SchemaId>,
    roots: Vec<SchemaId>,
}

impl<T> InheritanceGraph<T> {
    /// Returns the node map.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &HashMap<SchemaId, T> {
        &self.nodes
    }

    /// Returns the edge list.
    #[inline]
    #[must_use]
    pub fn edges(&self) -> &[InheritanceEdge] {
        &self.edges
    }

    /// Returns node IDs in topological order.
    #[inline]
    #[must_use]
    pub fn order(&self) -> &[SchemaId] {
        &self.order
    }

    /// Returns the root node IDs.
    #[inline]
    #[must_use]
    pub fn roots(&self) -> &[SchemaId] {
        &self.roots
    }

    #[inline]
    #[must_use]
    pub(crate) fn into_parts(self) -> GraphParts<T> {
        (self.nodes, self.edges, self.order, self.roots)
    }

    pub(crate) fn map_payload<U, F>(
        &self,
        mut f: F,
    ) -> Result<InheritanceGraph<U>, SchemaInheritanceError>
    where
        F: FnMut(&T) -> U,
        U: NodeDepthMut,
    {
        let adjacency = AdjacencyMap::from_nodes_and_edges(
            self.nodes.keys().copied(),
            self.edges.iter().map(|edge| (edge.parent, edge.child)),
        );
        let raw_graph: Graph<U, ()> = Graph::<U, ()>::from_nodes_and_edges(
            &self.nodes,
            &adjacency,
            |node| f(node),
        );
        InheritanceGraph::try_from(raw_graph)
    }

    #[must_use]
    pub(crate) fn affected_subtree(
        &self,
        changed_ids: &HashSet<SchemaId>,
    ) -> HashSet<SchemaId> {
        let adjacency = AdjacencyMap::from_nodes_and_edges(
            self.nodes.keys().copied(),
            self.edges.iter().map(|edge| (edge.parent, edge.child)),
        );

        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "reachability traversal ignores iteration order"
        )]
        for &id in changed_ids {
            queue.push_back(id);
            affected.insert(id);
        }

        while let Some(id) = queue.pop_front() {
            for &child_id in adjacency.children_of(id) {
                if affected.insert(child_id) {
                    queue.push_back(child_id);
                }
            }
        }

        affected
    }
}

impl<T> InheritanceGraph<T>
where
    T: NodeAccessor,
{
    fn try_from_scoped<R>(
        &self,
        graph: &Graph<T, R>,
        affected: &HashSet<SchemaId>,
    ) -> Result<ScopedOrder, SchemaInheritanceError> {
        let adjacency = AdjacencyMap::from_graph(graph);
        let sorter = TopologicalSorter::try_new(
            graph.nodes().keys().copied(),
            &adjacency,
        )
        .map_err(|err| map_sorter_error(&err))?;
        let affected_order = sorter
            .sort_scoped(affected)
            .map_err(|err| map_sorter_error(&err))?;

        let depth_map = Graph::<T, R>::compute_depths(
            self.nodes(),
            affected,
            affected_order.order(),
            &adjacency,
        );
        let roots = sorter.sort().map_err(|err| map_sorter_error(&err))?;

        Ok((affected_order, roots.roots().to_vec(), depth_map))
    }
}

fn map_sorter_error(error: &SchemaResolutionError) -> SchemaInheritanceError {
    match *error {
        SchemaResolutionError::NotDirected => {
            SchemaInheritanceError::NotDirected
        }
        SchemaResolutionError::CycleDetected {
            ..
        }
        | SchemaResolutionError::DuplicateSchemaName {
            ..
        }
        | SchemaResolutionError::MissingNode {
            ..
        }
        | SchemaResolutionError::ParentNotFound {
            ..
        } => SchemaInheritanceError::CycleDetected {
            nodes: Vec::new(),
        },
    }
}

impl<T, R> TryFrom<Graph<T, R>> for InheritanceGraph<T>
where
    T: NodeDepthMut,
{
    type Error = SchemaInheritanceError;

    #[inline]
    fn try_from(graph: Graph<T, R>) -> Result<Self, Self::Error> {
        let (nodes, edges) = graph.into_parts();

        for edge in &edges {
            if !nodes.contains_key(&edge.from()) {
                return Err(SchemaInheritanceError::MissingNode {
                    id: edge.from(),
                });
            }
            if !nodes.contains_key(&edge.to()) {
                return Err(SchemaInheritanceError::MissingNode {
                    id: edge.to(),
                });
            }
        }

        let adjacency = AdjacencyMap::from_nodes_and_edges(
            nodes.keys().copied(),
            edges.iter().map(|edge| (edge.from(), edge.to())),
        );
        let sorter =
            TopologicalSorter::try_new(nodes.keys().copied(), &adjacency)
                .map_err(|err| map_sorter_error(&err))?;
        let order = sorter.sort().map_err(|err| map_sorter_error(&err))?;

        let depth_by_id = Graph::<T, R>::compute_depths::<InheritanceNode>(
            &HashMap::new(),
            &nodes.keys().copied().collect(),
            order.order(),
            &adjacency,
        );

        let mut nodes = nodes;
        #[expect(
            clippy::iter_over_hash_type,
            reason = "depth assignment ignores iteration order"
        )]
        for (id, depth) in &depth_by_id {
            if let Some(node) = nodes.get_mut(id) {
                node.set_depth(*depth);
                node.payload_mut().set_depth(*depth);
            }
        }

        let nodes: HashMap<SchemaId, T> = nodes
            .into_iter()
            .map(|(id, node)| (id, node.into_payload()))
            .collect();

        let edges = edges
            .into_iter()
            .map(|edge| InheritanceEdge {
                parent: edge.from(),
                child: edge.to(),
            })
            .collect();

        Ok(Self {
            nodes,
            edges,
            order: order.order().to_vec(),
            roots: order.roots().to_vec(),
        })
    }
}

/// Minimal stored node for the inheritance graph.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InheritanceNode {
    id: SchemaId,
    depth: NodeDepth,
}

impl InheritanceNode {
    /// Creates a new root node (depth 0).
    #[inline]
    #[must_use]
    pub fn new_root(id: SchemaId) -> Self {
        Self {
            id,
            depth: NodeDepth::ROOT,
        }
    }

    /// Creates a new node with specific depth.
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, depth: NodeDepth) -> Self {
        Self {
            id,
            depth,
        }
    }
}

impl NodeAccessor for InheritanceNode {
    /// Returns the node identifier.
    #[inline]
    fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the node depth.
    #[inline]
    fn depth(&self) -> NodeDepth {
        self.depth
    }
}

impl NodeDepthMut for InheritanceNode {
    #[inline]
    fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }
}

/// Directed edge in the inheritance graph.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InheritanceEdge {
    parent: SchemaId,
    child: SchemaId,
}

impl InheritanceEdge {
    /// Returns the parent schema ID.
    #[inline]
    #[must_use]
    pub fn parent(&self) -> SchemaId {
        self.parent
    }

    /// Returns the child schema ID.
    #[inline]
    #[must_use]
    pub fn child(&self) -> SchemaId {
        self.child
    }
}

// ============================================================================
//  INHERITANCE EDITOR
// ============================================================================

/// Editor for scoped inheritance graph updates.
pub struct InheritanceEditor<T> {
    base: InheritanceGraph<T>,
    nodes: HashMap<SchemaId, T>,
    edges: Vec<InheritanceEdge>,
    order: Vec<SchemaId>,
    roots: Vec<SchemaId>,
    changed_ids: HashSet<SchemaId>,
    deleted_ids: HashSet<SchemaId>,
}

impl<T> InheritanceEditor<T>
where
    T: NodeAccessor + NodeDepthMut + Clone,
{
    /// Creates a new editor from a validated base graph.
    #[must_use]
    #[inline]
    pub fn new(base: InheritanceGraph<T>) -> Self {
        let (nodes, edges, order, roots) = base.clone().into_parts();
        Self {
            base,
            nodes,
            edges,
            order,
            roots,
            changed_ids: HashSet::new(),
            deleted_ids: HashSet::new(),
        }
    }

    /// Inserts a node into the working set.
    #[inline]
    pub fn insert_node(&mut self, node: T) {
        self.nodes.insert(node.id(), node);
    }

    /// Applies a parent list change to a node.
    #[inline]
    pub fn apply_change(&mut self, id: SchemaId, mut parents: Vec<SchemaId>) {
        ChildParentsMap::normalize_parents(&mut parents);

        self.edges.retain(|edge| edge.child != id);
        for parent_id in parents {
            self.edges.push(InheritanceEdge {
                parent: parent_id,
                child: id,
            });
            self.changed_ids.insert(parent_id);
        }
        self.changed_ids.insert(id);
    }

    /// Deletes a node and its edges from the working set.
    #[inline]
    pub fn delete_node(&mut self, id: SchemaId) {
        self.nodes.remove(&id);
        self.edges.retain(|edge| edge.parent != id && edge.child != id);
        self.order.retain(|entry| *entry != id);
        self.roots.retain(|entry| *entry != id);
        self.deleted_ids.insert(id);
        self.changed_ids.insert(id);
    }

    /// Applies all queued changes and returns the updated graph.
    ///
    /// # Errors
    /// Returns a loader error if patching introduces a cycle or inconsistency.
    #[inline]
    pub fn patch(self) -> Result<InheritanceGraph<T>, SchemaLoaderError> {
        let mut nodes = self.nodes;
        let edges = self.edges;
        let base = self.base;

        let adjacency = AdjacencyMap::from_nodes_and_edges(
            nodes.keys().copied(),
            edges.iter().map(|edge| (edge.parent, edge.child)),
        );
        let mut affected = HashSet::new();
        let mut queue: VecDeque<SchemaId> = VecDeque::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "reachability traversal ignores iteration order"
        )]
        for &id in &self.changed_ids {
            affected.insert(id);
            queue.push_back(id);
        }

        while let Some(id) = queue.pop_front() {
            for &child in adjacency.children_of(id) {
                if affected.insert(child) {
                    queue.push_back(child);
                }
            }
        }

        if affected.is_empty() {
            return Ok(InheritanceGraph {
                nodes,
                edges,
                order: self.order,
                roots: self.roots,
            });
        }

        let raw_graph: Graph<T, ()> =
            Graph::<T, ()>::from_nodes_and_edges(&nodes, &adjacency, |node| {
                node.clone()
            });

        let (affected_order, roots, depth_map) =
            base.try_from_scoped(&raw_graph, &affected).map_err(|e| {
                SchemaLoaderError::Resolution(SchemaError::Inheritance(e))
            })?;

        #[expect(
            clippy::iter_over_hash_type,
            reason = "depth assignment ignores iteration order"
        )]
        for (id, depth) in depth_map {
            if let Some(node) = nodes.get_mut(&id) {
                node.set_depth(depth);
            }
        }

        let new_order = splice_order(
            &self.order,
            &affected_order,
            &affected,
            &adjacency,
            nodes.len(),
        )?;

        Ok(InheritanceGraph {
            nodes,
            edges,
            order: new_order,
            roots,
        })
    }
}

fn splice_order(
    existing: &[SchemaId],
    affected_order: &TopologicalOrder,
    affected: &HashSet<SchemaId>,
    adjacency: &AdjacencyMap,
    node_count: usize,
) -> Result<Vec<SchemaId>, SchemaLoaderError> {
    let mut anchor_map: HashMap<Option<SchemaId>, Vec<SchemaId>> =
        HashMap::new();

    for &id in affected_order.order() {
        let anchor = nearest_unaffected_ancestor(adjacency, id, affected);
        anchor_map.entry(anchor).or_default().push(id);
    }

    let mut new_order = Vec::with_capacity(node_count);
    for id in existing.iter().copied().filter(|id| !affected.contains(id)) {
        new_order.push(id);
        if let Some(mut list) = anchor_map.remove(&Some(id)) {
            new_order.append(&mut list);
        }
    }
    if let Some(mut list) = anchor_map.remove(&None) {
        new_order.append(&mut list);
    }
    for mut list in anchor_map.into_values() {
        new_order.append(&mut list);
    }

    if new_order.len() != node_count {
        return Err(SchemaLoaderError::Resolution(SchemaError::Inheritance(
            SchemaInheritanceError::CycleDetected {
                nodes: Vec::new(),
            },
        )));
    }

    Ok(new_order)
}

fn nearest_unaffected_ancestor(
    adjacency: &AdjacencyMap,
    id: SchemaId,
    affected: &HashSet<SchemaId>,
) -> Option<SchemaId> {
    for &parent in adjacency.parents_of(id) {
        if !affected.contains(&parent) {
            return Some(parent);
        }
        if let Some(ancestor) =
            nearest_unaffected_ancestor(adjacency, parent, affected)
        {
            return Some(ancestor);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: SchemaId,
        depth: NodeDepth,
    }

    impl NodeAccessor for TestNode {
        fn id(&self) -> SchemaId {
            self.id
        }

        fn depth(&self) -> NodeDepth {
            self.depth
        }
    }

    impl NodeDepthMut for TestNode {
        fn set_depth(&mut self, depth: NodeDepth) {
            self.depth = depth;
        }
    }

    #[test]
    fn try_from_computes_depths_and_roots() {
        let parent = SchemaId::new();
        let child = SchemaId::new();

        let mut graph: Graph<TestNode, ()> = Graph::new();
        graph.add_node(parent, TestNode {
            id: parent,
            depth: NodeDepth::ROOT,
        });
        graph.add_node(child, TestNode {
            id: child,
            depth: NodeDepth::ROOT,
        });
        graph.add_edge(parent, child);

        let validated = InheritanceGraph::try_from(graph).expect("valid graph");
        assert!(validated.order().starts_with(&[parent]));
        assert!(validated.roots.contains(&parent));

        let nodes = validated.nodes();
        assert_eq!(nodes.get(&parent).unwrap().depth(), NodeDepth::ROOT);
        assert_eq!(nodes.get(&child).unwrap().depth(), NodeDepth::new(1));
    }

    #[test]
    fn editor_recomputes_roots_for_removed_parent() {
        let parent = SchemaId::new();
        let child = SchemaId::new();

        let mut raw_graph: Graph<TestNode, ()> = Graph::new();
        raw_graph.add_node(parent, TestNode {
            id: parent,
            depth: NodeDepth::ROOT,
        });
        raw_graph.add_node(child, TestNode {
            id: child,
            depth: NodeDepth::ROOT,
        });
        raw_graph.add_edge(parent, child);

        let graph = InheritanceGraph::try_from(raw_graph).expect("valid graph");
        let mut editor = InheritanceEditor::new(graph);
        editor.apply_change(child, Vec::new());
        let updated = editor.patch().expect("patch graph");

        let mut roots = updated.roots.clone();
        roots.sort();
        let mut expected = vec![child, parent];
        expected.sort();
        assert_eq!(roots, expected);
    }
}
