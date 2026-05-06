# Graph Module API Organization

**Date**: 2026-03-31
**Status**: Design Approved

This document summarizes the final API organization for `lithos-core/src/schema/graph.rs`, eliminating all free-floating functions in favor of methods and helper structs.

---

## Module Structure

```
graph.rs
├── Core Types
│   ├── NodeDepth (newtype)
│   ├── InheritanceNode (storage)
│   ├── GraphNode<T> (processing)
│   └── TopologicalGraph<T> (container)
│
├── Payload Structs
│   ├── Fresh
│   ├── StaleTimestamps { times, view }
│   ├── StaleContent { raw, content_hash, times }
│   ├── New { raw, content_hash, times }
│   └── NodeStatus { file_path }
│
└── Helper Structs
    ├── DagValidator (cycle detection + validation)
    └── DagBuilder (graph construction from FileStatus)
```

---

## 1. TopologicalGraph\<T> (13 methods)

**Purpose**: Container for DAG with topological order maintenance.

### Graph-Wide Operations

```rust
impl<T: AsRef<InheritanceNode>> TopologicalGraph<T> {
    // Topological sorting
    fn topological_sort(&self) -> Result<(Vec<SchemaId>, Vec<SchemaId>), Error>;
    fn topological_sort_scoped(&self, affected: &HashSet<SchemaId>) -> Result<Vec<SchemaId>, Error>;

    // Traversal
    fn affected_subtree(&self, changed: &HashSet<SchemaId>) -> HashSet<SchemaId>;

    // Maintenance
    fn prune(&mut self, deleted: &[SchemaId]);
    fn splice_order(&mut self, affected_order: &[SchemaId], affected: &HashSet<SchemaId>) -> Result<(), Error>;

    // Internal helper
    fn nearest_unaffected_ancestor(&self, id: SchemaId, affected: &HashSet<SchemaId>) -> Option<SchemaId>;
}
```

### Depth Computation (InheritanceNode only)

```rust
impl TopologicalGraph<InheritanceNode> {
    // Depth computation (requires mutable access to nodes)
    fn compute_depths(&mut self);
    fn compute_depths_scoped(&mut self, affected: &HashSet<SchemaId>);

    // Bidirectional consistency
    fn set_parents(&mut self, node_id: SchemaId, new_parents: Vec<SchemaId>) -> Result<(), Error>;

    // Debug helper
    #[cfg(debug_assertions)]
    fn validate_consistency(&self) -> Result<(), Error>;
}
```

**Why split?**
- Generic methods work on any `T: AsRef<InheritanceNode>` (read-only)
- Depth computation needs `&mut InheritanceNode` (write access)

---

## 2. DagValidator (4 methods)

**Purpose**: Cycle detection with temporary state (visited/in-progress sets).

```rust
pub struct DagValidator<'graph> {
    nodes: &'graph HashMap<SchemaId, InheritanceNode>,
    visited: HashSet<SchemaId>,
    in_progress: HashSet<SchemaId>,
}

impl<'graph> DagValidator<'graph> {
    // Constructor
    pub fn new(nodes: &'graph HashMap<SchemaId, InheritanceNode>) -> Self;

    // Public validation methods
    pub fn detect_cycles(&mut self) -> Result<(), SchemaResolutionError>;
    pub fn detect_cycles_scoped(&mut self, affected: &HashSet<SchemaId>) -> Result<(), Error>;

    // Private DFS helpers
    fn visit(&mut self, node_id: SchemaId) -> Result<(), Error>;
    fn visit_scoped(&mut self, node_id: SchemaId, affected: &HashSet<SchemaId>) -> Result<(), Error>;
}
```

**Usage**:
```rust
let nodes: HashMap<SchemaId, InheritanceNode> = /* ... */;
let mut validator = DagValidator::new(&nodes);
validator.detect_cycles()?;
```

**Why not a method on TopologicalGraph?**
- Needs mutable temporary state (visited/in-progress sets)
- Used during construction before graph exists
- Separate struct makes state lifetime explicit

