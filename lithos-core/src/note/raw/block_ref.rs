use std::borrow::Cow;

use crate::note::position::SourceByteOffset;

/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawBlockRef<'source> {
    pub id: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawBlockRef<'source> {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            position,
        }
    }
}

impl RawBlockRef<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawBlockRef<'static> {
        RawBlockRef {
            id: Cow::Owned(self.id.into_owned()),
            position: self.position,
        }
    }
}
