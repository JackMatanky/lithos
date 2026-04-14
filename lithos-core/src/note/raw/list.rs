use std::borrow::Cow;

use super::{inline_field::RawInlineField, tag::RawTag};
use crate::note::position::{SourceByteOffset, SourceByteRange};

/// Raw list container extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawList {
    pub kind: RawListKind,
    pub depth: RawListDepth,
    pub range: SourceByteRange,
    pub item_positions: Vec<SourceByteOffset>,
}

impl RawList {
    /// Create a new raw list container.
    #[inline]
    #[must_use]
    pub fn new(
        kind: RawListKind,
        depth: RawListDepth,
        range: SourceByteRange,
        item_positions: Vec<SourceByteOffset>,
    ) -> Self {
        Self {
            kind,
            depth,
            range,
            item_positions,
        }
    }

    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawList {
        RawList {
            kind: self.kind,
            depth: self.depth,
            range: self.range,
            item_positions: self.item_positions,
        }
    }
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem<'source> {
    pub list_kind: RawListKind,
    pub depth: RawListDepth,
    pub text: Cow<'source, str>,
    pub is_checked: Option<bool>,
    pub task_marker: Option<RawTaskStatusSymbol>,
    pub range: SourceByteRange,
    pub text_range: SourceByteRange,
    pub parent: Option<SourceByteOffset>,
    pub tags: Vec<RawTag<'source>>,
    pub inline_fields: Vec<RawInlineField<'source>>,
}

impl<'source> RawListItem<'source> {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub fn new(
        list_kind: RawListKind,
        depth: RawListDepth,
        text: Cow<'source, str>,
        is_checked: Option<bool>,
        task_marker: Option<RawTaskStatusSymbol>,
        range: SourceByteRange,
        text_range: SourceByteRange,
        parent: Option<SourceByteOffset>,
        tags: Vec<RawTag<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
    ) -> Self {
        Self {
            list_kind,
            depth,
            text,
            is_checked,
            task_marker,
            range,
            text_range,
            parent,
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
            list_kind: self.list_kind,
            depth: self.depth,
            text: Cow::Owned(self.text.into_owned()),
            is_checked: self.is_checked,
            task_marker: self.task_marker,
            range: self.range,
            text_range: self.text_range,
            parent: self.parent,
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineField::into_owned)
                .collect(),
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

/// Raw task status symbol with source position.
///
/// This combines a [`RawTaskMarker`] classification with the exact source
/// position where the marker character appears in the markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawTaskStatusSymbol {
    /// The task marker classification.
    pub marker: RawTaskMarker,
    /// The absolute source position of the marker character.
    pub position: SourceByteOffset,
}

impl RawTaskStatusSymbol {
    /// Create a new raw task status symbol.
    #[inline]
    #[must_use]
    pub const fn new(
        marker: RawTaskMarker,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            marker,
            position,
        }
    }

    /// Returns the raw marker character.
    #[inline]
    #[must_use]
    pub const fn marker_char(self) -> char {
        self.marker.marker()
    }
}

/// Raw task marker kind extracted from a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTaskMarker {
    /// Unchecked task marker (typically `[ ]`).
    Unchecked(char),
    /// Checked task marker (typically `[x]`).
    Checked(char),
    /// Task marker with a non-standard symbol.
    Other(char),
}

impl RawTaskMarker {
    /// Create a raw task marker from a character.
    #[inline]
    #[must_use]
    pub fn from_char(marker: char) -> Self {
        match marker {
            ' ' => Self::Unchecked(marker),
            'x' | 'X' => Self::Checked(marker),
            _ => Self::Other(marker),
        }
    }

    /// Returns the raw marker character.
    #[inline]
    #[must_use]
    pub const fn marker(self) -> char {
        match self {
            Self::Unchecked(marker)
            | Self::Checked(marker)
            | Self::Other(marker) => marker,
        }
    }
}
