//! Directed acyclic graph (DAG) structures for schema inheritance.
//!
//! This module provides the foundational data structures and algorithms for
//! managing schema inheritance hierarchies with support for multiple parents.
//!
//! # Architecture
//!
//! The module is organized around three core abstractions:
//!
//! - **`InheritanceNode`**: Minimal storage representation with bidirectional
//!   parent/child links
//! - **`GraphNode<T>`**: Processing representation with generic payload for
//!   pipeline stages
//! - **`TopologicalGraph<T>`**: Container maintaining topological order and
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
//! - Topological sort uses Kahn's algorithm (works with multiple parents)
//! - Cycle detection via DFS ensures acyclicity
//!
//! # Helper Structs
//!
//! - **`DagValidator`**: Stateful cycle detection with DFS
//! - **`DagBuilder`**: Constructs graphs from `FileStatus` map
//!
//! # Usage
//!
//! ```ignore
//! use crate::schema::graph::{DagBuilder, TopologicalGraph, InheritanceNode};
//!
//! // Build graph from file statuses
//! let graph = DagBuilder::new(&statuses).build()?;
//!
//! // Query operations
//! let affected = graph.affected_subtree(&changed_ids);
//! let (order, roots) = graph.topological_sort()?;
//!
//! // Mutations (InheritanceNode only)
//! graph.compute_depths();
//! graph.set_parents(node_id, vec![parent1, parent2])?;
//! ```

#![expect(
    clippy::missing_inline_in_public_items,
    reason = "public API favors readability over forced inlining"
)]
#![expect(
    clippy::iter_over_hash_type,
    reason = "graph algorithms do not rely on HashMap iteration order"
)]
// rkyv derives emit archived structs that trigger exhaustive_structs.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv derives emit archived structs without non_exhaustive"
)]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{
    aggregate::{SchemaId, SchemaName},
    error::{SchemaError, SchemaLoaderError, SchemaResolutionError},
    raw::{RawFileTimes, RawSchema},
    schema_pipeline::FileStatus,
    views::RawSchemaView,
};

// ═════════════════════════════════════════════════════════════════════════════
//  CORE TYPES
// ═════════════════════════════════════════════════════════════════════════════

/// Container for a topologically-ordered DAG.
///
/// **Invariants**:
/// - `order` contains all node IDs in topological order (parents before
///   children)
/// - `nodes` contains all nodes indexed by ID
/// - `roots` contains all nodes with no parents
/// - All parent/child references are bidirectional and consistent
///
/// **Generic Parameter**:
/// - `T = InheritanceNode` for storage
/// - `T = GraphNode<Payload>` for processing
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TopologicalGraph<T> {
    /// Node ids in topological order (parents before children).
    pub order: Vec<SchemaId>,
    /// Node storage keyed by schema id.
    pub nodes: HashMap<SchemaId, T>,
    /// Root node ids with no parents.
    pub roots: Vec<SchemaId>,
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
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InheritanceNode {
    /// Schema identifier for this node.
    pub id: SchemaId,
    /// Parent schema identifiers (sorted, unique).
    pub parents: Vec<SchemaId>,
    /// Child schema identifiers (sorted, unique).
    pub children: Vec<SchemaId>,
    /// Cached inheritance depth for this node.
    pub depth: NodeDepth,
}

impl InheritanceNode {
    /// Create a new root node (no parents).
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
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Add a parent to this node (maintains sorted order).
    pub fn add_parent(&mut self, parent_id: SchemaId) {
        if !self.parents.contains(&parent_id) {
            self.parents.push(parent_id);
            self.parents.sort();
        }
    }

    /// Remove a parent from this node.
    pub fn remove_parent(&mut self, parent_id: SchemaId) {
        self.parents.retain(|id| *id != parent_id);
    }

    /// Add a child to this node (maintains sorted order).
    pub fn add_child(&mut self, child_id: SchemaId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
            self.children.sort();
        }
    }

    /// Remove a child from this node.
    pub fn remove_child(&mut self, child_id: SchemaId) {
        self.children.retain(|id| *id != child_id);
    }

    /// Convert to processing representation with given payload.
    #[must_use]
    pub fn with_payload<T>(self, payload: T) -> GraphNode<T> {
        GraphNode {
            id: self.id,
            parents: self.parents,
            children: self.children,
            depth: self.depth,
            payload,
        }
    }
}

