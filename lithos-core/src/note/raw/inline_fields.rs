//! Raw inline field types.

use crate::note::position::SourceByteOffset;

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField {
    key: Box<str>,
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawInlineField {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: Box<str>,
        value: Box<str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            key,
            value,
            position,
        }
    }

    /// Return the normalized key.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the raw value string.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the field key.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
