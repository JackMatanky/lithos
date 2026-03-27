use std::borrow::Cow;

use crate::note::position::SourceByteRange;

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawTag<'source> {
    pub value: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawTag<'source> {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub const fn new(value: Cow<'source, str>, range: SourceByteRange) -> Self {
        Self {
            value,
            range,
        }
    }
}

impl RawTag<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTag<'static> {
        RawTag {
            value: Cow::Owned(self.value.into_owned()),
            range: self.range,
        }
    }
}
