use std::borrow::Cow;

use crate::note::position::{SourceByteOffset, SourceByteRange};

/// Raw heading extracted from the AST.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawHeading<'source> {
    pub level: u8,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
    pub position: SourceByteOffset,
}

impl<'source> RawHeading<'source> {
    /// Create a new raw heading entry.
    #[inline]
    #[must_use]
    pub const fn new(
        level: u8,
        text: Cow<'source, str>,
        range: SourceByteRange,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            level,
            text,
            range,
            position,
        }
    }
}

impl RawHeading<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawHeading<'static> {
        RawHeading {
            level: self.level,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
            position: self.position,
        }
    }
}
