//! List value objects for notes.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]
#![expect(dead_code, reason = "Legacy list builders retained for future use")]

use std::fmt;

use super::{error::NoteError, position::SourceByteOffset, task::TaskId};
use crate::{
    config::task::StatusSymbol,
    note::raw::{RawListDepth, RawListItem},
};

/// Markdown list structure.
///
/// Represents a collection of [`ListItem`]s, which can be ordered or unordered.
/// A `List` tracks its nesting depth within the document.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{List, ListDepth, ListType};
/// let list = List::new(ListType::Unordered);
/// assert_eq!(list.depth(), ListDepth::root());
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct List {
    kind: ListType,
    items: Vec<ListItem>,
    depth: ListDepth,
}

impl List {
    /// Creates a new empty list with depth 0.
    #[inline]
    #[must_use]
    pub fn new(list_type: ListType) -> Self {
        Self {
            kind: list_type,
            items: Vec::new(),
            depth: ListDepth::root(),
        }
    }

    /// Creates a new empty list with an explicit depth.
    #[inline]
    #[must_use]
    pub fn with_depth(list_type: ListType, depth: ListDepth) -> Self {
        Self {
            kind: list_type,
            items: Vec::new(),
            depth,
        }
    }

    /// Creates a new empty list with an explicit depth and capacity hint.
    #[inline]
    #[must_use]
    pub fn with_capacity(
        list_type: ListType,
        depth: ListDepth,
        capacity: usize,
    ) -> Self {
        Self {
            kind: list_type,
            items: Vec::with_capacity(capacity),
            depth,
        }
    }

    /// Appends a list item, preserving source order.
    #[inline]
    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    /// Returns the list type.
    #[inline]
    #[must_use]
    pub const fn list_type(&self) -> ListType {
        self.kind
    }

    /// Returns the list items in source order.
    #[inline]
    #[must_use]
    pub fn items(&self) -> ListItems<'_> {
        ListItems {
            inner: self.items.iter(),
        }
    }

    /// Returns the list nesting depth (0 = top level).
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> ListDepth {
        self.depth
    }

    /// Sets the list nesting depth.
    #[inline]
    pub fn set_depth(&mut self, depth: ListDepth) {
        self.depth = depth;
    }
}

/// Validated list nesting depth.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct ListDepth(u8);

impl ListDepth {
    /// Returns the root list depth (0).
    #[inline]
    #[must_use]
    pub const fn root() -> Self {
        Self(0)
    }

    /// Creates a validated list depth from a numeric value.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::ListDepthOutOfRange`] if the depth is out of range.
    #[inline]
    pub fn try_new(depth: usize) -> Result<Self, NoteError> {
        u8::try_from(depth).map(Self).map_err(|_error| {
            NoteError::ListDepthOutOfRange {
                depth,
                reason: "depth exceeds maximum allowed value of 255",
            }
        })
    }

    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl fmt::Display for ListDepth {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Borrowed iterator over list items.
pub struct ListItems<'list> {
    inner: std::slice::Iter<'list, ListItem>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'list> Iterator for ListItems<'list> {
    type Item = &'list ListItem;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Single item in a markdown list.
///
/// Items can be plain text or checkbox items. Checkbox items may be
/// promoted to [`crate::note::task::Task`] entities while remaining part of
/// their parent list.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{list::ListItem, position::SourceByteOffset};
/// let item = ListItem::Plain {
///     text: "Buy groceries".into(),
///     position: SourceByteOffset::new(0),
/// };
/// assert_eq!(item.text(), "Buy groceries");
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum ListItem {
    /// Plain list item (no checkbox).
    Plain {
        /// Raw text content.
        text: Box<str>,
        /// Source byte offset in the note.
        position: SourceByteOffset,
    },
    /// Checkbox list item.
    Checkbox {
        /// Raw text content.
        text: Box<str>,
        /// Checkbox status symbol.
        status: StatusSymbol,
        /// Source byte offset in the note.
        position: SourceByteOffset,
        /// Task id if promoted to a Task.
        task_id: Option<TaskId>,
    },
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
    depth: ListDepth,
    text: InlineText,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ListItemBuilder {
    pub(crate) fn new(position: SourceByteOffset, depth: ListDepth) -> Self {
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

    pub(crate) const fn depth(&self) -> ListDepth {
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
}

/// List item metadata entry for indexing.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ListItemEntry {
    position: SourceByteOffset,
    depth: ListDepth,
    parent: Option<SourceByteOffset>,
    status: Option<StatusSymbol>,
    task_id: Option<TaskId>,
}

impl ListItemEntry {
    /// Creates a new list item entry.
    #[inline]
    #[must_use]
    pub fn new(
        position: SourceByteOffset,
        depth: ListDepth,
        parent: Option<SourceByteOffset>,
        status: Option<StatusSymbol>,
        task_id: Option<TaskId>,
    ) -> Self {
        Self {
            position,
            depth,
            parent,
            status,
            task_id,
        }
    }

    /// Returns the list item byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Returns the list item depth.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> ListDepth {
        self.depth
    }

    /// Returns the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }

    /// Returns the task status symbol, if this is a checkbox item.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> Option<StatusSymbol> {
        self.status
    }

    /// Returns the task id, if this is a promoted task item.
    #[inline]
    #[must_use]
    pub const fn task_id(&self) -> Option<TaskId> {
        self.task_id
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

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &self keep accessors concise."
)]
impl ListItem {
    /// Returns the source byte position of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn position(&self) -> SourceByteOffset {
        match self {
            Self::Plain {
                position,
                ..
            }
            | Self::Checkbox {
                position,
                ..
            } => *position,
        }
    }

    /// Returns the text content of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::Plain {
                text,
                ..
            }
            | Self::Checkbox {
                text,
                ..
            } => text.as_ref(),
        }
    }

    /// Returns the checkbox status symbol if this is a checkbox item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn status(&self) -> Option<StatusSymbol> {
        match self {
            Self::Checkbox {
                status,
                ..
            } => Some(*status),
            Self::Plain {
                ..
            } => None,
        }
    }

    /// Returns the task id if this checkbox was promoted.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn task_id(&self) -> Option<TaskId> {
        match self {
            Self::Checkbox {
                task_id,
                ..
            } => *task_id,
            Self::Plain {
                ..
            } => None,
        }
    }

    /// Sets the task id for a promoted checkbox item.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &mut self"
    )]
    pub fn set_task_id(&mut self, task_id: TaskId) {
        if let Self::Checkbox {
            task_id: slot,
            ..
        } = self
        {
            *slot = Some(task_id);
        }
    }

    /// Clears the task id for a checkbox item.
    #[inline]
    pub fn clear_task_id(&mut self) {
        if let Self::Checkbox {
            task_id: slot,
            ..
        } = self
        {
            *slot = None;
        }
    }
}

/// Markdown list type.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum ListType {
    /// Ordered list starting at the given number.
    Ordered {
        /// Starting index for the list (usually 1).
        start: u64,
    },
    /// Unordered list (bullets).
    Unordered,
}