---

## 3. DagBuilder (7+ methods)

**Purpose**: Construct `TopologicalGraph` from `FileStatus` map.

```rust
pub struct DagBuilder<'a> {
    statuses: &'a HashMap<SchemaId, FileStatus>,
}

impl<'a> DagBuilder<'a> {
    // Constructor
    pub fn new(statuses: &'a HashMap<SchemaId, FileStatus>) -> Self;

    // Main build method (orchestrates all steps)
    pub fn build(self) -> Result<TopologicalGraph<GraphNode<NodeStatus>>, SchemaLoaderError>;

    // Private helpers
    fn build_name_index(&self) -> Result<HashMap<SchemaName, SchemaId>, Error>;
    fn extract_name(&self, id: SchemaId, status: &FileStatus, index: &HashMap<SchemaName, SchemaId>) -> Result<SchemaName, Error>;
    fn resolve_parents(&self, id: SchemaId, name: &SchemaName, status: &FileStatus, index: &HashMap<SchemaName, SchemaId>) -> Result<Vec<SchemaId>, Error>;
    fn build_children(&self, nodes: &mut HashMap<SchemaId, GraphNode<NodeStatus>>);
    fn to_inheritance_nodes(&self, nodes: &HashMap<SchemaId, GraphNode<NodeStatus>>) -> HashMap<SchemaId, InheritanceNode>;
}
```

**Usage**:
```rust
let statuses: HashMap<SchemaId, FileStatus> = /* from pipeline */;
let graph = DagBuilder::new(&statuses).build()?;
// ^ Handles: name resolution, parent resolution, children building,
//            cycle detection, depth computation, topological sort
```

**Why not free functions?**
- Encapsulates complex construction logic
- Shares `statuses` reference across helper methods
- Makes dependencies explicit (only needs FileStatus map)

---

## 4. InheritanceNode (10+ methods)

**Purpose**: Storage representation with bidirectional relationship helpers.

```rust
impl InheritanceNode {
    // Constructors
    pub fn new_root(id: SchemaId) -> Self;
    pub fn new_child(id: SchemaId, parents: Vec<SchemaId>, depth: NodeDepth) -> Self;

    // Queries
    pub fn is_root(&self) -> bool;

    // Parent mutation
    pub fn add_parent(&mut self, parent_id: SchemaId);
    pub fn remove_parent(&mut self, parent_id: SchemaId);

    // Child mutation
    pub fn add_child(&mut self, child_id: SchemaId);
    pub fn remove_child(&mut self, child_id: SchemaId);

    // Conversion
    pub fn with_payload<T>(self, payload: T) -> GraphNode<T>;
}
```

---

## 5. GraphNode\<T> (2 methods)

**Purpose**: Processing representation with generic payload.

```rust
impl<T> GraphNode<T> {
    // Conversion
    pub fn to_inheritance_node(self) -> InheritanceNode;
}
```

---

## 6. NodeDepth (3 methods)

**Purpose**: Newtype wrapper for type-safe depth values.

```rust
impl NodeDepth {
    pub const ROOT: Self = Self(0);
    pub const fn new(depth: usize) -> Self;
    pub const fn as_usize(self) -> usize;
    pub const fn increment(self) -> Self;
}
```

---

## Comparison: Old vs New

### Old (Free Functions)

