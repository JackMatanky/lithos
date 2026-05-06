# Inheritance Graph Refactor Plan (ProcessedGraph + Scoped Validation)

## Goal
Refactor the schema pipeline to use a `ProcessedGraph` type that provides the flexibility needed for the `SchemaProcessor` while guaranteeing topological validity before being converted to a final `InheritanceGraph`. We will replace the error-prone `InheritanceEditor` stitching logic with a single-pass Kahn's topological sort, strictly adhering to the "parse, don't validate" architectural rule.

## Phase 1: Simplify `graph.rs`
Since we are removing the scoped/incremental depth updates, the `Graph::compute_depths` algorithm can be significantly simplified. Because it processes nodes in a guaranteed topological order, it only needs to look at the immediate previously-computed values.

1.  **Refactor `compute_depths`**:
    *   **Remove parameters**: `base_nodes` and `affected`.
    *   **Update logic**: Iterate over the topologically sorted `order`. For each node's parents, retrieve their depths directly from the local `depth_by_id` map (since parents are guaranteed to be processed before children).
2.  **Delete `parent_depth`**:
    *   Remove this standalone helper function entirely, as it existed solely to fall back to `base_nodes` during incremental `affected` subgraph updates.
3.  **Add `from_parts` constructor**:
    *   Add `pub fn from_parts(nodes: HashMap<SchemaId, Node<T>>, edges: Vec<Edge<R>>) -> Self` to instantiate a raw graph directly from extracted components.
4.  **Review Mutators**:
    *   Ensure `add_node`, `add_edge`, `remove_node`, and `remove_edge` are public and ergonomic.

## Phase 2: Eradicate Incremental Logic in `inheritance.rs`
We will delete the complex algorithmic stitching code that attempted to optimize updates but ended up causing architectural friction.

1.  **Delete `InheritanceEditor`**:
    *   Remove the `InheritanceEditor` struct and its entire `impl` block (`new`, `insert_node`, `apply_change`, `delete_node`, `patch`).
2.  **Delete `try_from_scoped`**:
    *   Remove the `try_from_scoped` method from `InheritanceGraph`.
3.  **Delete Splicing Helpers**:
    *   Remove `splice_order` and `nearest_unaffected_ancestor` standalone functions.
    *   Remove the `ScopedOrder` type alias.
4.  **Update `try_from`**:
    *   Update `InheritanceGraph::try_from` to accept a `ProcessedGraph` and extract the validated data.
5.  **Update Tests**:
    *   Delete the `editor_recomputes_roots_for_removed_parent` test.
    *   Keep `try_from_computes_depths_and_roots` and `affected_subtree` logic.

## Phase 3: Introduce `ProcessedGraph` in `topo_sort.rs`
We will introduce `ProcessedGraph<T, R>` as a validated intermediate step between the completely raw `Graph<T, R>` and the final storage `InheritanceGraph<T>`.

1.  **Create `ProcessedGraph<T, R>`**:
    *   Contains `nodes: HashMap<SchemaId, Node<T>>`, `edges: Vec<Edge<R>>`, `order: TopologicalOrder`, and `adjacency: AdjacencyMap`. This structure makes it trivial to convert to `InheritanceGraph` or inspect nodes without nesting through a raw graph instance.
2.  **Implement `TryFrom<Graph<T, R>> for ProcessedGraph<T, R>`**:
    *   This trait implementation will perform the Kahn's topological sort via `TopologicalSorter`.
    *   It will run `compute_depths` and assign them to the inner graph nodes.
    *   If successful, it guarantees that the graph has a valid topological order and computed depths.
3.  **Add Accessors**:
    *   `nodes() -> &HashMap<SchemaId, Node<T>>`
    *   `edges() -> &[Edge<R>]`
    *   `order() -> &TopologicalOrder`
    *   `adjacency() -> &AdjacencyMap`
    *   `into_parts() -> (HashMap<SchemaId, Node<T>>, Vec<Edge<R>>, TopologicalOrder, AdjacencyMap)`

## Phase 4: Refactor `schema_processor.rs` Pipeline
We will rewrite the `SchemaProcessor::build_graph` method (approx lines 1330–1594) to cleanly project a new raw `Graph` from the incoming `Parsed` state, convert it to a `ProcessedGraph`, and pass the validated structure through the pipeline.

1.  **Extract State**:
    *   Deconstruct the incoming `graph` to get the old nodes, edges, order, and build an `AdjacencyMap` to track old parent/child relationships.
2.  **Build Global Name Index**:
    *   Create a `HashMap<SchemaName, SchemaId>` encompassing all surviving old nodes and all `new_schemas`.
3.  **Initialize Clean Slate**:
    *   Instantiate `mut raw_graph = Graph::<ProcessorNode<InheritanceBranch>, ()>::new()`.
    *   Instantiate `mut edge_relations: HashMap<(SchemaId, SchemaId), ProcessorEdge> = HashMap::new()`.
4.  **Project Surviving Nodes**:
    *   Iterate over the surviving existing nodes.
    *   Determine `old_parent` from the old adjacency map.
    *   Determine `new_parent` by checking the parsed payload's `extends` field against the `name_index` (if the file was re-parsed).
    *   Calculate `ExtendsChangeKind` by comparing `old_parent` and `new_parent`.
    *   Add the node to `raw_graph` using `raw_graph.add_node(...)`. (Depth can be set to `ROOT`).
    *   If `new_parent` exists, wire it up: `raw_graph.add_edge(new_parent, id)` and insert into `edge_relations`.
5.  **Project New Nodes**:
    *   Iterate over `new_schemas`.
    *   Determine `new_parent` via `extends` and `name_index`.
    *   Add the node to `raw_graph`.
    *   If `new_parent` exists, wire it up: `raw_graph.add_edge(new_parent, id)` and insert into `edge_relations` with `ExtendsChangeKind::Unchanged`.
6.  **Validate via `ProcessedGraph`**:
    *   **Crucial Step**: Validate the graph with `let processed_graph = ProcessedGraph::try_from(raw_graph)?`. This guarantees a full topological sort, cycle detection, and depth computation.
7.  **Transition Pipeline**:
    *   Change the pipeline state to use `ProcessedGraph<ProcessorNode<AnalysisBranch>, ()>` instead of `InheritanceGraph`. Note: `edge_relations` will still be tracked outside the graph or encoded inside generic `R`.
    *   Update subsequent stages (`PropertyAnalysis`, `Refresh`, `Construction`, `Completion`) to use the `ProcessedGraph`.
    *   At the very end of `Completion` (or `Construction` if saving to DB), convert `ProcessedGraph` to `InheritanceGraph` using `InheritanceGraph::try_from(processed_graph)`.

## Phase 5: Verification
1.  Run `mise run fmt` to ensure code style consistency.
2.  Run `mise run lint` to confirm no new Clippy warnings are introduced.
3.  Run `mise run test:unit:schema` to guarantee that all pipeline and graph validations still pass perfectly with the new rebuild strategy.