/// Processing node with generic payload for pipeline stages.
///
/// **Shape matches `InheritanceNode` + payload**:
/// - Same `id`, `parents`, `children`, `depth` fields
/// - Additional `payload` for stage-specific data
///
/// Used during schema pipeline, then converted to `InheritanceNode` for
/// storage.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GraphNode<T> {
    /// Schema identifier for this node.
    pub id: SchemaId,
    /// Parent schema identifiers (sorted, unique).
    pub parents: Vec<SchemaId>,
    /// Child schema identifiers (sorted, unique).
    pub children: Vec<SchemaId>,
    /// Cached inheritance depth for this node.
    pub depth: NodeDepth,
    /// Stage-specific payload attached to this node.
    pub payload: T,
}

impl<T> GraphNode<T> {
    /// Convert to storage representation (discards payload).
    #[must_use]
    pub fn to_inheritance_node(self) -> InheritanceNode {
        InheritanceNode {
            id: self.id,
            parents: self.parents,
            children: self.children,
            depth: self.depth,
        }
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
    #[must_use]
    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }

    /// Extract the underlying usize value.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Increment depth by 1 (saturating).
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
type TopologicalOrder = (Vec<SchemaId>, Vec<SchemaId>);

// ═════════════════════════════════════════════════════════════════════════════
//  PAYLOAD STRUCTS
// ═════════════════════════════════════════════════════════════════════════════

/// Payload for fresh nodes (no rebuild needed).
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Fresh;

/// Payload for nodes with stale timestamps.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StaleTimestamps {
    /// Current file timestamps from the filesystem.
    pub times: RawFileTimes,
    /// Cached raw schema view from storage.
    pub view: RawSchemaView,
}

/// Payload for nodes with stale content.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StaleContent {
    /// Current file timestamps from the filesystem.
    pub times: RawFileTimes,
    /// Content hash of the file contents.
    pub content_hash: [u8; 32],
    /// Parsed raw schema from the filesystem.
    pub raw: RawSchema,
    /// Cached raw schema view from storage.
    pub view: RawSchemaView,
}

/// Payload for new nodes.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct New {
    /// Path to the schema file on disk.
    pub file_path: PathBuf,
    /// Current file timestamps from the filesystem.
    pub times: RawFileTimes,
    /// Content hash of the file contents.
    pub content_hash: [u8; 32],
    /// Parsed raw schema from the filesystem.
    pub raw: RawSchema,
}

// ═════════════════════════════════════════════════════════════════════════════
//  TOPOLOGICAL GRAPH METHODS (Generic over any T)
// ═════════════════════════════════════════════════════════════════════════════

/// Trait for accessing inheritance fields from generic node types.
pub trait InheritanceAccess {
    /// Returns child schema ids.
    fn children(&self) -> &[SchemaId];
    /// Returns cached depth for this node.
    fn depth(&self) -> NodeDepth;
    /// Returns the schema id for this node.
    fn id(&self) -> SchemaId;
    /// Returns parent schema ids.
    fn parents(&self) -> &[SchemaId];
}

impl InheritanceAccess for InheritanceNode {
    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn depth(&self) -> NodeDepth {
        self.depth
    }

    fn id(&self) -> SchemaId {
        self.id
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }
}

impl<T> InheritanceAccess for GraphNode<T> {
    fn children(&self) -> &[SchemaId] {
        &self.children
    }

    fn depth(&self) -> NodeDepth {
        self.depth
    }

    fn id(&self) -> SchemaId {
        self.id
    }

    fn parents(&self) -> &[SchemaId] {
        &self.parents
    }
}

