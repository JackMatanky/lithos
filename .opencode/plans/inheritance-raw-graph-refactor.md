# Inheritance Graph Refactor Plan (Raw Graph + Scoped Validation)

## Scope
Refactor `lithos-core/src/schema/graph.rs`, `lithos-core/src/schema/inheritance.rs`, and `lithos-core/src/schema/schema_processor.rs` to:
- Remove unchecked graph construction from `InheritanceGraph`.
- Introduce a raw, mutable `Graph<T>` with `Node<T>` + `Edge` and a computed `AdjacencyMap` view.
- Make `TryFrom<Graph<T>> for InheritanceGraph<T>` the only validation boundary.
- Preserve incremental performance via scoped updates in a renamed `InheritanceEditor`.
- Split inheritance storage to node/edge types to align with graph structure and simplify extends analysis.

## Current State (as of now)
### graph.rs
- `GraphBuilder` constructs `InheritanceGraph<T>` directly via `from_parts`.
- `GraphEditor` mutates internal parts and returns `InheritanceGraph<T>` via `from_parts`.
- `TopologicalSorter` returns `(Vec<SchemaId>, Vec<SchemaId>)` (order + roots).
- `ChildParentsMap` used for building, normalization.

### inheritance.rs
- `InheritanceGraph` exposes `from_parts`, `into_parts`, read-only accessors, `map_payload`.
- `validate_consistency` exists on `InheritanceGraph` (debug helper).
- `InheritanceNode` stores id, parents, children, depth.
- `GraphNode` trait provides `set_edges`/`set_depth` for mutable node types.

### schema_processor.rs
- Uses `GraphBuilder` for `build_new_graph`.
- Uses `GraphEditor` for incremental updates in `build_graph`.
- Many `InheritanceGraph::from_parts` call sites (present graph, compared graph, parsed graph, analyzed graph, etc.).
- `PreProcessNode` and `PostProcessNode` implement `GraphNode` for builder/editor operations.

## Target Design Summary
- **Raw graph (mutable, unchecked)**: `Graph<T, R>` with `Node<T>` and `Edge<R>`.
- **AdjacencyMap (computed view)**: built on demand from `Graph<T, R>`; no extra fields in `Graph<T, R>`.
- **Validated graph (immutable)**: `InheritanceGraph<T>` constructed only via `TryFrom<Graph<T, R>>` and editor patch results.
- **Editor**: renamed to `InheritanceEditor`; performs scoped revalidation and splicing.
- **No `InheritanceGraph::from_parts`** and **no `InheritanceGraph::validate_consistency`**.
- **Keep `InheritanceGraph::into_parts()` as `pub(crate)`**.
- **Split inheritance structure**: keep `InheritanceNode` for validated node storage, add `InheritanceEdge` as an edge-centric helper for extends analysis and edge metadata handling.

## Design Decisions
1) **Module placement**: keep raw graph types (`Graph`, `Node`, `Edge`, `AdjacencyMap`) in `schema/graph.rs`. Move `TopologicalSorter` + `TopologicalOrder` into `schema/topo_sort.rs`. Move `InheritanceEditor` into `inheritance.rs`. Implement `TryFrom<Graph<T, R>> for InheritanceGraph<T>` in `inheritance.rs` to match existing placement conventions.
2) `Graph<T, R>` stores `nodes: HashMap<SchemaId, Node<T>>` and `edges: Vec<Edge<R>>`.
   - No redundant `SchemaId` in `Node<T>` (id is the map key).
   - `Node<T>` includes `depth: NodeDepth` for scoped updates.
   - `Edge<R>` is generic to carry relation metadata during processing (no default type params).
3) `AdjacencyMap` is computed on demand from `Graph<T, R>` and is the source of truth for parents/children during validation and sorting.
4) `TopologicalSorter` and `TopologicalOrder` live in `topo_sort.rs` (or `toposort.rs`), operate on adjacency/in-degree, and derive roots during sort.
5) `InheritanceEditor` returns a validated `InheritanceGraph<T>` and performs scoped recomputation with `sort_scoped()` + depth rebuild before final assembly.
6) `InheritanceNode` remains the validated node storage type (id + parents + children + depth). `InheritanceEdge` is introduced for edge-centric processing and to model relation metadata where needed (e.g., `ExtendsChangeKind`), with metadata dropped during validation.
7) `NodeDepth` + its tests move to `graph.rs` (graph semantics). Replace `GraphNode`/`NodeAccessor` with two read-only traits: `NodeAccessor` and `EdgeAccessor` (for node fields vs edge lists). With `Graph<T, R>` mutations handled by the graph component itself, we avoid a separate mutating trait.

