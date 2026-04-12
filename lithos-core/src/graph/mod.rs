//! Directed graph infrastructure with optional DAG validation.
//!
//! This module provides a raw graph plus a validated DAG wrapper with cached
//! topological ordering and zero-copy serialization support.
//!
//! # Architecture
//!
//! The graph infrastructure uses a builder pattern for construction:
//!
//! ```text
//! GraphBuilder<Id, T>  (mutable)
//!     ↓ build()
//! Graph<Id, T>         (immutable, raw)
//!     ↓ try_into()
//! DagGraph<Id, T>      (validated, cached topology)
//! ```
//!
//! # Design Decisions
//!
//! - **ID storage**: HashMap key only (not in Node)
//! - **Edges**: Adjacency lists (no Edge struct)
//! - **Generics**: Id + payload T (no unused R)
//! - **Caching**: Topological order computed once in `DagGraph`
//! - **Serialization**: rkyv-native (skips cached data)
//!
//! # Examples
//!
//! Building a schema inheritance graph:
//!
//! ```
//! use lithos_core::graph::{DagGraph, GraphBuilder, GraphError};
//!
//! # fn main() -> Result<(), GraphError<u8>> {
//! let mut builder = GraphBuilder::new();
//! builder.add_node(1u8, Box::<str>::from("SchemaA"));
//! builder.add_node(2u8, Box::<str>::from("SchemaB"));
//! builder.add_parent(2, 1); // SchemaB extends SchemaA
//!
//! let dag = DagGraph::try_from(builder.build())?;
//! assert_eq!(dag.topo_order(), &[1, 2]);
//! # Ok(())
//! # }
//! ```

mod core;
mod dag;
mod error;
mod node;
mod sorting;

pub use core::{Graph, GraphBuilder};

pub use dag::DagGraph;
pub use error::{CycleError, GraphError};
pub use node::{Node, NodeDepth};
pub(crate) use sorting::topological_sort_with_nodes;
