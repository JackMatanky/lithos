use std::borrow::Cow;

use crate::note::position::SourceByteOffset;

/// Raw reference-style link definition.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawReferenceLink<'source> {
    pub id: Cow<'source, str>,
    pub target: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawReferenceLink<'source> {
    /// Create a new raw reference link definition.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        target: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }
}

impl RawReferenceLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawReferenceLink<'static> {
        RawReferenceLink {
            id: Cow::Owned(self.id.into_owned()),
            target: Cow::Owned(self.target.into_owned()),
            position: self.position,
        }
    }
}