### Rationale for generic `Edge<R>` (no defaults)
- During schema processing, edges may need metadata (e.g., `ExtendsChangeKind`).
- Keeping `relation` on raw edges avoids polluting node payloads or validated graph structures.
- No default type parameters (`Edge<R>` and `Node<T>`) to avoid accidental reliance on `()` and to keep errors explicit at call sites.
- The validated `InheritanceGraph` remains structural-only; edge metadata is dropped at conversion time.

---

## Phase 1: Introduce Raw Graph Types (graph.rs)
### 1.1 Add types
Add below to `graph.rs`:
```rust
pub(crate) struct Node<T> {
    depth: NodeDepth,
    payload: T,
}

pub(crate) struct Edge<R> {
    from: SchemaId,
    to: SchemaId,
    relation: R,
}

pub(crate) struct Graph<T, R> {
    nodes: HashMap<SchemaId, Node<T>>,
    edges: Vec<Edge<R>>,
}
```

### 1.2 Add `Graph<T, R>` API
Minimum API for construction and edits:
- `new()`
- `add_node(id, payload)` (initialize depth as `NodeDepth::ROOT`)
- `remove_node(id)`
- `node(id)` / `node_mut(id)`
- `node_depth(id)` (returns `Option<NodeDepth>` for quick access)
- `add_edge(from, to)` and `add_edge_with(from, to, relation)`
- `remove_edge(from, to)`
- `edges()` (readonly slice)

Optional helpers:
- `subgraph(affected: &HashSet<SchemaId>) -> Graph<T>` (clone nodes + filtered edges)
- `reset_depths(ids)`

### 1.3 Add computed view
```rust
pub(crate) struct AdjacencyMap {
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
    pub(crate) fn from_graph<T, R>(graph: &Graph<T, R>) -> Self { /* normalize */ }
}
```
Normalize parent/child lists (sort + dedup) inside `from_graph`.

### 1.4 Add raw graph ingest from `ChildParentsMap`
- Keep `ChildParentsMap` as the simplest RawSchema ingestion shape.
- Add helper to convert `ChildParentsMap` to `Graph<T, R>` by:
  - creating nodes for each id
  - emitting edges `(parent -> child)` for each parent list

---

## Phase 2: Replace GraphBuilder
### 2.1 Remove `GraphBuilder`
- Delete struct and helpers in `graph.rs`.

### 2.2 Update builder-style usages
- All previous builder flows now use raw `Graph<T>` and `TryFrom` (see Phase 4).

---

## Phase 3: Topological Sort Module
### 3.1 Add module
- Create `lithos-core/src/schema/topo_sort.rs` to host `TopologicalSorter` + `TopologicalOrder`.

### 3.2 Add order struct
```rust
pub(crate) struct TopologicalOrder {
    order: Vec<SchemaId>,
    roots: Vec<SchemaId>,
}
```

### 3.3 Add methods
- `order(&self) -> &[SchemaId]`
- `roots(&self) -> &[SchemaId]`

### 3.4 Update TopologicalSorter
- `sort()` derives in-degree from `AdjacencyMap` and returns `TopologicalOrder`.
- `sort_scoped()` uses a scoped adjacency/in-degree view and returns `TopologicalOrder`.

---

## Phase 4: Validation Boundary (`TryFrom<Graph<T, R>> for InheritanceGraph<T>`) (inheritance.rs)
### 4.1 Remove from InheritanceGraph
- Remove `from_parts`.
- Remove `validate_consistency`.

### 4.2 Keep `into_parts()`
- Remains `pub(crate)`.

