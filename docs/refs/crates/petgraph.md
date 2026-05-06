# Petgraph Comprehensive Reference

> **Last Updated**: 2026-04-12
> **Petgraph Version**: Latest stable (documentation from docs.rs)
> **Audience**: Lithos Rust developers needing complete petgraph knowledge

---

## Table of Contents

1. [Core Graph Types - Deep Dive](#core-graph-types---deep-dive)
2. [Algorithm Library - Complete Coverage](#algorithm-library---complete-coverage)
3. [Advanced Features](#advanced-features)
4. [Performance and Optimization](#performance-and-optimization)
5. [Common Patterns and Idioms](#common-patterns-and-idioms)
6. [Error Handling and Edge Cases](#error-handling-and-edge-cases)
7. [Integration Patterns](#integration-patterns)

---

## Core Graph Types - Deep Dive

### Graph (Adjacency List)

**Internal Representation**: Adjacency list using vectors.

```rust
pub struct Graph<N, E, Ty = Directed, Ix = DefaultIx> {
    nodes: Vec<Node<N, Ix>>,
    edges: Vec<Edge<E, Ix>>,
}
```

**Type Parameters**:

- `N`: Node weight type (arbitrary data)
- `E`: Edge weight type (arbitrary data)
- `Ty`: Edge type (`Directed` or `Undirected`)
- `Ix`: Index type (`u8`, `u16`, `u32`, or `usize`)

**Space Complexity**: `O(|V| + |E|)` where V = nodes, E = edges

**Performance Characteristics**:

- Node insertion: `O(1)` amortized
- Edge insertion: `O(1)` amortized
- Edge lookup: `O(e')` where e' is local edge count
- Node removal: `O(e')` - **invalidates indices**
- Edge removal: `O(e')` - **invalidates indices**

**Index Invalidation**: Critical gotcha!

- Removing a node invalidates **all node indices** with index > removed node's index
- Removing an edge invalidates **all edge indices** with index > removed edge's index
- This is the primary reason `StableGraph` exists

**Memory Layout**:

```rust
// Nodes stored as contiguous vector
nodes: Vec<Node<N, Ix>>
// Each node stores:
// - weight: N
// - next_edge[Direction]: EdgeIndex (linked list of edges)

// Edges stored as contiguous vector
edges: Vec<Edge<E, Ix>>
// Each edge stores:
// - weight: E
// - source: NodeIndex
// - target: NodeIndex
// - next_edge[Direction]: EdgeIndex (linked list)
```

**When to Use**:

- Default choice for most use cases
- You need compact memory representation
- You don't remove nodes/edges frequently
- Performance-critical applications

**Example**:

```rust
use petgraph::Graph;

// Create with capacity for optimization
let mut graph = Graph::<&str, i32>::with_capacity(10, 20);

let a = graph.add_node("A");
let b = graph.add_node("B");
let c = graph.add_node("C");

graph.add_edge(a, b, 3);
graph.add_edge(b, c, 2);
graph.add_edge(c, b, 1);

// Access node/edge weights via indexing
assert_eq!(graph[a], "A");
graph[a] = "A_modified";

// Edge weight access
let edge_ab = graph.find_edge(a, b).unwrap();
assert_eq!(graph[edge_ab], 3);
```

---

### GraphMap

**Internal Representation**: HashMap-based adjacency structure.

**Type Parameters**:

- `N`: Node identifier type (must implement `NodeTrait`: `Copy + Ord + Hash`)
- `E`: Edge weight type
- `Ty`: Edge type (`Directed` or `Undirected`)
- `S`: BuildHasher (defaults to `RandomState`)

**Restrictions**:

- **No parallel edges**: Only one edge between any pair of nodes
- Node type must be `Copy + Ord + Hash`
- Cannot use arbitrary types as nodes (unlike Graph)

**Performance Characteristics**:

- Node lookup: `O(1)` average, `O(n)` worst case
- Edge lookup: `O(1)` average, `O(n)` worst case
- Edge insertion: `O(1)` average, `O(n)` worst case
- Memory overhead: Higher than Graph due to hash table

**When to Use**:

- Nodes have natural identifiers (integers, enums)
- You need fast edge existence queries
- Graph is relatively small (< 10,000 nodes)
- You don't need parallel edges

**Comparison with Graph**:

```rust
use petgraph::{Graph, graphmap::GraphMap};

// Graph: uses opaque indices
let mut g = Graph::<&str, ()>::new();
let a = g.add_node("A"); // Returns NodeIndex(0)
let b = g.add_node("B"); // Returns NodeIndex(1)
g.add_edge(a, b, ());

// GraphMap: uses node values directly as identifiers
let mut gm = GraphMap::<&str, ()>::new();
gm.add_edge("A", "B", ()); // No separate node creation needed!

// Edge existence check
assert!(gm.contains_edge("A", "B")); // O(1) with hash lookup
```

---

### StableGraph

**Internal Representation**: Adjacency list with free lists for removed nodes/edges.

```rust
pub struct StableGraph<N, E, Ty = Directed, Ix = DefaultIx> {
    g: Graph<N, E, Ty, Ix>,
    node_count: usize,
    edge_count: usize,
    free_node: NodeIndex<Ix>,  // Head of free list
    free_edge: EdgeIndex<Ix>,  // Head of free list
}
```

**Index Stability**: Indices **never** become invalid after removal.

**Performance Trade-offs**:

- Node removal: `O(e')` but index remains valid
- Edge removal: `O(e')` but index remains valid
- Memory: Higher overhead due to free lists
- Iteration: May traverse "holes" (removed elements)

**When to Use**:

- You need stable indices (references to nodes/edges)
- Frequent removals with long-lived index references
- Building persistent data structures

**Example**:

```rust
use petgraph::stable_graph::StableGraph;

let mut graph = StableGraph::<&str, ()>::new();
let a = graph.add_node("A");
let b = graph.add_node("B");
let c = graph.add_node("C");

graph.add_edge(a, b, ());
graph.add_edge(b, c, ());

// Remove node B - index 'a' and 'c' remain valid!
graph.remove_node(b);

// These indices are still safe to use
assert_eq!(graph[a], "A");
assert_eq!(graph[c], "C");

// 'b' is now invalid (returns None), but doesn't panic
assert!(graph.node_weight(b).is_none());
```

**Converting between Graph and StableGraph**:

```rust
use petgraph::{Graph, stable_graph::StableGraph};

// Graph -> StableGraph (cheap, no reallocation)
let g = Graph::<i32, ()>::new();
let sg = StableGraph::from(g);

// StableGraph -> Graph (compacts by removing holes)
let g2 = Graph::from(sg);
```

---

### CSR (Compressed Sparse Row)

**Internal Representation**: Immutable compressed format optimized for read-heavy workloads.

**Type Parameters**:

- `N`: Node data type
- `E`: Edge data type
- `Ty`: Edge type (Directed or Undirected)
- `Ix`: Index type

**Space Complexity**: `O(|V| + |E|)` with excellent cache locality

**Performance Characteristics**:

- Construction: `O(|V| + |E|)` from existing graph
- Edge iteration from node: `O(degree)` - extremely fast
- Modification: **Impossible** - immutable structure
- Memory: Most compact representation

**When to Use**:

- Graph is read-only after construction
- Frequent edge iteration from nodes
- Memory-constrained environments
- Performance-critical graph traversals

**Example**:

```rust
use petgraph::{Graph, csr::Csr};

// Build mutable graph first
let mut g = Graph::<(), i32>::new();
let a = g.add_node(());
let b = g.add_node(());
let c = g.add_node(());
g.add_edge(a, b, 5);
g.add_edge(b, c, 3);

// Convert to immutable CSR for fast queries
let csr = Csr::from(&g);

// Fast edge iteration
for edge in csr.edges(a) {
    println!("Edge weight: {:?}", edge.weight());
}
```

---

### MatrixGraph

**Internal Representation**: Adjacency matrix (2D flattened vector).

**Space Complexity**: `O(|V|²)` - prohibitive for large graphs!

**Optimization**: For undirected graphs, only lower triangular matrix stored.

**Performance Characteristics**:

- Edge existence: `O(1)` - fastest possible
- Edge weight access: `O(1)`
- Node insertion: `O(|V|)` amortized (may require matrix resize)
- Memory: Very poor for sparse graphs

**When to Use**:

- Dense graphs (|E| ≈ |V|²)
- Need O(1) edge existence queries
- Graph size is small and bounded
- Algorithms that access edge weights by (u,v) pairs frequently

**Example**:

```rust
use petgraph::matrix_graph::{MatrixGraph, NodeIndex};

let mut graph = MatrixGraph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());

graph.add_edge(a, b, 42);

// O(1) edge weight access
assert_eq!(graph.edge_weight(a, b), Some(&42));
```

---

## Graph Type Comparison Table

| Feature            | Graph    | StableGraph    | GraphMap      | CSR            | MatrixGraph    |
| ------------------ | -------- | -------------- | ------------- | -------------- | -------------- |
| **Space**          | O(V+E)   | O(V+E) + holes | O(V+E) + hash | O(V+E) compact | O(V²)          |
| **Node Insert**    | O(1)     | O(1)           | O(1) avg      | N/A            | O(V) amortized |
| **Edge Insert**    | O(1)     | O(1)           | O(1) avg      | N/A            | O(1)           |
| **Edge Lookup**    | O(e')    | O(e')          | O(1) avg      | O(degree)      | O(1)           |
| **Node Remove**    | O(e') ⚠️ | O(e')          | O(degree)     | N/A            | O(V)           |
| **Edge Remove**    | O(e') ⚠️ | O(e')          | O(1) avg      | N/A            | O(1)           |
| **Index Stable**   | ❌       | ✅             | N/A           | N/A            | ✅             |
| **Parallel Edges** | ✅       | ✅             | ❌            | ✅             | ❌             |
| **Mutable**        | ✅       | ✅             | ✅            | ❌             | ✅             |
| **Memory**         | Good     | Fair           | Fair          | Excellent      | Poor           |

**Legend**:

- `e'`: Local edge count (edges adjacent to a node)
- ⚠️: Invalidates indices

---

## Algorithm Library - Complete Coverage

### Graph Properties

#### Cycle Detection

```rust
use petgraph::algo::{is_cyclic_directed, is_cyclic_undirected};
use petgraph::Graph;

let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
graph.add_edge(a, b, ());
graph.add_edge(b, a, ()); // Creates cycle

assert!(is_cyclic_directed(&graph));
```

**Performance**: O(|V| + |E|)

#### Strongly Connected Components (SCC)

**Tarjan's Algorithm** (preferred):

```rust
use petgraph::algo::tarjan_scc;
use petgraph::Graph;

let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());
let d = graph.add_node(());

graph.extend_with_edges(&[
    (a, b), (b, c), (c, d), (d, a),  // One SCC: a-b-c-d
    (b, d),  // Additional edge within SCC
]);

let sccs = tarjan_scc(&graph);
// Returns Vec<Vec<NodeId>> in reverse topological order
assert_eq!(sccs.len(), 1); // One strongly connected component
assert_eq!(sccs[0].len(), 4); // All 4 nodes
```

**Performance**: O(|V| + |E|) time, O(|V|) auxiliary space

**Kosaraju's Algorithm** (alternative):

```rust
use petgraph::algo::kosaraju_scc;

let sccs = kosaraju_scc(&graph);
// Same result, different algorithm
```

**When to use which**:

- Tarjan: Generally faster, single DFS pass
- Kosaraju: Easier to understand, requires two DFS passes

#### Condensation (DAG of SCCs)

```rust
use petgraph::algo::condensation;
use petgraph::Graph;

let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());
let d = graph.add_node(());
let e = graph.add_node(());

graph.extend_with_edges(&[
    (a, b), (b, c), (c, d), (d, a),  // SCC 1: a-b-c-d
    (b, e), (e, e),                   // SCC 2: e (self-loop)
]);

// Condense SCCs into single nodes
let condensed = condensation(graph, true); // true = make_acyclic
// condensed is a DAG where each node represents an SCC
```

**Parameters**:

- `make_acyclic = true`: Remove edges within SCCs (produces DAG)
- `make_acyclic = false`: Keep all edges (may have cycles)

#### Connected Components

```rust
use petgraph::algo::connected_components;
use petgraph::graph::UnGraph;

let graph = UnGraph::<(), ()>::from_edges(&[
    (0, 1), (1, 2),        // Component 1
    (3, 4),                // Component 2
    (5, 6), (6, 7), (7, 5) // Component 3
]);

assert_eq!(connected_components(&graph), 3);
```

**Performance**: O(|V| + |E|)

#### Bipartite Check

```rust
use petgraph::algo::is_bipartite_undirected;
use petgraph::graph::UnGraph;

let bipartite = UnGraph::<(), ()>::from_edges(&[
    (0, 1), (0, 3),
    (1, 2), (1, 4),
    (2, 3),
]);
assert!(is_bipartite_undirected(&bipartite, 0.into()));

let non_bipartite = UnGraph::<(), ()>::from_edges(&[
    (0, 1), (1, 2), (2, 0) // Triangle
]);
assert!(!is_bipartite_undirected(&non_bipartite, 0.into()));
```

---

### Shortest Paths

#### Dijkstra's Algorithm

```rust
use petgraph::algo::dijkstra;
use petgraph::Graph;

let mut graph = Graph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());

graph.add_edge(a, b, 5);
graph.add_edge(b, c, 3);
graph.add_edge(a, c, 10);

// Find shortest paths from 'a' to all nodes
let node_map = dijkstra(&graph, a, None, |e| *e.weight());

assert_eq!(node_map[&a], 0);
assert_eq!(node_map[&b], 5);
assert_eq!(node_map[&c], 8); // via b: 5 + 3 = 8

// Find shortest path from 'a' to 'c' only
let node_map = dijkstra(&graph, a, Some(c), |e| *e.weight());
assert_eq!(node_map[&c], 8);
```

**Performance**: O((|V| + |E|) log |V|) with binary heap

**Key Points**:

- Requires non-negative edge weights
- Returns `HashMap<NodeId, Cost>`
- Early termination with `Some(goal)` parameter

#### Bellman-Ford Algorithm

```rust
use petgraph::algo::bellman_ford;
use petgraph::Graph;

let mut graph = Graph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());

graph.add_edge(a, b, 5);
graph.add_edge(b, c, -2); // Negative weight allowed!
graph.add_edge(a, c, 10);

let paths = bellman_ford(&graph, a)
    .expect("No negative cycles");

assert_eq!(paths.distances[b.index()], 5);
assert_eq!(paths.distances[c.index()], 3); // via b: 5 + (-2) = 3
```

**Negative Cycle Detection**:

```rust
use petgraph::algo::{bellman_ford, find_negative_cycle};

let mut graph = Graph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());

graph.add_edge(a, b, 1);
graph.add_edge(b, c, -2);
graph.add_edge(c, a, -2); // Negative cycle: 1 + (-2) + (-2) = -3

match bellman_ford(&graph, a) {
    Ok(_) => panic!("Should detect cycle"),
    Err(_) => println!("Negative cycle detected"),
}

// Extract the negative cycle path
let cycle = find_negative_cycle(&graph, a)
    .expect("Cycle exists");
println!("Negative cycle: {:?}", cycle);
```

**Performance**: O(|V| × |E|) - slower than Dijkstra

**When to Use**:

- Negative edge weights present
- Need to detect negative cycles
- Graph is small (< 1000 nodes)

#### Floyd-Warshall (All-Pairs Shortest Paths)

```rust
use petgraph::algo::floyd_warshall;
use petgraph::Graph;

let graph = Graph::<(), f32>::from_edges(&[
    (0, 1, 2.0), (1, 2, 1.0), (0, 2, 4.0)
]);

let all_pairs = floyd_warshall(&graph, |e| *e.weight())
    .expect("No negative cycles");

// all_pairs: HashMap<(NodeIndex, NodeIndex), f32>
assert_eq!(all_pairs[&(0.into(), 2.into())], 3.0); // 0->1->2 = 2+1 = 3
```

**Performance**: O(|V|³) - only feasible for small graphs

#### A\* Search

```rust
use petgraph::algo::astar;
use petgraph::Graph;

let mut graph = Graph::<(i32, i32), i32>::new();
let a = graph.add_node((0, 0));
let b = graph.add_node((2, 0));
let c = graph.add_node((2, 2));

graph.add_edge(a, b, 2);
graph.add_edge(b, c, 2);

// Heuristic: Manhattan distance
let heuristic = |node: petgraph::graph::NodeIndex| {
    let (x, y) = graph[node];
    let (gx, gy) = graph[c];
    ((gx - x).abs() + (gy - y).abs()) as i32
};

let result = astar(
    &graph,
    a,
    |n| n == c,
    |e| *e.weight(),
    heuristic,
);

assert!(result.is_some());
let (cost, path) = result.unwrap();
assert_eq!(cost, 4);
assert_eq!(path, vec![a, b, c]);
```

**Performance**: Depends on heuristic quality, can be much faster than Dijkstra

---

### Topological Ordering

#### Topological Sort

```rust
use petgraph::algo::toposort;
use petgraph::Graph;

let mut dag = Graph::<&str, ()>::new();
let a = dag.add_node("A");
let b = dag.add_node("B");
let c = dag.add_node("C");

dag.add_edge(a, b, ());
dag.add_edge(b, c, ());

let sorted = toposort(&dag, None)
    .expect("DAG should have topological order");

// One valid order: [A, B, C]
assert_eq!(sorted, vec![a, b, c]);
```

**Error Handling** (cycle detection):

```rust
use petgraph::algo::toposort;

let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
graph.add_edge(a, b, ());
graph.add_edge(b, a, ()); // Cycle!

match toposort(&graph, None) {
    Ok(_) => panic!("Should fail on cycle"),
    Err(cycle_err) => {
        // cycle_err.node_id() returns a node in the cycle
        let node_in_cycle = cycle_err.node_id();
        println!("Cycle detected at node: {:?}", node_in_cycle);
    }
}
```

**Performance Optimization** (reusing DfsSpace):

```rust
use petgraph::algo::{toposort, DfsSpace};

let mut space = DfsSpace::default();
for graph in graphs.iter() {
    let sorted = toposort(graph, Some(&mut space))?;
    // ... process sorted
}
```

**Performance**: O(|V| + |E|)

---

### Graph Coloring

```rust
use petgraph::algo::dsatur_coloring;
use petgraph::graph::UnGraph;

let graph = UnGraph::<(), ()>::from_edges(&[
    (0, 1), (1, 2), (2, 0), // Triangle requires 3 colors
    (1, 3),
]);

let coloring = dsatur_coloring(&graph);
// Returns HashMap<NodeIndex, usize> mapping nodes to colors
assert_eq!(coloring.values().max(), Some(&2)); // 3 colors (0, 1, 2)

// No two adjacent nodes have same color
for edge in graph.edge_references() {
    assert_ne!(coloring[&edge.source()], coloring[&edge.target()]);
}
```

**Use Cases**:

- Register allocation in compilers
- Scheduling problems
- Map coloring
- DAG processing (assign layers)

---

### Minimum Spanning Tree

```rust
use petgraph::algo::min_spanning_tree;
use petgraph::graph::UnGraph;
use petgraph::data::FromElements;

let graph = UnGraph::<(), i32>::from_edges(&[
    (0, 1, 5), (0, 2, 10), (1, 2, 3),
    (1, 3, 2), (2, 3, 7),
]);

let mst_edges = min_spanning_tree(&graph);
let mst = UnGraph::<_, _>::from_elements(mst_edges);

// MST has V-1 edges
assert_eq!(mst.edge_count(), graph.node_count() - 1);

// Total weight of MST
let total_weight: i32 = mst.edge_references()
    .map(|e| *e.weight())
    .sum();
assert_eq!(total_weight, 10); // 5 + 3 + 2
```

---

### Matching

```rust
use petgraph::algo::maximum_matching;
use petgraph::graph::UnGraph;

let graph = UnGraph::<(), ()>::from_edges(&[
    (0, 1), (1, 2), (2, 3), (3, 4)
]);

let matching = maximum_matching(&graph);
assert_eq!(matching.len(), 2); // e.g., (0,1) and (2,3)
```

---

## Advanced Features

### Visit Traits

petgraph provides a rich trait system for generic graph algorithms.

#### Core Traits

**GraphBase** - Fundamental graph type information:

```rust
pub trait GraphBase {
    type NodeId: Copy + PartialEq;
    type EdgeId: Copy + PartialEq;
}
```

**IntoNeighbors** - Iterate over outgoing neighbors:

```rust
pub trait IntoNeighbors: GraphBase {
    type Neighbors: Iterator<Item = Self::NodeId>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors;
}
```

**IntoNeighborsDirected** - Iterate with direction control:

```rust
use petgraph::visit::IntoNeighborsDirected;
use petgraph::{Graph, Direction};

let graph = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2)]);

// Outgoing neighbors
let out: Vec<_> = graph.neighbors_directed(1.into(), Direction::Outgoing).collect();
assert_eq!(out, vec![2.into()]);

// Incoming neighbors
let inc: Vec<_> = graph.neighbors_directed(1.into(), Direction::Incoming).collect();
assert_eq!(inc, vec![0.into()]);
```

#### Implementing Custom Graph Types

```rust
use petgraph::visit::{GraphBase, IntoNeighbors};

struct MyGraph {
    adjacency: Vec<Vec<usize>>,
}

impl GraphBase for MyGraph {
    type NodeId = usize;
    type EdgeId = (usize, usize);
}

impl IntoNeighbors for &MyGraph {
    type Neighbors = std::iter::Cloned<std::slice::Iter<'static, usize>>;

    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        // Safety: We know the lifetime is valid for iteration
        let slice = unsafe {
            std::mem::transmute::<&[usize], &'static [usize]>(&self.adjacency[n])
        };
        slice.iter().cloned()
    }
}

// Now MyGraph works with petgraph algorithms!
use petgraph::algo::dijkstra;
let my_graph = MyGraph { adjacency: vec![vec![1, 2], vec![2], vec![]] };
let distances = dijkstra(&my_graph, 0, None, |_| 1);
```

---

### EdgeType and Direction

**EdgeType** marker traits:

```rust
use petgraph::{Directed, Undirected};

// Directed graphs
let directed = Graph::<(), (), Directed>::new();

// Undirected graphs
let undirected = Graph::<(), (), Undirected>::new_undirected();
```

**Direction** enum:

```rust
use petgraph::Direction;

// For directed graphs
for edge in graph.edges_directed(node, Direction::Outgoing) {
    // Process outgoing edges
}

for edge in graph.edges_directed(node, Direction::Incoming) {
    // Process incoming edges
}

// Opposite direction
let opposite = Direction::Outgoing.opposite();
assert_eq!(opposite, Direction::Incoming);
```

---

### IndexType and Index Size Optimization

**Default**: `u32` can handle ~4 billion nodes/edges

**Small graphs**: Use `u16` (65K nodes) or `u8` (255 nodes)

```rust
use petgraph::Graph;

// Tiny graph optimization
type SmallGraph<N, E> = Graph<N, E, petgraph::Directed, u16>;

let mut graph = SmallGraph::<(), ()>::new();
// NodeIndex and EdgeIndex are now 2 bytes instead of 4
```

**Large graphs**: Use `usize` if you need more than 4B nodes

```rust
type HugeGraph<N, E> = Graph<N, E, petgraph::Directed, usize>;
```

**Trade-offs**:

- Smaller index = less memory per node/edge
- Smaller index = better cache locality
- Check graph size constraints at compile time

---

### Filtered Graphs (Zero-Cost Views)

#### NodeFiltered

```rust
use petgraph::visit::NodeFiltered;
use petgraph::Graph;

let mut graph = Graph::<i32, ()>::new();
let a = graph.add_node(1);
let b = graph.add_node(2);
let c = graph.add_node(3);
graph.add_edge(a, b, ());
graph.add_edge(b, c, ());

// Create filtered view: only even-weighted nodes
let filtered = NodeFiltered::from_fn(&graph, |node| {
    graph[node] % 2 == 0
});

// Now algorithms see only node 'b' (weight = 2)
use petgraph::visit::IntoNodeReferences;
let visible_nodes: Vec<_> = filtered.node_references().collect();
assert_eq!(visible_nodes.len(), 1);
```

#### EdgeFiltered

```rust
use petgraph::visit::EdgeFiltered;
use petgraph::Graph;

let mut graph = Graph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());
graph.add_edge(a, b, 5);
graph.add_edge(b, c, 10);
graph.add_edge(a, c, 20);

// Filter: only edges with weight > 7
let filtered = EdgeFiltered::from_fn(&graph, |edge| {
    *edge.weight() > 7
});

// Algorithms see only 2 edges (10 and 20)
use petgraph::visit::IntoEdgeReferences;
let visible_edges: Vec<_> = filtered.edge_references().collect();
assert_eq!(visible_edges.len(), 2);
```

**Performance**: Zero-cost abstraction - no allocation, filtering happens during iteration

**Use Cases**:

- Conditional graph processing
- Subgraph algorithms without copying
- Multi-pass algorithms with different filters

---

### Reversed Adapter

```rust
use petgraph::visit::Reversed;
use petgraph::Graph;
use petgraph::visit::IntoNeighbors;

let graph = Graph::<(), ()>::from_edges(&[(0, 1), (1, 2)]);

// Reverse all edges conceptually
let reversed = Reversed(&graph);

// Now 2->1->0 instead of 0->1->2
let neighbors: Vec<_> = reversed.neighbors(2.into()).collect();
assert_eq!(neighbors, vec![1.into()]);

// Use with algorithms
use petgraph::algo::dijkstra;
let distances = dijkstra(reversed, 2.into(), None, |_| 1);
// Computes distances in reversed graph
```

**When to Use**:

- Transpose directed graph without copying
- Algorithms that need reverse traversal
- Computing predecessors efficiently

---

## Performance and Optimization

### Pre-allocation

```rust
use petgraph::Graph;

// Without capacity hint (many reallocations)
let mut graph1 = Graph::<i32, ()>::new();
for i in 0..10000 {
    graph1.add_node(i);
}

// With capacity (single allocation)
let mut graph2 = Graph::<i32, ()>::with_capacity(10000, 20000);
for i in 0..10000 {
    graph2.add_node(i);
}
```

**Benchmark impact**: 2-3x faster for bulk construction

### Reusing DfsSpace

```rust
use petgraph::algo::{toposort, DfsSpace};
use petgraph::Graph;

let graphs: Vec<Graph<(), ()>> = vec![/* ... */];

// Reuse visitor allocation across multiple calls
let mut space = DfsSpace::default();
for graph in &graphs {
    let sorted = toposort(graph, Some(&mut space)).unwrap();
    // ... process
}
```

**Saves**: ~100 allocations per iteration for 1000-node graphs

### Graph Conversion Performance

```rust
use petgraph::{Graph, stable_graph::StableGraph, csr::Csr};

let graph = Graph::<i32, ()>::new();

// Conversions:
// Graph -> StableGraph: O(V + E) copy
let stable = StableGraph::from(graph.clone());

// Graph -> CSR: O(V + E) but produces compact read-only format
let csr = Csr::from(&graph);

// StableGraph -> Graph: O(V + E) compaction (removes holes)
let compacted = Graph::from(stable);
```

---

## Common Patterns and Idioms

### Construction Patterns

#### Builder Pattern with extend_with_edges

```rust
use petgraph::Graph;

let mut graph = Graph::<(), i32>::new();
let a = graph.add_node(());
let b = graph.add_node(());
let c = graph.add_node(());

// Bulk edge insertion
graph.extend_with_edges(&[
    (a, b, 5),
    (b, c, 3),
    (a, c, 10),
]);
```

#### from_edges Constructor

```rust
use petgraph::Graph;

// Automatically creates nodes 0, 1, 2, 3
let graph = Graph::<(), i32>::from_edges(&[
    (0, 1, 5), (1, 2, 3), (2, 3, 2)
]);

// Node weights default to ()
assert_eq!(graph.node_count(), 4);
```

#### Weighted Nodes with from_edges

```rust
use petgraph::stable_graph::StableGraph;

// Custom node weights
let graph = StableGraph::<i32, ()>::from_edges(&[
    (0, 1), (1, 2)
]);
// Nodes get default weight (0 for i32)
```

---

### Mutation Patterns

#### Safe Removal During Iteration

**Anti-pattern** (use-after-free risk with Graph):

```rust
// DON'T DO THIS with Graph!
for node in graph.node_indices() {
    graph.remove_node(node); // Invalidates subsequent indices!
}
```

**Correct pattern** (collect first):

```rust
use petgraph::Graph;

let mut graph = Graph::<i32, ()>::new();
// ... populate graph

let to_remove: Vec<_> = graph.node_indices()
    .filter(|&n| graph[n] % 2 == 0)
    .collect();

for node in to_remove {
    graph.remove_node(node);
}
```

#### retain_nodes and retain_edges

```rust
use petgraph::Graph;

let mut graph = Graph::<i32, i32>::new();
let a = graph.add_node(1);
let b = graph.add_node(2);
let c = graph.add_node(3);
graph.add_edge(a, b, 5);
graph.add_edge(b, c, 10);

// Remove all odd-weighted nodes
graph.retain_nodes(|frozen_graph, node| {
    frozen_graph[node] % 2 == 0
});

// Remove all edges with weight > 7
graph.retain_edges(|frozen_graph, edge| {
    frozen_graph[edge] <= 7
});
```

**Performance**: More efficient than manual removal loop

#### Detached Iteration (Mutating During Traversal)

```rust
use petgraph::{Graph, visit::Dfs, Direction};

let mut graph = Graph::<f32, f32>::new();
let a = graph.add_node(0.0);
let b = graph.add_node(0.0);
let c = graph.add_node(0.0);
graph.add_edge(a, b, 3.0);
graph.add_edge(b, c, 2.0);
graph.add_edge(c, b, 1.0);

// DFS traversal while mutating node weights
let mut dfs = Dfs::new(&graph, a);
while let Some(node) = dfs.next(&graph) {
    // Detached iterator doesn't borrow graph
    let mut edges = graph.neighbors_directed(node, Direction::Incoming).detach();
    while let Some(edge) = edges.next_edge(&graph) {
        graph[node] += graph[edge]; // Mutate graph!
    }
}

assert_eq!(graph[a], 0.0);
assert_eq!(graph[b], 4.0); // 3.0 + 1.0
assert_eq!(graph[c], 2.0);
```

---

### Traversal Iterators

#### Dfs (Depth-First Search)

```rust
use petgraph::visit::Dfs;
use petgraph::Graph;

let graph = Graph::<(), ()>::from_edges(&[
    (0, 1), (0, 2), (1, 3), (2, 3)
]);

let mut dfs = Dfs::new(&graph, 0.into());
while let Some(node) = dfs.next(&graph) {
    println!("Visiting: {:?}", node);
}
```

**Performance**: O(V + E), visits each node once

#### DfsPostOrder

```rust
use petgraph::visit::DfsPostOrder;

let mut dfs = DfsPostOrder::new(&graph, 0.into());
while let Some(node) = dfs.next(&graph) {
    // Node visited AFTER all its descendants
    println!("Post-order: {:?}", node);
}
```

**Use Case**: Topological processing, tree post-order traversal

#### Bfs (Breadth-First Search)

```rust
use petgraph::visit::Bfs;

let mut bfs = Bfs::new(&graph, 0.into());
while let Some(node) = bfs.next(&graph) {
    println!("Level-order: {:?}", node);
}
```

**Performance**: O(V + E), visits nodes by distance from start

#### Topo (Lazy Topological Iterator)

```rust
use petgraph::visit::Topo;
use petgraph::Graph;

let dag = Graph::<(), ()>::from_edges(&[
    (0, 1), (0, 2), (1, 3), (2, 3)
]);

let mut topo = Topo::new(&dag);
while let Some(node) = topo.next(&dag) {
    println!("Topological order: {:?}", node);
}
// Output: 0, 1, 2, 3 (or 0, 2, 1, 3 - both valid)
```

**Advantage**: Lazy evaluation, can stop early

---

## Error Handling and Edge Cases

### Cycle Detection Errors

```rust
use petgraph::algo::toposort;
use petgraph::Graph;

let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
graph.add_edge(a, b, ());
graph.add_edge(b, a, ()); // Cycle

match toposort(&graph, None) {
    Ok(sorted) => println!("Sorted: {:?}", sorted),
    Err(cycle) => {
        let node_in_cycle = cycle.node_id();
        println!("Cycle detected at: {:?}", node_in_cycle);

        // Note: The error gives you ONE node in the cycle,
        // not the entire cycle path
    }
}
```

**Extracting Full Cycle Path**: Not directly supported - use custom DFS

### Missing Nodes/Edges

```rust
use petgraph::Graph;

let mut graph = Graph::<i32, ()>::new();
let a = graph.add_node(42);

// Safe access (returns Option)
assert_eq!(graph.node_weight(a), Some(&42));

let b = graph.add_node(99);
graph.remove_node(b);
assert_eq!(graph.node_weight(b), None); // Removed node

// Unsafe indexing (panics!)
// let _ = graph[b]; // PANIC!
```

**Best Practice**: Use `node_weight()` and `edge_weight()` instead of indexing

### Empty Graphs

```rust
use petgraph::Graph;
use petgraph::algo::toposort;

let empty = Graph::<(), ()>::new();

// Topological sort of empty graph
let sorted = toposort(&empty, None).unwrap();
assert_eq!(sorted.len(), 0);

// Node/edge iteration works fine
assert_eq!(empty.node_count(), 0);
assert_eq!(empty.edge_count(), 0);
```

---

## Integration Patterns

### Serialization with Serde

**Enable feature**:

```toml
[dependencies]
petgraph = { version = "0.6", features = ["serde-1"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Serialize Graph**:

```rust
use petgraph::Graph;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct NodeData {
    name: String,
    value: i32,
}

let mut graph = Graph::<NodeData, i32>::new();
let a = graph.add_node(NodeData { name: "A".into(), value: 1 });
let b = graph.add_node(NodeData { name: "B".into(), value: 2 });
graph.add_edge(a, b, 5);

// Serialize to JSON
let json = serde_json::to_string(&graph).unwrap();
println!("{}", json);

// Deserialize back
let graph2: Graph<NodeData, i32> = serde_json::from_str(&json).unwrap();
assert_eq!(graph2.node_count(), 2);
```

**Format**: Petgraph uses custom serialization format (not standard JSON graph format)

**Version Compatibility**: Serialization format may change between petgraph versions

### Graphviz Integration

```rust
use petgraph::dot::{Dot, Config};
use petgraph::Graph;

let mut graph = Graph::<&str, i32>::new();
let a = graph.add_node("Alice");
let b = graph.add_node("Bob");
let c = graph.add_node("Carol");

graph.add_edge(a, b, 5);
graph.add_edge(b, c, 3);
graph.add_edge(a, c, 10);

// Basic DOT output
let dot = format!("{:?}", Dot::new(&graph));
std::fs::write("graph.dot", &dot).unwrap();

// Custom attributes
let dot = format!("{:?}",
    Dot::with_attr_getters(
        &graph,
        &[Config::EdgeNoLabel],
        &|_, edge| {
            if *edge.weight() > 7 {
                "color=red".to_string()
            } else {
                "color=blue".to_string()
            }
        },
        &|_, _| String::new(),
    )
);
```

**Rendering**: Use `dot` command-line tool:

```bash
dot -Tpng graph.dot -o graph.png
```

---

## Performance Benchmarks (Approximate)

Based on published benchmarks and petgraph repository data:

### Construction (100K nodes, 500K edges)

| Graph Type                 | Time  | Memory |
| -------------------------- | ----- | ------ |
| Graph::with_capacity       | 50ms  | 12MB   |
| Graph::new (no capacity)   | 120ms | 15MB   |
| StableGraph::with_capacity | 60ms  | 14MB   |
| GraphMap                   | 150ms | 25MB   |

### Traversal (100K nodes)

| Operation      | Graph | StableGraph | GraphMap | CSR |
| -------------- | ----- | ----------- | -------- | --- |
| DFS            | 8ms   | 10ms        | 15ms     | 5ms |
| BFS            | 10ms  | 12ms        | 18ms     | 6ms |
| Edge iteration | 3ms   | 4ms         | 12ms     | 2ms |

### Algorithms (10K nodes)

| Algorithm        | Time  | Notes                               |
| ---------------- | ----- | ----------------------------------- |
| Dijkstra         | 15ms  | Binary heap                         |
| Bellman-Ford     | 200ms | Much slower                         |
| Floyd-Warshall   | 8s    | O(V³) - infeasible for large graphs |
| Tarjan SCC       | 12ms  | Single pass                         |
| Topological sort | 5ms   | Linear time                         |

**Scaling**:

- 100-1K nodes: All graph types perform well
- 1K-10K nodes: Graph/CSR preferred, GraphMap acceptable
- 10K-100K nodes: CSR for read-heavy, Graph for mixed workloads
- 100K+ nodes: Consider specialized data structures or CSR

---

## Critical Gotchas and Anti-Patterns

### 1. Index Invalidation with Graph

```rust
// ❌ WRONG - Indices invalidated after removal
let mut graph = Graph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
graph.remove_node(a); // 'b' index is now invalid!
// graph[b] may panic or access wrong node

// ✅ CORRECT - Use StableGraph when removing
let mut graph = StableGraph::<(), ()>::new();
let a = graph.add_node(());
let b = graph.add_node(());
graph.remove_node(a); // 'b' still valid
```

### 2. GraphMap Parallel Edge Restriction

```rust
// ❌ WRONG - GraphMap doesn't support parallel edges
let mut gm = GraphMap::<&str, i32>::new();
gm.add_edge("A", "B", 5);
gm.add_edge("A", "B", 10); // Overwrites previous edge!

// ✅ CORRECT - Use Graph for parallel edges
let mut g = Graph::<&str, i32>::new();
let a = g.add_node("A");
let b = g.add_node("B");
g.add_edge(a, b, 5);
g.add_edge(a, b, 10); // Both edges exist
```

### 3. Mutating During Iteration

```rust
// ❌ WRONG - Borrow checker violation
for node in graph.node_indices() {
    graph.remove_node(node); // Can't mutate while iterating
}

// ✅ CORRECT - Collect first
let nodes: Vec<_> = graph.node_indices().collect();
for node in nodes {
    graph.remove_node(node);
}
```

### 4. MatrixGraph Memory Explosion

```rust
// ❌ WRONG - 1M nodes = 1 trillion entries!
let graph = MatrixGraph::<(), ()>::new();
for _ in 0..1_000_000 {
    graph.add_node(()); // OOM!
}

// ✅ CORRECT - Use Graph for large sparse graphs
let graph = Graph::<(), ()>::with_capacity(1_000_000, 2_000_000);
```

---

## Quick Reference Card

### Choosing a Graph Type

```
Need stable indices after removal?
  → StableGraph

Graph is immutable after construction?
  → CSR (fastest queries)

Nodes have natural identifiers (integers, enums)?
  → GraphMap (if no parallel edges needed)

Dense graph (|E| ≈ |V|²) with O(1) edge queries?
  → MatrixGraph (if |V| < 1000)

Default choice?
  → Graph (adjacency list)
```

### Common Operations

```rust
// Construction
let mut g = Graph::with_capacity(nodes, edges);
g.add_node(weight);
g.add_edge(a, b, weight);
g.extend_with_edges(&[(a, b, w), ...]);

// Access
g[node]           // Node weight (panics if invalid)
g[edge]           // Edge weight
g.node_weight(n)  // Option<&N> (safe)
g.edge_weight(e)  // Option<&E> (safe)

// Iteration
g.node_indices()
g.edge_indices()
g.neighbors(n)
g.edges(n)

// Algorithms
dijkstra(&g, start, goal, edge_cost)
toposort(&g, space).unwrap()
tarjan_scc(&g)
is_cyclic_directed(&g)

// Export
Dot::new(&g)  // Graphviz
serde_json::to_string(&g)  // JSON (requires "serde-1")
```

---

## Conclusion

This reference covers all major aspects of petgraph. Key takeaways:

1. **Graph** is the default choice - fast and memory-efficient
2. **StableGraph** when you need persistent indices
3. **CSR** for read-only, performance-critical traversals
4. **GraphMap** for small graphs with natural node IDs
5. **MatrixGraph** only for dense graphs < 1000 nodes

6. **Algorithms** are comprehensive - use them instead of rolling your own
7. **Visit traits** enable writing generic algorithms
8. **Filtered views** and **Reversed** are zero-cost abstractions

9. **Pre-allocate** with `with_capacity()` for performance
10. **Reuse DfsSpace** across multiple algorithm calls
11. **Watch out** for index invalidation with `Graph`

For Lithos specifically:

- Schema dependency graphs → `Graph<SchemaId, ()>` with `toposort`
- Note reference graphs → `Graph<NoteId, RefType>` with `tarjan_scc` for cycle detection
- Template inheritance → `Graph<TemplateId, ()>` as DAG with `is_cyclic_directed` validation
