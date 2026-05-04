//! Node primitives for the graph module.
//!
//! Defines `NodeDepth` and `Node` used by graph structures.
//!
//! # Examples
//!
//! ```
//! use lithos_core::graph::{Node, NodeDepth};
//!
//! let node = Node::new(Box::<str>::from("payload"));
//! assert_eq!(node.depth(), NodeDepth::ROOT);
//! assert_eq!(&**node.payload(), "payload");
//! ```

use std::hash::Hash;

use rkyv::{Archive, Deserialize, Serialize};

/// Provides immutable access to a graph node's payload.
pub trait GraphNode {
    /// The type of payload stored in this node.
    type Payload;

    /// Returns the payload stored in this node.
    #[must_use]
    fn payload(&self) -> &Self::Payload;
}

/// Provides mutable access to a graph node's payload.
pub trait GraphNodeMut: GraphNode {
    /// Returns a mutable reference to the payload.
    fn payload_mut(&mut self) -> &mut Self::Payload;
}

/// Marker trait for directed graph nodes, adding depth tracking.
///
/// Extends GraphNode with depth information for hierarchical graphs.
pub trait DiGraphNode: GraphNode {
    /// Returns the depth of this node in the graph.
    ///
    /// Depth represents the distance from root nodes (depth 0) in a directed
    /// graph.
    #[must_use]
    fn depth(&self) -> NodeDepth;

    /// Consumes the node and returns its payload and depth.
    fn into_parts(self) -> (Self::Payload, NodeDepth);

    /// Constructs a node from a payload and depth.
    fn from_parts(payload: Self::Payload, depth: NodeDepth) -> Self;
}

/// Provides mutable access to directed graph node metadata.
///
/// Extends both GraphNodeMut and DiGraphNode with operations for modifying
/// depth.
pub trait DiGraphNodeMut: GraphNodeMut + DiGraphNode {
    /// Sets the depth of this node.
    fn set_depth(&mut self, depth: NodeDepth);
}

/// A node in the graph.
///
/// **Pure infrastructure** - NO serialization constraint. Domain wrappers
/// add Archive bounds when needed for persistence.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{Node, NodeDepth};
///
/// let mut node = Node::new(Box::<str>::from("payload"));
/// assert_eq!(node.depth(), NodeDepth::ROOT);
/// node.set_depth(NodeDepth::new(1));
/// assert_eq!(node.depth().as_usize(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Node<T> {
    /// Inheritance depth (0 for roots, `max(parent_depths)` + 1 for children).
    depth: NodeDepth,

    /// Application-specific node data.
    payload: T,
}

impl<T> Node<T> {
    /// Creates a new node with ROOT depth.
    #[inline]
    #[must_use]
    pub fn new(payload: T) -> Self {
        Self {
            depth: NodeDepth::ROOT,
            payload,
        }
    }

    /// Returns the node's depth.
    #[inline]
    #[must_use]
    pub fn depth(&self) -> NodeDepth {
        self.depth
    }

    /// Sets the node's depth.
    #[inline]
    pub fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }

    /// Returns a reference to the payload.
    #[inline]
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Returns a mutable reference to the payload.
    #[inline]
    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Consumes the node and returns the payload and depth.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (T, NodeDepth) {
        (self.payload, self.depth)
    }

    /// Creates a node from payload and depth.
    #[inline]
    #[must_use]
    pub fn from_parts(payload: T, depth: NodeDepth) -> Self {
        Self {
            depth,
            payload,
        }
    }
}

// GraphNode trait implementation for Node<T>
impl<T> GraphNode for Node<T> {
    type Payload = T;

    #[inline]
    fn payload(&self) -> &Self::Payload {
        &self.payload
    }
}

// GraphNodeMut trait implementation for Node<T>
impl<T> GraphNodeMut for Node<T> {
    #[inline]
    fn payload_mut(&mut self) -> &mut Self::Payload {
        &mut self.payload
    }
}

// DiGraphNode trait implementation for Node<T>
impl<T> DiGraphNode for Node<T> {
    #[inline]
    fn depth(&self) -> NodeDepth {
        self.depth
    }

    #[inline]
    fn into_parts(self) -> (Self::Payload, NodeDepth) {
        (self.payload, self.depth)
    }

    #[inline]
    fn from_parts(payload: Self::Payload, depth: NodeDepth) -> Self {
        Self {
            depth,
            payload,
        }
    }
}

// DiGraphNodeMut trait implementation for Node<T>
impl<T> DiGraphNodeMut for Node<T> {
    #[inline]
    fn set_depth(&mut self, depth: NodeDepth) {
        self.depth = depth;
    }
}

// Unit type implementations for trivial graph nodes
impl GraphNode for () {
    type Payload = ();

    #[inline]
    fn payload(&self) -> &Self::Payload {
        self
    }
}

impl GraphNodeMut for () {
    #[inline]
    fn payload_mut(&mut self) -> &mut Self::Payload {
        self
    }
}

impl DiGraphNode for () {
    #[inline]
    fn depth(&self) -> NodeDepth {
        NodeDepth::ROOT
    }

    #[inline]
    fn into_parts(self) -> (Self::Payload, NodeDepth) {
        ((), NodeDepth::ROOT)
    }

    #[inline]
    fn from_parts(_payload: Self::Payload, _depth: NodeDepth) -> Self {
        ()
    }
}

impl DiGraphNodeMut for () {
    #[inline]
    fn set_depth(&mut self, _depth: NodeDepth) {
        // No-op for unit type
    }
}

/// Inheritance depth in a DAG (0-indexed for roots).
///
/// - Root nodes: `depth = 0`
/// - Child nodes: `depth = max(parent_depths) + 1`
///
/// # Examples
///
/// ```
/// use lithos_core::graph::NodeDepth;
///
/// let depth = NodeDepth::new(2);
/// assert_eq!(depth.as_usize(), 2);
/// assert_eq!(NodeDepth::ROOT.as_usize(), 0);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(bytecheck(bounds()))]
pub struct NodeDepth(usize);

impl NodeDepth {
    /// Root node depth (no parents).
    pub const ROOT: Self = Self(0);

    /// Creates a depth value from a raw count.
    #[inline]
    #[must_use]
    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }

    /// Returns the raw depth value.
    #[inline]
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Returns the next depth value, saturating on overflow.
    #[inline]
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod node_depth {
        use super::*;

        #[test]
        fn root_is_zero() {
            assert_eq!(
                NodeDepth::ROOT.as_usize(),
                0,
                "expected ROOT depth to be zero"
            );
        }

        #[test]
        fn increment_saturates_at_max() {
            let depth = NodeDepth::new(usize::MAX).increment();
            assert_eq!(
                depth.as_usize(),
                usize::MAX,
                "expected increment to saturate at usize::MAX"
            );
        }
    }

    mod node {
        use super::*;

        #[test]
        fn new_sets_root_depth() {
            let node = Node::new(Box::<str>::from("payload"));
            assert_eq!(
                node.depth(),
                NodeDepth::ROOT,
                "expected new node to start at root depth"
            );
        }

        #[test]
        fn payload_returns_reference() {
            let node = Node::new(Box::<str>::from("payload"));
            assert_eq!(
                &**node.payload(),
                "payload",
                "expected payload to match original value"
            );
        }
    }
}
