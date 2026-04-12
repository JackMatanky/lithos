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

use rkyv::{Archive, Deserialize, Serialize};

use crate::graph::{Graph, GraphError, sorting::topological_sort_with_nodes};

/// Validated DAG wrapper that owns the graph and caches topology.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{DagGraph, GraphBuilder};
///
/// let mut builder = GraphBuilder::new();
/// builder.add_node(1u8, Box::<str>::from("A"));
/// builder.add_node(2u8, Box::<str>::from("B"));
/// builder.add_parent(2, 1);
///
/// let dag = DagGraph::try_from(builder.build()).unwrap();
/// assert_eq!(dag.topo_order(), &[1, 2]);
/// ```
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct DagGraph<Id, T>
where
    Id: Copy + Eq + Hash + Ord + Archive,
    T: Archive,
{
    graph: Graph<Id, T>,

    #[rkyv(with = rkyv::with::Skip)]
    topo_order: Vec<Id>,

    #[rkyv(with = rkyv::with::Skip)]
    roots: Vec<Id>,
}

impl<Id, T> TryFrom<Graph<Id, T>> for DagGraph<Id, T>
where
    Id: Copy + Eq + Hash + Ord + Archive,
    T: Archive,
{
    type Error = GraphError<Id>;

    fn try_from(graph: Graph<Id, T>) -> Result<Self, Self::Error> {
        let (order, roots) =
            topological_sort_with_nodes(graph.parents(), graph.node_ids())?;
        Ok(Self {
            graph,
            topo_order: order,
            roots,
        })
    }
}

impl<Id, T> DagGraph<Id, T>
where
    Id: Copy + Eq + Hash + Ord + Archive,
    T: Archive,
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
    pub fn graph(&self) -> &Graph<Id, T> {
        &self.graph
    }

    /// Consumes the wrapper and returns the raw graph.
    #[inline]
    #[must_use]
    pub fn into_graph(self) -> Graph<Id, T> {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    mod try_from {
        use super::*;

        #[test]
        fn rejects_cycles() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            builder.add_node(2, Box::<str>::from("B"));
            builder.add_parent(1, 2);
            builder.add_parent(2, 1);

            let graph = builder.build();
            let result = DagGraph::try_from(graph);

            assert!(
                matches!(&result, Err(GraphError::CycleDetected { .. })),
                "expected cycle error, got {:?}",
                result
            );
        }

        #[test]
        fn returns_missing_node_error_when_child_absent() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            builder.add_parent(2, 1);

            let graph = builder.build();
            let result = DagGraph::try_from(graph);

            assert!(
                matches!(&result, Err(GraphError::MissingNode { .. })),
                "expected missing-node error, got {:?}",
                result
            );
        }
    }

    mod roots {
        use super::*;

        #[test]
        fn returns_sorted_roots_for_disconnected_nodes() {
            let mut builder = GraphBuilder::new();
            builder.add_node(1, Box::<str>::from("A"));
            builder.add_node(2, Box::<str>::from("B"));
            builder.add_node(3, Box::<str>::from("C"));
            builder.add_parent(2, 1);

            let dag = DagGraph::try_from(builder.build())
                .expect("expected DAG to be valid");
            assert_eq!(
                dag.roots(),
                &[1, 3],
                "expected roots [1, 3], got {:?}",
                dag.roots()
            );
        }
    }
}
