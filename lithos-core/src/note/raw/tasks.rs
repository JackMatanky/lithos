//! Raw task types.

use crate::note::{position::SourceByteOffset, raw::list_items::RawTaskKind};

type RawInlineField = (Box<str>, Box<str>);

/// Raw task extracted from a checkbox list item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTask {
    task_kind: RawTaskKind,
    text: Box<str>,
    tags: Vec<Box<str>>,
    inline_fields: Vec<RawInlineField>,
    emoji_dates: Vec<RawInlineField>,
    position: SourceByteOffset,
}

impl RawTask {
    /// Create a raw task entry.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw tasks capture full source metadata"
    )]
    pub fn new(
        task_kind: RawTaskKind,
        text: Box<str>,
        tags: Vec<Box<str>>,
        inline_fields: Vec<RawInlineField>,
        emoji_dates: Vec<RawInlineField>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            task_kind,
            text,
            tags,
            inline_fields,
            emoji_dates,
            position,
        }
    }

    /// Return the task marker kind.
    #[inline]
    #[must_use]
    pub const fn task_kind(&self) -> RawTaskKind {
        self.task_kind
    }

    /// Return the raw task text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return raw tag tokens found in the task text.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }

    /// Return raw inline fields parsed from the task text.
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[RawInlineField] {
        &self.inline_fields
    }

    /// Return raw emoji date entries parsed from the task text.
    #[inline]
    #[must_use]
    pub fn emoji_dates(&self) -> &[RawInlineField] {
        &self.emoji_dates
    }

    /// Return the source byte position for the task marker.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
