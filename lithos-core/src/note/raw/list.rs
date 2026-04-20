use std::borrow::Cow;

use super::{inline_field::RawInlineFieldToken, tag::RawTag};
use crate::note::position::{SourceByteOffset, SourceByteRange};

/// Raw text content and its source range for a list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawListItemText<'source> {
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawListItemText<'source> {
    /// Create a new raw list item text container.
    #[inline]
    #[must_use]
    pub const fn new(text: Cow<'source, str>, range: SourceByteRange) -> Self {
        Self {
            text,
            range,
        }
    }
}

/// Raw list type extracted from markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListKind {
    Ordered(u64),
    Unordered,
}

/// Raw list nesting depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListDepth {
    Root,
    Nested(u8),
}

impl RawListDepth {
    /// Converts this depth to its u32 representation.
    ///
    /// [`Root`](RawListDepth::Root) maps to `0`;
    /// [`Nested(n)`](RawListDepth::Nested) maps to `n` as a `u32`.
    #[inline]
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        match self {
            Self::Root => 0,
            #[expect(
                clippy::as_conversions,
                reason = "u8 → u32 is always lossless; u32::from(u8) is not \
                          const-stable"
            )]
            Self::Nested(n) => n as u32,
        }
    }
}

impl From<u32> for RawListDepth {
    /// Converts a u32 nesting depth to [`RawListDepth`].
    ///
    /// `0` maps to [`Root`](RawListDepth::Root); values `1..=255` map to
    /// [`Nested(n)`](RawListDepth::Nested). Values greater than `255` saturate
    /// to [`Nested(u8::MAX)`](RawListDepth::Nested).
    #[inline]
    fn from(depth: u32) -> Self {
        match depth {
            0 => Self::Root,
            1..=255 => Self::Nested(
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::as_conversions,
                    reason = "range guard ensures depth fits in u8"
                )]
                (depth as u8),
            ),
            _ => Self::Nested(u8::MAX),
        }
    }
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem<'source> {
    pub kind: RawListKind,
    pub depth: RawListDepth,
    pub parent: Option<SourceByteOffset>,
    pub is_checkbox: Option<bool>,
    pub text: RawListItemText<'source>,
    pub range: SourceByteRange,
    pub tags: Vec<RawTag<'source>>,
    pub inline_fields: Vec<RawInlineFieldToken<'source>>,
}

impl<'source> RawListItem<'source> {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full structural metadata"
    )]
    pub fn new(
        kind: RawListKind,
        depth: RawListDepth,
        parent: Option<SourceByteOffset>,
        is_checkbox: Option<bool>,
        text: RawListItemText<'source>,
        range: SourceByteRange,
        tags: Vec<RawTag<'source>>,
        inline_fields: Vec<RawInlineFieldToken<'source>>,
    ) -> Self {
        Self {
            kind,
            depth,
            parent,
            is_checkbox,
            text,
            range,
            tags,
            inline_fields,
        }
    }
}

impl RawListItem<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawListItem<'static> {
        RawListItem {
            kind: self.kind,
            depth: self.depth,
            parent: self.parent,
            is_checkbox: self.is_checkbox,
            text: RawListItemText {
                text: Cow::Owned(self.text.text.into_owned()),
                range: self.text.range,
            },
            range: self.range,
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineFieldToken::into_owned)
                .collect(),
        }
    }
}
