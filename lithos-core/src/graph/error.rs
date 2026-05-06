//! Error types for graph construction and validation.
//!
//! These errors are used by graph algorithms and DAG validation.
//!
//! # Examples
//!
//! ```
//! use lithos_core::graph::GraphError;
//!
//! let err: GraphError<u8> = GraphError::MissingNode {
//!     id: 7,
//! };
//! assert_eq!(err.to_string(), "node not found: 7");
//! ```

use std::hash::Hash;

use thiserror::Error;

/// Errors produced by graph algorithms and validation.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::GraphError;
///
/// let err: GraphError<u8> = GraphError::CycleDetected {
///     nodes: vec![1, 2],
/// };
/// assert_eq!(err.to_string(), "cycle detected in graph");
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GraphError<Id>
where
    Id: Copy + Eq + Hash,
{
    /// Returned when a cycle is detected in the graph.
    #[error("cycle detected in graph")]
    CycleDetected {
        /// IDs involved in cycle (if detectable).
        nodes: Vec<Id>,
    },

    /// Returned when the graph is not directed.
    #[error("graph is not directed (bidirectional edge found)")]
    NotDirected,

    /// Returned when a node referenced by an edge is missing.
    #[error("node not found: {id}")]
    MissingNode {
        /// The ID of the missing node.
        id: Id,
    },
}

/// Alias for cycle-related graph errors.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::CycleError;
///
/// let err: CycleError<u8> = CycleError::CycleDetected {
///     nodes: vec![1],
/// };
/// assert_eq!(err.to_string(), "cycle detected in graph");
/// ```
pub type CycleError<Id> = GraphError<Id>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_detected_displays_message() {
        let err = GraphError::CycleDetected {
            nodes: vec![1u8],
        };
        assert_eq!(
            err.to_string(),
            "cycle detected in graph",
            "expected cycle error display"
        );
    }

    #[test]
    fn missing_node_displays_message() {
        let err = GraphError::MissingNode {
            id: 7u8,
        };
        assert_eq!(
            err.to_string(),
            "node not found: 7",
            "expected missing-node display"
        );
    }

    #[test]
    fn not_directed_displays_message() {
        let err: GraphError<u8> = GraphError::NotDirected;
        assert_eq!(
            err.to_string(),
            "graph is not directed (bidirectional edge found)",
            "expected not-directed display"
        );
    }
}