```rust
// schema_pipeline.rs (scattered across 1950 lines)
pub fn detect_cycles_all(nodes: &HashMap<SchemaId, GraphNode<T>>, ...) -> Result<(), Error>;
pub fn detect_cycles_scoped(nodes: &HashMap<SchemaId, GraphNode<T>>, affected: &HashSet<SchemaId>, ...) -> Result<(), Error>;
pub fn compute_depths_all(nodes: &mut HashMap<SchemaId, GraphNode<T>>, ...);
pub fn compute_depths_scoped(nodes: &mut HashMap<SchemaId, GraphNode<T>>, affected: &HashSet<SchemaId>, ...);
pub fn kahn_order_all(nodes: &HashMap<SchemaId, GraphNode<T>>, ...) -> Result<(Vec<SchemaId>, Vec<SchemaId>), Error>;
pub fn kahn_order_scoped(nodes: &HashMap<SchemaId, GraphNode<T>>, affected: &HashSet<SchemaId>, ...) -> Result<Vec<SchemaId>, Error>;
pub fn collect_affected_subtrees(children: &HashMap<SchemaId, Vec<SchemaId>>, changed: &HashSet<SchemaId>, new: &HashSet<SchemaId>) -> HashSet<SchemaId>;
pub fn prune_graph(graph: TopologicalGraph<NodeStatus>, deleted: &[SchemaId]) -> TopologicalGraph<NodeStatus>;
pub fn splice_order(old: &[SchemaId], affected: &[SchemaId], nodes: &HashMap<SchemaId, GraphNode<NodeStatus>>, affected_set: &HashSet<SchemaId>) -> Result<Vec<SchemaId>, Error>;
pub fn build_graph_from_statuses(statuses: &HashMap<SchemaId, FileStatus>, index: &SchemaIndex) -> Result<TopologicalGraph<GraphNode<NodeStatus>>, Error>;
pub fn build_children_map<T>(nodes: &HashMap<SchemaId, GraphNode<T>>) -> HashMap<SchemaId, Vec<SchemaId>>;
```

**Problems**:
- Hard to discover (no IDE autocomplete after typing `graph.`)
- No clear ownership (who owns these functions?)
- Parameter soup (many functions take 3-5 parameters)
- Unclear usage order (what calls what?)

### New (Methods + Helper Structs)

```rust
// graph.rs (organized into cohesive types)

// Graph operations
let affected = graph.affected_subtree(&changed);
graph.compute_depths_scoped(&affected);
graph.prune(&deleted);
let (order, roots) = graph.topological_sort()?;
graph.splice_order(&affected_order, &affected)?;

// Validation
let mut validator = DagValidator::new(&graph.nodes);
validator.detect_cycles()?;

// Construction
let graph = DagBuilder::new(&statuses).build()?;
```

**Benefits**:
- ✅ IDE autocomplete works (`graph.` shows all methods)
- ✅ Clear ownership (methods belong to structs)
- ✅ Fewer parameters (context implicit via `self`)
- ✅ Usage patterns obvious (construct → validate → operate)
- ✅ Easy to mock/test (trait implementations on structs)

---

## Migration Guide

### Before (schema_pipeline.rs)

```rust
use crate::schema::schema_pipeline::{
    detect_cycles_all,
    compute_depths_all,
    kahn_order_all,
    collect_affected_subtrees,
    prune_graph,
};

// Scattered function calls
detect_cycles_all(&graph.nodes, &id_to_name)?;
compute_depths_all(&mut graph.nodes, &children);
let (order, roots) = kahn_order_all(&graph.nodes, &children)?;
let affected = collect_affected_subtrees(&children, &changed, &new);
let pruned = prune_graph(graph, &deleted);
```

### After (graph.rs)

```rust
use crate::schema::graph::{
    DagBuilder,
    DagValidator,
    TopologicalGraph,
};

// Build graph (validation included)
let mut graph = DagBuilder::new(&statuses).build()?;

// Operate on graph
let affected = graph.affected_subtree(&changed);
graph.compute_depths_scoped(&affected);
graph.prune(&deleted);
```

---

## Success Criteria

- ✅ Zero free-floating functions in `graph.rs`
- ✅ All operations discoverable via IDE autocomplete
- ✅ Clear ownership (every function belongs to a struct)
- ✅ Reduced parameter count (context via `self`)
- ✅ Easier to test (mock structs, not functions)
- ✅ Better encapsulation (private helpers inside impl blocks)

---

## Next Steps

1. Implement `graph.rs` with this organization
2. Update `schema_pipeline.rs` to use new API
3. Verify all tests pass
4. Update documentation with usage examples

---

**Questions?** See main plan at `.opencode/plans/graph-module-dag-refactor.md`
