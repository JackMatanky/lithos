# Schema Graph Module Refactor: Tree to DAG

**Status**: Ready for Implementation
**Created**: 2026-03-31
**Goal**: Extract graph structures from `schema_pipeline.rs` into dedicated `graph.rs` module with full DAG support (multiple parents) for schema inheritance.

---

## Executive Summary

This refactor creates a new `lithos-core/src/schema/graph.rs` module to replace the current single-parent tree with a full directed acyclic graph (DAG) supporting multiple schema inheritance. This prepares the codebase for parity with Obsidian's metadata-menu plugin.

**Key Changes**:
- Create `graph.rs` with `InheritanceNode`, `GraphNode<T>`, `TopologicalGraph<T>`, `NodeDepth`
- Change from `parent: Option<SchemaId>` to `parents: Vec<SchemaId>` (DAG support)
- Store `children: Vec<SchemaId>` directly in nodes (denormalized, with consistency helpers)
- Delete `views/inheritance.rs` completely (no migration - we haven't shipped)
- Remove `SchemaInheritanceView` table and all related storage methods
- Use data-carrying payload structs for pipeline stages (not enums)

**Benefits**:
- Multiple parent support for complex schema hierarchies
- ~51% reduction in storage overhead (204 → 100 bytes per schema)
- Cleaner separation of concerns (graph logic isolated)
- -750 lines in `schema_pipeline.rs` (moved to dedicated module)
- Future-proof architecture for Obsidian metadata-menu parity

---

## Architecture Overview

### Current State (Single-Parent Tree)

```
schema_pipeline.rs (1950 lines)
├── TopologicalGraph<NodeStatus>
├── GraphNode<T> { parent_id: Option<SchemaId>, ... }
├── NodeStatus { file_path: PathBuf }
├── 10+ graph algorithm functions
└── CycleChecker

views/inheritance.rs (465 lines)
└── SchemaInheritanceView { parent, ancestors, depth, ancestors_hash, ... }

Storage: 3 tables
├── SCHEMA_TOPOLOGICAL_GRAPH (singleton)
├── SCHEMA_INHERITANCE (per-node metadata)
└── SCHEMA_CHILDREN_BY_PARENT (multimap)
```

### Target State (Multi-Parent DAG)

```
graph.rs (800 lines) ← NEW MODULE
├── InheritanceNode { parents: Vec<SchemaId>, children: Vec<SchemaId>, ... }
├── GraphNode<T> { parents: Vec<SchemaId>, children: Vec<SchemaId>, payload: T }
├── TopologicalGraph<T> { nodes, order, roots }
├── NodeDepth(usize) newtype
├── Payload structs (Fresh, StaleTimestamps, StaleContent, New)
└── All graph algorithms (Kahn, cycle detection, depth computation)

schema_pipeline.rs (1200 lines) ← REDUCED
├── Pipeline state machines
├── FileStatus enum (unchanged)
└── Delta computation logic

Storage: 1 table
└── SCHEMA_TOPOLOGICAL_GRAPH (singleton with InheritanceNode)
```

---

## Data Structures

### 1. NodeDepth (Newtype)

```rust
/// Inheritance depth in the DAG (0-indexed for roots).
///
/// - Root nodes: `depth = 0`
/// - Child nodes: `depth = max(parent_depths) + 1`
///
/// This enforces type safety and prevents mixing depth values with other counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize)]
pub struct NodeDepth(usize);

impl NodeDepth {
    pub const ROOT: Self = Self(0);

    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
```

### 2. InheritanceNode (Storage)

```rust
/// Minimal DAG node for database storage.
///
/// **Storage Layout** (typical: 2 parents, 3 children):
/// - `id`: 16 bytes (SchemaId is UUID)
/// - `parents`: 24 + 32 bytes (Vec header + 2 × 16)
/// - `children`: 24 + 48 bytes (Vec header + 3 × 16)
/// - `depth`: 8 bytes (NodeDepth wrapping usize)
/// **Total**: ~152 bytes (vs 204 bytes with SchemaInheritanceView)
///
/// **Why no `name`?** Retrieved from Schema aggregate via `id`.
/// **Why no `file_path`?** Stored in processing payloads only.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct InheritanceNode {
    pub id: SchemaId,
    pub parents: Vec<SchemaId>,
    pub children: Vec<SchemaId>,
    pub depth: NodeDepth,
}

impl InheritanceNode {
    /// Create a new root node (no parents).
    pub fn new_root(id: SchemaId) -> Self {
        Self {
            id,
            parents: Vec::new(),
            children: Vec::new(),
            depth: NodeDepth::ROOT,
        }
    }

    /// Create a new child node with given parents.
    pub fn new_child(id: SchemaId, parents: Vec<SchemaId>, depth: NodeDepth) -> Self {
        Self {
            id,
            parents,
            children: Vec::new(),
            depth,
        }
    }

    /// Check if this is a root node.
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Add a parent to this node.
    pub fn add_parent(&mut self, parent_id: SchemaId) {
        if !self.parents.contains(&parent_id) {
            self.parents.push(parent_id);
            self.parents.sort(); // Maintain stable order
        }
    }

    /// Remove a parent from this node.
    pub fn remove_parent(&mut self, parent_id: SchemaId) {
        self.parents.retain(|id| *id != parent_id);
    }

    /// Add a child to this node.
    pub fn add_child(&mut self, child_id: SchemaId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
            self.children.sort(); // Maintain stable order
        }
    }

    /// Remove a child from this node.
    pub fn remove_child(&mut self, child_id: SchemaId) {
        self.children.retain(|id| *id != child_id);
    }
}
```

### 3. GraphNode<T> (Processing)

```rust
/// Processing node with generic payload for pipeline stages.
///
/// **Shape matches InheritanceNode + payload**:
/// - Same `id`, `parents`, `children`, `depth` fields
/// - Additional `payload` for stage-specific data
///
/// Used during schema pipeline, then converted to `InheritanceNode` for storage.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct GraphNode<T> {
    pub id: SchemaId,
    pub parents: Vec<SchemaId>,
    pub children: Vec<SchemaId>,
    pub depth: NodeDepth,
    pub payload: T,
}

impl<T> GraphNode<T> {
    /// Convert to storage representation (discards payload).
    pub fn to_inheritance_node(self) -> InheritanceNode {
        InheritanceNode {
            id: self.id,
            parents: self.parents,
            children: self.children,
            depth: self.depth,
        }
    }
}

impl InheritanceNode {
    /// Convert to processing representation with given payload.
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
```

### 4. Payload Structs (Data-Carrying)

```rust
/// Payload for fresh nodes (no data needed, but not zero-sized for consistency).
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct Fresh;

/// Payload for nodes with stale timestamps.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct StaleTimestamps {
    pub times: RawFileTimes,
    pub view: RawSchemaView,
}

/// Payload for nodes with stale content.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct StaleContent {
    pub raw: RawSchema,
    pub content_hash: [u8; 32],
    pub times: RawFileTimes,
}

/// Payload for new nodes.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct New {
    pub raw: RawSchema,
    pub content_hash: [u8; 32],
    pub times: RawFileTimes,
}

/// Minimal node status for storage (file path only).
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct NodeStatus {
    #[rkyv(with = rkyv::with::AsString)]
    pub file_path: PathBuf,
}
```

### 5. TopologicalGraph<T> (Container)

```rust
/// Container for a topologically-ordered DAG.
///
/// **Invariants**:
/// - `order` contains all node IDs in topological order (parents before children)
/// - `nodes` contains all nodes indexed by ID
/// - `roots` contains all nodes with no parents
/// - All parent/child references are bidirectional and consistent
///
/// **Generic Parameter**:
/// - `T = InheritanceNode` for storage
/// - `T = GraphNode<Payload>` for processing
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct TopologicalGraph<T> {
    pub order: Vec<SchemaId>,
    pub nodes: HashMap<SchemaId, T>,
    pub roots: Vec<SchemaId>,
}

impl TopologicalGraph<InheritanceNode> {
    /// Apply a parent change, maintaining bidirectional consistency.
    pub fn set_parents(&mut self, node_id: SchemaId, new_parents: Vec<SchemaId>) -> Result<(), GraphError> {
        let old_parents = self.nodes.get(&node_id)
            .ok_or(GraphError::NodeNotFound(node_id))?
            .parents.clone();

        // Remove node from old parents' children
        for old_parent in &old_parents {
            if let Some(parent_node) = self.nodes.get_mut(old_parent) {
                parent_node.remove_child(node_id);
            }
        }

        // Add node to new parents' children
        for &new_parent in &new_parents {
            let parent_node = self.nodes.get_mut(&new_parent)
                .ok_or(GraphError::NodeNotFound(new_parent))?;
            parent_node.add_child(node_id);
        }

        // Update node's parents
        let node = self.nodes.get_mut(&node_id)
            .ok_or(GraphError::NodeNotFound(node_id))?;
        node.parents = new_parents;

        Ok(())
    }

    /// Validate bidirectional consistency (debug helper).
    #[cfg(debug_assertions)]
    pub fn validate_consistency(&self) -> Result<(), GraphError> {
        for (id, node) in &self.nodes {
            // Check parent → child links
            for &parent_id in &node.parents {
                let parent = self.nodes.get(&parent_id)
                    .ok_or(GraphError::DanglingReference(*id, parent_id))?;
                if !parent.children.contains(id) {
                    return Err(GraphError::InconsistentLink {
                        parent: parent_id,
                        child: *id,
                        direction: "parent→child missing",
                    });
                }
            }

            // Check child → parent links
            for &child_id in &node.children {
                let child = self.nodes.get(&child_id)
                    .ok_or(GraphError::DanglingReference(*id, child_id))?;
                if !child.parents.contains(id) {
                    return Err(GraphError::InconsistentLink {
                        parent: *id,
                        child: child_id,
                        direction: "child→parent missing",
                    });
                }
            }
        }
        Ok(())
    }
}
```

---

## Graph Algorithms (Organized by Struct)

### 1. TopologicalGraph Methods

All graph-wide operations are methods on `TopologicalGraph`:

```rust
impl<T> TopologicalGraph<T> {
    /// Compute topological order using Kahn's algorithm.
    ///
    /// Returns the order (parents before children) and root nodes.
    ///
    /// # Errors
    /// Returns `CycleDetected` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Result<(Vec<SchemaId>, Vec<SchemaId>), SchemaResolutionError>
    where
        T: AsRef<InheritanceNode>,
    {
        let mut in_degree: HashMap<SchemaId, usize> = self.nodes
            .values()
            .map(|node| (node.as_ref().id, node.as_ref().parents.len()))
            .collect();

        let mut queue: VecDeque<SchemaId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        let roots: Vec<SchemaId> = queue.iter().copied().collect();

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(node) = self.nodes.get(&node_id) {
                for &child_id in &node.as_ref().children {
                    let deg = in_degree.get_mut(&child_id).unwrap();
                    *deg = deg.saturating_sub(1);

                    if *deg == 0 {
                        queue.push_back(child_id);
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
    pub fn topological_sort_scoped(
        &self,
        affected: &HashSet<SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaResolutionError>
    where
        T: AsRef<InheritanceNode>,
    {
        // Similar to topological_sort but only processes affected nodes
        // ... (implementation omitted for brevity)
    }

    /// Compute all descendants of the given nodes (BFS).
    pub fn affected_subtree(&self, changed_ids: &HashSet<SchemaId>) -> HashSet<SchemaId>
    where
        T: AsRef<InheritanceNode>,
    {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        for &id in changed_ids {
            queue.push_back(id);
            affected.insert(id);
        }

        while let Some(id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for &child_id in &node.as_ref().children {
                    if affected.insert(child_id) {
                        queue.push_back(child_id);
                    }
                }
            }
        }

        affected
    }

    /// Remove nodes from the graph and update order/roots.
    pub fn prune(&mut self, deleted_ids: &[SchemaId])
    where
        T: AsRef<InheritanceNode> + AsMut<InheritanceNode>,
    {
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
    pub fn splice_order(
        &mut self,
        affected_order: &[SchemaId],
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaResolutionError>
    where
        T: AsRef<InheritanceNode>,
    {
        let mut anchor_map: HashMap<Option<SchemaId>, Vec<SchemaId>> = HashMap::new();

        for &id in affected_order {
            let anchor = self.nearest_unaffected_ancestor(id, affected);
            anchor_map.entry(anchor).or_default().push(id);
        }

        let mut new_order = Vec::with_capacity(self.order.len() + affected.len());
        for id in self.order.iter().copied().filter(|id| !affected.contains(id)) {
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
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        self.order = new_order;
        Ok(())
    }

    fn nearest_unaffected_ancestor(
        &self,
        id: SchemaId,
        affected: &HashSet<SchemaId>,
    ) -> Option<SchemaId>
    where
        T: AsRef<InheritanceNode>,
    {
        let node = self.nodes.get(&id)?;
        for &parent_id in &node.as_ref().parents {
            if !affected.contains(&parent_id) {
                return Some(parent_id);
            }
            // Recursively check grandparents
            if let Some(ancestor) = self.nearest_unaffected_ancestor(parent_id, affected) {
                return Some(ancestor);
            }
        }
        None
    }
}

impl TopologicalGraph<InheritanceNode> {
    /// Compute depths for all nodes: depth = max(parent_depths) + 1.
    pub fn compute_depths(&mut self) {
        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = self.nodes
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
                    let child = self.nodes.get(&child_id).unwrap();

                    let max_parent_depth = child.parents.iter()
                        .filter_map(|pid| depths.get(pid).copied())
                        .max()
                        .unwrap_or(0);

                    if child.parents.iter().all(|pid| depths.contains_key(pid)) {
                        depths.insert(child_id, max_parent_depth + 1);
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
    pub fn compute_depths_scoped(&mut self, affected: &HashSet<SchemaId>) {
        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = VecDeque::new();

        for &id in affected {
            let node = self.nodes.get(&id).unwrap();
            let all_parents_unaffected = node.parents.iter()
                .all(|parent| !affected.contains(parent));

            if all_parents_unaffected {
                let depth = node.parents.iter()
                    .map(|parent| self.nodes.get(parent).unwrap().depth.as_usize())
                    .max()
                    .unwrap_or(0);
                depths.insert(id, depth + 1);
                queue.push_back(id);
            }
        }

        while let Some(id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for &child_id in &node.children {
                    if !affected.contains(&child_id) {
                        continue;
                    }
                    let child = self.nodes.get(&child_id).unwrap();

                    let max_parent_depth = child.parents.iter()
                        .filter_map(|pid| depths.get(pid).copied())
                        .max()
                        .unwrap_or(0);

                    if child.parents.iter().all(|pid| depths.contains_key(pid)) {
                        depths.insert(child_id, max_parent_depth + 1);
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
}
```

### 2. DagValidator (Stateful Validation)

Cycle detection requires temporary state (visited/in-progress sets):

```rust
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
    pub fn new(nodes: &'graph HashMap<SchemaId, InheritanceNode>) -> Self {
        Self {
            nodes,
            visited: HashSet::with_capacity(nodes.len()),
            in_progress: HashSet::new(),
        }
    }

    /// Detect cycles in the entire graph.
    pub fn detect_cycles(&mut self) -> Result<(), SchemaResolutionError> {
        for &node_id in self.nodes.keys() {
            self.visit(node_id)?;
        }
        Ok(())
    }

    /// Detect cycles only in the affected subtree.
    pub fn detect_cycles_scoped(
        &mut self,
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaResolutionError> {
        for &node_id in affected {
            self.visit_scoped(node_id, affected)?;
        }
        Ok(())
    }

    fn visit(&mut self, node_id: SchemaId) -> Result<(), SchemaResolutionError> {
        if self.visited.contains(&node_id) {
            return Ok(());
        }

        if !self.in_progress.insert(node_id) {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: vec![/* collect cycle path */],
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
```

### 3. DagBuilder (Construction Helper)

Graph construction from FileStatus:

```rust
/// Builder for constructing `TopologicalGraph` from pipeline state.
///
/// This struct encapsulates the complex logic of building a graph from
/// file statuses, resolving parent relationships, and validating the result.
pub struct DagBuilder<'a> {
    statuses: &'a HashMap<SchemaId, FileStatus>,
}

impl<'a> DagBuilder<'a> {
    /// Create a new builder from file statuses.
    pub fn new(statuses: &'a HashMap<SchemaId, FileStatus>) -> Self {
        Self { statuses }
    }

    /// Build a complete graph from statuses.
    pub fn build(self) -> Result<TopologicalGraph<GraphNode<NodeStatus>>, SchemaLoaderError> {
        let index = self.build_name_index()?;
        let mut nodes = HashMap::with_capacity(self.statuses.len());

        for (id, status) in self.statuses {
            let name = self.extract_name(*id, status, &index)?;
            let parents = self.resolve_parents(*id, &name, status, &index)?;

            nodes.insert(*id, GraphNode {
                id: *id,
                parents,
                children: Vec::new(), // Computed below
                depth: NodeDepth::ROOT, // Computed after graph complete
                payload: NodeStatus {
                    file_path: status.path().to_path_buf(),
                },
            });
        }

        // Build bidirectional children links
        self.build_children(&mut nodes);

        // Validate and compute depths
        let mut graph = TopologicalGraph {
            nodes,
            order: Vec::new(),
            roots: Vec::new(),
        };

        let mut validator = DagValidator::new(&self.to_inheritance_nodes(&graph.nodes));
        validator.detect_cycles()?;

        graph.compute_depths();
        let (order, roots) = graph.topological_sort()?;
        graph.order = order;
        graph.roots = roots;

        Ok(graph)
    }

    fn build_name_index(&self) -> Result<HashMap<SchemaName, SchemaId>, SchemaLoaderError> {
        // ... implementation
    }

    fn extract_name(
        &self,
        id: SchemaId,
        status: &FileStatus,
        index: &HashMap<SchemaName, SchemaId>,
    ) -> Result<SchemaName, SchemaLoaderError> {
        // ... implementation
    }

    fn resolve_parents(
        &self,
        id: SchemaId,
        name: &SchemaName,
        status: &FileStatus,
        index: &HashMap<SchemaName, SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaLoaderError> {
        // ... implementation (handles multiple parent names)
    }

    fn build_children(&self, nodes: &mut HashMap<SchemaId, GraphNode<NodeStatus>>) {
        let parent_to_children: HashMap<SchemaId, Vec<SchemaId>> = nodes
            .values()
            .flat_map(|node| {
                node.parents.iter().map(move |&parent| (parent, node.id))
            })
            .fold(HashMap::new(), |mut acc, (parent, child)| {
                acc.entry(parent).or_default().push(child);
                acc
            });

        for (parent_id, children) in parent_to_children {
            if let Some(node) = nodes.get_mut(&parent_id) {
                node.children = children;
                node.children.sort(); // Stable order
            }
        }
    }

    fn to_inheritance_nodes(
        &self,
        nodes: &HashMap<SchemaId, GraphNode<NodeStatus>>,
    ) -> HashMap<SchemaId, InheritanceNode> {
        nodes.iter().map(|(&id, node)| {
            (id, InheritanceNode {
                id: node.id,
                parents: node.parents.clone(),
                children: node.children.clone(),
                depth: node.depth,
            })
        }).collect()
    }
}
```

### API Summary

**No free-floating functions!** All operations organized into structs:

| Operation | Old (Free Function) | New (Method/Struct) |
|-----------|---------------------|---------------------|
| Topological sort | `kahn_order_all(nodes)` | `graph.topological_sort()` |
| Scoped topo sort | `kahn_order_scoped(nodes, affected)` | `graph.topological_sort_scoped(affected)` |
| Compute depths | `compute_depths_all(nodes)` | `graph.compute_depths()` |
| Scoped depths | `compute_depths_scoped(nodes, affected)` | `graph.compute_depths_scoped(affected)` |
| Affected subtree | `collect_affected_subtrees(nodes, changed)` | `graph.affected_subtree(changed)` |
| Prune deleted | `prune_graph(graph, deleted)` | `graph.prune(deleted)` |
| Splice order | `splice_order(order, affected, nodes, ...)` | `graph.splice_order(affected_order, affected)` |
| Detect cycles | `detect_cycles_all(nodes)` | `DagValidator::new(nodes).detect_cycles()` |
| Scoped cycles | `detect_cycles_scoped(nodes, affected)` | `validator.detect_cycles_scoped(affected)` |
| Build graph | `build_graph_from_statuses(statuses, index)` | `DagBuilder::new(statuses).build()` |

**Usage Example**:

```rust
// Old way (free functions)
let graph = build_graph_from_statuses(&statuses, &index)?;
detect_cycles_all(&graph.nodes)?;
compute_depths_all(&mut graph.nodes);
let (order, roots) = kahn_order_all(&graph.nodes)?;

// New way (methods)
let graph = DagBuilder::new(&statuses).build()?;
// ^ Validation and depth computation happen inside build()

// Later, for incremental updates:
let affected = graph.affected_subtree(&changed_ids);
graph.compute_depths_scoped(&affected);
graph.splice_order(&affected_order, &affected)?;
```

---

## Storage Changes

### Remove These Tables

```rust
// DELETE from mod.rs db_table module:
pub(crate) const SCHEMA_INHERITANCE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_inheritance");

pub(crate) const SCHEMA_PARENT_TO_CHILDREN: MultimapTableDefinition<&str, &[u8]> =
    MultimapTableDefinition::new("schema_parent_to_children");

pub(crate) const SCHEMA_INHERITANCE_EDGES: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_inheritance_edges");
```

### Update Existing Table

```rust
// UPDATE in mod.rs db_table module:
/// Topologically sorted inheritance graph singleton.
///
/// Key: Constant `TOPOLOGICAL_GRAPH_KEY` (singleton)
/// Value: rkyv-serialized `TopologicalGraph<InheritanceNode>`.
///
/// **Changed from**: `TopologicalGraph<NodeStatus>` (processing only)
/// **Changed to**: `TopologicalGraph<InheritanceNode>` (storage representation)
pub(crate) const SCHEMA_TOPOLOGICAL_GRAPH: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_topological_graph");
```

### Remove Repository Methods

```rust
// DELETE from Repository trait in storage.rs:
fn get_inheritance_metadata(&self, id: SchemaId) -> Result<Option<SchemaInheritanceView>, Self::Error>;
fn save_inheritance_metadata(&self, id: SchemaId, metadata: &SchemaInheritanceView) -> Result<(), Self::Error>;
fn delete_inheritance_metadata(&self, id: SchemaId) -> Result<(), Self::Error>;
fn with_inheritance_metadata<F, R>(&self, id: SchemaId, f: F) -> Result<Option<R>, Self::Error>;
fn list_inheritance_children(&self) -> Result<InheritanceChildren, Self::Error>;
fn list_descendant_ids(&self, parent_id: SchemaId) -> Result<Vec<SchemaId>, Self::Error>;
```

### Update Repository Methods

```rust
// UPDATE in Repository trait:
fn get_topological_graph(&self) -> Result<Option<TopologicalGraph<InheritanceNode>>, Self::Error>;
fn save_topological_graph(&self, graph: &TopologicalGraph<InheritanceNode>) -> Result<(), Self::Error>;
```

---

## Implementation Plan

### Phase 1: Create graph.rs Module (No Breaking Changes)

1. **Create `lithos-core/src/schema/graph.rs`**:
   - Define `NodeDepth`, `InheritanceNode`, `GraphNode<T>`, `TopologicalGraph<T>`
   - Define payload structs: `Fresh`, `StaleTimestamps`, `StaleContent`, `New`, `NodeStatus`
   - Add rkyv derives and constructors

2. **Move graph algorithms from `schema_pipeline.rs`**:
   - `detect_cycles_all`, `detect_cycles_scoped`
   - `compute_depths_all`, `compute_depths_scoped`
   - `kahn_order_all`, `kahn_order_scoped`
   - `collect_affected_subtrees`
   - `build_children_map` (if still needed)
   - `splice_order`
   - `prune_graph`
   - `CycleChecker` struct

3. **Add `pub mod graph;` to `lithos-core/src/schema/mod.rs`**

4. **Run tests**: `mise run test:unit:schema` (should still pass)

### Phase 2: Update Storage Layer

1. **Update `storage.rs`**:
   - Change `get_topological_graph()` return type to `TopologicalGraph<InheritanceNode>`
   - Change `save_topological_graph()` parameter to `TopologicalGraph<InheritanceNode>`
   - Remove all `*_inheritance_metadata()` methods
   - Remove `list_inheritance_children()` and `list_descendant_ids()`

2. **Update `mod.rs` db_table module**:
   - Remove `SCHEMA_INHERITANCE`, `SCHEMA_PARENT_TO_CHILDREN`, `SCHEMA_INHERITANCE_EDGES`
   - Update docs for `SCHEMA_TOPOLOGICAL_GRAPH`

3. **Run tests**: `mise run test:unit:schema`

### Phase 3: Update schema_pipeline.rs

1. **Import from `graph` module**:
   ```rust
   use crate::schema::graph::{
       InheritanceNode, GraphNode, TopologicalGraph, NodeDepth,
       Fresh, StaleTimestamps, StaleContent, New, NodeStatus,
       detect_cycles_all, detect_cycles_scoped,
       compute_depths_all, compute_depths_scoped,
       kahn_order_all, kahn_order_scoped,
       collect_affected_subtrees,
   };
   ```

2. **Update TreeGraphed stage**:
   - Change `parent_id: Option<SchemaId>` → `parents: Vec<SchemaId>` everywhere
   - Update `resolve_parent()` to `resolve_parents()` returning `Vec<SchemaId>`
   - Maintain `children` vectors in nodes during graph building
   - Use bidirectional consistency helpers

3. **Update Completion stage**:
   - Remove `SchemaInheritanceView` persistence loop
   - Convert `GraphNode<NodeStatus>` → `InheritanceNode` before saving
   - Save only `TopologicalGraph<InheritanceNode>`

4. **Run tests**: `mise run test:unit:schema`

### Phase 4: Update Builder & Loader

1. **Update `builder.rs`**:
   - No changes needed (uses pipeline facade)

2. **Update `loader.rs`**:
   - Remove `persist_inheritance_metadata()` method
   - Update any direct graph access to use new types

3. **Run integration tests**: `mise run test:integration`

### Phase 5: Delete views/inheritance.rs

1. **Delete file**: `lithos-core/src/schema/views/inheritance.rs`

2. **Update `views/mod.rs`**:
   - Remove `pub mod inheritance;`
   - Remove `pub use inheritance::*;`

3. **Update any remaining imports**:
   - Search for `SchemaInheritanceView` usage
   - Replace or remove

4. **Run full test suite**: `mise run test`

### Phase 6: Update Tests

1. **Update unit tests in `schema_pipeline.rs`**:
   - Change test data from single parent to multiple parents
   - Add multi-parent test cases

2. **Add graph algorithm tests in `graph.rs`**:
   - Cycle detection with multiple parents
   - Depth computation with diamond inheritance
   - Topological sort correctness
   - Bidirectional consistency validation

3. **Update integration tests**:
   - Test multi-parent schemas
   - Test graph persistence/loading

4. **Run verification**: `mise run verify`

### Phase 7: Documentation & Cleanup

1. **Update AGENTS.md** with graph module info

2. **Add module-level docs to `graph.rs`**:
   - Architecture overview
   - DAG vs tree explanation
   - Usage examples

3. **Update inline docs**:
   - Fix any outdated references to single-parent
   - Add examples of multi-parent usage

4. **Final verification**: `mise run verify`

---

## Migration Notes

**No migration needed** - we haven't shipped yet. Existing databases will have stale tables that can be cleaned up manually or ignored (they won't cause issues).

