use std::borrow::Cow;

use super::value::RawFieldValue;
use crate::note::position::SourceByteRange;

/// Raw inline field token extracted from markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawInlineFieldToken<'source> {
    pub key: Cow<'source, str>,
    pub value: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawInlineFieldToken<'source> {
    /// Create a raw inline field token.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: Cow<'source, str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
        }
    }
}

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: RawFieldValue<'source>,
    pub range: SourceByteRange,
}

impl<'source> RawInlineField<'source> {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: RawFieldValue<'source>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
        }
    }
}

impl RawInlineField<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineField<'static> {
        RawInlineField {
            key: Cow::Owned(self.key.into_owned()),
            value: self.value.into_owned(),
            range: self.range,
        }
    }
}

impl RawInlineFieldToken<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineFieldToken<'static> {
        RawInlineFieldToken {
            key: Cow::Owned(self.key.into_owned()),
            value: Cow::Owned(self.value.into_owned()),
            range: self.range,
        }
    }
}
