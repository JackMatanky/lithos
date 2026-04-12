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
//! assert_eq!(node.payload(), "payload");
//! ```

use std::hash::Hash;

use rkyv::{Archive, Deserialize, Serialize};

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
#[archive(check_bytes)]
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

/// A node in the graph.
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
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Node<T>
where
    T: Archive,
{
    /// Inheritance depth (0 for roots, max(parent_depths) + 1 for children).
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
}