For future versions after shipping, we would need:
1. Detect old table presence
2. Rebuild graph from Schema aggregates
3. Delete old tables
4. Save new graph

---

## Testing Strategy

### Unit Tests (graph.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_parent_tree_still_works() {
        // Ensure backward compatibility
    }

    #[test]
    fn multiple_parents_diamond_inheritance() {
        // Schema D extends both B and C, which both extend A
        //     A
        //    / \
        //   B   C
        //    \ /
        //     D
        let mut nodes = HashMap::new();
        // ... test depth = 2 for D (max of B and C + 1)
    }

    #[test]
    fn cycle_detection_with_multiple_parents() {
        // A extends B, B extends C, C extends A (cycle)
        // Should detect cycle even with multiple entry points
    }

    #[test]
    fn topological_order_respects_all_parents() {
        // Ensure all parents come before children
    }

    #[test]
    fn bidirectional_consistency_maintained() {
        // Test set_parents() helper
    }

    #[test]
    fn affected_subtree_with_multiple_parents() {
        // Changing A should affect B, C, and D in diamond pattern
    }
}
```

### Integration Tests

```rust
#[test]
fn schema_with_multiple_parents_loads_correctly() {
    // test-schemas/multi-parent.toml:
    // extends: ["base", "mixin"]
}

