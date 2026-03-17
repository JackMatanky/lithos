//! Raw tag types.

use crate::note::position::SourceByteOffset;

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTag {
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawTag {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub fn new(value: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            value,
            position,
        }
    }

    /// Return the raw tag token value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the tag token.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