### 4.3 Implement `TryFrom<Graph<T, R>>`
Place in `inheritance.rs`: `impl TryFrom<Graph<T, R>> for InheritanceGraph<T>`.
Error type: use existing `SchemaInheritanceError` for graph invariants. Callers (e.g., schema processor) wrap via `SchemaLoaderError::Resolution`.
Algorithm:
1. Build `AdjacencyMap` from raw graph.
2. Validate missing nodes: for every edge endpoint ensure node exists.
3. Build `parents` + `children` lists from adjacency map.
4. Run `TopologicalSorter::sort` (acyclic validation) and use the returned roots.
5. Compute depths from topological order.
6. Construct `InheritanceGraph<T>` directly inside the `TryFrom` impl (no separate constructor).

Note: edge `relation` metadata is ignored during validation and dropped when producing `InheritanceGraph<T>`.

### 4.4 Add edge-centric helper type
- Introduce a new edge type in `inheritance.rs`:
  ```rust
  pub struct InheritanceEdge {
      parent: SchemaId,
      child: SchemaId,
  }
  ```
- Keep `InheritanceNode` as the validated node (id, depth, parents, children).
- Use `InheritanceEdge` in schema processing for extends analysis to reduce noise and focus on edges.
- Ensure conversion helpers exist to derive `InheritanceEdge` lists from `InheritanceGraph` or raw `Graph<T, R>`.

---

## Phase 5: Move Editor and Accessor Traits
### 5.1 Rename type and move location
- `GraphEditor` -> `InheritanceEditor`
- Move implementation from `graph.rs` to `inheritance.rs`.
- Update all call sites in `schema_processor.rs` and tests.

### 5.2 Move accessor traits and NodeDepth
- Move `NodeDepth` and its tests from `inheritance.rs` to `graph.rs`.
- Replace `GraphNode`/`NodeAccessor` with two read-only traits in `graph.rs`:
  - `NodeAccessor`: node field access (id/depth/any node-owned fields used by algorithms).
  - `EdgeAccessor`: parent/child list access for validated nodes.
- Update trait bounds throughout `graph.rs`, `inheritance.rs`, and `schema_processor.rs` to use the new traits.

### 5.3 Update editor internals to use raw Graph + adjacency views
Current editor mutates `nodes/order/roots` directly. Replace with:
1. Accept validated `InheritanceGraph<T>` (via `into_parts` or reference).
2. Build a raw `Graph<T, R>` subgraph for affected nodes + required neighbors.
3. Apply changes in raw graph (edges + payloads + depths reset to `ROOT` for affected).
4. Build `AdjacencyMap` for affected subgraph.
5. Run scoped topological sort for affected set, returning `TopologicalOrder`.
6. Recompute depths only for affected nodes.
7. Derive roots from `TopologicalOrder::roots` and splice order into old order (current logic).
8. Return a validated `InheritanceGraph<T>` by using the scoped validation path (no full `TryFrom`).

### 5.4 Scoped validation helper
Add a helper in `graph.rs`:
```rust
fn try_from_scoped<T, R>(
    graph: &Graph<T, R>,
    affected: &HashSet<SchemaId>,
    base: &InheritanceGraph<T>,
) -> Result<(TopologicalOrder, Vec<SchemaId>, HashMap<SchemaId, NodeDepth>), SchemaResolutionError>;
```
This will centralize scoped topological sort + depth recompute. Roots returned are derived via `TopologicalOrder::roots`.

### 5.5 Internal validated constructor
- Add a module-private constructor in `inheritance.rs` for editor use only:
  ```rust
  pub(crate) fn from_validated_parts(
      nodes: HashMap<SchemaId, T>,
      order: Vec<SchemaId>,
      roots: Vec<SchemaId>,
  ) -> Self
  ```
- This replaces `from_parts` without reintroducing unchecked public construction (only used after `try_from_scoped`).

---

## Phase 6: Update InheritanceGraph map_payload (inheritance.rs)
### 6.1 Remove map_payload’s dependence on `from_parts`
Current `map_payload` uses `from_parts`. Replace with:
- Build a raw `Graph<U, ()>` from `self`:
  - For each node in order: `graph.add_node(id, mapped_payload)`
  - For each node in order: add edges for each parent relationship
- Convert using `TryFrom<Graph<U, ()>>` (validates structure again, but safe).

If validation overhead is a concern, add a module-private helper in `inheritance.rs` used only by `map_payload` (not exposed) to construct `InheritanceGraph` from already-validated parts.

