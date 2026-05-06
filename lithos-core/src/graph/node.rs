//! Node primitives for the graph module.
//!
//! Defines `Node` used by graph structures.
//!
//! # Examples
//! ```
//! use lithos_core::graph::{GraphNode, Node};
//!
//! let node = Node::new(Box::<str>::from("payload"));
//! assert_eq!(&**node.payload(), "payload");
//! ```

/// Provides immutable access to a graph node's payload.
pub trait GraphNode {
    /// The type of payload stored in this node.
    type Payload;

    /// Creates a node from a payload.
    fn from_payload(payload: Self::Payload) -> Self;

    /// Returns the payload stored in this node.
    #[must_use]
    fn payload(&self) -> &Self::Payload;
}

/// Provides mutable access to a graph node's payload.
pub trait GraphNodeMut: GraphNode {
    /// Returns a mutable reference to the payload.
    fn payload_mut(&mut self) -> &mut Self::Payload;
}

/// A node in the graph.
///
/// **Pure infrastructure** - NO serialization constraint. Domain wrappers
/// add Archive bounds when needed.
///
/// # Examples
///
/// ```
/// use lithos_core::graph::{GraphNode, Node};
///
/// let node = Node::new(Box::<str>::from("payload"));
/// assert_eq!(&**node.payload(), "payload");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Node<T> {
    /// Application-specific node data.
    pub payload: T,
}

impl<T> Node<T> {
    /// Creates a new node.
    #[inline]
    #[must_use]
    pub fn new(payload: T) -> Self {
        Self {
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

    #[inline]
    fn from_payload(payload: Self::Payload) -> Self {
        Self::new(payload)
    }
}

// GraphNodeMut trait implementation for Node<T>
impl<T> GraphNodeMut for Node<T> {
    #[inline]
    fn payload_mut(&mut self) -> &mut Self::Payload {
        &mut self.payload
    }
}

// Unit type implementations for trivial graph nodes
impl GraphNode for () {
    type Payload = ();

    #[inline]
    fn payload(&self) -> &Self::Payload {
        self
    }

    #[inline]
    fn from_payload(_payload: Self::Payload) -> Self {}
}

impl GraphNodeMut for () {
    #[inline]
    fn payload_mut(&mut self) -> &mut Self::Payload {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod node {
        use super::*;

        #[test]
        fn payload_returns_reference() {
            let node = Node::new(Box::<str>::from("payload"));
            assert_eq!(
                &**node.payload(),
                "payload",
                "expected payload to match original value"
            );
        }

        #[test]
        fn new_creates_node() {
            let node = Node::new(42u32);
            assert_eq!(node.payload, 42);
        }
    }
}
