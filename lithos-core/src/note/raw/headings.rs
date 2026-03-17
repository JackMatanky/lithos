//! Raw heading types.

use crate::note::position::{SourceByteOffset, SourceByteRange};

/// Raw heading extracted from the AST.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawHeading {
    level: u8,
    text: Box<str>,
    range: SourceByteRange,
    position: SourceByteOffset,
}

impl RawHeading {
    /// Create a new raw heading entry.
    #[inline]
    #[must_use]
    pub fn new(
        level: u8,
        text: Box<str>,
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

    /// Return the raw heading level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Return the raw heading text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the byte range for the heading.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Return the start byte offset for the heading.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
