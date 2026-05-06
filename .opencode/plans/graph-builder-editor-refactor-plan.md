# Graph Builder/Editor Refactor Plan

## Goal
- Refactor `lithos-core/src/schema/graph.rs` so DAG construction and scoped patching are handled by `GraphBuilder` and `GraphEditor`, while `InheritanceGraph` remains a read-only container with algorithms.
- Ensure `GraphBuilder` and `GraphEditor` operate strictly on `SchemaId` + `Vec<SchemaId>` (no `SchemaName` usage).
- Implement separate `compute_depths` methods on `GraphBuilder` and `GraphEditor`.

## Scope
- Rename `DagBuilder` to `GraphBuilder`.
- Introduce `GraphEditor` for scoped updates (including deletes).
- Remove or restrict mutation helpers on `InheritanceNode` and `InheritanceGraph` to avoid invariant drift.
- Update call sites in schema pipeline to use the new APIs.

## Non-Goals
- No changes to schema parsing or name resolution logic outside the graph module.
- No new domain types or cross-context imports.

## Current Issues to Resolve
- Graph construction and patching logic is split across `InheritanceNode`, `InheritanceGraph`, and `DagBuilder`.
- `DagBuilder` mixes build and patch responsibilities.
- `InheritanceGraph` exposes mutation (`set_parents`, depth recompute) that bypasses invariant pipelines.
- `DagBuilder` currently depends on `SchemaName` and index-based resolution.

## Target Design (High Level)
- `InheritanceNode`: pure storage + minimal constructors (`new_root`, `new_child`, `is_root`).
- `InheritanceGraph`: read-only algorithms + safe accessors + order splicing helpers; no patching.
- `GraphBuilder`: builds full graph from `SchemaId` + parent lists.
- `GraphEditor`: applies scoped updates (including deletes), recomputes only affected metadata.

## API Shape

### GraphBuilder
Public API:
- `pub(crate) fn new() -> Self`
- `pub(crate) fn with_nodes(nodes: HashMap<SchemaId, Vec<SchemaId>>) -> Self`
- `pub(crate) fn insert_node(&mut self, id: SchemaId, parents: Vec<SchemaId>)`
- `pub(crate) fn build(self) -> Result<InheritanceGraph<InheritanceNode>, SchemaLoaderError>`

Internal methods (builder-only):
- `fn build_nodes(&self) -> HashMap<SchemaId, InheritanceNode>`
- `fn validate_parents_exist(nodes: &HashMap<SchemaId, InheritanceNode>) -> Result<(), SchemaLoaderError>`
- `fn build_children(nodes: &mut HashMap<SchemaId, InheritanceNode>)`
- `fn is_not_cyclic(nodes: &HashMap<SchemaId, InheritanceNode>) -> Result<(), SchemaLoaderError>`
- `fn compute_depths(nodes: &mut HashMap<SchemaId, InheritanceNode>)`
- `fn compute_topological_order(nodes: &HashMap<SchemaId, InheritanceNode>) -> Result<(Vec<SchemaId>, Vec<SchemaId>), SchemaResolutionError>`
- `fn assemble(nodes: HashMap<SchemaId, InheritanceNode>, order: Vec<SchemaId>, roots: Vec<SchemaId>) -> InheritanceGraph<InheritanceNode>`

### GraphEditor
Public API:
- `pub(crate) fn from_graph(graph: &InheritanceGraph<InheritanceNode>) -> Self`
- `pub(crate) fn apply_change(&mut self, id: SchemaId, parents: Vec<SchemaId>)`
- `pub(crate) fn delete_node(&mut self, id: SchemaId)`
- `pub(crate) fn patch(self) -> Result<InheritanceGraph<InheritanceNode>, SchemaLoaderError>`

Internal methods (editor-only):
- `fn apply_deletes_cleanup(graph: &mut InheritanceGraph<InheritanceNode>, deleted: &HashSet<SchemaId>)`
- `fn rebuild_children(graph: &mut InheritanceGraph<InheritanceNode>, affected: &HashSet<SchemaId>)`
- `fn is_not_cyclic(graph: &InheritanceGraph<InheritanceNode>, affected: &HashSet<SchemaId>) -> Result<(), SchemaLoaderError>`
- `fn recompute_depths(graph: &mut InheritanceGraph<InheritanceNode>, affected: &HashSet<SchemaId>)`
- `fn compute_topological_order(graph: &InheritanceGraph<InheritanceNode>, affected: &HashSet<SchemaId>) -> Result<Vec<SchemaId>, SchemaResolutionError>`
- `fn splice_order(graph: &mut InheritanceGraph<InheritanceNode>, affected_order: &[SchemaId], affected: &HashSet<SchemaId>) -> Result<(), SchemaLoaderError>`
- `fn rebuild_roots(graph: &mut InheritanceGraph<InheritanceNode>, affected: &HashSet<SchemaId>)`

