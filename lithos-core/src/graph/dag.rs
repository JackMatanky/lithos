//! Validated DAG wrapper for graph structures.
//!
//! Wraps a raw `Graph` and caches topological order and roots after validation.
//!
//! # Examples
//!
//! ```
//! use lithos_core::graph::{DagGraph, GraphBuilder};
//!
//! let mut builder = GraphBuilder::new();
//! builder.add_node(1u8, Box::<str>::from("A"));
//! builder.add_node(2u8, Box::<str>::from("B"));
//! builder.add_parent(2, 1);
//!
//! let dag = DagGraph::try_from(builder.build()).unwrap();
//! assert_eq!(dag.roots(), &[1]);
//! ```

use std::hash::Hash;

use crate::graph::{
    Graph, GraphError,
    node::{DiGraphNode, GraphNode},
    sorting::topological_sort_with_nodes,
};

/// Validated DAG wrapper that owns the graph and caches topology.
///
/// **Pure infrastructure** - NO serialization constraint. Domain wrappers
/// (like `schema::InheritanceGraph`) add Archive bounds when needed.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{DagGraph, GraphBuilder, Node};
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Box::<str>::from("A"));
/// builder.add_node(2u8, Box::<str>::from("B"));
/// builder.add_parent(2, 1);
///
/// let dag = DagGraph::try_from(builder.build::<Node<_>>()).unwrap();
/// assert_eq!(dag.topo_order(), &[1, 2]);
/// ```
#[derive(Debug, Clone)]
pub struct DagGraph<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode + DiGraphNode,
{
    graph: Graph<Id, N>,
    topo_order: Vec<Id>,
    roots: Vec<Id>,
}

impl<Id, N> TryFrom<Graph<Id, N>> for DagGraph<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode + DiGraphNode,
{
    type Error = GraphError<Id>;

    #[inline]
    fn try_from(graph: Graph<Id, N>) -> Result<Self, Self::Error> {
        let (order, roots) =
            topological_sort_with_nodes(graph.parents(), graph.node_ids())?;
        Ok(Self {
            graph,
            topo_order: order,
            roots,
        })
    }
}

impl<Id, N> DagGraph<Id, N>
where
    Id: Copy + Eq + Hash + Ord,
    N: GraphNode + DiGraphNode,
{
    /// Returns the cached topological order.
    #[inline]
    #[must_use]
    pub fn topo_order(&self) -> &[Id] {
        &self.topo_order
    }

    /// Returns the cached roots (nodes with no parents).
    #[inline]
    #[must_use]
    pub fn roots(&self) -> &[Id] {
        &self.roots
    }

    /// Returns a reference to the underlying raw graph.
    #[inline]
    #[must_use]
    pub fn graph(&self) -> &Graph<Id, N> {
        &self.graph
    }

    /// Returns a mutable reference to the underlying raw graph.
    ///
    /// **Warning**: Mutating the graph structure (adding/removing nodes or
    /// edges) will invalidate the cached topological order. Only use this
    /// for in-place payload updates.
    #[inline]
    pub fn graph_mut(&mut self) -> &mut Graph<Id, N> {
        &mut self.graph
    }

    /// Consumes the wrapper and returns the raw graph.
    #[inline]
    #[must_use]
    pub fn into_graph(self) -> Graph<Id, N> {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod try_from {
        use super::*;
        use crate::graph::{GraphBuilder, node::Node};

        #[test]
        fn rejects_cycles() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Box::<str>::from("A"));
            builder.add_node(2i32, Box::<str>::from("B"));
            builder.add_parent(1i32, 2i32);
            builder.add_parent(2i32, 1i32);

            let graph = builder.build::<Node<_>>();
            let result = DagGraph::try_from(graph);

            assert!(
                matches!(&result, Err(GraphError::CycleDetected { .. })),
                "expected cycle error, got {result:?}",
            );
        }

        #[test]
        fn returns_missing_node_error_when_child_absent() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Box::<str>::from("A"));
            builder.add_parent(2i32, 1i32);

            let graph = builder.build::<Node<_>>();
            let result = DagGraph::try_from(graph);

            assert!(
                matches!(&result, Err(GraphError::MissingNode { .. })),
                "expected missing-node error, got {result:?}"
            );
        }
    }

    mod roots {
        use super::*;
        use crate::graph::{GraphBuilder, node::Node};

        #[test]
        fn returns_sorted_roots_for_disconnected_nodes() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1i32, Box::<str>::from("A"));
            builder.add_node(2i32, Box::<str>::from("B"));
            builder.add_node(3i32, Box::<str>::from("C"));
            builder.add_parent(2i32, 1i32);

            let dag = DagGraph::try_from(builder.build::<Node<_>>())
                .expect("expected DAG to be valid");
            assert_eq!(
                dag.roots(),
                &[1i32, 3i32],
                "expected roots [1, 3], got {:?}",
                dag.roots()
            );
        }
    }
}
