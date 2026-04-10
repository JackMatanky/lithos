//! Directed acyclic graph (DAG) core types for schema inheritance.
//!
//! This module provides the foundational data structures for managing schema
//! inheritance hierarchies with support for multiple parents.
//!
//! # Architecture
//!
//! The module is organized around two core abstractions:
//!
//! - **`InheritanceNode`**: Minimal storage representation with bidirectional
//!   parent/child links
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
//! - Nodes can have **multiple parents** (`Vec<SchemaId>`)
//! - Depth = `max(parent_depths) + 1`
//!
//! # Usage
//!
//! Construction is typically handled internally by `GraphBuilder` and
//! `GraphEditor` in `schema::graph`.
//!
//! ```rust
//! use lithos_core::schema::{
//!     aggregate::SchemaId,
//!     inheritance::{InheritanceNode, NodeDepth},
//! };
//!
//! let root_id = SchemaId::new();
//! let child_id = SchemaId::new();
//! let root = InheritanceNode::new_root(root_id);
//! let child =
//!     InheritanceNode::new_child(child_id, vec![root_id], NodeDepth::ROOT);
//! assert!(root.is_root());
//! assert!(!child.is_root());
//! ```

#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv derives emit archived structs without non_exhaustive"
)]

use std::collections::{HashMap, HashSet, VecDeque};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{
    aggregate::SchemaId,
    error::{SchemaError, SchemaLoaderError, SchemaResolutionError},
};

type GraphParts<T> = (HashMap<SchemaId, T>, Vec<SchemaId>, Vec<SchemaId>);

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
    /// Node storage keyed by schema id.
    nodes: HashMap<SchemaId, T>,
    /// Node ids in topological order (parents before children).
    order: Vec<SchemaId>,
    /// Root node ids with no parents.
    roots: Vec<SchemaId>,
}

impl<T> InheritanceGraph<T> {
    #[inline]
    #[must_use]
    pub(crate) fn from_parts(
        nodes: HashMap<SchemaId, T>,
        order: Vec<SchemaId>,
        roots: Vec<SchemaId>,
    ) -> Self {
        Self {
            nodes,
            order,
            roots,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn nodes(&self) -> &HashMap<SchemaId, T> {
        &self.nodes
    }

    #[inline]
    #[must_use]
    pub(crate) fn order(&self) -> &[SchemaId] {
        &self.order
    }

    #[inline]
    #[must_use]
    pub(crate) fn roots(&self) -> &[SchemaId] {
        &self.roots
    }

    #[inline]
    #[must_use]
    pub(crate) fn into_parts(self) -> GraphParts<T> {
        (self.nodes, self.order, self.roots)
    }

    pub(crate) fn map_payload<U, F>(&self, mut f: F) -> InheritanceGraph<U>
    where
        F: FnMut(&T) -> U,
    {
        let mut nodes = HashMap::with_capacity(self.nodes.len());
        for id in self.order() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            nodes.insert(*id, f(node));
        }

        InheritanceGraph::from_parts(
            nodes,
            self.order.clone(),
            self.roots.clone(),
        )
    }
}

/// Minimal DAG node for database storage.
///
/// **Storage Layout** (typical: 2 parents, 3 children):
/// - `id`: 16 bytes (`SchemaId` is UUID)
/// - `parents`: 24 + 32 bytes (Vec header + 2 × 16)
/// - `children`: 24 + 48 bytes (Vec header + 3 × 16)
/// - `depth`: 8 bytes (`NodeDepth` wrapping usize) **Total**: ~152 bytes (vs
///   204 bytes with `SchemaInheritanceView`)
///
/// **Why no `name`?** Retrieved from Schema aggregate via `id`.
/// **Why no `file_path`?** Stored in processing payloads only.
///
/// # Examples
///
/// ```rust
/// use lithos_core::schema::{
///     aggregate::SchemaId,
///     inheritance::{InheritanceNode, NodeDepth},
/// };
///
/// let id = SchemaId::new();
/// let node = InheritanceNode::new_child(id, Vec::new(), NodeDepth::ROOT);
/// assert_eq!(node.id(), id);
/// ```
///
/// # Accessors
///
/// ```rust
/// use lithos_core::schema::{
///     aggregate::SchemaId,
///     inheritance::{InheritanceNode, NodeDepth},
/// };
///
/// let id = SchemaId::new();
/// let node = InheritanceNode::new_child(id, Vec::new(), NodeDepth::ROOT);
/// assert_eq!(node.id(), id);
/// assert_eq!(node.parents().len(), 0);
/// assert_eq!(node.children().len(), 0);
/// assert_eq!(node.depth().as_usize(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InheritanceNode {
    /// Schema identifier for this node.
    id: SchemaId,
    /// Parent schema identifiers (sorted, unique).
    parents: Vec<SchemaId>,
    /// Child schema identifiers (sorted, unique).
    children: Vec<SchemaId>,
    /// Cached inheritance depth for this node.
    depth: NodeDepth,
}