---

## Phase 7: SchemaProcessor Updates (schema_processor.rs)

### 7.1 Replace GraphBuilder usage
`build_new_graph` currently uses `GraphBuilder` to create a graph from extends.
New flow:
1. Build raw `Graph<InheritanceNode, ()>` using `ChildParentsMap` or direct `add_node` and `add_edge` (extends relationship).
2. Convert to `InheritanceGraph<InheritanceNode>` via `TryFrom`.
3. Continue with existing payload wrapping.

When needed, use `add_edge_with` to attach `ExtendsChangeKind` on raw edges during processing, then drop it on validation.

### 7.2 Replace GraphEditor usage
`build_graph` currently uses `GraphEditor::from_parts`.
New flow:
1. Build editor using `InheritanceEditor::new(validated_graph)`.
2. Use `insert_node`, `apply_change`, `delete_node` as before.
3. Call `patch()` to get a validated graph (scoped).

### 7.2a Move extends metadata to edges
- Remove `extends_change` from `PostProcessNode`.
- Track extends relation changes using `Edge<ExtendsChangeKind>` in the raw graph used by `build_graph`.
- When determining merge roots, derive the change kind from the edge relation for each node (e.g., edge from parent -> child carries the change kind for the child's extends).

### 7.3 Replace `InheritanceGraph::from_parts` call sites
All places that currently rewrap `nodes/order/roots` must change to:
- Build raw `Graph<T, R>` from `nodes` + edges using a dedicated helper (see Phase 7.5).
- Convert via `TryFrom`

Specific call sites to update:
- `build_present_graph`
- `compare`
- `parse`
- `analyze_properties`
- `refresh_metadata`
- `build_graph` (final assembly)
- `builder.rs` test helper (`setup_test_repo_with_graph`)
- `graph.rs` tests (`build_diamond_graph`, `patch_updates_bidirectional_links`, etc.)
- `inheritance.rs` tests (`build_diamond_graph`)

### 7.4 Ensure Node access traits remain correct
`PreProcessNode` and `PostProcessNode` still implement `GraphNode` for editor operations.

### 7.5 Add helper: rebuild raw graph from node map + parent lists
- Implement a shared helper (likely in `graph.rs`):
  ```rust
   fn graph_from_nodes_and_parents<T, R, F>(
       nodes: &HashMap<SchemaId, T>,
       clone_payload: F,
   ) -> Graph<TPayload, R>
   where
       T: NodeAccessor,
       F: Fn(&T) -> TPayload,
       R: Default;
   ```
- This ensures every call site uses the same normalization rules without requiring `T: Clone`.

---

## Phase 8: Tests & Docs
### 8.1 graph.rs tests
- Replace `from_parts` usage with raw `Graph<T>` + `TryFrom`.
- Update `patch_updates_bidirectional_links` to use `InheritanceEditor`.

### 8.2 inheritance.rs tests
- Replace `from_parts` usage with `Graph<T>` + `TryFrom` or editor paths.

### 8.3 Update module docs
- Remove references to `GraphBuilder` / `GraphEditor`.
- Document `Graph<T>` + `InheritanceEditor`.

---

## Phase 9: Cleanup & Quality
### 9.1 Remove obsolete types
- Remove leftover `GraphBuilder` references and exports.
- Keep `ChildParentsMap` as RawSchema ingestion shape.

### 9.2 Run checks
- `mise run fmt`
- `mise run lint`
- `mise run test:unit:schema`

---

## Risks / Mitigations
- **Performance risk**: recomputing adjacency on demand each time.
  - Mitigation: keep adjacency computed only in the scope where needed (editor + TryFrom). If hotspots appear, consider caching adjacency in the editor only.
- **Behavioral risk**: order/roots preservation logic in scoped updates.
  - Mitigation: add dedicated tests for splice ordering and root derivation.

---

## Deliverables
- Updated `graph.rs` with raw `Graph<T>`, `AdjacencyMap`, `InheritanceEditor`, `TopologicalOrder`.
- Updated `inheritance.rs` removing `from_parts` + `validate_consistency`.
- Updated `schema_processor.rs` using `Graph<T>` + `TryFrom` and `InheritanceEditor`.
- Updated tests and docs.
