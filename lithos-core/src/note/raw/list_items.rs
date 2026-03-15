//! Raw list item extraction helpers.

use crate::{
    config::task::StatusSymbol,
    note::{
        error::NoteError,
        list::{ListDepth, ListItemEntry},
        position::SourceByteOffset,
    },
};

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
    is_checkbox: bool,
    status_symbol: Option<char>,
    position: SourceByteOffset,
    parent: Option<SourceByteOffset>,
}

impl RawListItem {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    pub fn new(
        list_type: RawListType,
        depth: RawListDepth,
        text: Box<str>,
        is_checkbox: bool,
        status_symbol: Option<char>,
        position: SourceByteOffset,
        parent: Option<SourceByteOffset>,
    ) -> Self {
        Self {
            list_type,
            depth,
            text,
            is_checkbox,
            status_symbol,
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

    /// Return true if this list item is a checkbox.
    #[inline]
    #[must_use]
    pub const fn is_checkbox(&self) -> bool {
        self.is_checkbox
    }

    /// Return the raw status symbol, if present.
    #[inline]
    #[must_use]
    pub const fn status_symbol(&self) -> Option<char> {
        self.status_symbol
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

    fn try_from(raw: RawListItem) -> Result<Self, Self::Error> {
        let depth = match raw.depth() {
            RawListDepth::Root => ListDepth::root(),
            RawListDepth::Nested(value) => {
                ListDepth::try_new(usize::from(value))?
            }
        };
        let status = if raw.is_checkbox() {
            raw.status_symbol().map(StatusSymbol::try_new).transpose()?
        } else {
            None
        };

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
    is_checkbox: bool,
    status_symbol: Option<char>,
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
            is_checkbox: false,
            status_symbol: None,
        }
    }

    pub(crate) fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked {
            'x'
        } else {
            ' '
        });
    }

    pub(crate) const fn position(&self) -> SourceByteOffset {
        self.position
    }

    pub(crate) const fn depth(&self) -> crate::note::list::ListDepth {
        self.depth
    }

    pub(crate) const fn is_checkbox(&self) -> bool {
        self.is_checkbox
    }

    pub(crate) const fn status_symbol(&self) -> Option<char> {
        self.status_symbol
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
            self.is_checkbox,
            self.status_symbol,
            self.position,
            parent,
        )
    }
}
