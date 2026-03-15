//! Raw heading extraction helpers.

#![expect(dead_code, reason = "Raw heading builders retained for legacy use")]

use crate::note::{
    error::NoteError,
    heading::{Heading, HeadingLevel},
    position::{SourceByteOffset, SourceByteRange},
};

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

/// Builder for accumulating heading data during parsing.
#[derive(Debug)]
pub(crate) struct HeadingBuilder {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}

impl HeadingBuilder {
    #[inline]
    pub(crate) fn new(level: HeadingLevel, position: SourceByteOffset) -> Self {
        Self {
            level,
            text: String::new(),
            position,
        }
    }

    #[inline]
    pub(crate) fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    #[inline]
    pub(crate) fn push_break(&mut self) {
        self.text.push(' ');
    }

    #[inline]
    pub(crate) fn build(self) -> Result<Heading, NoteError> {
        Heading::try_new(self.level, self.text, self.position)
    }

    #[inline]
    pub(crate) fn build_raw(self, range: SourceByteRange) -> RawHeading {
        RawHeading::new(
            self.level.as_u8(),
            self.text.into_boxed_str(),
            range,
            self.position,
        )
    }
}