impl<T: InheritanceAccess> TopologicalGraph<T> {
    /// Compute topological order using Kahn's algorithm.
    ///
    /// Returns the order (parents before children) and root nodes.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if the graph contains a cycle.
    #[expect(
        clippy::excessive_nesting,
        reason = "graph traversal keeps nesting explicit for clarity"
    )]
    pub fn topological_sort(
        &self,
    ) -> Result<TopologicalOrder, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = self
            .nodes
            .values()
            .map(|node| (node.id(), node.parents().len()))
            .collect();

        let mut queue: VecDeque<SchemaId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        let roots: Vec<SchemaId> = queue.iter().copied().collect();

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(node) = self.nodes.get(&node_id) {
                for &child_id in node.children() {
                    if let Some(deg) = in_degree.get_mut(&child_id) {
                        *deg = deg.saturating_sub(1);

                        if *deg == 0 {
                            queue.push_back(child_id);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok((order, roots))
    }

    /// Topological sort for only the affected subtree.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if the affected subtree contains a cycle.
    #[expect(
        clippy::excessive_nesting,
        reason = "scoped traversal keeps nesting explicit for clarity"
    )]
    pub fn topological_sort_scoped(
        &self,
        affected: &HashSet<SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = HashMap::new();

        for &id in affected {
            if let Some(node) = self.nodes.get(&id) {
                let parent_in_scope_count = node
                    .parents()
                    .iter()
                    .filter(|pid| affected.contains(pid))
                    .count();
                in_degree.insert(id, parent_in_scope_count);
            }
        }

        let mut queue: VecDeque<SchemaId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(affected.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(node) = self.nodes.get(&node_id) {
                for &child_id in node.children() {
                    if !affected.contains(&child_id) {
                        continue;
                    }

                    if let Some(deg) = in_degree.get_mut(&child_id) {
                        *deg = deg.saturating_sub(1);

                        if *deg == 0 {
                            queue.push_back(child_id);
                        }
                    }
                }
            }
        }

        if order.len() != affected.len() {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok(order)
    }

    /// Compute all descendants of the given nodes (BFS).
    #[expect(
        clippy::excessive_nesting,
        reason = "traversal keeps nesting explicit for clarity"
    )]
    #[must_use]
    pub fn affected_subtree(
        &self,
        changed_ids: &HashSet<SchemaId>,
    ) -> HashSet<SchemaId> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

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

    /// Remove nodes from the graph and update order/roots.
    pub fn prune(&mut self, deleted_ids: &[SchemaId]) {
        for id in deleted_ids {
            self.nodes.remove(id);
        }
        self.order.retain(|id| self.nodes.contains_key(id));
        self.roots.retain(|id| self.nodes.contains_key(id));
    }

    /// Splice affected subtree order into stable graph order.
    ///
    /// Maintains stable positions for unaffected nodes while inserting
    /// affected nodes in topological order relative to their nearest
    /// unaffected ancestor.
    ///
    /// # Errors
    ///
    /// Returns error if the resulting order doesn't match the node count
    /// (indicates a cycle or logic error).
    pub fn splice_order(
        &mut self,
        affected_order: &[SchemaId],
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaLoaderError> {
        let mut anchor_map: HashMap<Option<SchemaId>, Vec<SchemaId>> =
            HashMap::new();

        for &id in affected_order {
            let anchor = self.nearest_unaffected_ancestor(id, affected);
            anchor_map.entry(anchor).or_default().push(id);
        }

        let capacity = self.order.len().saturating_add(affected.len());
        let mut new_order = Vec::with_capacity(capacity);
        for id in self.order.iter().copied().filter(|id| !affected.contains(id))
        {
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

        if new_order.len() != self.nodes.len() {
            return Err(SchemaLoaderError::Resolution(
                crate::schema::error::SchemaError::Resolution(
                    SchemaResolutionError::CycleDetected {
                        schemas: Vec::new(),
                    },
                ),
            ));
        }

        self.order = new_order;
        Ok(())
    }

    fn nearest_unaffected_ancestor(
        &self,
        id: SchemaId,
        affected: &HashSet<SchemaId>,
    ) -> Option<SchemaId> {
        let node = self.nodes.get(&id)?;

        for &parent_id in node.parents() {
            if !affected.contains(&parent_id) {
                return Some(parent_id);
            }
            // Recursively check grandparents
            if let Some(ancestor) =
                self.nearest_unaffected_ancestor(parent_id, affected)
            {
                return Some(ancestor);
            }
        }
        None
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TOPOLOGICAL GRAPH METHODS (InheritanceNode only - requires mutation)
// ═════════════════════════════════════════════════════════════════════════════

impl TopologicalGraph<InheritanceNode> {
    /// Compute depths for all nodes: depth = `max(parent_depths)` + 1.
    #[expect(
        clippy::excessive_nesting,
        reason = "depth computation keeps nesting explicit for clarity"
    )]
    pub fn compute_depths(&mut self) {
        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = self
            .nodes
            .values()
            .filter(|node| node.is_root())
            .map(|node| node.id)
            .collect();

        for &root_id in &queue {
            depths.insert(root_id, 0);
        }

        while let Some(id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for &child_id in &node.children {
                    let Some(child) = self.nodes.get(&child_id) else {
                        debug_assert!(
                            false,
                            "graph invariant violated: missing child node"
                        );
                        continue;
                    };

                    let max_parent_depth = child
                        .parents
                        .iter()
                        .filter_map(|pid| depths.get(pid).copied())
                        .max()
                        .unwrap_or(0);

                    if child.parents.iter().all(|pid| depths.contains_key(pid))
                    {
                        depths.insert(
                            child_id,
                            max_parent_depth.saturating_add(1),
                        );
                        queue.push_back(child_id);
                    }
                }
            }
        }

        for (id, depth) in depths {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.depth = NodeDepth::new(depth);
            }
        }
    }

    /// Compute depths only for affected subtree.
    #[expect(
        clippy::excessive_nesting,
        reason = "depth computation keeps nesting explicit for clarity"
    )]
    pub fn compute_depths_scoped(&mut self, affected: &HashSet<SchemaId>) {
        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = VecDeque::new();

        for &id in affected {
            let Some(node) = self.nodes.get(&id) else {
                debug_assert!(
                    false,
                    "graph invariant violated: missing affected node"
                );
                continue;
            };
            let all_parents_unaffected =
                node.parents.iter().all(|parent| !affected.contains(parent));

            if all_parents_unaffected {
                let depth = node
                    .parents
                    .iter()
                    .filter_map(|parent| {
                        self.nodes
                            .get(parent)
                            .map(|parent_node| parent_node.depth.as_usize())
                    })
                    .max()
                    .map_or(0, |d| d.saturating_add(1));
                depths.insert(id, depth);
                queue.push_back(id);
            }
        }

        while let Some(id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for &child_id in &node.children {
                    if !affected.contains(&child_id) {
                        continue;
                    }
                    let Some(child) = self.nodes.get(&child_id) else {
                        debug_assert!(
                            false,
                            "graph invariant violated: missing child node"
                        );
                        continue;
                    };

                    let max_parent_depth = child
                        .parents
                        .iter()
                        .filter_map(|pid| depths.get(pid).copied())
                        .max()
                        .unwrap_or(0);

                    if child.parents.iter().all(|pid| depths.contains_key(pid))
                    {
                        depths.insert(
                            child_id,
                            max_parent_depth.saturating_add(1),
                        );
                        queue.push_back(child_id);
                    }
                }
            }
        }

        for (id, depth) in depths {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.depth = NodeDepth::new(depth);
            }
        }
    }

    /// Apply a parent change, maintaining bidirectional consistency.
    ///
    /// # Errors
    ///
    /// Returns error if node or parent not found.
    pub fn set_parents(
        &mut self,
        node_id: SchemaId,
        new_parents: Vec<SchemaId>,
    ) -> Result<(), SchemaLoaderError> {
        let old_parents = self
            .nodes
            .get(&node_id)
            .ok_or({
                SchemaLoaderError::Resolution(
                    crate::schema::error::SchemaError::Resolution(
                        SchemaResolutionError::MissingNode {
                            id: node_id,
                        },
                    ),
                )
            })?
            .parents
            .clone();

        // Remove node from old parents' children
        for old_parent in &old_parents {
            if let Some(parent_node) = self.nodes.get_mut(old_parent) {
                parent_node.remove_child(node_id);
            }
        }

        // Add node to new parents' children
        for &new_parent in &new_parents {
            let parent_node = self.nodes.get_mut(&new_parent).ok_or({
                SchemaLoaderError::Resolution(
                    crate::schema::error::SchemaError::Resolution(
                        SchemaResolutionError::MissingNode {
                            id: new_parent,
                        },
                    ),
                )
            })?;
            parent_node.add_child(node_id);
        }

        // Update node's parents
        let node = self.nodes.get_mut(&node_id).ok_or({
            SchemaLoaderError::Resolution(
                crate::schema::error::SchemaError::Resolution(
                    SchemaResolutionError::MissingNode {
                        id: node_id,
                    },
                ),
            )
        })?;
        node.parents = new_parents;

        Ok(())
    }

    /// Validate bidirectional consistency (debug helper).
    ///
    /// # Errors
    ///
    /// Returns error if any inconsistency is found.
    #[cfg(debug_assertions)]
    pub fn validate_consistency(&self) -> Result<(), SchemaLoaderError> {
        for (id, node) in &self.nodes {
            // Check parent → child links
            for &parent_id in &node.parents {
                let parent = self.nodes.get(&parent_id).ok_or({
                    SchemaLoaderError::Resolution(
                        crate::schema::error::SchemaError::Resolution(
                            SchemaResolutionError::MissingNode {
                                id: parent_id,
                            },
                        ),
                    )
                })?;
                if !parent.children.contains(id) {
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
            for &child_id in &node.children {
                let child = self.nodes.get(&child_id).ok_or({
                    SchemaLoaderError::Resolution(
                        crate::schema::error::SchemaError::Resolution(
                            SchemaResolutionError::MissingNode {
                                id: child_id,
                            },
                        ),
                    )
                })?;
                if !child.parents.contains(id) {
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

// ═════════════════════════════════════════════════════════════════════════════
//  DAG VALIDATOR (Cycle Detection)
// ═════════════════════════════════════════════════════════════════════════════

/// DAG validator for cycle detection and structural validation.
///
/// This struct maintains temporary state during validation and should be
/// created, used, and dropped for each validation pass.
pub struct DagValidator<'graph> {
    nodes: &'graph HashMap<SchemaId, InheritanceNode>,
    visited: HashSet<SchemaId>,
    in_progress: HashSet<SchemaId>,
}

impl<'graph> DagValidator<'graph> {
    /// Create a new validator for the given nodes.
    #[must_use]
    pub fn new(nodes: &'graph HashMap<SchemaId, InheritanceNode>) -> Self {
        Self {
            nodes,
            visited: HashSet::with_capacity(nodes.len()),
            in_progress: HashSet::new(),
        }
    }

    /// Detect cycles in the entire graph.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if a cycle is found.
    pub fn detect_cycles(&mut self) -> Result<(), SchemaResolutionError> {
        let ids: Vec<SchemaId> = self.nodes.keys().copied().collect();
        for node_id in ids {
            self.visit(node_id)?;
        }
        Ok(())
    }

    /// Detect cycles only in the affected subtree.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if a cycle is found in the affected subtree.
    pub fn detect_cycles_scoped(
        &mut self,
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaResolutionError> {
        for &node_id in affected {
            self.visit_scoped(node_id, affected)?;
        }
        Ok(())
    }

    fn visit(
        &mut self,
        node_id: SchemaId,
    ) -> Result<(), SchemaResolutionError> {
        if self.visited.contains(&node_id) {
            return Ok(());
        }

        if !self.in_progress.insert(node_id) {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: vec![],
            });
        }

        if let Some(node) = self.nodes.get(&node_id) {
            for &parent_id in &node.parents {
                self.visit(parent_id)?;
            }
        }

        self.in_progress.remove(&node_id);
        self.visited.insert(node_id);
        Ok(())
    }

    fn visit_scoped(
        &mut self,
        node_id: SchemaId,
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaResolutionError> {
        if self.visited.contains(&node_id) {
            return Ok(());
        }

        if !self.in_progress.insert(node_id) {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: vec![],
            });
        }

        if let Some(node) = self.nodes.get(&node_id) {
            for &parent_id in &node.parents {
                if affected.contains(&parent_id) {
                    self.visit_scoped(parent_id, affected)?;
                }
            }
        }

        self.in_progress.remove(&node_id);
        self.visited.insert(node_id);
        Ok(())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DAG BUILDER (Graph Construction from FileStatus)
// ═════════════════════════════════════════════════════════════════════════════

/// Builder for constructing `TopologicalGraph` from pipeline state.
///
/// This struct encapsulates the complex logic of building a graph from
/// file statuses, resolving parent relationships, and validating the result.
pub(crate) struct DagBuilder<'status> {
    statuses: &'status HashMap<SchemaId, FileStatus>,
}

impl<'status> DagBuilder<'status> {
    /// Create a new builder from file statuses.
    #[must_use]
    pub(crate) fn new(
        statuses: &'status HashMap<SchemaId, FileStatus>,
    ) -> Self {
        Self {
            statuses,
        }
    }

    /// Build a complete graph from statuses.
    ///
    /// # Errors
    ///
    /// Returns error if graph construction fails (e.g., cycle detected, missing
    /// parent).
    pub(crate) fn build(
        self,
    ) -> Result<TopologicalGraph<InheritanceNode>, SchemaLoaderError> {
        let index = self.build_name_index()?;
        let mut nodes: HashMap<SchemaId, InheritanceNode> =
            HashMap::with_capacity(self.statuses.len());

        // Build initial nodes with parents resolved
        for (id, status) in self.statuses {
            let parents = Self::resolve_parents(status, &index)?;

            nodes.insert(*id, InheritanceNode {
                id: *id,
                parents,
                children: Vec::new(),
                depth: NodeDepth::ROOT,
            });
        }

        // Build bidirectional children links
        Self::build_children(&mut nodes);

        // Validate no cycles
        let mut validator = DagValidator::new(&nodes);
        validator.detect_cycles().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;

        // Compute depths
        let mut graph = TopologicalGraph {
            nodes,
            order: Vec::new(),
            roots: Vec::new(),
        };
        graph.compute_depths();

        // Compute topological order
        let (order, roots) = graph.topological_sort().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;
        graph.order = order;
        graph.roots = roots;

        Ok(graph)
    }

    fn build_name_index(
        &self,
    ) -> Result<HashMap<SchemaName, SchemaId>, SchemaLoaderError> {
        let mut index = HashMap::with_capacity(self.statuses.len());

        for (id, status) in self.statuses {
            let name = if let Some(raw) = status.raw() {
                SchemaName::try_new(raw.name())
                    .map_err(SchemaLoaderError::Resolution)?
            } else if let Some(view) = status.view() {
                SchemaName::try_new(view.name())
                    .map_err(SchemaLoaderError::Resolution)?
            } else {
                return Err(SchemaLoaderError::Ingestion(
                    crate::schema::error::SchemaIngestionError::Storage(
                        crate::schema::error::SchemaStorageError::NotFound {
                            name: status.path().to_string_lossy().into(),
                        },
                    ),
                ));
            };

            if index.insert(name.clone(), *id).is_some() {
                return Err(SchemaLoaderError::Resolution(
                    crate::schema::error::SchemaError::Resolution(
                        SchemaResolutionError::DuplicateSchemaName {
                            name: name.as_ref().into(),
                        },
                    ),
                ));
            }
        }

        Ok(index)
    }

    fn resolve_parents(
        status: &FileStatus,
        index: &HashMap<SchemaName, SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaLoaderError> {
        let child_name = if let Some(raw) = status.raw() {
            SchemaName::try_new(raw.name())
                .map_err(SchemaLoaderError::Resolution)?
        } else if let Some(view) = status.view() {
            SchemaName::try_new(view.name())
                .map_err(SchemaLoaderError::Resolution)?
        } else {
            return Err(SchemaLoaderError::Ingestion(
                crate::schema::error::SchemaIngestionError::Storage(
                    crate::schema::error::SchemaStorageError::NotFound {
                        name: status.path().to_string_lossy().into(),
                    },
                ),
            ));
        };

        let extends = if let Some(raw) = status.raw() {
            raw.extends().cloned()
        } else if let Some(view) = status.view() {
            view.extends().cloned()
        } else {
            None
        };

        if let Some(parent_name) = extends {
            let parent_id =
                index.get(&parent_name).copied().ok_or_else(|| {
                    SchemaLoaderError::Resolution(
                        crate::schema::error::SchemaError::Resolution(
                            SchemaResolutionError::ParentNotFound {
                                child: child_name.clone(),
                                parent: parent_name.clone(),
                            },
                        ),
                    )
                })?;
            Ok(vec![parent_id])
        } else {
            Ok(Vec::new())
        }
    }

    fn build_children(nodes: &mut HashMap<SchemaId, InheritanceNode>) {
        let parent_to_children: HashMap<SchemaId, Vec<SchemaId>> = nodes
            .values()
            .flat_map(|node| {
                node.parents.iter().map(move |&parent| (parent, node.id))
            })
            .fold(HashMap::new(), |mut acc, (parent, child)| {
                acc.entry(parent).or_default().push(child);
                acc
            });

        for (parent_id, mut children) in parent_to_children {
            if let Some(node) = nodes.get_mut(&parent_id) {
                children.sort();
                node.children = children;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    fn build_diamond_graph()
    -> (TopologicalGraph<InheritanceNode>, Vec<SchemaId>) {
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

        node_a.add_child(id_b);
        node_a.add_child(id_c);
        node_b.add_child(id_d);
        node_c.add_child(id_d);

        let nodes = HashMap::from([
            (id_a, node_a),
            (id_b, node_b),
            (id_c, node_c),
            (id_d, node_d),
        ]);

        let graph = TopologicalGraph {
            nodes,
            order: Vec::new(),
            roots: Vec::new(),
        };

        (graph, vec![id_a, id_b, id_c, id_d])
    }

    #[test]
    fn computes_depths_for_diamond_inheritance() {
        let (mut graph, ids) = build_diamond_graph();
        let id_a = *ids.first().expect("id a");
        let id_d = *ids.get(3).expect("id d");

        graph.compute_depths();

        let depth_a = graph.nodes.get(&id_a).expect("node a").depth;
        let depth_d = graph.nodes.get(&id_d).expect("node d").depth;

        assert_eq!(depth_a, NodeDepth::ROOT);
        assert_eq!(depth_d, NodeDepth::new(2));
    }

    #[test]
    fn topological_order_respects_all_parents() {
        let (mut graph, ids) = build_diamond_graph();
        graph.compute_depths();
        let (order, _roots) = graph.topological_sort().expect("topo sort");

        let position: HashMap<SchemaId, usize> =
            order.iter().copied().enumerate().map(|(i, id)| (id, i)).collect();

        let id_a = *ids.first().expect("id a");
        let id_b = *ids.get(1).expect("id b");
        let id_c = *ids.get(2).expect("id c");
        let id_d = *ids.get(3).expect("id d");

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

        let node_a =
            InheritanceNode::new_child(id_a, vec![id_c], NodeDepth::ROOT);
        let node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);
        let node_c =
            InheritanceNode::new_child(id_c, vec![id_b], NodeDepth::ROOT);

        let nodes =
            HashMap::from([(id_a, node_a), (id_b, node_b), (id_c, node_c)]);
        let mut validator = DagValidator::new(&nodes);

        assert!(validator.detect_cycles().is_err());
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

    #[test]
    fn set_parents_updates_bidirectional_links() {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();

        let mut node_a = InheritanceNode::new_root(id_a);
        let node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);
        node_a.add_child(id_b);

        let mut graph = TopologicalGraph {
            nodes: HashMap::from([(id_a, node_a), (id_b, node_b)]),
            order: Vec::new(),
            roots: Vec::new(),
        };

        graph.set_parents(id_b, Vec::new()).expect("set parents");

        #[cfg(debug_assertions)]
        graph.validate_consistency().expect("consistent graph");

        let parent = graph.nodes.get(&id_a).expect("node a");
        let child = graph.nodes.get(&id_b).expect("node b");
        assert!(parent.children.is_empty());
        assert!(child.parents.is_empty());
    }
}
