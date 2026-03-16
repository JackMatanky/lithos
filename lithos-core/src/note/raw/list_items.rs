//! Raw list item extraction helpers.

#![expect(dead_code, reason = "Raw list item builders retained for legacy use")]

use crate::{
    config::task::StatusSymbol,
    note::{
        error::NoteError,
        list::{ListDepth, ListItemEntry},
        position::SourceByteOffset,
    },
};

/// Raw task marker kind extracted from a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTaskKind {
    /// Unchecked task marker (typically `[ ]`).
    Unchecked(char),
    /// Checked task marker (typically `[x]`).
    Checked(char),
    /// Task marker with a non-standard symbol.
    Other(char),
}

impl RawTaskKind {
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

/// Raw list type extracted from markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListType {
    Ordered {
        start: u64,
    },
    Unordered,
}

/// Raw list nesting depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListDepth {
    Root,
    Nested(u8),
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem {
    list_type: RawListType,
    depth: RawListDepth,
    text: Box<str>,
    task_kind: Option<RawTaskKind>,
    position: SourceByteOffset,
    parent: Option<SourceByteOffset>,
}

impl RawListItem {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub fn new(
        list_type: RawListType,
        depth: RawListDepth,
        text: Box<str>,
        task_kind: Option<RawTaskKind>,
        position: SourceByteOffset,
        parent: Option<SourceByteOffset>,
    ) -> Self {
        Self {
            list_type,
            depth,
            text,
            task_kind,
            position,
            parent,
        }
    }

    /// Return the list type.
    #[inline]
    #[must_use]
    pub const fn list_type(&self) -> RawListType {
        self.list_type
    }

    /// Return the list nesting depth.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> RawListDepth {
        self.depth
    }

    /// Return the raw list item text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the raw task marker kind, if present.
    #[inline]
    #[must_use]
    pub const fn task_kind(&self) -> Option<RawTaskKind> {
        self.task_kind
    }

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Return the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }
}

impl RawListDepth {
    /// Convert a domain list depth into a raw depth value.
    #[inline]
    #[must_use]
    pub fn from_list_depth(depth: crate::note::list::ListDepth) -> Self {
        let value = depth.as_u8();
        if value == 0 {
            Self::Root
        } else {
            Self::Nested(value)
        }
    }
}

impl TryFrom<RawListItem> for ListItemEntry {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawListItem) -> Result<Self, Self::Error> {
        let depth = match raw.depth() {
            RawListDepth::Root => ListDepth::root(),
            RawListDepth::Nested(value) => {
                ListDepth::try_new(usize::from(value))?
            }
        };
        let status = raw
            .task_kind()
            .map(|kind| StatusSymbol::try_new(kind.marker()))
            .transpose()?;

        Ok(ListItemEntry::new(
            raw.position(),
            depth,
            raw.parent(),
            status,
            None,
        ))
    }
}

#[derive(Debug, Default)]
pub(crate) struct InlineText {
    buffer: String,
}

impl InlineText {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push_text(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    pub(crate) fn push_break(&mut self) {
        if !self.buffer.ends_with(' ') {
            self.buffer.push(' ');
        }
    }

    pub(crate) fn finish(self) -> String {
        self.buffer
    }
}

#[derive(Debug)]
pub(crate) struct ListItemBuilder {
    position: SourceByteOffset,
    depth: crate::note::list::ListDepth,
    text: InlineText,
    task_kind: Option<RawTaskKind>,
}

impl ListItemBuilder {
    pub(crate) fn new(
        position: SourceByteOffset,
        depth: crate::note::list::ListDepth,
    ) -> Self {
        Self {
            position,
            depth,
            text: InlineText::new(),
            task_kind: None,
        }
    }

    pub(crate) fn mark_as_checkbox(&mut self, checked: bool) {
        self.task_kind = Some(if checked {
            RawTaskKind::Checked('x')
        } else {
            RawTaskKind::Unchecked(' ')
        });
    }

    pub(crate) const fn position(&self) -> SourceByteOffset {
        self.position
    }

    pub(crate) const fn depth(&self) -> crate::note::list::ListDepth {
        self.depth
    }

    pub(crate) const fn task_kind(&self) -> Option<RawTaskKind> {
        self.task_kind
    }

    pub(crate) fn add_text(&mut self, text: &str) {
        self.text.push_text(text);
    }

    pub(crate) fn add_break(&mut self) {
        self.text.push_break();
    }

    pub(crate) fn into_text(self) -> String {
        self.text.finish()
    }

    #[inline]
    pub(crate) fn into_raw(
        self,
        list_type: RawListType,
        parent: Option<SourceByteOffset>,
    ) -> RawListItem {
        RawListItem::new(
            list_type,
            RawListDepth::from_list_depth(self.depth),
            self.text.finish().into_boxed_str(),
            self.task_kind,
            self.position,
            parent,
        )
    }
}
