//! Raw frontmatter types.

use crate::note::frontmatter::FrontmatterFormat;

/// Raw frontmatter block captured from metadata events.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawFrontmatter {
    kind: FrontmatterFormat,
    text: Box<str>,
    range: crate::note::position::SourceByteRange,
}

impl RawFrontmatter {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub fn new(
        kind: FrontmatterFormat,
        text: Box<str>,
        range: crate::note::position::SourceByteRange,
    ) -> Self {
        Self {
            kind,
            text,
            range,
        }
    }

    /// Return the metadata block kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> FrontmatterFormat {
        self.kind
    }

    /// Return the raw frontmatter text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the source byte range for the frontmatter block.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> crate::note::position::SourceByteRange {
        self.range
    }
}
