//! Inline field value objects.

use super::raw::RawInlineField;
use crate::note::position::SourceByteOffset;

/// Inline field extracted from markdown.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct InlineField {
    key: Box<str>,
    value: Box<str>,
    position: SourceByteOffset,
}

impl InlineField {
    /// Create a new inline field entry.
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

    /// Return the normalized field key.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the raw field value string.
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

impl From<RawInlineField> for InlineField {
    #[inline]
    fn from(raw: RawInlineField) -> Self {
        InlineField::new(raw.key().into(), raw.value().into(), raw.position())
    }
}
