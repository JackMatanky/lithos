//! Raw reference link definition helpers.

use crate::note::position::SourceByteOffset;

/// Raw reference-style link definition.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawReferenceLink {
    id: Box<str>,
    target: Box<str>,
    position: SourceByteOffset,
}

impl RawReferenceLink {
    /// Create a new raw reference link definition.
    #[inline]
    #[must_use]
    pub fn new(
        id: Box<str>,
        target: Box<str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }

    /// Return the definition id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