impl InheritanceNode {
    /// Create a new root node (no parents).
    #[inline]
    #[must_use]
    pub fn new_root(id: SchemaId) -> Self {
        Self {
            id,
            parents: Vec::new(),
            children: Vec::new(),
            depth: NodeDepth::ROOT,
        }
    }

    /// Create a new child node with given parents.
    #[inline]
    #[must_use]
    pub fn new_child(
        id: SchemaId,
        parents: Vec<SchemaId>,
        depth: NodeDepth,
    ) -> Self {
        Self {
            id,
            parents,
            children: Vec::new(),
            depth,
        }
    }

    /// Check if this is a root node.
    #[inline]
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Returns the schema identifier for this node.
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "accessor name matches trait method"
    )]
    pub fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the parent schema identifiers (sorted, unique).
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "accessor name matches trait method"
    )]
    pub fn parents(&self) -> &[SchemaId] {
        &self.parents
    }

    /// Returns the child schema identifiers (sorted, unique).
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "accessor name matches trait method"
    )]
    pub fn children(&self) -> &[SchemaId] {
        &self.children
    }

    /// Returns the cached inheritance depth for this node.
    #[inline]
    #[must_use]
    #[expect(
        clippy::same_name_method,
        reason = "accessor name matches trait method"
    )]
    pub fn depth(&self) -> NodeDepth {
        self.depth
    }

    // GraphNode trait provides mutation hooks used by builders/editors.
}

/// Trait for mutating inheritance fields on graph nodes.
///
/// Builders and editors operate on `GraphNode` to guarantee structural
/// updates remain consistent.
pub trait GraphNode: NodeAccessor {
    /// Sets the cached inheritance depth for this node.
    fn set_depth(&mut self, depth: NodeDepth);
    /// Updates node edges (parents and children) atomically.
    fn set_edges(&mut self, parents: Vec<SchemaId>, children: Vec<SchemaId>);
}

/// Trait for accessing inheritance fields from generic node types.
pub trait NodeAccessor {
    /// Returns child schema ids.
    fn children(&self) -> &[SchemaId];
    /// Returns the inheritance depth of this node.
    fn depth(&self) -> NodeDepth;
    /// Returns the schema id for this node.
    fn id(&self) -> SchemaId;
    /// Returns parent schema ids.
    fn parents(&self) -> &[SchemaId];
}

impl NodeAccessor for InheritanceNode {
    #[inline]
    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    #[inline]
    fn depth(&self) -> NodeDepth {
        self.depth
    }

    #[inline]
    fn id(&self) -> SchemaId {
        self.id
    }

    #[inline]
    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }
}

impl GraphNode for InheritanceNode {
    #[inline]
    fn set_edges(&mut self, parents: Vec<SchemaId>, children: Vec<SchemaId>) {
        self.parents = parents;
        self.children = children;
    }

    #[inline]
    fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }
}