#[test]
fn graph_persists_and_reloads_correctly() {
    // Save graph with multi-parent nodes, reload, verify structure
}
```

---

## Performance Impact

### Storage Size Comparison

**Before** (per schema):
- `SchemaInheritanceView`: ~172 bytes
- Multimap entry: ~32 bytes per child
- **Total**: ~204 bytes + 32 × children

**After** (per schema):
- `InheritanceNode`: ~100 bytes (includes embedded children)
- **Total**: ~100 bytes

**Savings**: ~51% reduction

### Query Performance

| Operation | Before | After | Change |
|-----------|--------|-------|--------|
| Get parent(s) | O(log N) table read | O(log N) graph lookup | Same |
| Get children | O(log N + C) multimap | O(1) in-node vector | ✅ Faster |
| Descendant traversal | O(D×log N) multimap | O(D) BFS | ✅ Faster |
| Staleness check | O(log N) + hash compute | O(1) comparison | ✅ Simpler |
| Graph rebuild | O(N×log N) | O(N) | ✅ Faster |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bidirectional consistency bugs | High | Add `validate_consistency()` helper, run in debug builds |
| Cycle detection complexity | Medium | DFS is well-tested, add comprehensive test cases |
| Storage format change | Low | Single table to update, no migration (pre-ship) |
| Performance regression | Low | DAG algorithms are O(V+E), same as tree in practice |

---

## Success Criteria

- ✅ All tests pass (`mise run verify`)
- ✅ `views/inheritance.rs` deleted
- ✅ Storage uses single table (`SCHEMA_TOPOLOGICAL_GRAPH`)
- ✅ Multiple parents work (test with 2+ parent schema)
- ✅ No clippy warnings
- ✅ Code formatted (`mise run fmt`)
- ✅ ~750 lines removed from `schema_pipeline.rs`
- ✅ New `graph.rs` module with ~800 lines

---

## Future Work (Post-Refactor)

1. **Obsidian metadata-menu parity**: Update CLI to support `extends: ["parent1", "parent2"]` syntax
2. **Conflict resolution**: Define merge strategy when multiple parents provide same property
3. **Visualization**: Add `lithos graph` command to render inheritance DAG
4. **Performance tuning**: Profile affected-subtree computation on large graphs (1000+ schemas)
5. **Cache optimization**: Consider keeping `children` map separate if consistency becomes a bottleneck

---

## References

- [DAG Research Task Results](/.opencode/tasks/dag-research.md) (if saved)
- [Rust Naming Taxonomy](/docs/refs/rust/naming-taxonomy.md)
- [Architecture Decisions](/docs/adr/)
- [Property Bank Processor](lithos-core/src/schema/property_bank_processor.rs) (pattern reference)

---

**Next Steps**: Review this plan, then proceed with Phase 1 implementation.