## Implementation Steps

### 1) Refactor Core Types (Graph Module)
- Make `InheritanceNode` a storage-only type:
  - Keep `new_root`, `new_child`, `is_root`.
  - Remove or make internal-only: `add_parent`, `remove_parent`, `add_child`, `remove_child`.
- Restrict `InheritanceGraph` mutation surface:
  - Remove `set_parents` from `InheritanceGraph`.
  - Move depth computation off `InheritanceGraph` methods into `GraphBuilder` and `GraphEditor` (module-private helpers if needed).

### 2) Rename `DagBuilder` to `GraphBuilder`
- Rename struct and update all internal references.
- Remove SchemaName usage and pending parent resolution.
- Replace `from_new_schemas` / `from_schemas_with_index` with input that already provides parent IDs:
  - `HashMap<SchemaId, Vec<SchemaId>>` or insert calls.

### 3) Implement GraphBuilder Build Pipeline
Implement these internal steps (module-private helpers or private methods):
- `build_nodes`: convert parent lists to `InheritanceNode` with empty children and `depth = ROOT`.
- `validate_parents_exist`: error if any parent ID is missing.
- `build_children`: derive children from parent lists and sort children vectors.
- `validate_no_cycles`: use `DagValidator` across all nodes.
- `compute_depths`: compute depth for all nodes.
- `compute_topological_order`: Kahn sort + roots.
- `assemble`: produce `InheritanceGraph<InheritanceNode>` with order + roots.

### 4) Add GraphEditor for Scoped Updates
State:
- `graph: InheritanceGraph<InheritanceNode>`
- `changed_ids: HashSet<SchemaId>`
- `deleted_ids: HashSet<SchemaId>`

Editor operations:
- `apply_change(id, parents)`:
  - Update node parents; clear node children.
  - Track `changed_ids`.
- `delete_node(id)`:
  - Remove node from graph.
  - Track `deleted_ids` + `changed_ids`.

Scoped finalize:
- `apply_deletes_cleanup`:
  - Remove deleted ids from other nodes' parents/children.
- `affected = graph.affected_subtree(&changed_ids)`
- `rebuild_children`:
  - Recompute children lists for affected nodes and their direct parents.
- `is_not_cyclic`:
  - Use `DagValidator::detect_cycles_scoped`.
- `recompute_depths`:
  - Scoped depth recompute for affected nodes (GraphEditor method).
- `affected_order = graph.topological_sort_scoped(&affected)`
- `graph.splice_order(&affected_order, &affected)`
- `rebuild_roots`:
  - Recompute roots for affected nodes; if too complex, recompute roots globally (document choice).

### 5) Update Call Sites
- Replace `DagBuilder` usage with `GraphBuilder`.
- Update patch/update paths to use `GraphEditor` with scoped finalize.
- Ensure input data to builder/editor is `SchemaId` + `Vec<SchemaId>` (no name resolution).

### 6) Tests
- Update existing tests to new names and APIs.
- Add/adjust tests for:
  - Build from scratch with multiple parents.
  - Scoped update (change parents) keeps unaffected order stable.
  - Scoped delete removes references and updates order/depth.
  - Cycle detection in scoped updates.

## Open Decisions (Resolve Before Coding)
- Whether `rebuild_roots_scoped` recomputes roots globally or incrementally.
  - Recommendation: recompute roots globally for correctness and simplicity.

## Success Criteria
- `InheritanceGraph` is read-only for mutation and no longer owns patching logic.
- `GraphBuilder` and `GraphEditor` exclusively own graph mutation and metadata rebuild.
- Scoped updates correctly recompute order/depth for affected nodes only.
- All call sites use `SchemaId` + `Vec<SchemaId>` inputs.
- Tests cover build and scoped patch behaviors.