impl<T: NodeAccessor> InheritanceGraph<T> {
    /// Compute all descendants of the given nodes (BFS).
    #[expect(
        clippy::excessive_nesting,
        reason = "traversal keeps nesting explicit for clarity"
    )]
    #[must_use]
    pub(crate) fn affected_subtree(
        &self,
        changed_ids: &HashSet<SchemaId>,
    ) -> HashSet<SchemaId> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "traversal order is irrelevant for reachability"
        )]
        for &id in changed_ids {
            queue.push_back(id);
            affected.insert(id);
        }

        while let Some(id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for &child_id in node.children() {
                    if affected.insert(child_id) {
                        queue.push_back(child_id);
                    }
                }
            }
        }

        affected
    }

    /// Validate bidirectional consistency (debug helper).
    ///
    /// # Errors
    ///
    /// Returns error if any inconsistency is found.
    #[cfg(debug_assertions)]
    pub(crate) fn validate_consistency(&self) -> Result<(), SchemaLoaderError> {
        #[expect(
            clippy::iter_over_hash_type,
            reason = "validation covers all nodes; order is irrelevant"
        )]
        for (id, node) in &self.nodes {
            // Check parent → child links
            for &parent_id in node.parents() {
                let parent = self.nodes.get(&parent_id).ok_or({
                    SchemaLoaderError::Resolution(SchemaError::Resolution(
                        SchemaResolutionError::MissingNode {
                            id: parent_id,
                        },
                    ))
                })?;
                if !parent.children().contains(id) {
                    return Err(SchemaLoaderError::Ingestion(
                        crate::schema::error::SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: format!(
                                    "graph invariant violated: parent \
                                     {parent_id} missing child {id} in \
                                     children list"
                                )
                                .into(),
                            },
                        ),
                    ));
                }
            }

            // Check child → parent links
            for &child_id in node.children() {
                let child = self.nodes.get(&child_id).ok_or({
                    SchemaLoaderError::Resolution(SchemaError::Resolution(
                        SchemaResolutionError::MissingNode {
                            id: child_id,
                        },
                    ))
                })?;
                if !child.parents().contains(id) {
                    return Err(SchemaLoaderError::Ingestion(
                        crate::schema::error::SchemaIngestionError::File(
                            crate::schema::error::SchemaFileError::FileSystem {
                                reason: format!(
                                    "graph invariant violated: child \
                                     {child_id} missing parent {id} in \
                                     parents list"
                                )
                                .into(),
                            },
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Inheritance depth in the DAG (0-indexed for roots).
///
/// - Root nodes: `depth = 0`
/// - Child nodes: `depth = max(parent_depths) + 1`
///
/// This newtype enforces type safety and prevents mixing depth values with
/// other counts.
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

    /// Create a new depth value.
    #[inline]
    #[must_use]
    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }

    /// Extract the underlying usize value.
    #[inline]
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Increment depth by 1 (saturating).
    #[inline]
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    fn build_diamond_graph()
    -> (InheritanceGraph<InheritanceNode>, Vec<SchemaId>) {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();
        let id_c = SchemaId::new();
        let id_d = SchemaId::new();

        let mut node_a = InheritanceNode::new_root(id_a);
        let mut node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);
        let mut node_c =
            InheritanceNode::new_child(id_c, vec![id_a], NodeDepth::ROOT);
        let node_d =
            InheritanceNode::new_child(id_d, vec![id_b, id_c], NodeDepth::ROOT);

        node_a.set_edges(Vec::new(), vec![id_b, id_c]);
        node_b.set_edges(vec![id_a], vec![id_d]);
        node_c.set_edges(vec![id_a], vec![id_d]);

        let nodes = HashMap::from([
            (id_a, node_a),
            (id_b, node_b),
            (id_c, node_c),
            (id_d, node_d),
        ]);

        let graph = InheritanceGraph::from_parts(nodes, Vec::new(), Vec::new());

        (graph, vec![id_a, id_b, id_c, id_d])
    }

    #[test]
    fn affected_subtree_includes_all_descendants() {
        let (graph, ids) = build_diamond_graph();
        let id_a = *ids.first().expect("id a");

        let changed: HashSet<SchemaId> = HashSet::from([id_a]);
        let affected = graph.affected_subtree(&changed);

        for id in ids {
            assert!(affected.contains(&id));
        }
    }
}
