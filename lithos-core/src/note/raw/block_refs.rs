//! Raw block reference types.

use crate::note::position::SourceByteOffset;

/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawBlockRef {
    id: Box<str>,
    position: SourceByteOffset,
}

impl RawBlockRef {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub fn new(id: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            id,
            position,
        }
    }

    /// Return the raw block reference id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the source byte position for the block reference.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
